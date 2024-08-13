use read_input::prelude::*;

/// Function to collect paths interactively using `read_input`
pub fn collect_paths_interactively() -> (String, String) {
    let services_path: String = input()
        .msg("Enter the path to services.json: ")
        .get();

    let error_codes_path: String = input()
        .msg("Enter the path to error_codes.json: ")
        .get();

    (services_path, error_codes_path)
}
