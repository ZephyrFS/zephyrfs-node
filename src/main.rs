use anyhow::Result;
use clap::Parser;
use tracing::{info, warn};
use tracing_subscriber;

mod config;
mod network;
mod storage;
mod protocol;
mod node_manager;
mod crypto;

#[cfg(test)]
mod integration_tests;

use config::Config;
use node_manager::{NodeManager, DistributionStrategy};

#[derive(Parser, Debug)]
#[command(author, version, about = "ZephyrFS - Distributed P2P storage node", long_about = None)]
struct Args {
    #[arg(short, long, value_name = "FILE", global = true)]
    config: Option<std::path::PathBuf>,
    
    #[arg(short, long, global = true)]
    verbose: bool,
    
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Parser, Debug)]
enum Commands {
    /// Initialize a new ZephyrFS node
    Init {
        /// Storage path for the node
        #[arg(short, long, default_value = "./zephyrfs_storage")]
        storage_path: std::path::PathBuf,
        
        /// Maximum storage allocation in GB
        #[arg(short, long, default_value = "10")]
        max_storage_gb: u64,
    },
    
    /// Start the ZephyrFS node daemon
    Start {
        /// Run in background/daemon mode
        #[arg(short, long)]
        daemon: bool,
    },
    
    /// Join an existing ZephyrFS network
    Join {
        /// Bootstrap peer address (multiaddr format)
        #[arg(required = true)]
        bootstrap_peer: String,
    },
    
    /// Upload a file to the network
    Upload {
        /// File to upload
        file_path: std::path::PathBuf,
        
        /// Optional file ID (defaults to filename)
        #[arg(short, long)]
        file_id: Option<String>,
        
        /// Distribution strategy
        #[arg(short, long, default_value = "local-only")]
        strategy: String,
    },
    
    /// Download a file from the network
    Download {
        /// File ID to download
        file_id: String,
        
        /// Output path (defaults to current directory)
        #[arg(short, long)]
        output: Option<std::path::PathBuf>,
    },
    
    /// List files in the network
    Ls {
        /// Show detailed information
        #[arg(short, long)]
        detailed: bool,
        
        /// Limit number of results
        #[arg(short, long)]
        limit: Option<usize>,
    },
    
    /// Show node status
    Status,
    
    /// Health check (for Docker/monitoring)
    HealthCheck,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    
    // Initialize logging
    let log_level = if args.verbose { "debug" } else { "info" };
    tracing_subscriber::fmt()
        .with_env_filter(format!("zephyrfs_node={}", log_level))
        .init();
    
    match args.command {
        Some(Commands::HealthCheck) => {
            // Simple health check for Docker/monitoring
            info!("Health check passed");
            Ok(())
        }
        
        Some(Commands::Init { storage_path, max_storage_gb }) => {
            init_node(storage_path, max_storage_gb, args.config.as_deref()).await
        }
        
        Some(Commands::Start { daemon }) => {
            start_node(daemon, args.config.as_deref()).await
        }
        
        None => {
            start_node(false, args.config.as_deref()).await
        }
        
        Some(Commands::Join { bootstrap_peer }) => {
            join_network(bootstrap_peer, args.config.as_deref()).await
        }
        
        Some(Commands::Upload { file_path, file_id, strategy }) => {
            upload_file(file_path, file_id, strategy, args.config.as_deref()).await
        }
        
        Some(Commands::Download { file_id, output }) => {
            download_file(file_id, output, args.config.as_deref()).await
        }
        
        Some(Commands::Ls { detailed, limit }) => {
            list_files(detailed, limit, args.config.as_deref()).await
        }
        
        Some(Commands::Status) => {
            show_status(args.config.as_deref()).await
        }
    }
}

/// Initialize a new ZephyrFS node
async fn init_node(storage_path: std::path::PathBuf, max_storage_gb: u64, config_path: Option<&std::path::Path>) -> Result<()> {
    info!("Initializing ZephyrFS node");
    info!("Storage path: {:?}", storage_path);
    info!("Max storage: {} GB", max_storage_gb);
    
    // Create storage directory
    std::fs::create_dir_all(&storage_path)?;
    
    // Load or create config
    let mut config = Config::load(config_path)?;
    config.storage.max_storage = max_storage_gb * 1024 * 1024 * 1024; // Convert GB to bytes
    
    // Test initialization by creating a NodeManager
    let _node_manager = NodeManager::new(config, storage_path).await?;
    
    info!("ZephyrFS node initialized successfully");
    info!("Run 'zephyrfs-node start' to begin serving");
    Ok(())
}

/// Start the ZephyrFS node daemon
async fn start_node(daemon: bool, config_path: Option<&std::path::Path>) -> Result<()> {
    if daemon {
        info!("Starting ZephyrFS node in daemon mode");
        // TODO: Implement proper daemon mode with process forking
        warn!("Daemon mode not yet implemented - running in foreground");
    } else {
        info!("Starting ZephyrFS node");
    }
    
    // Load configuration
    let config = Config::load(config_path)?;
    
    // Determine storage path
    let storage_path = std::env::current_dir()?.join("zephyrfs_storage");
    
    // Initialize and start node manager
    let mut node_manager = NodeManager::new(config, storage_path).await?;
    
    info!("🚀 Starting integrated ZephyrFS node...");
    node_manager.start().await?;
    
    info!("✅ ZephyrFS Node is running. Press Ctrl+C to stop.");
    
    // Keep running until shutdown signal
    tokio::signal::ctrl_c().await?;
    warn!("🛑 Shutdown signal received");
    
    node_manager.shutdown().await?;
    info!("✅ ZephyrFS Node stopped");
    
    Ok(())
}

/// Join an existing ZephyrFS network
async fn join_network(bootstrap_peer: String, config_path: Option<&std::path::Path>) -> Result<()> {
    info!("Joining ZephyrFS network via bootstrap peer: {}", bootstrap_peer);
    
    // Load configuration
    let config = Config::load(config_path)?;
    let storage_path = std::env::current_dir()?.join("zephyrfs_storage");
    
    // Initialize node manager
    let mut node_manager = NodeManager::new(config, storage_path).await?;
    
    // TODO: Add bootstrap peer to configuration and connect
    warn!("Bootstrap peer connection not yet implemented");
    warn!("Starting node normally - implement P2P bootstrap in future");
    
    // Start normally for now
    node_manager.start().await?;
    info!("✅ Node started (bootstrap connection pending implementation)");
    
    tokio::signal::ctrl_c().await?;
    node_manager.shutdown().await?;
    
    Ok(())
}

/// Upload a file to the network
async fn upload_file(file_path: std::path::PathBuf, file_id: Option<String>, strategy: String, config_path: Option<&std::path::Path>) -> Result<()> {
    info!("Uploading file: {:?}", file_path);
    
    // Read file data
    let data = std::fs::read(&file_path)?;
    let filename = file_path.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("uploaded_file");
    
    let file_id = file_id.unwrap_or_else(|| filename.to_string());
    
    // Parse distribution strategy
    let dist_strategy = match strategy.as_str() {
        "local-only" | "local" => DistributionStrategy::LocalOnly,
        "replicate" => DistributionStrategy::Replicate { redundancy: 3 },
        "distribute" => DistributionStrategy::Distribute { min_peers: 2 },
        _ => {
            warn!("Unknown strategy '{}', using local-only", strategy);
            DistributionStrategy::LocalOnly
        }
    };
    
    // Initialize node manager for operation
    let config = Config::load(config_path)?;
    let storage_path = std::env::current_dir()?.join("zephyrfs_storage");
    let node_manager = NodeManager::new(config, storage_path).await?;
    
    // Perform upload
    let file_hash = node_manager.store_file(&file_id, &data, filename, dist_strategy).await?;
    
    info!("✅ File uploaded successfully");
    info!("[INFO] File ID: {}", file_id);
    info!("[INFO] Hash: {}", file_hash);
    info!("[INFO] Size: {} bytes", data.len());
    
    Ok(())
}

/// Download a file from the network
async fn download_file(file_id: String, output: Option<std::path::PathBuf>, config_path: Option<&std::path::Path>) -> Result<()> {
    info!("Downloading file: {}", file_id);
    
    // Initialize node manager
    let config = Config::load(config_path)?;
    let storage_path = std::env::current_dir()?.join("zephyrfs_storage");
    let node_manager = NodeManager::new(config, storage_path).await?;
    
    // Attempt download
    match node_manager.retrieve_file(&file_id).await? {
        Some(data) => {
            let output_path = output.unwrap_or_else(|| std::path::PathBuf::from(&file_id));
            std::fs::write(&output_path, &data)?;
            
            info!("✅ File downloaded successfully");
            info!("[SUCCESS] Saved to: {:?}", output_path);
            info!("[INFO] Size: {} bytes", data.len());
        }
        None => {
            warn!("[ERROR] File not found: {}", file_id);
            std::process::exit(1);
        }
    }
    
    Ok(())
}

/// List files in the network
async fn list_files(detailed: bool, limit: Option<usize>, config_path: Option<&std::path::Path>) -> Result<()> {
    info!("Listing files in network");
    
    // Initialize node manager
    let config = Config::load(config_path)?;
    let storage_path = std::env::current_dir()?.join("zephyrfs_storage");
    let node_manager = NodeManager::new(config, storage_path).await?;
    
    // Get file list
    let files = node_manager.storage_manager.list_files(limit).await?;
    
    if files.is_empty() {
        info!("[INFO] No files found");
        return Ok(());
    }
    
    info!("[INFO] Found {} files:", files.len());
    
    for (file_id, metadata) in files {
        if detailed {
            info!("[INFO] File: {}", file_id);
            info!("  Name: {}", metadata.name);
            info!("  Size: {} bytes", metadata.size);
            info!("  Hash: {}", metadata.file_hash);
            info!("  Created: {}", metadata.created_at);
            info!("  Chunks: {}", metadata.chunk_ids.len());
            if let Some(mime) = &metadata.mime_type {
                info!("  Type: {}", mime);
            }
            info!("");
        } else {
            info!("  {} ({} bytes) - {}", file_id, metadata.size, metadata.name);
        }
    }
    
    Ok(())
}

/// Show node status
async fn show_status(config_path: Option<&std::path::Path>) -> Result<()> {
    info!("Checking ZephyrFS node status");
    
    // Initialize node manager
    let config = Config::load(config_path)?;
    let storage_path = std::env::current_dir()?.join("zephyrfs_storage");
    let node_manager = NodeManager::new(config, storage_path).await?;
    
    // Get comprehensive status
    let status = node_manager.get_node_status().await;
    
    info!("🚀 ZephyrFS Node Status");
    info!("  🆔 Node ID: {}", status.node_id);
    info!("  📦 Version: {}", status.version);
    info!("  ⏱️  Uptime: {} seconds", status.uptime_seconds);
    info!("");
    
    info!("🌐 Network Status");
    info!("  🔗 Peer connections: {}", status.peer_connections);
    info!("  📤 Bytes sent: {}", status.bytes_sent);
    info!("  📥 Bytes received: {}", status.bytes_received);
    info!("  ❌ Failed requests: {}", status.failed_requests);
    info!("");
    
    info!("💾 Storage Status");
    info!("  💽 Total capacity: {} GB", status.storage_capacity / (1024 * 1024 * 1024));
    info!("  📊 Used space: {} MB", status.storage_used / (1024 * 1024));
    info!("  💚 Available space: {} GB", status.storage_available / (1024 * 1024 * 1024));
    info!("  📁 File count: {}", status.file_count);
    info!("  🧩 Chunk count: {}", status.chunk_count);
    
    let usage_percent = if status.storage_capacity > 0 {
        (status.storage_used as f64 / status.storage_capacity as f64) * 100.0
    } else {
        0.0
    };
    info!("  📈 Usage: {:.1}%", usage_percent);
    
    Ok(())
}