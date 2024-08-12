use read_input::prelude::*;

pub struct CliPaths {
    pub services_path: String,
    pub error_codes_path: String,
}

pub fn collect_paths() -> CliPaths {
    let services_path: String = input()
        .msg("Enter the path to services.json: ")
        .get();

    let error_codes_path: String = input()
        .msg("Enter the path to error_codes.json: ")
        .get();

    CliPaths {
        services_path,
        error_codes_path,
    }
}
