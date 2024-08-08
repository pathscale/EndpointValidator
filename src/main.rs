use tokio;
use serde_json::Value;
use std::path::PathBuf;
use tokio::time::{sleep, Duration};
use websocket_client::ws::WsClient;
use websocket_client::config::Config;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config_path = PathBuf::from("config.toml");

    // Load the configuration
    let config = Config::try_from(config_path)?;

    // Print loaded configuration
    println!("Loaded config: {:?}", config);

    // Convert the params JSON string to serde_json::Value
    let params: Value = serde_json::from_str(&config.params)?;

    println!("Connecting to server at {}", config.server_url);
    let mut client = WsClient::new(&config.server_url, &config.credentials).await?;
    println!("Connected");

    loop {
        // Receive raw data from the client
        client.send_req(config.method_id, params.clone()).await?;
        let resp = client.recv_raw().await?;
        println!("Response: {:?}", resp);
        // Sleep for 1 second before the next iteration
        sleep(Duration::from_secs(1)).await;
    }

    client.close().await?;
    println!("Connection closed successfully");

    Ok(())
}
