//! Predictive Replication Module
//!
//! Machine learning-based node failure prediction and proactive data migration

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::time::{Duration, Instant};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeMetrics {
    pub node_id: String,
    pub uptime_percentage: f32,
    pub response_latency: Duration,
    pub storage_usage: f32,
    pub bandwidth_utilization: f32,
    pub error_rate: f32,
    pub last_failure: Option<Instant>,
    pub hardware_health: HardwareHealth,
    pub geographic_risk: GeographicRisk,
    pub network_stability: NetworkStability,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareHealth {
    pub cpu_temperature: f32,
    pub disk_health_score: f32,
    pub memory_errors: u32,
    pub power_stability: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeographicRisk {
    pub natural_disaster_risk: f32,
    pub political_stability: f32,
    pub infrastructure_quality: f32,
    pub connectivity_redundancy: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkStability {
    pub connection_drops: u32,
    pub peer_count: u32,
    pub routing_efficiency: f32,
    pub congestion_level: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailurePrediction {
    pub node_id: String,
    pub failure_probability: f32,
    pub predicted_failure_time: Option<Instant>,
    pub confidence_score: f32,
    pub risk_factors: Vec<RiskFactor>,
    pub recommended_actions: Vec<RecommendedAction>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RiskFactor {
    HighLatency,
    FrequentDisconnections,
    StorageExhaustion,
    HardwareDeterioration,
    NetworkCongestion,
    GeographicInstability,
    PerformanceDegradation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecommendedAction {
    MigrateChunksImmediately,
    IncreaseRedundancy,
    ScheduleMaintenance,
    ReduceLoad,
    MonitorClosely,
    PrepareFailover,
}

pub struct MLPredictor {
    node_history: HashMap<String, Vec<NodeMetrics>>,
    prediction_models: HashMap<String, PredictionModel>,
    feature_weights: FeatureWeights,
    training_data: Vec<TrainingExample>,
}

#[derive(Debug, Clone)]
struct PredictionModel {
    weights: Vec<f32>,
    bias: f32,
    accuracy: f32,
    last_updated: Instant,
}

#[derive(Debug, Clone)]
struct FeatureWeights {
    uptime: f32,
    latency: f32,
    storage: f32,
    bandwidth: f32,
    error_rate: f32,
    hardware_health: f32,
    geographic_risk: f32,
    network_stability: f32,
}

#[derive(Debug, Clone)]
struct TrainingExample {
    features: Vec<f32>,
    outcome: bool, // true if node failed
    timestamp: Instant,
}

impl MLPredictor {
    pub fn new() -> Self {
        Self {
            node_history: HashMap::new(),
            prediction_models: HashMap::new(),
            feature_weights: FeatureWeights::default(),
            training_data: Vec::new(),
        }
    }

    pub async fn update_node_metrics(&mut self, metrics: NodeMetrics) {
        let node_id = metrics.node_id.clone();
        let history = self.node_history.entry(node_id.clone()).or_insert_with(Vec::new);

        history.push(metrics.clone());

        // Keep only last 1000 data points per node
        if history.len() > 1000 {
            history.drain(0..history.len() - 1000);
        }

        // Update training data based on actual failures
        if let Some(last_metrics) = history.get(history.len().saturating_sub(2)) {
            if self.detect_failure_transition(last_metrics, &metrics) {
                let features = self.extract_features(last_metrics);
                self.training_data.push(TrainingExample {
                    features,
                    outcome: true,
                    timestamp: Instant::now(),
                });
            }
        }

        // Retrain model periodically
        if history.len() % 100 == 0 {
            self.retrain_model(&node_id).await;
        }
    }

    pub async fn predict_node_failure(&self, node_id: &str) -> Option<FailurePrediction> {
        let history = self.node_history.get(node_id)?;
        let latest_metrics = history.last()?;
        let model = self.prediction_models.get(node_id)?;

        let features = self.extract_features(latest_metrics);
        let failure_probability = self.calculate_failure_probability(&features, model);

        if failure_probability < 0.1 {
            return None; // Low risk, no prediction needed
        }

        let confidence_score = self.calculate_confidence(&features, model);
        let risk_factors = self.identify_risk_factors(latest_metrics);
        let recommended_actions = self.generate_recommendations(failure_probability, &risk_factors);

        let predicted_failure_time = if failure_probability > 0.7 {
            Some(Instant::now() + Duration::from_secs(3600)) // 1 hour
        } else if failure_probability > 0.5 {
            Some(Instant::now() + Duration::from_secs(7200)) // 2 hours
        } else {
            Some(Instant::now() + Duration::from_secs(14400)) // 4 hours
        };

        Some(FailurePrediction {
            node_id: node_id.to_string(),
            failure_probability,
            predicted_failure_time,
            confidence_score,
            risk_factors,
            recommended_actions,
        })
    }

    pub async fn get_high_risk_nodes(&self) -> Vec<FailurePrediction> {
        let mut high_risk = Vec::new();

        for node_id in self.node_history.keys() {
            if let Some(prediction) = self.predict_node_failure(node_id).await {
                if prediction.failure_probability > 0.3 {
                    high_risk.push(prediction);
                }
            }
        }

        // Sort by failure probability (highest first)
        high_risk.sort_by(|a, b| b.failure_probability.partial_cmp(&a.failure_probability).unwrap());
        high_risk
    }

    fn extract_features(&self, metrics: &NodeMetrics) -> Vec<f32> {
        vec![
            metrics.uptime_percentage,
            metrics.response_latency.as_millis() as f32,
            metrics.storage_usage,
            metrics.bandwidth_utilization,
            metrics.error_rate,
            self.calculate_hardware_score(&metrics.hardware_health),
            self.calculate_geographic_risk_score(&metrics.geographic_risk),
            self.calculate_network_stability_score(&metrics.network_stability),
        ]
    }

    fn calculate_failure_probability(&self, features: &[f32], model: &PredictionModel) -> f32 {
        let mut score = model.bias;
        for (feature, weight) in features.iter().zip(model.weights.iter()) {
            score += feature * weight;
        }

        // Sigmoid activation
        1.0 / (1.0 + (-score).exp())
    }

    fn calculate_confidence(&self, features: &[f32], model: &PredictionModel) -> f32 {
        // Confidence based on feature consistency and model accuracy
        let feature_variance = self.calculate_feature_variance(features);
        let base_confidence = model.accuracy;

        // Higher variance reduces confidence
        base_confidence * (1.0 - feature_variance.min(0.5))
    }

    fn calculate_feature_variance(&self, features: &[f32]) -> f32 {
        if features.is_empty() {
            return 0.0;
        }

        let mean: f32 = features.iter().sum::<f32>() / features.len() as f32;
        let variance: f32 = features.iter()
            .map(|x| (x - mean).powi(2))
            .sum::<f32>() / features.len() as f32;

        variance.sqrt()
    }

    fn identify_risk_factors(&self, metrics: &NodeMetrics) -> Vec<RiskFactor> {
        let mut factors = Vec::new();

        if metrics.response_latency > Duration::from_millis(1000) {
            factors.push(RiskFactor::HighLatency);
        }

        if metrics.error_rate > 0.05 {
            factors.push(RiskFactor::FrequentDisconnections);
        }

        if metrics.storage_usage > 0.9 {
            factors.push(RiskFactor::StorageExhaustion);
        }

        if metrics.hardware_health.disk_health_score < 0.7
            || metrics.hardware_health.cpu_temperature > 80.0 {
            factors.push(RiskFactor::HardwareDeterioration);
        }

        if metrics.network_stability.congestion_level > 0.8 {
            factors.push(RiskFactor::NetworkCongestion);
        }

        if metrics.geographic_risk.natural_disaster_risk > 0.6
            || metrics.geographic_risk.political_stability < 0.4 {
            factors.push(RiskFactor::GeographicInstability);
        }

        if metrics.uptime_percentage < 0.95 && metrics.bandwidth_utilization < 0.3 {
            factors.push(RiskFactor::PerformanceDegradation);
        }

        factors
    }

    fn generate_recommendations(
        &self,
        failure_probability: f32,
        risk_factors: &[RiskFactor]
    ) -> Vec<RecommendedAction> {
        let mut actions = Vec::new();

        if failure_probability > 0.8 {
            actions.push(RecommendedAction::MigrateChunksImmediately);
            actions.push(RecommendedAction::PrepareFailover);
        } else if failure_probability > 0.6 {
            actions.push(RecommendedAction::IncreaseRedundancy);
            actions.push(RecommendedAction::MonitorClosely);
        } else if failure_probability > 0.4 {
            actions.push(RecommendedAction::ScheduleMaintenance);
        }

        for factor in risk_factors {
            match factor {
                RiskFactor::StorageExhaustion => {
                    actions.push(RecommendedAction::ReduceLoad);
                }
                RiskFactor::HardwareDeterioration => {
                    actions.push(RecommendedAction::ScheduleMaintenance);
                }
                RiskFactor::NetworkCongestion => {
                    actions.push(RecommendedAction::ReduceLoad);
                }
                _ => {}
            }
        }

        actions.sort();
        actions.dedup();
        actions
    }

    async fn retrain_model(&mut self, node_id: &str) {
        if let Some(history) = self.node_history.get(node_id) {
            if history.len() < 50 {
                return; // Need more data
            }

            let mut model = PredictionModel {
                weights: vec![0.1; 8], // Initialize with small weights
                bias: 0.0,
                accuracy: 0.5,
                last_updated: Instant::now(),
            };

            // Simple gradient descent training
            let learning_rate = 0.01;
            let epochs = 100;

            for _ in 0..epochs {
                for example in &self.training_data {
                    if example.features.len() == 8 {
                        let prediction = self.calculate_failure_probability(&example.features, &model);
                        let target = if example.outcome { 1.0 } else { 0.0 };
                        let error = prediction - target;

                        // Update weights
                        for (i, feature) in example.features.iter().enumerate() {
                            model.weights[i] -= learning_rate * error * feature;
                        }
                        model.bias -= learning_rate * error;
                    }
                }
            }

            // Calculate accuracy on validation set
            let mut correct = 0;
            let mut total = 0;
            for example in &self.training_data {
                if example.features.len() == 8 {
                    let prediction = self.calculate_failure_probability(&example.features, &model);
                    let predicted_outcome = prediction > 0.5;
                    if predicted_outcome == example.outcome {
                        correct += 1;
                    }
                    total += 1;
                }
            }

            if total > 0 {
                model.accuracy = correct as f32 / total as f32;
            }

            self.prediction_models.insert(node_id.to_string(), model);
        }
    }

    fn detect_failure_transition(&self, previous: &NodeMetrics, current: &NodeMetrics) -> bool {
        // Detect if node has failed based on metrics change
        let uptime_drop = previous.uptime_percentage - current.uptime_percentage;
        let latency_spike = current.response_latency.as_millis() as f32 /
                           previous.response_latency.as_millis() as f32;
        let error_increase = current.error_rate / previous.error_rate.max(0.001);

        uptime_drop > 0.2 || latency_spike > 2.0 || error_increase > 3.0
    }

    fn calculate_hardware_score(&self, health: &HardwareHealth) -> f32 {
        let temp_score = if health.cpu_temperature > 90.0 { 0.0 }
                        else if health.cpu_temperature > 80.0 { 0.3 }
                        else if health.cpu_temperature > 70.0 { 0.7 }
                        else { 1.0 };

        let disk_score = health.disk_health_score;
        let memory_score = if health.memory_errors > 10 { 0.2 }
                          else if health.memory_errors > 5 { 0.6 }
                          else { 1.0 };
        let power_score = health.power_stability;

        (temp_score + disk_score + memory_score + power_score) / 4.0
    }

    fn calculate_geographic_risk_score(&self, risk: &GeographicRisk) -> f32 {
        let disaster_score = 1.0 - risk.natural_disaster_risk;
        let political_score = risk.political_stability;
        let infrastructure_score = risk.infrastructure_quality;
        let connectivity_score = risk.connectivity_redundancy;

        (disaster_score + political_score + infrastructure_score + connectivity_score) / 4.0
    }

    fn calculate_network_stability_score(&self, stability: &NetworkStability) -> f32 {
        let connection_score = if stability.connection_drops > 10 { 0.2 }
                              else if stability.connection_drops > 5 { 0.6 }
                              else { 1.0 };

        let peer_score = if stability.peer_count < 3 { 0.3 }
                        else if stability.peer_count < 8 { 0.7 }
                        else { 1.0 };

        let routing_score = stability.routing_efficiency;
        let congestion_score = 1.0 - stability.congestion_level;

        (connection_score + peer_score + routing_score + congestion_score) / 4.0
    }
}

impl Default for FeatureWeights {
    fn default() -> Self {
        Self {
            uptime: 0.25,
            latency: 0.20,
            storage: 0.15,
            bandwidth: 0.10,
            error_rate: 0.15,
            hardware_health: 0.05,
            geographic_risk: 0.05,
            network_stability: 0.05,
        }
    }
}

pub struct ProactiveReplicationManager {
    predictor: MLPredictor,
    migration_scheduler: MigrationScheduler,
    chunk_priority_queue: Vec<ChunkMigrationTask>,
}

#[derive(Debug, Clone)]
struct ChunkMigrationTask {
    chunk_id: String,
    source_nodes: Vec<String>,
    target_nodes: Vec<String>,
    priority: u8, // 1-10, higher is more urgent
    deadline: Instant,
    estimated_transfer_time: Duration,
}

struct MigrationScheduler {
    active_migrations: HashMap<String, MigrationProgress>,
    bandwidth_budget: BandwidthBudget,
    node_capacities: HashMap<String, NodeCapacity>,
}

#[derive(Debug, Clone)]
struct MigrationProgress {
    task: ChunkMigrationTask,
    bytes_transferred: u64,
    total_bytes: u64,
    start_time: Instant,
    estimated_completion: Instant,
}

#[derive(Debug, Clone)]
struct BandwidthBudget {
    total_available: u64, // bytes per second
    reserved_for_users: u64,
    available_for_migration: u64,
    current_usage: u64,
}

#[derive(Debug, Clone)]
struct NodeCapacity {
    storage_available: u64,
    bandwidth_capacity: u64,
    current_load: f32,
    reliability_score: f32,
}

impl ProactiveReplicationManager {
    pub fn new() -> Self {
        Self {
            predictor: MLPredictor::new(),
            migration_scheduler: MigrationScheduler::new(),
            chunk_priority_queue: Vec::new(),
        }
    }

    pub async fn analyze_and_migrate(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Get high-risk nodes
        let high_risk_nodes = self.predictor.get_high_risk_nodes().await;

        for prediction in high_risk_nodes {
            if prediction.failure_probability > 0.5 {
                self.schedule_emergency_migration(&prediction.node_id, prediction.failure_probability).await?;
            } else if prediction.failure_probability > 0.3 {
                self.schedule_preemptive_migration(&prediction.node_id, prediction.failure_probability).await?;
            }
        }

        // Execute scheduled migrations
        self.execute_migration_queue().await?;

        Ok(())
    }

    async fn schedule_emergency_migration(&mut self, node_id: &str, risk: f32) -> Result<(), Box<dyn std::error::Error>> {
        let chunks = self.get_chunks_on_node(node_id).await?;

        for chunk_id in chunks {
            let task = ChunkMigrationTask {
                chunk_id,
                source_nodes: vec![node_id.to_string()],
                target_nodes: self.select_migration_targets(2, Some(node_id)).await?,
                priority: 10, // Highest priority
                deadline: Instant::now() + Duration::from_secs(1800), // 30 minutes
                estimated_transfer_time: Duration::from_secs(300), // 5 minutes estimate
            };

            self.chunk_priority_queue.push(task);
        }

        // Sort by priority and deadline
        self.chunk_priority_queue.sort_by(|a, b| {
            b.priority.cmp(&a.priority)
                .then_with(|| a.deadline.cmp(&b.deadline))
        });

        Ok(())
    }

    async fn schedule_preemptive_migration(&mut self, node_id: &str, risk: f32) -> Result<(), Box<dyn std::error::Error>> {
        let chunks = self.get_chunks_on_node(node_id).await?;
        let priority = ((risk - 0.3) * 20.0) as u8 + 3; // Priority 3-7

        for chunk_id in chunks {
            let task = ChunkMigrationTask {
                chunk_id,
                source_nodes: vec![node_id.to_string()],
                target_nodes: self.select_migration_targets(1, Some(node_id)).await?,
                priority,
                deadline: Instant::now() + Duration::from_secs(7200), // 2 hours
                estimated_transfer_time: Duration::from_secs(600), // 10 minutes estimate
            };

            self.chunk_priority_queue.push(task);
        }

        self.chunk_priority_queue.sort_by(|a, b| {
            b.priority.cmp(&a.priority)
                .then_with(|| a.deadline.cmp(&b.deadline))
        });

        Ok(())
    }

    async fn execute_migration_queue(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let available_bandwidth = self.migration_scheduler.bandwidth_budget.available_for_migration;
        let mut current_bandwidth_usage = 0u64;

        while let Some(task) = self.chunk_priority_queue.pop() {
            if current_bandwidth_usage + self.estimate_bandwidth_usage(&task) > available_bandwidth {
                // Put task back and wait for next cycle
                self.chunk_priority_queue.push(task);
                break;
            }

            if self.can_start_migration(&task).await {
                self.start_migration(task).await?;
                current_bandwidth_usage += self.estimate_bandwidth_usage(&self.chunk_priority_queue.last().unwrap());
            }
        }

        Ok(())
    }

    async fn get_chunks_on_node(&self, _node_id: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        // Placeholder: In reality, this would query the chunk index
        Ok(vec!["chunk_1".to_string(), "chunk_2".to_string()])
    }

    async fn select_migration_targets(&self, count: usize, avoid_node: Option<&str>) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let mut candidates: Vec<_> = self.migration_scheduler.node_capacities.iter()
            .filter(|(node_id, capacity)| {
                if let Some(avoid) = avoid_node {
                    *node_id != avoid && capacity.current_load < 0.8 && capacity.reliability_score > 0.7
                } else {
                    capacity.current_load < 0.8 && capacity.reliability_score > 0.7
                }
            })
            .collect();

        candidates.sort_by(|a, b| b.1.reliability_score.partial_cmp(&a.1.reliability_score).unwrap());

        Ok(candidates.into_iter()
            .take(count)
            .map(|(node_id, _)| node_id.clone())
            .collect())
    }

    async fn can_start_migration(&self, task: &ChunkMigrationTask) -> bool {
        // Check if target nodes have capacity
        for target_node in &task.target_nodes {
            if let Some(capacity) = self.migration_scheduler.node_capacities.get(target_node) {
                if capacity.current_load > 0.85 {
                    return false;
                }
            }
        }

        // Check if we're not already migrating this chunk
        !self.migration_scheduler.active_migrations.contains_key(&task.chunk_id)
    }

    async fn start_migration(&mut self, task: ChunkMigrationTask) -> Result<(), Box<dyn std::error::Error>> {
        let progress = MigrationProgress {
            task: task.clone(),
            bytes_transferred: 0,
            total_bytes: 1024 * 1024, // 1MB estimate
            start_time: Instant::now(),
            estimated_completion: Instant::now() + task.estimated_transfer_time,
        };

        self.migration_scheduler.active_migrations.insert(task.chunk_id, progress);

        // Placeholder: In reality, this would initiate the actual transfer
        println!("Starting migration of chunk {} from {:?} to {:?}",
                task.chunk_id, task.source_nodes, task.target_nodes);

        Ok(())
    }

    fn estimate_bandwidth_usage(&self, task: &ChunkMigrationTask) -> u64 {
        // Estimate based on chunk size and transfer time
        let chunk_size = 1024 * 1024; // 1MB estimate
        let transfer_duration = task.estimated_transfer_time.as_secs().max(1);
        chunk_size / transfer_duration
    }
}

impl MigrationScheduler {
    fn new() -> Self {
        Self {
            active_migrations: HashMap::new(),
            bandwidth_budget: BandwidthBudget {
                total_available: 100 * 1024 * 1024, // 100 MB/s
                reserved_for_users: 70 * 1024 * 1024, // 70 MB/s for users
                available_for_migration: 30 * 1024 * 1024, // 30 MB/s for migration
                current_usage: 0,
            },
            node_capacities: HashMap::new(),
        }
    }
}