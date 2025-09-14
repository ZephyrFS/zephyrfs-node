//! Intelligent Replication Strategy
//!
//! Adaptive redundancy system that optimizes data durability based on content importance,
//! network conditions, and cost considerations

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, BTreeMap};
use chrono::{DateTime, Utc, Duration};

use crate::economics::{NetworkHealthMetrics, VolunteerMetrics, GeographicRegion};

/// Intelligent replication manager
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntelligentReplicationManager {
    /// Replication policies for different content types
    pub policies: HashMap<ContentType, ReplicationPolicy>,
    /// Current replication state tracking
    pub replication_state: HashMap<String, ChunkReplicationState>,
    /// Node performance tracking for replication decisions
    pub node_performance: HashMap<String, NodePerformanceProfile>,
    /// Geographic distribution requirements
    pub geo_distribution: GeographicDistributionConfig,
    /// Adaptive redundancy configuration
    pub adaptive_config: AdaptiveRedundancyConfig,
    /// Cost optimization settings
    pub cost_config: CostOptimizationConfig,
}

#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub enum ContentType {
    Critical,      // System-critical data
    Important,     // User-important files
    Standard,      // Regular user files
    Archive,       // Long-term storage
    Temporary,     // Short-term cache
    Backup,        // Backup copies
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplicationPolicy {
    pub content_type: ContentType,
    pub min_replicas: u32,
    pub max_replicas: u32,
    pub target_replicas: u32,
    pub geographic_spread: GeographicSpread,
    pub node_quality_requirements: NodeQualityRequirements,
    pub redundancy_scheme: RedundancyScheme,
    pub replication_priority: ReplicationPriority,
    pub cost_sensitivity: f64, // 0.0 = cost-insensitive, 1.0 = highly cost-sensitive
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GeographicSpread {
    SingleRegion,
    MultiRegion(u32),     // Minimum number of regions
    GlobalDistribution,   // Maximum geographic spread
    RegionSpecific(Vec<GeographicRegion>), // Specific regions required
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeQualityRequirements {
    pub min_uptime_percentage: f64,
    pub min_bandwidth_mbps: f64,
    pub max_latency_ms: u64,
    pub min_reliability_score: f64,
    pub required_connection_quality: Option<ConnectionQuality>,
    pub exclude_unstable_nodes: bool,
    pub prefer_premium_nodes: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RedundancyScheme {
    SimpleReplication,           // Basic copying
    ReedSolomon { data: u32, parity: u32 }, // (n,k) Reed-Solomon
    HybridErasure { replicas: u32, erasure: (u32, u32) }, // Combination
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReplicationPriority {
    Immediate,
    High,
    Normal,
    Low,
    Background,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConnectionQuality {
    Fiber,
    Broadband,
    Mobile,
    Satellite,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkReplicationState {
    pub chunk_id: String,
    pub content_type: ContentType,
    pub current_replicas: Vec<ReplicaLocation>,
    pub target_replicas: u32,
    pub health_score: f64,
    pub last_verification: DateTime<Utc>,
    pub replication_status: ReplicationStatus,
    pub repair_history: Vec<RepairEvent>,
    pub access_patterns: AccessPatterns,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplicaLocation {
    pub node_id: String,
    pub region: GeographicRegion,
    pub quality_score: f64,
    pub created_at: DateTime<Utc>,
    pub last_verified: DateTime<Utc>,
    pub status: ReplicaStatus,
    pub performance_metrics: ReplicaPerformance,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReplicaStatus {
    Healthy,
    Degraded,
    Unreachable,
    Corrupted,
    Missing,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplicaPerformance {
    pub response_time_ms: u64,
    pub transfer_speed_mbps: f64,
    pub success_rate: f64,
    pub last_access: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReplicationStatus {
    Optimal,              // Meets all requirements
    Adequate,             // Meets minimum requirements
    Degraded,             // Below minimum requirements
    Critical,             // Immediate action needed
    Repairing,            // Currently being repaired
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepairEvent {
    pub event_id: String,
    pub timestamp: DateTime<Utc>,
    pub event_type: RepairEventType,
    pub affected_replicas: Vec<String>,
    pub repair_strategy: RepairStrategy,
    pub success: bool,
    pub duration_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RepairEventType {
    NodeFailure,
    NetworkPartition,
    CorruptionDetected,
    PerformanceDegradation,
    ScheduledMaintenance,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RepairStrategy {
    CreateNewReplica,
    RepairExistingReplica,
    MigrateReplica,
    IncreaseRedundancy,
    RebuildFromErasure,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessPatterns {
    pub access_frequency: AccessFrequency,
    pub geographic_access: HashMap<GeographicRegion, u32>,
    pub time_patterns: HashMap<u8, u32>, // Hour of day -> access count
    pub last_access: DateTime<Utc>,
    pub predicted_next_access: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AccessFrequency {
    VeryHigh,  // Multiple times per hour
    High,      // Multiple times per day
    Medium,    // Daily access
    Low,       // Weekly access
    VeryLow,   // Monthly access
    Archive,   // Rarely accessed
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodePerformanceProfile {
    pub node_id: String,
    pub region: GeographicRegion,
    pub uptime_percentage: f64,
    pub bandwidth_mbps: f64,
    pub latency_ms: u64,
    pub reliability_score: f64,
    pub connection_quality: ConnectionQuality,
    pub storage_capacity_gb: u64,
    pub available_capacity_gb: u64,
    pub cost_per_gb_month: f64,
    pub performance_tier: PerformanceTier,
    pub last_updated: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PerformanceTier {
    Premium,   // Top 10% performers
    High,      // Top 25% performers
    Standard,  // Average performers
    Basic,     // Below average
    Unreliable, // Poor performers
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeographicDistributionConfig {
    pub min_regions_per_chunk: u32,
    pub preferred_regions: Vec<GeographicRegion>,
    pub region_weights: HashMap<GeographicRegion, f64>,
    pub latency_requirements: HashMap<GeographicRegion, u64>,
    pub regulatory_constraints: HashMap<GeographicRegion, Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdaptiveRedundancyConfig {
    pub enable_dynamic_adjustment: bool,
    pub adjustment_frequency_hours: u32,
    pub network_health_threshold: f64,
    pub failure_rate_threshold: f64,
    pub auto_scale_replicas: bool,
    pub max_auto_replicas: u32,
    pub min_auto_replicas: u32,
    pub cost_efficiency_target: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostOptimizationConfig {
    pub enable_cost_optimization: bool,
    pub cost_efficiency_weight: f64,
    pub durability_weight: f64,
    pub performance_weight: f64,
    pub max_cost_per_gb_month: f64,
    pub prefer_cheaper_nodes: bool,
    pub cost_monitoring_enabled: bool,
}

impl Default for ReplicationPolicy {
    fn default() -> Self {
        Self {
            content_type: ContentType::Standard,
            min_replicas: 3,
            max_replicas: 10,
            target_replicas: 5,
            geographic_spread: GeographicSpread::MultiRegion(2),
            node_quality_requirements: NodeQualityRequirements {
                min_uptime_percentage: 95.0,
                min_bandwidth_mbps: 10.0,
                max_latency_ms: 200,
                min_reliability_score: 90.0,
                required_connection_quality: None,
                exclude_unstable_nodes: true,
                prefer_premium_nodes: false,
            },
            redundancy_scheme: RedundancyScheme::SimpleReplication,
            replication_priority: ReplicationPriority::Normal,
            cost_sensitivity: 0.3,
        }
    }
}

impl IntelligentReplicationManager {
    /// Create new intelligent replication manager
    pub fn new() -> Self {
        let mut manager = Self {
            policies: HashMap::new(),
            replication_state: HashMap::new(),
            node_performance: HashMap::new(),
            geo_distribution: GeographicDistributionConfig {
                min_regions_per_chunk: 2,
                preferred_regions: vec![
                    GeographicRegion::NorthAmerica,
                    GeographicRegion::Europe,
                    GeographicRegion::Asia,
                ],
                region_weights: HashMap::new(),
                latency_requirements: HashMap::new(),
                regulatory_constraints: HashMap::new(),
            },
            adaptive_config: AdaptiveRedundancyConfig {
                enable_dynamic_adjustment: true,
                adjustment_frequency_hours: 6,
                network_health_threshold: 95.0,
                failure_rate_threshold: 0.01,
                auto_scale_replicas: true,
                max_auto_replicas: 15,
                min_auto_replicas: 3,
                cost_efficiency_target: 0.8,
            },
            cost_config: CostOptimizationConfig {
                enable_cost_optimization: true,
                cost_efficiency_weight: 0.3,
                durability_weight: 0.5,
                performance_weight: 0.2,
                max_cost_per_gb_month: 0.05,
                prefer_cheaper_nodes: false,
                cost_monitoring_enabled: true,
            },
        };

        manager.initialize_default_policies();
        manager
    }

    /// Initialize default replication policies
    fn initialize_default_policies(&mut self) {
        // Critical content policy
        self.policies.insert(ContentType::Critical, ReplicationPolicy {
            content_type: ContentType::Critical,
            min_replicas: 5,
            max_replicas: 15,
            target_replicas: 8,
            geographic_spread: GeographicSpread::GlobalDistribution,
            node_quality_requirements: NodeQualityRequirements {
                min_uptime_percentage: 99.0,
                min_bandwidth_mbps: 50.0,
                max_latency_ms: 100,
                min_reliability_score: 95.0,
                required_connection_quality: Some(ConnectionQuality::Fiber),
                exclude_unstable_nodes: true,
                prefer_premium_nodes: true,
            },
            redundancy_scheme: RedundancyScheme::ReedSolomon { data: 6, parity: 3 },
            replication_priority: ReplicationPriority::Immediate,
            cost_sensitivity: 0.1, // Low cost sensitivity for critical data
        });

        // Important content policy
        self.policies.insert(ContentType::Important, ReplicationPolicy {
            content_type: ContentType::Important,
            min_replicas: 4,
            max_replicas: 10,
            target_replicas: 6,
            geographic_spread: GeographicSpread::MultiRegion(3),
            node_quality_requirements: NodeQualityRequirements {
                min_uptime_percentage: 97.0,
                min_bandwidth_mbps: 25.0,
                max_latency_ms: 150,
                min_reliability_score: 92.0,
                required_connection_quality: None,
                exclude_unstable_nodes: true,
                prefer_premium_nodes: true,
            },
            redundancy_scheme: RedundancyScheme::ReedSolomon { data: 4, parity: 2 },
            replication_priority: ReplicationPriority::High,
            cost_sensitivity: 0.2,
        });

        // Standard content policy
        self.policies.insert(ContentType::Standard, ReplicationPolicy::default());

        // Archive content policy
        self.policies.insert(ContentType::Archive, ReplicationPolicy {
            content_type: ContentType::Archive,
            min_replicas: 3,
            max_replicas: 8,
            target_replicas: 4,
            geographic_spread: GeographicSpread::MultiRegion(2),
            node_quality_requirements: NodeQualityRequirements {
                min_uptime_percentage: 90.0,
                min_bandwidth_mbps: 5.0,
                max_latency_ms: 500,
                min_reliability_score: 85.0,
                required_connection_quality: None,
                exclude_unstable_nodes: false,
                prefer_premium_nodes: false,
            },
            redundancy_scheme: RedundancyScheme::ReedSolomon { data: 3, parity: 2 },
            replication_priority: ReplicationPriority::Background,
            cost_sensitivity: 0.8, // High cost sensitivity for archive data
        });

        // Temporary content policy
        self.policies.insert(ContentType::Temporary, ReplicationPolicy {
            content_type: ContentType::Temporary,
            min_replicas: 2,
            max_replicas: 4,
            target_replicas: 3,
            geographic_spread: GeographicSpread::SingleRegion,
            node_quality_requirements: NodeQualityRequirements {
                min_uptime_percentage: 85.0,
                min_bandwidth_mbps: 10.0,
                max_latency_ms: 300,
                min_reliability_score: 80.0,
                required_connection_quality: None,
                exclude_unstable_nodes: false,
                prefer_premium_nodes: false,
            },
            redundancy_scheme: RedundancyScheme::SimpleReplication,
            replication_priority: ReplicationPriority::Low,
            cost_sensitivity: 1.0, // Maximum cost sensitivity
        });
    }

    /// Determine optimal replication strategy for a chunk
    pub fn determine_replication_strategy(
        &self,
        chunk_id: &str,
        content_type: ContentType,
        access_patterns: &AccessPatterns,
        network_health: &NetworkHealthMetrics,
    ) -> Result<ReplicationStrategy> {
        let policy = self.policies.get(&content_type)
            .ok_or_else(|| anyhow::anyhow!("No policy found for content type: {:?}", content_type))?;

        // Calculate base replication requirements
        let mut target_replicas = policy.target_replicas;

        // Adjust based on access patterns
        target_replicas = self.adjust_for_access_patterns(target_replicas, access_patterns);

        // Adjust based on network health
        target_replicas = self.adjust_for_network_health(target_replicas, network_health);

        // Ensure within policy bounds
        target_replicas = target_replicas.max(policy.min_replicas).min(policy.max_replicas);

        // Select optimal nodes
        let selected_nodes = self.select_optimal_nodes(
            target_replicas,
            &policy.node_quality_requirements,
            &policy.geographic_spread,
            policy.cost_sensitivity,
        )?;

        Ok(ReplicationStrategy {
            chunk_id: chunk_id.to_string(),
            content_type,
            target_replicas,
            selected_nodes,
            redundancy_scheme: policy.redundancy_scheme.clone(),
            priority: policy.replication_priority.clone(),
            estimated_cost: self.calculate_replication_cost(&selected_nodes),
            durability_score: self.calculate_durability_score(&selected_nodes, &policy.redundancy_scheme),
        })
    }

    /// Adjust replica count based on access patterns
    fn adjust_for_access_patterns(&self, base_replicas: u32, access_patterns: &AccessPatterns) -> u32 {
        let frequency_multiplier = match access_patterns.access_frequency {
            AccessFrequency::VeryHigh => 1.5,
            AccessFrequency::High => 1.2,
            AccessFrequency::Medium => 1.0,
            AccessFrequency::Low => 0.9,
            AccessFrequency::VeryLow => 0.8,
            AccessFrequency::Archive => 0.7,
        };

        // Geographic access diversity bonus
        let geo_diversity_bonus = if access_patterns.geographic_access.len() > 2 {
            1.1
        } else {
            1.0
        };

        ((base_replicas as f64) * frequency_multiplier * geo_diversity_bonus) as u32
    }

    /// Adjust replica count based on network health
    fn adjust_for_network_health(&self, base_replicas: u32, network_health: &NetworkHealthMetrics) -> u32 {
        let health_factor = if network_health.average_uptime < 90.0 {
            1.3 // Increase replicas for poor network health
        } else if network_health.average_uptime < 95.0 {
            1.1
        } else {
            1.0 // Normal replication for healthy network
        };

        // Adjust for utilization pressure
        let utilization_factor = if network_health.utilization_rate > 90.0 {
            0.9 // Reduce replicas under high utilization
        } else {
            1.0
        };

        ((base_replicas as f64) * health_factor * utilization_factor) as u32
    }

    /// Select optimal nodes for replication
    fn select_optimal_nodes(
        &self,
        target_replicas: u32,
        quality_requirements: &NodeQualityRequirements,
        geographic_spread: &GeographicSpread,
        cost_sensitivity: f64,
    ) -> Result<Vec<String>> {
        // Filter nodes by quality requirements
        let eligible_nodes: Vec<_> = self.node_performance
            .values()
            .filter(|node| self.meets_quality_requirements(node, quality_requirements))
            .collect();

        if eligible_nodes.is_empty() {
            return Err(anyhow::anyhow!("No nodes meet quality requirements"));
        }

        // Group by region for geographic distribution
        let nodes_by_region = self.group_nodes_by_region(&eligible_nodes);

        // Select nodes based on geographic spread requirements
        let selected_nodes = self.apply_geographic_selection(
            nodes_by_region,
            target_replicas,
            geographic_spread,
            cost_sensitivity,
        )?;

        Ok(selected_nodes)
    }

    /// Check if node meets quality requirements
    fn meets_quality_requirements(
        &self,
        node: &NodePerformanceProfile,
        requirements: &NodeQualityRequirements,
    ) -> bool {
        node.uptime_percentage >= requirements.min_uptime_percentage
            && node.bandwidth_mbps >= requirements.min_bandwidth_mbps
            && node.latency_ms <= requirements.max_latency_ms
            && node.reliability_score >= requirements.min_reliability_score
            && node.available_capacity_gb > 0
            && (!requirements.exclude_unstable_nodes || !matches!(node.performance_tier, PerformanceTier::Unreliable))
            && requirements.required_connection_quality.as_ref()
                .map_or(true, |required| self.connection_quality_matches(&node.connection_quality, required))
    }

    /// Check if connection quality matches requirement
    fn connection_quality_matches(&self, actual: &ConnectionQuality, required: &ConnectionQuality) -> bool {
        match (actual, required) {
            (ConnectionQuality::Fiber, _) => true,
            (ConnectionQuality::Broadband, ConnectionQuality::Fiber) => false,
            (ConnectionQuality::Broadband, _) => true,
            (ConnectionQuality::Mobile, ConnectionQuality::Fiber | ConnectionQuality::Broadband) => false,
            (ConnectionQuality::Mobile, _) => true,
            (ConnectionQuality::Satellite, ConnectionQuality::Satellite) => true,
            (ConnectionQuality::Satellite, _) => false,
        }
    }

    /// Group nodes by geographic region
    fn group_nodes_by_region(
        &self,
        nodes: &[&NodePerformanceProfile],
    ) -> HashMap<GeographicRegion, Vec<&NodePerformanceProfile>> {
        let mut grouped = HashMap::new();

        for node in nodes {
            grouped.entry(node.region.clone())
                .or_insert_with(Vec::new)
                .push(*node);
        }

        // Sort nodes within each region by performance score
        for region_nodes in grouped.values_mut() {
            region_nodes.sort_by(|a, b| {
                let score_a = self.calculate_node_score(a, 0.3); // Default cost sensitivity
                let score_b = self.calculate_node_score(b, 0.3);
                score_b.partial_cmp(&score_a).unwrap_or(std::cmp::Ordering::Equal)
            });
        }

        grouped
    }

    /// Apply geographic selection strategy
    fn apply_geographic_selection(
        &self,
        nodes_by_region: HashMap<GeographicRegion, Vec<&NodePerformanceProfile>>,
        target_replicas: u32,
        geographic_spread: &GeographicSpread,
        cost_sensitivity: f64,
    ) -> Result<Vec<String>> {
        let mut selected_nodes = Vec::new();

        match geographic_spread {
            GeographicSpread::SingleRegion => {
                // Select all from the best region
                let best_region = self.find_best_region(&nodes_by_region, cost_sensitivity)?;
                if let Some(region_nodes) = nodes_by_region.get(&best_region) {
                    for node in region_nodes.iter().take(target_replicas as usize) {
                        selected_nodes.push(node.node_id.clone());
                    }
                }
            },
            GeographicSpread::MultiRegion(min_regions) => {
                selected_nodes = self.select_multi_region_nodes(
                    &nodes_by_region,
                    target_replicas,
                    *min_regions,
                    cost_sensitivity,
                )?;
            },
            GeographicSpread::GlobalDistribution => {
                selected_nodes = self.select_global_distribution_nodes(
                    &nodes_by_region,
                    target_replicas,
                    cost_sensitivity,
                )?;
            },
            GeographicSpread::RegionSpecific(required_regions) => {
                selected_nodes = self.select_region_specific_nodes(
                    &nodes_by_region,
                    target_replicas,
                    required_regions,
                    cost_sensitivity,
                )?;
            },
        }

        if selected_nodes.len() < target_replicas as usize {
            tracing::warn!("Could only select {} nodes out of {} requested",
                selected_nodes.len(), target_replicas);
        }

        Ok(selected_nodes)
    }

    /// Find the best region based on cost and performance
    fn find_best_region(
        &self,
        nodes_by_region: &HashMap<GeographicRegion, Vec<&NodePerformanceProfile>>,
        cost_sensitivity: f64,
    ) -> Result<GeographicRegion> {
        let mut best_region = None;
        let mut best_score = 0.0;

        for (region, nodes) in nodes_by_region {
            if nodes.is_empty() {
                continue;
            }

            let avg_score = nodes.iter()
                .map(|node| self.calculate_node_score(node, cost_sensitivity))
                .sum::<f64>() / nodes.len() as f64;

            if avg_score > best_score {
                best_score = avg_score;
                best_region = Some(region.clone());
            }
        }

        best_region.ok_or_else(|| anyhow::anyhow!("No suitable region found"))
    }

    /// Select nodes across multiple regions
    fn select_multi_region_nodes(
        &self,
        nodes_by_region: &HashMap<GeographicRegion, Vec<&NodePerformanceProfile>>,
        target_replicas: u32,
        min_regions: u32,
        cost_sensitivity: f64,
    ) -> Result<Vec<String>> {
        let mut selected_nodes = Vec::new();
        let available_regions: Vec<_> = nodes_by_region.keys()
            .filter(|region| !nodes_by_region[*region].is_empty())
            .collect();

        if available_regions.len() < min_regions as usize {
            return Err(anyhow::anyhow!("Not enough regions available: {} < {}",
                available_regions.len(), min_regions));
        }

        // Calculate replicas per region
        let replicas_per_region = target_replicas / min_regions;
        let extra_replicas = target_replicas % min_regions;

        // Sort regions by quality
        let mut sorted_regions = available_regions;
        sorted_regions.sort_by(|a, b| {
            let score_a = self.calculate_region_score(nodes_by_region[*a].as_slice(), cost_sensitivity);
            let score_b = self.calculate_region_score(nodes_by_region[*b].as_slice(), cost_sensitivity);
            score_b.partial_cmp(&score_a).unwrap_or(std::cmp::Ordering::Equal)
        });

        // Select nodes from each region
        for (i, region) in sorted_regions.iter().take(min_regions as usize).enumerate() {
            let region_nodes = &nodes_by_region[*region];
            let region_replicas = replicas_per_region + if i < extra_replicas as usize { 1 } else { 0 };

            for node in region_nodes.iter().take(region_replicas as usize) {
                selected_nodes.push(node.node_id.clone());
            }
        }

        Ok(selected_nodes)
    }

    /// Select nodes for global distribution
    fn select_global_distribution_nodes(
        &self,
        nodes_by_region: &HashMap<GeographicRegion, Vec<&NodePerformanceProfile>>,
        target_replicas: u32,
        cost_sensitivity: f64,
    ) -> Result<Vec<String>> {
        // Try to distribute across all available regions
        let available_regions = nodes_by_region.len() as u32;
        self.select_multi_region_nodes(nodes_by_region, target_replicas, available_regions, cost_sensitivity)
    }

    /// Select nodes from specific regions
    fn select_region_specific_nodes(
        &self,
        nodes_by_region: &HashMap<GeographicRegion, Vec<&NodePerformanceProfile>>,
        target_replicas: u32,
        required_regions: &[GeographicRegion],
        cost_sensitivity: f64,
    ) -> Result<Vec<String>> {
        let mut selected_nodes = Vec::new();
        let replicas_per_region = target_replicas / required_regions.len() as u32;
        let extra_replicas = target_replicas % required_regions.len() as u32;

        for (i, region) in required_regions.iter().enumerate() {
            if let Some(region_nodes) = nodes_by_region.get(region) {
                let region_replicas = replicas_per_region + if i < extra_replicas as usize { 1 } else { 0 };

                for node in region_nodes.iter().take(region_replicas as usize) {
                    selected_nodes.push(node.node_id.clone());
                }
            }
        }

        Ok(selected_nodes)
    }

    /// Calculate node performance score
    fn calculate_node_score(&self, node: &NodePerformanceProfile, cost_sensitivity: f64) -> f64 {
        let performance_score = (node.uptime_percentage / 100.0)
            * (node.reliability_score / 100.0)
            * (node.bandwidth_mbps / 100.0).min(1.0)
            * (200.0 / (node.latency_ms as f64 + 50.0));

        let cost_score = if cost_sensitivity > 0.0 {
            1.0 / (node.cost_per_gb_month + 0.01) // Avoid division by zero
        } else {
            1.0
        };

        // Weighted combination
        performance_score * (1.0 - cost_sensitivity) + cost_score * cost_sensitivity
    }

    /// Calculate region quality score
    fn calculate_region_score(&self, nodes: &[&NodePerformanceProfile], cost_sensitivity: f64) -> f64 {
        if nodes.is_empty() {
            return 0.0;
        }

        nodes.iter()
            .map(|node| self.calculate_node_score(node, cost_sensitivity))
            .sum::<f64>() / nodes.len() as f64
    }

    /// Calculate estimated replication cost
    fn calculate_replication_cost(&self, selected_nodes: &[String]) -> f64 {
        selected_nodes.iter()
            .filter_map(|node_id| self.node_performance.get(node_id))
            .map(|node| node.cost_per_gb_month)
            .sum()
    }

    /// Calculate expected durability score
    fn calculate_durability_score(&self, selected_nodes: &[String], redundancy_scheme: &RedundancyScheme) -> f64 {
        let avg_reliability = selected_nodes.iter()
            .filter_map(|node_id| self.node_performance.get(node_id))
            .map(|node| node.reliability_score / 100.0)
            .sum::<f64>() / selected_nodes.len() as f64;

        // Calculate durability based on redundancy scheme
        match redundancy_scheme {
            RedundancyScheme::SimpleReplication => {
                // Simple calculation: 1 - (1 - reliability)^replicas
                1.0 - (1.0 - avg_reliability).powf(selected_nodes.len() as f64)
            },
            RedundancyScheme::ReedSolomon { data, parity } => {
                // Can survive up to 'parity' failures
                let total_shards = data + parity;
                let failure_tolerance = *parity as f64;

                // Simplified calculation
                let failure_prob = 1.0 - avg_reliability;
                let survive_prob = (0..=failure_tolerance as u32)
                    .map(|failures| {
                        binomial_probability(total_shards, failures, failure_prob)
                    })
                    .sum::<f64>();

                survive_prob
            },
            RedundancyScheme::HybridErasure { replicas, erasure } => {
                // Combination of replication and erasure coding
                let replication_durability = 1.0 - (1.0 - avg_reliability).powf(*replicas as f64);
                let erasure_durability = self.calculate_durability_score(
                    selected_nodes,
                    &RedundancyScheme::ReedSolomon { data: erasure.0, parity: erasure.1 }
                );

                // Best of both
                replication_durability.max(erasure_durability)
            },
        }
    }

    /// Update node performance profile
    pub fn update_node_performance(&mut self, node_id: String, profile: NodePerformanceProfile) {
        self.node_performance.insert(node_id, profile);
    }

    /// Update chunk replication state
    pub fn update_chunk_state(&mut self, chunk_id: String, state: ChunkReplicationState) {
        self.replication_state.insert(chunk_id, state);
    }

    /// Get replication recommendations for a chunk
    pub fn get_replication_recommendations(&self, chunk_id: &str) -> Result<Vec<ReplicationRecommendation>> {
        let state = self.replication_state.get(chunk_id)
            .ok_or_else(|| anyhow::anyhow!("Chunk state not found"))?;

        let mut recommendations = Vec::new();

        // Check if we need more replicas
        if state.current_replicas.len() < state.target_replicas as usize {
            recommendations.push(ReplicationRecommendation {
                recommendation_type: RecommendationType::IncreaseReplicas,
                priority: ReplicationPriority::High,
                estimated_cost: 0.02, // Placeholder
                durability_impact: 0.15,
                description: "Increase replica count to meet target".to_string(),
            });
        }

        // Check for unhealthy replicas
        let unhealthy_replicas: Vec<_> = state.current_replicas.iter()
            .filter(|replica| !matches!(replica.status, ReplicaStatus::Healthy))
            .collect();

        if !unhealthy_replicas.is_empty() {
            recommendations.push(ReplicationRecommendation {
                recommendation_type: RecommendationType::RepairReplicas,
                priority: ReplicationPriority::Immediate,
                estimated_cost: 0.05,
                durability_impact: 0.25,
                description: format!("Repair {} unhealthy replicas", unhealthy_replicas.len()),
            });
        }

        // Check geographic distribution
        let regions: std::collections::HashSet<_> = state.current_replicas.iter()
            .map(|replica| &replica.region)
            .collect();

        if regions.len() < self.geo_distribution.min_regions_per_chunk as usize {
            recommendations.push(ReplicationRecommendation {
                recommendation_type: RecommendationType::ImproveGeographicDistribution,
                priority: ReplicationPriority::Normal,
                estimated_cost: 0.03,
                durability_impact: 0.10,
                description: "Improve geographic distribution".to_string(),
            });
        }

        Ok(recommendations)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplicationStrategy {
    pub chunk_id: String,
    pub content_type: ContentType,
    pub target_replicas: u32,
    pub selected_nodes: Vec<String>,
    pub redundancy_scheme: RedundancyScheme,
    pub priority: ReplicationPriority,
    pub estimated_cost: f64,
    pub durability_score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplicationRecommendation {
    pub recommendation_type: RecommendationType,
    pub priority: ReplicationPriority,
    pub estimated_cost: f64,
    pub durability_impact: f64,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecommendationType {
    IncreaseReplicas,
    DecreaseReplicas,
    RepairReplicas,
    MigrateReplicas,
    ImproveGeographicDistribution,
    OptimizeCost,
    UpgradeRedundancyScheme,
}

/// Calculate binomial probability
fn binomial_probability(n: u32, k: u32, p: f64) -> f64 {
    if k > n {
        return 0.0;
    }

    let combination = factorial(n) / (factorial(k) * factorial(n - k));
    combination as f64 * p.powi(k as i32) * (1.0 - p).powi((n - k) as i32)
}

/// Calculate factorial (simplified for small numbers)
fn factorial(n: u32) -> u64 {
    (1..=n as u64).product()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_replication_manager_creation() {
        let manager = IntelligentReplicationManager::new();
        assert!(!manager.policies.is_empty());
        assert!(manager.policies.contains_key(&ContentType::Critical));
        assert!(manager.policies.contains_key(&ContentType::Standard));
    }

    #[test]
    fn test_node_quality_requirements() {
        let manager = IntelligentReplicationManager::new();

        let high_quality_node = NodePerformanceProfile {
            node_id: "node1".to_string(),
            region: GeographicRegion::NorthAmerica,
            uptime_percentage: 99.5,
            bandwidth_mbps: 100.0,
            latency_ms: 50,
            reliability_score: 98.0,
            connection_quality: ConnectionQuality::Fiber,
            storage_capacity_gb: 1000,
            available_capacity_gb: 500,
            cost_per_gb_month: 0.02,
            performance_tier: PerformanceTier::Premium,
            last_updated: Utc::now(),
        };

        let requirements = &manager.policies[&ContentType::Critical].node_quality_requirements;
        assert!(manager.meets_quality_requirements(&high_quality_node, requirements));
    }

    #[test]
    fn test_access_pattern_adjustment() {
        let manager = IntelligentReplicationManager::new();

        let high_access_patterns = AccessPatterns {
            access_frequency: AccessFrequency::VeryHigh,
            geographic_access: HashMap::from([
                (GeographicRegion::NorthAmerica, 100),
                (GeographicRegion::Europe, 50),
                (GeographicRegion::Asia, 25),
            ]),
            time_patterns: HashMap::new(),
            last_access: Utc::now(),
            predicted_next_access: None,
        };

        let adjusted = manager.adjust_for_access_patterns(5, &high_access_patterns);
        assert!(adjusted > 5); // Should increase for high-access content
    }
}