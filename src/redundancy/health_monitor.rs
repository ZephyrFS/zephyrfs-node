//! Real-Time Chunk Health Monitoring
//!
//! Comprehensive monitoring system that tracks chunk health, replica status,
//! and data integrity across the distributed network

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque, BTreeMap};
use chrono::{DateTime, Utc, Duration};
use tokio::time::{sleep, Duration as TokioDuration};

use crate::economics::GeographicRegion;

/// Real-time chunk health monitoring system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkHealthMonitor {
    /// Health status for all monitored chunks
    pub chunk_health: HashMap<String, ChunkHealth>,
    /// Node health tracking
    pub node_health: HashMap<String, NodeHealth>,
    /// Real-time metrics
    pub monitoring_metrics: MonitoringMetrics,
    /// Alert configuration
    pub alert_config: AlertConfiguration,
    /// Health check scheduler
    pub check_scheduler: HealthCheckScheduler,
    /// Historical health data
    pub health_history: HashMap<String, VecDeque<HealthSnapshot>>,
    /// Performance analytics
    pub analytics: HealthAnalytics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkHealth {
    pub chunk_id: String,
    pub overall_health: HealthStatus,
    pub replica_health: Vec<ReplicaHealth>,
    pub integrity_status: IntegrityStatus,
    pub availability_score: f64,
    pub durability_score: f64,
    pub performance_metrics: ChunkPerformanceMetrics,
    pub last_verified: DateTime<Utc>,
    pub next_check_due: DateTime<Utc>,
    pub risk_factors: Vec<RiskFactor>,
    pub repair_history: Vec<RepairRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplicaHealth {
    pub replica_id: String,
    pub node_id: String,
    pub region: GeographicRegion,
    pub status: ReplicaStatus,
    pub health_score: f64,
    pub last_accessed: DateTime<Utc>,
    pub last_verified: DateTime<Utc>,
    pub integrity_hash: String,
    pub performance_metrics: ReplicaPerformanceMetrics,
    pub connectivity_status: ConnectivityStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HealthStatus {
    Excellent,   // All replicas healthy, high durability
    Good,        // Most replicas healthy, adequate durability
    Warning,     // Some replicas degraded, durability at risk
    Critical,    // Many replicas unhealthy, immediate action needed
    Failed,      // Cannot guarantee data availability
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReplicaStatus {
    Healthy,
    Degraded,
    Slow,
    Unreachable,
    Corrupted,
    Missing,
    Verifying,
    Repairing,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IntegrityStatus {
    Verified,
    Pending,
    Suspicious,
    Corrupted,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConnectivityStatus {
    Online,
    Intermittent,
    Offline,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkPerformanceMetrics {
    pub avg_response_time_ms: f64,
    pub success_rate: f64,
    pub throughput_mbps: f64,
    pub error_rate: f64,
    pub access_frequency: AccessFrequency,
    pub bandwidth_utilization: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplicaPerformanceMetrics {
    pub response_time_ms: f64,
    pub transfer_speed_mbps: f64,
    pub success_rate: f64,
    pub error_count: u32,
    pub last_error: Option<String>,
    pub uptime_percentage: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AccessFrequency {
    VeryHigh,  // > 1000 accesses/day
    High,      // 100-1000 accesses/day
    Medium,    // 10-100 accesses/day
    Low,       // 1-10 accesses/day
    VeryLow,   // < 1 access/day
    Archived,  // Not accessed recently
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskFactor {
    pub risk_type: RiskType,
    pub severity: RiskSeverity,
    pub probability: f64, // 0.0 to 1.0
    pub impact: f64,      // 0.0 to 1.0
    pub description: String,
    pub mitigation_actions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RiskType {
    NodeFailure,
    NetworkPartition,
    GeographicRisk,
    PerformanceDegradation,
    CapacityLimits,
    ComplianceViolation,
    SecurityThreat,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RiskSeverity {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepairRecord {
    pub repair_id: String,
    pub timestamp: DateTime<Utc>,
    pub repair_type: RepairType,
    pub affected_replicas: Vec<String>,
    pub repair_strategy: String,
    pub success: bool,
    pub duration_seconds: u64,
    pub cost: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RepairType {
    ReplicationIncrease,
    ReplicaReplacement,
    IntegrityRepair,
    PerformanceOptimization,
    GeographicRebalancing,
    EmergencyRecovery,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeHealth {
    pub node_id: String,
    pub region: GeographicRegion,
    pub overall_health: HealthStatus,
    pub uptime_percentage: f64,
    pub response_time_ms: f64,
    pub bandwidth_mbps: f64,
    pub storage_health: StorageHealth,
    pub connectivity_quality: ConnectivityQuality,
    pub load_metrics: LoadMetrics,
    pub last_seen: DateTime<Utc>,
    pub consecutive_failures: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageHealth {
    pub total_capacity_gb: u64,
    pub used_capacity_gb: u64,
    pub available_capacity_gb: u64,
    pub disk_health_score: f64,
    pub io_performance: f64,
    pub error_rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectivityQuality {
    pub latency_ms: f64,
    pub jitter_ms: f64,
    pub packet_loss: f64,
    pub bandwidth_stability: f64,
    pub connection_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadMetrics {
    pub cpu_usage: f64,
    pub memory_usage: f64,
    pub network_utilization: f64,
    pub disk_utilization: f64,
    pub active_connections: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringMetrics {
    pub total_chunks_monitored: u64,
    pub healthy_chunks: u64,
    pub degraded_chunks: u64,
    pub critical_chunks: u64,
    pub failed_chunks: u64,
    pub total_replicas: u64,
    pub healthy_replicas: u64,
    pub degraded_replicas: u64,
    pub average_health_score: f64,
    pub monitoring_efficiency: f64,
    pub last_updated: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertConfiguration {
    pub enable_alerts: bool,
    pub health_thresholds: HealthThresholds,
    pub notification_channels: Vec<NotificationChannel>,
    pub alert_cooldown_minutes: u32,
    pub escalation_rules: Vec<EscalationRule>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthThresholds {
    pub critical_health_score: f64,
    pub warning_health_score: f64,
    pub max_response_time_ms: f64,
    pub min_success_rate: f64,
    pub max_error_rate: f64,
    pub min_replica_count: u32,
    pub max_consecutive_failures: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationChannel {
    pub channel_type: NotificationType,
    pub endpoint: String,
    pub severity_filter: Vec<RiskSeverity>,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NotificationType {
    Email,
    Slack,
    Webhook,
    SMS,
    PagerDuty,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EscalationRule {
    pub condition: String,
    pub delay_minutes: u32,
    pub action: EscalationAction,
    pub repeat_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EscalationAction {
    NotifyManager,
    AutoRepair,
    IncreaseReplication,
    EmergencyProtocol,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckScheduler {
    pub check_intervals: HashMap<HealthStatus, Duration>,
    pub priority_queue: BTreeMap<DateTime<Utc>, Vec<String>>, // chunk_ids
    pub concurrent_checks: u32,
    pub batch_size: u32,
    pub adaptive_scheduling: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthSnapshot {
    pub timestamp: DateTime<Utc>,
    pub health_status: HealthStatus,
    pub health_score: f64,
    pub replica_count: u32,
    pub healthy_replicas: u32,
    pub performance_metrics: ChunkPerformanceMetrics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthAnalytics {
    pub health_trends: HashMap<String, HealthTrend>,
    pub failure_patterns: Vec<FailurePattern>,
    pub performance_baselines: HashMap<String, PerformanceBaseline>,
    pub prediction_models: HashMap<String, PredictionModel>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthTrend {
    pub chunk_id: String,
    pub trend_direction: TrendDirection,
    pub trend_strength: f64,
    pub prediction_confidence: f64,
    pub time_to_critical: Option<Duration>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TrendDirection {
    Improving,
    Stable,
    Degrading,
    Volatile,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailurePattern {
    pub pattern_id: String,
    pub pattern_type: String,
    pub frequency: f64,
    pub affected_chunks: Vec<String>,
    pub common_factors: Vec<String>,
    pub prevention_strategies: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceBaseline {
    pub metric_name: String,
    pub baseline_value: f64,
    pub acceptable_variance: f64,
    pub seasonal_adjustments: HashMap<u8, f64>, // Month -> adjustment factor
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredictionModel {
    pub model_type: String,
    pub accuracy: f64,
    pub last_trained: DateTime<Utc>,
    pub parameters: HashMap<String, f64>,
}

impl Default for HealthThresholds {
    fn default() -> Self {
        Self {
            critical_health_score: 50.0,
            warning_health_score: 75.0,
            max_response_time_ms: 1000.0,
            min_success_rate: 95.0,
            max_error_rate: 5.0,
            min_replica_count: 3,
            max_consecutive_failures: 3,
        }
    }
}

impl ChunkHealthMonitor {
    /// Create new chunk health monitor
    pub fn new() -> Self {
        Self {
            chunk_health: HashMap::new(),
            node_health: HashMap::new(),
            monitoring_metrics: MonitoringMetrics {
                total_chunks_monitored: 0,
                healthy_chunks: 0,
                degraded_chunks: 0,
                critical_chunks: 0,
                failed_chunks: 0,
                total_replicas: 0,
                healthy_replicas: 0,
                degraded_replicas: 0,
                average_health_score: 100.0,
                monitoring_efficiency: 100.0,
                last_updated: Utc::now(),
            },
            alert_config: AlertConfiguration {
                enable_alerts: true,
                health_thresholds: HealthThresholds::default(),
                notification_channels: Vec::new(),
                alert_cooldown_minutes: 15,
                escalation_rules: Vec::new(),
            },
            check_scheduler: HealthCheckScheduler {
                check_intervals: HashMap::from([
                    (HealthStatus::Excellent, Duration::hours(24)),
                    (HealthStatus::Good, Duration::hours(6)),
                    (HealthStatus::Warning, Duration::hours(1)),
                    (HealthStatus::Critical, Duration::minutes(15)),
                    (HealthStatus::Failed, Duration::minutes(5)),
                ]),
                priority_queue: BTreeMap::new(),
                concurrent_checks: 10,
                batch_size: 100,
                adaptive_scheduling: true,
            },
            health_history: HashMap::new(),
            analytics: HealthAnalytics {
                health_trends: HashMap::new(),
                failure_patterns: Vec::new(),
                performance_baselines: HashMap::new(),
                prediction_models: HashMap::new(),
            },
        }
    }

    /// Add chunk for monitoring
    pub fn add_chunk_monitoring(&mut self, chunk_id: String, initial_replicas: Vec<ReplicaHealth>) {
        let health_score = self.calculate_chunk_health_score(&initial_replicas);
        let health_status = self.determine_health_status(health_score, &initial_replicas);

        let chunk_health = ChunkHealth {
            chunk_id: chunk_id.clone(),
            overall_health: health_status.clone(),
            replica_health: initial_replicas,
            integrity_status: IntegrityStatus::Pending,
            availability_score: health_score,
            durability_score: health_score,
            performance_metrics: ChunkPerformanceMetrics {
                avg_response_time_ms: 0.0,
                success_rate: 100.0,
                throughput_mbps: 0.0,
                error_rate: 0.0,
                access_frequency: AccessFrequency::Low,
                bandwidth_utilization: 0.0,
            },
            last_verified: Utc::now(),
            next_check_due: Utc::now() + self.get_check_interval(&health_status),
            risk_factors: Vec::new(),
            repair_history: Vec::new(),
        };

        self.chunk_health.insert(chunk_id.clone(), chunk_health);

        // Schedule health check
        self.schedule_health_check(chunk_id, Utc::now() + self.get_check_interval(&health_status));

        // Initialize health history
        self.health_history.insert(chunk_id, VecDeque::with_capacity(1000));

        self.update_monitoring_metrics();
    }

    /// Perform health check on a chunk
    pub async fn perform_health_check(&mut self, chunk_id: &str) -> Result<HealthCheckResult> {
        let chunk_health = self.chunk_health.get_mut(chunk_id)
            .ok_or_else(|| anyhow::anyhow!("Chunk not found in monitoring"))?;

        let mut check_results = Vec::new();
        let mut healthy_replicas = 0;
        let mut total_response_time = 0.0;

        // Check each replica
        for replica in &mut chunk_health.replica_health {
            let replica_result = self.check_replica_health(replica).await?;

            if matches!(replica_result.status, ReplicaStatus::Healthy) {
                healthy_replicas += 1;
            }

            total_response_time += replica_result.response_time_ms;
            check_results.push(replica_result);
        }

        // Update chunk health based on check results
        let new_health_score = self.calculate_chunk_health_score(&chunk_health.replica_health);
        let new_health_status = self.determine_health_status(new_health_score, &chunk_health.replica_health);

        chunk_health.overall_health = new_health_status.clone();
        chunk_health.availability_score = new_health_score;
        chunk_health.performance_metrics.avg_response_time_ms = total_response_time / check_results.len() as f64;
        chunk_health.performance_metrics.success_rate = (healthy_replicas as f64 / check_results.len() as f64) * 100.0;
        chunk_health.last_verified = Utc::now();
        chunk_health.next_check_due = Utc::now() + self.get_check_interval(&new_health_status);

        // Update risk factors
        chunk_health.risk_factors = self.assess_risk_factors(chunk_health);

        // Record health snapshot
        self.record_health_snapshot(chunk_id, chunk_health);

        // Schedule next check
        self.schedule_health_check(chunk_id.to_string(), chunk_health.next_check_due);

        // Check for alerts
        if self.alert_config.enable_alerts {
            self.check_alert_conditions(chunk_id, chunk_health).await?;
        }

        self.update_monitoring_metrics();

        Ok(HealthCheckResult {
            chunk_id: chunk_id.to_string(),
            health_status: new_health_status,
            health_score: new_health_score,
            replica_results: check_results,
            issues_detected: chunk_health.risk_factors.clone(),
            recommendations: self.generate_recommendations(chunk_health),
        })
    }

    /// Check individual replica health
    async fn check_replica_health(&mut self, replica: &mut ReplicaHealth) -> Result<ReplicaCheckResult> {
        let start_time = std::time::Instant::now();

        // Simulate health check (in real implementation, this would be actual network calls)
        let connectivity_check = self.check_replica_connectivity(&replica.node_id).await?;
        let integrity_check = self.verify_replica_integrity(replica).await?;
        let performance_check = self.measure_replica_performance(replica).await?;

        let check_duration = start_time.elapsed();

        // Update replica status based on checks
        replica.status = if connectivity_check && integrity_check && performance_check.response_time_ms < 1000.0 {
            ReplicaStatus::Healthy
        } else if connectivity_check && integrity_check {
            ReplicaStatus::Slow
        } else if connectivity_check {
            ReplicaStatus::Degraded
        } else {
            ReplicaStatus::Unreachable
        };

        replica.last_verified = Utc::now();
        replica.performance_metrics = performance_check.clone();

        Ok(ReplicaCheckResult {
            replica_id: replica.replica_id.clone(),
            node_id: replica.node_id.clone(),
            status: replica.status.clone(),
            response_time_ms: performance_check.response_time_ms,
            connectivity_ok: connectivity_check,
            integrity_ok: integrity_check,
            performance_metrics: performance_check,
        })
    }

    /// Check replica connectivity
    async fn check_replica_connectivity(&self, node_id: &str) -> Result<bool> {
        // Simulate connectivity check
        tokio::time::sleep(TokioDuration::from_millis(10)).await;

        // Check if node is in our health records and recently seen
        if let Some(node_health) = self.node_health.get(node_id) {
            let time_since_last_seen = Utc::now() - node_health.last_seen;
            Ok(time_since_last_seen < Duration::minutes(5))
        } else {
            Ok(false)
        }
    }

    /// Verify replica integrity
    async fn verify_replica_integrity(&self, replica: &ReplicaHealth) -> Result<bool> {
        // Simulate integrity verification
        tokio::time::sleep(TokioDuration::from_millis(50)).await;

        // In real implementation, this would verify checksums, etc.
        Ok(!replica.integrity_hash.is_empty())
    }

    /// Measure replica performance
    async fn measure_replica_performance(&self, replica: &ReplicaHealth) -> Result<ReplicaPerformanceMetrics> {
        // Simulate performance measurement
        let base_latency = 100.0;
        let jitter = (rand::random::<f64>() - 0.5) * 50.0;
        let response_time = base_latency + jitter;

        tokio::time::sleep(TokioDuration::from_millis(response_time as u64)).await;

        Ok(ReplicaPerformanceMetrics {
            response_time_ms: response_time.max(0.0),
            transfer_speed_mbps: 50.0 + (rand::random::<f64>() * 50.0),
            success_rate: 95.0 + (rand::random::<f64>() * 5.0),
            error_count: 0,
            last_error: None,
            uptime_percentage: 99.0 + (rand::random::<f64>() * 1.0),
        })
    }

    /// Calculate overall chunk health score
    fn calculate_chunk_health_score(&self, replicas: &[ReplicaHealth]) -> f64 {
        if replicas.is_empty() {
            return 0.0;
        }

        let healthy_count = replicas.iter()
            .filter(|r| matches!(r.status, ReplicaStatus::Healthy))
            .count();

        let degraded_count = replicas.iter()
            .filter(|r| matches!(r.status, ReplicaStatus::Degraded | ReplicaStatus::Slow))
            .count();

        let unhealthy_count = replicas.len() - healthy_count - degraded_count;

        // Weight factors
        let healthy_weight = 1.0;
        let degraded_weight = 0.5;
        let unhealthy_weight = 0.0;

        let weighted_score = (healthy_count as f64 * healthy_weight
            + degraded_count as f64 * degraded_weight
            + unhealthy_count as f64 * unhealthy_weight) / replicas.len() as f64;

        weighted_score * 100.0
    }

    /// Determine health status from score and replica states
    fn determine_health_status(&self, health_score: f64, replicas: &[ReplicaHealth]) -> HealthStatus {
        let healthy_count = replicas.iter()
            .filter(|r| matches!(r.status, ReplicaStatus::Healthy))
            .count();

        let total_count = replicas.len();

        if health_score >= 90.0 && healthy_count >= (total_count * 3) / 4 {
            HealthStatus::Excellent
        } else if health_score >= 75.0 && healthy_count >= total_count / 2 {
            HealthStatus::Good
        } else if health_score >= 50.0 && healthy_count >= total_count / 3 {
            HealthStatus::Warning
        } else if healthy_count > 0 {
            HealthStatus::Critical
        } else {
            HealthStatus::Failed
        }
    }

    /// Get check interval for health status
    fn get_check_interval(&self, status: &HealthStatus) -> Duration {
        self.check_scheduler.check_intervals
            .get(status)
            .copied()
            .unwrap_or(Duration::hours(6))
    }

    /// Schedule health check
    fn schedule_health_check(&mut self, chunk_id: String, check_time: DateTime<Utc>) {
        self.check_scheduler.priority_queue
            .entry(check_time)
            .or_insert_with(Vec::new)
            .push(chunk_id);
    }

    /// Assess risk factors for a chunk
    fn assess_risk_factors(&self, chunk_health: &ChunkHealth) -> Vec<RiskFactor> {
        let mut risk_factors = Vec::new();

        // Check replica count
        let healthy_replicas = chunk_health.replica_health.iter()
            .filter(|r| matches!(r.status, ReplicaStatus::Healthy))
            .count();

        if healthy_replicas < 3 {
            risk_factors.push(RiskFactor {
                risk_type: RiskType::NodeFailure,
                severity: if healthy_replicas < 2 { RiskSeverity::Critical } else { RiskSeverity::High },
                probability: 0.8,
                impact: 0.9,
                description: format!("Only {} healthy replicas remaining", healthy_replicas),
                mitigation_actions: vec!["Increase replication".to_string(), "Replace unhealthy replicas".to_string()],
            });
        }

        // Check geographic distribution
        let regions: std::collections::HashSet<_> = chunk_health.replica_health.iter()
            .map(|r| &r.region)
            .collect();

        if regions.len() < 2 {
            risk_factors.push(RiskFactor {
                risk_type: RiskType::GeographicRisk,
                severity: RiskSeverity::Medium,
                probability: 0.3,
                impact: 0.7,
                description: "Poor geographic distribution".to_string(),
                mitigation_actions: vec!["Add replicas in different regions".to_string()],
            });
        }

        // Check performance degradation
        if chunk_health.performance_metrics.avg_response_time_ms > 1000.0 {
            risk_factors.push(RiskFactor {
                risk_type: RiskType::PerformanceDegradation,
                severity: RiskSeverity::Medium,
                probability: 0.6,
                impact: 0.4,
                description: "High response times detected".to_string(),
                mitigation_actions: vec!["Optimize replica placement".to_string(), "Check network conditions".to_string()],
            });
        }

        risk_factors
    }

    /// Generate recommendations for chunk health improvement
    fn generate_recommendations(&self, chunk_health: &ChunkHealth) -> Vec<String> {
        let mut recommendations = Vec::new();

        match chunk_health.overall_health {
            HealthStatus::Failed => {
                recommendations.push("URGENT: Immediate recovery required - chunk data may be lost".to_string());
                recommendations.push("Attempt recovery from any available replicas".to_string());
                recommendations.push("Check backup systems".to_string());
            },
            HealthStatus::Critical => {
                recommendations.push("Create additional replicas immediately".to_string());
                recommendations.push("Repair or replace unhealthy replicas".to_string());
                recommendations.push("Monitor closely for further degradation".to_string());
            },
            HealthStatus::Warning => {
                recommendations.push("Consider increasing replication factor".to_string());
                recommendations.push("Investigate cause of replica degradation".to_string());
                recommendations.push("Improve geographic distribution".to_string());
            },
            HealthStatus::Good => {
                recommendations.push("Monitor performance trends".to_string());
                recommendations.push("Consider optimizing replica placement for better performance".to_string());
            },
            HealthStatus::Excellent => {
                recommendations.push("Maintain current configuration".to_string());
                recommendations.push("Consider this as a model for other chunks".to_string());
            },
        }

        recommendations
    }

    /// Record health snapshot in history
    fn record_health_snapshot(&mut self, chunk_id: &str, chunk_health: &ChunkHealth) {
        let snapshot = HealthSnapshot {
            timestamp: Utc::now(),
            health_status: chunk_health.overall_health.clone(),
            health_score: chunk_health.availability_score,
            replica_count: chunk_health.replica_health.len() as u32,
            healthy_replicas: chunk_health.replica_health.iter()
                .filter(|r| matches!(r.status, ReplicaStatus::Healthy))
                .count() as u32,
            performance_metrics: chunk_health.performance_metrics.clone(),
        };

        if let Some(history) = self.health_history.get_mut(chunk_id) {
            history.push_back(snapshot);

            // Keep only last 1000 snapshots
            if history.len() > 1000 {
                history.pop_front();
            }
        }
    }

    /// Check alert conditions and send notifications
    async fn check_alert_conditions(&mut self, chunk_id: &str, chunk_health: &ChunkHealth) -> Result<()> {
        let thresholds = &self.alert_config.health_thresholds;

        let should_alert = match chunk_health.overall_health {
            HealthStatus::Failed | HealthStatus::Critical => true,
            HealthStatus::Warning => chunk_health.availability_score < thresholds.warning_health_score,
            _ => false,
        };

        if should_alert {
            let alert = HealthAlert {
                alert_id: format!("alert_{}_{}", chunk_id, Utc::now().timestamp()),
                chunk_id: chunk_id.to_string(),
                severity: match chunk_health.overall_health {
                    HealthStatus::Failed => RiskSeverity::Critical,
                    HealthStatus::Critical => RiskSeverity::High,
                    HealthStatus::Warning => RiskSeverity::Medium,
                    _ => RiskSeverity::Low,
                },
                message: format!("Chunk {} health degraded to {:?}", chunk_id, chunk_health.overall_health),
                timestamp: Utc::now(),
                health_score: chunk_health.availability_score,
                recommendations: self.generate_recommendations(chunk_health),
            };

            self.send_alert(alert).await?;
        }

        Ok(())
    }

    /// Send health alert
    async fn send_alert(&self, alert: HealthAlert) -> Result<()> {
        // In real implementation, this would send to configured notification channels
        tracing::warn!("Health Alert: {} - {} (Score: {:.1})",
            alert.alert_id, alert.message, alert.health_score);

        for recommendation in &alert.recommendations {
            tracing::info!("Recommendation: {}", recommendation);
        }

        Ok(())
    }

    /// Update monitoring metrics
    fn update_monitoring_metrics(&mut self) {
        let total_chunks = self.chunk_health.len() as u64;
        let mut healthy = 0;
        let mut degraded = 0;
        let mut critical = 0;
        let mut failed = 0;
        let mut total_score = 0.0;
        let mut total_replicas = 0;
        let mut healthy_replicas = 0;

        for chunk_health in self.chunk_health.values() {
            match chunk_health.overall_health {
                HealthStatus::Excellent | HealthStatus::Good => healthy += 1,
                HealthStatus::Warning => degraded += 1,
                HealthStatus::Critical => critical += 1,
                HealthStatus::Failed => failed += 1,
            }

            total_score += chunk_health.availability_score;
            total_replicas += chunk_health.replica_health.len();
            healthy_replicas += chunk_health.replica_health.iter()
                .filter(|r| matches!(r.status, ReplicaStatus::Healthy))
                .count();
        }

        self.monitoring_metrics = MonitoringMetrics {
            total_chunks_monitored: total_chunks,
            healthy_chunks: healthy,
            degraded_chunks: degraded,
            critical_chunks: critical,
            failed_chunks: failed,
            total_replicas: total_replicas as u64,
            healthy_replicas: healthy_replicas as u64,
            degraded_replicas: (total_replicas - healthy_replicas) as u64,
            average_health_score: if total_chunks > 0 { total_score / total_chunks as f64 } else { 100.0 },
            monitoring_efficiency: 100.0, // Would be calculated based on check success rate
            last_updated: Utc::now(),
        };
    }

    /// Run continuous health monitoring
    pub async fn run_continuous_monitoring(&mut self) -> Result<()> {
        let mut check_interval = tokio::time::interval(TokioDuration::from_secs(60)); // Check every minute

        loop {
            check_interval.tick().await;

            // Process due health checks
            let now = Utc::now();
            let due_chunks: Vec<String> = self.check_scheduler.priority_queue
                .range(..=now)
                .flat_map(|(_, chunks)| chunks.iter().cloned())
                .collect();

            // Remove processed items from queue
            self.check_scheduler.priority_queue.retain(|time, _| *time > now);

            // Process health checks in batches
            for chunk_batch in due_chunks.chunks(self.check_scheduler.batch_size as usize) {
                let mut check_tasks = Vec::new();

                for chunk_id in chunk_batch {
                    if check_tasks.len() >= self.check_scheduler.concurrent_checks as usize {
                        // Wait for some tasks to complete
                        let _ = futures::future::join_all(check_tasks.drain(..)).await;
                    }

                    let chunk_id_clone = chunk_id.clone();
                    let task = async move {
                        // Would perform actual health check here
                        tokio::time::sleep(TokioDuration::from_millis(100)).await;
                        Ok(chunk_id_clone)
                    };

                    check_tasks.push(task);
                }

                // Wait for remaining tasks
                let results = futures::future::join_all(check_tasks).await;

                // Process results
                for result in results {
                    match result {
                        Ok(chunk_id) => {
                            if let Err(e) = self.perform_health_check(&chunk_id).await {
                                tracing::error!("Health check failed for chunk {}: {}", chunk_id, e);
                            }
                        },
                        Err(e) => {
                            tracing::error!("Health check task failed: {}", e);
                        }
                    }
                }
            }

            // Update analytics
            self.update_health_analytics();
        }
    }

    /// Update health analytics and trends
    fn update_health_analytics(&mut self) {
        // Analyze health trends for each chunk
        for (chunk_id, history) in &self.health_history {
            if history.len() >= 10 {
                let trend = self.analyze_health_trend(history);
                self.analytics.health_trends.insert(chunk_id.clone(), trend);
            }
        }

        // Detect failure patterns
        self.detect_failure_patterns();

        // Update performance baselines
        self.update_performance_baselines();
    }

    /// Analyze health trend for a chunk
    fn analyze_health_trend(&self, history: &VecDeque<HealthSnapshot>) -> HealthTrend {
        let recent_scores: Vec<f64> = history.iter()
            .rev()
            .take(10)
            .map(|snapshot| snapshot.health_score)
            .collect();

        let trend_direction = if recent_scores.len() >= 2 {
            let first_half_avg = recent_scores.iter().take(recent_scores.len() / 2).sum::<f64>() / (recent_scores.len() / 2) as f64;
            let second_half_avg = recent_scores.iter().skip(recent_scores.len() / 2).sum::<f64>() / (recent_scores.len() - recent_scores.len() / 2) as f64;

            let diff = second_half_avg - first_half_avg;
            if diff > 5.0 {
                TrendDirection::Improving
            } else if diff < -5.0 {
                TrendDirection::Degrading
            } else {
                TrendDirection::Stable
            }
        } else {
            TrendDirection::Stable
        };

        HealthTrend {
            chunk_id: String::new(), // Would be filled by caller
            trend_direction,
            trend_strength: 0.5, // Simplified calculation
            prediction_confidence: 0.7,
            time_to_critical: None, // Would calculate based on trend
        }
    }

    /// Detect common failure patterns
    fn detect_failure_patterns(&mut self) {
        // Simplified pattern detection
        // In real implementation, this would use more sophisticated ML algorithms
    }

    /// Update performance baselines
    fn update_performance_baselines(&mut self) {
        // Update baselines based on recent performance data
        // In real implementation, this would calculate statistical baselines
    }

    /// Get health summary
    pub fn get_health_summary(&self) -> HealthSummary {
        HealthSummary {
            overall_health: if self.monitoring_metrics.average_health_score >= 90.0 {
                HealthStatus::Excellent
            } else if self.monitoring_metrics.average_health_score >= 75.0 {
                HealthStatus::Good
            } else if self.monitoring_metrics.average_health_score >= 50.0 {
                HealthStatus::Warning
            } else {
                HealthStatus::Critical
            },
            metrics: self.monitoring_metrics.clone(),
            top_risks: self.get_top_risk_factors(),
            recommendations: self.get_global_recommendations(),
        }
    }

    /// Get top risk factors across all chunks
    fn get_top_risk_factors(&self) -> Vec<RiskFactor> {
        let mut all_risks: Vec<RiskFactor> = self.chunk_health
            .values()
            .flat_map(|chunk| chunk.risk_factors.iter().cloned())
            .collect();

        all_risks.sort_by(|a, b| {
            let score_a = a.probability * a.impact;
            let score_b = b.probability * b.impact;
            score_b.partial_cmp(&score_a).unwrap_or(std::cmp::Ordering::Equal)
        });

        all_risks.into_iter().take(10).collect()
    }

    /// Get global recommendations
    fn get_global_recommendations(&self) -> Vec<String> {
        let mut recommendations = Vec::new();

        if self.monitoring_metrics.critical_chunks > 0 {
            recommendations.push("URGENT: Address critical chunks immediately".to_string());
        }

        if self.monitoring_metrics.degraded_chunks > self.monitoring_metrics.total_chunks_monitored / 4 {
            recommendations.push("High number of degraded chunks - investigate infrastructure issues".to_string());
        }

        if self.monitoring_metrics.average_health_score < 80.0 {
            recommendations.push("Overall system health is below optimal - consider increasing redundancy".to_string());
        }

        recommendations
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckResult {
    pub chunk_id: String,
    pub health_status: HealthStatus,
    pub health_score: f64,
    pub replica_results: Vec<ReplicaCheckResult>,
    pub issues_detected: Vec<RiskFactor>,
    pub recommendations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplicaCheckResult {
    pub replica_id: String,
    pub node_id: String,
    pub status: ReplicaStatus,
    pub response_time_ms: f64,
    pub connectivity_ok: bool,
    pub integrity_ok: bool,
    pub performance_metrics: ReplicaPerformanceMetrics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthAlert {
    pub alert_id: String,
    pub chunk_id: String,
    pub severity: RiskSeverity,
    pub message: String,
    pub timestamp: DateTime<Utc>,
    pub health_score: f64,
    pub recommendations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthSummary {
    pub overall_health: HealthStatus,
    pub metrics: MonitoringMetrics,
    pub top_risks: Vec<RiskFactor>,
    pub recommendations: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_monitor_creation() {
        let monitor = ChunkHealthMonitor::new();
        assert!(monitor.chunk_health.is_empty());
        assert!(monitor.alert_config.enable_alerts);
        assert_eq!(monitor.monitoring_metrics.total_chunks_monitored, 0);
    }

    #[test]
    fn test_health_score_calculation() {
        let monitor = ChunkHealthMonitor::new();

        let replicas = vec![
            ReplicaHealth {
                replica_id: "replica1".to_string(),
                node_id: "node1".to_string(),
                region: GeographicRegion::NorthAmerica,
                status: ReplicaStatus::Healthy,
                health_score: 100.0,
                last_accessed: Utc::now(),
                last_verified: Utc::now(),
                integrity_hash: "hash1".to_string(),
                performance_metrics: ReplicaPerformanceMetrics {
                    response_time_ms: 100.0,
                    transfer_speed_mbps: 50.0,
                    success_rate: 99.0,
                    error_count: 0,
                    last_error: None,
                    uptime_percentage: 99.9,
                },
                connectivity_status: ConnectivityStatus::Online,
            },
            ReplicaHealth {
                replica_id: "replica2".to_string(),
                node_id: "node2".to_string(),
                region: GeographicRegion::Europe,
                status: ReplicaStatus::Degraded,
                health_score: 70.0,
                last_accessed: Utc::now(),
                last_verified: Utc::now(),
                integrity_hash: "hash2".to_string(),
                performance_metrics: ReplicaPerformanceMetrics {
                    response_time_ms: 300.0,
                    transfer_speed_mbps: 20.0,
                    success_rate: 95.0,
                    error_count: 2,
                    last_error: Some("Network timeout".to_string()),
                    uptime_percentage: 98.0,
                },
                connectivity_status: ConnectivityStatus::Intermittent,
            },
        ];

        let score = monitor.calculate_chunk_health_score(&replicas);
        assert!(score > 50.0 && score < 100.0); // Should be between 50-100 for mixed health
    }

    #[tokio::test]
    async fn test_health_check_workflow() {
        let mut monitor = ChunkHealthMonitor::new();

        let replicas = vec![
            ReplicaHealth {
                replica_id: "replica1".to_string(),
                node_id: "node1".to_string(),
                region: GeographicRegion::NorthAmerica,
                status: ReplicaStatus::Healthy,
                health_score: 100.0,
                last_accessed: Utc::now(),
                last_verified: Utc::now(),
                integrity_hash: "hash1".to_string(),
                performance_metrics: ReplicaPerformanceMetrics {
                    response_time_ms: 100.0,
                    transfer_speed_mbps: 50.0,
                    success_rate: 99.0,
                    error_count: 0,
                    last_error: None,
                    uptime_percentage: 99.9,
                },
                connectivity_status: ConnectivityStatus::Online,
            },
        ];

        monitor.add_chunk_monitoring("test_chunk".to_string(), replicas);
        assert_eq!(monitor.chunk_health.len(), 1);

        // Perform health check
        let result = monitor.perform_health_check("test_chunk").await.unwrap();
        assert_eq!(result.chunk_id, "test_chunk");
        assert!(!result.replica_results.is_empty());
    }
}