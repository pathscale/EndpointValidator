use std::process::ExitCode;

use endpoint_validator::cli::{self, Command};
use endpoint_validator::{parser, tui};
use eyre::Result;
use nagoya::reactor::{Reactor, block_on_with};

fn main() -> Result<ExitCode> {
    let cli = cli::parse_args();

    // endpoint-libs' client runs on a nagoya reactor the caller owns; this is
    // the one reactor, and the TUI runs as a single task on it.
    let reactor = Reactor::local()?;
    let handle = reactor.handle();

    match cli.command {
        Some(Command::List { services }) => {
            let services = parser::load_services(&services)?;
            for service in &services.services {
                println!("{} (service {})", service.name, service.id);
                for endpoint in &service.endpoints {
                    let params = endpoint
                        .parameters
                        .iter()
                        .map(|p| {
                            format!(
                                "{}: {}",
                                p.name,
                                serde_json::to_string(&p.ty).unwrap_or_default()
                            )
                        })
                        .collect::<Vec<_>>()
                        .join(", ");
                    println!("  {:>6} {}({params})", endpoint.code, endpoint.name);
                }
            }
            Ok(ExitCode::SUCCESS)
        }
        None => {
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
            block_on_with(
                &reactor,
                tui::run(endpoint_names, endpoint_data, param_defaults, handle),
            )?;
            Ok(ExitCode::SUCCESS)
        }
    }
}
