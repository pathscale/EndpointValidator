use std::path::PathBuf;

use clap::{Parser, Subcommand};

/// With no subcommand, the interactive TUI.
#[derive(Parser, Debug)]
#[command(name = "endpoint-validator", version)]
#[command(about = "Exercise an endpoint-libs WebSocket service interactively")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,
    /// Path to the services.json file (TUI)
    #[arg(long)]
    pub services_path: Option<String>,
    /// Path to the config.toml of preset parameters (TUI)
    #[arg(long)]
    pub config_path: Option<String>,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// List the endpoints in a services.json: code, name, parameters.
    List { services: PathBuf },
}

/// Function to parse command-line arguments
pub fn parse_args() -> Cli {
    Cli::parse()
}
