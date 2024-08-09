use tokio::fs;
use std::error::Error;
use serde::Deserialize;
use std::path::PathBuf;
use serde_json::Value;
use tokio::time::{sleep, Duration};
use crate::ws::WsClient;
use crate::config::Config;
use crate::types::*;

pub async fn load_json_file<T: for<'de> Deserialize<'de>>(path: &str) -> Result<T, Box<dyn std::error::Error>> {
    let file_content = fs::read_to_string(path).await?;
    let data: T = serde_json::from_str(&file_content)?;
    Ok(data)
}

pub async fn load_config(path: PathBuf) -> Result<Config, Box<dyn Error>> {
    // Use the custom TryFrom implementation to load the configuration
    let config = Config::try_from(path)?;
    Ok(config)
}

pub async fn process_endpoints(path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let endpoints: Endpoints = load_json_file(path).await?;
    for endpoint in endpoints.0 {
        println!("{:?}", endpoint);
    }
    Ok(())
}

pub async fn process_services(path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let services_data: Services = load_json_file(path).await?;
    for service in &services_data.services {
        println!("Service with {} endpoints:", service.endpoints.len());
        for endpoint in &service.endpoints {
            println!("  Endpoint Name: {}", endpoint.name);
            println!("  Code: {}", endpoint.code);
            println!("  Description: {:?}", endpoint.description);
            println!("  Parameters: {:?}", endpoint.parameters);
            println!("  Returns: {:?}", endpoint.returns);
        }
    }
    Ok(())
}

pub async fn process_error_codes(path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let error_codes: ErrorCodes = load_json_file(path).await?;
    println!("Language: {}", error_codes.language);
    for code in &error_codes.codes {
        println!("{:?}", code);
    }
    Ok(())
}

pub async fn connect_and_process(config: &Config) -> Result<(), Box<dyn std::error::Error>> {
    // Connect the WebSocket server
    println!("Connecting to server at {}", config.server_url);
    let mut client = WsClient::new(&config.server_url, &config.credentials).await?;
    println!("Connected");

    // // Example usage of WebSocket client
    // let params: Value = serde_json::from_str(&config.params)?;
    // loop {
    //     client.send_req(config.method_id, params.clone()).await?;
    //     let resp = client.recv_raw().await?;
    //     println!("Response: {:?}", resp);
    //     sleep(Duration::from_secs(1)).await;
    // }
    Ok(())
}
