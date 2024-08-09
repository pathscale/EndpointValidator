use std::path::PathBuf;
use websocket_client::helper::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config_path = PathBuf::from("config.toml");

    // Load the configuration
    let config = load_config(config_path).await?;
    println!("Loaded config: {:?}", config);

    // Process JSON files
    process_endpoints(&config.endpoints_path).await?;
    process_services(&config.services_path).await?;
    process_error_codes(&config.error_codes_path).await?;

    // Uncomment if you want to connect and process via WebSocket
    connect_and_process(&config).await?;

    Ok(())
}
