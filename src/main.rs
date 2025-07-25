mod cli;
mod parser;
mod tui;
mod ws;

use serde_json;
use std::collections::HashMap;

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
    let (endpoint_names, endpoint_metadata, endpoint_data) =
        services.extract_endpoints_with_metadata();

    let config = parser::load_config(&config_path)?;
    let param_defaults = parser::extract_param_defaults(&config.endpoints);

    if cli.headless {
        // CLI mode
        let endpoint_name = cli.endpoint.ok_or_else(|| {
            Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Endpoint name is required in headless mode (--endpoint)",
            )) as Box<dyn std::error::Error>
        })?;

        if !endpoint_metadata.contains_key(&endpoint_name) {
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("Endpoint '{}' not found", endpoint_name),
            )) as Box<dyn std::error::Error>);
        }

        let param_defaults: HashMap<String, serde_json::Value> = param_defaults
            .get(&endpoint_name)
            .map(|params| {
                params
                    .iter()
                    .map(|(k, v)| (k.clone(), serde_json::Value::String(v.clone())))
                    .collect()
            })
            .unwrap_or_default();

        let endpoint_data = endpoint_data.get(&endpoint_name).unwrap().clone();
        let endpoint_metadata = endpoint_metadata.get(&endpoint_name).unwrap().clone();
        let runner = cli::CliRunner::new(
            endpoint_name,
            endpoint_metadata,
            endpoint_data,
            param_defaults,
            cli.ws_url,
            cli.params,
            cli.auth_username,
            cli.auth_password,
            cli.method,
        );
        runner.run().await?;
    } else {
        // TUI mode
        tui::run(endpoint_names, endpoint_metadata, param_defaults).await?;
    }

    Ok(())
}
