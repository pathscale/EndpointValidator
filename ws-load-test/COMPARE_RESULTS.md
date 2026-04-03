# WebSocket Load Test Results Comparison

Generate comparative graphs from multiple load test runs.

## Setup

```bash
# Create virtual environment (one-time)
python -m venv venv
source venv/bin/activate

# Install dependencies
pip install -r requirements.txt
```

## Usage

```bash
# Make sure venv is activated
source venv/bin/activate

# Run comparison on results directory
python compare_results.py /path/to/results/

# Or run from the results directory itself
cd /path/to/results/
python ../../endpoint-optimizer/ws-load-test/compare_results.py
```

## Expected Folder Structure

```
results/
├── aws-results-c5large/
│   ├── ws_simple_1743820800.json
│   └── ws_simple_1743820900.json
├── aws-results-c5xlarge/
│   └── ...
├── gcp-results-n2standard/
│   └── ...
└── graphs/  (created by script)
```

Folder naming convention: `<provider>-results-<machine_type>/`

## Output

The script generates these graphs in the `graphs/` subdirectory:

1. **rps_vs_parallelism.png** - Throughput comparison across all request sizes
2. **rps_vs_parallelism_requests<N>.png** - RPS filtered by specific request count
3. **latency_p99_vs_parallelism.png** - P99 latency comparison
4. **latency_p95_vs_parallelism.png** - P95 latency comparison
5. **error_rate_vs_parallelism.png** - Error rate comparison
6. **rps_latency_scatter.png** - Throughput vs latency trade-off
7. **summary_table.png** - Summary table with key metrics

## Example

```bash
# Run tests and save results
cargo run --release -p ws-load-test --bin ws-simple -- \
  --num-parallel 48 --num-requests 1000 \
  -o ./aws-results-c5large

# Generate comparison graphs
python compare_results.py .
```
