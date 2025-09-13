//! Automated Market Maker for ZephyrCoin Price Stability
//!
//! Maintains stable token value through algorithmic trading and liquidity provision

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use chrono::{DateTime, Utc, Duration};

/// Automated Market Maker for ZephyrCoin
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZephyrCoinAMM {
    /// Liquidity pools for different pairs
    pub pools: HashMap<TradingPair, LiquidityPool>,
    /// Target price in USD (stable value target)
    pub target_price_usd: f64,
    /// Price stability configuration
    pub stability_config: StabilityConfig,
    /// Trading history for price analysis
    pub price_history: VecDeque<PriceSnapshot>,
    /// Current reserves
    pub reserves: Reserves,
    /// Fee structure
    pub fees: FeeStructure,
}

#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub enum TradingPair {
    ZEPH_USD,
    ZEPH_ETH,
    ZEPH_BTC,
    ZEPH_USDC,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiquidityPool {
    /// Pool reserves
    pub reserve_a: u64, // ZEPH tokens
    pub reserve_b: u64, // Other asset (in smallest units)
    /// Total liquidity provider shares
    pub total_shares: u64,
    /// Liquidity provider positions
    pub lp_positions: HashMap<String, LPPosition>,
    /// Pool fee rate (e.g., 0.003 for 0.3%)
    pub fee_rate: f64,
    /// Last price update
    pub last_update: DateTime<Utc>,
    /// Price impact protection
    pub max_slippage: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LPPosition {
    pub shares: u64,
    pub provided_at: DateTime<Utc>,
    pub initial_zeph: u64,
    pub initial_other: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StabilityConfig {
    /// Price deviation threshold for intervention (5%)
    pub intervention_threshold: f64,
    /// Maximum daily price change (10%)
    pub max_daily_change: f64,
    /// Rebalancing frequency (hours)
    pub rebalance_frequency: u32,
    /// Emergency circuit breaker threshold (20%)
    pub circuit_breaker_threshold: f64,
    /// Minimum liquidity ratio
    pub min_liquidity_ratio: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reserves {
    /// ZEPH token reserves for market making
    pub zeph_reserve: u64,
    /// USD equivalent reserves
    pub usd_reserve: u64,
    /// Emergency reserves
    pub emergency_reserve: u64,
    /// Insurance fund
    pub insurance_fund: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeeStructure {
    /// Trading fee (0.3%)
    pub trading_fee: f64,
    /// Stability fee for interventions (0.1%)
    pub stability_fee: f64,
    /// LP reward rate (daily APY)
    pub lp_reward_rate: f64,
    /// Protocol fee (goes to treasury)
    pub protocol_fee: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceSnapshot {
    pub timestamp: DateTime<Utc>,
    pub price_usd: f64,
    pub volume_24h: u64,
    pub liquidity_depth: u64,
    pub volatility: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeExecution {
    pub pair: TradingPair,
    pub amount_in: u64,
    pub amount_out: u64,
    pub price: f64,
    pub fee: u64,
    pub slippage: f64,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MarketOperation {
    Buy { amount_usd: u64 },
    Sell { amount_zeph: u64 },
    AddLiquidity { zeph_amount: u64, usd_amount: u64 },
    RemoveLiquidity { shares: u64 },
    Rebalance,
    EmergencyHalt,
}

impl Default for StabilityConfig {
    fn default() -> Self {
        Self {
            intervention_threshold: 0.05, // 5%
            max_daily_change: 0.10,       // 10%
            rebalance_frequency: 4,       // Every 4 hours
            circuit_breaker_threshold: 0.20, // 20%
            min_liquidity_ratio: 0.20,   // 20% min liquidity
        }
    }
}

impl Default for FeeStructure {
    fn default() -> Self {
        Self {
            trading_fee: 0.003,      // 0.3%
            stability_fee: 0.001,    // 0.1%
            lp_reward_rate: 0.05,    // 5% APY
            protocol_fee: 0.0005,    // 0.05%
        }
    }
}

impl ZephyrCoinAMM {
    /// Create new AMM with initial liquidity
    pub fn new(
        target_price_usd: f64,
        initial_zeph: u64,
        initial_usd: u64,
    ) -> Self {
        let mut pools = HashMap::new();

        // Initialize ZEPH/USD pool
        pools.insert(TradingPair::ZEPH_USD, LiquidityPool {
            reserve_a: initial_zeph,
            reserve_b: initial_usd,
            total_shares: (initial_zeph * initial_usd).integer_sqrt(),
            lp_positions: HashMap::new(),
            fee_rate: 0.003,
            last_update: Utc::now(),
            max_slippage: 0.05, // 5% max slippage
        });

        Self {
            pools,
            target_price_usd,
            stability_config: StabilityConfig::default(),
            price_history: VecDeque::with_capacity(1440), // 24 hours of minute data
            reserves: Reserves {
                zeph_reserve: initial_zeph / 2, // Keep 50% as reserves
                usd_reserve: initial_usd / 2,
                emergency_reserve: initial_zeph / 10, // 10% emergency
                insurance_fund: initial_usd / 20, // 5% insurance
            },
            fees: FeeStructure::default(),
        }
    }

    /// Get current price for a trading pair
    pub fn get_current_price(&self, pair: &TradingPair) -> Result<f64> {
        let pool = self.pools.get(pair)
            .ok_or_else(|| anyhow::anyhow!("Trading pair not found"))?;

        match pair {
            TradingPair::ZEPH_USD => {
                if pool.reserve_a == 0 {
                    return Err(anyhow::anyhow!("No ZEPH liquidity"));
                }
                Ok(pool.reserve_b as f64 / pool.reserve_a as f64)
            },
            _ => Err(anyhow::anyhow!("Price calculation not implemented for this pair")),
        }
    }

    /// Calculate price impact for a trade
    pub fn calculate_price_impact(&self, pair: &TradingPair, amount_in: u64, buy: bool) -> Result<f64> {
        let pool = self.pools.get(pair)
            .ok_or_else(|| anyhow::anyhow!("Trading pair not found"))?;

        let (reserve_in, reserve_out) = if buy {
            (pool.reserve_b, pool.reserve_a) // Buying ZEPH with USD
        } else {
            (pool.reserve_a, pool.reserve_b) // Selling ZEPH for USD
        };

        // Constant product formula: x * y = k
        let k = reserve_in * reserve_out;
        let new_reserve_in = reserve_in + amount_in;
        let new_reserve_out = k / new_reserve_in;
        let amount_out = reserve_out - new_reserve_out;

        // Calculate price impact
        let current_price = reserve_out as f64 / reserve_in as f64;
        let execution_price = amount_out as f64 / amount_in as f64;
        let price_impact = ((execution_price - current_price) / current_price).abs();

        Ok(price_impact)
    }

    /// Execute a swap with slippage protection
    pub fn execute_swap(
        &mut self,
        pair: TradingPair,
        amount_in: u64,
        min_amount_out: u64,
        buy: bool,
    ) -> Result<TradeExecution> {
        let pool = self.pools.get_mut(&pair)
            .ok_or_else(|| anyhow::anyhow!("Trading pair not found"))?;

        // Check price impact
        let price_impact = self.calculate_price_impact(&pair, amount_in, buy)?;
        if price_impact > pool.max_slippage {
            return Err(anyhow::anyhow!("Price impact too high: {:.2}%", price_impact * 100.0));
        }

        let (reserve_in, reserve_out) = if buy {
            (&mut pool.reserve_b, &mut pool.reserve_a)
        } else {
            (&mut pool.reserve_a, &mut pool.reserve_b)
        };

        // Calculate output amount with fee
        let amount_in_with_fee = (amount_in as f64 * (1.0 - pool.fee_rate)) as u64;
        let k = *reserve_in * *reserve_out;
        let new_reserve_in = *reserve_in + amount_in_with_fee;
        let new_reserve_out = k / new_reserve_in;
        let amount_out = *reserve_out - new_reserve_out;

        if amount_out < min_amount_out {
            return Err(anyhow::anyhow!("Slippage tolerance exceeded"));
        }

        // Update reserves
        *reserve_in += amount_in;
        *reserve_out = new_reserve_out;

        let execution_price = amount_out as f64 / amount_in as f64;
        let fee = amount_in - amount_in_with_fee;

        pool.last_update = Utc::now();

        Ok(TradeExecution {
            pair,
            amount_in,
            amount_out,
            price: execution_price,
            fee,
            slippage: price_impact,
            timestamp: Utc::now(),
        })
    }

    /// Add liquidity to a pool
    pub fn add_liquidity(
        &mut self,
        pair: TradingPair,
        user: String,
        amount_a: u64,
        amount_b: u64,
    ) -> Result<u64> {
        let pool = self.pools.get_mut(&pair)
            .ok_or_else(|| anyhow::anyhow!("Trading pair not found"))?;

        // Calculate optimal amounts based on current ratio
        let ratio = pool.reserve_b as f64 / pool.reserve_a as f64;
        let optimal_b = (amount_a as f64 * ratio) as u64;

        let (final_a, final_b) = if optimal_b <= amount_b {
            (amount_a, optimal_b)
        } else {
            let optimal_a = (amount_b as f64 / ratio) as u64;
            (optimal_a, amount_b)
        };

        // Calculate LP shares
        let liquidity = if pool.total_shares == 0 {
            (final_a * final_b).integer_sqrt()
        } else {
            std::cmp::min(
                final_a * pool.total_shares / pool.reserve_a,
                final_b * pool.total_shares / pool.reserve_b,
            )
        };

        // Update pool
        pool.reserve_a += final_a;
        pool.reserve_b += final_b;
        pool.total_shares += liquidity;

        // Record LP position
        pool.lp_positions.insert(user, LPPosition {
            shares: liquidity,
            provided_at: Utc::now(),
            initial_zeph: final_a,
            initial_other: final_b,
        });

        pool.last_update = Utc::now();

        Ok(liquidity)
    }

    /// Remove liquidity from a pool
    pub fn remove_liquidity(
        &mut self,
        pair: TradingPair,
        user: String,
        shares: u64,
    ) -> Result<(u64, u64)> {
        let pool = self.pools.get_mut(&pair)
            .ok_or_else(|| anyhow::anyhow!("Trading pair not found"))?;

        let position = pool.lp_positions.get_mut(&user)
            .ok_or_else(|| anyhow::anyhow!("No liquidity position found"))?;

        if position.shares < shares {
            return Err(anyhow::anyhow!("Insufficient LP shares"));
        }

        // Calculate withdrawal amounts
        let amount_a = shares * pool.reserve_a / pool.total_shares;
        let amount_b = shares * pool.reserve_b / pool.total_shares;

        // Update pool
        pool.reserve_a -= amount_a;
        pool.reserve_b -= amount_b;
        pool.total_shares -= shares;

        // Update position
        position.shares -= shares;
        if position.shares == 0 {
            pool.lp_positions.remove(&user);
        }

        pool.last_update = Utc::now();

        Ok((amount_a, amount_b))
    }

    /// Perform price stability intervention
    pub async fn perform_stability_intervention(&mut self) -> Result<Vec<MarketOperation>> {
        let current_price = self.get_current_price(&TradingPair::ZEPH_USD)?;
        let price_deviation = (current_price - self.target_price_usd) / self.target_price_usd;

        let mut operations = Vec::new();

        // Check if intervention is needed
        if price_deviation.abs() > self.stability_config.intervention_threshold {
            tracing::info!("Price deviation detected: {:.2}%, target: ${:.4}, current: ${:.4}",
                price_deviation * 100.0, self.target_price_usd, current_price);

            if price_deviation > 0.0 {
                // Price too high - sell ZEPH to decrease price
                let sell_amount = self.calculate_intervention_amount(price_deviation, false)?;
                operations.push(MarketOperation::Sell { amount_zeph: sell_amount });
            } else {
                // Price too low - buy ZEPH to increase price
                let buy_amount_usd = self.calculate_intervention_amount(price_deviation.abs(), true)?;
                operations.push(MarketOperation::Buy { amount_usd: buy_amount_usd });
            }
        }

        // Execute emergency halt if needed
        if price_deviation.abs() > self.stability_config.circuit_breaker_threshold {
            tracing::warn!("Emergency circuit breaker triggered at {:.2}% deviation", price_deviation * 100.0);
            operations.push(MarketOperation::EmergencyHalt);
        }

        Ok(operations)
    }

    /// Calculate intervention amount based on price deviation
    fn calculate_intervention_amount(&self, deviation: f64, is_buy: bool) -> Result<u64> {
        let pool = self.pools.get(&TradingPair::ZEPH_USD)
            .ok_or_else(|| anyhow::anyhow!("ZEPH/USD pool not found"))?;

        // Use a fraction of reserves based on deviation severity
        let intervention_factor = (deviation / self.stability_config.intervention_threshold).min(1.0);

        if is_buy {
            // Buy ZEPH with USD reserves
            let max_usd = self.reserves.usd_reserve / 10; // Max 10% of reserves per intervention
            Ok((max_usd as f64 * intervention_factor) as u64)
        } else {
            // Sell ZEPH from reserves
            let max_zeph = self.reserves.zeph_reserve / 10; // Max 10% of reserves per intervention
            Ok((max_zeph as f64 * intervention_factor) as u64)
        }
    }

    /// Update price history
    pub fn update_price_history(&mut self, price: f64, volume: u64) {
        let snapshot = PriceSnapshot {
            timestamp: Utc::now(),
            price_usd: price,
            volume_24h: volume,
            liquidity_depth: self.calculate_liquidity_depth(),
            volatility: self.calculate_volatility(),
        };

        self.price_history.push_back(snapshot);

        // Keep only last 24 hours
        while self.price_history.len() > 1440 {
            self.price_history.pop_front();
        }
    }

    /// Calculate current liquidity depth
    fn calculate_liquidity_depth(&self) -> u64 {
        self.pools.get(&TradingPair::ZEPH_USD)
            .map(|pool| pool.reserve_a + pool.reserve_b)
            .unwrap_or(0)
    }

    /// Calculate price volatility (24h)
    fn calculate_volatility(&self) -> f64 {
        if self.price_history.len() < 2 {
            return 0.0;
        }

        let prices: Vec<f64> = self.price_history.iter().map(|s| s.price_usd).collect();
        let mean = prices.iter().sum::<f64>() / prices.len() as f64;
        let variance = prices.iter()
            .map(|price| (price - mean).powi(2))
            .sum::<f64>() / prices.len() as f64;

        variance.sqrt() / mean // Coefficient of variation
    }

    /// Run automated market making loop
    pub async fn run_automated_trading(&mut self, interval_minutes: u64) -> Result<()> {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(interval_minutes * 60));

        loop {
            interval.tick().await;

            // Update current price
            if let Ok(price) = self.get_current_price(&TradingPair::ZEPH_USD) {
                self.update_price_history(price, 0); // Volume would be tracked separately

                // Perform stability intervention if needed
                let operations = self.perform_stability_intervention().await?;

                for operation in operations {
                    match operation {
                        MarketOperation::Buy { amount_usd } => {
                            self.execute_stability_buy(amount_usd).await?;
                        },
                        MarketOperation::Sell { amount_zeph } => {
                            self.execute_stability_sell(amount_zeph).await?;
                        },
                        MarketOperation::EmergencyHalt => {
                            self.emergency_halt().await?;
                            return Ok(()); // Stop trading
                        },
                        _ => {}, // Handle other operations as needed
                    }
                }

                // Rebalance if needed
                if self.should_rebalance().await? {
                    self.rebalance_pools().await?;
                }
            }

            tracing::debug!("AMM cycle complete");
        }
    }

    /// Execute stability buy operation
    async fn execute_stability_buy(&mut self, amount_usd: u64) -> Result<()> {
        if amount_usd > self.reserves.usd_reserve {
            return Err(anyhow::anyhow!("Insufficient USD reserves for stability buy"));
        }

        let trade = self.execute_swap(
            TradingPair::ZEPH_USD,
            amount_usd,
            0, // No minimum for stability operations
            true,
        )?;

        self.reserves.usd_reserve -= amount_usd;
        self.reserves.zeph_reserve += trade.amount_out;

        tracing::info!("Executed stability buy: {} USD -> {} ZEPH at ${:.4}",
            amount_usd, trade.amount_out, trade.price);

        Ok(())
    }

    /// Execute stability sell operation
    async fn execute_stability_sell(&mut self, amount_zeph: u64) -> Result<()> {
        if amount_zeph > self.reserves.zeph_reserve {
            return Err(anyhow::anyhow!("Insufficient ZEPH reserves for stability sell"));
        }

        let trade = self.execute_swap(
            TradingPair::ZEPH_USD,
            amount_zeph,
            0, // No minimum for stability operations
            false,
        )?;

        self.reserves.zeph_reserve -= amount_zeph;
        self.reserves.usd_reserve += trade.amount_out;

        tracing::info!("Executed stability sell: {} ZEPH -> {} USD at ${:.4}",
            amount_zeph, trade.amount_out, trade.price);

        Ok(())
    }

    /// Emergency halt trading
    async fn emergency_halt(&mut self) -> Result<()> {
        tracing::error!("Emergency halt activated - suspending all trading");
        // In real implementation, would pause all pools and notify operators
        Ok(())
    }

    /// Check if rebalancing is needed
    async fn should_rebalance(&self) -> Result<bool> {
        let last_rebalance = self.pools.get(&TradingPair::ZEPH_USD)
            .map(|pool| pool.last_update)
            .unwrap_or(Utc::now());

        let hours_since_rebalance = (Utc::now() - last_rebalance).num_hours();
        Ok(hours_since_rebalance >= self.stability_config.rebalance_frequency as i64)
    }

    /// Rebalance pools to maintain optimal ratios
    async fn rebalance_pools(&mut self) -> Result<()> {
        tracing::info!("Performing AMM rebalancing");

        // Rebalancing logic would optimize pool ratios
        // For now, just update timestamp
        if let Some(pool) = self.pools.get_mut(&TradingPair::ZEPH_USD) {
            pool.last_update = Utc::now();
        }

        Ok(())
    }
}

trait IntegerSqrt {
    fn integer_sqrt(self) -> Self;
}

impl IntegerSqrt for u64 {
    fn integer_sqrt(self) -> Self {
        if self < 2 {
            return self;
        }

        let mut x = self;
        let mut y = (x + 1) / 2;

        while y < x {
            x = y;
            y = (x + self / x) / 2;
        }

        x
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_amm_creation() {
        let amm = ZephyrCoinAMM::new(0.10, 1_000_000, 100_000); // $0.10 target price
        assert_eq!(amm.target_price_usd, 0.10);
        assert!(amm.pools.contains_key(&TradingPair::ZEPH_USD));
    }

    #[test]
    fn test_price_calculation() {
        let amm = ZephyrCoinAMM::new(0.10, 1_000_000, 100_000);
        let price = amm.get_current_price(&TradingPair::ZEPH_USD).unwrap();
        assert_eq!(price, 0.10); // 100,000 / 1,000,000
    }

    #[tokio::test]
    async fn test_swap_execution() {
        let mut amm = ZephyrCoinAMM::new(0.10, 1_000_000, 100_000);

        // Buy 1000 ZEPH with USD
        let trade = amm.execute_swap(
            TradingPair::ZEPH_USD,
            100, // 100 USD
            950, // Minimum 950 ZEPH (allowing for slippage)
            true,
        ).unwrap();

        assert!(trade.amount_out >= 950);
        assert!(trade.fee > 0);
    }
}