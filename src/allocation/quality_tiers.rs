//! Quality Tier Management
//!
//! Manages service quality levels based on contribution rather than payment

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};

use crate::economics::PriorityLevel;

/// Quality tier manager for contribution-based service levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityTierManager {
    /// Available quality tiers
    pub tiers: HashMap<QualityTier, TierConfiguration>,
    /// User tier assignments
    pub user_tiers: HashMap<String, UserTierAssignment>,
    /// Tier performance metrics
    pub tier_metrics: HashMap<QualityTier, TierMetrics>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Hash, PartialEq, Eq)]
pub enum QualityTier {
    /// Basic service for deficit contributors
    Basic,
    /// Standard service for balanced contributors
    Standard,
    /// Premium service for surplus contributors
    Premium,
    /// Enterprise service for generous contributors
    Enterprise,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TierConfiguration {
    pub tier: QualityTier,
    pub name: String,
    pub description: String,
    pub requirements: TierRequirements,
    pub benefits: TierBenefits,
    pub service_level: ServiceLevel,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TierRequirements {
    /// Minimum contribution score to access this tier
    pub min_contribution_score: f64,
    /// Minimum priority level required
    pub min_priority_level: PriorityLevel,
    /// Minimum storage contribution (GB)
    pub min_storage_contribution_gb: u64,
    /// Minimum account age (days)
    pub min_account_age_days: u32,
    /// Minimum uptime percentage
    pub min_uptime_percentage: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TierBenefits {
    /// Storage allocation multiplier (1.0 = 100% of contribution)
    pub storage_multiplier: f64,
    /// Bandwidth allocation multiplier
    pub bandwidth_multiplier: f64,
    /// Priority in allocation queue (higher = better priority)
    pub allocation_priority: u32,
    /// SLA guarantees
    pub sla_guarantees: SLAGuarantees,
    /// Special features access
    pub features: Vec<TierFeature>,
    /// Support level
    pub support_level: SupportTier,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SLAGuarantees {
    pub uptime_guarantee_percent: f64,
    pub max_latency_ms: u32,
    pub min_throughput_mbps: f64,
    pub data_durability_percent: f64,
    pub recovery_time_objective_hours: u32,
    pub recovery_point_objective_hours: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TierFeature {
    /// Priority customer support
    PrioritySupport,
    /// Advanced monitoring and analytics
    AdvancedMonitoring,
    /// Geographic replication options
    GeographicReplication,
    /// Custom SLA agreements
    CustomSLA,
    /// API access with higher rate limits
    EnhancedAPI,
    /// Beta feature access
    BetaFeatures,
    /// Dedicated technical account manager
    DedicatedAccountManager,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SupportTier {
    /// Community forum support only
    Community,
    /// Standard business hours email support
    Standard,
    /// Premium 24/7 support with phone
    Premium,
    /// Enterprise dedicated support team
    Enterprise,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceLevel {
    pub availability_target: f64,
    pub performance_targets: PerformanceTargets,
    pub data_protection: DataProtection,
    pub monitoring_level: MonitoringLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceTargets {
    pub response_time_p50_ms: u32,
    pub response_time_p95_ms: u32,
    pub response_time_p99_ms: u32,
    pub throughput_minimum_mbps: f64,
    pub concurrent_connections: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataProtection {
    pub backup_frequency_hours: u32,
    pub backup_retention_days: u32,
    pub geo_redundancy: bool,
    pub encryption_level: EncryptionLevel,
    pub compliance_standards: Vec<ComplianceStandard>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EncryptionLevel {
    /// AES-128 encryption
    Standard,
    /// AES-256 encryption
    Enhanced,
    /// AES-256 with hardware security modules
    Enterprise,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComplianceStandard {
    GDPR,
    HIPAA,
    SOC2,
    ISO27001,
    PCI_DSS,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MonitoringLevel {
    /// Basic uptime monitoring
    Basic,
    /// Standard performance monitoring
    Standard,
    /// Advanced monitoring with alerting
    Advanced,
    /// Enterprise monitoring with analytics
    Enterprise,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserTierAssignment {
    pub user_id: String,
    pub assigned_tier: QualityTier,
    pub assigned_at: DateTime<Utc>,
    pub last_evaluated: DateTime<Utc>,
    pub next_evaluation: DateTime<Utc>,
    pub tier_history: Vec<TierChange>,
    pub current_benefits: TierBenefits,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TierChange {
    pub from_tier: Option<QualityTier>,
    pub to_tier: QualityTier,
    pub reason: String,
    pub changed_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TierMetrics {
    pub tier: QualityTier,
    pub active_users: u32,
    pub average_satisfaction_score: f64,
    pub sla_compliance_percent: f64,
    pub average_resource_utilization: f64,
    pub support_ticket_count: u32,
    pub last_updated: DateTime<Utc>,
}

impl QualityTierManager {
    pub fn new() -> Self {
        let mut tiers = HashMap::new();

        // Basic Tier - for users with contribution deficits
        tiers.insert(QualityTier::Basic, TierConfiguration {
            tier: QualityTier::Basic,
            name: "Basic Service".to_string(),
            description: "Essential storage and bandwidth with basic guarantees".to_string(),
            requirements: TierRequirements {
                min_contribution_score: 0.0,
                min_priority_level: PriorityLevel::Deficit,
                min_storage_contribution_gb: 10,
                min_account_age_days: 0,
                min_uptime_percentage: 90.0,
            },
            benefits: TierBenefits {
                storage_multiplier: 0.5,  // Can use 50% of what they contribute
                bandwidth_multiplier: 0.5,
                allocation_priority: 100,
                sla_guarantees: SLAGuarantees {
                    uptime_guarantee_percent: 95.0,
                    max_latency_ms: 500,
                    min_throughput_mbps: 1.0,
                    data_durability_percent: 99.9,
                    recovery_time_objective_hours: 24,
                    recovery_point_objective_hours: 4,
                },
                features: vec![],
                support_level: SupportTier::Community,
            },
            service_level: ServiceLevel {
                availability_target: 95.0,
                performance_targets: PerformanceTargets {
                    response_time_p50_ms: 200,
                    response_time_p95_ms: 500,
                    response_time_p99_ms: 1000,
                    throughput_minimum_mbps: 1.0,
                    concurrent_connections: 10,
                },
                data_protection: DataProtection {
                    backup_frequency_hours: 24,
                    backup_retention_days: 30,
                    geo_redundancy: false,
                    encryption_level: EncryptionLevel::Standard,
                    compliance_standards: vec![],
                },
                monitoring_level: MonitoringLevel::Basic,
            },
            enabled: true,
        });

        // Standard Tier - for balanced contributors
        tiers.insert(QualityTier::Standard, TierConfiguration {
            tier: QualityTier::Standard,
            name: "Standard Service".to_string(),
            description: "Reliable storage and bandwidth with good performance guarantees".to_string(),
            requirements: TierRequirements {
                min_contribution_score: 1.0,
                min_priority_level: PriorityLevel::Balanced,
                min_storage_contribution_gb: 50,
                min_account_age_days: 7,
                min_uptime_percentage: 95.0,
            },
            benefits: TierBenefits {
                storage_multiplier: 1.0,  // Can use 100% of what they contribute
                bandwidth_multiplier: 1.0,
                allocation_priority: 500,
                sla_guarantees: SLAGuarantees {
                    uptime_guarantee_percent: 99.0,
                    max_latency_ms: 200,
                    min_throughput_mbps: 10.0,
                    data_durability_percent: 99.99,
                    recovery_time_objective_hours: 4,
                    recovery_point_objective_hours: 1,
                },
                features: vec![TierFeature::AdvancedMonitoring],
                support_level: SupportTier::Standard,
            },
            service_level: ServiceLevel {
                availability_target: 99.0,
                performance_targets: PerformanceTargets {
                    response_time_p50_ms: 100,
                    response_time_p95_ms: 200,
                    response_time_p99_ms: 500,
                    throughput_minimum_mbps: 10.0,
                    concurrent_connections: 50,
                },
                data_protection: DataProtection {
                    backup_frequency_hours: 12,
                    backup_retention_days: 90,
                    geo_redundancy: true,
                    encryption_level: EncryptionLevel::Enhanced,
                    compliance_standards: vec![ComplianceStandard::GDPR],
                },
                monitoring_level: MonitoringLevel::Standard,
            },
            enabled: true,
        });

        // Premium Tier - for surplus contributors
        tiers.insert(QualityTier::Premium, TierConfiguration {
            tier: QualityTier::Premium,
            name: "Premium Service".to_string(),
            description: "High-performance storage with premium support and features".to_string(),
            requirements: TierRequirements {
                min_contribution_score: 1.5,
                min_priority_level: PriorityLevel::Surplus,
                min_storage_contribution_gb: 200,
                min_account_age_days: 30,
                min_uptime_percentage: 98.0,
            },
            benefits: TierBenefits {
                storage_multiplier: 1.5,  // Can use 150% of what they contribute
                bandwidth_multiplier: 1.5,
                allocation_priority: 800,
                sla_guarantees: SLAGuarantees {
                    uptime_guarantee_percent: 99.5,
                    max_latency_ms: 100,
                    min_throughput_mbps: 50.0,
                    data_durability_percent: 99.999,
                    recovery_time_objective_hours: 1,
                    recovery_point_objective_hours: 0,
                },
                features: vec![
                    TierFeature::PrioritySupport,
                    TierFeature::AdvancedMonitoring,
                    TierFeature::GeographicReplication,
                    TierFeature::EnhancedAPI,
                ],
                support_level: SupportTier::Premium,
            },
            service_level: ServiceLevel {
                availability_target: 99.5,
                performance_targets: PerformanceTargets {
                    response_time_p50_ms: 50,
                    response_time_p95_ms: 100,
                    response_time_p99_ms: 200,
                    throughput_minimum_mbps: 50.0,
                    concurrent_connections: 200,
                },
                data_protection: DataProtection {
                    backup_frequency_hours: 6,
                    backup_retention_days: 365,
                    geo_redundancy: true,
                    encryption_level: EncryptionLevel::Enterprise,
                    compliance_standards: vec![
                        ComplianceStandard::GDPR,
                        ComplianceStandard::SOC2,
                        ComplianceStandard::ISO27001,
                    ],
                },
                monitoring_level: MonitoringLevel::Advanced,
            },
            enabled: true,
        });

        // Enterprise Tier - for generous contributors
        tiers.insert(QualityTier::Enterprise, TierConfiguration {
            tier: QualityTier::Enterprise,
            name: "Enterprise Service".to_string(),
            description: "Maximum performance with enterprise-grade guarantees and dedicated support".to_string(),
            requirements: TierRequirements {
                min_contribution_score: 2.0,
                min_priority_level: PriorityLevel::Generous,
                min_storage_contribution_gb: 1000,
                min_account_age_days: 90,
                min_uptime_percentage: 99.0,
            },
            benefits: TierBenefits {
                storage_multiplier: 2.0,  // Can use 200% of what they contribute
                bandwidth_multiplier: 2.0,
                allocation_priority: 1000,
                sla_guarantees: SLAGuarantees {
                    uptime_guarantee_percent: 99.9,
                    max_latency_ms: 50,
                    min_throughput_mbps: 100.0,
                    data_durability_percent: 99.9999,
                    recovery_time_objective_hours: 0,
                    recovery_point_objective_hours: 0,
                },
                features: vec![
                    TierFeature::PrioritySupport,
                    TierFeature::AdvancedMonitoring,
                    TierFeature::GeographicReplication,
                    TierFeature::CustomSLA,
                    TierFeature::EnhancedAPI,
                    TierFeature::BetaFeatures,
                    TierFeature::DedicatedAccountManager,
                ],
                support_level: SupportTier::Enterprise,
            },
            service_level: ServiceLevel {
                availability_target: 99.9,
                performance_targets: PerformanceTargets {
                    response_time_p50_ms: 25,
                    response_time_p95_ms: 50,
                    response_time_p99_ms: 100,
                    throughput_minimum_mbps: 100.0,
                    concurrent_connections: 1000,
                },
                data_protection: DataProtection {
                    backup_frequency_hours: 1,
                    backup_retention_days: 2555, // 7 years
                    geo_redundancy: true,
                    encryption_level: EncryptionLevel::Enterprise,
                    compliance_standards: vec![
                        ComplianceStandard::GDPR,
                        ComplianceStandard::HIPAA,
                        ComplianceStandard::SOC2,
                        ComplianceStandard::ISO27001,
                        ComplianceStandard::PCI_DSS,
                    ],
                },
                monitoring_level: MonitoringLevel::Enterprise,
            },
            enabled: true,
        });

        Self {
            tiers,
            user_tiers: HashMap::new(),
            tier_metrics: HashMap::new(),
        }
    }

    /// Assign appropriate tier based on user contribution
    pub async fn assign_user_tier(&mut self, user_id: String, contribution_score: f64, priority_level: PriorityLevel, storage_contribution_gb: u64, account_age_days: u32, uptime_percentage: f64) -> Result<QualityTier> {

        // Find the highest tier the user qualifies for
        let qualified_tier = if self.meets_requirements(&QualityTier::Enterprise, contribution_score, &priority_level, storage_contribution_gb, account_age_days, uptime_percentage) {
            QualityTier::Enterprise
        } else if self.meets_requirements(&QualityTier::Premium, contribution_score, &priority_level, storage_contribution_gb, account_age_days, uptime_percentage) {
            QualityTier::Premium
        } else if self.meets_requirements(&QualityTier::Standard, contribution_score, &priority_level, storage_contribution_gb, account_age_days, uptime_percentage) {
            QualityTier::Standard
        } else {
            QualityTier::Basic
        };

        // Get current assignment if exists
        let previous_tier = self.user_tiers.get(&user_id).map(|assignment| assignment.assigned_tier.clone());

        // Create or update tier assignment
        let benefits = self.tiers.get(&qualified_tier).unwrap().benefits.clone();

        let assignment = UserTierAssignment {
            user_id: user_id.clone(),
            assigned_tier: qualified_tier.clone(),
            assigned_at: if previous_tier.is_some() {
                self.user_tiers.get(&user_id).unwrap().assigned_at
            } else {
                Utc::now()
            },
            last_evaluated: Utc::now(),
            next_evaluation: Utc::now() + chrono::Duration::days(7), // Re-evaluate weekly
            tier_history: {
                let mut history = self.user_tiers.get(&user_id)
                    .map(|a| a.tier_history.clone())
                    .unwrap_or_default();

                if previous_tier != Some(qualified_tier.clone()) {
                    history.push(TierChange {
                        from_tier: previous_tier,
                        to_tier: qualified_tier.clone(),
                        reason: format!("Contribution score: {:.2}, Storage: {}GB", contribution_score, storage_contribution_gb),
                        changed_at: Utc::now(),
                    });
                }
                history
            },
            current_benefits: benefits,
        };

        self.user_tiers.insert(user_id, assignment);

        Ok(qualified_tier)
    }

    /// Check if user meets tier requirements
    fn meets_requirements(&self, tier: &QualityTier, contribution_score: f64, priority_level: &PriorityLevel, storage_contribution_gb: u64, account_age_days: u32, uptime_percentage: f64) -> bool {
        if let Some(tier_config) = self.tiers.get(tier) {
            let reqs = &tier_config.requirements;

            contribution_score >= reqs.min_contribution_score &&
            storage_contribution_gb >= reqs.min_storage_contribution_gb &&
            account_age_days >= reqs.min_account_age_days &&
            uptime_percentage >= reqs.min_uptime_percentage &&
            self.priority_level_meets_requirement(priority_level, &reqs.min_priority_level)
        } else {
            false
        }
    }

    fn priority_level_meets_requirement(&self, user_level: &PriorityLevel, required_level: &PriorityLevel) -> bool {
        let user_score = match user_level {
            PriorityLevel::Deficit => 0,
            PriorityLevel::Balanced => 1,
            PriorityLevel::Surplus => 2,
            PriorityLevel::Generous => 3,
        };

        let required_score = match required_level {
            PriorityLevel::Deficit => 0,
            PriorityLevel::Balanced => 1,
            PriorityLevel::Surplus => 2,
            PriorityLevel::Generous => 3,
        };

        user_score >= required_score
    }

    /// Get user's current tier assignment
    pub fn get_user_tier(&self, user_id: &str) -> Option<&UserTierAssignment> {
        self.user_tiers.get(user_id)
    }

    /// Get tier configuration
    pub fn get_tier_config(&self, tier: &QualityTier) -> Option<&TierConfiguration> {
        self.tiers.get(tier)
    }

    /// Get all available tiers
    pub fn get_available_tiers(&self) -> Vec<&TierConfiguration> {
        self.tiers.values().filter(|t| t.enabled).collect()
    }

    /// Update tier metrics
    pub async fn update_tier_metrics(&mut self) -> Result<()> {
        // Count users per tier and calculate metrics
        for tier in [QualityTier::Basic, QualityTier::Standard, QualityTier::Premium, QualityTier::Enterprise] {
            let active_users = self.user_tiers.values()
                .filter(|assignment| assignment.assigned_tier == tier)
                .count() as u32;

            let metrics = TierMetrics {
                tier: tier.clone(),
                active_users,
                average_satisfaction_score: 85.0, // Mock data - would be calculated from user feedback
                sla_compliance_percent: 99.2,     // Mock data - would be calculated from actual SLA metrics
                average_resource_utilization: 75.0, // Mock data
                support_ticket_count: active_users / 10, // Mock ratio
                last_updated: Utc::now(),
            };

            self.tier_metrics.insert(tier, metrics);
        }

        Ok(())
    }

    /// Get tier metrics
    pub fn get_tier_metrics(&self, tier: &QualityTier) -> Option<&TierMetrics> {
        self.tier_metrics.get(tier)
    }
}

impl Default for QualityTierManager {
    fn default() -> Self {
        Self::new()
    }
}