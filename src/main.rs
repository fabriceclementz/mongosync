use std::fs::File;

use anyhow::{Context, Result};
use clap::Parser;
use mongosync::cli::Args;
use mongosync::config::Config;
use mongosync::replication;

#[tokio::main]
async fn main() -> Result<()> {
    // Register a global tracing subscriber which will obey the RUST_LOG environment variable config.
    tracing_subscriber::fmt::init();

    let args = Args::parse();
    let config_file = File::open(&args.config).context("Failed to read mongosync config file")?;
    let config: Config = serde_yaml::from_reader(config_file).context("Bad config file format")?;

    replication::run(&config)
        .await
        .context("Failed to run realtime replication")?;

    Ok(())
}
