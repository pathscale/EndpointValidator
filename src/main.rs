mod cli;
mod tui;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {    
    // Parse command-line arguments
    let cli = cli::parse_args();

    // If paths are provided via command-line, use them; otherwise, fallback to interactive input
    let (services_path, error_codes_path) = match (cli.services_path, cli.error_codes_path) {
        (Some(services), Some(errors)) => (services, errors),
        _ => {
            println!("Missing command-line arguments. Switching to interactive mode...");
            cli::collect_paths_interactively()
        }
    };

    // TUI implementation
    tui::run().await?;
    Ok(())
}
