//! Contribution-Based Credit System
//!
//! Tracks storage and bandwidth contributions vs usage to allocate network access

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc, Duration};

/// Contribution tracking for users in the network
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContributionTracker {
    /// Per-user contribution records
    pub user_contributions: HashMap<String, UserContribution>,
    /// Network-wide statistics
    pub network_stats: NetworkContributionStats,
    /// Configuration for contribution requirements
    pub config: ContributionConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserContribution {
    pub user_id: String,
    /// Storage currently offered to network (in GB)
    pub storage_offered_gb: u64,
    /// Storage currently being used by this user (in GB)
    pub storage_used_gb: u64,
    /// Bandwidth offered (average over last 30 days, in Mbps)
    pub bandwidth_offered_mbps: f64,
    /// Bandwidth used (average over last 30 days, in Mbps)
    pub bandwidth_used_mbps: f64,
    /// Reliability metrics
    pub uptime_percentage: f64,
    pub response_time_ms: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    /// Contribution score (calculated from ratios and reliability)
    pub contribution_score: f64,
    /// Priority level for resource allocation
    pub priority_level: PriorityLevel,
    /// Account status
    pub account_status: AccountStatus,
    /// Timestamps
    pub joined_at: DateTime<Utc>,
    pub last_active: DateTime<Utc>,
    pub last_calculated: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum PriorityLevel {
    /// User taking more than giving - lowest priority
    Deficit,
    /// User giving slightly more than taking - normal priority
    Balanced,
    /// User giving significantly more than taking - high priority
    Surplus,
    /// User giving much more than taking - highest priority
    Generous,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AccountStatus {
    /// Account in good standing
    Active,
    /// Warning about low contribution ratio
    Warning,
    /// Limited access due to poor contribution ratio
    Limited,
    /// Access suspended due to not contributing
    Suspended,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkContributionStats {
    pub total_storage_offered_gb: u64,
    pub total_storage_used_gb: u64,
    pub total_bandwidth_offered_mbps: f64,
    pub total_bandwidth_used_mbps: f64,
    pub active_contributors: u32,
    pub network_utilization_percent: f64,
    pub average_contribution_score: f64,
    pub last_updated: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContributionConfig {
    /// Minimum contribution ratio to maintain good standing (1.0 = equal give/take)
    pub min_contribution_ratio: f64,
    /// Warning threshold ratio (below this triggers warning)
    pub warning_ratio: f64,
    /// Suspension threshold ratio (below this suspends access)
    pub suspension_ratio: f64,
    /// Minimum storage offering to participate (in GB)
    pub min_storage_offering_gb: u64,
    /// Grace period for new users (days)
    pub new_user_grace_days: u32,
    /// Weight of reliability in contribution score (0.0-1.0)
    pub reliability_weight: f64,
    /// Weight of storage ratio in contribution score (0.0-1.0)
    pub storage_ratio_weight: f64,
    /// Weight of bandwidth ratio in contribution score (0.0-1.0)
    pub bandwidth_ratio_weight: f64,
}

impl ContributionTracker {
    pub fn new() -> Self {
        Self {
            user_contributions: HashMap::new(),
            network_stats: NetworkContributionStats {
                total_storage_offered_gb: 0,
                total_storage_used_gb: 0,
                total_bandwidth_offered_mbps: 0.0,
                total_bandwidth_used_mbps: 0.0,
                active_contributors: 0,
                network_utilization_percent: 0.0,
                average_contribution_score: 0.0,
                last_updated: Utc::now(),
            },
            config: ContributionConfig {
                min_contribution_ratio: 1.0,  // Must give at least as much as you take
                warning_ratio: 0.8,           // Warning at 80%
                suspension_ratio: 0.5,        // Suspend at 50%
                min_storage_offering_gb: 10,  // Minimum 10GB to participate
                new_user_grace_days: 30,      // 30 day grace period
                reliability_weight: 0.3,      // 30% from reliability
                storage_ratio_weight: 0.4,    // 40% from storage contribution
                bandwidth_ratio_weight: 0.3,  // 30% from bandwidth contribution
            },
        }
    }

    /// Register a new user in the contribution system
    pub async fn register_user(&mut self, user_id: String, initial_storage_gb: u64) -> Result<()> {
        if self.user_contributions.contains_key(&user_id) {
            return Err(anyhow::anyhow!("User already registered"));
        }

        if initial_storage_gb < self.config.min_storage_offering_gb {
            return Err(anyhow::anyhow!("Initial storage offering too low. Minimum: {} GB",
                self.config.min_storage_offering_gb));
        }

        let user_contribution = UserContribution {
            user_id: user_id.clone(),
            storage_offered_gb: initial_storage_gb,
            storage_used_gb: 0,
            bandwidth_offered_mbps: 0.0,
            bandwidth_used_mbps: 0.0,
            uptime_percentage: 100.0,
            response_time_ms: 50,
            successful_requests: 0,
            failed_requests: 0,
            contribution_score: 1.0, // Start with neutral score
            priority_level: PriorityLevel::Balanced,
            account_status: AccountStatus::Active,
            joined_at: Utc::now(),
            last_active: Utc::now(),
            last_calculated: Utc::now(),
        };

        self.user_contributions.insert(user_id, user_contribution);
        self.update_network_stats().await?;

        Ok(())
    }

    /// Update user's storage offering
    pub async fn update_storage_offering(&mut self, user_id: &str, new_offering_gb: u64) -> Result<()> {
        let contribution = self.user_contributions.get_mut(user_id)
            .ok_or_else(|| anyhow::anyhow!("User not found"))?;

        if new_offering_gb < self.config.min_storage_offering_gb {
            return Err(anyhow::anyhow!("Storage offering too low. Minimum: {} GB",
                self.config.min_storage_offering_gb));
        }

        contribution.storage_offered_gb = new_offering_gb;
        contribution.last_active = Utc::now();

        self.recalculate_contribution_score(user_id).await?;
        self.update_network_stats().await?;

        Ok(())
    }

    /// Update user's storage usage
    pub async fn update_storage_usage(&mut self, user_id: &str, storage_used_gb: u64) -> Result<()> {
        let contribution = self.user_contributions.get_mut(user_id)
            .ok_or_else(|| anyhow::anyhow!("User not found"))?;

        contribution.storage_used_gb = storage_used_gb;
        contribution.last_active = Utc::now();

        self.recalculate_contribution_score(user_id).await?;
        self.update_network_stats().await?;

        Ok(())
    }

    /// Update user's bandwidth metrics
    pub async fn update_bandwidth_metrics(&mut self, user_id: &str, offered_mbps: f64, used_mbps: f64) -> Result<()> {
        let contribution = self.user_contributions.get_mut(user_id)
            .ok_or_else(|| anyhow::anyhow!("User not found"))?;

        contribution.bandwidth_offered_mbps = offered_mbps;
        contribution.bandwidth_used_mbps = used_mbps;
        contribution.last_active = Utc::now();

        self.recalculate_contribution_score(user_id).await?;

        Ok(())
    }

    /// Update user's reliability metrics
    pub async fn update_reliability_metrics(
        &mut self,
        user_id: &str,
        uptime_percentage: f64,
        response_time_ms: u64,
        successful_requests: u64,
        failed_requests: u64
    ) -> Result<()> {
        let contribution = self.user_contributions.get_mut(user_id)
            .ok_or_else(|| anyhow::anyhow!("User not found"))?;

        contribution.uptime_percentage = uptime_percentage;
        contribution.response_time_ms = response_time_ms;
        contribution.successful_requests = successful_requests;
        contribution.failed_requests = failed_requests;
        contribution.last_active = Utc::now();

        self.recalculate_contribution_score(user_id).await?;

        Ok(())
    }

    /// Recalculate contribution score for a user
    pub async fn recalculate_contribution_score(&mut self, user_id: &str) -> Result<()> {
        let contribution = self.user_contributions.get_mut(user_id)
            .ok_or_else(|| anyhow::anyhow!("User not found"))?;

        // Calculate storage ratio (offered / used, but handle zero usage)
        let storage_ratio = if contribution.storage_used_gb == 0 {
            2.0 // If not using storage, give good ratio
        } else {
            contribution.storage_offered_gb as f64 / contribution.storage_used_gb as f64
        };

        // Calculate bandwidth ratio
        let bandwidth_ratio = if contribution.bandwidth_used_mbps == 0.0 {
            2.0 // If not using bandwidth, give good ratio
        } else {
            contribution.bandwidth_offered_mbps / contribution.bandwidth_used_mbps
        };

        // Calculate reliability score (0.0-1.0)
        let total_requests = contribution.successful_requests + contribution.failed_requests;
        let success_rate = if total_requests == 0 {
            1.0
        } else {
            contribution.successful_requests as f64 / total_requests as f64
        };

        let uptime_score = contribution.uptime_percentage / 100.0;
        let response_score = (1000.0 - contribution.response_time_ms as f64).max(0.0) / 1000.0;
        let reliability_score = (success_rate + uptime_score + response_score) / 3.0;

        // Weighted contribution score
        let score = (storage_ratio * self.config.storage_ratio_weight) +
                   (bandwidth_ratio * self.config.bandwidth_ratio_weight) +
                   (reliability_score * self.config.reliability_weight);

        contribution.contribution_score = score;

        // Update priority level based on score
        contribution.priority_level = if score >= 2.0 {
            PriorityLevel::Generous
        } else if score >= 1.5 {
            PriorityLevel::Surplus
        } else if score >= self.config.min_contribution_ratio {
            PriorityLevel::Balanced
        } else {
            PriorityLevel::Deficit
        };

        // Update account status
        let is_new_user = (Utc::now() - contribution.joined_at).num_days() < self.config.new_user_grace_days as i64;

        contribution.account_status = if is_new_user {
            AccountStatus::Active // Grace period for new users
        } else if score < self.config.suspension_ratio {
            AccountStatus::Suspended
        } else if score < self.config.warning_ratio {
            AccountStatus::Limited
        } else if score < self.config.min_contribution_ratio {
            AccountStatus::Warning
        } else {
            AccountStatus::Active
        };

        contribution.last_calculated = Utc::now();

        Ok(())
    }

    /// Update network-wide statistics
    async fn update_network_stats(&mut self) -> Result<()> {
        let mut total_storage_offered = 0u64;
        let mut total_storage_used = 0u64;
        let mut total_bandwidth_offered = 0.0f64;
        let mut total_bandwidth_used = 0.0f64;
        let mut total_score = 0.0f64;
        let mut active_count = 0u32;

        for contribution in self.user_contributions.values() {
            if matches!(contribution.account_status, AccountStatus::Active | AccountStatus::Warning) {
                total_storage_offered += contribution.storage_offered_gb;
                total_storage_used += contribution.storage_used_gb;
                total_bandwidth_offered += contribution.bandwidth_offered_mbps;
                total_bandwidth_used += contribution.bandwidth_used_mbps;
                total_score += contribution.contribution_score;
                active_count += 1;
            }
        }

        self.network_stats = NetworkContributionStats {
            total_storage_offered_gb: total_storage_offered,
            total_storage_used_gb: total_storage_used,
            total_bandwidth_offered_mbps: total_bandwidth_offered,
            total_bandwidth_used_mbps: total_bandwidth_used,
            active_contributors: active_count,
            network_utilization_percent: if total_storage_offered > 0 {
                (total_storage_used as f64 / total_storage_offered as f64) * 100.0
            } else {
                0.0
            },
            average_contribution_score: if active_count > 0 {
                total_score / active_count as f64
            } else {
                0.0
            },
            last_updated: Utc::now(),
        };

        Ok(())
    }

    /// Check if user can request storage allocation
    pub fn can_request_storage(&self, user_id: &str, requested_gb: u64) -> Result<bool> {
        let contribution = self.user_contributions.get(user_id)
            .ok_or_else(|| anyhow::anyhow!("User not found"))?;

        match contribution.account_status {
            AccountStatus::Suspended => Ok(false),
            AccountStatus::Limited => {
                // Limited users can only request small amounts
                Ok(requested_gb <= 1 && contribution.storage_used_gb + requested_gb <= contribution.storage_offered_gb / 2)
            },
            AccountStatus::Warning | AccountStatus::Active => {
                // Check if request would violate their offering
                let new_total = contribution.storage_used_gb + requested_gb;
                Ok(new_total <= contribution.storage_offered_gb)
            },
        }
    }

    /// Get priority queue position for resource allocation
    pub fn get_allocation_priority(&self, user_id: &str) -> Result<u32> {
        let contribution = self.user_contributions.get(user_id)
            .ok_or_else(|| anyhow::anyhow!("User not found"))?;

        let priority_score = match contribution.priority_level {
            PriorityLevel::Generous => 1000 + (contribution.contribution_score * 100.0) as u32,
            PriorityLevel::Surplus => 800 + (contribution.contribution_score * 100.0) as u32,
            PriorityLevel::Balanced => 500 + (contribution.contribution_score * 100.0) as u32,
            PriorityLevel::Deficit => (contribution.contribution_score * 100.0) as u32,
        };

        Ok(priority_score)
    }

    /// Get user's current contribution status
    pub fn get_user_status(&self, user_id: &str) -> Option<&UserContribution> {
        self.user_contributions.get(user_id)
    }

    /// Get network statistics
    pub fn get_network_stats(&self) -> &NetworkContributionStats {
        &self.network_stats
    }

    /// Get users sorted by contribution score (highest first)
    pub fn get_contribution_leaderboard(&self, limit: Option<usize>) -> Vec<&UserContribution> {
        let mut users: Vec<_> = self.user_contributions.values()
            .filter(|u| matches!(u.account_status, AccountStatus::Active | AccountStatus::Warning))
            .collect();

        users.sort_by(|a, b| b.contribution_score.partial_cmp(&a.contribution_score).unwrap());

        if let Some(limit) = limit {
            users.truncate(limit);
        }

        users
    }
}

impl Default for ContributionTracker {
    fn default() -> Self {
        Self::new()
    }
}