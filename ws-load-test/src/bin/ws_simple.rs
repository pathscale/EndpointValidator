//! Hardcoded single-endpoint scenario.
//!
//! Edit the constants block below, or override at runtime via CLI flags:
//!   cargo run -p ws-load-test --bin scenario -- [OPTIONS]
//!
//! Run with --help for flag reference.

use std::sync::{Arc, LazyLock};
use std::time::{Duration, Instant};

use endpoint_libs::libs::ws::{WsClient, WsResponseGeneric};
use rustls::crypto::ring;
use serde_json::{Value, json};

#[derive(Clone, Copy)]
enum ScenarioType {
    /// Repeatedly open and close a connection; no messages sent.
    /// Latency measures the full connect-to-close round trip.
    Connection,
    /// Open one connection per worker and send messages until the quota is met.
    /// Latency measures send-to-response round trip.
    Message,
}

#[derive(Clone, Copy)]
struct Scenario {
    ty: ScenarioType,
    /// Number of concurrent workers.
    num_parallel: usize,
    /// Operations performed by each worker independently.
    /// Connection: connect/disconnect cycles per worker.
    /// Message:    requests sent per worker.
    num_requests: usize,
}

// ─────────────────────────────────────────────────────────────────────────────

#[tokio::main]
async fn main() -> eyre::Result<()> {
    // ── Connection settings ───────────────────────────────────────────────────────

    // ── Defaults (override via CLI flags) ────────────────────────────────────────
    const DEFAULT_SERVER_URL: &str = "ws://127.0.0.1:3006";
    const DEFAULT_NUM_REQUESTS: usize = 10;
    const DEFAULT_NUM_PARALLEL: usize = 20;
    const DEFAULT_TYPE: ScenarioType = ScenarioType::Message;

    const PROTOCOL_HEADER: &str = "some-protocol";

    /// Extra HTTP headers sent with the WebSocket upgrade request.
    const HEADERS: &[(&str, &str)] = &[
        // ("Authorization", "Bearer token"),
        // ("X-Client-ID", "load-test"),
    ];

    // ── Scenario settings ─────────────────────────────────────────────────────────

    const METHOD_CODE: u32 = 1;

    /// Fallback payload — used when --params-file is not provided.
    static DEFAULT_PARAMS: LazyLock<Value> = LazyLock::new(|| {
        json!({
            "message": "hello"
        })
    });

    // ─────────────────────────────────────────────────────────────────────────────

    ring::default_provider()
        .install_default()
        .expect("Could not install default crypto provider");

    let args: Vec<String> = std::env::args().skip(1).collect();

    if args.iter().any(|a| a == "--help" || a == "-h") {
        println!("Usage: scenario [OPTIONS]");
        println!();
        println!("Options:");
        println!("  --server-url   <url>                WebSocket server  (default: {DEFAULT_SERVER_URL})");
        println!("  --num-requests <n>                  Ops per worker    (default: {DEFAULT_NUM_REQUESTS})");
        println!("  --num-parallel <n>                  Worker count      (default: {DEFAULT_NUM_PARALLEL})");
        println!("  --type         <connection|message>  Scenario type   (default: message)");
        println!("  --params-file          <path>  JSON payload file        (default: hardcoded)");
        println!("  --protocol-header-file <path>  File containing protocol header string (default: hardcoded)");
        println!("  -h, --help                     Print this help");
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
            let raw = std::fs::read_to_string(&path)
                .unwrap_or_else(|e| { eprintln!("Cannot read protocol header file '{path}': {e}"); std::process::exit(1); });
            Box::leak(raw.trim().to_string().into_boxed_str())
        }
        None => PROTOCOL_HEADER,
    };

    let params: Arc<Value> = match parse_flag(&args, "--params-file") {
        Some(path) => {
            let raw = std::fs::read_to_string(&path)
                .unwrap_or_else(|e| { eprintln!("Cannot read params file '{path}': {e}"); std::process::exit(1); });
            let val: Value = serde_json::from_str(&raw)
                .unwrap_or_else(|e| { eprintln!("Invalid JSON in '{path}': {e}"); std::process::exit(1); });
            Arc::new(val)
        }
        None => Arc::new(DEFAULT_PARAMS.clone()),
    };

    let scenario = Scenario {
        ty: scenario_type,
        num_parallel,
        num_requests,
    };

    let start = Instant::now();

    let handles: Vec<_> = (0..scenario.num_parallel)
        .map(|id| {
            let params = Arc::clone(&params);
            tokio::spawn(async move {
                run_worker(id, scenario, server_url, protocol_header, HEADERS, METHOD_CODE, params)
                    .await
            })
        })
        .collect();

    let mut latencies_us: Vec<u64> = Vec::new();
    let mut errors: u64 = 0;
    let mut sent: u64 = 0;

    for h in handles {
        match h.await {
            Ok((w_sent, w_errors, w_lat)) => {
                sent += w_sent;
                errors += w_errors;
                latencies_us.extend(w_lat);
            }
            Err(e) => eprintln!("worker panicked: {e}"),
        }
    }

    print_summary(sent, errors, &latencies_us, start.elapsed());
    Ok(())
}

async fn run_worker(
    id: usize,
    scenario: Scenario,
    server_url: &'static str,
    protocol_header: &'static str,
    headers: &'static [(&'static str, &'static str)],
    method_code: u32,
    params: Arc<Value>,
) -> (u64, u64, Vec<u64>) {
    let mut latencies: Vec<u64> = Vec::new();
    let mut errors: u64 = 0;
    let mut sent: u64 = 0;

    match scenario.ty {
        ScenarioType::Connection => {
            for _ in 0..scenario.num_requests {
                let t0 = Instant::now();
                match connect(server_url, protocol_header, headers).await {
                    Ok(client) => {
                        // Drop closes the connection; time the full cycle.
                        drop(client);
                        latencies.push(t0.elapsed().as_micros() as u64);
                        sent += 1;
                    }
                    Err(e) => {
                        errors += 1;
                        eprintln!("[worker {id}] connect error: {e}");
                    }
                }
            }
        }

        ScenarioType::Message => {
            let mut client = match connect(server_url, protocol_header, headers).await {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("[worker {id}] connect error: {e}");
                    errors += 1;
                    return (sent, errors, latencies);
                }
            };

            for _ in 0..scenario.num_requests {
                if let Err(e) = client.send_req(method_code, &*params).await {
                    eprintln!("[worker {id}] send error: {e}");
                    errors += 1;
                    sent += 1;
                    break;
                }
                sent += 1;

                let t0 = Instant::now();
                match client.recv_raw().await {
                    Ok(WsResponseGeneric::Immediate(_) | WsResponseGeneric::Stream(_)) => {
                        latencies.push(t0.elapsed().as_micros() as u64);
                    }
                    Ok(WsResponseGeneric::Error(e)) => {
                        errors += 1;
                        eprintln!(
                            "[worker {id}] server error: code={} log={}",
                            e.code, e.log_id
                        );
                    }
                    Ok(WsResponseGeneric::Close) => {
                        eprintln!("[worker {id}] server closed connection");
                        break;
                    }
                    Ok(_) => {}
                    Err(e) => {
                        errors += 1;
                        eprintln!("[worker {id}] recv error: {e}");
                        break;
                    }
                }
            }
        }
    }

    (sent, errors, latencies)
}

async fn connect(
    server_url: &'static str,
    protocol_header: &'static str,
    headers: &[(&'static str, &'static str)],
) -> eyre::Result<WsClient> {
    let headers = if headers.is_empty() {
        None
    } else {
        Some(headers.to_vec())
    };
    WsClient::new(server_url, protocol_header, headers).await
}

fn parse_flag(args: &[String], flag: &str) -> Option<String> {
    args.windows(2)
        .find(|w| w[0] == flag)
        .map(|w| w[1].clone())
}

fn print_summary(sent: u64, errors: u64, latencies_us: &[u64], elapsed: Duration) {
    let ok = sent - errors;
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

    if !latencies_us.is_empty() {
        let mut sorted = latencies_us.to_vec();
        sorted.sort_unstable();
        let mean = sorted.iter().sum::<u64>() / sorted.len() as u64;
        let pct = |p: f64| -> u64 {
            let idx = ((p / 100.0) * (sorted.len() as f64 - 1.0)).round() as usize;
            sorted[idx.min(sorted.len() - 1)]
        };
        println!("│  Latency (RTT µs):");
        println!("│    mean  {:>8}", mean);
        println!("│    p50   {:>8}", pct(50.0));
        println!("│    p95   {:>8}", pct(95.0));
        println!("│    p99   {:>8}", pct(99.0));
        println!("│    min   {:>8}", pct(0.0));
        println!("│    max   {:>8}", pct(100.0));
    }
    println!("└─────────────────────────────────────────────");
}
