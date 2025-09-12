use anyhow::Result;
use clap::Parser;
use tracing::{info, warn};
use tracing_subscriber;

mod config;
mod network;
mod storage;
mod protocol;

use config::Config;
use network::NetworkManager;

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
    
    // Initialize network manager
    let mut network_manager = NetworkManager::new(config).await?;
    
    // Start the node
    info!("Starting P2P networking...");
    network_manager.start().await?;
    
    // Keep running until shutdown signal
    tokio::signal::ctrl_c().await?;
    warn!("Shutdown signal received");
    
    network_manager.shutdown().await?;
    info!("ZephyrFS Node stopped");
    
    Ok(())
}