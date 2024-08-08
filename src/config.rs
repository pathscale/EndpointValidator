use serde::Deserialize;
use std::str::FromStr;
use std::path::PathBuf;
use toml;
use eyre::Result;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub server_url: String,
    pub credentials: String,
    pub method_id: u32,
    pub params: String, 
}

impl FromStr for Config {
    type Err = toml::de::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        toml::from_str(s)
    }
}

impl TryFrom<PathBuf> for Config {
    type Error = eyre::Error;

    fn try_from(path: PathBuf) -> Result<Self, Self::Error> {
        let toml_str = std::fs::read_to_string(&path)
            .map_err(|e| eyre::eyre!("Failed to read file: {}", e))?;
        Config::from_str(&toml_str).map_err(|e| eyre::eyre!("Failed to parse TOML: {}", e))
    }
}