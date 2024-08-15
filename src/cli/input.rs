use read_input::prelude::*;

/// Function to collect paths interactively using `read_input`
pub fn collect_paths_interactively() -> (String, String) {
    let services_path: String = input()
        .msg("Enter the path to services.json: ")
        .get();

    let config_path: String = input()
        .msg("Enter the path to config.toml: ")
        .get();

    (services_path, config_path)
}
