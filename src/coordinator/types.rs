use std::collections::HashMap;

/// Request to register a node with the coordinator
#[derive(Debug, Clone)]
pub struct RegisterNodeRequest {
    pub node_id: String,
    pub addresses: Vec<String>,
    pub storage_capacity: i64,
    pub capabilities: HashMap<String, String>,
}

/// Response from node registration
#[derive(Debug, Clone)]
pub struct RegisterNodeResponse {
    pub success: bool,
    pub message: String,
    pub assigned_node_id: String,
    pub bootstrap_peers: Vec<String>,
}

/// Request to unregister a node
#[derive(Debug, Clone)]
pub struct UnregisterNodeRequest {
    pub node_id: String,
    pub reason: String,
}

/// Response from node unregistration
#[derive(Debug, Clone)]
pub struct UnregisterNodeResponse {
    pub success: bool,
    pub message: String,
}

/// Request to get active nodes
#[derive(Debug, Clone)]
pub struct GetActiveNodesRequest {
    pub limit: i32,
    pub exclude_nodes: Vec<String>,
}

/// Response with active nodes
#[derive(Debug, Clone)]
pub struct GetActiveNodesResponse {
    pub nodes: Vec<NodeStatus>,
    pub total_nodes: i32,
}

/// Node heartbeat request
#[derive(Debug, Clone)]
pub struct NodeHeartbeatRequest {
    pub node_id: String,
    pub stats: Option<NodeStats>,
}

/// Node heartbeat response
#[derive(Debug, Clone)]
pub struct NodeHeartbeatResponse {
    pub success: bool,
    pub message: String,
    pub tasks: Vec<String>,
}

/// Request to register a file
#[derive(Debug, Clone)]
pub struct RegisterFileRequest {
    pub file_id: String,
    pub file_name: String,
    pub file_size: i64,
    pub file_hash: String,
    pub chunks: Vec<ChunkMetadata>,
    pub owner_node_id: String,
}

/// Response from file registration
#[derive(Debug, Clone)]
pub struct RegisterFileResponse {
    pub success: bool,
    pub message: String,
    pub chunk_placements: Vec<ChunkPlacement>,
}

/// Request to get file information
#[derive(Debug, Clone)]
pub struct GetFileInfoRequest {
    pub file_id: String,
}

/// Response with file information
#[derive(Debug, Clone)]
pub struct GetFileInfoResponse {
    pub success: bool,
    pub message: String,
    pub file_info: Option<FileRecord>,
}

/// Request to update chunk locations
#[derive(Debug, Clone)]
pub struct UpdateChunkLocationsRequest {
    pub chunk_id: String,
    pub node_ids: Vec<String>,
    pub operation: String, // "add" or "remove"
}

/// Response from chunk location update
#[derive(Debug, Clone)]
pub struct UpdateChunkLocationsResponse {
    pub success: bool,
    pub message: String,
}

/// Request to find chunk locations
#[derive(Debug, Clone)]
pub struct FindChunkLocationsRequest {
    pub chunk_id: String,
    pub preferred_count: i32,
}

/// Response with chunk locations
#[derive(Debug, Clone)]
pub struct FindChunkLocationsResponse {
    pub success: bool,
    pub message: String,
    pub node_ids: Vec<String>,
    pub node_addresses: Vec<String>,
}

/// Request to get network status
#[derive(Debug, Clone)]
pub struct GetNetworkStatusRequest {}

/// Response with network status
#[derive(Debug, Clone)]
pub struct GetNetworkStatusResponse {
    pub network_stats: Option<NetworkStats>,
    pub active_nodes: Vec<NodeStatus>,
    pub timestamp: i64,
}

/// Node status information
#[derive(Debug, Clone)]
pub struct NodeStatus {
    pub node_id: String,
    pub addresses: Vec<String>,
    pub stats: Option<NodeStats>,
    pub last_heartbeat: i64,
    pub status: String, // "active", "inactive", "maintenance"
}

/// Node statistics
#[derive(Debug, Clone)]
pub struct NodeStats {
    pub storage_used: i64,
    pub storage_available: i64,
    pub chunks_stored: i64,
    pub bandwidth_up: i64,
    pub bandwidth_down: i64,
    pub cpu_usage: f64,
    pub memory_usage: f64,
    pub uptime_seconds: i64,
}

/// Chunk metadata
#[derive(Debug, Clone)]
pub struct ChunkMetadata {
    pub chunk_id: String,
    pub hash: String,
    pub size: i64,
    pub index: i32,
}

/// Chunk placement information
#[derive(Debug, Clone)]
pub struct ChunkPlacement {
    pub chunk_id: String,
    pub target_nodes: Vec<String>,
    pub replication_factor: i32,
}

/// File record information
#[derive(Debug, Clone)]
pub struct FileRecord {
    pub file_id: String,
    pub file_name: String,
    pub file_size: i64,
    pub file_hash: String,
    pub chunks: Vec<ChunkRecord>,
    pub owner_node_id: String,
    pub created_at: i64,
    pub last_accessed: i64,
}

/// Chunk record with location information
#[derive(Debug, Clone)]
pub struct ChunkRecord {
    pub chunk_id: String,
    pub hash: String,
    pub size: i64,
    pub index: i32,
    pub stored_at_nodes: Vec<String>,
    pub replication_count: i32,
}

/// Network statistics
#[derive(Debug, Clone)]
pub struct NetworkStats {
    pub total_nodes: i32,
    pub active_nodes: i32,
    pub total_storage_capacity: i64,
    pub total_storage_used: i64,
    pub total_files: i64,
    pub total_chunks: i64,
    pub average_node_uptime: f64,
    pub network_uptime_seconds: i64,
}