//! Network Health Early Warning System
//!
//! Monitors overall network health and provides early warnings for potential issues

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use tokio::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkHealthReport {
    pub timestamp: crate::SerializableInstant,
    pub overall_health_score: f32, // 0.0 to 1.0
    pub critical_alerts: Vec<HealthAlert>,
    pub warnings: Vec<HealthAlert>,
    pub network_metrics: GlobalNetworkMetrics,
    pub regional_health: HashMap<String, RegionalHealth>,
    pub trend_analysis: HealthTrend,
    pub risk_assessment: RiskAssessment,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthAlert {
    pub id: String,
    pub severity: AlertSeverity,
    pub alert_type: AlertType,
    pub message: String,
    pub affected_nodes: Vec<String>,
    pub affected_regions: Vec<String>,
    pub first_detected: crate::SerializableInstant,
    pub estimated_impact: ImpactAssessment,
    pub recommended_actions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertSeverity {
    Critical,  // Immediate action required
    High,      // Action required within 1 hour
    Medium,    // Action required within 4 hours
    Low,       // Monitor and plan
    Info,      // Informational only
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertType {
    NodeFailures,
    NetworkPartition,
    StorageCapacity,
    PerformanceDegradation,
    SecurityThreat,
    DataIntegrity,
    ConnectivityIssues,
    ResourceExhaustion,
    GeographicDisturbance,
    SystemOverload,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImpactAssessment {
    pub affected_data_percentage: f32,
    pub performance_impact: f32,
    pub availability_risk: f32,
    pub estimated_users_affected: u32,
    pub data_at_risk: u64, // bytes
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalNetworkMetrics {
    pub total_nodes: u32,
    pub healthy_nodes: u32,
    pub unhealthy_nodes: u32,
    pub offline_nodes: u32,
    pub average_uptime: f32,
    pub network_latency_p50: Duration,
    pub network_latency_p95: Duration,
    pub total_storage_capacity: u64,
    pub used_storage_capacity: u64,
    pub data_redundancy_level: f32,
    pub throughput_mbps: f32,
    pub error_rate: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegionalHealth {
    pub region: String,
    pub health_score: f32,
    pub node_count: u32,
    pub healthy_nodes: u32,
    pub average_latency: Duration,
    pub storage_utilization: f32,
    pub connectivity_status: ConnectivityStatus,
    pub risk_factors: Vec<RegionalRiskFactor>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConnectivityStatus {
    Excellent,   // All connections stable
    Good,        // Minor connectivity issues
    Degraded,    // Noticeable connectivity problems
    Poor,        // Significant connectivity issues
    Critical,    // Major connectivity failures
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RegionalRiskFactor {
    HighLatency,
    NodeConcentration,
    InfrastructureIssues,
    NetworkCongestion,
    GeographicEvents,
    RegulatoryChanges,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthTrend {
    pub direction: TrendDirection,
    pub confidence: f32,
    pub time_window: Duration,
    pub key_indicators: Vec<TrendIndicator>,
    pub predicted_issues: Vec<PredictedIssue>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TrendDirection {
    StronglyImproving,
    Improving,
    Stable,
    Declining,
    StronglyDeclining,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendIndicator {
    pub metric: String,
    pub current_value: f32,
    pub trend_direction: TrendDirection,
    pub rate_of_change: f32,
    pub significance: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredictedIssue {
    pub issue_type: AlertType,
    pub probability: f32,
    pub predicted_time: crate::SerializableInstant,
    pub potential_impact: ImpactAssessment,
    pub prevention_actions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskAssessment {
    pub overall_risk_level: RiskLevel,
    pub data_loss_risk: f32,
    pub availability_risk: f32,
    pub performance_risk: f32,
    pub security_risk: f32,
    pub mitigation_effectiveness: f32,
    pub risk_factors: Vec<NetworkRiskFactor>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RiskLevel {
    VeryLow,
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkRiskFactor {
    pub factor_type: String,
    pub severity: f32,
    pub likelihood: f32,
    pub impact_scope: String,
    pub mitigation_options: Vec<String>,
}

pub struct NetworkHealthMonitor {
    health_history: VecDeque<NetworkHealthReport>,
    active_alerts: HashMap<String, HealthAlert>,
    node_health_cache: HashMap<String, NodeHealthStatus>,
    regional_monitors: HashMap<String, RegionalMonitor>,
    alert_thresholds: AlertThresholds,
    predictive_models: HashMap<String, HealthPredictionModel>,
}

#[derive(Debug, Clone)]
struct NodeHealthStatus {
    node_id: String,
    last_seen: crate::SerializableInstant,
    health_score: f32,
    metrics: NodeMetrics,
    status: NodeStatus,
}

#[derive(Debug, Clone)]
struct NodeMetrics {
    cpu_usage: f32,
    memory_usage: f32,
    disk_usage: f32,
    network_latency: Duration,
    error_count: u32,
    uptime: Duration,
}

#[derive(Debug, Clone)]
enum NodeStatus {
    Healthy,
    Warning,
    Critical,
    Offline,
    Unknown,
}

struct RegionalMonitor {
    region: String,
    nodes: Vec<String>,
    health_score_history: VecDeque<f32>,
    connectivity_matrix: HashMap<String, HashMap<String, Duration>>,
    last_health_check: crate::SerializableInstant,
}

#[derive(Debug, Clone)]
struct AlertThresholds {
    node_failure_threshold: f32,
    network_latency_threshold: Duration,
    storage_utilization_threshold: f32,
    error_rate_threshold: f32,
    uptime_threshold: f32,
    redundancy_threshold: f32,
}

struct HealthPredictionModel {
    model_type: String,
    weights: Vec<f32>,
    accuracy: f32,
    training_data: VecDeque<HealthDataPoint>,
    last_prediction: Option<PredictedIssue>,
}

#[derive(Debug, Clone)]
struct HealthDataPoint {
    timestamp: crate::SerializableInstant,
    metrics: Vec<f32>,
    outcome: Option<AlertType>,
}

impl NetworkHealthMonitor {
    pub fn new() -> Self {
        Self {
            health_history: VecDeque::with_capacity(1440), // 24 hours of minute-by-minute data
            active_alerts: HashMap::new(),
            node_health_cache: HashMap::new(),
            regional_monitors: HashMap::new(),
            alert_thresholds: AlertThresholds::default(),
            predictive_models: HashMap::new(),
        }
    }

    pub async fn perform_health_check(&mut self) -> NetworkHealthReport {
        let timestamp = crate::SerializableInstant::now();

        // Update node health status
        self.update_node_health_status().await;

        // Calculate global metrics
        let network_metrics = self.calculate_global_metrics().await;

        // Assess regional health
        let regional_health = self.assess_regional_health().await;

        // Analyze trends
        let trend_analysis = self.analyze_health_trends();

        // Assess risks
        let risk_assessment = self.assess_network_risks(&network_metrics, &regional_health);

        // Calculate overall health score
        let overall_health_score = self.calculate_overall_health_score(&network_metrics, &regional_health, &risk_assessment);

        // Generate alerts
        let (critical_alerts, warnings) = self.generate_health_alerts(&network_metrics, &regional_health, &risk_assessment).await;

        let report = NetworkHealthReport {
            timestamp,
            overall_health_score,
            critical_alerts,
            warnings,
            network_metrics,
            regional_health,
            trend_analysis,
            risk_assessment,
        };

        // Store in history
        self.health_history.push_back(report.clone());
        if self.health_history.len() > 1440 {
            self.health_history.pop_front();
        }

        // Update predictive models
        self.update_predictive_models(&report).await;

        report
    }

    pub async fn get_current_health_status(&self) -> Option<NetworkHealthReport> {
        self.health_history.back().cloned()
    }

    pub fn get_active_alerts(&self) -> Vec<&HealthAlert> {
        self.active_alerts.values().collect()
    }

    pub fn get_critical_alerts(&self) -> Vec<&HealthAlert> {
        self.active_alerts.values()
            .filter(|alert| matches!(alert.severity, AlertSeverity::Critical))
            .collect()
    }

    pub async fn predict_future_issues(&self, time_horizon: Duration) -> Vec<PredictedIssue> {
        let mut predictions = Vec::new();

        for model in self.predictive_models.values() {
            if let Some(prediction) = self.run_prediction_model(model, time_horizon).await {
                predictions.push(prediction);
            }
        }

        // Sort by probability and impact
        predictions.sort_by(|a, b| {
            let score_a = a.probability * a.potential_impact.availability_risk;
            let score_b = b.probability * b.potential_impact.availability_risk;
            score_b.partial_cmp(&score_a).unwrap()
        });

        predictions
    }

    async fn update_node_health_status(&mut self) {
        // Placeholder: In reality, this would collect metrics from all nodes
        let now = crate::SerializableInstant::now();

        for node_id in ["node1", "node2", "node3"].iter() {
            let health_status = NodeHealthStatus {
                node_id: node_id.to_string(),
                last_seen: now,
                health_score: 0.85 + (now.elapsed().as_secs() as f32 % 100.0) / 1000.0,
                metrics: NodeMetrics {
                    cpu_usage: 0.4,
                    memory_usage: 0.6,
                    disk_usage: 0.3,
                    network_latency: Duration::from_millis(50),
                    error_count: 2,
                    uptime: Duration::from_secs(86400 * 30), // 30 days
                },
                status: NodeStatus::Healthy,
            };

            self.node_health_cache.insert(node_id.to_string(), health_status);
        }
    }

    async fn calculate_global_metrics(&self) -> GlobalNetworkMetrics {
        let total_nodes = self.node_health_cache.len() as u32;
        let healthy_nodes = self.node_health_cache.values()
            .filter(|node| matches!(node.status, NodeStatus::Healthy))
            .count() as u32;
        let unhealthy_nodes = self.node_health_cache.values()
            .filter(|node| matches!(node.status, NodeStatus::Warning | NodeStatus::Critical))
            .count() as u32;
        let offline_nodes = self.node_health_cache.values()
            .filter(|node| matches!(node.status, NodeStatus::Offline))
            .count() as u32;

        let average_uptime = if !self.node_health_cache.is_empty() {
            self.node_health_cache.values()
                .map(|node| node.health_score)
                .sum::<f32>() / total_nodes as f32
        } else {
            0.0
        };

        let latencies: Vec<_> = self.node_health_cache.values()
            .map(|node| node.metrics.network_latency.as_millis() as f32)
            .collect();

        let network_latency_p50 = Duration::from_millis(
            self.calculate_percentile(&latencies, 0.5) as u64
        );
        let network_latency_p95 = Duration::from_millis(
            self.calculate_percentile(&latencies, 0.95) as u64
        );

        let total_storage_capacity = 100 * 1024 * 1024 * 1024u64; // 100GB per node
        let used_storage_capacity = (total_storage_capacity as f32 * 0.4) as u64; // 40% used

        let error_rate = self.node_health_cache.values()
            .map(|node| node.metrics.error_count as f32)
            .sum::<f32>() / (total_nodes as f32).max(1.0) / 1000.0;

        GlobalNetworkMetrics {
            total_nodes,
            healthy_nodes,
            unhealthy_nodes,
            offline_nodes,
            average_uptime,
            network_latency_p50,
            network_latency_p95,
            total_storage_capacity: total_storage_capacity * total_nodes as u64,
            used_storage_capacity: used_storage_capacity * total_nodes as u64,
            data_redundancy_level: 2.5, // Average redundancy factor
            throughput_mbps: 150.0,
            error_rate,
        }
    }

    async fn assess_regional_health(&mut self) -> HashMap<String, RegionalHealth> {
        let mut regional_health = HashMap::new();

        let regions = vec!["us-east", "us-west", "europe", "asia-pacific"];

        for region in regions {
            let nodes_in_region: Vec<_> = self.node_health_cache.keys()
                .filter(|_| true) // Placeholder: filter by region
                .take(2)
                .cloned()
                .collect();

            let node_count = nodes_in_region.len() as u32;
            let healthy_nodes = nodes_in_region.iter()
                .filter(|node_id| {
                    if let Some(node) = self.node_health_cache.get(*node_id) {
                        matches!(node.status, NodeStatus::Healthy)
                    } else {
                        false
                    }
                })
                .count() as u32;

            let average_latency = if !nodes_in_region.is_empty() {
                let total_latency: u128 = nodes_in_region.iter()
                    .filter_map(|node_id| self.node_health_cache.get(node_id))
                    .map(|node| node.metrics.network_latency.as_millis())
                    .sum();
                Duration::from_millis((total_latency / nodes_in_region.len() as u128) as u64)
            } else {
                Duration::from_millis(0)
            };

            let health_score = if node_count > 0 {
                healthy_nodes as f32 / node_count as f32
            } else {
                0.0
            };

            let connectivity_status = if health_score > 0.9 {
                ConnectivityStatus::Excellent
            } else if health_score > 0.8 {
                ConnectivityStatus::Good
            } else if health_score > 0.6 {
                ConnectivityStatus::Degraded
            } else if health_score > 0.3 {
                ConnectivityStatus::Poor
            } else {
                ConnectivityStatus::Critical
            };

            let regional = RegionalHealth {
                region: region.to_string(),
                health_score,
                node_count,
                healthy_nodes,
                average_latency,
                storage_utilization: 0.4, // 40% utilized
                connectivity_status,
                risk_factors: self.identify_regional_risk_factors(region, health_score),
            };

            regional_health.insert(region.to_string(), regional);
        }

        regional_health
    }

    fn analyze_health_trends(&self) -> HealthTrend {
        if self.health_history.len() < 5 {
            return HealthTrend::default();
        }

        let recent_scores: Vec<_> = self.health_history.iter()
            .rev()
            .take(60) // Last hour
            .map(|report| report.overall_health_score)
            .collect();

        let trend_direction = self.calculate_trend_direction(&recent_scores);
        let confidence = self.calculate_trend_confidence(&recent_scores);

        let key_indicators = vec![
            TrendIndicator {
                metric: "Overall Health".to_string(),
                current_value: recent_scores.first().copied().unwrap_or(0.0),
                trend_direction: trend_direction.clone(),
                rate_of_change: self.calculate_rate_of_change(&recent_scores),
                significance: 0.9,
            },
            TrendIndicator {
                metric: "Node Availability".to_string(),
                current_value: 0.95,
                trend_direction: TrendDirection::Stable,
                rate_of_change: 0.001,
                significance: 0.8,
            },
        ];

        let predicted_issues = self.generate_trend_predictions(&recent_scores);

        HealthTrend {
            direction: trend_direction,
            confidence,
            time_window: Duration::from_secs(3600),
            key_indicators,
            predicted_issues,
        }
    }

    fn assess_network_risks(
        &self,
        metrics: &GlobalNetworkMetrics,
        regional_health: &HashMap<String, RegionalHealth>
    ) -> RiskAssessment {
        let data_loss_risk = if metrics.data_redundancy_level < 2.0 { 0.8 }
                            else if metrics.data_redundancy_level < 2.5 { 0.4 }
                            else { 0.1 };

        let availability_risk = 1.0 - (metrics.healthy_nodes as f32 / metrics.total_nodes as f32);

        let performance_risk = if metrics.network_latency_p95 > Duration::from_millis(1000) { 0.7 }
                              else if metrics.network_latency_p95 > Duration::from_millis(500) { 0.4 }
                              else { 0.1 };

        let security_risk = metrics.error_rate * 10.0;

        let overall_risk_score = (data_loss_risk + availability_risk + performance_risk + security_risk) / 4.0;
        let overall_risk_level = if overall_risk_score > 0.8 { RiskLevel::Critical }
                                else if overall_risk_score > 0.6 { RiskLevel::High }
                                else if overall_risk_score > 0.4 { RiskLevel::Medium }
                                else if overall_risk_score > 0.2 { RiskLevel::Low }
                                else { RiskLevel::VeryLow };

        let risk_factors = vec![
            NetworkRiskFactor {
                factor_type: "Node Concentration".to_string(),
                severity: 0.3,
                likelihood: 0.4,
                impact_scope: "Regional availability".to_string(),
                mitigation_options: vec!["Increase geographic distribution".to_string()],
            },
        ];

        RiskAssessment {
            overall_risk_level,
            data_loss_risk,
            availability_risk,
            performance_risk,
            security_risk,
            mitigation_effectiveness: 0.7,
            risk_factors,
        }
    }

    fn calculate_overall_health_score(
        &self,
        metrics: &GlobalNetworkMetrics,
        regional_health: &HashMap<String, RegionalHealth>,
        risk_assessment: &RiskAssessment,
    ) -> f32 {
        let availability_score = metrics.healthy_nodes as f32 / metrics.total_nodes as f32;
        let performance_score = if metrics.network_latency_p95 < Duration::from_millis(200) { 1.0 }
                               else if metrics.network_latency_p95 < Duration::from_millis(500) { 0.8 }
                               else if metrics.network_latency_p95 < Duration::from_millis(1000) { 0.6 }
                               else { 0.3 };

        let regional_score = if regional_health.is_empty() { 0.5 } else {
            regional_health.values().map(|r| r.health_score).sum::<f32>() / regional_health.len() as f32
        };

        let risk_score = 1.0 - risk_assessment.availability_risk;

        (availability_score * 0.4 + performance_score * 0.3 + regional_score * 0.2 + risk_score * 0.1)
    }

    async fn generate_health_alerts(
        &mut self,
        metrics: &GlobalNetworkMetrics,
        regional_health: &HashMap<String, RegionalHealth>,
        risk_assessment: &RiskAssessment,
    ) -> (Vec<HealthAlert>, Vec<HealthAlert>) {
        let mut critical_alerts = Vec::new();
        let mut warnings = Vec::new();

        // Check for critical node failures
        if metrics.offline_nodes > metrics.total_nodes / 4 {
            let alert = HealthAlert {
                id: format!("critical_node_failures_{}", crate::SerializableInstant::now().elapsed().as_secs()),
                severity: AlertSeverity::Critical,
                alert_type: AlertType::NodeFailures,
                message: format!("{} nodes are offline ({}% of network)", metrics.offline_nodes,
                    (metrics.offline_nodes as f32 / metrics.total_nodes as f32 * 100.0) as u32),
                affected_nodes: vec!["multiple".to_string()],
                affected_regions: regional_health.keys().cloned().collect(),
                first_detected: crate::SerializableInstant::now(),
                estimated_impact: ImpactAssessment {
                    affected_data_percentage: metrics.offline_nodes as f32 / metrics.total_nodes as f32,
                    performance_impact: 0.8,
                    availability_risk: 0.9,
                    estimated_users_affected: 10000,
                    data_at_risk: metrics.used_storage_capacity / 4,
                },
                recommended_actions: vec![
                    "Investigate node failures immediately".to_string(),
                    "Activate emergency replication".to_string(),
                    "Contact affected regions".to_string(),
                ],
            };
            critical_alerts.push(alert);
        }

        // Check storage capacity
        let storage_utilization = metrics.used_storage_capacity as f32 / metrics.total_storage_capacity as f32;
        if storage_utilization > 0.9 {
            let alert = HealthAlert {
                id: format!("storage_capacity_{}", crate::SerializableInstant::now().elapsed().as_secs()),
                severity: AlertSeverity::High,
                alert_type: AlertType::StorageCapacity,
                message: format!("Network storage is {}% full", (storage_utilization * 100.0) as u32),
                affected_nodes: vec!["all".to_string()],
                affected_regions: regional_health.keys().cloned().collect(),
                first_detected: crate::SerializableInstant::now(),
                estimated_impact: ImpactAssessment {
                    affected_data_percentage: 1.0,
                    performance_impact: 0.6,
                    availability_risk: 0.4,
                    estimated_users_affected: 50000,
                    data_at_risk: metrics.used_storage_capacity,
                },
                recommended_actions: vec![
                    "Add storage capacity".to_string(),
                    "Implement data cleanup policies".to_string(),
                    "Scale up storage nodes".to_string(),
                ],
            };
            warnings.push(alert);
        }

        // Check network performance
        if metrics.network_latency_p95 > Duration::from_millis(1000) {
            let alert = HealthAlert {
                id: format!("network_latency_{}", crate::SerializableInstant::now().elapsed().as_secs()),
                severity: AlertSeverity::Medium,
                alert_type: AlertType::PerformanceDegradation,
                message: format!("Network latency is high: {}ms (95th percentile)",
                    metrics.network_latency_p95.as_millis()),
                affected_nodes: vec!["multiple".to_string()],
                affected_regions: regional_health.keys().cloned().collect(),
                first_detected: crate::SerializableInstant::now(),
                estimated_impact: ImpactAssessment {
                    affected_data_percentage: 0.0,
                    performance_impact: 0.7,
                    availability_risk: 0.2,
                    estimated_users_affected: 25000,
                    data_at_risk: 0,
                },
                recommended_actions: vec![
                    "Investigate network congestion".to_string(),
                    "Optimize routing".to_string(),
                    "Check regional connectivity".to_string(),
                ],
            };
            warnings.push(alert);
        }

        (critical_alerts, warnings)
    }

    async fn update_predictive_models(&mut self, report: &NetworkHealthReport) {
        // Update models based on new health report data
        let data_point = HealthDataPoint {
            timestamp: report.timestamp,
            metrics: vec![
                report.overall_health_score,
                report.network_metrics.healthy_nodes as f32 / report.network_metrics.total_nodes as f32,
                report.network_metrics.network_latency_p95.as_millis() as f32 / 1000.0,
                report.network_metrics.error_rate,
            ],
            outcome: None, // Would be populated when actual issues occur
        };

        for model in self.predictive_models.values_mut() {
            model.training_data.push_back(data_point.clone());
            if model.training_data.len() > 1000 {
                model.training_data.pop_front();
            }
        }
    }

    async fn run_prediction_model(&self, model: &HealthPredictionModel, time_horizon: Duration) -> Option<PredictedIssue> {
        if model.training_data.len() < 10 {
            return None;
        }

        // Simple prediction based on recent trends
        let recent_health: Vec<_> = model.training_data.iter()
            .rev()
            .take(10)
            .map(|dp| dp.metrics[0])
            .collect();

        let trend = self.calculate_rate_of_change(&recent_health);
        let current_health = recent_health.first().copied().unwrap_or(0.5);

        if trend < -0.01 && current_health < 0.7 {
            Some(PredictedIssue {
                issue_type: AlertType::PerformanceDegradation,
                probability: 0.6,
                predicted_time: crate::SerializableInstant::now() + time_horizon,
                potential_impact: ImpactAssessment {
                    affected_data_percentage: 0.3,
                    performance_impact: 0.5,
                    availability_risk: 0.3,
                    estimated_users_affected: 15000,
                    data_at_risk: 1024 * 1024 * 1024, // 1GB
                },
                prevention_actions: vec![
                    "Increase monitoring frequency".to_string(),
                    "Prepare additional resources".to_string(),
                ],
            })
        } else {
            None
        }
    }

    fn identify_regional_risk_factors(&self, _region: &str, health_score: f32) -> Vec<RegionalRiskFactor> {
        let mut factors = Vec::new();

        if health_score < 0.7 {
            factors.push(RegionalRiskFactor::InfrastructureIssues);
        }

        factors
    }

    fn calculate_percentile(&self, values: &[f32], percentile: f32) -> f32 {
        if values.is_empty() {
            return 0.0;
        }

        let mut sorted_values = values.to_vec();
        sorted_values.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let index = (percentile * (sorted_values.len() - 1) as f32).round() as usize;
        sorted_values[index.min(sorted_values.len() - 1)]
    }

    fn calculate_trend_direction(&self, values: &[f32]) -> TrendDirection {
        if values.len() < 2 {
            return TrendDirection::Stable;
        }

        let slope = self.calculate_rate_of_change(values);

        if slope > 0.05 { TrendDirection::StronglyImproving }
        else if slope > 0.02 { TrendDirection::Improving }
        else if slope > -0.02 { TrendDirection::Stable }
        else if slope > -0.05 { TrendDirection::Declining }
        else { TrendDirection::StronglyDeclining }
    }

    fn calculate_trend_confidence(&self, values: &[f32]) -> f32 {
        if values.len() < 3 {
            return 0.1;
        }

        let mean = values.iter().sum::<f32>() / values.len() as f32;
        let variance = values.iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f32>() / values.len() as f32;

        1.0 / (1.0 + variance * 10.0)
    }

    fn calculate_rate_of_change(&self, values: &[f32]) -> f32 {
        if values.len() < 2 {
            return 0.0;
        }

        let first = values.last().copied().unwrap_or(0.0);
        let last = values.first().copied().unwrap_or(0.0);

        (last - first) / values.len() as f32
    }

    fn generate_trend_predictions(&self, _values: &[f32]) -> Vec<PredictedIssue> {
        // Placeholder: Would generate predictions based on trend analysis
        Vec::new()
    }
}

impl Default for AlertThresholds {
    fn default() -> Self {
        Self {
            node_failure_threshold: 0.1,     // 10% node failures trigger alert
            network_latency_threshold: Duration::from_millis(500),
            storage_utilization_threshold: 0.85, // 85% storage usage
            error_rate_threshold: 0.05,     // 5% error rate
            uptime_threshold: 0.95,         // 95% uptime required
            redundancy_threshold: 2.0,      // Minimum 2x redundancy
        }
    }
}

impl Default for HealthTrend {
    fn default() -> Self {
        Self {
            direction: TrendDirection::Stable,
            confidence: 0.5,
            time_window: Duration::from_secs(3600),
            key_indicators: Vec::new(),
            predicted_issues: Vec::new(),
        }
    }
}