use anyhow::Result;
use clap::Parser;
use tracing::{info, warn};
use tracing_subscriber;

mod config;
mod network;
mod storage;
mod protocol;
mod node_manager;

#[cfg(test)]
mod integration_tests;

use config::Config;
use node_manager::{NodeManager, DistributionStrategy};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long, value_name = "FILE")]
    config: Option<std::path::PathBuf>,
    
    #[arg(long)]
    health_check: bool,
    
    #[arg(short, long)]
    verbose: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    
    // Initialize logging
    let log_level = if args.verbose { "debug" } else { "info" };
    tracing_subscriber::fmt()
        .with_env_filter(format!("zephyrfs_node={}", log_level))
        .init();

    if args.health_check {
        // Simple health check for Docker
        info!("Health check passed");
        return Ok(());
    }

    info!("Starting ZephyrFS Node");
    
    // Load configuration
    let config = Config::load(args.config.as_deref())?;
    info!("Configuration loaded: {:?}", config);
    
    // Determine storage path
    let storage_path = std::env::current_dir()?
        .join("zephyrfs_storage");
    
    // Initialize integrated node manager
    let mut node_manager = NodeManager::new(config.clone(), storage_path).await?;
    
    // Start the integrated node
    info!("Starting integrated ZephyrFS node...");
    node_manager.start().await?;
    
    info!("ZephyrFS Node is running. Press Ctrl+C to stop.");
    
    // Keep running until shutdown signal
    tokio::signal::ctrl_c().await?;
    warn!("Shutdown signal received");
    
    node_manager.shutdown().await?;
    info!("ZephyrFS Node stopped");
    
    Ok(())
}