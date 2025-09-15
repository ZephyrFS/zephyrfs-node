//! Automatic Replication System
//!
//! Handles automatic replication when nodes go offline, ensuring data durability
//! through intelligent recovery and replacement strategies

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque, HashSet};
use chrono::{DateTime, Utc, Duration};
use tokio::time::{sleep, Duration as TokioDuration};

use crate::economics::earnings_calculator::GeographicRegion;
use super::health_monitor::{ChunkHealth, ReplicaHealth, ReplicaStatus, HealthStatus};
use super::intelligent_replication::{ReplicationStrategy, ContentType};

/// Automatic replication manager
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoReplicationManager {
    /// Node failure detection
    pub failure_detection: FailureDetectionConfig,
    /// Replication policies
    pub replication_policies: HashMap<String, AutoReplicationPolicy>,
    /// Active replication tasks
    pub active_tasks: HashMap<String, ReplicationTask>,
    /// Node status tracking
    pub node_status: HashMap<String, NodeStatus>,
    /// Recovery strategies
    pub recovery_strategies: RecoveryStrategyConfig,
    /// Performance metrics
    pub metrics: AutoReplicationMetrics,
    /// Emergency protocols
    pub emergency_config: EmergencyReplicationConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailureDetectionConfig {
    /// Heartbeat interval for node monitoring
    pub heartbeat_interval_seconds: u32,
    /// Timeout before considering node offline
    pub offline_timeout_seconds: u32,
    /// Number of consecutive failures before triggering replication
    pub failure_threshold: u32,
    /// Grace period for temporary network issues
    pub grace_period_seconds: u32,
    /// Enable predictive failure detection
    pub predictive_detection: bool,
    /// Network partition detection
    pub partition_detection: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoReplicationPolicy {
    pub policy_id: String,
    pub content_types: Vec<ContentType>,
    pub trigger_conditions: Vec<TriggerCondition>,
    pub replication_strategy: ReplicationResponseStrategy,
    pub priority: ReplicationPriority,
    pub max_concurrent_replications: u32,
    pub resource_limits: ResourceLimits,
    pub geographic_constraints: GeographicConstraints,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TriggerCondition {
    NodeOffline { grace_period_seconds: u32 },
    ReplicaCountBelowThreshold { min_replicas: u32 },
    HealthScoreBelowThreshold { min_score: f64 },
    GeographicDistributionLoss { min_regions: u32 },
    PerformanceDegradation { max_response_time_ms: f64 },
    IntegrityViolation,
    NetworkPartition,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReplicationResponseStrategy {
    Immediate,           // Start replication immediately
    Delayed { delay_seconds: u32 }, // Wait before starting
    Batched { batch_size: u32 },    // Batch multiple chunks
    Adaptive,            // Adapt based on network conditions
    Conservative,        // Wait for confirmation of permanent failure
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReplicationPriority {
    Emergency,   // Critical data, immediate action
    High,        // Important data, prioritized
    Normal,      // Standard priority
    Low,         // Background processing
    Deferred,    // Wait for better conditions
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    pub max_bandwidth_mbps: f64,
    pub max_concurrent_transfers: u32,
    pub max_storage_usage_gb: u64,
    pub max_cost_per_hour: f64,
    pub cpu_usage_limit: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeographicConstraints {
    pub required_regions: Vec<GeographicRegion>,
    pub forbidden_regions: Vec<GeographicRegion>,
    pub min_distance_km: f64,
    pub regulatory_compliance: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeStatus {
    pub node_id: String,
    pub region: GeographicRegion,
    pub status: NodeState,
    pub last_seen: DateTime<Utc>,
    pub consecutive_failures: u32,
    pub failure_history: VecDeque<FailureEvent>,
    pub predicted_availability: f64,
    pub maintenance_scheduled: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NodeState {
    Online,
    Degraded,
    Offline,
    Maintenance,
    Unknown,
    Suspected,  // Suspected of being offline
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailureEvent {
    pub timestamp: DateTime<Utc>,
    pub failure_type: FailureType,
    pub duration_seconds: Option<u64>,
    pub recovery_time_seconds: Option<u64>,
    pub root_cause: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FailureType {
    NetworkTimeout,
    DiskFailure,
    PowerOutage,
    MaintenanceShutdown,
    ProcessCrash,
    NetworkPartition,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplicationTask {
    pub task_id: String,
    pub chunk_id: String,
    pub content_type: ContentType,
    pub trigger_reason: TriggerReason,
    pub source_replicas: Vec<String>,
    pub target_nodes: Vec<String>,
    pub status: TaskStatus,
    pub progress: TaskProgress,
    pub created_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub estimated_completion: Option<DateTime<Utc>>,
    pub resource_usage: ResourceUsage,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TriggerReason {
    NodeFailure { failed_nodes: Vec<String> },
    HealthDegradation { health_score: f64 },
    PolicyViolation { policy_id: String },
    ManualRequest { requested_by: String },
    PredictiveAction { predicted_failure: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskStatus {
    Queued,
    Planning,
    Executing,
    Verifying,
    Completed,
    Failed,
    Cancelled,
    Paused,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskProgress {
    pub stage: ReplicationStage,
    pub percentage_complete: f64,
    pub bytes_transferred: u64,
    pub total_bytes: u64,
    pub transfer_rate_mbps: f64,
    pub estimated_time_remaining_seconds: u64,
    pub current_operation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReplicationStage {
    Initializing,
    SelectingNodes,
    PreparingTransfer,
    Transferring,
    Verifying,
    Finalizing,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUsage {
    pub bandwidth_used_mbps: f64,
    pub storage_used_gb: u64,
    pub cpu_usage_percent: f64,
    pub cost_incurred: f64,
    pub network_transfers: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryStrategyConfig {
    pub prefer_local_replicas: bool,
    pub max_recovery_distance_km: f64,
    pub parallel_recovery_streams: u32,
    pub verification_level: VerificationLevel,
    pub fallback_strategies: Vec<FallbackStrategy>,
    pub optimization_goals: OptimizationGoals,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VerificationLevel {
    Basic,       // Hash verification only
    Standard,    // Hash + size + basic integrity
    Thorough,    // Full content verification
    Paranoid,    // Multiple verification methods
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FallbackStrategy {
    UseRemoteReplicas,
    IncreaseReplicationFactor,
    RelaxGeographicConstraints,
    UseExpensiveNodes,
    WaitForNodeRecovery,
    EmergencyProtocol,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationGoals {
    pub minimize_cost: bool,
    pub minimize_latency: bool,
    pub maximize_durability: bool,
    pub balance_geographic_distribution: bool,
    pub prefer_high_performance_nodes: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoReplicationMetrics {
    pub total_replications_triggered: u64,
    pub successful_replications: u64,
    pub failed_replications: u64,
    pub average_replication_time_seconds: f64,
    pub total_data_recovered_gb: u64,
    pub cost_of_replications: f64,
    pub nodes_replaced: u64,
    pub emergency_recoveries: u64,
    pub last_updated: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmergencyReplicationConfig {
    pub enable_emergency_mode: bool,
    pub emergency_triggers: Vec<EmergencyTrigger>,
    pub emergency_resources: EmergencyResources,
    pub escalation_timeouts: EscalationTimeouts,
    pub emergency_contacts: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EmergencyTrigger {
    DataLossImminent { chunks_at_risk: u32 },
    NetworkPartition { partition_size: f64 },
    MassNodeFailure { failure_rate: f64 },
    StorageCapacityCritical { utilization: f64 },
    ComplianceViolation { severity: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmergencyResources {
    pub reserved_bandwidth_mbps: f64,
    pub reserved_storage_gb: u64,
    pub priority_node_access: bool,
    pub cost_override_enabled: bool,
    pub geographic_restriction_override: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EscalationTimeouts {
    pub initial_response_minutes: u32,
    pub escalation_interval_minutes: u32,
    pub max_escalation_levels: u32,
    pub emergency_override_minutes: u32,
}

impl Default for FailureDetectionConfig {
    fn default() -> Self {
        Self {
            heartbeat_interval_seconds: 30,
            offline_timeout_seconds: 180, // 3 minutes
            failure_threshold: 3,
            grace_period_seconds: 60,
            predictive_detection: true,
            partition_detection: true,
        }
    }
}

impl AutoReplicationManager {
    /// Create new auto replication manager
    pub fn new() -> Self {
        let mut manager = Self {
            failure_detection: FailureDetectionConfig::default(),
            replication_policies: HashMap::new(),
            active_tasks: HashMap::new(),
            node_status: HashMap::new(),
            recovery_strategies: RecoveryStrategyConfig {
                prefer_local_replicas: true,
                max_recovery_distance_km: 5000.0,
                parallel_recovery_streams: 3,
                verification_level: VerificationLevel::Standard,
                fallback_strategies: vec![
                    FallbackStrategy::UseRemoteReplicas,
                    FallbackStrategy::RelaxGeographicConstraints,
                    FallbackStrategy::IncreaseReplicationFactor,
                ],
                optimization_goals: OptimizationGoals {
                    minimize_cost: true,
                    minimize_latency: true,
                    maximize_durability: true,
                    balance_geographic_distribution: true,
                    prefer_high_performance_nodes: false,
                },
            },
            metrics: AutoReplicationMetrics {
                total_replications_triggered: 0,
                successful_replications: 0,
                failed_replications: 0,
                average_replication_time_seconds: 0.0,
                total_data_recovered_gb: 0,
                cost_of_replications: 0.0,
                nodes_replaced: 0,
                emergency_recoveries: 0,
                last_updated: Utc::now(),
            },
            emergency_config: EmergencyReplicationConfig {
                enable_emergency_mode: true,
                emergency_triggers: vec![
                    EmergencyTrigger::DataLossImminent { chunks_at_risk: 10 },
                    EmergencyTrigger::MassNodeFailure { failure_rate: 0.1 },
                ],
                emergency_resources: EmergencyResources {
                    reserved_bandwidth_mbps: 100.0,
                    reserved_storage_gb: 1000,
                    priority_node_access: true,
                    cost_override_enabled: true,
                    geographic_restriction_override: false,
                },
                escalation_timeouts: EscalationTimeouts {
                    initial_response_minutes: 5,
                    escalation_interval_minutes: 15,
                    max_escalation_levels: 3,
                    emergency_override_minutes: 60,
                },
                emergency_contacts: Vec::new(),
            },
        };

        manager.initialize_default_policies();
        manager
    }

    /// Initialize default replication policies
    fn initialize_default_policies(&mut self) {
        // Critical data policy
        self.replication_policies.insert("critical".to_string(), AutoReplicationPolicy {
            policy_id: "critical".to_string(),
            content_types: vec![ContentType::Critical],
            trigger_conditions: vec![
                TriggerCondition::NodeOffline { grace_period_seconds: 30 },
                TriggerCondition::ReplicaCountBelowThreshold { min_replicas: 5 },
                TriggerCondition::HealthScoreBelowThreshold { min_score: 80.0 },
            ],
            replication_strategy: ReplicationResponseStrategy::Immediate,
            priority: ReplicationPriority::Emergency,
            max_concurrent_replications: 10,
            resource_limits: ResourceLimits {
                max_bandwidth_mbps: 1000.0,
                max_concurrent_transfers: 20,
                max_storage_usage_gb: 10000,
                max_cost_per_hour: 100.0,
                cpu_usage_limit: 80.0,
            },
            geographic_constraints: GeographicConstraints {
                required_regions: vec![],
                forbidden_regions: vec![],
                min_distance_km: 1000.0,
                regulatory_compliance: vec!["SOX".to_string(), "HIPAA".to_string()],
            },
        });

        // Standard data policy
        self.replication_policies.insert("standard".to_string(), AutoReplicationPolicy {
            policy_id: "standard".to_string(),
            content_types: vec![ContentType::Standard, ContentType::Important],
            trigger_conditions: vec![
                TriggerCondition::NodeOffline { grace_period_seconds: 120 },
                TriggerCondition::ReplicaCountBelowThreshold { min_replicas: 3 },
                TriggerCondition::HealthScoreBelowThreshold { min_score: 70.0 },
            ],
            replication_strategy: ReplicationResponseStrategy::Delayed { delay_seconds: 300 },
            priority: ReplicationPriority::Normal,
            max_concurrent_replications: 5,
            resource_limits: ResourceLimits {
                max_bandwidth_mbps: 500.0,
                max_concurrent_transfers: 10,
                max_storage_usage_gb: 5000,
                max_cost_per_hour: 50.0,
                cpu_usage_limit: 60.0,
            },
            geographic_constraints: GeographicConstraints {
                required_regions: vec![],
                forbidden_regions: vec![],
                min_distance_km: 500.0,
                regulatory_compliance: vec![],
            },
        });

        // Archive data policy
        self.replication_policies.insert("archive".to_string(), AutoReplicationPolicy {
            policy_id: "archive".to_string(),
            content_types: vec![ContentType::Archive],
            trigger_conditions: vec![
                TriggerCondition::NodeOffline { grace_period_seconds: 3600 }, // 1 hour
                TriggerCondition::ReplicaCountBelowThreshold { min_replicas: 2 },
            ],
            replication_strategy: ReplicationResponseStrategy::Conservative,
            priority: ReplicationPriority::Low,
            max_concurrent_replications: 2,
            resource_limits: ResourceLimits {
                max_bandwidth_mbps: 100.0,
                max_concurrent_transfers: 3,
                max_storage_usage_gb: 1000,
                max_cost_per_hour: 10.0,
                cpu_usage_limit: 30.0,
            },
            geographic_constraints: GeographicConstraints {
                required_regions: vec![],
                forbidden_regions: vec![],
                min_distance_km: 100.0,
                regulatory_compliance: vec![],
            },
        });
    }

    /// Update node status
    pub fn update_node_status(&mut self, node_id: String, status: NodeState) {
        let now = Utc::now();

        if let Some(node_status) = self.node_status.get_mut(&node_id) {
            let previous_status = node_status.status.clone();
            node_status.status = status.clone();
            node_status.last_seen = now;

            // Track state transitions
            if !matches!(previous_status, NodeState::Online) && matches!(status, NodeState::Online) {
                // Node came back online
                node_status.consecutive_failures = 0;
                tracing::info!("Node {} came back online", node_id);
            } else if matches!(previous_status, NodeState::Online) && !matches!(status, NodeState::Online) {
                // Node went offline
                node_status.consecutive_failures += 1;

                node_status.failure_history.push_back(FailureEvent {
                    timestamp: now,
                    failure_type: FailureType::NetworkTimeout, // Default, would be determined by failure detection
                    duration_seconds: None,
                    recovery_time_seconds: None,
                    root_cause: None,
                });

                // Keep only last 100 failure events
                if node_status.failure_history.len() > 100 {
                    node_status.failure_history.pop_front();
                }

                tracing::warn!("Node {} went offline (failure #{}) ", node_id, node_status.consecutive_failures);
            }
        } else {
            // New node
            self.node_status.insert(node_id.clone(), NodeStatus {
                node_id: node_id.clone(),
                region: GeographicRegion::NorthAmerica, // Would be determined from node info
                status,
                last_seen: now,
                consecutive_failures: 0,
                failure_history: VecDeque::new(),
                predicted_availability: 1.0,
                maintenance_scheduled: None,
            });
        }
    }

    /// Detect node failures and trigger replication
    pub async fn detect_failures_and_replicate(&mut self, chunk_health_data: &HashMap<String, ChunkHealth>) -> Result<()> {
        let failed_nodes = self.detect_failed_nodes();

        if !failed_nodes.is_empty() {
            tracing::info!("Detected {} failed nodes: {:?}", failed_nodes.len(), failed_nodes);

            // Find affected chunks
            let affected_chunks = self.find_affected_chunks(chunk_health_data, &failed_nodes);

            for chunk_id in affected_chunks {
                if let Some(chunk_health) = chunk_health_data.get(&chunk_id) {
                    // Check if replication is needed
                    if self.should_trigger_replication(chunk_health, &failed_nodes)? {
                        self.trigger_replication(chunk_id, chunk_health, &failed_nodes).await?;
                    }
                }
            }
        }

        // Check for other trigger conditions
        self.check_other_trigger_conditions(chunk_health_data).await?;

        Ok(())
    }

    /// Detect failed nodes
    fn detect_failed_nodes(&self) -> Vec<String> {
        let now = Utc::now();
        let offline_threshold = Duration::seconds(self.failure_detection.offline_timeout_seconds as i64);

        self.node_status
            .iter()
            .filter(|(_, status)| {
                matches!(status.status, NodeState::Offline | NodeState::Unknown) ||
                (now - status.last_seen) > offline_threshold
            })
            .map(|(node_id, _)| node_id.clone())
            .collect()
    }

    /// Find chunks affected by node failures
    fn find_affected_chunks(&self, chunk_health_data: &HashMap<String, ChunkHealth>, failed_nodes: &[String]) -> Vec<String> {
        let failed_node_set: HashSet<_> = failed_nodes.iter().collect();

        chunk_health_data
            .iter()
            .filter(|(_, chunk_health)| {
                chunk_health.replica_health
                    .iter()
                    .any(|replica| failed_node_set.contains(&replica.node_id))
            })
            .map(|(chunk_id, _)| chunk_id.clone())
            .collect()
    }

    /// Check if replication should be triggered
    fn should_trigger_replication(&self, chunk_health: &ChunkHealth, failed_nodes: &[String]) -> Result<bool> {
        let content_type = ContentType::Standard; // Would be determined from chunk metadata

        // Find applicable policy
        let policy = self.find_applicable_policy(&content_type)?;

        // Check each trigger condition
        for condition in &policy.trigger_conditions {
            match condition {
                TriggerCondition::NodeOffline { grace_period_seconds } => {
                    // Check if any replica is on a failed node and grace period has passed
                    let affected_replicas = chunk_health.replica_health
                        .iter()
                        .filter(|replica| failed_nodes.contains(&replica.node_id))
                        .count();

                    if affected_replicas > 0 {
                        let grace_period = Duration::seconds(*grace_period_seconds as i64);
                        let oldest_failure = Utc::now() - grace_period; // Simplified

                        // In real implementation, would check actual failure times
                        return Ok(true);
                    }
                },
                TriggerCondition::ReplicaCountBelowThreshold { min_replicas } => {
                    let healthy_replicas = chunk_health.replica_health
                        .iter()
                        .filter(|replica| matches!(replica.status, ReplicaStatus::Healthy))
                        .count();

                    if healthy_replicas < *min_replicas as usize {
                        return Ok(true);
                    }
                },
                TriggerCondition::HealthScoreBelowThreshold { min_score } => {
                    if chunk_health.availability_score < *min_score {
                        return Ok(true);
                    }
                },
                TriggerCondition::GeographicDistributionLoss { min_regions } => {
                    let regions: HashSet<_> = chunk_health.replica_health
                        .iter()
                        .filter(|replica| matches!(replica.status, ReplicaStatus::Healthy))
                        .map(|replica| &replica.region)
                        .collect();

                    if regions.len() < *min_regions as usize {
                        return Ok(true);
                    }
                },
                _ => {
                    // Handle other conditions
                }
            }
        }

        Ok(false)
    }

    /// Find applicable replication policy
    fn find_applicable_policy(&self, content_type: &ContentType) -> Result<&AutoReplicationPolicy> {
        for policy in self.replication_policies.values() {
            if policy.content_types.contains(content_type) {
                return Ok(policy);
            }
        }

        // Default to standard policy
        self.replication_policies.get("standard")
            .ok_or_else(|| anyhow::anyhow!("No applicable replication policy found"))
    }

    /// Trigger replication for a chunk
    pub async fn trigger_replication(
        &mut self,
        chunk_id: String,
        chunk_health: &ChunkHealth,
        failed_nodes: &[String],
    ) -> Result<String> {
        let content_type = ContentType::Standard; // Would be determined from chunk metadata
        let policy = self.find_applicable_policy(&content_type)?;

        // Check if we're already replicating this chunk
        let existing_task = self.active_tasks.values()
            .find(|task| task.chunk_id == chunk_id && matches!(task.status, TaskStatus::Queued | TaskStatus::Executing));

        if existing_task.is_some() {
            tracing::info!("Replication already in progress for chunk {}", chunk_id);
            return Ok(existing_task.unwrap().task_id.clone());
        }

        // Check resource limits
        if self.active_tasks.len() >= policy.max_concurrent_replications as usize {
            tracing::warn!("Maximum concurrent replications reached, queuing chunk {}", chunk_id);
        }

        // Create replication task
        let task_id = format!("repl_{}_{}", chunk_id, Utc::now().timestamp());
        let task = ReplicationTask {
            task_id: task_id.clone(),
            chunk_id: chunk_id.clone(),
            content_type,
            trigger_reason: TriggerReason::NodeFailure {
                failed_nodes: failed_nodes.to_vec(),
            },
            source_replicas: chunk_health.replica_health
                .iter()
                .filter(|replica| matches!(replica.status, ReplicaStatus::Healthy))
                .map(|replica| replica.replica_id.clone())
                .collect(),
            target_nodes: Vec::new(), // Will be determined during planning
            status: TaskStatus::Queued,
            progress: TaskProgress {
                stage: ReplicationStage::Initializing,
                percentage_complete: 0.0,
                bytes_transferred: 0,
                total_bytes: 0, // Would be determined from chunk size
                transfer_rate_mbps: 0.0,
                estimated_time_remaining_seconds: 0,
                current_operation: "Queued for replication".to_string(),
            },
            created_at: Utc::now(),
            started_at: None,
            completed_at: None,
            estimated_completion: None,
            resource_usage: ResourceUsage {
                bandwidth_used_mbps: 0.0,
                storage_used_gb: 0,
                cpu_usage_percent: 0.0,
                cost_incurred: 0.0,
                network_transfers: 0,
            },
        };

        self.active_tasks.insert(task_id.clone(), task);
        self.metrics.total_replications_triggered += 1;

        tracing::info!("Triggered replication for chunk {} (task: {})", chunk_id, task_id);

        // Schedule task execution based on policy strategy
        match &policy.replication_strategy {
            ReplicationResponseStrategy::Immediate => {
                self.execute_replication_task(task_id.clone()).await?;
            },
            ReplicationResponseStrategy::Delayed { delay_seconds } => {
                // In real implementation, would schedule with delay
                tokio::spawn(async move {
                    sleep(TokioDuration::from_secs(*delay_seconds as u64)).await;
                    // Execute task after delay
                });
            },
            _ => {
                // Handle other strategies
            }
        }

        Ok(task_id)
    }

    /// Execute replication task
    async fn execute_replication_task(&mut self, task_id: String) -> Result<()> {
        let task = self.active_tasks.get_mut(&task_id)
            .ok_or_else(|| anyhow::anyhow!("Replication task not found"))?;

        task.status = TaskStatus::Planning;
        task.started_at = Some(Utc::now());
        task.progress.stage = ReplicationStage::SelectingNodes;
        task.progress.current_operation = "Selecting target nodes".to_string();

        tracing::info!("Executing replication task {}", task_id);

        // Select target nodes
        let target_nodes = self.select_target_nodes(&task.chunk_id, &task.content_type).await?;

        if target_nodes.is_empty() {
            task.status = TaskStatus::Failed;
            self.metrics.failed_replications += 1;
            return Err(anyhow::anyhow!("No suitable target nodes found"));
        }

        task.target_nodes = target_nodes;
        task.status = TaskStatus::Executing;
        task.progress.stage = ReplicationStage::Transferring;

        // Execute the actual replication
        self.perform_replication(&task_id).await?;

        Ok(())
    }

    /// Select target nodes for replication
    async fn select_target_nodes(&self, chunk_id: &str, content_type: &ContentType) -> Result<Vec<String>> {
        // Simplified node selection - in real implementation would use
        // the intelligent replication manager
        let available_nodes: Vec<String> = self.node_status
            .iter()
            .filter(|(_, status)| matches!(status.status, NodeState::Online))
            .map(|(node_id, _)| node_id.clone())
            .take(2) // Select 2 replacement nodes
            .collect();

        Ok(available_nodes)
    }

    /// Perform the actual replication
    async fn perform_replication(&mut self, task_id: &str) -> Result<()> {
        let task = self.active_tasks.get_mut(task_id)
            .ok_or_else(|| anyhow::anyhow!("Task not found"))?;

        // Simulate replication process
        task.progress.current_operation = "Transferring data".to_string();
        task.progress.total_bytes = 1_000_000_000; // 1GB example

        // Simulate transfer progress
        for i in 0..=10 {
            task.progress.percentage_complete = i as f64 * 10.0;
            task.progress.bytes_transferred = (task.progress.total_bytes as f64 * (i as f64 / 10.0)) as u64;
            task.progress.current_operation = format!("Transferring chunk data ({:.0}%)", task.progress.percentage_complete);

            // Simulate transfer time
            sleep(TokioDuration::from_millis(100)).await;
        }

        // Verification stage
        task.progress.stage = ReplicationStage::Verifying;
        task.progress.current_operation = "Verifying replica integrity".to_string();
        sleep(TokioDuration::from_millis(200)).await;

        // Finalization
        task.progress.stage = ReplicationStage::Finalizing;
        task.progress.current_operation = "Finalizing replication".to_string();
        task.status = TaskStatus::Completed;
        task.completed_at = Some(Utc::now());
        task.progress.percentage_complete = 100.0;

        // Update metrics
        self.metrics.successful_replications += 1;
        if let Some(started_at) = task.started_at {
            let duration = (Utc::now() - started_at).num_seconds() as f64;
            self.metrics.average_replication_time_seconds =
                (self.metrics.average_replication_time_seconds * (self.metrics.successful_replications - 1) as f64 + duration)
                / self.metrics.successful_replications as f64;
        }

        tracing::info!("Replication task {} completed successfully", task_id);

        Ok(())
    }

    /// Check other trigger conditions beyond node failures
    async fn check_other_trigger_conditions(&mut self, chunk_health_data: &HashMap<String, ChunkHealth>) -> Result<()> {
        for (chunk_id, chunk_health) in chunk_health_data {
            // Check health score thresholds
            if chunk_health.availability_score < 70.0 {
                match chunk_health.overall_health {
                    HealthStatus::Critical | HealthStatus::Failed => {
                        if !self.active_tasks.values().any(|task| task.chunk_id == *chunk_id) {
                            self.trigger_replication(
                                chunk_id.clone(),
                                chunk_health,
                                &[], // No specific failed nodes
                            ).await?;
                        }
                    },
                    _ => {}
                }
            }

            // Check geographic distribution
            let regions: HashSet<_> = chunk_health.replica_health
                .iter()
                .filter(|replica| matches!(replica.status, ReplicaStatus::Healthy))
                .map(|replica| &replica.region)
                .collect();

            if regions.len() < 2 {
                // Consider triggering replication for better geographic distribution
                tracing::warn!("Chunk {} has poor geographic distribution ({} regions)", chunk_id, regions.len());
            }
        }

        Ok(())
    }

    /// Clean up completed tasks
    pub fn cleanup_completed_tasks(&mut self) {
        let cutoff_time = Utc::now() - Duration::hours(24); // Keep completed tasks for 24 hours

        self.active_tasks.retain(|_, task| {
            if matches!(task.status, TaskStatus::Completed | TaskStatus::Failed | TaskStatus::Cancelled) {
                if let Some(completed_at) = task.completed_at {
                    completed_at > cutoff_time
                } else {
                    task.created_at > cutoff_time
                }
            } else {
                true // Keep active tasks
            }
        });
    }

    /// Get replication status summary
    pub fn get_replication_status(&self) -> ReplicationStatus {
        let active_count = self.active_tasks.values()
            .filter(|task| matches!(task.status, TaskStatus::Queued | TaskStatus::Executing))
            .count();

        let completed_count = self.active_tasks.values()
            .filter(|task| matches!(task.status, TaskStatus::Completed))
            .count();

        let failed_count = self.active_tasks.values()
            .filter(|task| matches!(task.status, TaskStatus::Failed))
            .count();

        ReplicationStatus {
            active_replications: active_count as u32,
            completed_replications: completed_count as u32,
            failed_replications: failed_count as u32,
            total_nodes_online: self.node_status.values()
                .filter(|status| matches!(status.status, NodeState::Online))
                .count() as u32,
            total_nodes_offline: self.node_status.values()
                .filter(|status| matches!(status.status, NodeState::Offline))
                .count() as u32,
            average_replication_time: self.metrics.average_replication_time_seconds,
            total_data_replicated: self.metrics.total_data_recovered_gb,
            metrics: self.metrics.clone(),
        }
    }

    /// Run continuous monitoring and auto-replication
    pub async fn run_auto_replication_loop(&mut self, chunk_health_data: HashMap<String, ChunkHealth>) -> Result<()> {
        let mut check_interval = tokio::time::interval(TokioDuration::from_secs(
            self.failure_detection.heartbeat_interval_seconds as u64
        ));

        loop {
            check_interval.tick().await;

            // Detect failures and trigger replication
            if let Err(e) = self.detect_failures_and_replicate(&chunk_health_data).await {
                tracing::error!("Auto-replication check failed: {}", e);
            }

            // Clean up old tasks
            self.cleanup_completed_tasks();

            // Update metrics
            self.metrics.last_updated = Utc::now();

            tracing::debug!("Auto-replication check complete. Active tasks: {}", self.active_tasks.len());
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplicationStatus {
    pub active_replications: u32,
    pub completed_replications: u32,
    pub failed_replications: u32,
    pub total_nodes_online: u32,
    pub total_nodes_offline: u32,
    pub average_replication_time: f64,
    pub total_data_replicated: u64,
    pub metrics: AutoReplicationMetrics,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auto_replication_manager_creation() {
        let manager = AutoReplicationManager::new();
        assert!(!manager.replication_policies.is_empty());
        assert!(manager.failure_detection.heartbeat_interval_seconds > 0);
        assert!(manager.emergency_config.enable_emergency_mode);
    }

    #[test]
    fn test_node_status_updates() {
        let mut manager = AutoReplicationManager::new();

        // Add node as online
        manager.update_node_status("node1".to_string(), NodeState::Online);
        assert_eq!(manager.node_status.len(), 1);

        // Update to offline
        manager.update_node_status("node1".to_string(), NodeState::Offline);
        let status = manager.node_status.get("node1").unwrap();
        assert_eq!(status.consecutive_failures, 1);
        assert!(!status.failure_history.is_empty());

        // Back to online
        manager.update_node_status("node1".to_string(), NodeState::Online);
        let status = manager.node_status.get("node1").unwrap();
        assert_eq!(status.consecutive_failures, 0);
    }

    #[test]
    fn test_failure_detection() {
        let mut manager = AutoReplicationManager::new();

        // Add some nodes
        manager.update_node_status("node1".to_string(), NodeState::Online);
        manager.update_node_status("node2".to_string(), NodeState::Offline);
        manager.update_node_status("node3".to_string(), NodeState::Unknown);

        let failed_nodes = manager.detect_failed_nodes();
        assert!(failed_nodes.contains(&"node2".to_string()));
        assert!(failed_nodes.contains(&"node3".to_string()));
        assert!(!failed_nodes.contains(&"node1".to_string()));
    }

    #[tokio::test]
    async fn test_replication_trigger() {
        let mut manager = AutoReplicationManager::new();

        // Create mock chunk health with failed replica
        let chunk_health = ChunkHealth {
            chunk_id: "test_chunk".to_string(),
            overall_health: HealthStatus::Critical,
            replica_health: vec![
                ReplicaHealth {
                    replica_id: "replica1".to_string(),
                    node_id: "failed_node".to_string(),
                    region: GeographicRegion::NorthAmerica,
                    status: ReplicaStatus::Unreachable,
                    health_score: 0.0,
                    last_accessed: Utc::now(),
                    last_verified: Utc::now(),
                    integrity_hash: "hash1".to_string(),
                    performance_metrics: super::super::health_monitor::ReplicaPerformanceMetrics {
                        response_time_ms: 0.0,
                        transfer_speed_mbps: 0.0,
                        success_rate: 0.0,
                        error_count: 0,
                        last_error: None,
                        uptime_percentage: 0.0,
                    },
                    connectivity_status: super::super::health_monitor::ConnectivityStatus::Offline,
                },
            ],
            integrity_status: super::super::health_monitor::IntegrityStatus::Unknown,
            availability_score: 30.0,
            durability_score: 30.0,
            performance_metrics: super::super::health_monitor::ChunkPerformanceMetrics {
                avg_response_time_ms: 0.0,
                success_rate: 0.0,
                throughput_mbps: 0.0,
                error_rate: 100.0,
                access_frequency: super::super::health_monitor::AccessFrequency::Low,
                bandwidth_utilization: 0.0,
            },
            last_verified: Utc::now(),
            next_check_due: Utc::now(),
            risk_factors: vec![],
            repair_history: vec![],
        };

        let failed_nodes = vec!["failed_node".to_string()];

        let task_id = manager.trigger_replication(
            "test_chunk".to_string(),
            &chunk_health,
            &failed_nodes,
        ).await.unwrap();

        assert!(!task_id.is_empty());
        assert!(manager.active_tasks.contains_key(&task_id));

        let task = manager.active_tasks.get(&task_id).unwrap();
        assert_eq!(task.chunk_id, "test_chunk");
        assert!(matches!(task.trigger_reason, TriggerReason::NodeFailure { .. }));
    }
}