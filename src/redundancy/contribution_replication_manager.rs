//! Contribution-Based Replication Manager
//!
//! High-level manager that integrates contribution tracking with smart redundancy

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};

use crate::economics::{ContributionTracker, UserContribution};
use super::contribution_node_selector::{
    ContributionNodeSelector, NodeSelectionCriteria, NodeSelectionResult, NodeAvailability
};
use super::reputation_system::{NodeReputation, ReputationManager};
use super::intelligent_replication::{ContentType, ReplicationPolicy};

/// Main replication manager using contribution-based node selection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContributionReplicationManager {
    /// Node selector for finding optimal nodes
    pub node_selector: ContributionNodeSelector,
    /// Replication policies for different content types
    pub replication_policies: HashMap<ContentType, ContributionReplicationPolicy>,
    /// Current replication jobs
    pub active_replications: HashMap<String, ReplicationJob>,
    /// Performance statistics
    pub performance_stats: ReplicationPerformanceStats,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContributionReplicationPolicy {
    pub content_type: ContentType,
    pub min_replicas: u32,
    pub max_replicas: u32,
    pub target_replicas: u32,
    pub min_contribution_score: f64,
    pub min_reliability_score: f64,
    pub min_uptime_percentage: f64,
    pub prefer_high_contributors: bool,
    pub geographic_distribution: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplicationJob {
    pub job_id: String,
    pub chunk_id: String,
    pub content_type: ContentType,
    pub selected_nodes: Vec<String>,
    pub replication_status: ReplicationJobStatus,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub performance_metrics: JobPerformanceMetrics,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ReplicationJobStatus {
    Pending,
    NodeSelection,
    Replicating,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobPerformanceMetrics {
    pub node_selection_time_ms: u32,
    pub replication_time_ms: u32,
    pub bytes_replicated: u64,
    pub successful_replicas: u32,
    pub failed_replicas: u32,
    pub average_node_response_time_ms: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplicationPerformanceStats {
    pub total_jobs_completed: u64,
    pub total_jobs_failed: u64,
    pub average_job_time_ms: u32,
    pub average_nodes_per_job: f32,
    pub contribution_score_distribution: HashMap<String, u32>, // Score ranges -> count
    pub reliability_score_distribution: HashMap<String, u32>,
    pub success_rate_by_contributor_level: HashMap<String, f32>,
    pub last_updated: DateTime<Utc>,
}

impl ContributionReplicationManager {
    pub fn new() -> Self {
        let mut policies = HashMap::new();

        // Critical data - highest requirements
        policies.insert(ContentType::Critical, ContributionReplicationPolicy {
            content_type: ContentType::Critical,
            min_replicas: 5,
            max_replicas: 9,
            target_replicas: 7,
            min_contribution_score: 1.5, // Require surplus contributors
            min_reliability_score: 0.9,
            min_uptime_percentage: 99.0,
            prefer_high_contributors: true,
            geographic_distribution: true,
        });

        // Important data - high requirements
        policies.insert(ContentType::Important, ContributionReplicationPolicy {
            content_type: ContentType::Important,
            min_replicas: 3,
            max_replicas: 7,
            target_replicas: 5,
            min_contribution_score: 1.0, // Require balanced contributors
            min_reliability_score: 0.8,
            min_uptime_percentage: 95.0,
            prefer_high_contributors: true,
            geographic_distribution: true,
        });

        // Standard data - moderate requirements
        policies.insert(ContentType::Standard, ContributionReplicationPolicy {
            content_type: ContentType::Standard,
            min_replicas: 2,
            max_replicas: 5,
            target_replicas: 3,
            min_contribution_score: 0.8, // Accept lower contributors
            min_reliability_score: 0.7,
            min_uptime_percentage: 90.0,
            prefer_high_contributors: false,
            geographic_distribution: false,
        });

        // Archive data - basic requirements
        policies.insert(ContentType::Archive, ContributionReplicationPolicy {
            content_type: ContentType::Archive,
            min_replicas: 2,
            max_replicas: 4,
            target_replicas: 3,
            min_contribution_score: 0.5, // Accept deficit contributors
            min_reliability_score: 0.6,
            min_uptime_percentage: 85.0,
            prefer_high_contributors: false,
            geographic_distribution: false,
        });

        Self {
            node_selector: ContributionNodeSelector::new(),
            replication_policies: policies,
            active_replications: HashMap::new(),
            performance_stats: ReplicationPerformanceStats {
                total_jobs_completed: 0,
                total_jobs_failed: 0,
                average_job_time_ms: 0,
                average_nodes_per_job: 0.0,
                contribution_score_distribution: HashMap::new(),
                reliability_score_distribution: HashMap::new(),
                success_rate_by_contributor_level: HashMap::new(),
                last_updated: Utc::now(),
            },
        }
    }

    /// Update node data from contribution tracker and reputation system
    pub async fn update_node_data(
        &mut self,
        contribution_tracker: &ContributionTracker,
        reputation_manager: &ReputationManager,
    ) -> Result<()> {

        // Update contribution data for all nodes
        for (node_id, user_contribution) in contribution_tracker.user_contributions.iter() {
            self.node_selector.update_node_contribution(node_id.clone(), user_contribution).await?;
        }

        // Update reliability data for all nodes (simplified - would iterate through actual reputation data)
        // For now, create mock reliability data based on contribution
        for node_id in contribution_tracker.user_contributions.keys() {
            // In a real system, this would get actual reputation data
            let mock_reliability = self.create_mock_reliability(node_id);
            self.node_selector.update_node_reliability(node_id.clone(), &mock_reliability).await?;

            // Set node availability (simplified)
            let user_contrib = contribution_tracker.user_contributions.get(node_id).unwrap();
            let availability = NodeAvailability::Available {
                available_storage_gb: user_contrib.storage_offered_gb - user_contrib.storage_used_gb,
                available_bandwidth_mbps: user_contrib.bandwidth_offered_mbps - user_contrib.bandwidth_used_mbps,
                current_load_percent: (user_contrib.storage_used_gb as f64 / user_contrib.storage_offered_gb.max(1) as f64) * 100.0,
            };
            self.node_selector.update_node_availability(node_id.clone(), availability);
        }

        Ok(())
    }

    /// Create mock reliability data (in real system, this would come from reputation manager)
    fn create_mock_reliability(&self, node_id: &str) -> NodeReputation {
        use super::reputation_system::{ReliabilityMetrics, PerformanceMetrics};
        use tokio::time::{Duration, Instant};

        NodeReputation {
            node_id: node_id.to_string(),
            overall_score: 0.85, // Mock score
            reliability_metrics: ReliabilityMetrics {
                uptime_score: 0.95,
                data_integrity_score: 0.99,
                response_consistency: 0.9,
                failure_recovery_time: Duration::from_secs(300),
                consecutive_failures: 0,
                mean_time_between_failures: Duration::from_secs(86400 * 30), // 30 days
            },
            performance_metrics: PerformanceMetrics {
                average_latency: Duration::from_millis(50),
                throughput_score: 0.8,
                storage_efficiency: 0.85,
                bandwidth_utilization: 0.75,
                resource_stability: 0.9,
                load_handling_capacity: 0.8,
            },
            historical_events: vec![],
            reputation_trend: super::reputation_system::ReputationTrend::Stable,
            last_updated: Instant::now(),
        }
    }

    /// Replicate a chunk using contribution-based node selection
    pub async fn replicate_chunk(
        &mut self,
        chunk_id: String,
        content_type: ContentType,
        chunk_size_bytes: u64,
    ) -> Result<ReplicationJob> {

        let job_id = format!("job_{}", uuid::Uuid::new_v4());
        let start_time = Utc::now();

        // Get replication policy for this content type
        let policy = self.replication_policies.get(&content_type)
            .ok_or_else(|| anyhow::anyhow!("No policy found for content type: {:?}", content_type))?;

        // Create selection criteria based on policy
        let criteria = NodeSelectionCriteria {
            min_contribution_score: policy.min_contribution_score,
            min_reliability_score: policy.min_reliability_score,
            min_uptime_percentage: policy.min_uptime_percentage,
            max_failure_rate: 1.0 - policy.min_reliability_score,
            min_available_storage_gb: (chunk_size_bytes / 1_000_000_000) + 1, // Convert to GB with buffer
            min_available_bandwidth_mbps: 10.0, // Minimum 10 Mbps
            geographic_requirements: if policy.geographic_distribution {
                Some(super::contribution_node_selector::GeographicRequirements {
                    preferred_regions: vec![],
                    excluded_regions: vec![],
                    min_regions: Some(2),
                    max_distance_km: None,
                })
            } else {
                None
            },
            exclude_recent_failures: true,
        };

        // Select nodes using contribution-based selector
        let selection_result = self.node_selector.select_nodes(
            chunk_id.clone(),
            policy.target_replicas,
            criteria,
        ).await?;

        let node_selection_time_ms = (Utc::now() - start_time).num_milliseconds() as u32;

        // Create replication job
        let job = ReplicationJob {
            job_id: job_id.clone(),
            chunk_id: chunk_id.clone(),
            content_type,
            selected_nodes: selection_result.selected_nodes.iter().map(|n| n.node_id.clone()).collect(),
            replication_status: ReplicationJobStatus::NodeSelection,
            started_at: start_time,
            completed_at: None,
            performance_metrics: JobPerformanceMetrics {
                node_selection_time_ms,
                replication_time_ms: 0,
                bytes_replicated: 0,
                successful_replicas: 0,
                failed_replicas: 0,
                average_node_response_time_ms: 0,
            },
        };

        // Store active job
        self.active_replications.insert(job_id.clone(), job.clone());

        // Update performance statistics
        self.update_performance_stats(&selection_result);

        Ok(job)
    }

    /// Update performance statistics based on node selection results
    fn update_performance_stats(&mut self, selection_result: &NodeSelectionResult) {
        // Update contribution score distribution
        for node in &selection_result.selected_nodes {
            let score_range = self.get_score_range(node.contribution_score);
            *self.performance_stats.contribution_score_distribution.entry(score_range).or_insert(0) += 1;

            let reliability_range = self.get_score_range(node.reliability_score);
            *self.performance_stats.reliability_score_distribution.entry(reliability_range).or_insert(0) += 1;
        }

        self.performance_stats.last_updated = Utc::now();
    }

    /// Get score range for statistics
    fn get_score_range(&self, score: f64) -> String {
        if score >= 2.0 {
            "2.0+".to_string()
        } else if score >= 1.5 {
            "1.5-2.0".to_string()
        } else if score >= 1.0 {
            "1.0-1.5".to_string()
        } else if score >= 0.5 {
            "0.5-1.0".to_string()
        } else {
            "0.0-0.5".to_string()
        }
    }

    /// Complete a replication job
    pub async fn complete_replication_job(
        &mut self,
        job_id: String,
        successful_replicas: u32,
        failed_replicas: u32,
        bytes_replicated: u64,
    ) -> Result<()> {

        if let Some(job) = self.active_replications.get_mut(&job_id) {
            job.replication_status = if failed_replicas == 0 {
                ReplicationJobStatus::Completed
            } else if successful_replicas > 0 {
                ReplicationJobStatus::Completed // Partial success still counts as completed
            } else {
                ReplicationJobStatus::Failed
            };

            job.completed_at = Some(Utc::now());
            job.performance_metrics.successful_replicas = successful_replicas;
            job.performance_metrics.failed_replicas = failed_replicas;
            job.performance_metrics.bytes_replicated = bytes_replicated;

            if let Some(completed_at) = job.completed_at {
                job.performance_metrics.replication_time_ms = (completed_at - job.started_at).num_milliseconds() as u32;
            }

            // Update global stats
            if job.replication_status == ReplicationJobStatus::Completed {
                self.performance_stats.total_jobs_completed += 1;
            } else {
                self.performance_stats.total_jobs_failed += 1;
            }

            // Update average job time
            let total_jobs = self.performance_stats.total_jobs_completed + self.performance_stats.total_jobs_failed;
            if total_jobs > 0 {
                let total_time = (self.performance_stats.average_job_time_ms as u64 * (total_jobs - 1)) + job.performance_metrics.replication_time_ms as u64;
                self.performance_stats.average_job_time_ms = (total_time / total_jobs) as u32;
            }

            // Update average nodes per job
            let total_completed = self.performance_stats.total_jobs_completed as f32;
            if total_completed > 0.0 {
                let current_avg = self.performance_stats.average_nodes_per_job;
                let new_count = job.selected_nodes.len() as f32;
                self.performance_stats.average_nodes_per_job = (current_avg * (total_completed - 1.0) + new_count) / total_completed;
            }
        } else {
            return Err(anyhow::anyhow!("Job not found: {}", job_id));
        }

        Ok(())
    }

    /// Get replication statistics
    pub fn get_performance_stats(&self) -> &ReplicationPerformanceStats {
        &self.performance_stats
    }

    /// Get active replication jobs
    pub fn get_active_jobs(&self) -> Vec<&ReplicationJob> {
        self.active_replications.values().collect()
    }

    /// Get node selection statistics
    pub fn get_node_selection_stats(&self, node_id: &str) -> super::contribution_node_selector::NodeSelectionStats {
        self.node_selector.get_node_selection_stats(node_id)
    }
}

impl Default for ContributionReplicationManager {
    fn default() -> Self {
        Self::new()
    }
}