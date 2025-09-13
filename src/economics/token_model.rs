//! ZephyrCoin Token Economics Model
//!
//! Sustainable token supply with network-health-based mechanisms

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc, Duration};

/// ZephyrCoin token economics configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenEconomics {
    /// Total supply cap (21M tokens, Bitcoin-inspired scarcity)
    pub max_supply: u64,
    /// Current circulating supply
    pub circulating_supply: u64,
    /// Tokens reserved for ecosystem development (10%)
    pub ecosystem_reserve: u64,
    /// Tokens allocated for volunteer rewards (70%)
    pub volunteer_rewards_pool: u64,
    /// Tokens for network maintenance and operations (20%)
    pub operations_pool: u64,
    /// Minimum network capacity before token minting
    pub min_network_capacity_gb: u64,
    /// Base reward rate per GB per day (in wei-equivalent)
    pub base_reward_rate: u64,
    /// Inflation rate control parameters
    pub inflation_control: InflationControl,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InflationControl {
    /// Maximum annual inflation rate (2.5%)
    pub max_annual_inflation: f64,
    /// Target network growth rate (20% monthly)
    pub target_network_growth: f64,
    /// Supply adjustment frequency (weekly)
    pub adjustment_frequency_days: u32,
    /// Burn rate for unused tokens (quarterly)
    pub burn_rate: f64,
}

/// Token supply management
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenSupply {
    /// Available tokens for immediate rewards
    pub reward_pool: u64,
    /// Tokens locked for future minting
    pub locked_reserve: u64,
    /// Burned tokens (deflationary mechanism)
    pub burned_tokens: u64,
    /// Last supply adjustment timestamp
    pub last_adjustment: DateTime<Utc>,
}

/// Network health metrics for token economics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkHealthMetrics {
    /// Total network storage capacity in GB
    pub total_capacity_gb: u64,
    /// Active volunteer nodes
    pub active_volunteers: u32,
    /// Network utilization percentage (0-100)
    pub utilization_rate: f64,
    /// Average node uptime percentage
    pub average_uptime: f64,
    /// Geographic distribution score (0-100)
    pub geographic_diversity: f64,
    /// Data durability percentage
    pub data_durability: f64,
}

/// Token economics manager
pub struct TokenEconomicsManager {
    config: TokenEconomics,
    supply: TokenSupply,
    health_metrics: NetworkHealthMetrics,
    reward_history: HashMap<String, Vec<RewardRecord>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RewardRecord {
    pub volunteer_id: String,
    pub amount: u64,
    pub earned_at: DateTime<Utc>,
    pub reason: RewardReason,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RewardReason {
    StorageProvision { gb_days: u64 },
    UptimeBonus { uptime_hours: u32 },
    PerformanceBonus { score: f64 },
    GeographicDiversity,
    NetworkStabilization,
}

impl Default for TokenEconomics {
    fn default() -> Self {
        Self {
            max_supply: 21_000_000 * 1_000_000_000_000_000_000, // 21M tokens with 18 decimals
            circulating_supply: 0,
            ecosystem_reserve: 2_100_000 * 1_000_000_000_000_000_000, // 10%
            volunteer_rewards_pool: 14_700_000 * 1_000_000_000_000_000_000, // 70%
            operations_pool: 4_200_000 * 1_000_000_000_000_000_000, // 20%
            min_network_capacity_gb: 100, // Start rewards at 100GB network capacity
            base_reward_rate: 20_000_000_000_000_000, // 0.02 tokens per GB per day
            inflation_control: InflationControl {
                max_annual_inflation: 0.025, // 2.5%
                target_network_growth: 0.20, // 20%
                adjustment_frequency_days: 7,
                burn_rate: 0.01, // 1% quarterly burn of unused tokens
            },
        }
    }
}

impl TokenEconomicsManager {
    /// Create new token economics manager
    pub fn new(config: TokenEconomics) -> Self {
        Self {
            config,
            supply: TokenSupply {
                reward_pool: 0,
                locked_reserve: config.volunteer_rewards_pool,
                burned_tokens: 0,
                last_adjustment: Utc::now(),
            },
            health_metrics: NetworkHealthMetrics {
                total_capacity_gb: 0,
                active_volunteers: 0,
                utilization_rate: 0.0,
                average_uptime: 0.0,
                geographic_diversity: 0.0,
                data_durability: 0.0,
            },
            reward_history: HashMap::new(),
        }
    }

    /// Calculate sustainable token release rate
    pub fn calculate_token_release_rate(&self) -> Result<u64> {
        // Base release tied to network growth
        let network_growth_factor = self.calculate_network_growth_factor()?;
        let utilization_factor = (self.health_metrics.utilization_rate / 100.0).min(1.0);
        let quality_factor = self.calculate_network_quality_factor();

        // Release rate formula: base_rate * growth_factor * utilization_factor * quality_factor
        let daily_release = (self.config.base_reward_rate as f64
            * self.health_metrics.total_capacity_gb as f64
            * network_growth_factor
            * utilization_factor
            * quality_factor) as u64;

        // Cap by inflation control
        let max_daily_inflation = self.calculate_max_daily_inflation()?;
        Ok(daily_release.min(max_daily_inflation))
    }

    /// Calculate network growth factor for token release
    fn calculate_network_growth_factor(&self) -> Result<f64> {
        // Encourage growth but prevent runaway inflation
        let growth_factor = if self.health_metrics.total_capacity_gb < 1000 {
            2.0 // High incentive for early adoption
        } else if self.health_metrics.total_capacity_gb < 10000 {
            1.5 // Moderate incentive for scaling
        } else {
            1.0 // Stable rewards for mature network
        };

        Ok(growth_factor)
    }

    /// Calculate network quality factor
    fn calculate_network_quality_factor(&self) -> f64 {
        let uptime_factor = self.health_metrics.average_uptime / 100.0;
        let diversity_factor = self.health_metrics.geographic_diversity / 100.0;
        let durability_factor = self.health_metrics.data_durability / 100.0;

        // Weighted average: uptime (40%), diversity (30%), durability (30%)
        (uptime_factor * 0.4 + diversity_factor * 0.3 + durability_factor * 0.3).max(0.1)
    }

    /// Calculate maximum daily inflation allowed
    fn calculate_max_daily_inflation(&self) -> Result<u64> {
        let annual_max = (self.config.circulating_supply as f64
            * self.config.inflation_control.max_annual_inflation) as u64;
        Ok(annual_max / 365)
    }

    /// Mint new tokens for rewards
    pub async fn mint_rewards(&mut self, amount: u64) -> Result<()> {
        // Verify supply constraints
        if self.config.circulating_supply + amount > self.config.max_supply {
            return Err(anyhow::anyhow!("Minting would exceed maximum supply"));
        }

        // Verify inflation constraints
        let max_daily = self.calculate_max_daily_inflation()?;
        if amount > max_daily {
            return Err(anyhow::anyhow!("Minting exceeds daily inflation limit"));
        }

        // Mint tokens
        self.supply.reward_pool += amount;
        self.config.circulating_supply += amount;
        self.supply.locked_reserve = self.supply.locked_reserve.saturating_sub(amount);

        tracing::info!("Minted {} tokens for rewards. Circulating supply: {}",
            amount, self.config.circulating_supply);

        Ok(())
    }

    /// Burn unused tokens (deflationary mechanism)
    pub async fn burn_unused_tokens(&mut self) -> Result<u64> {
        let days_since_adjustment = (Utc::now() - self.supply.last_adjustment).num_days();

        // Quarterly burn
        if days_since_adjustment >= 90 {
            let burn_amount = (self.supply.reward_pool as f64
                * self.config.inflation_control.burn_rate) as u64;

            if burn_amount > 0 {
                self.supply.reward_pool = self.supply.reward_pool.saturating_sub(burn_amount);
                self.supply.burned_tokens += burn_amount;
                self.config.circulating_supply = self.config.circulating_supply.saturating_sub(burn_amount);

                tracing::info!("Burned {} unused tokens. Total burned: {}",
                    burn_amount, self.supply.burned_tokens);

                return Ok(burn_amount);
            }
        }

        Ok(0)
    }

    /// Calculate reward for volunteer
    pub fn calculate_volunteer_reward(
        &self,
        volunteer_id: &str,
        storage_gb: u64,
        uptime_hours: u32,
        performance_score: f64
    ) -> Result<u64> {
        // Base storage reward
        let storage_reward = storage_gb * self.config.base_reward_rate;

        // Uptime bonus (up to 50% extra)
        let uptime_bonus = if uptime_hours >= 24 {
            (storage_reward as f64 * 0.5) as u64
        } else {
            (storage_reward as f64 * (uptime_hours as f64 / 24.0) * 0.5) as u64
        };

        // Performance bonus (up to 25% extra)
        let performance_bonus = (storage_reward as f64 * performance_score * 0.25) as u64;

        // Geographic diversity bonus
        let diversity_bonus = if self.is_rare_location(volunteer_id) {
            (storage_reward as f64 * 0.15) as u64
        } else {
            0
        };

        let total_reward = storage_reward + uptime_bonus + performance_bonus + diversity_bonus;

        // Ensure we have enough tokens in reward pool
        if total_reward > self.supply.reward_pool {
            return Err(anyhow::anyhow!("Insufficient tokens in reward pool"));
        }

        Ok(total_reward)
    }

    /// Check if volunteer is in a geographically rare location
    fn is_rare_location(&self, _volunteer_id: &str) -> bool {
        // Placeholder for geographic diversity calculation
        // Would integrate with actual geographic distribution data
        false
    }

    /// Distribute reward to volunteer
    pub async fn distribute_reward(
        &mut self,
        volunteer_id: String,
        amount: u64,
        reason: RewardReason
    ) -> Result<()> {
        if amount > self.supply.reward_pool {
            return Err(anyhow::anyhow!("Insufficient reward pool balance"));
        }

        // Deduct from reward pool
        self.supply.reward_pool -= amount;

        // Record reward
        let record = RewardRecord {
            volunteer_id: volunteer_id.clone(),
            amount,
            earned_at: Utc::now(),
            reason,
        };

        self.reward_history.entry(volunteer_id)
            .or_insert_with(Vec::new)
            .push(record);

        tracing::info!("Distributed {} tokens to volunteer. Remaining pool: {}",
            amount, self.supply.reward_pool);

        Ok(())
    }

    /// Update network health metrics
    pub fn update_network_metrics(&mut self, metrics: NetworkHealthMetrics) {
        self.health_metrics = metrics;
    }

    /// Get current token supply status
    pub fn get_supply_status(&self) -> TokenSupply {
        self.supply.clone()
    }

    /// Get total value locked in the system
    pub fn get_total_value_locked(&self) -> u64 {
        self.supply.reward_pool + self.supply.locked_reserve
    }

    /// Perform periodic supply adjustment
    pub async fn perform_supply_adjustment(&mut self) -> Result<()> {
        let days_since_adjustment = (Utc::now() - self.supply.last_adjustment).num_days();

        if days_since_adjustment >= self.config.inflation_control.adjustment_frequency_days as i64 {
            // Calculate and mint new rewards based on network health
            let daily_release = self.calculate_token_release_rate()?;
            let adjustment_amount = daily_release * days_since_adjustment as u64;

            if adjustment_amount > 0 {
                self.mint_rewards(adjustment_amount).await?;
            }

            // Perform quarterly burn
            self.burn_unused_tokens().await?;

            self.supply.last_adjustment = Utc::now();

            tracing::info!("Performed supply adjustment: +{} tokens", adjustment_amount);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_token_economics_basic() {
        let config = TokenEconomics::default();
        let mut manager = TokenEconomicsManager::new(config);

        // Test initial state
        assert_eq!(manager.get_supply_status().reward_pool, 0);
        assert_eq!(manager.config.circulating_supply, 0);

        // Test minting
        manager.mint_rewards(1000).await.unwrap();
        assert_eq!(manager.get_supply_status().reward_pool, 1000);
        assert_eq!(manager.config.circulating_supply, 1000);
    }

    #[tokio::test]
    async fn test_reward_calculation() {
        let config = TokenEconomics::default();
        let manager = TokenEconomicsManager::new(config);

        let reward = manager.calculate_volunteer_reward("test_volunteer", 10, 24, 0.8).unwrap();

        // Base: 10 GB * 0.02 tokens = 0.2 tokens
        // Uptime bonus: 50% of base = 0.1 tokens
        // Performance bonus: 80% * 25% of base = 0.04 tokens
        // Total should be around 0.34 tokens (in wei)
        assert!(reward > 0);
    }

    #[tokio::test]
    async fn test_supply_constraints() {
        let mut config = TokenEconomics::default();
        config.max_supply = 1000; // Small cap for testing
        let mut manager = TokenEconomicsManager::new(config);

        // Should fail when exceeding max supply
        assert!(manager.mint_rewards(2000).await.is_err());
    }
}