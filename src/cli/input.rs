use read_input::prelude::*;

/// Function to collect paths interactively using `read_input`
pub fn collect_paths_interactively() -> String {
    let services_path: String = input()
        .msg("Enter the path to services.json: ")
        .get();

    services_path
}
