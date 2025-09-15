//! Democratic space allocation algorithm for ZephyrFS
//!
//! Implements fair, transparent, and democratic allocation of storage space
//! across the network based on capacity, demand, and volunteer preferences.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, BTreeMap};
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

/// Configuration for democratic space allocation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllocationConfig {
    /// Minimum storage allocation per volunteer (GB)
    pub min_allocation_gb: f64,
    /// Maximum storage allocation per volunteer (GB)
    pub max_allocation_gb: f64,
    /// Target network utilization percentage (0.0-1.0)
    pub target_utilization: f64,
    /// Weight for capacity-based allocation (0.0-1.0)
    pub capacity_weight: f64,
    /// Weight for demand-based allocation (0.0-1.0)
    pub demand_weight: f64,
    /// Weight for performance-based allocation (0.0-1.0)
    pub performance_weight: f64,
    /// Rebalancing frequency (seconds)
    pub rebalancing_interval: u64,
    /// Enable fair queuing for requests
    pub enable_fair_queuing: bool,
}

impl Default for AllocationConfig {
    fn default() -> Self {
        Self {
            min_allocation_gb: 1.0,
            max_allocation_gb: 100.0,
            target_utilization: 0.75,
            capacity_weight: 0.4,
            demand_weight: 0.3,
            performance_weight: 0.3,
            rebalancing_interval: 3600, // 1 hour
            enable_fair_queuing: true,
        }
    }
}

/// Information about a storage volunteer node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VolunteerNode {
    /// Unique node identifier
    pub node_id: Uuid,
    /// Total available capacity (GB)
    pub total_capacity_gb: f64,
    /// Currently allocated space (GB)
    pub allocated_space_gb: f64,
    /// Currently used space (GB)
    pub used_space_gb: f64,
    /// Node performance metrics
    pub performance: NodePerformance,
    /// Node preferences and constraints
    pub preferences: NodePreferences,
    /// Geographic location (for distribution)
    pub location: Option<GeographicLocation>,
    /// Node reliability score (0.0-1.0)
    pub reliability_score: f64,
    /// Last seen timestamp
    pub last_seen: u64,
    /// Node status
    pub status: NodeStatus,
}

/// Node performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodePerformance {
    /// Average response time (milliseconds)
    pub avg_response_time_ms: f64,
    /// Bandwidth capacity (Mbps)
    pub bandwidth_mbps: f64,
    /// Uptime percentage (0.0-1.0)
    pub uptime_percentage: f64,
    /// Error rate (0.0-1.0)
    pub error_rate: f64,
    /// CPU usage percentage (0.0-1.0)
    pub cpu_usage: f64,
    /// Memory usage percentage (0.0-1.0)
    pub memory_usage: f64,
}

/// Node preferences and constraints
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodePreferences {
    /// Maximum space willing to provide (GB)
    pub max_contribution_gb: f64,
    /// Preferred operating hours (24-hour format)
    pub preferred_hours: Option<(u8, u8)>,
    /// Bandwidth throttling preferences
    pub bandwidth_limit_mbps: Option<f64>,
    /// Content type preferences
    pub content_preferences: Vec<String>,
    /// Minimum reward rate required
    pub min_reward_rate: Option<f64>,
}

/// Geographic location for distribution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeographicLocation {
    /// Country code
    pub country: String,
    /// Region/state
    pub region: String,
    /// City
    pub city: String,
    /// Latitude
    pub latitude: f64,
    /// Longitude
    pub longitude: f64,
}

/// Node operational status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NodeStatus {
    Active,
    Inactive,
    Maintenance,
    Overloaded,
    Error(String),
}

/// Storage allocation request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllocationRequest {
    /// Request identifier
    pub request_id: Uuid,
    /// Requesting user ID
    pub user_id: String,
    /// Requested storage amount (GB)
    pub requested_gb: f64,
    /// Priority level (0-10, higher is more urgent)
    pub priority: u8,
    /// Content type hint
    pub content_type: Option<String>,
    /// Geographic preference
    pub geo_preference: Option<String>,
    /// Performance requirements
    pub performance_requirements: PerformanceRequirements,
    /// Request timestamp
    pub created_at: u64,
}

/// Performance requirements for allocation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceRequirements {
    /// Maximum acceptable latency (milliseconds)
    pub max_latency_ms: Option<f64>,
    /// Minimum bandwidth requirement (Mbps)
    pub min_bandwidth_mbps: Option<f64>,
    /// Minimum uptime requirement (0.0-1.0)
    pub min_uptime: Option<f64>,
    /// Redundancy factor (number of copies)
    pub redundancy_factor: u8,
}

/// Result of space allocation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllocationResult {
    /// Request this allocation fulfills
    pub request_id: Uuid,
    /// Allocated nodes and their contributions
    pub allocations: Vec<NodeAllocation>,
    /// Total allocated space (GB)
    pub total_allocated_gb: f64,
    /// Allocation quality score (0.0-1.0)
    pub quality_score: f64,
    /// Allocation strategy used
    pub strategy: AllocationStrategy,
    /// Allocation timestamp
    pub allocated_at: u64,
}

/// Individual node allocation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeAllocation {
    /// Node receiving the allocation
    pub node_id: Uuid,
    /// Amount allocated to this node (GB)
    pub allocated_gb: f64,
    /// Expected reward for this allocation
    pub reward_amount: f64,
    /// Allocation priority on this node
    pub priority: u8,
    /// Performance score for this allocation
    pub performance_score: f64,
}

/// Allocation strategy used
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum AllocationStrategy {
    /// Capacity-based allocation
    CapacityBased,
    /// Performance-based allocation
    PerformanceBased,
    /// Geographic distribution
    GeographicDistribution,
    /// Load balancing
    LoadBalancing,
    /// Hybrid approach
    Hybrid,
}

/// Network allocation statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllocationStats {
    /// Total network capacity (GB)
    pub total_capacity_gb: f64,
    /// Total allocated space (GB)
    pub total_allocated_gb: f64,
    /// Total used space (GB)
    pub total_used_gb: f64,
    /// Network utilization (0.0-1.0)
    pub utilization: f64,
    /// Number of active nodes
    pub active_nodes: usize,
    /// Number of pending requests
    pub pending_requests: usize,
    /// Average allocation quality
    pub avg_quality_score: f64,
}

/// Democratic space allocation engine
pub struct DemocraticAllocator {
    config: AllocationConfig,
    volunteer_nodes: HashMap<Uuid, VolunteerNode>,
    pending_requests: BTreeMap<u64, AllocationRequest>, // Ordered by timestamp
    active_allocations: HashMap<Uuid, AllocationResult>,
    allocation_history: Vec<AllocationResult>,
}

impl DemocraticAllocator {
    /// Create new democratic allocator
    pub fn new(config: AllocationConfig) -> Self {
        Self {
            config,
            volunteer_nodes: HashMap::new(),
            pending_requests: BTreeMap::new(),
            active_allocations: HashMap::new(),
            allocation_history: Vec::new(),
        }
    }

    /// Register a new volunteer node
    pub fn register_volunteer(&mut self, node: VolunteerNode) -> Result<()> {
        self.volunteer_nodes.insert(node.node_id, node);
        Ok(())
    }

    /// Update volunteer node information
    pub fn update_volunteer(&mut self, node_id: Uuid, updates: VolunteerNodeUpdate) -> Result<()> {
        if let Some(node) = self.volunteer_nodes.get_mut(&node_id) {
            self.apply_node_updates(node, updates);
            Ok(())
        } else {
            Err(anyhow::anyhow!("Volunteer node not found: {}", node_id))
        }
    }

    /// Submit a storage allocation request
    pub fn request_allocation(&mut self, mut request: AllocationRequest) -> Result<Uuid> {
        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .context("Failed to get timestamp")?
            .as_secs();

        request.created_at = current_time;
        let request_id = request.request_id;

        self.pending_requests.insert(current_time, request);
        Ok(request_id)
    }

    /// Process pending allocation requests
    pub fn process_allocations(&mut self) -> Result<Vec<AllocationResult>> {
        let mut results = Vec::new();

        // Process requests in order (FIFO with priority consideration)
        let requests_to_process: Vec<_> = self.pending_requests.values().cloned().collect();

        for request in requests_to_process {
            if let Ok(result) = self.allocate_storage(&request) {
                // Remove from pending
                self.pending_requests.retain(|_, r| r.request_id != request.request_id);

                // Store result
                self.active_allocations.insert(result.request_id, result.clone());
                self.allocation_history.push(result.clone());
                results.push(result);
            }
        }

        Ok(results)
    }

    /// Allocate storage for a specific request
    pub fn allocate_storage(&self, request: &AllocationRequest) -> Result<AllocationResult> {
        // Filter eligible nodes
        let eligible_nodes = self.filter_eligible_nodes(request)?;

        if eligible_nodes.is_empty() {
            return Err(anyhow::anyhow!("No eligible nodes available for allocation"));
        }

        // Calculate optimal allocation strategy
        let strategy = self.determine_allocation_strategy(request, &eligible_nodes);

        // Perform allocation based on strategy
        let allocations = match strategy {
            AllocationStrategy::CapacityBased => self.allocate_by_capacity(request, &eligible_nodes)?,
            AllocationStrategy::PerformanceBased => self.allocate_by_performance(request, &eligible_nodes)?,
            AllocationStrategy::GeographicDistribution => self.allocate_by_geography(request, &eligible_nodes)?,
            AllocationStrategy::LoadBalancing => self.allocate_by_load_balancing(request, &eligible_nodes)?,
            AllocationStrategy::Hybrid => self.allocate_hybrid(request, &eligible_nodes)?,
        };

        let total_allocated_gb = allocations.iter().map(|a| a.allocated_gb).sum();
        let quality_score = self.calculate_allocation_quality(&allocations, request);

        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .context("Failed to get timestamp")?
            .as_secs();

        Ok(AllocationResult {
            request_id: request.request_id,
            allocations,
            total_allocated_gb,
            quality_score,
            strategy,
            allocated_at: current_time,
        })
    }

    /// Get network allocation statistics
    pub fn get_allocation_stats(&self) -> AllocationStats {
        let total_capacity_gb = self.volunteer_nodes.values()
            .map(|n| n.total_capacity_gb)
            .sum();

        let total_allocated_gb = self.volunteer_nodes.values()
            .map(|n| n.allocated_space_gb)
            .sum();

        let total_used_gb = self.volunteer_nodes.values()
            .map(|n| n.used_space_gb)
            .sum();

        let utilization = if total_capacity_gb > 0.0 {
            total_used_gb / total_capacity_gb
        } else {
            0.0
        };

        let active_nodes = self.volunteer_nodes.values()
            .filter(|n| matches!(n.status, NodeStatus::Active))
            .count();

        let avg_quality_score = if !self.allocation_history.is_empty() {
            self.allocation_history.iter()
                .map(|a| a.quality_score)
                .sum::<f64>() / self.allocation_history.len() as f64
        } else {
            0.0
        };

        AllocationStats {
            total_capacity_gb,
            total_allocated_gb,
            total_used_gb,
            utilization,
            active_nodes,
            pending_requests: self.pending_requests.len(),
            avg_quality_score,
        }
    }

    /// Rebalance allocations across the network
    pub fn rebalance_network(&mut self) -> Result<Vec<RebalancingAction>> {
        let stats = self.get_allocation_stats();
        let mut actions = Vec::new();

        // Check if rebalancing is needed
        if stats.utilization > self.config.target_utilization + 0.1 ||
           stats.utilization < self.config.target_utilization - 0.2 {

            // Calculate optimal rebalancing moves
            let rebalancing_plan = self.calculate_rebalancing_plan(&stats)?;

            for action in rebalancing_plan {
                actions.push(action);
            }
        }

        Ok(actions)
    }

    /// Filter nodes eligible for a request
    fn filter_eligible_nodes(&self, request: &AllocationRequest) -> Result<Vec<&VolunteerNode>> {
        Ok(self.volunteer_nodes.values()
            .filter(|node| {
                // Basic eligibility checks
                matches!(node.status, NodeStatus::Active) &&
                node.total_capacity_gb - node.allocated_space_gb >= 0.1 && // At least 100MB free
                node.performance.uptime_percentage >= 0.9 && // At least 90% uptime
                self.meets_performance_requirements(node, &request.performance_requirements)
            })
            .collect())
    }

    /// Check if node meets performance requirements
    fn meets_performance_requirements(&self, node: &VolunteerNode, reqs: &PerformanceRequirements) -> bool {
        if let Some(max_latency) = reqs.max_latency_ms {
            if node.performance.avg_response_time_ms > max_latency {
                return false;
            }
        }

        if let Some(min_bandwidth) = reqs.min_bandwidth_mbps {
            if node.performance.bandwidth_mbps < min_bandwidth {
                return false;
            }
        }

        if let Some(min_uptime) = reqs.min_uptime {
            if node.performance.uptime_percentage < min_uptime {
                return false;
            }
        }

        true
    }

    /// Determine optimal allocation strategy
    fn determine_allocation_strategy(
        &self,
        request: &AllocationRequest,
        eligible_nodes: &[&VolunteerNode],
    ) -> AllocationStrategy {
        // Simple heuristic-based strategy selection
        if request.performance_requirements.min_bandwidth_mbps.is_some() ||
           request.performance_requirements.max_latency_ms.is_some() {
            AllocationStrategy::PerformanceBased
        } else if request.geo_preference.is_some() {
            AllocationStrategy::GeographicDistribution
        } else if eligible_nodes.len() > 10 {
            AllocationStrategy::LoadBalancing
        } else {
            AllocationStrategy::Hybrid
        }
    }

    /// Allocate storage based on node capacity
    fn allocate_by_capacity(
        &self,
        request: &AllocationRequest,
        eligible_nodes: &[&VolunteerNode],
    ) -> Result<Vec<NodeAllocation>> {
        let mut allocations = Vec::new();
        let mut remaining_gb = request.requested_gb;

        // Sort nodes by available capacity (descending)
        let mut sorted_nodes: Vec<_> = eligible_nodes.iter().collect();
        sorted_nodes.sort_by(|a, b| {
            let a_available = a.total_capacity_gb - a.allocated_space_gb;
            let b_available = b.total_capacity_gb - b.allocated_space_gb;
            b_available.partial_cmp(&a_available).unwrap()
        });

        for node in sorted_nodes {
            if remaining_gb <= 0.0 {
                break;
            }

            let available_gb = node.total_capacity_gb - node.allocated_space_gb;
            let to_allocate = remaining_gb.min(available_gb).min(self.config.max_allocation_gb);

            if to_allocate >= self.config.min_allocation_gb {
                allocations.push(NodeAllocation {
                    node_id: node.node_id,
                    allocated_gb: to_allocate,
                    reward_amount: to_allocate * 0.01, // $0.01 per GB
                    priority: request.priority,
                    performance_score: self.calculate_node_performance_score(node),
                });

                remaining_gb -= to_allocate;
            }
        }

        Ok(allocations)
    }

    /// Allocate storage based on node performance
    fn allocate_by_performance(
        &self,
        request: &AllocationRequest,
        eligible_nodes: &[&VolunteerNode],
    ) -> Result<Vec<NodeAllocation>> {
        let mut allocations = Vec::new();
        let mut remaining_gb = request.requested_gb;

        // Sort nodes by performance score (descending)
        let mut sorted_nodes: Vec<_> = eligible_nodes.iter()
            .map(|node| (node, self.calculate_node_performance_score(node)))
            .collect();
        sorted_nodes.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        for (node, performance_score) in sorted_nodes {
            if remaining_gb <= 0.0 {
                break;
            }

            let available_gb = node.total_capacity_gb - node.allocated_space_gb;
            let to_allocate = remaining_gb.min(available_gb).min(self.config.max_allocation_gb);

            if to_allocate >= self.config.min_allocation_gb {
                allocations.push(NodeAllocation {
                    node_id: node.node_id,
                    allocated_gb: to_allocate,
                    reward_amount: to_allocate * 0.015 * performance_score, // Performance bonus
                    priority: request.priority,
                    performance_score,
                });

                remaining_gb -= to_allocate;
            }
        }

        Ok(allocations)
    }

    /// Allocate storage based on geographic distribution
    fn allocate_by_geography(
        &self,
        request: &AllocationRequest,
        eligible_nodes: &[&VolunteerNode],
    ) -> Result<Vec<NodeAllocation>> {
        // For now, fall back to capacity-based allocation
        // In a real implementation, this would consider geographic distribution
        self.allocate_by_capacity(request, eligible_nodes)
    }

    /// Allocate storage using load balancing
    fn allocate_by_load_balancing(
        &self,
        request: &AllocationRequest,
        eligible_nodes: &[&VolunteerNode],
    ) -> Result<Vec<NodeAllocation>> {
        let mut allocations = Vec::new();
        let mut remaining_gb = request.requested_gb;

        // Sort nodes by current utilization (ascending)
        let mut sorted_nodes: Vec<_> = eligible_nodes.iter().collect();
        sorted_nodes.sort_by(|a, b| {
            let a_utilization = a.used_space_gb / a.total_capacity_gb;
            let b_utilization = b.used_space_gb / b.total_capacity_gb;
            a_utilization.partial_cmp(&b_utilization).unwrap()
        });

        for node in sorted_nodes {
            if remaining_gb <= 0.0 {
                break;
            }

            let available_gb = node.total_capacity_gb - node.allocated_space_gb;
            let to_allocate = remaining_gb.min(available_gb).min(self.config.max_allocation_gb);

            if to_allocate >= self.config.min_allocation_gb {
                allocations.push(NodeAllocation {
                    node_id: node.node_id,
                    allocated_gb: to_allocate,
                    reward_amount: to_allocate * 0.01,
                    priority: request.priority,
                    performance_score: self.calculate_node_performance_score(node),
                });

                remaining_gb -= to_allocate;
            }
        }

        Ok(allocations)
    }

    /// Allocate storage using hybrid approach
    fn allocate_hybrid(
        &self,
        request: &AllocationRequest,
        eligible_nodes: &[&VolunteerNode],
    ) -> Result<Vec<NodeAllocation>> {
        // Hybrid scoring based on capacity, performance, and load
        let mut node_scores: Vec<_> = eligible_nodes.iter()
            .map(|node| {
                let available_gb = node.total_capacity_gb - node.allocated_space_gb;
                let utilization = node.used_space_gb / node.total_capacity_gb;
                let performance_score = self.calculate_node_performance_score(node);

                let capacity_score = available_gb / self.config.max_allocation_gb;
                let load_score = 1.0 - utilization;

                let hybrid_score =
                    self.config.capacity_weight * capacity_score +
                    self.config.performance_weight * performance_score +
                    (1.0 - self.config.capacity_weight - self.config.performance_weight) * load_score;

                (node, hybrid_score)
            })
            .collect();

        // Sort by hybrid score (descending)
        node_scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        let mut allocations = Vec::new();
        let mut remaining_gb = request.requested_gb;

        for (node, score) in node_scores {
            if remaining_gb <= 0.0 {
                break;
            }

            let available_gb = node.total_capacity_gb - node.allocated_space_gb;
            let to_allocate = remaining_gb.min(available_gb).min(self.config.max_allocation_gb);

            if to_allocate >= self.config.min_allocation_gb {
                allocations.push(NodeAllocation {
                    node_id: node.node_id,
                    allocated_gb: to_allocate,
                    reward_amount: to_allocate * 0.012 * score, // Score-based reward
                    priority: request.priority,
                    performance_score: self.calculate_node_performance_score(node),
                });

                remaining_gb -= to_allocate;
            }
        }

        Ok(allocations)
    }

    /// Calculate node performance score
    fn calculate_node_performance_score(&self, node: &VolunteerNode) -> f64 {
        let latency_score = (1000.0 - node.performance.avg_response_time_ms).max(0.0) / 1000.0;
        let bandwidth_score = (node.performance.bandwidth_mbps / 100.0).min(1.0);
        let uptime_score = node.performance.uptime_percentage;
        let reliability_score = node.reliability_score;
        let error_score = 1.0 - node.performance.error_rate;

        (latency_score + bandwidth_score + uptime_score + reliability_score + error_score) / 5.0
    }

    /// Calculate quality score for an allocation
    fn calculate_allocation_quality(&self, allocations: &[NodeAllocation], request: &AllocationRequest) -> f64 {
        if allocations.is_empty() {
            return 0.0;
        }

        let total_allocated = allocations.iter().map(|a| a.allocated_gb).sum::<f64>();
        let fulfillment_ratio = (total_allocated / request.requested_gb).min(1.0);

        let avg_performance = allocations.iter()
            .map(|a| a.performance_score)
            .sum::<f64>() / allocations.len() as f64;

        let diversity_bonus = if allocations.len() > 1 { 0.1 } else { 0.0 };

        (fulfillment_ratio * 0.6 + avg_performance * 0.3 + diversity_bonus).min(1.0)
    }

    /// Apply updates to a volunteer node
    fn apply_node_updates(&self, node: &mut VolunteerNode, updates: VolunteerNodeUpdate) {
        if let Some(capacity) = updates.total_capacity_gb {
            node.total_capacity_gb = capacity;
        }
        if let Some(used) = updates.used_space_gb {
            node.used_space_gb = used;
        }
        if let Some(performance) = updates.performance {
            node.performance = performance;
        }
        if let Some(status) = updates.status {
            node.status = status;
        }
    }

    /// Calculate rebalancing plan
    fn calculate_rebalancing_plan(&self, _stats: &AllocationStats) -> Result<Vec<RebalancingAction>> {
        // Simplified rebalancing - in production this would be more sophisticated
        Ok(vec![])
    }
}

/// Updates for volunteer node information
#[derive(Debug, Clone)]
pub struct VolunteerNodeUpdate {
    pub total_capacity_gb: Option<f64>,
    pub used_space_gb: Option<f64>,
    pub performance: Option<NodePerformance>,
    pub status: Option<NodeStatus>,
}

/// Rebalancing action
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RebalancingAction {
    MoveAllocation {
        from_node: Uuid,
        to_node: Uuid,
        amount_gb: f64,
    },
    ScaleUp {
        node_id: Uuid,
        additional_gb: f64,
    },
    ScaleDown {
        node_id: Uuid,
        reduction_gb: f64,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_node(node_id: Uuid, capacity_gb: f64) -> VolunteerNode {
        VolunteerNode {
            node_id,
            total_capacity_gb: capacity_gb,
            allocated_space_gb: 0.0,
            used_space_gb: 0.0,
            performance: NodePerformance {
                avg_response_time_ms: 50.0,
                bandwidth_mbps: 100.0,
                uptime_percentage: 0.99,
                error_rate: 0.01,
                cpu_usage: 0.3,
                memory_usage: 0.4,
            },
            preferences: NodePreferences {
                max_contribution_gb: capacity_gb,
                preferred_hours: None,
                bandwidth_limit_mbps: None,
                content_preferences: vec![],
                min_reward_rate: None,
            },
            location: None,
            reliability_score: 0.95,
            last_seen: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            status: NodeStatus::Active,
        }
    }

    #[test]
    fn test_democratic_allocation_basic() -> Result<()> {
        let config = AllocationConfig::default();
        let mut allocator = DemocraticAllocator::new(config);

        // Register some volunteer nodes
        let node1 = create_test_node(Uuid::new_v4(), 10.0);
        let node2 = create_test_node(Uuid::new_v4(), 20.0);
        let node3 = create_test_node(Uuid::new_v4(), 15.0);

        allocator.register_volunteer(node1)?;
        allocator.register_volunteer(node2)?;
        allocator.register_volunteer(node3)?;

        // Create allocation request
        let request = AllocationRequest {
            request_id: Uuid::new_v4(),
            user_id: "test-user".to_string(),
            requested_gb: 5.0,
            priority: 5,
            content_type: None,
            geo_preference: None,
            performance_requirements: PerformanceRequirements {
                max_latency_ms: None,
                min_bandwidth_mbps: None,
                min_uptime: None,
                redundancy_factor: 1,
            },
            created_at: 0, // Will be set by request_allocation
        };

        let request_id = allocator.request_allocation(request)?;
        let results = allocator.process_allocations()?;

        assert!(!results.is_empty());
        assert_eq!(results[0].request_id, request_id);
        assert!(results[0].total_allocated_gb > 0.0);

        Ok(())
    }

    #[test]
    fn test_allocation_stats() {
        let config = AllocationConfig::default();
        let mut allocator = DemocraticAllocator::new(config);

        // Add some nodes
        allocator.register_volunteer(create_test_node(Uuid::new_v4(), 10.0)).unwrap();
        allocator.register_volunteer(create_test_node(Uuid::new_v4(), 20.0)).unwrap();

        let stats = allocator.get_allocation_stats();

        assert_eq!(stats.total_capacity_gb, 30.0);
        assert_eq!(stats.active_nodes, 2);
        assert_eq!(stats.utilization, 0.0); // No usage yet
    }

    #[test]
    fn test_node_performance_score() {
        let config = AllocationConfig::default();
        let allocator = DemocraticAllocator::new(config);
        let node = create_test_node(Uuid::new_v4(), 10.0);

        let score = allocator.calculate_node_performance_score(&node);
        assert!(score > 0.8); // Should be high score for good test node
        assert!(score <= 1.0);
    }
}