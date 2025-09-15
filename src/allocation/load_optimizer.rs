//! Contribution-Based Load Balancer
//!
//! Load balancing based on node contribution and performance rather than cost

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};

/// Load balancer using contribution metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContributionLoadBalancer {
    pub load_balancing_decisions: Vec<LoadBalancingDecision>,
    pub resource_weights: HashMap<String, ResourceWeight>,
    pub routing_cache: HashMap<String, OptimizedRouting>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadBalancingDecision {
    pub decision_id: String,
    pub selected_nodes: Vec<String>,
    pub load_distribution: HashMap<String, f64>,
    pub decision_time: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceWeight {
    pub node_id: String,
    pub contribution_weight: f64,
    pub performance_weight: f64,
    pub reliability_weight: f64,
    pub total_weight: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizedRouting {
    pub route_id: String,
    pub primary_nodes: Vec<String>,
    pub backup_nodes: Vec<String>,
    pub expected_performance: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceAllocation {
    pub allocation_id: String,
    pub performance_target: f64,
    pub actual_performance: f64,
    pub nodes_used: Vec<String>,
}

impl ContributionLoadBalancer {
    pub fn new() -> Self {
        Self {
            load_balancing_decisions: Vec::new(),
            resource_weights: HashMap::new(),
            routing_cache: HashMap::new(),
        }
    }
}

impl Default for ContributionLoadBalancer {
    fn default() -> Self {
        Self::new()
    }
}