use anyhow::{Result, Context};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::time::{interval, sleep};
use tracing::{debug, info, warn, error};
use uuid::Uuid;

pub mod client;
pub mod types;

pub use client::CoordinatorClient;
pub use types::*;

/// Coordinator integration for node registration and coordination
pub struct CoordinatorManager {
    client: CoordinatorClient,
    node_id: String,
    coordinator_url: String,
    heartbeat_interval: Duration,
    registration_status: RegistrationStatus,
}

#[derive(Debug, Clone)]
pub enum RegistrationStatus {
    NotRegistered,
    Registering,
    Registered,
    Failed(String),
}

impl CoordinatorManager {
    /// Create a new coordinator manager
    pub async fn new(coordinator_url: String) -> Result<Self> {
        let client = CoordinatorClient::new(&coordinator_url).await
            .context("Failed to create coordinator client")?;

        let node_id = Uuid::new_v4().to_string();
        let heartbeat_interval = Duration::from_secs(10);

        Ok(Self {
            client,
            node_id,
            coordinator_url,
            heartbeat_interval,
            registration_status: RegistrationStatus::NotRegistered,
        })
    }

    /// Register this node with the coordinator
    pub async fn register_node(
        &mut self,
        addresses: Vec<String>,
        storage_capacity: u64,
        capabilities: std::collections::HashMap<String, String>,
    ) -> Result<RegisterNodeResponse> {
        info!("Registering node {} with coordinator at {}", self.node_id, self.coordinator_url);
        self.registration_status = RegistrationStatus::Registering;

        let request = RegisterNodeRequest {
            node_id: self.node_id.clone(),
            addresses,
            storage_capacity: storage_capacity as i64,
            capabilities,
        };

        match self.client.register_node(request).await {
            Ok(response) => {
                if response.success {
                    self.registration_status = RegistrationStatus::Registered;
                    info!("Successfully registered with coordinator. Assigned ID: {}", response.assigned_node_id);

                    // Update node ID if coordinator assigned a different one
                    if !response.assigned_node_id.is_empty() {
                        self.node_id = response.assigned_node_id.clone();
                    }

                    let success_response = RegisterNodeResponse {
                        success: response.success,
                        message: response.message,
                        assigned_node_id: response.assigned_node_id,
                        bootstrap_peers: response.bootstrap_peers,
                    };
                    Ok(success_response)
                } else {
                    let error_msg = format!("Registration failed: {}", response.message);
                    self.registration_status = RegistrationStatus::Failed(error_msg.clone());
                    warn!("{}", error_msg);
                    Ok(response)
                }
            }
            Err(e) => {
                let error_msg = format!("Failed to register with coordinator: {}", e);
                self.registration_status = RegistrationStatus::Failed(error_msg.clone());
                error!("{}", error_msg);
                Err(e)
            }
        }
    }

    /// Start heartbeat loop
    pub async fn start_heartbeat(&self, stats_provider: impl Fn() -> NodeStats + Send + 'static) {
        let client = self.client.clone();
        let node_id = self.node_id.clone();
        let heartbeat_interval = self.heartbeat_interval;

        tokio::spawn(async move {
            let mut interval = interval(heartbeat_interval);

            loop {
                interval.tick().await;

                let stats = stats_provider();
                let request = NodeHeartbeatRequest {
                    node_id: node_id.clone(),
                    stats: Some(stats),
                };

                match client.node_heartbeat(request).await {
                    Ok(response) => {
                        if response.success {
                            debug!("Heartbeat sent successfully");
                            if !response.tasks.is_empty() {
                                debug!("Coordinator assigned {} tasks", response.tasks.len());
                                // TODO: Handle assigned tasks
                            }
                        } else {
                            warn!("Heartbeat failed: {}", response.message);
                        }
                    }
                    Err(e) => {
                        warn!("Failed to send heartbeat: {}", e);
                        // TODO: Implement exponential backoff
                        sleep(Duration::from_secs(5)).await;
                    }
                }
            }
        });
    }

    /// Register a file with the coordinator
    pub async fn register_file(
        &self,
        file_id: String,
        file_name: String,
        file_size: u64,
        file_hash: String,
        chunks: Vec<ChunkMetadata>,
    ) -> Result<RegisterFileResponse> {
        debug!("Registering file {} with coordinator", file_id);

        let request = RegisterFileRequest {
            file_id,
            file_name,
            file_size: file_size as i64,
            file_hash,
            chunks,
            owner_node_id: self.node_id.clone(),
        };

        self.client.register_file(request).await
            .context("Failed to register file with coordinator")
    }

    /// Find chunk locations from coordinator
    pub async fn find_chunk_locations(&self, chunk_id: String, preferred_count: i32) -> Result<FindChunkLocationsResponse> {
        debug!("Finding locations for chunk {} from coordinator", chunk_id);

        let request = FindChunkLocationsRequest {
            chunk_id,
            preferred_count,
        };

        self.client.find_chunk_locations(request).await
            .context("Failed to find chunk locations from coordinator")
    }

    /// Get active nodes from coordinator
    pub async fn get_active_nodes(&self, limit: Option<i32>, exclude_nodes: Vec<String>) -> Result<GetActiveNodesResponse> {
        debug!("Getting active nodes from coordinator");

        let request = GetActiveNodesRequest {
            limit: limit.unwrap_or(10),
            exclude_nodes,
        };

        self.client.get_active_nodes(request).await
            .context("Failed to get active nodes from coordinator")
    }

    /// Get network status from coordinator
    pub async fn get_network_status(&self) -> Result<GetNetworkStatusResponse> {
        debug!("Getting network status from coordinator");

        let request = GetNetworkStatusRequest {};

        self.client.get_network_status(request).await
            .context("Failed to get network status from coordinator")
    }

    /// Update chunk locations
    pub async fn update_chunk_locations(
        &self,
        chunk_id: String,
        node_ids: Vec<String>,
        operation: String,
    ) -> Result<UpdateChunkLocationsResponse> {
        debug!("Updating chunk locations for {} (operation: {})", chunk_id, operation);

        let request = UpdateChunkLocationsRequest {
            chunk_id,
            node_ids,
            operation,
        };

        self.client.update_chunk_locations(request).await
            .context("Failed to update chunk locations")
    }

    /// Unregister this node from coordinator
    pub async fn unregister_node(&mut self, reason: Option<String>) -> Result<UnregisterNodeResponse> {
        info!("Unregistering node {} from coordinator", self.node_id);

        let request = UnregisterNodeRequest {
            node_id: self.node_id.clone(),
            reason: reason.unwrap_or_else(|| "Normal shutdown".to_string()),
        };

        match self.client.unregister_node(request).await {
            Ok(response) => {
                if response.success {
                    self.registration_status = RegistrationStatus::NotRegistered;
                    info!("Successfully unregistered from coordinator");
                } else {
                    warn!("Unregistration failed: {}", response.message);
                }
                Ok(response)
            }
            Err(e) => {
                error!("Failed to unregister from coordinator: {}", e);
                Err(e)
            }
        }
    }

    /// Get current registration status
    pub fn get_registration_status(&self) -> &RegistrationStatus {
        &self.registration_status
    }

    /// Get node ID
    pub fn get_node_id(&self) -> &str {
        &self.node_id
    }
}

/// Convert system time to Unix timestamp
pub fn system_time_to_unix_timestamp(time: SystemTime) -> i64 {
    time.duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}