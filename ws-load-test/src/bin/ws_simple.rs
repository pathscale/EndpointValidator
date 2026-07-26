//! Single-endpoint scenario - optimized for maximum throughput.
//!
//! Run with --help for flag reference.

use std::sync::Arc;
use std::sync::OnceLock;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};

use endpoint_libs::libs::ws::{WsClient, WsResponseGeneric, WsResponseValue};
use eyre::{Context, Result};
use hdrhistogram::Histogram;
use rustls::crypto::ring;
use serde_json::Value;
use tokio::io::AsyncWriteExt;
use tokio::sync::Mutex;

#[derive(Clone, Copy)]
enum ScenarioType {
    /// Repeatedly open and close a connection; no messages sent.
    Connection,
    /// Open one connection per worker and send messages until the quota is met.
    Message,
}

#[derive(Clone, Copy)]
struct Scenario {
    ty: ScenarioType,
    num_parallel: usize,
    num_requests: usize,
    timeout_ms: u64,
    store_raw_responses: bool,
    reconnect_attempts: usize,
}

struct WorkerResult {
    sent: u64,
    errors: u64,
    latency_histogram: Histogram<u64>,
    raw_responses: Vec<serde_json::Value>,
}

// Atomic counter for round-robin CPU core assignment
static CORE_INDEX: AtomicUsize = AtomicUsize::new(0);

// Cached stderr handle for efficient error logging
static STDERR_HANDLE: OnceLock<Arc<Mutex<tokio::io::Stderr>>> = OnceLock::new();

// Limit raw responses to prevent OOM
const MAX_RAW_RESPONSES: usize = 1000;

// ─────────────────────────────────────────────────────────────────────────────

fn main() -> eyre::Result<()> {
    // ── Defaults (override via CLI flags) ────────────────────────────────────────
    const DEFAULT_SERVER_URL: &str = "ws://127.0.0.1:3006";
    const DEFAULT_NUM_REQUESTS: usize = 10;
    const DEFAULT_NUM_PARALLEL: usize = 48;
    const DEFAULT_TYPE: ScenarioType = ScenarioType::Message;
    const DEFAULT_WORKER_THREADS: usize = 0; // 0 = auto (num_parallel)
    const DEFAULT_TIMEOUT_MS: u64 = 30000; // 30 seconds
    const DEFAULT_RECONNECT_ATTEMPTS: usize = 1; // 1 = no retries, just initial attempt

    const PROTOCOL_HEADER: &str = "some-protocol";
    const HEADERS: &[(&str, &str)] = &[];
    const METHOD_CODE: u32 = 1;

    /// Fallback payload — used when --params-file is not provided.
    const DEFAULT_PARAMS_JSON: &str = r#"{"message": "hello"}"#;

    // ─────────────────────────────────────────────────────────────────────────────

    ring::default_provider()
        .install_default()
        .expect("Could not install default crypto provider");

    let args: Vec<String> = std::env::args().skip(1).collect();

    if args.iter().any(|a| a == "--help" || a == "-h") {
        println!("Usage: ws-simple [OPTIONS]");
        println!();
        println!("Options:");
        println!(
            "  --server-url              <url>   WebSocket server (default: {DEFAULT_SERVER_URL})"
        );
        println!(
            "  --num-requests            <n>     Ops per worker (default: {DEFAULT_NUM_REQUESTS})"
        );
        println!(
            "  --num-parallel            <n>     Worker count (default: {DEFAULT_NUM_PARALLEL})"
        );
        println!(
            "  --worker-threads          <n>     Tokio worker threads, 0=auto (default: {DEFAULT_WORKER_THREADS})"
        );
        println!(
            "  --timeout-ms              <ms>    Network operation timeout (default: {DEFAULT_TIMEOUT_MS})"
        );
        println!(
            "  --reconnect-attempts      <n>     Retry connection N times on failure (default: {DEFAULT_RECONNECT_ATTEMPTS})"
        );
        println!(
            "  --type                    <type>  Scenario: connection|message (default: message)"
        );
        println!("  --params-file             <path>  JSON payload file (default: hardcoded)");
        println!("  --protocol-header-file    <path>  Protocol header file (default: hardcoded)");
        println!(
            "  --store-raw-responses             Store raw responses (limited to {MAX_RAW_RESPONSES} to prevent OOM)"
        );
        println!("  -o, --output              <dir>   Write results JSON to this directory");
        println!("  -h, --help                        Print this help");
        return Ok(());
    }

    let server_url: &'static str = parse_flag(&args, "--server-url")
        .map(|s| Box::leak(s.into_boxed_str()) as &'static str)
        .unwrap_or(DEFAULT_SERVER_URL);

    let num_requests: usize = parse_flag(&args, "--num-requests")
        .and_then(|s| s.parse().ok())
        .unwrap_or(DEFAULT_NUM_REQUESTS);

    let num_parallel: usize = parse_flag(&args, "--num-parallel")
        .and_then(|s| s.parse().ok())
        .unwrap_or(DEFAULT_NUM_PARALLEL);

    let worker_threads: usize = parse_flag(&args, "--worker-threads")
        .and_then(|s| s.parse().ok())
        .unwrap_or(DEFAULT_WORKER_THREADS);

    let timeout_ms: u64 = parse_flag(&args, "--timeout-ms")
        .and_then(|s| s.parse().ok())
        .unwrap_or(DEFAULT_TIMEOUT_MS);

    let store_raw_responses = args.iter().any(|a| a == "--store-raw-responses");

    let reconnect_attempts: usize = parse_flag(&args, "--reconnect-attempts")
        .and_then(|s| s.parse().ok())
        .unwrap_or(DEFAULT_RECONNECT_ATTEMPTS);

    let output_dir = parse_flag(&args, "--output").or_else(|| parse_flag(&args, "-o"));

    let scenario_type: ScenarioType = match parse_flag(&args, "--type").as_deref() {
        Some("connection") => ScenarioType::Connection,
        Some("message") => ScenarioType::Message,
        Some(other) => {
            eprintln!("Unknown --type '{other}'. Valid values: connection, message");
            std::process::exit(1);
        }
        None => DEFAULT_TYPE,
    };

    let protocol_header: &'static str = match parse_flag(&args, "--protocol-header-file") {
        Some(path) => {
            let raw = std::fs::read_to_string(&path).unwrap_or_else(|e| {
                eprintln!("Cannot read protocol header file '{path}': {e}");
                std::process::exit(1);
            });
            Box::leak(raw.trim().to_string().into_boxed_str())
        }
        None => PROTOCOL_HEADER,
    };

    // Pre-serialize the full request envelope once at startup (method, seq, params)
    // Using fixed seq=1 since recv_raw doesn't validate seq
    let request_bytes: Arc<Vec<u8>> = match parse_flag(&args, "--params-file") {
        Some(path) => {
            let raw = std::fs::read_to_string(&path).unwrap_or_else(|e| {
                eprintln!("Cannot read params file '{path}': {e}");
                std::process::exit(1);
            });
            // Validate JSON first
            serde_json::from_str::<Value>(&raw).unwrap_or_else(|e| {
                eprintln!("Invalid JSON in '{path}': {e}");
                std::process::exit(1);
            });
            let full_req = format!(r#"{{"method":{},"seq":1,"params":{}}}"#, METHOD_CODE, raw);
            Arc::new(full_req.into_bytes())
        }
        None => {
            let full_req = format!(
                r#"{{"method":{},"seq":1,"params":{}}}"#,
                METHOD_CODE, DEFAULT_PARAMS_JSON
            );
            Arc::new(full_req.into_bytes())
        }
    };

    let scenario = Scenario {
        ty: scenario_type,
        num_parallel,
        num_requests,
        timeout_ms,
        store_raw_responses,
        reconnect_attempts,
    };

    // Determine worker thread count
    let actual_worker_threads = if worker_threads > 0 {
        worker_threads
    } else {
        num_parallel.max(num_cpus::get())
    };

    // Build runtime with explicit configuration and CPU pinning
    let core_ids = core_affinity::get_core_ids().unwrap_or_default();
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(actual_worker_threads)
        .global_queue_interval(32)
        .on_thread_start(move || {
            // Pin each thread to a CPU core (round-robin)
            if !core_ids.is_empty() {
                let idx = CORE_INDEX.fetch_add(1, Ordering::Relaxed) % core_ids.len();
                let _ = core_affinity::set_for_current(core_ids[idx]);
            }
        })
        .enable_all()
        .build()
        .context("Failed to build Tokio runtime")?;

    runtime.block_on(async_main(
        &args,
        scenario,
        server_url,
        protocol_header,
        HEADERS,
        METHOD_CODE,
        request_bytes,
        output_dir.as_deref(),
    ))
}

async fn async_main(
    args: &[String],
    scenario: Scenario,
    server_url: &'static str,
    protocol_header: &'static str,
    headers: &'static [(&'static str, &'static str)],
    method_code: u32,
    request_bytes: Arc<Vec<u8>>,
    output_dir: Option<&str>,
) -> Result<()> {
    let start = Instant::now();

    // Use JoinSet for efficient task management
    let mut join_set = tokio::task::JoinSet::new();
    let total_workers = scenario.num_parallel;

    eprintln!("Starting {} workers...", total_workers);

    for id in 0..scenario.num_parallel {
        let request = Arc::clone(&request_bytes);
        join_set.spawn(async move {
            run_worker(id, scenario, server_url, protocol_header, headers, request).await
        });
    }

    // Aggregate results with progress tracking
    let mut total_sent: u64 = 0;
    let mut total_errors: u64 = 0;
    let mut completed_workers: usize = 0;
    let mut combined_histogram = Histogram::<u64>::new(3).expect("Failed to create histogram");
    let mut all_raw_responses: Vec<serde_json::Value> = Vec::new();

    eprintln!("Waiting for workers to complete...");

    while let Some(result) = join_set.join_next().await {
        completed_workers += 1;
        let progress = (completed_workers * 100) / total_workers;

        match result {
            Ok(Ok(worker_result)) => {
                total_sent += worker_result.sent;
                total_errors += worker_result.errors;
                // Efficiently merge histograms using add() instead of iterating buckets
                let _ = combined_histogram.add(&worker_result.latency_histogram);
                // Aggregate raw responses if collecting
                all_raw_responses.extend(worker_result.raw_responses);
            }
            Ok(Err(e)) => {
                let _ = async_stderr_write(&format!("Worker failed: {e}\n")).await;
            }
            Err(e) => {
                let _ = async_stderr_write(&format!("Worker panicked: {e}\n")).await;
            }
        }

        // Print progress every 10% or on last worker
        if progress % 10 == 0 || completed_workers == total_workers {
            eprintln!(
                "Progress: {completed_workers}/{total_workers} workers complete ({progress}%)"
            );
        }
    }

    eprintln!("All workers complete. Aggregating results...");

    let elapsed = start.elapsed();
    eprintln!("Generating summary...");
    print_summary(total_sent, total_errors, &combined_histogram, elapsed);

    if let Some(dir) = output_dir {
        eprintln!("Writing results to {dir}...");
        write_results(
            dir,
            args,
            server_url,
            protocol_header,
            method_code,
            scenario,
            total_sent,
            total_errors,
            &combined_histogram,
            elapsed,
            all_raw_responses,
        )?;
    }

    Ok(())
}

async fn run_worker(
    id: usize,
    scenario: Scenario,
    server_url: &'static str,
    protocol_header: &'static str,
    headers: &'static [(&'static str, &'static str)],
    request_bytes: Arc<Vec<u8>>,
) -> Result<WorkerResult> {
    // Pre-allocate histogram for latency recording
    let mut latency_histogram: Histogram<u64> =
        Histogram::new(3).wrap_err("Failed to create latency histogram")?;
    let mut errors: u64 = 0;
    let mut sent: u64 = 0;
    let mut logged_first_error = false;

    // Expected responses: Connection scenario has 0, Message scenario has up to num_requests
    // Cap at MAX_RAW_RESPONSES to prevent OOM
    let expected_responses = (scenario
        .num_requests
        .checked_mul(scenario.num_parallel)
        .unwrap_or(MAX_RAW_RESPONSES))
    .min(MAX_RAW_RESPONSES);

    let mut raw_responses: Vec<serde_json::Value> = Vec::with_capacity(expected_responses);

    match scenario.ty {
        ScenarioType::Connection => {
            for i in 0..scenario.num_requests {
                let t0 = Instant::now();
                sent += 1;

                match tokio::time::timeout(
                    Duration::from_millis(scenario.timeout_ms),
                    connect(server_url, protocol_header, headers),
                )
                .await
                {
                    Ok(Ok((client, status_code, initial_payload))) => {
                        drop(client);
                        let elapsed_us = t0.elapsed().as_micros() as u64;
                        latency_histogram.record(elapsed_us).ok();
                        if scenario.store_raw_responses && raw_responses.len() < MAX_RAW_RESPONSES {
                            raw_responses.push(serde_json::json!({
                                "type": "handshake",
                                "status": status_code,
                                "payload": initial_payload,
                            }));
                        }
                    }
                    Ok(Err(e)) => {
                        errors += 1;
                        if !logged_first_error {
                            let _ =
                                async_stderr_write(&format!("[worker {id}] connect error: {e}\n"))
                                    .await;
                            let _ = async_stderr_write(&format!("[worker {id}] further errors will not be logged (total errors: {})\n", scenario.num_requests - i - 1)).await;
                            logged_first_error = true;
                        }
                    }
                    Err(_) => {
                        errors += 1;
                        if !logged_first_error {
                            let _ = async_stderr_write(&format!(
                                "[worker {id}] connect error: timed out\n"
                            ))
                            .await;
                            let _ = async_stderr_write(&format!("[worker {id}] further errors will not be logged (total errors: {})\n", scenario.num_requests - i - 1)).await;
                            logged_first_error = true;
                        }
                    }
                }
            }
        }

        ScenarioType::Message => {
            // Try to establish initial connection with retries
            let mut client = None;

            for attempt in 0..scenario.reconnect_attempts {
                match tokio::time::timeout(
                    Duration::from_millis(scenario.timeout_ms),
                    connect(server_url, protocol_header, headers),
                )
                .await
                {
                    Ok(Ok((c, status_code, initial_payload))) => {
                        if scenario.store_raw_responses && raw_responses.len() < MAX_RAW_RESPONSES {
                            raw_responses.push(serde_json::json!({
                                "type": "handshake",
                                "status": status_code,
                                "payload": initial_payload,
                            }));
                        }
                        client = Some(c);
                        break;
                    }
                    Ok(Err(e)) => {
                        if attempt < scenario.reconnect_attempts - 1 {
                            if !logged_first_error {
                                let _ = async_stderr_write(&format!(
                                    "[worker {id}] connect error (attempt {}/{}): {e}\n",
                                    attempt + 1,
                                    scenario.reconnect_attempts
                                ))
                                .await;
                            }
                        } else {
                            if !logged_first_error {
                                let _ = async_stderr_write(&format!(
                                    "[worker {id}] connect error (final attempt): {e}\n"
                                ))
                                .await;
                                logged_first_error = true;
                            }
                        }
                    }
                    Err(_) => {
                        if attempt < scenario.reconnect_attempts - 1 {
                            if !logged_first_error {
                                let _ = async_stderr_write(&format!(
                                    "[worker {id}] connect error (attempt {}/{}): timed out\n",
                                    attempt + 1,
                                    scenario.reconnect_attempts
                                ))
                                .await;
                            }
                        } else {
                            if !logged_first_error {
                                let _ = async_stderr_write(&format!(
                                    "[worker {id}] connect error (final attempt): timed out\n"
                                ))
                                .await;
                                logged_first_error = true;
                            }
                        }
                    }
                }
            }

            // If all connection attempts failed, count all requests as errors
            // TODO: Classify this separately
            if client.is_none() {
                errors += scenario.num_requests as u64;
                sent += scenario.num_requests as u64;
                return Ok(WorkerResult {
                    sent,
                    errors,
                    latency_histogram,
                    raw_responses,
                });
            }

            let mut client = client.unwrap();

            for i in 0..scenario.num_requests {
                sent += 1;
                let mut iteration_failed = false;

                // Send attempt
                match tokio::time::timeout(
                    Duration::from_millis(scenario.timeout_ms),
                    client.send_raw(&request_bytes),
                )
                .await
                {
                    Ok(Ok(())) => {}
                    Ok(Err(e)) => {
                        errors += 1;
                        iteration_failed = true;
                        if !logged_first_error {
                            let _ = async_stderr_write(&format!("[worker {id}] send error: {e}\n"))
                                .await;
                            let _ = async_stderr_write(&format!("[worker {id}] further errors will not be logged (total errors: {})\n", scenario.num_requests - i - 1)).await;
                            logged_first_error = true;
                        }
                        break;
                    }
                    Err(_) => {
                        errors += 1;
                        iteration_failed = true;
                        if !logged_first_error {
                            let _ = async_stderr_write(&format!(
                                "[worker {id}] send error: timed out\n"
                            ))
                            .await;
                            let _ = async_stderr_write(&format!("[worker {id}] further errors will not be logged (total errors: {})\n", scenario.num_requests - i - 1)).await;
                            logged_first_error = true;
                        }
                        break;
                    }
                }

                // Only attempt recv if send succeeded
                if !iteration_failed {
                    let t0 = Instant::now();
                    match tokio::time::timeout(
                        Duration::from_millis(scenario.timeout_ms),
                        client.recv_resp::<WsResponseValue>(),
                    )
                    .await
                    {
                        Ok(Ok(WsResponseGeneric::Immediate(data))) => {
                            let elapsed_us = t0.elapsed().as_micros() as u64;
                            latency_histogram.record(elapsed_us).ok();
                            if scenario.store_raw_responses
                                && raw_responses.len() < MAX_RAW_RESPONSES
                            {
                                raw_responses
                                    .push(serde_json::json!({"type": "immediate", "data": data}));
                            }
                        }
                        Ok(Ok(WsResponseGeneric::Stream(data))) => {
                            let elapsed_us = t0.elapsed().as_micros() as u64;
                            latency_histogram.record(elapsed_us).ok();
                            if scenario.store_raw_responses
                                && raw_responses.len() < MAX_RAW_RESPONSES
                            {
                                raw_responses
                                    .push(serde_json::json!({"type": "stream", "data": data}));
                            }
                        }
                        Ok(Ok(WsResponseGeneric::Error(e))) => {
                            errors += 1;
                            if !logged_first_error {
                                let _ = async_stderr_write(&format!(
                                    "[worker {id}] server error: code={} log={}\n",
                                    e.code, e.log_id
                                ))
                                .await;
                                let _ = async_stderr_write(&format!(
                                    "[worker {id}] further errors will not be logged\n"
                                ))
                                .await;
                                logged_first_error = true;
                            }
                        }
                        Ok(Ok(WsResponseGeneric::Close)) => {
                            if !logged_first_error {
                                let _ = async_stderr_write(&format!(
                                    "[worker {id}] server closed connection\n"
                                ))
                                .await;
                            }
                            break;
                        }
                        Ok(Ok(_)) => {}
                        Ok(Err(e)) => {
                            errors += 1;
                            if !logged_first_error {
                                let _ =
                                    async_stderr_write(&format!("[worker {id}] recv error: {e}\n"))
                                        .await;
                                let _ = async_stderr_write(&format!(
                                    "[worker {id}] further errors will not be logged\n"
                                ))
                                .await;
                                logged_first_error = true;
                            }
                            break;
                        }
                        Err(_) => {
                            errors += 1;
                            if !logged_first_error {
                                let _ = async_stderr_write(&format!(
                                    "[worker {id}] recv error: timed out\n"
                                ))
                                .await;
                                let _ = async_stderr_write(&format!(
                                    "[worker {id}] further errors will not be logged\n"
                                ))
                                .await;
                                logged_first_error = true;
                            }
                            break;
                        }
                    }
                }
            }
        }
    }

    Ok(WorkerResult {
        sent,
        errors,
        latency_histogram,
        raw_responses,
    })
}

/// Async write to stderr to avoid blocking on I/O
async fn async_stderr_write(msg: &str) -> std::io::Result<()> {
    let handle = STDERR_HANDLE.get_or_init(|| Arc::new(Mutex::new(tokio::io::stderr())));
    let mut stderr = handle.lock().await;
    stderr.write_all(msg.as_bytes()).await
}

async fn connect(
    server_url: &'static str,
    protocol_header: &'static str,
    headers: &[(&'static str, &'static str)],
) -> Result<(WsClient, u16, Option<serde_json::Value>)> {
    let headers = if headers.is_empty() {
        None
    } else {
        Some(headers.to_vec())
    };
    let (mut client, response) = WsClient::new(server_url, protocol_header, headers).await?;
    let status_code = response.status().as_u16();

    // Try to receive initial payload from endpoint
    let initial_payload = match tokio::time::timeout(
        Duration::from_millis(500),
        client.recv_resp::<WsResponseValue>(),
    )
    .await
    {
        Ok(Ok(msg)) => Some(serde_json::to_value(&msg)?),
        _ => None,
    };

    Ok((client, status_code, initial_payload))
}

fn parse_flag(args: &[String], flag: &str) -> Option<String> {
    args.windows(2).find(|w| w[0] == flag).map(|w| w[1].clone())
}

fn print_summary(sent: u64, errors: u64, histogram: &Histogram<u64>, elapsed: Duration) {
    let ok = sent.saturating_sub(errors);
    let rps = ok as f64 / elapsed.as_secs_f64();
    let err_pct = if sent > 0 {
        errors as f64 / sent as f64 * 100.0
    } else {
        0.0
    };

    println!("┌── Scenario ─────────────────────────────────");
    println!("│  Elapsed:   {:.3}s", elapsed.as_secs_f64());
    println!("│  Sent:      {sent}");
    println!("│  OK:        {ok}");
    println!("│  Errors:    {errors}  ({err_pct:.1}%)");
    println!("│  RPS:       {rps:.1}");

    if histogram.len() > 0 {
        println!("│  Latency (RTT ms):");
        println!("│    mean  {:>8.2}", histogram.mean() as f64 / 1000.0);
        println!(
            "│    p50   {:>8.2}",
            histogram.value_at_percentile(50.0) as f64 / 1000.0
        );
        println!(
            "│    p95   {:>8.2}",
            histogram.value_at_percentile(95.0) as f64 / 1000.0
        );
        println!(
            "│    p99   {:>8.2}",
            histogram.value_at_percentile(99.0) as f64 / 1000.0
        );
        println!("│    min   {:>8.2}", histogram.min() as f64 / 1000.0);
        println!("│    max   {:>8.2}", histogram.max() as f64 / 1000.0);
    }
    println!("└─────────────────────────────────────────────");
}

fn write_results(
    dir: &str,
    args: &[String],
    server_url: &str,
    protocol_header: &str,
    method_code: u32,
    scenario: Scenario,
    sent: u64,
    errors: u64,
    histogram: &Histogram<u64>,
    elapsed: Duration,
    raw_responses: Vec<serde_json::Value>,
) -> Result<()> {
    use serde_json::json;
    use std::fs;
    use std::time::SystemTime;

    let ok = sent.saturating_sub(errors);
    let rps = ok as f64 / elapsed.as_secs_f64();
    let err_pct = if sent > 0 {
        errors as f64 / sent as f64 * 100.0
    } else {
        0.0
    };

    let latency = if histogram.len() > 0 {
        json!({
            "mean_ms": (histogram.mean() as f64 / 1000.0).round() as u64,
            "p50_ms": (histogram.value_at_percentile(50.0) as f64 / 1000.0).round() as u64,
            "p95_ms": (histogram.value_at_percentile(95.0) as f64 / 1000.0).round() as u64,
            "p99_ms": (histogram.value_at_percentile(99.0) as f64 / 1000.0).round() as u64,
            "min_ms": (histogram.min() as f64 / 1000.0).round() as u64,
            "max_ms": (histogram.max() as f64 / 1000.0).round() as u64,
        })
    } else {
        serde_json::Value::Null
    };

    let scenario_type = match scenario.ty {
        ScenarioType::Connection => "connection",
        ScenarioType::Message => "message",
    };

    let mut doc = json!({
        "cli_args": args,
        "scenario": {
            "type": scenario_type,
            "server_url": server_url,
            "protocol_header": protocol_header,
            "method_code": method_code,
            "num_parallel": scenario.num_parallel,
            "num_requests": scenario.num_requests,
            "timeout_ms": scenario.timeout_ms,
            "reconnect_attempts": scenario.reconnect_attempts,
        },
        "results": {
            "elapsed_s": elapsed.as_secs_f64(),
            "sent": sent,
            "ok": ok,
            "errors": errors,
            "error_pct": err_pct,
            "rps": rps,
            "latency": latency,
        },
    });

    // Add raw responses if any were collected
    if !raw_responses.is_empty() {
        if let Some(results) = doc.get_mut("results") {
            results["raw_responses"] = serde_json::Value::Array(raw_responses);
        }
    }

    fs::create_dir_all(dir)?;
    let ts = SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let path = format!(
        "{dir}/ws_simple_{ts}_p{}_r{}.json",
        scenario.num_parallel, scenario.num_requests
    );
    fs::write(&path, serde_json::to_string_pretty(&doc)?)?;
    println!("Results written to {path}");
    Ok(())
}
