//! Contribution-Based Node Selection
//!
//! Selects optimal nodes for replication based on contribution ratios and reliability

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};

use crate::economics::{UserContribution, PriorityLevel, ContributionTracker};
use super::reputation_system::{NodeReputation, ReliabilityMetrics, PerformanceMetrics};

/// Node selector that prioritizes based on contribution and reliability
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContributionNodeSelector {
    /// Node contribution data
    pub node_contributions: HashMap<String, NodeContribution>,
    /// Node reliability scores
    pub node_reliability: HashMap<String, NodeReliability>,
    /// Selection criteria weights
    pub selection_weights: SelectionWeights,
    /// Node availability status
    pub node_availability: HashMap<String, NodeAvailability>,
    /// Recent selection history for fairness
    pub selection_history: Vec<SelectionRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeContribution {
    pub node_id: String,
    pub user_id: String,
    pub contribution_score: f64,
    pub priority_level: PriorityLevel,
    /// Storage offered to the network
    pub storage_offered_gb: u64,
    /// Current storage utilization
    pub storage_used_gb: u64,
    /// Bandwidth offered to the network
    pub bandwidth_offered_mbps: f64,
    /// Current bandwidth utilization
    pub bandwidth_used_mbps: f64,
    /// How long node has been active
    pub tenure_days: u32,
    /// Recent contribution trend
    pub contribution_trend: ContributionTrend,
    pub last_updated: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ContributionTrend {
    Improving,
    Stable,
    Declining,
    New,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeReliability {
    pub node_id: String,
    pub overall_reliability_score: f64, // 0.0 to 1.0
    pub uptime_percentage: f64,
    pub response_time_ms: u32,
    pub data_integrity_score: f64,
    pub failure_rate: f64,
    pub recovery_time_minutes: u32,
    pub consistency_score: f64,
    pub performance_stability: f64,
    pub last_failure: Option<DateTime<Utc>>,
    pub consecutive_successful_operations: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectionWeights {
    /// Weight for contribution score (0.0-1.0)
    pub contribution_weight: f64,
    /// Weight for reliability score (0.0-1.0)
    pub reliability_weight: f64,
    /// Weight for performance metrics (0.0-1.0)
    pub performance_weight: f64,
    /// Weight for geographic distribution (0.0-1.0)
    pub geographic_weight: f64,
    /// Weight for fairness/load balancing (0.0-1.0)
    pub fairness_weight: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NodeAvailability {
    Available {
        available_storage_gb: u64,
        available_bandwidth_mbps: f64,
        current_load_percent: f64,
    },
    Busy {
        estimated_available_in_minutes: u32,
    },
    Maintenance {
        expected_return: DateTime<Utc>,
    },
    Offline,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectionRecord {
    pub selection_time: DateTime<Utc>,
    pub chunk_id: String,
    pub selected_nodes: Vec<String>,
    pub selection_reason: String,
    pub total_candidates: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeSelectionCriteria {
    /// Minimum contribution score required
    pub min_contribution_score: f64,
    /// Minimum reliability score required
    pub min_reliability_score: f64,
    /// Minimum uptime percentage required
    pub min_uptime_percentage: f64,
    /// Maximum acceptable failure rate
    pub max_failure_rate: f64,
    /// Minimum available storage required
    pub min_available_storage_gb: u64,
    /// Minimum available bandwidth required
    pub min_available_bandwidth_mbps: f64,
    /// Geographic constraints
    pub geographic_requirements: Option<GeographicRequirements>,
    /// Exclude nodes with recent failures
    pub exclude_recent_failures: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeographicRequirements {
    pub preferred_regions: Vec<String>,
    pub excluded_regions: Vec<String>,
    pub min_regions: Option<u32>,
    pub max_distance_km: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeSelectionResult {
    pub selected_nodes: Vec<SelectedNode>,
    pub total_candidates: u32,
    pub selection_method: SelectionMethod,
    pub selection_quality: SelectionQuality,
    pub fallback_used: bool,
    pub selection_rationale: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectedNode {
    pub node_id: String,
    pub selection_score: f64,
    pub contribution_score: f64,
    pub reliability_score: f64,
    pub selection_reason: String,
    pub expected_performance: ExpectedPerformance,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpectedPerformance {
    pub uptime_percentage: f64,
    pub response_time_ms: u32,
    pub throughput_mbps: f64,
    pub reliability_confidence: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SelectionMethod {
    ContributionPrimary,  // Prioritize highest contributors
    ReliabilityPrimary,   // Prioritize most reliable nodes
    Balanced,             // Balance contribution and reliability
    Geographic,           // Optimize for geographic distribution
    LoadBalanced,         // Ensure fair distribution of work
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SelectionQuality {
    Excellent,  // All criteria met with high-quality nodes
    Good,       // Most criteria met with good nodes
    Acceptable, // Minimum criteria met
    Compromised, // Had to lower standards to find nodes
}

impl ContributionNodeSelector {
    pub fn new() -> Self {
        Self {
            node_contributions: HashMap::new(),
            node_reliability: HashMap::new(),
            selection_weights: SelectionWeights {
                contribution_weight: 0.4,  // 40% based on contribution
                reliability_weight: 0.3,   // 30% based on reliability
                performance_weight: 0.2,   // 20% based on performance
                geographic_weight: 0.05,   // 5% for geographic distribution
                fairness_weight: 0.05,     // 5% for load balancing fairness
            },
            node_availability: HashMap::new(),
            selection_history: Vec::new(),
        }
    }

    /// Update node contribution data from contribution tracker
    pub async fn update_node_contribution(&mut self, node_id: String, user_contribution: &UserContribution) -> Result<()> {
        let tenure_days = (Utc::now() - user_contribution.joined_at).num_days() as u32;

        let contribution_trend = if tenure_days < 7 {
            ContributionTrend::New
        } else {
            // In a real system, this would analyze historical data
            if user_contribution.contribution_score > 1.2 {
                ContributionTrend::Improving
            } else if user_contribution.contribution_score > 0.8 {
                ContributionTrend::Stable
            } else {
                ContributionTrend::Declining
            }
        };

        let node_contribution = NodeContribution {
            node_id: node_id.clone(),
            user_id: user_contribution.user_id.clone(),
            contribution_score: user_contribution.contribution_score,
            priority_level: user_contribution.priority_level.clone(),
            storage_offered_gb: user_contribution.storage_offered_gb,
            storage_used_gb: user_contribution.storage_used_gb,
            bandwidth_offered_mbps: user_contribution.bandwidth_offered_mbps,
            bandwidth_used_mbps: user_contribution.bandwidth_used_mbps,
            tenure_days,
            contribution_trend,
            last_updated: Utc::now(),
        };

        self.node_contributions.insert(node_id, node_contribution);
        Ok(())
    }

    /// Update node reliability data from reputation system
    pub async fn update_node_reliability(&mut self, node_id: String, reputation: &NodeReputation) -> Result<()> {
        let reliability = NodeReliability {
            node_id: node_id.clone(),
            overall_reliability_score: reputation.overall_score as f64,
            uptime_percentage: (reputation.reliability_metrics.uptime_score as f64) * 100.0,
            response_time_ms: reputation.performance_metrics.average_latency.as_millis() as u32,
            data_integrity_score: reputation.reliability_metrics.data_integrity_score as f64,
            failure_rate: 1.0 - (reputation.reliability_metrics.uptime_score as f64),
            recovery_time_minutes: (reputation.reliability_metrics.failure_recovery_time.as_secs() / 60) as u32,
            consistency_score: reputation.reliability_metrics.response_consistency as f64,
            performance_stability: reputation.performance_metrics.resource_stability as f64,
            last_failure: None, // Would be extracted from reputation events in real system
            consecutive_successful_operations: 100 - reputation.reliability_metrics.consecutive_failures as u64,
        };

        self.node_reliability.insert(node_id, reliability);
        Ok(())
    }

    /// Select optimal nodes for chunk replication
    pub async fn select_nodes(
        &mut self,
        chunk_id: String,
        num_nodes_needed: u32,
        criteria: NodeSelectionCriteria,
    ) -> Result<NodeSelectionResult> {

        // Get all candidate nodes that meet minimum criteria
        let candidates = self.get_candidate_nodes(&criteria)?;

        if candidates.is_empty() {
            return Ok(NodeSelectionResult {
                selected_nodes: Vec::new(),
                total_candidates: 0,
                selection_method: SelectionMethod::Balanced,
                selection_quality: SelectionQuality::Compromised,
                fallback_used: true,
                selection_rationale: "No nodes meet minimum criteria".to_string(),
            });
        }

        // Calculate selection scores for each candidate
        let scored_candidates = self.calculate_selection_scores(&candidates, &criteria)?;

        // Select top nodes based on scores
        let mut selected_nodes = scored_candidates;
        selected_nodes.sort_by(|a, b| b.selection_score.partial_cmp(&a.selection_score).unwrap());
        selected_nodes.truncate(num_nodes_needed as usize);

        // Determine selection quality
        let selection_quality = self.assess_selection_quality(&selected_nodes, &criteria);

        // Record selection for fairness tracking
        let selection_record = SelectionRecord {
            selection_time: Utc::now(),
            chunk_id: chunk_id.clone(),
            selected_nodes: selected_nodes.iter().map(|n| n.node_id.clone()).collect(),
            selection_reason: "Contribution and reliability based selection".to_string(),
            total_candidates: candidates.len() as u32,
        };
        self.selection_history.push(selection_record);

        // Keep only recent history
        if self.selection_history.len() > 1000 {
            self.selection_history.drain(0..500);
        }

        Ok(NodeSelectionResult {
            selected_nodes,
            total_candidates: candidates.len() as u32,
            selection_method: SelectionMethod::Balanced,
            selection_quality,
            fallback_used: false,
            selection_rationale: "Selected based on contribution score and reliability metrics".to_string(),
        })
    }

    /// Get nodes that meet minimum criteria
    fn get_candidate_nodes(&self, criteria: &NodeSelectionCriteria) -> Result<Vec<String>> {
        let mut candidates = Vec::new();

        for (node_id, contribution) in &self.node_contributions {
            // Check contribution requirements
            if contribution.contribution_score < criteria.min_contribution_score {
                continue;
            }

            // Check reliability requirements
            if let Some(reliability) = self.node_reliability.get(node_id) {
                if reliability.overall_reliability_score < criteria.min_reliability_score {
                    continue;
                }
                if reliability.uptime_percentage < criteria.min_uptime_percentage {
                    continue;
                }
                if reliability.failure_rate > criteria.max_failure_rate {
                    continue;
                }

                // Check for recent failures if required
                if criteria.exclude_recent_failures {
                    if let Some(last_failure) = reliability.last_failure {
                        if (Utc::now() - last_failure).num_hours() < 24 {
                            continue;
                        }
                    }
                }
            } else {
                // Skip nodes without reliability data
                continue;
            }

            // Check availability requirements
            if let Some(availability) = self.node_availability.get(node_id) {
                match availability {
                    NodeAvailability::Available { available_storage_gb, available_bandwidth_mbps, current_load_percent } => {
                        if *available_storage_gb < criteria.min_available_storage_gb {
                            continue;
                        }
                        if *available_bandwidth_mbps < criteria.min_available_bandwidth_mbps {
                            continue;
                        }
                        if *current_load_percent > 90.0 {
                            continue; // Skip overloaded nodes
                        }
                    },
                    _ => continue, // Skip non-available nodes
                }
            }

            candidates.push(node_id.clone());
        }

        Ok(candidates)
    }

    /// Calculate selection scores for candidate nodes
    fn calculate_selection_scores(&self, candidates: &[String], criteria: &NodeSelectionCriteria) -> Result<Vec<SelectedNode>> {
        let mut scored_nodes = Vec::new();

        for node_id in candidates {
            let contribution = self.node_contributions.get(node_id).unwrap();
            let reliability = self.node_reliability.get(node_id).unwrap();

            // Normalize scores (0.0 to 1.0)
            let contribution_score = (contribution.contribution_score / 3.0).min(1.0); // Cap at 3.0 for normalization
            let reliability_score = reliability.overall_reliability_score;
            let performance_score = self.calculate_performance_score(reliability);
            let fairness_score = self.calculate_fairness_score(node_id);
            let geographic_score = 1.0; // Simplified for now

            // Calculate weighted total score
            let total_score =
                (contribution_score * self.selection_weights.contribution_weight) +
                (reliability_score * self.selection_weights.reliability_weight) +
                (performance_score * self.selection_weights.performance_weight) +
                (geographic_score * self.selection_weights.geographic_weight) +
                (fairness_score * self.selection_weights.fairness_weight);

            let selected_node = SelectedNode {
                node_id: node_id.clone(),
                selection_score: total_score,
                contribution_score: contribution.contribution_score,
                reliability_score,
                selection_reason: format!("Score: {:.3} (Contrib: {:.2}, Reliab: {:.2})",
                    total_score, contribution_score, reliability_score),
                expected_performance: ExpectedPerformance {
                    uptime_percentage: reliability.uptime_percentage,
                    response_time_ms: reliability.response_time_ms,
                    throughput_mbps: contribution.bandwidth_offered_mbps * 0.8, // Estimate 80% utilization
                    reliability_confidence: reliability.consistency_score,
                },
            };

            scored_nodes.push(selected_node);
        }

        Ok(scored_nodes)
    }

    /// Calculate performance score based on reliability metrics
    fn calculate_performance_score(&self, reliability: &NodeReliability) -> f64 {
        let response_score = (1000.0 - reliability.response_time_ms as f64).max(0.0) / 1000.0;
        let stability_score = reliability.performance_stability;
        let consistency_score = reliability.consistency_score;

        (response_score + stability_score + consistency_score) / 3.0
    }

    /// Calculate fairness score to promote load balancing
    fn calculate_fairness_score(&self, node_id: &str) -> f64 {
        // Count recent selections for this node
        let recent_selections = self.selection_history.iter()
            .rev()
            .take(100) // Look at last 100 selections
            .filter(|record| record.selected_nodes.contains(&node_id.to_string()))
            .count();

        // Nodes with fewer recent selections get higher fairness scores
        (10.0 - recent_selections as f64).max(0.0) / 10.0
    }

    /// Assess the quality of the selection
    fn assess_selection_quality(&self, selected_nodes: &[SelectedNode], criteria: &NodeSelectionCriteria) -> SelectionQuality {
        if selected_nodes.is_empty() {
            return SelectionQuality::Compromised;
        }

        let avg_contribution = selected_nodes.iter()
            .map(|n| n.contribution_score)
            .sum::<f64>() / selected_nodes.len() as f64;

        let avg_reliability = selected_nodes.iter()
            .map(|n| n.reliability_score)
            .sum::<f64>() / selected_nodes.len() as f64;

        if avg_contribution >= 1.5 && avg_reliability >= 0.9 {
            SelectionQuality::Excellent
        } else if avg_contribution >= 1.0 && avg_reliability >= 0.8 {
            SelectionQuality::Good
        } else if avg_contribution >= 0.8 && avg_reliability >= 0.7 {
            SelectionQuality::Acceptable
        } else {
            SelectionQuality::Compromised
        }
    }

    /// Update node availability status
    pub fn update_node_availability(&mut self, node_id: String, availability: NodeAvailability) {
        self.node_availability.insert(node_id, availability);
    }

    /// Get selection statistics for a node
    pub fn get_node_selection_stats(&self, node_id: &str) -> NodeSelectionStats {
        let total_selections = self.selection_history.iter()
            .filter(|record| record.selected_nodes.contains(&node_id.to_string()))
            .count();

        let recent_selections = self.selection_history.iter()
            .rev()
            .take(100)
            .filter(|record| record.selected_nodes.contains(&node_id.to_string()))
            .count();

        NodeSelectionStats {
            node_id: node_id.to_string(),
            total_selections: total_selections as u32,
            recent_selections: recent_selections as u32,
            last_selected: self.selection_history.iter()
                .rev()
                .find(|record| record.selected_nodes.contains(&node_id.to_string()))
                .map(|record| record.selection_time),
            selection_rate: if self.selection_history.len() > 0 {
                (total_selections as f64 / self.selection_history.len() as f64) * 100.0
            } else {
                0.0
            },
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeSelectionStats {
    pub node_id: String,
    pub total_selections: u32,
    pub recent_selections: u32,
    pub last_selected: Option<DateTime<Utc>>,
    pub selection_rate: f64, // Percentage of total selections
}

impl Default for ContributionNodeSelector {
    fn default() -> Self {
        Self::new()
    }
}