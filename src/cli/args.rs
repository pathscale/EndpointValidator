use clap::Parser;

/// Command-line arguments structure using `clap`
#[derive(Parser, Debug)]
#[command(name = "endpoint_validator")]
#[command(about = "A tool to validate service endpoints")]
pub struct Cli {
    /// Path to the services.json file
    #[arg(long)]
    pub services_path: Option<String>,

    /// Path to the config.toml file
    #[arg(long)]
    pub config_path: Option<String>,

    /// Run in headless mode (CLI instead of TUI)
    #[arg(long)]
    pub headless: bool,

    /// Endpoint name to validate (required in headless mode)
    #[arg(long)]
    pub endpoint: Option<String>,

    /// Method ID (e.g., 21002 for Login)
    #[arg(long)]
    pub method: Option<u32>,

    /// WebSocket URL to connect to (optional)
    #[arg(long)]
    pub ws_url: Option<String>,

    /// Parameters for the endpoint in JSON format (optional)
    #[arg(long)]
    pub params: Option<String>,

    /// Username for WebSocket handshake authentication
    #[arg(long)]
    pub auth_username: Option<String>,

    /// Password for WebSocket handshake authentication
    #[arg(long)]
    pub auth_password: Option<String>,
}

/// Function to parse command-line arguments
pub fn parse_args() -> Cli {
    Cli::parse()
}
