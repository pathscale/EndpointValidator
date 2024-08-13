mod cli;
mod tui;
mod ws;
mod log;    
mod parser;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {    
    // Parse command-line arguments
    let cli = cli::parse_args();

    // If paths are provided via command-line, use them; otherwise, fallback to interactive input
    let services_path = match cli.services_path {
        Some(services) => services,
        _ => {
            println!("Missing command-line arguments. Switching to interactive mode...");
            cli::collect_paths_interactively()
        }
    };

    let services = parser::load_services(&services_path)?;
    let (endpoint_names, endpoint_data) = services.extract_endpoints();

    // TUI implementation
    tui::run(endpoint_names, endpoint_data).await?;
    Ok(())
}
