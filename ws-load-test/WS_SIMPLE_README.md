# ws-simple — single-endpoint load test

A focused binary for testing one WebSocket endpoint. All settings have hardcoded
defaults in `src/bin/ws-simple.rs`; most can be overridden at runtime via CLI flags.

## Build & run

```bash
# From the workspace root
cargo build -p ws-load-test --bin ws-simple

# Run with defaults
cargo run -p ws-load-test --bin ws-simple

# Run with flags
cargo run -p ws-load-test --bin ws-simple -- --server-url ws://localhost:3006 --num-parallel 50
```

## CLI flags

| Flag | Default | Description |
|---|---|---|
| `--server-url <url>` | `ws://127.0.0.1:3006` | WebSocket server address |
| `--type <connection\|message>` | `message` | ws-simple type (see below) |
| `--num-parallel <n>` | `20` | Number of concurrent workers |
| `--num-requests <n>` | `10` | Operations **per worker** |
| `--params-file <path>` | hardcoded | Path to a JSON payload file |
| `--protocol-header-file <path>` | hardcoded | Path to a file containing the protocol header string |
| `-h, --help` | — | Print flag reference and exit |

Total operations = `num_parallel × num_requests`.

## ws-simple types

### `message`

Each worker opens one connection and sends `num_requests` messages on it.
Latency is measured as send-to-response round trip (µs).

```bash
cargo run -p ws-load-test --bin ws-simple -- \
  --type message \
  --num-parallel 20 \
  --num-requests 500 \
  --params-file payload.json
```

### `connection`

Each worker repeatedly opens and closes a connection `num_requests` times.
No messages are sent. Latency measures the full connect-to-close cycle (µs).
Useful for testing connection establishment overhead and server accept capacity.

```bash
cargo run -p ws-load-test --bin ws-simple -- \
  --type connection \
  --num-parallel 50 \
  --num-requests 100
```

## Params file

The `--params-file` flag accepts a JSON file whose contents are sent as the
request payload for every message in a `message` ws-simple.

```json
{
  "message": "hello"
}
```

If omitted, the payload hardcoded in `DEFAULT_PARAMS` inside `src/bin/ws-simple.rs`
is used.

## Protocol header file

The `--protocol-header-file` flag accepts a plain text file containing a single
string used as the `Sec-WebSocket-Protocol` header value. Leading/trailing
whitespace is trimmed automatically.

```
some-protocol
```

See `example_protocol_header.txt` for reference. If omitted, `PROTOCOL_HEADER`
from the source constants is used.

## Hardcoded settings

The following are only configurable by editing the source constants in
`src/bin/ws-simple.rs`:

- `PROTOCOL_HEADER` — `Sec-WebSocket-Protocol` header value
- `HEADERS` — additional HTTP upgrade headers (e.g. `Authorization`)
- `METHOD_CODE` — numeric method ID sent with each request
- `DEFAULT_PARAMS` — fallback JSON payload

## Output

```
┌── ws-simple ─────────────────────────────────
│  Elapsed:   1.243s
│  Sent:      10000
│  OK:        10000
│  Errors:    0  (0.0%)
│  RPS:       8045.2
│  Latency (RTT µs):
│    mean       1243
│    p50        1180
│    p95        2341
│    p99        4102
│    min         412
│    max        9871
└─────────────────────────────────────────────
```

Latency is not recorded for `connection` ws-simples where a connection error
occurs, or for `message` ws-simples where a send/recv error interrupts the loop.
