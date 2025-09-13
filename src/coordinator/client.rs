use anyhow::{Result, Context};
use std::time::Duration;
use tonic::transport::{Channel, Endpoint};
use tonic::{Request, Response, Status};
use tracing::{debug, warn};

use super::types::*;

/// Generated gRPC client code
pub mod coordinator_service {
    tonic::include_proto!("zephyrfs.coordinator");
}

use coordinator_service::{
    coordinator_service_client::CoordinatorServiceClient,
    RegisterNodeRequest as ProtoRegisterNodeRequest,
    RegisterNodeResponse as ProtoRegisterNodeResponse,
    UnregisterNodeRequest as ProtoUnregisterNodeRequest,
    UnregisterNodeResponse as ProtoUnregisterNodeResponse,
    GetActiveNodesRequest as ProtoGetActiveNodesRequest,
    GetActiveNodesResponse as ProtoGetActiveNodesResponse,
    NodeHeartbeatRequest as ProtoNodeHeartbeatRequest,
    NodeHeartbeatResponse as ProtoNodeHeartbeatResponse,
    RegisterFileRequest as ProtoRegisterFileRequest,
    RegisterFileResponse as ProtoRegisterFileResponse,
    GetFileInfoRequest as ProtoGetFileInfoRequest,
    GetFileInfoResponse as ProtoGetFileInfoResponse,
    UpdateChunkLocationsRequest as ProtoUpdateChunkLocationsRequest,
    UpdateChunkLocationsResponse as ProtoUpdateChunkLocationsResponse,
    FindChunkLocationsRequest as ProtoFindChunkLocationsRequest,
    FindChunkLocationsResponse as ProtoFindChunkLocationsResponse,
    GetNetworkStatusRequest as ProtoGetNetworkStatusRequest,
    GetNetworkStatusResponse as ProtoGetNetworkStatusResponse,
};

/// Coordinator gRPC client
#[derive(Clone)]
pub struct CoordinatorClient {
    client: CoordinatorServiceClient<Channel>,
}

impl CoordinatorClient {
    /// Create a new coordinator client
    pub async fn new(coordinator_url: &str) -> Result<Self> {
        debug!("Connecting to coordinator at: {}", coordinator_url);

        let endpoint = Endpoint::from_shared(coordinator_url.to_string())
            .context("Invalid coordinator URL")?
            .timeout(Duration::from_secs(10))
            .connect_timeout(Duration::from_secs(5));

        let channel = endpoint.connect().await
            .context("Failed to connect to coordinator")?;

        let client = CoordinatorServiceClient::new(channel);

        debug!("Successfully connected to coordinator");
        Ok(Self { client })
    }

    /// Register node with coordinator
    pub async fn register_node(&self, request: RegisterNodeRequest) -> Result<RegisterNodeResponse> {
        let proto_request = ProtoRegisterNodeRequest {
            node_id: request.node_id,
            addresses: request.addresses,
            storage_capacity: request.storage_capacity,
            capabilities: request.capabilities,
        };

        let response = self.client.clone()
            .register_node(Request::new(proto_request))
            .await
            .context("gRPC call failed")?
            .into_inner();

        Ok(RegisterNodeResponse {
            success: response.success,
            message: response.message,
            assigned_node_id: response.assigned_node_id,
            bootstrap_peers: response.bootstrap_peers,
        })
    }

    /// Unregister node from coordinator
    pub async fn unregister_node(&self, request: UnregisterNodeRequest) -> Result<UnregisterNodeResponse> {
        let proto_request = ProtoUnregisterNodeRequest {
            node_id: request.node_id,
            reason: request.reason,
        };

        let response = self.client.clone()
            .unregister_node(Request::new(proto_request))
            .await
            .context("gRPC call failed")?
            .into_inner();

        Ok(UnregisterNodeResponse {
            success: response.success,
            message: response.message,
        })
    }

    /// Get active nodes from coordinator
    pub async fn get_active_nodes(&self, request: GetActiveNodesRequest) -> Result<GetActiveNodesResponse> {
        let proto_request = ProtoGetActiveNodesRequest {
            limit: request.limit,
            exclude_nodes: request.exclude_nodes,
        };

        let response = self.client.clone()
            .get_active_nodes(Request::new(proto_request))
            .await
            .context("gRPC call failed")?
            .into_inner();

        let nodes = response.nodes.into_iter()
            .map(|node| NodeStatus {
                node_id: node.node_id,
                addresses: node.addresses,
                stats: node.stats.map(|stats| NodeStats {
                    storage_used: stats.storage_used,
                    storage_available: stats.storage_available,
                    chunks_stored: stats.chunks_stored,
                    bandwidth_up: stats.bandwidth_up,
                    bandwidth_down: stats.bandwidth_down,
                    cpu_usage: stats.cpu_usage,
                    memory_usage: stats.memory_usage,
                    uptime_seconds: stats.uptime_seconds,
                }),
                last_heartbeat: node.last_heartbeat,
                status: node.status,
            })
            .collect();

        Ok(GetActiveNodesResponse {
            nodes,
            total_nodes: response.total_nodes,
        })
    }

    /// Send heartbeat to coordinator
    pub async fn node_heartbeat(&self, request: NodeHeartbeatRequest) -> Result<NodeHeartbeatResponse> {
        let proto_stats = request.stats.map(|stats| coordinator_service::NodeStats {
            storage_used: stats.storage_used,
            storage_available: stats.storage_available,
            chunks_stored: stats.chunks_stored,
            bandwidth_up: stats.bandwidth_up,
            bandwidth_down: stats.bandwidth_down,
            cpu_usage: stats.cpu_usage,
            memory_usage: stats.memory_usage,
            uptime_seconds: stats.uptime_seconds,
        });

        let proto_request = ProtoNodeHeartbeatRequest {
            node_id: request.node_id,
            stats: proto_stats,
        };

        let response = self.client.clone()
            .node_heartbeat(Request::new(proto_request))
            .await
            .context("gRPC call failed")?
            .into_inner();

        Ok(NodeHeartbeatResponse {
            success: response.success,
            message: response.message,
            tasks: response.tasks,
        })
    }

    /// Register file with coordinator
    pub async fn register_file(&self, request: RegisterFileRequest) -> Result<RegisterFileResponse> {
        let proto_chunks = request.chunks.into_iter()
            .map(|chunk| coordinator_service::ChunkMetadata {
                chunk_id: chunk.chunk_id,
                hash: chunk.hash,
                size: chunk.size,
                index: chunk.index,
            })
            .collect();

        let proto_request = ProtoRegisterFileRequest {
            file_id: request.file_id,
            file_name: request.file_name,
            file_size: request.file_size,
            file_hash: request.file_hash,
            chunks: proto_chunks,
            owner_node_id: request.owner_node_id,
        };

        let response = self.client.clone()
            .register_file(Request::new(proto_request))
            .await
            .context("gRPC call failed")?
            .into_inner();

        let chunk_placements = response.chunk_placements.into_iter()
            .map(|placement| ChunkPlacement {
                chunk_id: placement.chunk_id,
                target_nodes: placement.target_nodes,
                replication_factor: placement.replication_factor,
            })
            .collect();

        Ok(RegisterFileResponse {
            success: response.success,
            message: response.message,
            chunk_placements,
        })
    }

    /// Get file info from coordinator
    pub async fn get_file_info(&self, request: GetFileInfoRequest) -> Result<GetFileInfoResponse> {
        let proto_request = ProtoGetFileInfoRequest {
            file_id: request.file_id,
        };

        let response = self.client.clone()
            .get_file_info(Request::new(proto_request))
            .await
            .context("gRPC call failed")?
            .into_inner();

        let file_info = response.file_info.map(|info| FileRecord {
            file_id: info.file_id,
            file_name: info.file_name,
            file_size: info.file_size,
            file_hash: info.file_hash,
            chunks: info.chunks.into_iter()
                .map(|chunk| ChunkRecord {
                    chunk_id: chunk.chunk_id,
                    hash: chunk.hash,
                    size: chunk.size,
                    index: chunk.index,
                    stored_at_nodes: chunk.stored_at_nodes,
                    replication_count: chunk.replication_count,
                })
                .collect(),
            owner_node_id: info.owner_node_id,
            created_at: info.created_at,
            last_accessed: info.last_accessed,
        });

        Ok(GetFileInfoResponse {
            success: response.success,
            message: response.message,
            file_info,
        })
    }

    /// Update chunk locations
    pub async fn update_chunk_locations(&self, request: UpdateChunkLocationsRequest) -> Result<UpdateChunkLocationsResponse> {
        let proto_request = ProtoUpdateChunkLocationsRequest {
            chunk_id: request.chunk_id,
            node_ids: request.node_ids,
            operation: request.operation,
        };

        let response = self.client.clone()
            .update_chunk_locations(Request::new(proto_request))
            .await
            .context("gRPC call failed")?
            .into_inner();

        Ok(UpdateChunkLocationsResponse {
            success: response.success,
            message: response.message,
        })
    }

    /// Find chunk locations
    pub async fn find_chunk_locations(&self, request: FindChunkLocationsRequest) -> Result<FindChunkLocationsResponse> {
        let proto_request = ProtoFindChunkLocationsRequest {
            chunk_id: request.chunk_id,
            preferred_count: request.preferred_count,
        };

        let response = self.client.clone()
            .find_chunk_locations(Request::new(proto_request))
            .await
            .context("gRPC call failed")?
            .into_inner();

        Ok(FindChunkLocationsResponse {
            success: response.success,
            message: response.message,
            node_ids: response.node_ids,
            node_addresses: response.node_addresses,
        })
    }

    /// Get network status
    pub async fn get_network_status(&self, request: GetNetworkStatusRequest) -> Result<GetNetworkStatusResponse> {
        let proto_request = ProtoGetNetworkStatusRequest {};

        let response = self.client.clone()
            .get_network_status(Request::new(proto_request))
            .await
            .context("gRPC call failed")?
            .into_inner();

        let network_stats = response.network_stats.map(|stats| NetworkStats {
            total_nodes: stats.total_nodes,
            active_nodes: stats.active_nodes,
            total_storage_capacity: stats.total_storage_capacity,
            total_storage_used: stats.total_storage_used,
            total_files: stats.total_files,
            total_chunks: stats.total_chunks,
            average_node_uptime: stats.average_node_uptime,
            network_uptime_seconds: stats.network_uptime_seconds,
        });

        let active_nodes = response.active_nodes.into_iter()
            .map(|node| NodeStatus {
                node_id: node.node_id,
                addresses: node.addresses,
                stats: node.stats.map(|stats| NodeStats {
                    storage_used: stats.storage_used,
                    storage_available: stats.storage_available,
                    chunks_stored: stats.chunks_stored,
                    bandwidth_up: stats.bandwidth_up,
                    bandwidth_down: stats.bandwidth_down,
                    cpu_usage: stats.cpu_usage,
                    memory_usage: stats.memory_usage,
                    uptime_seconds: stats.uptime_seconds,
                }),
                last_heartbeat: node.last_heartbeat,
                status: node.status,
            })
            .collect();

        Ok(GetNetworkStatusResponse {
            network_stats,
            active_nodes,
            timestamp: response.timestamp,
        })
    }
}