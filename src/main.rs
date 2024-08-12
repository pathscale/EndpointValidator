mod cli;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Collect paths from the CLI
    let paths = cli::collect_paths();
    Ok(())
}
