use clap::Parser;

/// Command-line arguments structure using `clap`
#[derive(Parser, Debug)]
#[command(name = "endpoint_validator")]
#[command(about = "A tool to validate service endpoints")]
pub struct Cli {
    /// Path to the services.json file
    #[arg(long)]
    pub services_path: Option<String>,

    /// Path to the error_codes.json file
    #[arg(long)]
    pub error_codes_path: Option<String>,
}

/// Function to parse command-line arguments
pub fn parse_args() -> Cli {
    Cli::parse()
}
