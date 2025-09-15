//! Real-Time Earnings Calculation System
//!
//! Comprehensive earnings tracking and calculation for ZephyrFS volunteers

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use chrono::{DateTime, Utc, Duration};

// Moved from legacy token_model for backward compatibility

/// Reason for reward calculation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RewardReason {
    StorageContribution,
    UptimeBonus,
    PerformanceBonus,
    GeographicBonus,
    NetworkHealthBonus,
}

/// Network health metrics for earnings calculation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkHealthMetrics {
    pub total_storage_tb: f64,
    pub total_capacity_gb: f64,
    pub active_nodes: u32,
    pub active_volunteers: u32,
    pub average_uptime_percentage: f64,
    pub average_uptime: f64,
    pub network_utilization_percentage: f64,
    pub utilization_rate: f64,
    pub geographic_distribution_score: f64,
    pub geographic_diversity: f64,
    pub data_redundancy_factor: f64,
    pub data_durability: f64,
}

/// Real-time earnings calculator for volunteers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EarningsCalculator {
    /// Volunteer performance tracking
    pub volunteer_metrics: HashMap<String, VolunteerMetrics>,
    /// Earnings rates configuration
    pub rates: EarningsRates,
    /// Bonus multipliers
    pub bonuses: BonusStructure,
    /// Performance history for analytics
    pub performance_history: HashMap<String, VecDeque<PerformanceRecord>>,
    /// Network-wide metrics
    pub network_metrics: NetworkHealthMetrics,
    /// Daily earnings summary
    pub daily_earnings: HashMap<String, DailyEarnings>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VolunteerMetrics {
    pub volunteer_id: String,
    pub total_storage_gb: u64,
    pub available_storage_gb: u64,
    pub used_storage_gb: u64,
    pub uptime_hours_24h: f64,
    pub uptime_percentage: f64,
    pub response_time_ms: u64,
    pub transfer_speed_mbps: f64,
    pub successful_transfers: u64,
    pub failed_transfers: u64,
    pub geographic_region: GeographicRegion,
    pub connection_quality: ConnectionQuality,
    pub reliability_score: f64,
    pub last_seen: DateTime<Utc>,
    pub joined_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EarningsRates {
    /// Base rate per GB per day (in wei-equivalent tokens)
    pub base_storage_rate: u64,
    /// Uptime bonus rate (per hour of 100% uptime)
    pub uptime_bonus_rate: u64,
    /// Performance bonus rate (based on speed/reliability)
    pub performance_bonus_rate: u64,
    /// Geographic diversity bonus
    pub geographic_bonus_rate: u64,
    /// Longevity bonus (tenure rewards)
    pub longevity_bonus_rate: u64,
    /// Network contribution bonus
    pub network_contribution_rate: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BonusStructure {
    /// Uptime thresholds and multipliers
    pub uptime_tiers: Vec<UptimeTier>,
    /// Performance score multipliers
    pub performance_multipliers: PerformanceMultipliers,
    /// Geographic diversity bonuses
    pub geographic_bonuses: HashMap<GeographicRegion, f64>,
    /// Tenure bonuses (loyalty rewards)
    pub tenure_bonuses: Vec<TenureTier>,
    /// Network health bonuses
    pub network_health_bonus: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UptimeTier {
    pub threshold_percent: f64,
    pub multiplier: f64,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMultipliers {
    pub excellent_speed: f64,      // >100 Mbps
    pub good_speed: f64,          // 50-100 Mbps
    pub average_speed: f64,       // 10-50 Mbps
    pub low_response_time: f64,   // <100ms
    pub high_reliability: f64,    // >99% success rate
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenureTier {
    pub months: u32,
    pub multiplier: f64,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GeographicRegion {
    NorthAmerica,
    Europe,
    Asia,
    SouthAmerica,
    Africa,
    Oceania,
    Rare, // Underrepresented regions
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConnectionQuality {
    Excellent, // Fiber, low latency
    Good,      // Broadband, stable
    Fair,      // Adequate speed
    Poor,      // Slow or unstable
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceRecord {
    pub timestamp: DateTime<Utc>,
    pub uptime_hours: f64,
    pub storage_provided_gb: u64,
    pub transfer_speed_mbps: f64,
    pub response_time_ms: u64,
    pub success_rate: f64,
    pub earnings_tokens: u64,
    pub bonus_tokens: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyEarnings {
    pub date: DateTime<Utc>,
    pub base_earnings: u64,
    pub uptime_bonus: u64,
    pub performance_bonus: u64,
    pub geographic_bonus: u64,
    pub longevity_bonus: u64,
    pub network_bonus: u64,
    pub total_earnings: u64,
    pub storage_gb_hours: u64,
    pub actual_uptime_hours: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EarningsProjection {
    pub daily_estimate: u64,
    pub weekly_estimate: u64,
    pub monthly_estimate: u64,
    pub annual_estimate: u64,
    pub factors: ProjectionFactors,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectionFactors {
    pub current_performance: f64,
    pub network_demand: f64,
    pub seasonal_adjustment: f64,
    pub growth_factor: f64,
}

impl Default for EarningsRates {
    fn default() -> Self {
        Self {
            base_storage_rate: 20_000_000_000_000_000, // 0.02 tokens per GB per day
            uptime_bonus_rate: 1_000_000_000_000_000,  // 0.001 tokens per hour
            performance_bonus_rate: 5_000_000_000_000_000, // 0.005 tokens for high performance
            geographic_bonus_rate: 3_000_000_000_000_000, // 0.003 tokens for rare regions
            longevity_bonus_rate: 2_000_000_000_000_000, // 0.002 tokens longevity bonus
            network_contribution_rate: 1_000_000_000_000_000, // 0.001 tokens network contribution
        }
    }
}

impl Default for BonusStructure {
    fn default() -> Self {
        Self {
            uptime_tiers: vec![
                UptimeTier { threshold_percent: 99.5, multiplier: 2.0, name: "Platinum".to_string() },
                UptimeTier { threshold_percent: 98.0, multiplier: 1.5, name: "Gold".to_string() },
                UptimeTier { threshold_percent: 95.0, multiplier: 1.2, name: "Silver".to_string() },
                UptimeTier { threshold_percent: 90.0, multiplier: 1.0, name: "Bronze".to_string() },
            ],
            performance_multipliers: PerformanceMultipliers {
                excellent_speed: 1.3,
                good_speed: 1.1,
                average_speed: 1.0,
                low_response_time: 1.2,
                high_reliability: 1.25,
            },
            geographic_bonuses: HashMap::from([
                (GeographicRegion::Rare, 0.5),
                (GeographicRegion::Africa, 0.3),
                (GeographicRegion::SouthAmerica, 0.2),
                (GeographicRegion::Oceania, 0.2),
                (GeographicRegion::Asia, 0.1),
                (GeographicRegion::Europe, 0.05),
                (GeographicRegion::NorthAmerica, 0.0),
            ]),
            tenure_bonuses: vec![
                TenureTier { months: 24, multiplier: 1.5, name: "Veteran".to_string() },
                TenureTier { months: 12, multiplier: 1.3, name: "Experienced".to_string() },
                TenureTier { months: 6, multiplier: 1.15, name: "Established".to_string() },
                TenureTier { months: 3, multiplier: 1.05, name: "Regular".to_string() },
            ],
            network_health_bonus: 0.1, // 10% bonus when network is healthy
        }
    }
}

impl EarningsCalculator {
    /// Create new earnings calculator
    pub fn new() -> Self {
        Self {
            volunteer_metrics: HashMap::new(),
            rates: EarningsRates::default(),
            bonuses: BonusStructure::default(),
            performance_history: HashMap::new(),
            network_metrics: NetworkHealthMetrics {
                total_storage_tb: 0.0,
                total_capacity_gb: 0.0,
                active_nodes: 0,
                active_volunteers: 0,
                average_uptime_percentage: 0.0,
                average_uptime: 0.0,
                network_utilization_percentage: 0.0,
                utilization_rate: 0.0,
                geographic_distribution_score: 0.0,
                geographic_diversity: 0.0,
                data_redundancy_factor: 0.0,
                data_durability: 0.0,
            },
            daily_earnings: HashMap::new(),
        }
    }

    /// Update volunteer metrics
    pub fn update_volunteer_metrics(&mut self, metrics: VolunteerMetrics) {
        let volunteer_id = metrics.volunteer_id.clone();

        // Record performance history
        let record = PerformanceRecord {
            timestamp: Utc::now(),
            uptime_hours: metrics.uptime_hours_24h,
            storage_provided_gb: metrics.used_storage_gb,
            transfer_speed_mbps: metrics.transfer_speed_mbps,
            response_time_ms: metrics.response_time_ms,
            success_rate: if metrics.successful_transfers + metrics.failed_transfers > 0 {
                metrics.successful_transfers as f64 / (metrics.successful_transfers + metrics.failed_transfers) as f64
            } else {
                1.0
            },
            earnings_tokens: 0, // Will be calculated
            bonus_tokens: 0,
        };

        self.performance_history
            .entry(volunteer_id.clone())
            .or_insert_with(|| VecDeque::with_capacity(720)) // 30 days of hourly records
            .push_back(record);

        // Keep only last 30 days
        if let Some(history) = self.performance_history.get_mut(&volunteer_id) {
            while history.len() > 720 {
                history.pop_front();
            }
        }

        self.volunteer_metrics.insert(volunteer_id, metrics);
    }

    /// Calculate real-time earnings for a volunteer
    pub fn calculate_real_time_earnings(&self, volunteer_id: &str) -> Result<u64> {
        let metrics = self.volunteer_metrics.get(volunteer_id)
            .ok_or_else(|| anyhow::anyhow!("Volunteer metrics not found"))?;

        // Calculate base storage earnings
        let base_earnings = self.calculate_base_storage_earnings(metrics);

        // Calculate uptime bonus
        let uptime_bonus = self.calculate_uptime_bonus(metrics);

        // Calculate performance bonus
        let performance_bonus = self.calculate_performance_bonus(metrics);

        // Calculate geographic bonus
        let geographic_bonus = self.calculate_geographic_bonus(metrics);

        // Calculate longevity bonus
        let longevity_bonus = self.calculate_longevity_bonus(metrics);

        // Calculate network health bonus
        let network_bonus = self.calculate_network_health_bonus(base_earnings);

        let total_earnings = base_earnings + uptime_bonus + performance_bonus
            + geographic_bonus + longevity_bonus + network_bonus;

        Ok(total_earnings)
    }

    /// Calculate base storage earnings
    fn calculate_base_storage_earnings(&self, metrics: &VolunteerMetrics) -> u64 {
        // Earnings based on storage provided and utilization
        let utilization_factor = if metrics.total_storage_gb > 0 {
            metrics.used_storage_gb as f64 / metrics.total_storage_gb as f64
        } else {
            0.0
        };

        // Base rate * storage * utilization (with minimum guarantee)
        let base = metrics.total_storage_gb * self.rates.base_storage_rate;
        let utilization_earnings = (base as f64 * utilization_factor) as u64;

        // Guarantee at least 20% of base rate even with no utilization
        let minimum = base / 5;

        utilization_earnings.max(minimum)
    }

    /// Calculate uptime bonus
    fn calculate_uptime_bonus(&self, metrics: &VolunteerMetrics) -> u64 {
        let uptime_tier = self.bonuses.uptime_tiers.iter()
            .find(|tier| metrics.uptime_percentage >= tier.threshold_percent)
            .unwrap_or(&self.bonuses.uptime_tiers[self.bonuses.uptime_tiers.len() - 1]);

        let base_bonus = (metrics.uptime_hours_24h * self.rates.uptime_bonus_rate as f64) as u64;
        (base_bonus as f64 * uptime_tier.multiplier) as u64
    }

    /// Calculate performance bonus
    fn calculate_performance_bonus(&self, metrics: &VolunteerMetrics) -> u64 {
        let mut multiplier = 1.0;

        // Speed bonus
        if metrics.transfer_speed_mbps >= 100.0 {
            multiplier *= self.bonuses.performance_multipliers.excellent_speed;
        } else if metrics.transfer_speed_mbps >= 50.0 {
            multiplier *= self.bonuses.performance_multipliers.good_speed;
        } else if metrics.transfer_speed_mbps >= 10.0 {
            multiplier *= self.bonuses.performance_multipliers.average_speed;
        }

        // Response time bonus
        if metrics.response_time_ms < 100 {
            multiplier *= self.bonuses.performance_multipliers.low_response_time;
        }

        // Reliability bonus
        let success_rate = if metrics.successful_transfers + metrics.failed_transfers > 0 {
            metrics.successful_transfers as f64 / (metrics.successful_transfers + metrics.failed_transfers) as f64
        } else {
            1.0
        };

        if success_rate >= 0.99 {
            multiplier *= self.bonuses.performance_multipliers.high_reliability;
        }

        (self.rates.performance_bonus_rate as f64 * multiplier) as u64
    }

    /// Calculate geographic diversity bonus
    fn calculate_geographic_bonus(&self, metrics: &VolunteerMetrics) -> u64 {
        let bonus_multiplier = self.bonuses.geographic_bonuses
            .get(&metrics.geographic_region)
            .copied()
            .unwrap_or(0.0);

        (self.rates.geographic_bonus_rate as f64 * (1.0 + bonus_multiplier)) as u64
    }

    /// Calculate longevity bonus
    fn calculate_longevity_bonus(&self, metrics: &VolunteerMetrics) -> u64 {
        let months_active = (Utc::now() - metrics.joined_at).num_days() / 30;

        let tenure_tier = self.bonuses.tenure_bonuses.iter()
            .find(|tier| months_active >= tier.months as i64)
            .cloned()
            .unwrap_or(TenureTier { months: 0, multiplier: 1.0, name: "New".to_string() });

        (self.rates.longevity_bonus_rate as f64 * tenure_tier.multiplier) as u64
    }

    /// Calculate network health bonus
    fn calculate_network_health_bonus(&self, base_earnings: u64) -> u64 {
        // Bonus when network is performing well
        let health_score = self.calculate_network_health_score();

        if health_score >= 95.0 {
            (base_earnings as f64 * self.bonuses.network_health_bonus) as u64
        } else {
            0
        }
    }

    /// Calculate overall network health score
    fn calculate_network_health_score(&self) -> f64 {
        let metrics = &self.network_metrics;

        // Weighted health score
        let uptime_score = metrics.average_uptime.min(100.0);
        let diversity_score = metrics.geographic_diversity.min(100.0);
        let durability_score = metrics.data_durability.min(100.0);

        (uptime_score * 0.4 + diversity_score * 0.3 + durability_score * 0.3)
    }

    /// Calculate hourly earnings for a volunteer
    pub fn calculate_hourly_earnings(&self, volunteer_id: &str) -> Result<u64> {
        let daily_earnings = self.calculate_real_time_earnings(volunteer_id)?;
        Ok(daily_earnings / 24) // Hourly rate
    }

    /// Calculate earnings projection
    pub fn calculate_earnings_projection(&self, volunteer_id: &str) -> Result<EarningsProjection> {
        let daily_earnings = self.calculate_real_time_earnings(volunteer_id)?;

        // Get performance trend
        let performance_trend = self.calculate_performance_trend(volunteer_id)?;

        // Network demand factor (based on utilization)
        let demand_factor = (self.network_metrics.utilization_rate / 100.0).min(1.2); // Cap at 120%

        // Seasonal adjustment (placeholder - would use historical data)
        let seasonal_factor = 1.0;

        // Growth factor based on network expansion
        let growth_factor = if self.network_metrics.active_volunteers < 100 {
            1.2 // Early network bonus
        } else {
            1.0
        };

        let factors = ProjectionFactors {
            current_performance: performance_trend,
            network_demand: demand_factor,
            seasonal_adjustment: seasonal_factor,
            growth_factor,
        };

        let adjusted_daily = (daily_earnings as f64 * performance_trend * demand_factor * growth_factor) as u64;

        Ok(EarningsProjection {
            daily_estimate: adjusted_daily,
            weekly_estimate: adjusted_daily * 7,
            monthly_estimate: adjusted_daily * 30,
            annual_estimate: adjusted_daily * 365,
            factors,
        })
    }

    /// Calculate performance trend for projections
    fn calculate_performance_trend(&self, volunteer_id: &str) -> Result<f64> {
        let history = self.performance_history.get(volunteer_id)
            .ok_or_else(|| anyhow::anyhow!("No performance history found"))?;

        if history.len() < 2 {
            return Ok(1.0); // No trend data
        }

        // Calculate trend over last 7 days
        let recent_records: Vec<_> = history.iter().rev().take(168).collect(); // Last 7 days (hourly)

        if recent_records.len() < 24 {
            return Ok(1.0);
        }

        let recent_performance = recent_records.iter().take(24)
            .map(|r| r.success_rate * (r.transfer_speed_mbps / 50.0).min(2.0))
            .sum::<f64>() / 24.0;

        let older_performance = recent_records.iter().skip(24).take(24)
            .map(|r| r.success_rate * (r.transfer_speed_mbps / 50.0).min(2.0))
            .sum::<f64>() / 24.0;

        if older_performance > 0.0 {
            Ok((recent_performance / older_performance).max(0.5).min(2.0)) // Cap trend between 0.5x and 2x
        } else {
            Ok(1.0)
        }
    }

    /// Update daily earnings summary
    pub fn update_daily_earnings(&mut self, volunteer_id: &str) -> Result<()> {
        let metrics = self.volunteer_metrics.get(volunteer_id)
            .ok_or_else(|| anyhow::anyhow!("Volunteer metrics not found"))?;

        let today = Utc::now().date_naive();
        let key = format!("{}_{}", volunteer_id, today.format("%Y-%m-%d"));

        let base_earnings = self.calculate_base_storage_earnings(metrics);
        let uptime_bonus = self.calculate_uptime_bonus(metrics);
        let performance_bonus = self.calculate_performance_bonus(metrics);
        let geographic_bonus = self.calculate_geographic_bonus(metrics);
        let longevity_bonus = self.calculate_longevity_bonus(metrics);
        let network_bonus = self.calculate_network_health_bonus(base_earnings);

        let daily_earnings = DailyEarnings {
            date: Utc::now(),
            base_earnings,
            uptime_bonus,
            performance_bonus,
            geographic_bonus,
            longevity_bonus,
            network_bonus,
            total_earnings: base_earnings + uptime_bonus + performance_bonus
                + geographic_bonus + longevity_bonus + network_bonus,
            storage_gb_hours: metrics.used_storage_gb * 24, // GB-hours for the day
            actual_uptime_hours: metrics.uptime_hours_24h,
        };

        self.daily_earnings.insert(key, daily_earnings);

        Ok(())
    }

    /// Get earnings breakdown for display
    pub fn get_earnings_breakdown(&self, volunteer_id: &str) -> Result<HashMap<String, u64>> {
        let metrics = self.volunteer_metrics.get(volunteer_id)
            .ok_or_else(|| anyhow::anyhow!("Volunteer metrics not found"))?;

        let mut breakdown = HashMap::new();
        breakdown.insert("base_storage".to_string(), self.calculate_base_storage_earnings(metrics));
        breakdown.insert("uptime_bonus".to_string(), self.calculate_uptime_bonus(metrics));
        breakdown.insert("performance_bonus".to_string(), self.calculate_performance_bonus(metrics));
        breakdown.insert("geographic_bonus".to_string(), self.calculate_geographic_bonus(metrics));
        breakdown.insert("longevity_bonus".to_string(), self.calculate_longevity_bonus(metrics));
        breakdown.insert("network_bonus".to_string(), self.calculate_network_health_bonus(self.calculate_base_storage_earnings(metrics)));

        Ok(breakdown)
    }

    /// Update network metrics
    pub fn update_network_metrics(&mut self, metrics: NetworkHealthMetrics) {
        self.network_metrics = metrics;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_earnings_calculation() {
        let mut calculator = EarningsCalculator::new();

        let metrics = VolunteerMetrics {
            volunteer_id: "test_volunteer".to_string(),
            total_storage_gb: 100,
            available_storage_gb: 50,
            used_storage_gb: 50,
            uptime_hours_24h: 24.0,
            uptime_percentage: 100.0,
            response_time_ms: 50,
            transfer_speed_mbps: 100.0,
            successful_transfers: 100,
            failed_transfers: 0,
            geographic_region: GeographicRegion::Europe,
            connection_quality: ConnectionQuality::Excellent,
            reliability_score: 1.0,
            last_seen: Utc::now(),
            joined_at: Utc::now() - Duration::days(365),
        };

        calculator.update_volunteer_metrics(metrics);

        let earnings = calculator.calculate_real_time_earnings("test_volunteer").unwrap();
        assert!(earnings > 0);

        let breakdown = calculator.get_earnings_breakdown("test_volunteer").unwrap();
        assert!(breakdown.contains_key("base_storage"));
        assert!(breakdown.contains_key("uptime_bonus"));
    }

    #[test]
    fn test_performance_multipliers() {
        let calculator = EarningsCalculator::new();

        let high_performance_metrics = VolunteerMetrics {
            volunteer_id: "high_perf".to_string(),
            total_storage_gb: 100,
            available_storage_gb: 0,
            used_storage_gb: 100,
            uptime_hours_24h: 24.0,
            uptime_percentage: 99.9,
            response_time_ms: 25,
            transfer_speed_mbps: 150.0,
            successful_transfers: 1000,
            failed_transfers: 1,
            geographic_region: GeographicRegion::Rare,
            connection_quality: ConnectionQuality::Excellent,
            reliability_score: 1.0,
            last_seen: Utc::now(),
            joined_at: Utc::now() - Duration::days(730),
        };

        let high_perf_bonus = calculator.calculate_performance_bonus(&high_performance_metrics);

        let low_performance_metrics = VolunteerMetrics {
            volunteer_id: "low_perf".to_string(),
            total_storage_gb: 100,
            available_storage_gb: 80,
            used_storage_gb: 20,
            uptime_hours_24h: 20.0,
            uptime_percentage: 83.3,
            response_time_ms: 200,
            transfer_speed_mbps: 5.0,
            successful_transfers: 80,
            failed_transfers: 20,
            geographic_region: GeographicRegion::NorthAmerica,
            connection_quality: ConnectionQuality::Fair,
            reliability_score: 0.8,
            last_seen: Utc::now(),
            joined_at: Utc::now() - Duration::days(30),
        };

        let low_perf_bonus = calculator.calculate_performance_bonus(&low_performance_metrics);

        assert!(high_perf_bonus > low_perf_bonus);
    }
}