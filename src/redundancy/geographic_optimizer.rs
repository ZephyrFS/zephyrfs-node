//! Geographic Distribution Optimization
//!
//! Advanced geographic distribution system that optimizes data placement
//! for latency, durability, and regulatory compliance

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, BTreeMap};
use chrono::{DateTime, Utc, Duration};

use crate::economics::earnings_calculator::GeographicRegion;

/// Geographic distribution optimizer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeographicOptimizer {
    /// Regional infrastructure mapping
    pub region_info: HashMap<GeographicRegion, RegionInfo>,
    /// Latency matrix between regions
    pub latency_matrix: LatencyMatrix,
    /// Regulatory compliance requirements
    pub compliance_rules: HashMap<String, ComplianceRule>,
    /// Cost optimization settings
    pub cost_optimization: CostOptimizationSettings,
    /// Performance analytics
    pub performance_analytics: PerformanceAnalytics,
    /// Real-time network conditions
    pub network_conditions: HashMap<GeographicRegion, NetworkCondition>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegionInfo {
    pub region: GeographicRegion,
    pub name: String,
    pub country_codes: Vec<String>,
    pub timezone_offset: i32,
    pub data_sovereignty_laws: Vec<String>,
    pub infrastructure_quality: InfrastructureQuality,
    pub cost_factors: CostFactors,
    pub capacity_info: RegionalCapacity,
    pub disaster_risk: DisasterRisk,
    pub political_stability: f64, // 0.0 to 1.0
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InfrastructureQuality {
    pub internet_penetration: f64,
    pub average_bandwidth_mbps: f64,
    pub fiber_coverage: f64,
    pub power_reliability: f64,
    pub data_center_density: u32,
    pub submarine_cable_connections: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostFactors {
    pub electricity_cost_per_kwh: f64,
    pub data_center_costs: f64,
    pub labor_costs: f64,
    pub regulatory_overhead: f64,
    pub tax_rates: f64,
    pub currency_stability: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegionalCapacity {
    pub total_nodes: u32,
    pub active_nodes: u32,
    pub total_storage_gb: u64,
    pub available_storage_gb: u64,
    pub utilization_rate: f64,
    pub growth_rate_monthly: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisasterRisk {
    pub natural_disasters: f64,     // 0.0 to 1.0
    pub political_instability: f64, // 0.0 to 1.0
    pub cyber_security_threats: f64, // 0.0 to 1.0
    pub infrastructure_fragility: f64, // 0.0 to 1.0
    pub overall_risk_score: f64,    // Composite score
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatencyMatrix {
    /// Latency measurements between regions (in milliseconds)
    pub measurements: HashMap<(GeographicRegion, GeographicRegion), LatencyMeasurement>,
    /// Last update timestamp
    pub last_updated: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatencyMeasurement {
    pub avg_latency_ms: f64,
    pub min_latency_ms: f64,
    pub max_latency_ms: f64,
    pub jitter_ms: f64,
    pub packet_loss: f64,
    pub bandwidth_mbps: f64,
    pub measurement_count: u32,
    pub last_measured: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceRule {
    pub rule_id: String,
    pub name: String,
    pub description: String,
    pub applicable_regions: Vec<GeographicRegion>,
    pub data_residency_required: bool,
    pub cross_border_restrictions: Vec<GeographicRegion>,
    pub encryption_requirements: Vec<String>,
    pub audit_requirements: Vec<String>,
    pub data_retention_days: Option<u32>,
    pub severity: ComplianceSeverity,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComplianceSeverity {
    Critical,    // Must comply
    Important,   // Should comply
    Recommended, // Nice to comply
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostOptimizationSettings {
    pub enable_cost_optimization: bool,
    pub cost_weight: f64,
    pub latency_weight: f64,
    pub reliability_weight: f64,
    pub compliance_weight: f64,
    pub preferred_cost_regions: Vec<GeographicRegion>,
    pub max_cost_difference_percent: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceAnalytics {
    pub regional_performance: HashMap<GeographicRegion, RegionalPerformance>,
    pub cross_region_performance: HashMap<(GeographicRegion, GeographicRegion), CrossRegionPerformance>,
    pub optimization_history: Vec<OptimizationEvent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegionalPerformance {
    pub region: GeographicRegion,
    pub avg_response_time_ms: f64,
    pub success_rate: f64,
    pub throughput_mbps: f64,
    pub availability: f64,
    pub cost_per_gb: f64,
    pub user_satisfaction: f64,
    pub last_updated: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossRegionPerformance {
    pub source_region: GeographicRegion,
    pub target_region: GeographicRegion,
    pub transfer_speed_mbps: f64,
    pub latency_ms: f64,
    pub reliability: f64,
    pub cost_per_gb_transfer: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationEvent {
    pub timestamp: DateTime<Utc>,
    pub event_type: OptimizationEventType,
    pub affected_regions: Vec<GeographicRegion>,
    pub improvement_metrics: HashMap<String, f64>,
    pub cost_impact: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OptimizationEventType {
    RegionalRebalancing,
    LatencyOptimization,
    CostOptimization,
    ComplianceAdjustment,
    DisasterResponse,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkCondition {
    pub region: GeographicRegion,
    pub current_load: f64,           // 0.0 to 1.0
    pub available_bandwidth: f64,    // Mbps
    pub congestion_level: CongestionLevel,
    pub incident_count: u32,
    pub predicted_performance: f64,  // 0.0 to 1.0
    pub last_updated: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CongestionLevel {
    Minimal,
    Light,
    Moderate,
    Heavy,
    Severe,
}

impl GeographicOptimizer {
    /// Create new geographic optimizer
    pub fn new() -> Self {
        let mut optimizer = Self {
            region_info: HashMap::new(),
            latency_matrix: LatencyMatrix {
                measurements: HashMap::new(),
                last_updated: Utc::now(),
            },
            compliance_rules: HashMap::new(),
            cost_optimization: CostOptimizationSettings {
                enable_cost_optimization: true,
                cost_weight: 0.3,
                latency_weight: 0.4,
                reliability_weight: 0.2,
                compliance_weight: 0.1,
                preferred_cost_regions: vec![
                    GeographicRegion::Asia,
                    GeographicRegion::SouthAmerica,
                ],
                max_cost_difference_percent: 50.0,
            },
            performance_analytics: PerformanceAnalytics {
                regional_performance: HashMap::new(),
                cross_region_performance: HashMap::new(),
                optimization_history: Vec::new(),
            },
            network_conditions: HashMap::new(),
        };

        optimizer.initialize_region_info();
        optimizer.initialize_compliance_rules();
        optimizer.initialize_latency_matrix();

        optimizer
    }

    /// Initialize region information
    fn initialize_region_info(&mut self) {
        // North America
        self.region_info.insert(GeographicRegion::NorthAmerica, RegionInfo {
            region: GeographicRegion::NorthAmerica,
            name: "North America".to_string(),
            country_codes: vec!["US".to_string(), "CA".to_string(), "MX".to_string()],
            timezone_offset: -5, // EST
            data_sovereignty_laws: vec!["CCPA".to_string(), "PIPEDA".to_string()],
            infrastructure_quality: InfrastructureQuality {
                internet_penetration: 95.0,
                average_bandwidth_mbps: 100.0,
                fiber_coverage: 80.0,
                power_reliability: 99.9,
                data_center_density: 150,
                submarine_cable_connections: 25,
            },
            cost_factors: CostFactors {
                electricity_cost_per_kwh: 0.12,
                data_center_costs: 1.0, // Baseline
                labor_costs: 1.0,
                regulatory_overhead: 0.3,
                tax_rates: 0.25,
                currency_stability: 0.95,
            },
            capacity_info: RegionalCapacity {
                total_nodes: 5000,
                active_nodes: 4800,
                total_storage_gb: 50_000_000,
                available_storage_gb: 15_000_000,
                utilization_rate: 70.0,
                growth_rate_monthly: 5.0,
            },
            disaster_risk: DisasterRisk {
                natural_disasters: 0.3,
                political_instability: 0.1,
                cyber_security_threats: 0.4,
                infrastructure_fragility: 0.2,
                overall_risk_score: 0.25,
            },
            political_stability: 0.85,
        });

        // Europe
        self.region_info.insert(GeographicRegion::Europe, RegionInfo {
            region: GeographicRegion::Europe,
            name: "Europe".to_string(),
            country_codes: vec!["DE".to_string(), "FR".to_string(), "GB".to_string(), "NL".to_string()],
            timezone_offset: 1, // CET
            data_sovereignty_laws: vec!["GDPR".to_string(), "DPA".to_string()],
            infrastructure_quality: InfrastructureQuality {
                internet_penetration: 92.0,
                average_bandwidth_mbps: 80.0,
                fiber_coverage: 75.0,
                power_reliability: 99.8,
                data_center_density: 120,
                submarine_cable_connections: 30,
            },
            cost_factors: CostFactors {
                electricity_cost_per_kwh: 0.20,
                data_center_costs: 1.2,
                labor_costs: 1.1,
                regulatory_overhead: 0.5,
                tax_rates: 0.30,
                currency_stability: 0.90,
            },
            capacity_info: RegionalCapacity {
                total_nodes: 4000,
                active_nodes: 3900,
                total_storage_gb: 40_000_000,
                available_storage_gb: 12_000_000,
                utilization_rate: 70.0,
                growth_rate_monthly: 4.0,
            },
            disaster_risk: DisasterRisk {
                natural_disasters: 0.2,
                political_instability: 0.2,
                cyber_security_threats: 0.3,
                infrastructure_fragility: 0.1,
                overall_risk_score: 0.20,
            },
            political_stability: 0.90,
        });

        // Asia
        self.region_info.insert(GeographicRegion::Asia, RegionInfo {
            region: GeographicRegion::Asia,
            name: "Asia Pacific".to_string(),
            country_codes: vec!["JP".to_string(), "SG".to_string(), "KR".to_string(), "AU".to_string()],
            timezone_offset: 9, // JST
            data_sovereignty_laws: vec!["PDPA".to_string(), "Privacy_Act".to_string()],
            infrastructure_quality: InfrastructureQuality {
                internet_penetration: 85.0,
                average_bandwidth_mbps: 120.0,
                fiber_coverage: 90.0,
                power_reliability: 99.5,
                data_center_density: 100,
                submarine_cable_connections: 40,
            },
            cost_factors: CostFactors {
                electricity_cost_per_kwh: 0.08,
                data_center_costs: 0.7,
                labor_costs: 0.6,
                regulatory_overhead: 0.2,
                tax_rates: 0.20,
                currency_stability: 0.85,
            },
            capacity_info: RegionalCapacity {
                total_nodes: 6000,
                active_nodes: 5500,
                total_storage_gb: 60_000_000,
                available_storage_gb: 20_000_000,
                utilization_rate: 67.0,
                growth_rate_monthly: 8.0,
            },
            disaster_risk: DisasterRisk {
                natural_disasters: 0.5,
                political_instability: 0.3,
                cyber_security_threats: 0.4,
                infrastructure_fragility: 0.3,
                overall_risk_score: 0.38,
            },
            political_stability: 0.80,
        });

        // Add other regions...
        self.initialize_remaining_regions();
    }

    /// Initialize remaining regions
    fn initialize_remaining_regions(&mut self) {
        // South America
        self.region_info.insert(GeographicRegion::SouthAmerica, RegionInfo {
            region: GeographicRegion::SouthAmerica,
            name: "South America".to_string(),
            country_codes: vec!["BR".to_string(), "AR".to_string(), "CL".to_string()],
            timezone_offset: -3,
            data_sovereignty_laws: vec!["LGPD".to_string()],
            infrastructure_quality: InfrastructureQuality {
                internet_penetration: 70.0,
                average_bandwidth_mbps: 40.0,
                fiber_coverage: 45.0,
                power_reliability: 97.0,
                data_center_density: 30,
                submarine_cable_connections: 8,
            },
            cost_factors: CostFactors {
                electricity_cost_per_kwh: 0.05,
                data_center_costs: 0.5,
                labor_costs: 0.4,
                regulatory_overhead: 0.15,
                tax_rates: 0.25,
                currency_stability: 0.70,
            },
            capacity_info: RegionalCapacity {
                total_nodes: 1000,
                active_nodes: 800,
                total_storage_gb: 8_000_000,
                available_storage_gb: 3_000_000,
                utilization_rate: 62.0,
                growth_rate_monthly: 12.0,
            },
            disaster_risk: DisasterRisk {
                natural_disasters: 0.4,
                political_instability: 0.4,
                cyber_security_threats: 0.3,
                infrastructure_fragility: 0.4,
                overall_risk_score: 0.38,
            },
            political_stability: 0.70,
        });

        // Africa
        self.region_info.insert(GeographicRegion::Africa, RegionInfo {
            region: GeographicRegion::Africa,
            name: "Africa".to_string(),
            country_codes: vec!["ZA".to_string(), "NG".to_string(), "EG".to_string()],
            timezone_offset: 2,
            data_sovereignty_laws: vec!["POPIA".to_string()],
            infrastructure_quality: InfrastructureQuality {
                internet_penetration: 50.0,
                average_bandwidth_mbps: 20.0,
                fiber_coverage: 25.0,
                power_reliability: 85.0,
                data_center_density: 15,
                submarine_cable_connections: 12,
            },
            cost_factors: CostFactors {
                electricity_cost_per_kwh: 0.08,
                data_center_costs: 0.6,
                labor_costs: 0.3,
                regulatory_overhead: 0.1,
                tax_rates: 0.15,
                currency_stability: 0.60,
            },
            capacity_info: RegionalCapacity {
                total_nodes: 200,
                active_nodes: 150,
                total_storage_gb: 1_500_000,
                available_storage_gb: 800_000,
                utilization_rate: 47.0,
                growth_rate_monthly: 15.0,
            },
            disaster_risk: DisasterRisk {
                natural_disasters: 0.3,
                political_instability: 0.6,
                cyber_security_threats: 0.5,
                infrastructure_fragility: 0.7,
                overall_risk_score: 0.53,
            },
            political_stability: 0.60,
        });

        // Oceania
        self.region_info.insert(GeographicRegion::Oceania, RegionInfo {
            region: GeographicRegion::Oceania,
            name: "Oceania".to_string(),
            country_codes: vec!["AU".to_string(), "NZ".to_string()],
            timezone_offset: 10,
            data_sovereignty_laws: vec!["Privacy_Act_AU".to_string(), "Privacy_Act_NZ".to_string()],
            infrastructure_quality: InfrastructureQuality {
                internet_penetration: 90.0,
                average_bandwidth_mbps: 60.0,
                fiber_coverage: 70.0,
                power_reliability: 99.0,
                data_center_density: 25,
                submarine_cable_connections: 15,
            },
            cost_factors: CostFactors {
                electricity_cost_per_kwh: 0.15,
                data_center_costs: 1.1,
                labor_costs: 1.2,
                regulatory_overhead: 0.2,
                tax_rates: 0.20,
                currency_stability: 0.85,
            },
            capacity_info: RegionalCapacity {
                total_nodes: 800,
                active_nodes: 750,
                total_storage_gb: 6_000_000,
                available_storage_gb: 2_000_000,
                utilization_rate: 67.0,
                growth_rate_monthly: 6.0,
            },
            disaster_risk: DisasterRisk {
                natural_disasters: 0.6,
                political_instability: 0.1,
                cyber_security_threats: 0.2,
                infrastructure_fragility: 0.2,
                overall_risk_score: 0.28,
            },
            political_stability: 0.95,
        });
    }

    /// Initialize compliance rules
    fn initialize_compliance_rules(&mut self) {
        // GDPR
        self.compliance_rules.insert("gdpr".to_string(), ComplianceRule {
            rule_id: "gdpr".to_string(),
            name: "General Data Protection Regulation".to_string(),
            description: "EU data protection regulation".to_string(),
            applicable_regions: vec![GeographicRegion::Europe],
            data_residency_required: true,
            cross_border_restrictions: vec![
                GeographicRegion::NorthAmerica, // Requires adequacy decision
            ],
            encryption_requirements: vec!["AES-256".to_string()],
            audit_requirements: vec!["regular_audit".to_string(), "breach_notification".to_string()],
            data_retention_days: Some(2555), // 7 years
            severity: ComplianceSeverity::Critical,
        });

        // CCPA
        self.compliance_rules.insert("ccpa".to_string(), ComplianceRule {
            rule_id: "ccpa".to_string(),
            name: "California Consumer Privacy Act".to_string(),
            description: "California privacy regulation".to_string(),
            applicable_regions: vec![GeographicRegion::NorthAmerica],
            data_residency_required: false,
            cross_border_restrictions: vec![],
            encryption_requirements: vec!["encryption_at_rest".to_string()],
            audit_requirements: vec!["privacy_audit".to_string()],
            data_retention_days: Some(1095), // 3 years
            severity: ComplianceSeverity::Important,
        });

        // Data Sovereignty (General)
        self.compliance_rules.insert("data_sovereignty".to_string(), ComplianceRule {
            rule_id: "data_sovereignty".to_string(),
            name: "Data Sovereignty Requirements".to_string(),
            description: "General data sovereignty compliance".to_string(),
            applicable_regions: vec![
                GeographicRegion::Europe,
                GeographicRegion::Asia,
                GeographicRegion::Oceania,
            ],
            data_residency_required: true,
            cross_border_restrictions: vec![],
            encryption_requirements: vec!["sovereign_encryption".to_string()],
            audit_requirements: vec!["sovereignty_audit".to_string()],
            data_retention_days: None,
            severity: ComplianceSeverity::Critical,
        });
    }

    /// Initialize latency matrix with baseline measurements
    fn initialize_latency_matrix(&mut self) {
        let regions = vec![
            GeographicRegion::NorthAmerica,
            GeographicRegion::Europe,
            GeographicRegion::Asia,
            GeographicRegion::SouthAmerica,
            GeographicRegion::Africa,
            GeographicRegion::Oceania,
        ];

        // Initialize with estimated latencies
        for (i, region_a) in regions.iter().enumerate() {
            for (j, region_b) in regions.iter().enumerate() {
                if i != j {
                    let estimated_latency = self.estimate_baseline_latency(region_a, region_b);

                    self.latency_matrix.measurements.insert(
                        (region_a.clone(), region_b.clone()),
                        LatencyMeasurement {
                            avg_latency_ms: estimated_latency,
                            min_latency_ms: estimated_latency * 0.8,
                            max_latency_ms: estimated_latency * 1.5,
                            jitter_ms: estimated_latency * 0.1,
                            packet_loss: 0.01,
                            bandwidth_mbps: 100.0,
                            measurement_count: 10,
                            last_measured: Utc::now(),
                        },
                    );
                }
            }
        }

        self.latency_matrix.last_updated = Utc::now();
    }

    /// Estimate baseline latency between regions
    fn estimate_baseline_latency(&self, region_a: &GeographicRegion, region_b: &GeographicRegion) -> f64 {
        use GeographicRegion::*;

        match (region_a, region_b) {
            // North America connections
            (NorthAmerica, Europe) | (Europe, NorthAmerica) => 120.0,
            (NorthAmerica, Asia) | (Asia, NorthAmerica) => 180.0,
            (NorthAmerica, SouthAmerica) | (SouthAmerica, NorthAmerica) => 160.0,
            (NorthAmerica, Africa) | (Africa, NorthAmerica) => 200.0,
            (NorthAmerica, Oceania) | (Oceania, NorthAmerica) => 220.0,

            // Europe connections
            (Europe, Asia) | (Asia, Europe) => 140.0,
            (Europe, SouthAmerica) | (SouthAmerica, Europe) => 280.0,
            (Europe, Africa) | (Africa, Europe) => 100.0,
            (Europe, Oceania) | (Oceania, Europe) => 320.0,

            // Asia connections
            (Asia, SouthAmerica) | (SouthAmerica, Asia) => 350.0,
            (Asia, Africa) | (Africa, Asia) => 180.0,
            (Asia, Oceania) | (Oceania, Asia) => 120.0,

            // Other connections
            (SouthAmerica, Africa) | (Africa, SouthAmerica) => 250.0,
            (SouthAmerica, Oceania) | (Oceania, SouthAmerica) => 300.0,
            (Africa, Oceania) | (Oceania, Africa) => 280.0,

            // Rare region (treated as remote)
            (Rare, _) | (_, Rare) => 400.0,

            // Same region
            _ => 0.0,
        }
    }

    /// Optimize geographic distribution for a set of replicas
    pub fn optimize_geographic_distribution(
        &self,
        content_type: &str,
        target_replicas: u32,
        user_regions: &[GeographicRegion],
        constraints: &DistributionConstraints,
    ) -> Result<GeographicDistribution> {
        // Calculate region scores
        let region_scores = self.calculate_region_scores(user_regions, constraints)?;

        // Apply compliance filters
        let compliant_regions = self.filter_compliant_regions(&region_scores, constraints)?;

        // Select optimal regions
        let selected_regions = self.select_optimal_regions(
            compliant_regions,
            target_replicas,
            constraints,
        )?;

        // Calculate distribution metrics
        let distribution_metrics = self.calculate_distribution_metrics(&selected_regions, user_regions)?;

        Ok(GeographicDistribution {
            selected_regions,
            distribution_metrics: distribution_metrics.clone(),
            compliance_status: ComplianceStatus::Compliant,
            optimization_score: distribution_metrics.overall_score,
            estimated_cost: distribution_metrics.total_cost,
            estimated_latency: distribution_metrics.avg_latency,
        })
    }

    /// Calculate region performance scores
    fn calculate_region_scores(
        &self,
        user_regions: &[GeographicRegion],
        constraints: &DistributionConstraints,
    ) -> Result<HashMap<GeographicRegion, RegionScore>> {
        let mut region_scores = HashMap::new();

        for (region, info) in &self.region_info {
            // Skip regions with no capacity
            if info.capacity_info.available_storage_gb == 0 {
                continue;
            }

            let latency_score = self.calculate_latency_score(region, user_regions);
            let cost_score = self.calculate_cost_score(info);
            let reliability_score = self.calculate_reliability_score(info);
            let capacity_score = self.calculate_capacity_score(&info.capacity_info);
            let compliance_score = self.calculate_compliance_score(region, constraints);

            let weights = &self.cost_optimization;
            let overall_score = latency_score * weights.latency_weight
                + cost_score * weights.cost_weight
                + reliability_score * weights.reliability_weight
                + compliance_score * weights.compliance_weight;

            region_scores.insert(region.clone(), RegionScore {
                region: region.clone(),
                overall_score,
                latency_score,
                cost_score,
                reliability_score,
                capacity_score,
                compliance_score,
            });
        }

        Ok(region_scores)
    }

    /// Calculate latency score for a region
    fn calculate_latency_score(&self, region: &GeographicRegion, user_regions: &[GeographicRegion]) -> f64 {
        if user_regions.is_empty() {
            return 1.0;
        }

        let avg_latency = user_regions.iter()
            .filter_map(|user_region| {
                self.latency_matrix.measurements
                    .get(&(user_region.clone(), region.clone()))
                    .map(|measurement| measurement.avg_latency_ms)
            })
            .sum::<f64>() / user_regions.len() as f64;

        // Convert latency to score (lower latency = higher score)
        let max_acceptable_latency = 500.0; // ms
        ((max_acceptable_latency - avg_latency) / max_acceptable_latency).max(0.0)
    }

    /// Calculate cost score for a region
    fn calculate_cost_score(&self, region_info: &RegionInfo) -> f64 {
        let cost_factors = &region_info.cost_factors;

        // Normalize costs (lower cost = higher score)
        let electricity_score = 1.0 / (cost_factors.electricity_cost_per_kwh + 0.01);
        let datacenter_score = 1.0 / (cost_factors.data_center_costs + 0.1);
        let labor_score = 1.0 / (cost_factors.labor_costs + 0.1);
        let regulatory_score = 1.0 / (cost_factors.regulatory_overhead + 0.1);

        // Weighted average
        (electricity_score * 0.3 + datacenter_score * 0.3 + labor_score * 0.2 + regulatory_score * 0.2)
    }

    /// Calculate reliability score for a region
    fn calculate_reliability_score(&self, region_info: &RegionInfo) -> f64 {
        let infrastructure = &region_info.infrastructure_quality;
        let disaster_risk = &region_info.disaster_risk;

        let infrastructure_score = (infrastructure.power_reliability / 100.0)
            * (infrastructure.internet_penetration / 100.0)
            * (infrastructure.fiber_coverage / 100.0);

        let stability_score = region_info.political_stability * (1.0 - disaster_risk.overall_risk_score);

        (infrastructure_score + stability_score) / 2.0
    }

    /// Calculate capacity score for a region
    fn calculate_capacity_score(&self, capacity_info: &RegionalCapacity) -> f64 {
        let utilization_factor = 1.0 - (capacity_info.utilization_rate / 100.0);
        let growth_factor = (capacity_info.growth_rate_monthly / 20.0).min(1.0);
        let availability_factor = capacity_info.available_storage_gb as f64 / 1_000_000.0; // Scale to reasonable range

        (utilization_factor + growth_factor + availability_factor.min(1.0)) / 3.0
    }

    /// Calculate compliance score for a region
    fn calculate_compliance_score(&self, region: &GeographicRegion, constraints: &DistributionConstraints) -> f64 {
        let mut score = 1.0;

        for rule_id in &constraints.required_compliance {
            if let Some(rule) = self.compliance_rules.get(rule_id) {
                if !rule.applicable_regions.contains(region) {
                    match rule.severity {
                        ComplianceSeverity::Critical => score *= 0.0, // Cannot use this region
                        ComplianceSeverity::Important => score *= 0.5,
                        ComplianceSeverity::Recommended => score *= 0.8,
                    }
                }
            }
        }

        score
    }

    /// Filter regions by compliance requirements
    fn filter_compliant_regions(
        &self,
        region_scores: &HashMap<GeographicRegion, RegionScore>,
        constraints: &DistributionConstraints,
    ) -> Result<HashMap<GeographicRegion, RegionScore>> {
        let mut compliant_regions = HashMap::new();

        for (region, score) in region_scores {
            let mut is_compliant = true;

            // Check critical compliance requirements
            for rule_id in &constraints.required_compliance {
                if let Some(rule) = self.compliance_rules.get(rule_id) {
                    if matches!(rule.severity, ComplianceSeverity::Critical) {
                        if !rule.applicable_regions.contains(region) {
                            is_compliant = false;
                            break;
                        }

                        // Check cross-border restrictions
                        if rule.data_residency_required && !constraints.allowed_regions.contains(region) {
                            is_compliant = false;
                            break;
                        }
                    }
                }
            }

            // Check explicit region restrictions
            if !constraints.allowed_regions.is_empty() && !constraints.allowed_regions.contains(region) {
                is_compliant = false;
            }

            if constraints.forbidden_regions.contains(region) {
                is_compliant = false;
            }

            if is_compliant {
                compliant_regions.insert(region.clone(), score.clone());
            }
        }

        Ok(compliant_regions)
    }

    /// Select optimal regions based on scores and constraints
    fn select_optimal_regions(
        &self,
        mut region_scores: HashMap<GeographicRegion, RegionScore>,
        target_replicas: u32,
        constraints: &DistributionConstraints,
    ) -> Result<Vec<RegionAllocation>> {
        let mut selected_regions = Vec::new();

        // Sort regions by score
        let mut sorted_regions: Vec<_> = region_scores.into_iter().collect();
        sorted_regions.sort_by(|a, b| b.1.overall_score.partial_cmp(&a.1.overall_score).unwrap_or(std::cmp::Ordering::Equal));

        // Ensure minimum geographic diversity
        let mut regions_used = std::collections::HashSet::new();
        let min_regions = constraints.min_regions.max(1);

        // First pass: ensure minimum diversity
        for (region, score) in &sorted_regions {
            if regions_used.len() >= min_regions as usize {
                break;
            }

            if regions_used.insert(region.clone()) {
                let allocation = RegionAllocation {
                    region: region.clone(),
                    replica_count: 1,
                    score: score.clone(),
                    cost_per_replica: self.estimate_region_cost(region),
                };
                selected_regions.push(allocation);
            }
        }

        // Second pass: distribute remaining replicas
        let remaining_replicas = target_replicas.saturating_sub(selected_regions.len() as u32);
        for _ in 0..remaining_replicas {
            // Find best region for next replica
            let best_region = sorted_regions.iter()
                .find(|(region, _)| {
                    // Check if region can handle more replicas
                    let current_count = selected_regions.iter()
                        .find(|alloc| alloc.region == *region)
                        .map(|alloc| alloc.replica_count)
                        .unwrap_or(0);

                    current_count < constraints.max_replicas_per_region
                });

            if let Some((region, score)) = best_region {
                // Add replica to existing allocation or create new one
                if let Some(allocation) = selected_regions.iter_mut()
                    .find(|alloc| alloc.region == *region) {
                    allocation.replica_count += 1;
                } else {
                    selected_regions.push(RegionAllocation {
                        region: region.clone(),
                        replica_count: 1,
                        score: score.clone(),
                        cost_per_replica: self.estimate_region_cost(region),
                    });
                }
            } else {
                break; // No more suitable regions
            }
        }

        Ok(selected_regions)
    }

    /// Calculate distribution metrics
    fn calculate_distribution_metrics(
        &self,
        selected_regions: &[RegionAllocation],
        user_regions: &[GeographicRegion],
    ) -> Result<DistributionMetrics> {
        let total_replicas: u32 = selected_regions.iter().map(|alloc| alloc.replica_count).sum();
        let total_cost = selected_regions.iter()
            .map(|alloc| alloc.cost_per_replica * alloc.replica_count as f64)
            .sum();

        let avg_latency = if user_regions.is_empty() {
            100.0 // Default latency
        } else {
            let total_latency: f64 = selected_regions.iter()
                .flat_map(|alloc| {
                    user_regions.iter().map(move |user_region| {
                        self.latency_matrix.measurements
                            .get(&(user_region.clone(), alloc.region.clone()))
                            .map(|measurement| measurement.avg_latency_ms)
                            .unwrap_or(200.0)
                    })
                })
                .sum();

            total_latency / (selected_regions.len() * user_regions.len()) as f64
        };

        let geographic_diversity = selected_regions.len() as f64 / 6.0; // Normalize by max regions

        let avg_reliability = selected_regions.iter()
            .map(|alloc| alloc.score.reliability_score)
            .sum::<f64>() / selected_regions.len() as f64;

        let overall_score = (1.0 / (avg_latency / 100.0))
            * (1.0 / (total_cost / 0.05))
            * avg_reliability
            * geographic_diversity;

        Ok(DistributionMetrics {
            total_replicas,
            total_cost,
            avg_latency,
            geographic_diversity,
            avg_reliability,
            overall_score,
        })
    }

    /// Estimate cost for storing in a region
    fn estimate_region_cost(&self, region: &GeographicRegion) -> f64 {
        self.region_info.get(region)
            .map(|info| {
                let base_cost = 0.02; // Base cost per GB per month
                base_cost * info.cost_factors.data_center_costs
            })
            .unwrap_or(0.05) // Default cost
    }

    /// Update latency measurement between regions
    pub fn update_latency_measurement(
        &mut self,
        source: GeographicRegion,
        target: GeographicRegion,
        measurement: LatencyMeasurement,
    ) {
        self.latency_matrix.measurements.insert((source, target), measurement);
        self.latency_matrix.last_updated = Utc::now();
    }

    /// Update regional performance metrics
    pub fn update_regional_performance(&mut self, region: GeographicRegion, performance: RegionalPerformance) {
        self.performance_analytics.regional_performance.insert(region, performance);
    }

    /// Get latency between two regions
    pub fn get_latency(&self, source: &GeographicRegion, target: &GeographicRegion) -> Option<f64> {
        self.latency_matrix.measurements
            .get(&(source.clone(), target.clone()))
            .map(|measurement| measurement.avg_latency_ms)
    }

    /// Get optimal regions for user access
    pub fn get_optimal_access_regions(&self, user_regions: &[GeographicRegion], max_latency: f64) -> Vec<GeographicRegion> {
        let mut optimal_regions = Vec::new();

        for (region, _) in &self.region_info {
            let avg_latency = user_regions.iter()
                .filter_map(|user_region| self.get_latency(user_region, region))
                .sum::<f64>() / user_regions.len().max(1) as f64;

            if avg_latency <= max_latency {
                optimal_regions.push(region.clone());
            }
        }

        // Sort by performance score
        optimal_regions.sort_by(|a, b| {
            let score_a = self.region_info.get(a)
                .map(|info| self.calculate_reliability_score(info))
                .unwrap_or(0.0);
            let score_b = self.region_info.get(b)
                .map(|info| self.calculate_reliability_score(info))
                .unwrap_or(0.0);

            score_b.partial_cmp(&score_a).unwrap_or(std::cmp::Ordering::Equal)
        });

        optimal_regions
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistributionConstraints {
    pub min_regions: u32,
    pub max_regions: u32,
    pub max_replicas_per_region: u32,
    pub required_compliance: Vec<String>,
    pub allowed_regions: Vec<GeographicRegion>,
    pub forbidden_regions: Vec<GeographicRegion>,
    pub max_latency_ms: f64,
    pub max_cost_per_gb: f64,
    pub min_reliability: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegionScore {
    pub region: GeographicRegion,
    pub overall_score: f64,
    pub latency_score: f64,
    pub cost_score: f64,
    pub reliability_score: f64,
    pub capacity_score: f64,
    pub compliance_score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeographicDistribution {
    pub selected_regions: Vec<RegionAllocation>,
    pub distribution_metrics: DistributionMetrics,
    pub compliance_status: ComplianceStatus,
    pub optimization_score: f64,
    pub estimated_cost: f64,
    pub estimated_latency: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegionAllocation {
    pub region: GeographicRegion,
    pub replica_count: u32,
    pub score: RegionScore,
    pub cost_per_replica: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistributionMetrics {
    pub total_replicas: u32,
    pub total_cost: f64,
    pub avg_latency: f64,
    pub geographic_diversity: f64,
    pub avg_reliability: f64,
    pub overall_score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComplianceStatus {
    Compliant,
    PartiallyCompliant,
    NonCompliant,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_geographic_optimizer_creation() {
        let optimizer = GeographicOptimizer::new();
        assert!(!optimizer.region_info.is_empty());
        assert!(!optimizer.compliance_rules.is_empty());
        assert!(!optimizer.latency_matrix.measurements.is_empty());
    }

    #[test]
    fn test_region_scoring() {
        let optimizer = GeographicOptimizer::new();
        let user_regions = vec![GeographicRegion::NorthAmerica];
        let constraints = DistributionConstraints {
            min_regions: 2,
            max_regions: 5,
            max_replicas_per_region: 3,
            required_compliance: vec![],
            allowed_regions: vec![],
            forbidden_regions: vec![],
            max_latency_ms: 300.0,
            max_cost_per_gb: 0.10,
            min_reliability: 0.9,
        };

        let scores = optimizer.calculate_region_scores(&user_regions, &constraints).unwrap();
        assert!(!scores.is_empty());

        // North America should have good latency score for North American users
        let na_score = scores.get(&GeographicRegion::NorthAmerica).unwrap();
        assert!(na_score.latency_score > 0.8);
    }

    #[test]
    fn test_latency_estimation() {
        let optimizer = GeographicOptimizer::new();

        // Same region should have low latency
        let same_region_latency = optimizer.estimate_baseline_latency(
            &GeographicRegion::NorthAmerica,
            &GeographicRegion::NorthAmerica
        );
        assert_eq!(same_region_latency, 0.0);

        // Cross-continental should have higher latency
        let cross_continental_latency = optimizer.estimate_baseline_latency(
            &GeographicRegion::NorthAmerica,
            &GeographicRegion::Asia
        );
        assert!(cross_continental_latency > 100.0);
    }
}