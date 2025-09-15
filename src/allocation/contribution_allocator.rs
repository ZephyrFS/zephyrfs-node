//! Contribution-Based Resource Allocator
//!
//! Allocates network resources based on user contributions rather than monetary payments

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use chrono::{DateTime, Utc, Duration};

use crate::economics::{UserContribution, PriorityLevel, ContributionTracker};

/// Main resource allocator using contribution ratios
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContributionBasedAllocator {
    /// Allocation decisions cache
    pub allocation_cache: HashMap<String, AllocationDecision>,
    /// Resource availability tracking
    pub resource_availability: ResourceAvailability,
    /// Allocation strategies configuration
    pub strategies: HashMap<AllocationStrategy, StrategyConfig>,
    /// Historical allocation data
    pub allocation_history: VecDeque<HistoricalAllocation>,
    /// Performance metrics
    pub performance_metrics: AllocationMetrics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllocationRequest {
    pub request_id: String,
    pub user_id: String,
    pub resource_type: ResourceType,
    pub amount_requested: u64,
    pub quality_requirements: QualityRequirements,
    pub duration: Option<Duration>,
    pub deadline: Option<DateTime<Utc>>,
    pub priority: RequestPriority,
    pub requested_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResourceType {
    Storage {
        size_gb: u64,
        io_requirements: IORequirements,
    },
    Bandwidth {
        mbps: u64,
        burst_capacity: Option<u64>,
    },
    Compute {
        cpu_cores: u32,
        memory_gb: u32,
    },
    Network {
        connections: u32,
        max_latency_ms: u32,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IORequirements {
    pub read_iops: u32,
    pub write_iops: u32,
    pub sequential_read_mbps: u32,
    pub sequential_write_mbps: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityRequirements {
    pub reliability_level: ReliabilityLevel,
    pub performance_tier: PerformanceTier,
    pub availability_sla: f64, // e.g., 99.9%
    pub max_latency_ms: Option<u32>,
    pub redundancy_level: RedundancyLevel,
    pub geographic_constraints: Option<GeographicConstraints>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReliabilityLevel {
    Basic,    // 95% uptime, basic monitoring
    Standard, // 99% uptime, standard monitoring
    High,     // 99.9% uptime, advanced monitoring
    Critical, // 99.99% uptime, premium monitoring
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PerformanceTier {
    Economy,   // Best effort, no guarantees
    Standard,  // Baseline performance guarantees
    Premium,   // High performance guarantees
    Enterprise, // Maximum performance guarantees
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RedundancyLevel {
    Single,    // No redundancy (lowest cost)
    Mirror,    // 2x redundancy
    Standard,  // 3x redundancy
    High,      // 5x redundancy
    Critical,  // 7x redundancy
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeographicConstraints {
    pub allowed_regions: Option<Vec<String>>,
    pub prohibited_regions: Option<Vec<String>>,
    pub data_sovereignty_requirements: Option<String>,
    pub max_distance_km: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RequestPriority {
    Background, // Can wait indefinitely
    Normal,     // Standard priority
    High,       // Expedited processing
    Urgent,     // Immediate processing required
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllocationDecision {
    pub request_id: String,
    pub user_id: String,
    pub decision: AllocationOutcome,
    pub allocated_resources: Option<AllocatedResources>,
    pub reason: String,
    pub contribution_score_used: f64,
    pub priority_level_used: PriorityLevel,
    pub decided_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AllocationOutcome {
    Approved,
    Denied,
    PartiallyApproved,
    Queued,
    Expired,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllocatedResources {
    pub resource_type: ResourceType,
    pub amount_allocated: u64,
    pub quality_level: AllocationQuality,
    pub assigned_nodes: Vec<NodeAssignment>,
    pub estimated_performance: PerformanceEstimate,
    pub sla_commitments: SLACommitments,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeAssignment {
    pub node_id: String,
    pub resource_portion: f64, // 0.0-1.0
    pub role: NodeRole,
    pub expected_performance: NodePerformance,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NodeRole {
    Primary,   // Main storage/compute node
    Secondary, // Backup/redundancy node
    Cache,     // Caching/acceleration node
    Router,    // Network routing node
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodePerformance {
    pub uptime_percentage: f64,
    pub response_time_ms: u32,
    pub throughput_mbps: f64,
    pub reliability_score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceEstimate {
    pub expected_throughput_mbps: f64,
    pub expected_latency_ms: u32,
    pub expected_availability_percent: f64,
    pub confidence_level: f64, // 0.0-1.0
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SLACommitments {
    pub uptime_guarantee: f64,
    pub performance_guarantee: PerformanceGuarantee,
    pub data_durability: f64, // e.g., 99.999999999% (11 nines)
    pub support_level: SupportLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceGuarantee {
    pub min_throughput_mbps: f64,
    pub max_latency_ms: u32,
    pub response_time_percentile: PercentileGuarantee,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PercentileGuarantee {
    pub p50_latency_ms: u32,
    pub p95_latency_ms: u32,
    pub p99_latency_ms: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SupportLevel {
    Community, // Community support only
    Standard,  // Business hours support
    Premium,   // 24/7 support
    Enterprise, // Dedicated support
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AllocationQuality {
    /// Basic allocation with minimal guarantees
    Basic,
    /// Standard allocation with reasonable guarantees
    Standard,
    /// Premium allocation with strong guarantees
    Premium,
    /// Enterprise-grade allocation with maximum guarantees
    Enterprise,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum AllocationStrategy {
    /// Fair sharing based on contribution ratios
    ContributionBased,
    /// Priority to highest contributors
    ContributorFirst,
    /// Balanced approach considering multiple factors
    Balanced,
    /// Optimize for overall network efficiency
    EfficiencyOptimized,
    /// Geographic distribution optimization
    GeographicBalanced,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyConfig {
    pub strategy: AllocationStrategy,
    pub weight: f64, // Influence in decision making
    pub parameters: HashMap<String, f64>,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResourcePriority {
    Low,
    Normal,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceAvailability {
    pub total_storage_gb: u64,
    pub available_storage_gb: u64,
    pub total_bandwidth_gbps: f64,
    pub available_bandwidth_gbps: f64,
    pub active_nodes: u32,
    pub node_availability: HashMap<String, NodeAvailability>,
    pub regional_availability: HashMap<String, RegionalAvailability>,
    pub last_updated: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeAvailability {
    pub node_id: String,
    pub available_storage_gb: u64,
    pub available_bandwidth_mbps: f64,
    pub current_load_percent: f64,
    pub reliability_score: f64,
    pub geographic_region: String,
    pub last_seen: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegionalAvailability {
    pub region: String,
    pub total_nodes: u32,
    pub active_nodes: u32,
    pub total_capacity_gb: u64,
    pub available_capacity_gb: u64,
    pub average_performance: f64,
    pub utilization_percent: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoricalAllocation {
    pub timestamp: DateTime<Utc>,
    pub user_id: String,
    pub resource_type: ResourceType,
    pub amount_requested: u64,
    pub amount_allocated: u64,
    pub decision: AllocationOutcome,
    pub contribution_score: f64,
    pub processing_time_ms: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllocationMetrics {
    pub total_requests: u64,
    pub approved_requests: u64,
    pub denied_requests: u64,
    pub average_processing_time_ms: u32,
    pub resource_utilization_percent: f64,
    pub user_satisfaction_score: f64,
    pub fairness_index: f64, // Gini coefficient or similar
    pub last_calculated: DateTime<Utc>,
}

impl ContributionBasedAllocator {
    pub fn new() -> Self {
        let mut strategies = HashMap::new();

        strategies.insert(AllocationStrategy::ContributionBased, StrategyConfig {
            strategy: AllocationStrategy::ContributionBased,
            weight: 0.4,
            parameters: {
                let mut params = HashMap::new();
                params.insert("min_ratio_threshold".to_string(), 1.0);
                params.insert("bonus_multiplier".to_string(), 1.5);
                params
            },
            enabled: true,
        });

        strategies.insert(AllocationStrategy::Balanced, StrategyConfig {
            strategy: AllocationStrategy::Balanced,
            weight: 0.3,
            parameters: HashMap::new(),
            enabled: true,
        });

        strategies.insert(AllocationStrategy::EfficiencyOptimized, StrategyConfig {
            strategy: AllocationStrategy::EfficiencyOptimized,
            weight: 0.3,
            parameters: HashMap::new(),
            enabled: true,
        });

        Self {
            allocation_cache: HashMap::new(),
            resource_availability: ResourceAvailability {
                total_storage_gb: 0,
                available_storage_gb: 0,
                total_bandwidth_gbps: 0.0,
                available_bandwidth_gbps: 0.0,
                active_nodes: 0,
                node_availability: HashMap::new(),
                regional_availability: HashMap::new(),
                last_updated: Utc::now(),
            },
            strategies,
            allocation_history: VecDeque::with_capacity(10000),
            performance_metrics: AllocationMetrics {
                total_requests: 0,
                approved_requests: 0,
                denied_requests: 0,
                average_processing_time_ms: 0,
                resource_utilization_percent: 0.0,
                user_satisfaction_score: 0.0,
                fairness_index: 1.0,
                last_calculated: Utc::now(),
            },
        }
    }

    /// Make allocation decision based on contribution and resource availability
    pub async fn allocate_resources(
        &mut self,
        request: AllocationRequest,
        contribution_tracker: &ContributionTracker,
    ) -> Result<AllocationDecision> {
        let start_time = Utc::now();

        // Get user contribution data
        let user_contribution = contribution_tracker.get_user_status(&request.user_id)
            .ok_or_else(|| anyhow::anyhow!("User not found in contribution tracker"))?;

        // Calculate allocation eligibility based on contribution
        let allocation_eligibility = self.calculate_allocation_eligibility(user_contribution, &request)?;

        // Make allocation decision
        let decision = if allocation_eligibility.eligible {
            self.make_allocation_decision(request.clone(), allocation_eligibility).await?
        } else {
            AllocationDecision {
                request_id: request.request_id.clone(),
                user_id: request.user_id.clone(),
                decision: AllocationOutcome::Denied,
                allocated_resources: None,
                reason: allocation_eligibility.reason,
                contribution_score_used: user_contribution.contribution_score,
                priority_level_used: user_contribution.priority_level.clone(),
                decided_at: start_time,
                expires_at: None,
            }
        };

        // Update metrics
        self.update_allocation_metrics(&decision, start_time);

        // Cache decision
        self.allocation_cache.insert(request.request_id.clone(), decision.clone());

        // Record in history
        self.record_allocation_history(&request, &decision, start_time);

        Ok(decision)
    }

    /// Calculate whether user is eligible for allocation based on contribution
    fn calculate_allocation_eligibility(
        &self,
        user_contribution: &UserContribution,
        request: &AllocationRequest,
    ) -> Result<AllocationEligibility> {

        // Check account status
        match user_contribution.account_status {
            crate::economics::AccountStatus::Suspended => {
                return Ok(AllocationEligibility {
                    eligible: false,
                    max_allocation_gb: 0,
                    quality_level: AllocationQuality::Basic,
                    reason: "Account suspended due to poor contribution ratio".to_string(),
                });
            },
            crate::economics::AccountStatus::Limited => {
                // Limited accounts get minimal allocation
                let max_allocation = (user_contribution.storage_offered_gb / 4).min(10); // Max 25% of contribution, capped at 10GB

                if let ResourceType::Storage { size_gb, .. } = &request.resource_type {
                    if *size_gb > max_allocation {
                        return Ok(AllocationEligibility {
                            eligible: false,
                            max_allocation_gb: max_allocation,
                            quality_level: AllocationQuality::Basic,
                            reason: format!("Limited account. Maximum allocation: {}GB", max_allocation),
                        });
                    }
                }
            },
            _ => {} // Continue with normal processing
        }

        // Calculate maximum allocation based on contribution level
        let allocation_multiplier = match user_contribution.priority_level {
            PriorityLevel::Deficit => 0.5,
            PriorityLevel::Balanced => 1.0,
            PriorityLevel::Surplus => 1.5,
            PriorityLevel::Generous => 2.0,
        };

        let max_allocation_gb = (user_contribution.storage_offered_gb as f64 * allocation_multiplier) as u64;

        // Check if request exceeds allowed allocation
        if let ResourceType::Storage { size_gb, .. } = &request.resource_type {
            if *size_gb > max_allocation_gb {
                return Ok(AllocationEligibility {
                    eligible: false,
                    max_allocation_gb,
                    quality_level: self.determine_quality_level(user_contribution),
                    reason: format!("Requested {}GB exceeds maximum allowed allocation of {}GB based on contribution level", size_gb, max_allocation_gb),
                });
            }
        }

        Ok(AllocationEligibility {
            eligible: true,
            max_allocation_gb,
            quality_level: self.determine_quality_level(user_contribution),
            reason: "Allocation approved based on contribution level".to_string(),
        })
    }

    /// Determine quality level based on contribution
    fn determine_quality_level(&self, user_contribution: &UserContribution) -> AllocationQuality {
        match user_contribution.priority_level {
            PriorityLevel::Deficit => AllocationQuality::Basic,
            PriorityLevel::Balanced => AllocationQuality::Standard,
            PriorityLevel::Surplus => AllocationQuality::Premium,
            PriorityLevel::Generous => AllocationQuality::Enterprise,
        }
    }

    /// Make the actual allocation decision
    async fn make_allocation_decision(
        &mut self,
        request: AllocationRequest,
        eligibility: AllocationEligibility,
    ) -> Result<AllocationDecision> {

        // Find available nodes that can fulfill the request
        let suitable_nodes = self.find_suitable_nodes(&request, &eligibility)?;

        if suitable_nodes.is_empty() {
            return Ok(AllocationDecision {
                request_id: request.request_id.clone(),
                user_id: request.user_id.clone(),
                decision: AllocationOutcome::Queued,
                allocated_resources: None,
                reason: "No suitable nodes available at this time. Request queued.".to_string(),
                contribution_score_used: 0.0,
                priority_level_used: PriorityLevel::Balanced,
                decided_at: Utc::now(),
                expires_at: Some(Utc::now() + Duration::hours(24)),
            });
        }

        // Create resource allocation
        let allocated_resources = self.create_resource_allocation(&request, &suitable_nodes, &eligibility)?;

        Ok(AllocationDecision {
            request_id: request.request_id,
            user_id: request.user_id,
            decision: AllocationOutcome::Approved,
            allocated_resources: Some(allocated_resources),
            reason: "Resources allocated successfully".to_string(),
            contribution_score_used: 0.0,
            priority_level_used: PriorityLevel::Balanced,
            decided_at: Utc::now(),
            expires_at: None,
        })
    }

    /// Find nodes suitable for the allocation request
    fn find_suitable_nodes(
        &self,
        request: &AllocationRequest,
        eligibility: &AllocationEligibility,
    ) -> Result<Vec<NodeAssignment>> {

        let mut suitable_nodes = Vec::new();

        // For now, return a simple mock implementation
        // In a real system, this would query actual node availability
        if self.resource_availability.active_nodes > 0 {
            suitable_nodes.push(NodeAssignment {
                node_id: "mock_node_1".to_string(),
                resource_portion: 1.0,
                role: NodeRole::Primary,
                expected_performance: NodePerformance {
                    uptime_percentage: 99.5,
                    response_time_ms: 50,
                    throughput_mbps: 100.0,
                    reliability_score: 0.95,
                },
            });
        }

        Ok(suitable_nodes)
    }

    /// Create the resource allocation details
    fn create_resource_allocation(
        &self,
        request: &AllocationRequest,
        nodes: &[NodeAssignment],
        eligibility: &AllocationEligibility,
    ) -> Result<AllocatedResources> {

        Ok(AllocatedResources {
            resource_type: request.resource_type.clone(),
            amount_allocated: match &request.resource_type {
                ResourceType::Storage { size_gb, .. } => *size_gb,
                ResourceType::Bandwidth { mbps, .. } => *mbps,
                ResourceType::Compute { cpu_cores, .. } => *cpu_cores as u64,
                ResourceType::Network { connections, .. } => *connections as u64,
            },
            quality_level: eligibility.quality_level.clone(),
            assigned_nodes: nodes.to_vec(),
            estimated_performance: PerformanceEstimate {
                expected_throughput_mbps: 100.0,
                expected_latency_ms: 50,
                expected_availability_percent: 99.5,
                confidence_level: 0.9,
            },
            sla_commitments: SLACommitments {
                uptime_guarantee: match eligibility.quality_level {
                    AllocationQuality::Basic => 95.0,
                    AllocationQuality::Standard => 99.0,
                    AllocationQuality::Premium => 99.5,
                    AllocationQuality::Enterprise => 99.9,
                },
                performance_guarantee: PerformanceGuarantee {
                    min_throughput_mbps: 10.0,
                    max_latency_ms: 100,
                    response_time_percentile: PercentileGuarantee {
                        p50_latency_ms: 50,
                        p95_latency_ms: 100,
                        p99_latency_ms: 200,
                    },
                },
                data_durability: 99.999,
                support_level: match eligibility.quality_level {
                    AllocationQuality::Basic => SupportLevel::Community,
                    AllocationQuality::Standard => SupportLevel::Standard,
                    AllocationQuality::Premium => SupportLevel::Premium,
                    AllocationQuality::Enterprise => SupportLevel::Enterprise,
                },
            },
        })
    }

    /// Update allocation performance metrics
    fn update_allocation_metrics(&mut self, decision: &AllocationDecision, start_time: DateTime<Utc>) {
        self.performance_metrics.total_requests += 1;

        match decision.decision {
            AllocationOutcome::Approved | AllocationOutcome::PartiallyApproved => {
                self.performance_metrics.approved_requests += 1;
            },
            AllocationOutcome::Denied => {
                self.performance_metrics.denied_requests += 1;
            },
            _ => {}
        }

        let processing_time_ms = (Utc::now() - start_time).num_milliseconds() as u32;

        // Update running average
        let total_requests = self.performance_metrics.total_requests;
        let current_avg = self.performance_metrics.average_processing_time_ms;
        self.performance_metrics.average_processing_time_ms =
            ((current_avg as u64 * (total_requests - 1)) + processing_time_ms as u64) as u32 / total_requests as u32;

        self.performance_metrics.last_calculated = Utc::now();
    }

    /// Record allocation in history
    fn record_allocation_history(&mut self, request: &AllocationRequest, decision: &AllocationDecision, start_time: DateTime<Utc>) {
        let amount_allocated = decision.allocated_resources.as_ref()
            .map(|r| r.amount_allocated)
            .unwrap_or(0);

        let record = HistoricalAllocation {
            timestamp: start_time,
            user_id: request.user_id.clone(),
            resource_type: request.resource_type.clone(),
            amount_requested: match &request.resource_type {
                ResourceType::Storage { size_gb, .. } => *size_gb,
                ResourceType::Bandwidth { mbps, .. } => *mbps,
                ResourceType::Compute { cpu_cores, .. } => *cpu_cores as u64,
                ResourceType::Network { connections, .. } => *connections as u64,
            },
            amount_allocated,
            decision: decision.decision.clone(),
            contribution_score: decision.contribution_score_used,
            processing_time_ms: (decision.decided_at - start_time).num_milliseconds() as u32,
        };

        self.allocation_history.push_back(record);

        // Keep only recent history
        if self.allocation_history.len() > 10000 {
            self.allocation_history.pop_front();
        }
    }

    /// Get allocation statistics
    pub fn get_allocation_stats(&self) -> &AllocationMetrics {
        &self.performance_metrics
    }

    /// Get resource availability
    pub fn get_resource_availability(&self) -> &ResourceAvailability {
        &self.resource_availability
    }
}

#[derive(Debug, Clone)]
struct AllocationEligibility {
    eligible: bool,
    max_allocation_gb: u64,
    quality_level: AllocationQuality,
    reason: String,
}

impl Default for ContributionBasedAllocator {
    fn default() -> Self {
        Self::new()
    }
}