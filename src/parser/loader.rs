use crate::parser::Services;
use anyhow::{Context, Result};
use serde_json::from_reader;
use std::fs::File;
use std::path::Path;

pub fn load_services<P: AsRef<Path>>(path: P) -> Result<Services> {
    let file = File::open(&path).context("Failed to open services.json file")?;
    let services: Services = from_reader(file).context("Failed to parse services.json")?;
    Ok(services)
}
