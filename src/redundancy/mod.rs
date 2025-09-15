//! Redundancy Module
//!
//! Smart redundancy and data durability system for ZephyrFS

pub mod intelligent_replication;
pub mod geographic_optimizer;
pub mod health_monitor;
pub mod auto_replication;
pub mod reed_solomon;
pub mod recovery_optimizer;
pub mod predictive_replication;
pub mod reputation_system;
pub mod network_health_monitor;
pub mod contribution_node_selector;
pub mod contribution_replication_manager;

pub use intelligent_replication::{
    IntelligentReplicationManager, ReplicationStrategy, ContentType,
    ReplicationRecommendation, RecommendationType
};
pub use geographic_optimizer::{
    GeographicOptimizer, GeographicDistribution, DistributionConstraints,
    RegionScore, ComplianceStatus
};
pub use health_monitor::{
    ChunkHealthMonitor, ChunkHealth, ReplicaHealth, HealthStatus,
    HealthCheckResult, HealthSummary, RiskFactor
};
pub use auto_replication::{
    AutoReplicationManager, ReplicationTask, NodeStatus, NodeState,
    ReplicationStatus, AutoReplicationPolicy
};
pub use reed_solomon::{
    ReedSolomonCodec, ReedSolomonManager, ReedSolomonConfig,
    EncodedChunk, ReconstructionResult
};
pub use recovery_optimizer::{
    RecoveryOptimizer, RecoveryPlan, RecoveryStep,
    RecoveryExecutionResult, OptimizationStrategy
};
pub use predictive_replication::{
    MLPredictor, ProactiveReplicationManager, FailurePrediction,
    NodeMetrics, RecommendedAction
};
pub use reputation_system::{
    ReputationManager, NodeReputation, ReliabilityMetrics,
    PerformanceMetrics, ReputationEvent, EventType
};
pub use network_health_monitor::{
    NetworkHealthMonitor, NetworkHealthReport, HealthAlert,
    AlertSeverity, GlobalNetworkMetrics, RegionalHealth
};
pub use contribution_node_selector::{
    ContributionNodeSelector, NodeContribution, NodeReliability, SelectionWeights,
    NodeSelectionCriteria, NodeSelectionResult, SelectedNode
};
pub use contribution_replication_manager::{
    ContributionReplicationManager, ContributionReplicationPolicy, ReplicationJob,
    ReplicationPerformanceStats
};