mod cli;
mod tui;
mod ws;
mod parser;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {    
    // Parse command-line arguments
    let cli = cli::parse_args();

    // If paths are provided via command-line, use them; otherwise, fallback to interactive input
    let (services_path, config_path) = match (cli.services_path, cli.config_path) {
        (Some(services), Some(config)) => (services, config),
        _ => {
            println!("Missing command-line arguments. Switching to interactive mode...");
            cli::collect_paths_interactively()
        }
    };

    let services = parser::load_services(&services_path)?;
    let (endpoint_names, endpoint_data) = services.extract_endpoints();
  
    let config = parser::load_config(&config_path)?;
    let param_defaults = parser::extract_param_defaults(&config.endpoints);

    // TUI implementation
    tui::run(endpoint_names, endpoint_data, param_defaults).await?;
    Ok(())
}
