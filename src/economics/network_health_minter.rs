//! Network Health-Based Token Minting and Burning
//!
//! Automated token supply management based on ZephyrFS network metrics

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc, Duration};
use tokio::time::{sleep, Duration as TokioDuration};

use super::token_model::{TokenEconomicsManager, NetworkHealthMetrics};
use super::zephyr_coin::{ZephyrCoin, TokenEvent};

/// Network health-based minting controller
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthBasedMinter {
    /// Target network health thresholds
    pub health_thresholds: HealthThresholds,
    /// Minting rates based on health
    pub minting_rates: MintingRates,
    /// Burning policies
    pub burning_policies: BurningPolicies,
    /// Last operation timestamps
    pub last_operations: OperationTimestamps,
    /// Network performance history
    pub performance_history: Vec<PerformanceSnapshot>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthThresholds {
    /// Minimum uptime for minting (95%)
    pub min_uptime_percent: f64,
    /// Minimum geographic diversity (50%)
    pub min_geographic_diversity: f64,
    /// Minimum data durability (99.9%)
    pub min_data_durability: f64,
    /// Target utilization range (70-85%)
    pub target_utilization_min: f64,
    pub target_utilization_max: f64,
    /// Minimum active volunteers for rewards
    pub min_active_volunteers: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MintingRates {
    /// Base rate per healthy GB per day
    pub base_rate_per_gb: u64,
    /// Multipliers for health levels
    pub excellent_multiplier: f64, // >98% health
    pub good_multiplier: f64,      // 90-98% health
    pub fair_multiplier: f64,      // 80-90% health
    pub poor_multiplier: f64,      // <80% health (reduced/no minting)
    /// Bonus for rapid network growth
    pub growth_bonus_multiplier: f64,
    /// Emergency mint rate during network stress
    pub emergency_rate_multiplier: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BurningPolicies {
    /// Burn unused rewards after days
    pub unused_reward_burn_days: u32,
    /// Burn rate for idle tokens (percentage)
    pub idle_token_burn_rate: f64,
    /// Burn tokens during network congestion
    pub congestion_burn_enabled: bool,
    /// Maximum daily burn percentage
    pub max_daily_burn_percent: f64,
    /// Emergency burn during token oversupply
    pub emergency_burn_threshold: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationTimestamps {
    pub last_mint: DateTime<Utc>,
    pub last_burn: DateTime<Utc>,
    pub last_health_check: DateTime<Utc>,
    pub last_emergency_action: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceSnapshot {
    pub timestamp: DateTime<Utc>,
    pub metrics: NetworkHealthMetrics,
    pub health_score: f64,
    pub tokens_minted: u64,
    pub tokens_burned: u64,
    pub active_rewards: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HealthStatus {
    Excellent, // >98% health score
    Good,      // 90-98% health score
    Fair,      // 80-90% health score
    Poor,      // <80% health score
    Emergency, // Critical network issues
}

impl Default for HealthThresholds {
    fn default() -> Self {
        Self {
            min_uptime_percent: 95.0,
            min_geographic_diversity: 50.0,
            min_data_durability: 99.9,
            target_utilization_min: 70.0,
            target_utilization_max: 85.0,
            min_active_volunteers: 10,
        }
    }
}

impl Default for MintingRates {
    fn default() -> Self {
        Self {
            base_rate_per_gb: 20_000_000_000_000_000, // 0.02 ZEPH per GB per day
            excellent_multiplier: 1.5,
            good_multiplier: 1.0,
            fair_multiplier: 0.7,
            poor_multiplier: 0.3,
            growth_bonus_multiplier: 1.2,
            emergency_rate_multiplier: 2.0,
        }
    }
}

impl Default for BurningPolicies {
    fn default() -> Self {
        Self {
            unused_reward_burn_days: 90,
            idle_token_burn_rate: 0.01, // 1% quarterly
            congestion_burn_enabled: true,
            max_daily_burn_percent: 0.5, // 0.5% max daily burn
            emergency_burn_threshold: 1.1, // 110% of target supply
        }
    }
}

impl Default for HealthBasedMinter {
    fn default() -> Self {
        Self {
            health_thresholds: HealthThresholds::default(),
            minting_rates: MintingRates::default(),
            burning_policies: BurningPolicies::default(),
            last_operations: OperationTimestamps {
                last_mint: Utc::now(),
                last_burn: Utc::now(),
                last_health_check: Utc::now(),
                last_emergency_action: Utc::now(),
            },
            performance_history: Vec::new(),
        }
    }
}

/// Network health-based token operations
pub struct NetworkHealthController {
    minter: HealthBasedMinter,
    token_manager: TokenEconomicsManager,
    zephyr_coin: ZephyrCoin,
    events: Vec<TokenEvent>,
}

impl NetworkHealthController {
    /// Create new health-based controller
    pub fn new(
        minter: HealthBasedMinter,
        token_manager: TokenEconomicsManager,
        zephyr_coin: ZephyrCoin,
    ) -> Self {
        Self {
            minter,
            token_manager,
            zephyr_coin,
            events: Vec::new(),
        }
    }

    /// Calculate network health score
    pub fn calculate_health_score(&self, metrics: &NetworkHealthMetrics) -> f64 {
        let uptime_score = (metrics.average_uptime / 100.0).min(1.0);
        let diversity_score = (metrics.geographic_diversity / 100.0).min(1.0);
        let durability_score = (metrics.data_durability / 100.0).min(1.0);
        let utilization_score = self.calculate_utilization_score(metrics.utilization_rate);
        let volunteer_score = self.calculate_volunteer_score(metrics.active_volunteers);

        // Weighted health score
        let weights = [0.25, 0.20, 0.25, 0.15, 0.15]; // uptime, diversity, durability, utilization, volunteers
        let scores = [uptime_score, diversity_score, durability_score, utilization_score, volunteer_score];

        scores.iter().zip(weights.iter())
            .map(|(score, weight)| score * weight)
            .sum::<f64>() * 100.0
    }

    /// Calculate utilization score (optimal range: 70-85%)
    fn calculate_utilization_score(&self, utilization: f64) -> f64 {
        let min = self.minter.health_thresholds.target_utilization_min;
        let max = self.minter.health_thresholds.target_utilization_max;

        if utilization >= min && utilization <= max {
            1.0 // Perfect utilization
        } else if utilization < min {
            utilization / min // Underutilized
        } else {
            // Overutilized - exponential penalty
            (1.0 / (1.0 + (utilization - max) / 10.0)).max(0.1)
        }
    }

    /// Calculate volunteer participation score
    fn calculate_volunteer_score(&self, volunteers: u32) -> f64 {
        let min = self.minter.health_thresholds.min_active_volunteers;
        if volunteers >= min {
            (volunteers as f64 / (min as f64 * 2.0)).min(1.0)
        } else {
            volunteers as f64 / min as f64
        }
    }

    /// Determine health status from score
    pub fn determine_health_status(&self, health_score: f64) -> HealthStatus {
        if health_score >= 98.0 {
            HealthStatus::Excellent
        } else if health_score >= 90.0 {
            HealthStatus::Good
        } else if health_score >= 80.0 {
            HealthStatus::Fair
        } else if health_score >= 50.0 {
            HealthStatus::Poor
        } else {
            HealthStatus::Emergency
        }
    }

    /// Execute health-based minting
    pub async fn execute_health_based_minting(&mut self, metrics: NetworkHealthMetrics) -> Result<u64> {
        let health_score = self.calculate_health_score(&metrics);
        let health_status = self.determine_health_status(health_score);

        // Determine minting multiplier based on health
        let multiplier = match health_status {
            HealthStatus::Excellent => self.minter.minting_rates.excellent_multiplier,
            HealthStatus::Good => self.minter.minting_rates.good_multiplier,
            HealthStatus::Fair => self.minter.minting_rates.fair_multiplier,
            HealthStatus::Poor => self.minter.minting_rates.poor_multiplier,
            HealthStatus::Emergency => 0.0, // No minting during emergency
        };

        // Calculate growth bonus
        let growth_bonus = self.calculate_growth_bonus(&metrics)?;
        let final_multiplier = multiplier * growth_bonus;

        // Calculate mint amount
        let daily_base = metrics.total_capacity_gb * self.minter.minting_rates.base_rate_per_gb;
        let mint_amount = (daily_base as f64 * final_multiplier) as u64;

        if mint_amount > 0 && matches!(health_status, HealthStatus::Excellent | HealthStatus::Good | HealthStatus::Fair) {
            // Execute minting through token contract
            let mint_event = self.zephyr_coin.mint("network_minter", "reward_pool", mint_amount)?;
            self.events.push(mint_event);

            // Update token manager
            self.token_manager.mint_rewards(mint_amount).await?;

            // Record performance snapshot
            self.record_performance_snapshot(metrics, health_score, mint_amount, 0).await;

            tracing::info!("Minted {} tokens based on network health score: {:.2}%",
                mint_amount, health_score);

            self.minter.last_operations.last_mint = Utc::now();
            return Ok(mint_amount);
        }

        Ok(0)
    }

    /// Calculate growth bonus multiplier
    fn calculate_growth_bonus(&self, metrics: &NetworkHealthMetrics) -> Result<f64> {
        if self.minter.performance_history.len() < 7 {
            return Ok(1.0); // No bonus without sufficient history
        }

        // Calculate 7-day growth rate
        let current_capacity = metrics.total_capacity_gb;
        let week_ago_capacity = self.minter.performance_history
            .iter()
            .rev()
            .nth(6)
            .map(|snapshot| snapshot.metrics.total_capacity_gb)
            .unwrap_or(current_capacity);

        if week_ago_capacity == 0 {
            return Ok(1.0);
        }

        let growth_rate = (current_capacity as f64 - week_ago_capacity as f64) / week_ago_capacity as f64;

        // Apply growth bonus for healthy growth (10-30% weekly)
        if growth_rate >= 0.1 && growth_rate <= 0.3 {
            Ok(self.minter.minting_rates.growth_bonus_multiplier)
        } else {
            Ok(1.0)
        }
    }

    /// Execute health-based burning
    pub async fn execute_health_based_burning(&mut self, metrics: NetworkHealthMetrics) -> Result<u64> {
        let mut total_burned = 0u64;

        // Burn unused rewards
        total_burned += self.burn_unused_rewards().await?;

        // Emergency burn if oversupply
        total_burned += self.emergency_burn_oversupply(&metrics).await?;

        // Congestion burn
        if self.minter.burning_policies.congestion_burn_enabled {
            total_burned += self.burn_for_congestion(&metrics).await?;
        }

        if total_burned > 0 {
            self.minter.last_operations.last_burn = Utc::now();
            tracing::info!("Burned {} tokens based on network conditions", total_burned);
        }

        Ok(total_burned)
    }

    /// Burn unused reward tokens
    async fn burn_unused_rewards(&mut self) -> Result<u64> {
        let days_since_last = (Utc::now() - self.minter.last_operations.last_burn).num_days();

        if days_since_last >= self.minter.burning_policies.unused_reward_burn_days as i64 {
            let burn_amount = self.token_manager.burn_unused_tokens().await?;

            if burn_amount > 0 {
                let burn_event = self.zephyr_coin.burn("network_burner", "reward_pool", burn_amount)?;
                self.events.push(burn_event);
            }

            return Ok(burn_amount);
        }

        Ok(0)
    }

    /// Emergency burn during token oversupply
    async fn emergency_burn_oversupply(&mut self, _metrics: &NetworkHealthMetrics) -> Result<u64> {
        let current_supply = self.zephyr_coin.total_supply;
        let target_supply = 21_000_000 * 10_u64.pow(18); // 21M tokens
        let oversupply_threshold = (target_supply as f64 * self.minter.burning_policies.emergency_burn_threshold) as u64;

        if current_supply > oversupply_threshold {
            let excess = current_supply - target_supply;
            let burn_amount = (excess as f64 * 0.1) as u64; // Burn 10% of excess

            let max_daily_burn = (current_supply as f64 * self.minter.burning_policies.max_daily_burn_percent / 100.0) as u64;
            let final_burn = burn_amount.min(max_daily_burn);

            if final_burn > 0 {
                let burn_event = self.zephyr_coin.burn("emergency_burner", "reward_pool", final_burn)?;
                self.events.push(burn_event);

                tracing::warn!("Emergency burn of {} tokens due to oversupply", final_burn);
                return Ok(final_burn);
            }
        }

        Ok(0)
    }

    /// Burn tokens during network congestion
    async fn burn_for_congestion(&mut self, metrics: &NetworkHealthMetrics) -> Result<u64> {
        // Burn if utilization > 95% to incentivize capacity expansion
        if metrics.utilization_rate > 95.0 {
            let daily_rewards = metrics.total_capacity_gb * self.minter.minting_rates.base_rate_per_gb;
            let congestion_burn = (daily_rewards as f64 * 0.05) as u64; // 5% of daily rewards

            if congestion_burn > 0 {
                let burn_event = self.zephyr_coin.burn("congestion_burner", "reward_pool", congestion_burn)?;
                self.events.push(burn_event);

                tracing::info!("Congestion burn of {} tokens (utilization: {:.1}%)",
                    congestion_burn, metrics.utilization_rate);
                return Ok(congestion_burn);
            }
        }

        Ok(0)
    }

    /// Record performance snapshot
    async fn record_performance_snapshot(
        &mut self,
        metrics: NetworkHealthMetrics,
        health_score: f64,
        tokens_minted: u64,
        tokens_burned: u64,
    ) {
        let snapshot = PerformanceSnapshot {
            timestamp: Utc::now(),
            metrics,
            health_score,
            tokens_minted,
            tokens_burned,
            active_rewards: self.token_manager.get_supply_status().reward_pool,
        };

        self.minter.performance_history.push(snapshot);

        // Keep only last 30 days of history
        if self.minter.performance_history.len() > 30 {
            self.minter.performance_history.remove(0);
        }
    }

    /// Run automated health monitoring loop
    pub async fn run_health_monitor(&mut self, check_interval_hours: u64) -> Result<()> {
        let mut interval = tokio::time::interval(TokioDuration::from_secs(check_interval_hours * 3600));

        loop {
            interval.tick().await;

            // Get current network metrics (would be fetched from network in real implementation)
            let metrics = self.get_current_network_metrics().await?;

            // Update token manager with current metrics
            self.token_manager.update_network_metrics(metrics.clone());

            // Execute health-based operations
            let minted = self.execute_health_based_minting(metrics.clone()).await?;
            let burned = self.execute_health_based_burning(metrics.clone()).await?;

            // Perform token manager adjustments
            self.token_manager.perform_supply_adjustment().await?;

            self.minter.last_operations.last_health_check = Utc::now();

            tracing::info!("Health check complete: minted={}, burned={}", minted, burned);
        }
    }

    /// Get current network metrics (placeholder - would fetch from actual network)
    async fn get_current_network_metrics(&self) -> Result<NetworkHealthMetrics> {
        // This would fetch real metrics from the ZephyrFS network
        Ok(NetworkHealthMetrics {
            total_capacity_gb: 1000,
            active_volunteers: 50,
            utilization_rate: 75.0,
            average_uptime: 96.5,
            geographic_diversity: 65.0,
            data_durability: 99.95,
        })
    }

    /// Get recent events
    pub fn get_recent_events(&self) -> &[TokenEvent] {
        &self.events
    }

    /// Get performance history
    pub fn get_performance_history(&self) -> &[PerformanceSnapshot] {
        &self.minter.performance_history
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::economics::token_model::TokenEconomics;

    #[tokio::test]
    async fn test_health_score_calculation() {
        let minter = HealthBasedMinter::default();
        let token_manager = TokenEconomicsManager::new(TokenEconomics::default());
        let zephyr_coin = ZephyrCoin::new("test_owner".to_string(), 0);
        let controller = NetworkHealthController::new(minter, token_manager, zephyr_coin);

        let metrics = NetworkHealthMetrics {
            total_capacity_gb: 1000,
            active_volunteers: 25,
            utilization_rate: 75.0,
            average_uptime: 96.0,
            geographic_diversity: 60.0,
            data_durability: 99.9,
        };

        let health_score = controller.calculate_health_score(&metrics);
        assert!(health_score >= 80.0); // Should be "Good" or better
        assert!(health_score <= 100.0);
    }

    #[tokio::test]
    async fn test_health_based_minting() {
        let minter = HealthBasedMinter::default();
        let token_manager = TokenEconomicsManager::new(TokenEconomics::default());
        let mut zephyr_coin = ZephyrCoin::new("test_owner".to_string(), 0);
        zephyr_coin.add_minter("test_owner", "network_minter").unwrap();

        let mut controller = NetworkHealthController::new(minter, token_manager, zephyr_coin);

        let metrics = NetworkHealthMetrics {
            total_capacity_gb: 100,
            active_volunteers: 15,
            utilization_rate: 75.0,
            average_uptime: 98.0,
            geographic_diversity: 70.0,
            data_durability: 99.95,
        };

        let minted = controller.execute_health_based_minting(metrics).await.unwrap();
        assert!(minted > 0);
    }
}