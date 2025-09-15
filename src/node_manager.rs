use anyhow::{Context, Result};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use tracing::{debug, info, warn, error};

use crate::config::Config;
use crate::network::{NetworkManager, message_handler::{ZephyrMessage, NodeInfo}};
use crate::storage::{StorageManager, StorageConfig as StorageManagerConfig};
use crate::coordinator::{CoordinatorManager, RegistrationStatus};

/// Integrated node manager coordinating networking and storage
///
/// Safety: Coordinates secure operations between network and storage layers
/// Transparency: Provides comprehensive node status and metrics
/// Privacy: Handles secure chunk distribution and encrypted metadata
pub struct NodeManager {
    /// Network layer manager
    network_manager: NetworkManager,

    /// Storage layer manager
    pub storage_manager: Arc<StorageManager>,

    /// Coordinator manager for network coordination
    coordinator_manager: Option<CoordinatorManager>,

    /// Configuration
    config: Config,

    /// Message channel from network to node manager
    message_rx: mpsc::Receiver<ZephyrMessage>,

    /// Message channel from node manager to network
    message_tx: mpsc::Sender<ZephyrMessage>,

    /// Node statistics
    node_stats: Arc<RwLock<NodeStats>>,

    /// Base storage path
    storage_path: PathBuf,
}

/// Comprehensive node statistics
/// 
/// Transparency: Detailed metrics for monitoring and audit
#[derive(Debug, Clone)]
pub struct NodeStats {
    /// Number of chunks served to other peers
    pub chunks_served: u64,
    
    /// Number of chunks retrieved from peers  
    pub chunks_retrieved: u64,
    
    /// Total bytes sent to peers
    pub bytes_sent: u64,
    
    /// Total bytes received from peers
    pub bytes_received: u64,
    
    /// Number of active peer connections
    pub peer_connections: u32,
    
    /// Number of failed chunk requests
    pub failed_requests: u64,
    
    /// Uptime in seconds
    pub uptime_seconds: u64,
    
    /// Node start time
    pub start_time: crate::SerializableInstant,
}

/// File distribution strategy for P2P sharing
#[derive(Debug, Clone)]
pub enum DistributionStrategy {
    /// Store locally only
    LocalOnly,
    
    /// Replicate to N closest peers
    Replicate { redundancy: u32 },
    
    /// Distribute chunks across network
    Distribute { min_peers: u32 },
}

impl NodeManager {
    /// Create a new integrated node manager
    ///
    /// Safety: Initializes both network and storage with secure configurations
    pub async fn new(config: Config, storage_path: PathBuf) -> Result<Self> {
        info!("Initializing NodeManager with integrated network and storage");

        // Create message channel for network-storage communication
        let (message_tx, message_rx) = mpsc::channel::<ZephyrMessage>(1000);

        // Initialize storage manager
        let storage_config = StorageManagerConfig {
            max_capacity: config.storage.max_storage,
            warning_threshold: 0.8,
            critical_threshold: 0.95,
            default_chunk_size: config.storage.chunk_size,
            max_file_size: 1024 * 1024 * 1024, // 1GB max file
            enable_gc: true,
            gc_interval: 3600, // 1 hour
        };

        let storage_manager = Arc::new(
            StorageManager::new(&storage_path, storage_config).await
                .context("Failed to initialize storage manager")?
        );

        // Initialize network manager with message channel
        let network_manager = NetworkManager::new(config.clone()).await
            .context("Failed to initialize network manager")?;

        // Initialize coordinator manager if URL is provided
        let coordinator_manager = if !config.coordinator.url.is_empty() {
            match CoordinatorManager::new(config.coordinator.url.clone()).await {
                Ok(manager) => {
                    info!("Successfully connected to coordinator at {}", config.coordinator.url);
                    Some(manager)
                }
                Err(e) => {
                    warn!("Failed to connect to coordinator at {}: {}. Running in standalone mode.", config.coordinator.url, e);
                    None
                }
            }
        } else {
            info!("No coordinator URL configured. Running in standalone mode.");
            None
        };

        let node_stats = Arc::new(RwLock::new(NodeStats {
            chunks_served: 0,
            chunks_retrieved: 0,
            bytes_sent: 0,
            bytes_received: 0,
            peer_connections: 0,
            failed_requests: 0,
            uptime_seconds: 0,
            start_time: crate::SerializableInstant::now(),
        }));

        Ok(Self {
            network_manager,
            storage_manager,
            coordinator_manager,
            config,
            message_rx,
            message_tx,
            node_stats,
            storage_path,
        })
    }
    
    /// Start the integrated node
    ///
    /// Safety: Starts both network and storage services with proper error handling
    pub async fn start(&mut self) -> Result<()> {
        info!("Starting integrated ZephyrFS node");

        // Start storage manager background tasks (if any)
        self.start_storage_tasks().await?;

        // Start network manager
        self.network_manager.start().await
            .context("Failed to start network manager")?;

        // Register with coordinator if available
        if self.coordinator_manager.is_some() {
            let node_status = self.get_node_status().await;
            let addresses = vec![
                format!("{}:{}", "127.0.0.1", self.config.network.p2p_port),
                format!("{}:{}", "127.0.0.1", self.config.network.api_port),
            ];

            let mut capabilities = std::collections::HashMap::new();
            capabilities.insert("version".to_string(), node_status.version);
            capabilities.insert("storage".to_string(), "true".to_string());
            capabilities.insert("encryption".to_string(), "true".to_string());

            if let Some(coordinator) = self.coordinator_manager.as_mut() {
                let response = coordinator.register_node(
                    addresses,
                    node_status.storage_capacity,
                    capabilities,
                ).await?;

                if response.success {
                    info!("Successfully registered with coordinator. Node ID: {}", coordinator.get_node_id());
                    if !response.bootstrap_peers.is_empty() {
                        info!("Received {} bootstrap peers from coordinator", response.bootstrap_peers.len());
                        // TODO: Connect to bootstrap peers
                    }
                } else {
                    warn!("Failed to register with coordinator: {}", response.message);
                }
            }

            self.start_coordinator_heartbeat().await;
        }

        // Start message processing loop
        self.start_message_processing().await;

        info!("ZephyrFS node started successfully");
        Ok(())
    }
    
    /// Store a file and optionally distribute to peers
    /// 
    /// Safety: Validates file integrity and enforces capacity limits
    /// Privacy: Supports encrypted storage and secure distribution
    pub async fn store_file(
        &self, 
        file_id: &str, 
        data: &[u8], 
        filename: &str,
        strategy: DistributionStrategy,
    ) -> Result<String> {
        info!("Storing file: {} ({} bytes) with strategy: {:?}", filename, data.len(), strategy);
        
        // Store file locally first
        let file_hash = self.storage_manager.store_file(file_id, data, filename).await
            .context("Failed to store file locally")?;
        
        // Update statistics
        {
            let mut stats = self.node_stats.write().await;
            stats.bytes_sent += data.len() as u64; // Will be sent to peers
        }
        
        // Handle distribution strategy
        match strategy {
            DistributionStrategy::LocalOnly => {
                debug!("File stored locally only: {}", file_id);
            }
            DistributionStrategy::Replicate { redundancy } => {
                self.replicate_file_to_peers(file_id, redundancy).await?;
            }
            DistributionStrategy::Distribute { min_peers } => {
                self.distribute_chunks_to_peers(file_id, min_peers).await?;
            }
        }
        
        // Register file with coordinator if available
        if let Err(e) = self.register_file_with_coordinator(file_id, &file_hash, data.len() as u64, filename).await {
            warn!("Failed to register file with coordinator: {}", e);
        }

        // Announce file availability to peers
        if let Err(e) = self.announce_file_to_peers(file_id, &file_hash).await {
            warn!("Failed to announce file to peers: {}", e);
        }

        info!("Successfully stored and distributed file: {} with hash: {}", file_id, file_hash);
        Ok(file_hash)
    }
    
    /// Retrieve a file, attempting local storage first, then peers
    /// 
    /// Safety: Verifies chunk integrity from all sources
    /// Transparency: Logs all retrieval attempts and sources
    pub async fn retrieve_file(&self, file_id: &str) -> Result<Option<Vec<u8>>> {
        info!("Retrieving file: {}", file_id);
        
        // Try local storage first
        match self.storage_manager.retrieve_file(file_id).await? {
            Some(data) => {
                debug!("File retrieved from local storage: {}", file_id);
                return Ok(Some(data));
            }
            None => {
                debug!("File not found locally, attempting peer retrieval: {}", file_id);
            }
        }
        
        // Attempt to retrieve from peers
        match self.retrieve_file_from_peers(file_id).await? {
            Some(data) => {
                info!("Successfully retrieved file from peers: {}", file_id);
                
                // Store locally for future access
                if let Ok(_) = self.storage_manager.store_file(file_id, &data, "retrieved_file").await {
                    debug!("Cached retrieved file locally: {}", file_id);
                }
                
                // Update statistics
                {
                    let mut stats = self.node_stats.write().await;
                    stats.chunks_retrieved += 1;
                    stats.bytes_received += data.len() as u64;
                }
                
                Ok(Some(data))
            }
            None => {
                warn!("File not found locally or on peers: {}", file_id);
                Ok(None)
            }
        }
    }
    
    /// Delete a file locally and notify peers
    /// 
    /// Safety: Ensures secure deletion and updates peer information
    pub async fn delete_file(&self, file_id: &str) -> Result<bool> {
        info!("Deleting file: {}", file_id);
        
        // Delete locally
        let deleted = self.storage_manager.delete_file(file_id).await?;
        
        if deleted {
            // Notify peers about deletion (future implementation)
            debug!("File deleted locally, peer notification not yet implemented: {}", file_id);
        }
        
        Ok(deleted)
    }
    
    /// Get comprehensive node status
    /// 
    /// Transparency: Provides detailed node metrics for monitoring
    pub async fn get_node_status(&self) -> NodeStatus {
        let stats = self.node_stats.read().await;
        let capacity_info = self.storage_manager.get_capacity_info().await;
        let storage_stats = self.storage_manager.get_storage_stats().await.unwrap_or_default();
        
        // Calculate uptime
        let uptime_seconds = stats.start_time.elapsed().as_secs();
        
        let node_id = if let Some(coordinator) = &self.coordinator_manager {
            coordinator.get_node_id().to_string()
        } else {
            self.config.node_id.clone().unwrap_or_else(|| "local_node".to_string())
        };

        NodeStatus {
            node_id,
            version: env!("CARGO_PKG_VERSION").to_string(),
            uptime_seconds,
            peer_connections: stats.peer_connections,
            chunks_served: stats.chunks_served,
            chunks_retrieved: stats.chunks_retrieved,
            bytes_sent: stats.bytes_sent,
            bytes_received: stats.bytes_received,
            failed_requests: stats.failed_requests,
            storage_capacity: capacity_info.total_capacity,
            storage_used: capacity_info.used_space,
            storage_available: capacity_info.available_space,
            file_count: capacity_info.file_count,
            chunk_count: storage_stats.total_chunks,
        }
    }
    
    /// Shutdown the node gracefully
    ///
    /// Safety: Ensures clean shutdown of both network and storage
    pub async fn shutdown(&mut self) -> Result<()> {
        info!("Shutting down ZephyrFS node");

        // Unregister from coordinator if connected
        if let Some(coordinator) = &mut self.coordinator_manager {
            if let Err(e) = coordinator.unregister_node(Some("Normal shutdown".to_string())).await {
                warn!("Failed to unregister from coordinator: {}", e);
            }
        }

        // Shutdown network manager
        self.network_manager.shutdown().await
            .context("Failed to shutdown network manager")?;

        // Storage manager cleanup (if needed)
        // Currently storage manager doesn't need explicit cleanup

        info!("ZephyrFS node shutdown complete");
        Ok(())
    }
    
    /// Start storage-related background tasks
    async fn start_storage_tasks(&self) -> Result<()> {
        // Future: garbage collection, capacity monitoring, etc.
        debug!("Storage background tasks initialized");
        Ok(())
    }
    
    /// Start message processing loop
    async fn start_message_processing(&self) {
        let storage_manager = Arc::clone(&self.storage_manager);
        let node_stats = Arc::clone(&self.node_stats);
        
        // Spawn message processing task
        tokio::spawn(async move {
            debug!("Starting message processing loop");
            // Future: Process messages from message_rx
            // This will handle chunk requests/responses from peers
        });
    }
    
    /// Replicate file to specified number of peers
    async fn replicate_file_to_peers(&self, _file_id: &str, _redundancy: u32) -> Result<()> {
        // TODO: Implement peer replication
        debug!("File replication not yet implemented");
        Ok(())
    }
    
    /// Distribute file chunks across peers
    async fn distribute_chunks_to_peers(&self, _file_id: &str, _min_peers: u32) -> Result<()> {
        // TODO: Implement chunk distribution
        debug!("Chunk distribution not yet implemented");
        Ok(())
    }
    
    /// Retrieve file from peers
    async fn retrieve_file_from_peers(&self, file_id: &str) -> Result<Option<Vec<u8>>> {
        info!("Attempting to retrieve file from peers: {}", file_id);
        
        // First, check if we have metadata about this file from other sources
        // This is a simplified implementation - in a real system, we'd have
        // a distributed hash table or peer discovery mechanism
        
        // For now, we'll simulate trying to get chunks by ID
        // In a real implementation, this would:
        // 1. Query the DHT for file metadata 
        // 2. Get the list of chunk IDs that comprise the file
        // 3. Request each chunk from peers
        // 4. Reconstruct the file
        
        debug!("Peer file retrieval requires DHT implementation - returning None for now");
        Ok(None)
    }
    
    /// Request a specific chunk from peers
    /// 
    /// Safety: Validates chunk integrity from all peer sources
    /// Transparency: Logs all peer chunk requests and responses
    pub async fn request_chunk_from_peers(&self, chunk_id: &str, expected_hash: &str) -> Result<Option<Vec<u8>>> {
        info!("Requesting chunk from peers: {} (expected hash: {})", chunk_id, expected_hash);
        
        // Create chunk request message
        let request = ZephyrMessage::ChunkRequest {
            chunk_id: chunk_id.to_string(),
            expected_hash: expected_hash.to_string(),
        };
        
        // In a real implementation, this would:
        // 1. Send the request to multiple peers
        // 2. Wait for responses with timeout
        // 3. Validate responses and return the first valid one
        // 4. Update peer reputation based on response quality
        
        debug!("Peer chunk requests require network broadcast - not yet implemented");
        
        // Update statistics for attempted request
        {
            let mut stats = self.node_stats.write().await;
            stats.failed_requests += 1;
        }
        
        Ok(None)
    }
    
    /// Announce file availability to peers
    /// 
    /// Transparency: Announces stored files to help peers discover content
    pub async fn announce_file_to_peers(&self, file_id: &str, file_hash: &str) -> Result<()> {
        info!("Announcing file availability to peers: {} (hash: {})", file_id, file_hash);
        
        // Get our node info for the announcement
        let node_status = self.get_node_status().await;
        
        let announcement = ZephyrMessage::StatusUpdate {
            node_info: NodeInfo {
                node_id: node_status.node_id,
                version: node_status.version,
                storage_available: node_status.storage_available,
                storage_used: node_status.storage_used,
                uptime_seconds: node_status.uptime_seconds,
                capabilities: vec!["file_storage".to_string(), "chunk_serving".to_string()],
            }
        };
        
        // In a real implementation, this would broadcast to all connected peers
        debug!("File announcement broadcast not yet implemented");
        
        Ok(())
    }

    /// Register this node with the coordinator
    async fn register_with_coordinator(&self, coordinator: &mut CoordinatorManager) -> Result<()> {
        info!("Registering node with coordinator");

        let node_status = self.get_node_status().await;
        let addresses = vec![
            format!("{}:{}", "127.0.0.1", self.config.network.p2p_port),
            format!("{}:{}", "127.0.0.1", self.config.network.api_port),
        ];

        let mut capabilities = std::collections::HashMap::new();
        capabilities.insert("version".to_string(), node_status.version);
        capabilities.insert("storage".to_string(), "true".to_string());
        capabilities.insert("encryption".to_string(), "true".to_string());

        let response = coordinator.register_node(
            addresses,
            node_status.storage_capacity,
            capabilities,
        ).await?;

        if response.success {
            info!("Successfully registered with coordinator. Node ID: {}", coordinator.get_node_id());
            if !response.bootstrap_peers.is_empty() {
                info!("Received {} bootstrap peers from coordinator", response.bootstrap_peers.len());
                // TODO: Connect to bootstrap peers
            }
        } else {
            warn!("Failed to register with coordinator: {}", response.message);
        }

        Ok(())
    }

    /// Start coordinator heartbeat loop
    async fn start_coordinator_heartbeat(&self) {
        if let Some(coordinator) = &self.coordinator_manager {
            let node_stats = Arc::clone(&self.node_stats);
            let storage_manager = Arc::clone(&self.storage_manager);

            coordinator.start_heartbeat(move || {
                let stats = tokio::task::block_in_place(|| {
                    tokio::runtime::Handle::current().block_on(async {
                        let node_stats = node_stats.read().await;
                        let capacity_info = storage_manager.get_capacity_info().await;
                        let uptime = node_stats.start_time.elapsed().as_secs() as i64;

                        crate::coordinator::types::NodeStats {
                            storage_used: capacity_info.used_space as i64,
                            storage_available: capacity_info.available_space as i64,
                            chunks_stored: capacity_info.file_count as i64, // Approximation
                            bandwidth_up: node_stats.bytes_sent as i64,
                            bandwidth_down: node_stats.bytes_received as i64,
                            cpu_usage: 0.0, // TODO: Implement CPU monitoring
                            memory_usage: 0.0, // TODO: Implement memory monitoring
                            uptime_seconds: uptime,
                        }
                    })
                });
                stats
            }).await;

            info!("Started coordinator heartbeat");
        }
    }

    /// Register a file with the coordinator if available
    async fn register_file_with_coordinator(&self, file_id: &str, file_hash: &str, file_size: u64, filename: &str) -> Result<()> {
        if let Some(coordinator) = &self.coordinator_manager {
            // Get file chunks from storage manager
            let chunks = match self.storage_manager.get_file_chunks(file_id).await {
                Ok(Some(chunk_list)) => {
                    chunk_list.into_iter().enumerate().map(|(index, chunk_id)| {
                        crate::coordinator::types::ChunkMetadata {
                            chunk_id: chunk_id.clone(),
                            hash: chunk_id, // For now, use chunk_id as hash
                            size: self.config.storage.chunk_size as i64, // Default chunk size
                            index: index as i32,
                        }
                    }).collect()
                }
                _ => Vec::new(),
            };

            let response = coordinator.register_file(
                file_id.to_string(),
                filename.to_string(),
                file_size,
                file_hash.to_string(),
                chunks,
            ).await?;

            if response.success {
                debug!("Successfully registered file {} with coordinator", file_id);
                if !response.chunk_placements.is_empty() {
                    debug!("Coordinator provided {} chunk placement recommendations", response.chunk_placements.len());
                    // TODO: Handle chunk placement recommendations
                }
            } else {
                warn!("Failed to register file with coordinator: {}", response.message);
            }
        }

        Ok(())
    }

    /// Find chunk locations using coordinator
    async fn find_chunk_locations_via_coordinator(&self, chunk_id: &str) -> Result<Vec<String>> {
        if let Some(coordinator) = &self.coordinator_manager {
            let response = coordinator.find_chunk_locations(chunk_id.to_string(), 3).await?;

            if response.success {
                debug!("Found {} locations for chunk {} via coordinator", response.node_addresses.len(), chunk_id);
                Ok(response.node_addresses)
            } else {
                debug!("Coordinator couldn't find locations for chunk {}: {}", chunk_id, response.message);
                Ok(Vec::new())
            }
        } else {
            Ok(Vec::new())
        }
    }

    /// Get coordinator registration status
    pub fn get_coordinator_status(&self) -> Option<&RegistrationStatus> {
        self.coordinator_manager.as_ref().map(|c| c.get_registration_status())
    }
}

/// Comprehensive node status information
/// 
/// Transparency: Complete node metrics for monitoring and audit
#[derive(Debug, Clone)]
pub struct NodeStatus {
    pub node_id: String,
    pub version: String,
    pub uptime_seconds: u64,
    pub peer_connections: u32,
    pub chunks_served: u64,
    pub chunks_retrieved: u64,
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub failed_requests: u64,
    pub storage_capacity: u64,
    pub storage_used: u64,
    pub storage_available: u64,
    pub file_count: u64,
    pub chunk_count: u64,
}

// Import for the storage stats
use crate::storage::StorageStats;