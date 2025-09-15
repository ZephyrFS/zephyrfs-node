//! Contribution-Based Economic Manager
//!
//! Replaces token-based economics with contribution tracking and cooperative resource allocation

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc, Duration};

use super::contribution_tracker::{ContributionTracker, UserContribution, PriorityLevel, AccountStatus};

/// Main economic manager using contribution-based model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContributionEconomicManager {
    /// Contribution tracking system
    pub contribution_tracker: ContributionTracker,
    /// Resource allocation queue
    pub allocation_queue: AllocationQueue,
    /// Network resource management
    pub resource_manager: ResourceManager,
    /// Simple referral tracking
    pub referral_tracker: SimpleReferralTracker,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllocationQueue {
    /// Pending storage requests ordered by priority
    pub storage_requests: Vec<StorageRequest>,
    /// Active storage allocations
    pub active_allocations: HashMap<String, StorageAllocation>,
    /// Allocation history for analytics
    pub allocation_history: Vec<AllocationRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageRequest {
    pub request_id: String,
    pub user_id: String,
    pub requested_gb: u64,
    pub priority_score: u32,
    pub requested_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub requirements: StorageRequirements,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageRequirements {
    pub durability_level: DurabilityLevel,
    pub access_frequency: AccessFrequency,
    pub geographic_preference: Option<String>,
    pub max_latency_ms: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DurabilityLevel {
    /// Standard redundancy (3 copies)
    Standard,
    /// High redundancy (5 copies)
    High,
    /// Critical redundancy (7 copies)
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AccessFrequency {
    /// Rarely accessed, optimize for cost
    Cold,
    /// Occasionally accessed
    Warm,
    /// Frequently accessed, optimize for speed
    Hot,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageAllocation {
    pub allocation_id: String,
    pub user_id: String,
    pub allocated_gb: u64,
    pub allocated_to_nodes: Vec<String>,
    pub allocated_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub status: AllocationStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AllocationStatus {
    Active,
    Expired,
    Revoked,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllocationRecord {
    pub user_id: String,
    pub action: AllocationAction,
    pub amount_gb: u64,
    pub reason: String,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AllocationAction {
    Granted,
    Denied,
    Revoked,
    Expired,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceManager {
    /// Available storage capacity per node
    pub node_capacity: HashMap<String, NodeCapacity>,
    /// Current utilization statistics
    pub utilization_stats: UtilizationStats,
    /// Resource allocation policies
    pub allocation_policies: AllocationPolicies,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeCapacity {
    pub node_id: String,
    pub total_capacity_gb: u64,
    pub available_capacity_gb: u64,
    pub allocated_capacity_gb: u64,
    pub reliability_score: f64,
    pub performance_metrics: NodePerformance,
    pub last_updated: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodePerformance {
    pub uptime_percentage: f64,
    pub average_response_time_ms: u64,
    pub bandwidth_mbps: f64,
    pub success_rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UtilizationStats {
    pub total_network_capacity_gb: u64,
    pub total_allocated_gb: u64,
    pub total_available_gb: u64,
    pub utilization_percentage: f64,
    pub active_nodes: u32,
    pub last_updated: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllocationPolicies {
    /// Maximum allocation per user based on contribution level
    pub max_allocation_ratios: HashMap<PriorityLevel, f64>,
    /// Minimum contribution score for allocation
    pub min_contribution_score: f64,
    /// Grace period for new users (days)
    pub new_user_grace_period: u32,
    /// Allocation request timeout (hours)
    pub request_timeout_hours: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimpleReferralTracker {
    /// User referrals (referrer -> list of referred users)
    pub referrals: HashMap<String, Vec<ReferralRecord>>,
    /// Referral bonuses awarded
    pub bonuses: HashMap<String, Vec<ReferralBonus>>,
    /// Configuration
    pub config: ReferralConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReferralRecord {
    pub referred_user_id: String,
    pub referred_at: DateTime<Utc>,
    pub bonus_awarded: bool,
    pub bonus_awarded_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReferralBonus {
    pub bonus_gb: u64,
    pub reason: String,
    pub awarded_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReferralConfig {
    /// Storage bonus for successful referral (GB)
    pub referral_bonus_gb: u64,
    /// Minimum contribution from referee before bonus
    pub min_referee_contribution_gb: u64,
    /// Maximum number of referral bonuses per user
    pub max_referral_bonuses: u32,
    /// Bonus expiration time (days)
    pub bonus_expires_days: u32,
}

impl ContributionEconomicManager {
    pub fn new() -> Self {
        Self {
            contribution_tracker: ContributionTracker::new(),
            allocation_queue: AllocationQueue {
                storage_requests: Vec::new(),
                active_allocations: HashMap::new(),
                allocation_history: Vec::new(),
            },
            resource_manager: ResourceManager {
                node_capacity: HashMap::new(),
                utilization_stats: UtilizationStats {
                    total_network_capacity_gb: 0,
                    total_allocated_gb: 0,
                    total_available_gb: 0,
                    utilization_percentage: 0.0,
                    active_nodes: 0,
                    last_updated: Utc::now(),
                },
                allocation_policies: AllocationPolicies {
                    max_allocation_ratios: {
                        let mut ratios = HashMap::new();
                        ratios.insert(PriorityLevel::Deficit, 0.5);      // Can use 50% of what they offer
                        ratios.insert(PriorityLevel::Balanced, 1.0);     // Can use 100% of what they offer
                        ratios.insert(PriorityLevel::Surplus, 1.5);      // Can use 150% of what they offer
                        ratios.insert(PriorityLevel::Generous, 2.0);     // Can use 200% of what they offer
                        ratios
                    },
                    min_contribution_score: 0.5,
                    new_user_grace_period: 30,
                    request_timeout_hours: 24,
                },
            },
            referral_tracker: SimpleReferralTracker {
                referrals: HashMap::new(),
                bonuses: HashMap::new(),
                config: ReferralConfig {
                    referral_bonus_gb: 10,
                    min_referee_contribution_gb: 100,
                    max_referral_bonuses: 10,
                    bonus_expires_days: 365,
                },
            },
        }
    }

    /// Register a new node offering storage
    pub async fn register_node(&mut self, user_id: String, storage_gb: u64, performance: NodePerformance) -> Result<()> {
        // Register in contribution tracker
        self.contribution_tracker.register_user(user_id.clone(), storage_gb).await?;

        // Add node capacity
        self.resource_manager.node_capacity.insert(user_id.clone(), NodeCapacity {
            node_id: user_id,
            total_capacity_gb: storage_gb,
            available_capacity_gb: storage_gb,
            allocated_capacity_gb: 0,
            reliability_score: 1.0, // Start with neutral score
            performance_metrics: performance,
            last_updated: Utc::now(),
        });

        self.update_utilization_stats().await?;
        Ok(())
    }

    /// Request storage allocation
    pub async fn request_storage(&mut self, user_id: String, requested_gb: u64, requirements: StorageRequirements) -> Result<String> {
        // Check if user can make this request
        if !self.contribution_tracker.can_request_storage(&user_id, requested_gb)? {
            return Err(anyhow::anyhow!("Request denied: insufficient contribution or account status"));
        }

        // Get priority score
        let priority_score = self.contribution_tracker.get_allocation_priority(&user_id)?;

        // Create storage request
        let request_id = format!("req_{}", uuid::Uuid::new_v4());
        let request = StorageRequest {
            request_id: request_id.clone(),
            user_id: user_id.clone(),
            requested_gb,
            priority_score,
            requested_at: Utc::now(),
            expires_at: Utc::now() + Duration::hours(self.resource_manager.allocation_policies.request_timeout_hours as i64),
            requirements,
        };

        // Insert request in priority order
        self.allocation_queue.storage_requests.push(request);
        self.allocation_queue.storage_requests.sort_by(|a, b| b.priority_score.cmp(&a.priority_score));

        // Try to process immediately
        self.process_allocation_queue().await?;

        Ok(request_id)
    }

    /// Process pending storage allocations
    pub async fn process_allocation_queue(&mut self) -> Result<()> {
        let mut processed_requests = Vec::new();
        let mut allocations_to_record = Vec::new();
        let mut new_allocations = Vec::new();

        // First pass: collect requests to process without borrowing conflicts
        let requests_to_process: Vec<(usize, crate::allocation::AllocationRequest)> = self.allocation_queue.storage_requests
            .iter()
            .enumerate()
            .map(|(i, req)| (i, req.clone()))
            .collect();

        for (i, request) in requests_to_process {
            // Check if request expired
            if request.expires_at < Utc::now() {
                allocations_to_record.push((request.user_id.clone(), AllocationAction::Expired, request.requested_gb, "Request expired".to_string()));
                processed_requests.push(i);
                continue;
            }

            // Try to allocate storage
            match self.allocate_storage_for_request(&request).await {
                Ok(allocation) => {
                    new_allocations.push(allocation.clone());
                    allocations_to_record.push((request.user_id.clone(), AllocationAction::Granted, request.requested_gb, "Storage allocated successfully".to_string()));
                    processed_requests.push(i);
                },
                Err(_) => {
                    // Cannot allocate right now, leave in queue
                }
            }
        }

        // Record all allocations
        for (user_id, action, gb, message) in allocations_to_record {
            self.record_allocation(user_id, action, gb, message).await;
        }

        // Add new allocations
        for allocation in new_allocations {
            self.allocation_queue.active_allocations.insert(allocation.allocation_id.clone(), allocation);
        }

        // Remove processed requests (in reverse order to maintain indices)
        for &i in processed_requests.iter().rev() {
            self.allocation_queue.storage_requests.remove(i);
        }

        Ok(())
    }

    /// Allocate storage for a specific request
    async fn allocate_storage_for_request(&mut self, request: &StorageRequest) -> Result<StorageAllocation> {
        // Find suitable nodes based on requirements
        let suitable_nodes = self.find_suitable_nodes(&request.requirements, request.requested_gb)?;

        if suitable_nodes.is_empty() {
            return Err(anyhow::anyhow!("No suitable nodes available"));
        }

        // Update user's storage usage
        self.contribution_tracker.update_storage_usage(&request.user_id,
            self.contribution_tracker.get_user_status(&request.user_id).unwrap().storage_used_gb + request.requested_gb).await?;

        // Create allocation
        let allocation = StorageAllocation {
            allocation_id: format!("alloc_{}", uuid::Uuid::new_v4()),
            user_id: request.user_id.clone(),
            allocated_gb: request.requested_gb,
            allocated_to_nodes: suitable_nodes,
            allocated_at: Utc::now(),
            expires_at: None, // No expiration for now
            status: AllocationStatus::Active,
        };

        Ok(allocation)
    }

    /// Find suitable nodes for storage requirements
    fn find_suitable_nodes(&self, requirements: &StorageRequirements, needed_gb: u64) -> Result<Vec<String>> {
        let mut suitable_nodes: Vec<_> = self.resource_manager.node_capacity.values()
            .filter(|node| {
                node.available_capacity_gb >= needed_gb &&
                node.reliability_score >= 0.8 &&
                node.performance_metrics.uptime_percentage >= 95.0
            })
            .collect();

        // Sort by reliability and performance
        suitable_nodes.sort_by(|a, b| {
            b.reliability_score.partial_cmp(&a.reliability_score).unwrap()
                .then_with(|| b.performance_metrics.uptime_percentage.partial_cmp(&a.performance_metrics.uptime_percentage).unwrap())
        });

        // Return top nodes (could implement more sophisticated selection based on requirements)
        let num_nodes = match requirements.durability_level {
            DurabilityLevel::Standard => 3,
            DurabilityLevel::High => 5,
            DurabilityLevel::Critical => 7,
        };

        Ok(suitable_nodes.iter()
            .take(num_nodes.min(suitable_nodes.len()))
            .map(|node| node.node_id.clone())
            .collect())
    }

    /// Record allocation action in history
    async fn record_allocation(&mut self, user_id: String, action: AllocationAction, amount_gb: u64, reason: String) {
        let record = AllocationRecord {
            user_id,
            action,
            amount_gb,
            reason,
            timestamp: Utc::now(),
        };
        self.allocation_queue.allocation_history.push(record);
    }

    /// Update network utilization statistics
    async fn update_utilization_stats(&mut self) -> Result<()> {
        let mut total_capacity = 0u64;
        let mut total_allocated = 0u64;
        let mut active_nodes = 0u32;

        for capacity in self.resource_manager.node_capacity.values() {
            total_capacity += capacity.total_capacity_gb;
            total_allocated += capacity.allocated_capacity_gb;
            if capacity.available_capacity_gb > 0 {
                active_nodes += 1;
            }
        }

        self.resource_manager.utilization_stats = UtilizationStats {
            total_network_capacity_gb: total_capacity,
            total_allocated_gb: total_allocated,
            total_available_gb: total_capacity - total_allocated,
            utilization_percentage: if total_capacity > 0 {
                (total_allocated as f64 / total_capacity as f64) * 100.0
            } else {
                0.0
            },
            active_nodes,
            last_updated: Utc::now(),
        };

        Ok(())
    }

    /// Simple referral system - award bonus for successful referral
    pub async fn process_referral(&mut self, referrer_id: String, referred_user_id: String) -> Result<()> {
        // Check if referred user meets minimum contribution
        let referred_contribution = self.contribution_tracker.get_user_status(&referred_user_id)
            .ok_or_else(|| anyhow::anyhow!("Referred user not found"))?;

        if referred_contribution.storage_offered_gb >= self.referral_tracker.config.min_referee_contribution_gb {
            // Award bonus to both users
            let bonus = ReferralBonus {
                bonus_gb: self.referral_tracker.config.referral_bonus_gb,
                reason: format!("Referral bonus for {}", referred_user_id),
                awarded_at: Utc::now(),
                expires_at: Some(Utc::now() + Duration::days(self.referral_tracker.config.bonus_expires_days as i64)),
            };

            // Add bonus storage allocation for referrer
            self.referral_tracker.bonuses.entry(referrer_id.clone()).or_insert_with(Vec::new).push(bonus.clone());
            self.referral_tracker.bonuses.entry(referred_user_id.clone()).or_insert_with(Vec::new).push(bonus);

            // Record the referral
            let referral_record = ReferralRecord {
                referred_user_id: referred_user_id.clone(),
                referred_at: Utc::now(),
                bonus_awarded: true,
                bonus_awarded_at: Some(Utc::now()),
            };
            self.referral_tracker.referrals.entry(referrer_id).or_insert_with(Vec::new).push(referral_record);
        }

        Ok(())
    }

    /// Get contribution tracker reference
    pub fn get_contribution_tracker(&self) -> &ContributionTracker {
        &self.contribution_tracker
    }

    /// Get resource manager reference
    pub fn get_resource_manager(&self) -> &ResourceManager {
        &self.resource_manager
    }

    /// Get allocation queue reference
    pub fn get_allocation_queue(&self) -> &AllocationQueue {
        &self.allocation_queue
    }
}

impl Default for ContributionEconomicManager {
    fn default() -> Self {
        Self::new()
    }
}