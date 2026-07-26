//! Test harness for [`endpoint-libs`](https://crates.io/crates/endpoint-libs)
//! WebSocket RPC services.
//!
//! Reads `services.json` — the machine-readable endpoint description written by
//! [`endpoint-gen`](https://crates.io/crates/endpoint-gen) — plus a `config.toml`
//! of preset parameter values, and drives every endpoint over a live connection.
//!
//! The binary is the intended entry point; this library exists so the parsing and
//! value-conversion logic is testable and reusable.
//!
//! Schema types are re-exported from `endpoint_libs::model` rather than
//! redefined. See [`parser::models`] for why that matters.

pub mod cli;
pub mod parser;
pub mod tui;
pub mod ws;
