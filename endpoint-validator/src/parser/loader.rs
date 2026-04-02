use crate::parser::{Services, Config};
use anyhow::{Context, Result};
use serde_json::from_reader;
use std::fs::{self, File};
use std::path::Path;

pub fn load_services<P: AsRef<Path>>(path: P) -> Result<Services> {
    let file = File::open(&path).context("Failed to open services.json file")?;
    let services: Services = from_reader(file).context("Failed to parse services.json")?;
    Ok(services)
}

pub fn load_config(path: &str) -> Result<Config> {
    let config_content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read config file: {}", path))?;
    let config: Config = toml::from_str(&config_content)
        .with_context(|| "Failed to parse config file")?;
    
    Ok(config)
}
