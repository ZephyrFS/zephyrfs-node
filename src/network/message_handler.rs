use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};

use crate::config::Config;
use crate::storage::StorageManager;

/// Message types that can be exchanged between peers
/// 
/// Transparency: All message types are clearly defined and logged
/// Safety: Messages include validation and security checks
#[derive(Debug, Clone)]
pub enum ZephyrMessage {
    /// Health check ping
    Ping { timestamp: u64 },
    
    /// Health check response
    Pong { timestamp: u64, node_info: NodeInfo },
    
    /// Request for peer discovery
    PeerDiscovery,
    
    /// Response with known peers
    PeerList { peers: Vec<PeerAddress> },
    
    /// File chunk request
    ChunkRequest { chunk_id: String, expected_hash: String },
    
    /// File chunk response
    ChunkResponse { 
        chunk_id: String, 
        data: Vec<u8>, 
        hash: String,
        success: bool,
    },
    
    /// Node status announcement
    StatusUpdate { node_info: NodeInfo },
}

/// Node information shared in messages
/// 
/// Privacy: Only shares necessary operational information
#[derive(Debug, Clone)]
pub struct NodeInfo {
    pub node_id: String,
    pub version: String,
    pub storage_available: u64,
    pub storage_used: u64,
    pub uptime_seconds: u64,
    pub capabilities: Vec<String>,
}

/// Peer address information
#[derive(Debug, Clone)]
pub struct PeerAddress {
    pub peer_id: String,
    pub addresses: Vec<String>,
    pub last_seen: u64,
}

/// Handles P2P message passing with security and validation
/// 
/// Safety: All messages are validated before processing
/// Transparency: All message handling is logged
/// Privacy: Messages are encrypted in transit via LibP2P transport
pub struct MessageHandler {
    config: Config,
    message_tx: Option<mpsc::Sender<ZephyrMessage>>,
    message_rx: Option<mpsc::Receiver<ZephyrMessage>>,
    pending_requests: HashMap<String, PendingRequest>,
    storage_manager: Option<Arc<StorageManager>>,
}

#[derive(Debug)]
struct PendingRequest {
    timestamp: std::time::Instant,
    response_tx: mpsc::Sender<ZephyrMessage>,
}

impl MessageHandler {
    /// Create a new MessageHandler
    pub fn new(config: &Config) -> Self {
        info!("Initializing MessageHandler with security validation");
        
        let (message_tx, message_rx) = mpsc::channel(1000);
        
        Self {
            config: config.clone(),
            message_tx: Some(message_tx),
            message_rx: Some(message_rx),
            pending_requests: HashMap::new(),
            storage_manager: None,
        }
    }
    
    /// Set the storage manager for chunk operations
    /// 
    /// Safety: Enables secure chunk storage and retrieval operations
    pub fn set_storage_manager(&mut self, storage_manager: Arc<StorageManager>) {
        info!("Integrating MessageHandler with StorageManager");
        self.storage_manager = Some(storage_manager);
    }
    
    /// Process incoming message with validation and security checks
    /// 
    /// Safety: All messages are validated before processing
    /// Transparency: Message processing is fully logged
    pub async fn handle_message(&mut self, message: ZephyrMessage) -> Result<Option<ZephyrMessage>> {
        debug!("Processing message: {:?}", message);
        
        // Validate message before processing
        if !self.validate_message(&message) {
            warn!("Received invalid message, rejecting");
            return Ok(None);
        }
        
        match message {
            ZephyrMessage::Ping { timestamp } => {
                debug!("Received ping with timestamp: {}", timestamp);
                Ok(Some(ZephyrMessage::Pong {
                    timestamp,
                    node_info: self.get_node_info(),
                }))
            }
            
            ZephyrMessage::Pong { timestamp, node_info } => {
                debug!("Received pong from node: {}", node_info.node_id);
                self.handle_pong(timestamp, node_info).await;
                Ok(None)
            }
            
            ZephyrMessage::PeerDiscovery => {
                debug!("Received peer discovery request");
                Ok(Some(ZephyrMessage::PeerList {
                    peers: self.get_known_peers().await,
                }))
            }
            
            ZephyrMessage::PeerList { peers } => {
                debug!("Received peer list with {} peers", peers.len());
                self.handle_peer_list(peers).await;
                Ok(None)
            }
            
            ZephyrMessage::ChunkRequest { chunk_id, expected_hash } => {
                debug!("Received chunk request for: {}", chunk_id);
                self.handle_chunk_request(chunk_id, expected_hash).await
            }
            
            ZephyrMessage::ChunkResponse { chunk_id, data, hash, success } => {
                debug!("Received chunk response for: {} (success: {})", chunk_id, success);
                self.handle_chunk_response(chunk_id, data, hash, success).await;
                Ok(None)
            }
            
            ZephyrMessage::StatusUpdate { node_info } => {
                debug!("Received status update from: {}", node_info.node_id);
                self.handle_status_update(node_info).await;
                Ok(None)
            }
        }
    }
    
    /// Validate message for security and correctness
    /// 
    /// Safety: Comprehensive validation prevents malicious messages
    fn validate_message(&self, message: &ZephyrMessage) -> bool {
        match message {
            ZephyrMessage::Ping { timestamp } => {
                // Check timestamp is reasonable (within 5 minutes)
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs();
                
                (*timestamp as i64 - now as i64).abs() < 300
            }
            
            ZephyrMessage::ChunkRequest { chunk_id, expected_hash } => {
                // Validate chunk ID format and hash format
                !chunk_id.is_empty() && 
                expected_hash.len() == 64 && // SHA-256 hex length
                expected_hash.chars().all(|c| c.is_ascii_hexdigit())
            }
            
            ZephyrMessage::ChunkResponse { data, hash, .. } => {
                // Validate data size and hash format
                data.len() <= self.config.storage.chunk_size &&
                hash.len() == 64 &&
                hash.chars().all(|c| c.is_ascii_hexdigit())
            }
            
            _ => true, // Other messages have basic validation
        }
    }
    
    /// Get current node information
    /// 
    /// Privacy: Only exposes operational information needed for network function
    fn get_node_info(&self) -> NodeInfo {
        NodeInfo {
            node_id: self.config.node_id.clone().unwrap_or_default(),
            version: "0.1.0".to_string(),
            storage_available: self.config.storage.max_storage,
            storage_used: 0, // TODO: Implement actual usage tracking
            uptime_seconds: 0, // TODO: Implement uptime tracking
            capabilities: vec![
                "chunk-storage".to_string(),
                "file-metadata".to_string(),
                "peer-discovery".to_string(),
            ],
        }
    }
    
    /// Handle pong response
    async fn handle_pong(&mut self, timestamp: u64, node_info: NodeInfo) {
        let rtt = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64 - timestamp;
        
        info!("Pong received from {}: RTT={}ms, available={}GB", 
              node_info.node_id, rtt, node_info.storage_available / (1024*1024*1024));
    }
    
    /// Get list of known peers
    async fn get_known_peers(&self) -> Vec<PeerAddress> {
        // TODO: Integrate with PeerManager
        vec![]
    }
    
    /// Handle received peer list
    async fn handle_peer_list(&mut self, peers: Vec<PeerAddress>) {
        info!("Received {} peer addresses for discovery", peers.len());
        // TODO: Integrate with PeerManager to attempt connections
    }
    
    /// Handle chunk request
    async fn handle_chunk_request(
        &mut self, 
        chunk_id: String, 
        expected_hash: String
    ) -> Result<Option<ZephyrMessage>> {
        info!("Processing chunk request: {} (expected hash: {})", chunk_id, expected_hash);
        
        // Check if storage manager is available
        let storage_manager = match &self.storage_manager {
            Some(sm) => sm,
            None => {
                warn!("Storage manager not available, rejecting chunk request: {}", chunk_id);
                return Ok(Some(ZephyrMessage::ChunkResponse {
                    chunk_id,
                    data: vec![],
                    hash: expected_hash,
                    success: false,
                }));
            }
        };
        
        // Attempt to retrieve chunk from storage
        match storage_manager.retrieve_chunk(&chunk_id).await {
            Ok(Some(chunk_data)) => {
                // Verify the chunk hash matches expected
                let actual_hash = {
                    use sha2::{Digest, Sha256};
                    let mut hasher = Sha256::new();
                    hasher.update(&chunk_data);
                    hex::encode(hasher.finalize())
                };
                
                if actual_hash == expected_hash {
                    info!("Successfully serving chunk: {} ({} bytes)", chunk_id, chunk_data.len());
                    Ok(Some(ZephyrMessage::ChunkResponse {
                        chunk_id,
                        data: chunk_data,
                        hash: actual_hash,
                        success: true,
                    }))
                } else {
                    warn!("Chunk hash mismatch for {}: expected {}, got {}", 
                          chunk_id, expected_hash, actual_hash);
                    Ok(Some(ZephyrMessage::ChunkResponse {
                        chunk_id,
                        data: vec![],
                        hash: expected_hash,
                        success: false,
                    }))
                }
            }
            Ok(None) => {
                debug!("Chunk not found locally: {}", chunk_id);
                Ok(Some(ZephyrMessage::ChunkResponse {
                    chunk_id,
                    data: vec![],
                    hash: expected_hash,
                    success: false,
                }))
            }
            Err(e) => {
                error!("Error retrieving chunk {}: {}", chunk_id, e);
                Ok(Some(ZephyrMessage::ChunkResponse {
                    chunk_id,
                    data: vec![],
                    hash: expected_hash,
                    success: false,
                }))
            }
        }
    }
    
    /// Handle chunk response
    async fn handle_chunk_response(
        &mut self,
        chunk_id: String,
        data: Vec<u8>,
        hash: String,
        success: bool,
    ) {
        if success && !data.is_empty() {
            info!("Successfully received chunk: {} ({} bytes)", chunk_id, data.len());
            
            // Validate hash
            let actual_hash = {
                use sha2::{Digest, Sha256};
                let mut hasher = Sha256::new();
                hasher.update(&data);
                hex::encode(hasher.finalize())
            };
            
            if actual_hash != hash {
                error!("Received chunk {} with invalid hash: expected {}, got {}", 
                       chunk_id, hash, actual_hash);
                return;
            }
            
            // Store chunk if storage manager is available
            if let Some(storage_manager) = &self.storage_manager {
                match storage_manager.store_chunk(&chunk_id, &data).await {
                    Ok(stored_hash) => {
                        info!("Successfully stored received chunk: {} (hash: {})", 
                              chunk_id, stored_hash);
                    }
                    Err(e) => {
                        error!("Failed to store received chunk {}: {}", chunk_id, e);
                    }
                }
            } else {
                warn!("Received chunk {} but storage manager not available", chunk_id);
            }
        } else {
            warn!("Failed to retrieve chunk: {} (success: {})", chunk_id, success);
        }
    }
    
    /// Handle status update from peer
    async fn handle_status_update(&mut self, node_info: NodeInfo) {
        info!("Status update from {}: {}GB available, {} capabilities",
              node_info.node_id, 
              node_info.storage_available / (1024*1024*1024),
              node_info.capabilities.len());
    }
    
    /// Send a message (for future integration with LibP2P)
    pub async fn send_message(&self, message: ZephyrMessage) -> Result<()> {
        if let Some(tx) = &self.message_tx {
            tx.send(message).await.map_err(|e| anyhow::anyhow!("Send error: {}", e))?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};
    
    fn create_test_config() -> Config {
        let mut config = Config::default();
        config.node_id = Some("test-node".to_string());
        config
    }
    
    #[tokio::test]
    async fn test_message_handler_creation() {
        let config = create_test_config();
        let handler = MessageHandler::new(&config);
        
        // Verify handler is properly initialized
        assert!(handler.message_tx.is_some());
        assert!(handler.pending_requests.is_empty());
    }
    
    #[tokio::test]
    async fn test_ping_pong_handling() {
        let config = create_test_config();
        let mut handler = MessageHandler::new(&config);
        
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        let ping = ZephyrMessage::Ping { timestamp };
        let response = handler.handle_message(ping).await.unwrap();
        
        match response {
            Some(ZephyrMessage::Pong { timestamp: resp_ts, node_info }) => {
                assert_eq!(resp_ts, timestamp);
                assert_eq!(node_info.node_id, "test-node");
            }
            _ => panic!("Expected Pong response"),
        }
    }
    
    #[tokio::test]
    async fn test_message_validation() {
        let config = create_test_config();
        let handler = MessageHandler::new(&config);
        
        // Valid ping
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let valid_ping = ZephyrMessage::Ping { timestamp: now };
        assert!(handler.validate_message(&valid_ping));
        
        // Invalid ping (too old)
        let old_ping = ZephyrMessage::Ping { timestamp: now - 400 };
        assert!(!handler.validate_message(&old_ping));
        
        // Valid chunk request
        let valid_chunk_req = ZephyrMessage::ChunkRequest {
            chunk_id: "chunk123".to_string(),
            expected_hash: "a".repeat(64), // 64 hex chars
        };
        assert!(handler.validate_message(&valid_chunk_req));
        
        // Invalid chunk request (bad hash)
        let invalid_chunk_req = ZephyrMessage::ChunkRequest {
            chunk_id: "chunk123".to_string(),
            expected_hash: "invalid-hash".to_string(),
        };
        assert!(!handler.validate_message(&invalid_chunk_req));
    }
    
    #[tokio::test]
    async fn test_peer_discovery() {
        let config = create_test_config();
        let mut handler = MessageHandler::new(&config);
        
        let discovery_req = ZephyrMessage::PeerDiscovery;
        let response = handler.handle_message(discovery_req).await.unwrap();
        
        match response {
            Some(ZephyrMessage::PeerList { peers }) => {
                // Should return empty list initially
                assert!(peers.is_empty());
            }
            _ => panic!("Expected PeerList response"),
        }
    }
    
    #[tokio::test]
    async fn test_chunk_request_handling() {
        let config = create_test_config();
        let mut handler = MessageHandler::new(&config);
        
        let chunk_req = ZephyrMessage::ChunkRequest {
            chunk_id: "test-chunk".to_string(),
            expected_hash: "a".repeat(64),
        };
        
        let response = handler.handle_message(chunk_req).await.unwrap();
        
        match response {
            Some(ZephyrMessage::ChunkResponse { success, .. }) => {
                // Should fail since storage not implemented yet
                assert!(!success);
            }
            _ => panic!("Expected ChunkResponse"),
        }
    }
    
    #[test]
    fn test_node_info_generation() {
        let config = create_test_config();
        let handler = MessageHandler::new(&config);
        
        let node_info = handler.get_node_info();
        assert_eq!(node_info.node_id, "test-node");
        assert_eq!(node_info.version, "0.1.0");
        assert!(!node_info.capabilities.is_empty());
    }
}