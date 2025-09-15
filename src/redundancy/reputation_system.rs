//! Node Reliability Reputation System
//!
//! Tracks and scores node reliability based on historical performance

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeReputation {
    pub node_id: String,
    pub overall_score: f32, // 0.0 to 1.0
    pub reliability_metrics: ReliabilityMetrics,
    pub performance_metrics: PerformanceMetrics,
    pub historical_events: Vec<ReputationEvent>,
    pub reputation_trend: ReputationTrend,
    pub last_updated: crate::SerializableInstant,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReliabilityMetrics {
    pub uptime_score: f32,
    pub data_integrity_score: f32,
    pub response_consistency: f32,
    pub failure_recovery_time: Duration,
    pub consecutive_failures: u32,
    pub mean_time_between_failures: Duration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub average_latency: Duration,
    pub throughput_score: f32,
    pub storage_efficiency: f32,
    pub bandwidth_utilization: f32,
    pub resource_stability: f32,
    pub load_handling_capacity: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReputationEvent {
    pub timestamp: crate::SerializableInstant,
    pub event_type: EventType,
    pub impact: f32, // -1.0 to +1.0
    pub details: String,
    pub severity: EventSeverity,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventType {
    NodeFailure,
    DataCorruption,
    SlowResponse,
    ExceptionalPerformance,
    SuccessfulRecovery,
    MaintenanceCompleted,
    SecurityIncident,
    NetworkContribution,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventSeverity {
    Critical,    // -0.2 to -1.0
    Major,       // -0.1 to -0.2
    Minor,       // -0.05 to -0.1
    Neutral,     // -0.05 to +0.05
    Positive,    // +0.05 to +0.1
    Exceptional, // +0.1 to +0.2
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReputationTrend {
    pub direction: TrendDirection,
    pub slope: f32,
    pub confidence: f32,
    pub time_window: Duration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TrendDirection {
    StronglyImproving,
    Improving,
    Stable,
    Declining,
    StronglyDeclining,
}

pub struct ReputationManager {
    node_reputations: HashMap<String, NodeReputation>,
    reputation_weights: ReputationWeights,
    historical_data: HashMap<String, Vec<PerformanceSnapshot>>,
    global_network_stats: NetworkStats,
}

#[derive(Debug, Clone)]
struct ReputationWeights {
    uptime: f32,
    data_integrity: f32,
    performance: f32,
    recovery_ability: f32,
    consistency: f32,
    network_contribution: f32,
}

#[derive(Debug, Clone)]
struct PerformanceSnapshot {
    timestamp: crate::SerializableInstant,
    metrics: PerformanceMetrics,
    events: Vec<ReputationEvent>,
}

#[derive(Debug, Clone)]
struct NetworkStats {
    average_uptime: f32,
    median_latency: Duration,
    total_nodes: usize,
    healthy_nodes: usize,
    network_quality_score: f32,
}

impl ReputationManager {
    pub fn new() -> Self {
        Self {
            node_reputations: HashMap::new(),
            reputation_weights: ReputationWeights::default(),
            historical_data: HashMap::new(),
            global_network_stats: NetworkStats::default(),
        }
    }

    pub async fn update_node_performance(&mut self, node_id: &str, metrics: PerformanceMetrics) {
        let snapshot = PerformanceSnapshot {
            timestamp: crate::SerializableInstant::now(),
            metrics,
            events: Vec::new(),
        };

        let history = self.historical_data.entry(node_id.to_string()).or_insert_with(Vec::new);
        history.push(snapshot);

        // Keep only last 30 days of data
        let cutoff = crate::SerializableInstant::now() - Duration::from_secs(30 * 24 * 3600);
        history.retain(|s| s.timestamp > cutoff);

        // Update reputation based on new performance data
        self.recalculate_reputation(node_id).await;
    }

    pub async fn record_event(&mut self, node_id: &str, event: ReputationEvent) {
        // Add event to historical data
        if let Some(history) = self.historical_data.get_mut(node_id) {
            if let Some(latest) = history.last_mut() {
                latest.events.push(event.clone());
            }
        }

        // Update reputation immediately for significant events
        if matches!(event.severity, EventSeverity::Critical | EventSeverity::Exceptional) {
            self.apply_immediate_reputation_change(node_id, &event).await;
        }

        self.recalculate_reputation(node_id).await;
    }

    pub fn get_node_reputation(&self, node_id: &str) -> Option<&NodeReputation> {
        self.node_reputations.get(node_id)
    }

    pub fn get_top_nodes(&self, limit: usize) -> Vec<&NodeReputation> {
        let mut nodes: Vec<_> = self.node_reputations.values().collect();
        nodes.sort_by(|a, b| b.overall_score.partial_cmp(&a.overall_score).unwrap());
        nodes.into_iter().take(limit).collect()
    }

    pub fn get_nodes_by_reputation_range(&self, min_score: f32, max_score: f32) -> Vec<&NodeReputation> {
        self.node_reputations.values()
            .filter(|rep| rep.overall_score >= min_score && rep.overall_score <= max_score)
            .collect()
    }

    pub async fn get_recommended_nodes_for_storage(&self, count: usize) -> Vec<String> {
        let mut candidates: Vec<_> = self.node_reputations.iter()
            .filter(|(_, rep)| {
                rep.overall_score > 0.7
                && rep.reliability_metrics.uptime_score > 0.8
                && rep.reliability_metrics.consecutive_failures < 3
            })
            .collect();

        // Sort by composite score (reputation + recent performance)
        candidates.sort_by(|a, b| {
            let score_a = self.calculate_storage_suitability_score(a.1);
            let score_b = self.calculate_storage_suitability_score(b.1);
            score_b.partial_cmp(&score_a).unwrap()
        });

        candidates.into_iter()
            .take(count)
            .map(|(node_id, _)| node_id.clone())
            .collect()
    }

    async fn recalculate_reputation(&mut self, node_id: &str) {
        let history = match self.historical_data.get(node_id) {
            Some(h) => h,
            None => return,
        };

        if history.is_empty() {
            return;
        }

        let reliability_metrics = self.calculate_reliability_metrics(history);
        let performance_metrics = self.calculate_average_performance(history);
        let overall_score = self.calculate_overall_score(&reliability_metrics, &performance_metrics, history);
        let trend = self.calculate_reputation_trend(history);

        let reputation = NodeReputation {
            node_id: node_id.to_string(),
            overall_score,
            reliability_metrics,
            performance_metrics,
            historical_events: self.get_recent_events(history, Duration::from_secs(7 * 24 * 3600)),
            reputation_trend: trend,
            last_updated: crate::SerializableInstant::now(),
        };

        self.node_reputations.insert(node_id.to_string(), reputation);
    }

    fn calculate_reliability_metrics(&self, history: &[PerformanceSnapshot]) -> ReliabilityMetrics {
        let total_snapshots = history.len() as f32;
        if total_snapshots == 0.0 {
            return ReliabilityMetrics::default();
        }

        // Calculate uptime score based on performance consistency
        let uptime_score = history.iter()
            .map(|s| if s.metrics.average_latency < Duration::from_millis(1000) { 1.0 } else { 0.0 })
            .sum::<f32>() / total_snapshots;

        // Data integrity score based on lack of corruption events
        let corruption_events = history.iter()
            .flat_map(|s| &s.events)
            .filter(|e| matches!(e.event_type, EventType::DataCorruption))
            .count();
        let data_integrity_score = 1.0 - (corruption_events as f32 * 0.1).min(1.0);

        // Response consistency
        let latencies: Vec<_> = history.iter()
            .map(|s| s.metrics.average_latency.as_millis() as f32)
            .collect();
        let latency_variance = self.calculate_variance(&latencies);
        let response_consistency = 1.0 / (1.0 + latency_variance / 1000.0);

        // Failure analysis
        let failure_events: Vec<_> = history.iter()
            .flat_map(|s| &s.events)
            .filter(|e| matches!(e.event_type, EventType::NodeFailure))
            .collect();

        let consecutive_failures = self.count_consecutive_failures(&failure_events);
        let mean_time_between_failures = if failure_events.len() > 1 {
            let first = failure_events.first().unwrap().timestamp;
            let last = failure_events.last().unwrap().timestamp;
            (last - first) / failure_events.len() as u32
        } else {
            Duration::from_secs(u64::MAX)
        };

        let failure_recovery_time = failure_events.iter()
            .filter_map(|_| Some(Duration::from_secs(300))) // Average 5 minutes
            .next()
            .unwrap_or(Duration::from_secs(0));

        ReliabilityMetrics {
            uptime_score,
            data_integrity_score,
            response_consistency,
            failure_recovery_time,
            consecutive_failures,
            mean_time_between_failures,
        }
    }

    fn calculate_average_performance(&self, history: &[PerformanceSnapshot]) -> PerformanceMetrics {
        if history.is_empty() {
            return PerformanceMetrics::default();
        }

        let count = history.len() as f32;

        let average_latency = Duration::from_millis(
            (history.iter().map(|s| s.metrics.average_latency.as_millis()).sum::<u128>() / count as u128) as u64
        );

        let throughput_score = history.iter().map(|s| s.metrics.throughput_score).sum::<f32>() / count;
        let storage_efficiency = history.iter().map(|s| s.metrics.storage_efficiency).sum::<f32>() / count;
        let bandwidth_utilization = history.iter().map(|s| s.metrics.bandwidth_utilization).sum::<f32>() / count;
        let resource_stability = history.iter().map(|s| s.metrics.resource_stability).sum::<f32>() / count;
        let load_handling_capacity = history.iter().map(|s| s.metrics.load_handling_capacity).sum::<f32>() / count;

        PerformanceMetrics {
            average_latency,
            throughput_score,
            storage_efficiency,
            bandwidth_utilization,
            resource_stability,
            load_handling_capacity,
        }
    }

    fn calculate_overall_score(
        &self,
        reliability: &ReliabilityMetrics,
        performance: &PerformanceMetrics,
        history: &[PerformanceSnapshot],
    ) -> f32 {
        let weights = &self.reputation_weights;

        let reliability_score = (
            reliability.uptime_score * weights.uptime +
            reliability.data_integrity_score * weights.data_integrity +
            reliability.response_consistency * weights.consistency
        ) / (weights.uptime + weights.data_integrity + weights.consistency);

        let performance_score = (
            performance.throughput_score * 0.3 +
            performance.storage_efficiency * 0.2 +
            performance.resource_stability * 0.3 +
            performance.load_handling_capacity * 0.2
        );

        // Factor in recent events
        let recent_events_impact = self.calculate_recent_events_impact(history);

        let base_score = reliability_score * 0.6 + performance_score * 0.4;
        let final_score = (base_score + recent_events_impact).max(0.0).min(1.0);

        final_score
    }

    fn calculate_recent_events_impact(&self, history: &[PerformanceSnapshot]) -> f32 {
        let cutoff = crate::SerializableInstant::now() - Duration::from_secs(7 * 24 * 3600); // Last 7 days

        let recent_events: Vec<_> = history.iter()
            .flat_map(|s| &s.events)
            .filter(|e| e.timestamp > cutoff)
            .collect();

        if recent_events.is_empty() {
            return 0.0;
        }

        let total_impact: f32 = recent_events.iter().map(|e| e.impact).sum();
        let time_decay_factor = 0.8; // Recent events have more impact

        (total_impact / recent_events.len() as f32) * time_decay_factor
    }

    fn calculate_reputation_trend(&self, history: &[PerformanceSnapshot]) -> ReputationTrend {
        if history.len() < 5 {
            return ReputationTrend::default();
        }

        // Calculate reputation scores over time
        let window_size = 10;
        let mut scores = Vec::new();

        for window_start in 0..history.len().saturating_sub(window_size) {
            let window = &history[window_start..window_start + window_size];
            let reliability = self.calculate_reliability_metrics(window);
            let performance = self.calculate_average_performance(window);
            let score = self.calculate_overall_score(&reliability, &performance, window);
            scores.push(score);
        }

        if scores.len() < 2 {
            return ReputationTrend::default();
        }

        // Calculate linear regression slope
        let n = scores.len() as f32;
        let sum_x: f32 = (0..scores.len()).map(|i| i as f32).sum();
        let sum_y: f32 = scores.iter().sum();
        let sum_xy: f32 = scores.iter().enumerate().map(|(i, &y)| i as f32 * y).sum();
        let sum_x2: f32 = (0..scores.len()).map(|i| (i as f32).powi(2)).sum();

        let slope = (n * sum_xy - sum_x * sum_y) / (n * sum_x2 - sum_x.powi(2));

        let direction = if slope > 0.05 {
            TrendDirection::StronglyImproving
        } else if slope > 0.02 {
            TrendDirection::Improving
        } else if slope > -0.02 {
            TrendDirection::Stable
        } else if slope > -0.05 {
            TrendDirection::Declining
        } else {
            TrendDirection::StronglyDeclining
        };

        // Calculate confidence based on variance
        let mean_score = scores.iter().sum::<f32>() / n;
        let variance = scores.iter().map(|&s| (s - mean_score).powi(2)).sum::<f32>() / n;
        let confidence = 1.0 / (1.0 + variance * 10.0);

        ReputationTrend {
            direction,
            slope,
            confidence,
            time_window: Duration::from_secs(24 * 3600 * scores.len() as u64),
        }
    }

    fn calculate_storage_suitability_score(&self, reputation: &NodeReputation) -> f32 {
        let base_score = reputation.overall_score;
        let reliability_bonus = if reputation.reliability_metrics.consecutive_failures == 0 { 0.1 } else { 0.0 };
        let performance_bonus = if reputation.performance_metrics.average_latency < Duration::from_millis(500) { 0.05 } else { 0.0 };
        let trend_bonus = match reputation.reputation_trend.direction {
            TrendDirection::StronglyImproving | TrendDirection::Improving => 0.05,
            TrendDirection::Declining | TrendDirection::StronglyDeclining => -0.1,
            _ => 0.0,
        };

        base_score + reliability_bonus + performance_bonus + trend_bonus
    }

    async fn apply_immediate_reputation_change(&mut self, node_id: &str, event: &ReputationEvent) {
        if let Some(reputation) = self.node_reputations.get_mut(node_id) {
            let change = match event.severity {
                EventSeverity::Critical => -0.2,
                EventSeverity::Major => -0.1,
                EventSeverity::Minor => -0.05,
                EventSeverity::Positive => 0.05,
                EventSeverity::Exceptional => 0.1,
                EventSeverity::Neutral => 0.0,
            };

            reputation.overall_score = (reputation.overall_score + change).max(0.0).min(1.0);
        }
    }

    fn get_recent_events(&self, history: &[PerformanceSnapshot], window: Duration) -> Vec<ReputationEvent> {
        let cutoff = crate::SerializableInstant::now() - window;
        history.iter()
            .flat_map(|s| &s.events)
            .filter(|e| e.timestamp > cutoff)
            .cloned()
            .collect()
    }

    fn count_consecutive_failures(&self, events: &[&ReputationEvent]) -> u32 {
        let mut consecutive = 0;
        let mut max_consecutive = 0;

        for event in events.iter().rev() {
            if matches!(event.event_type, EventType::NodeFailure) {
                consecutive += 1;
                max_consecutive = max_consecutive.max(consecutive);
            } else {
                consecutive = 0;
            }
        }

        max_consecutive
    }

    fn calculate_variance(&self, values: &[f32]) -> f32 {
        if values.is_empty() {
            return 0.0;
        }

        let mean = values.iter().sum::<f32>() / values.len() as f32;
        values.iter().map(|&x| (x - mean).powi(2)).sum::<f32>() / values.len() as f32
    }
}

impl Default for ReputationWeights {
    fn default() -> Self {
        Self {
            uptime: 0.3,
            data_integrity: 0.25,
            performance: 0.2,
            recovery_ability: 0.1,
            consistency: 0.1,
            network_contribution: 0.05,
        }
    }
}

impl Default for ReliabilityMetrics {
    fn default() -> Self {
        Self {
            uptime_score: 0.5,
            data_integrity_score: 1.0,
            response_consistency: 0.5,
            failure_recovery_time: Duration::from_secs(300),
            consecutive_failures: 0,
            mean_time_between_failures: Duration::from_secs(u64::MAX),
        }
    }
}

impl Default for PerformanceMetrics {
    fn default() -> Self {
        Self {
            average_latency: Duration::from_millis(500),
            throughput_score: 0.5,
            storage_efficiency: 0.5,
            bandwidth_utilization: 0.5,
            resource_stability: 0.5,
            load_handling_capacity: 0.5,
        }
    }
}

impl Default for ReputationTrend {
    fn default() -> Self {
        Self {
            direction: TrendDirection::Stable,
            slope: 0.0,
            confidence: 0.5,
            time_window: Duration::from_secs(7 * 24 * 3600),
        }
    }
}

impl Default for NetworkStats {
    fn default() -> Self {
        Self {
            average_uptime: 0.95,
            median_latency: Duration::from_millis(200),
            total_nodes: 0,
            healthy_nodes: 0,
            network_quality_score: 0.8,
        }
    }
}