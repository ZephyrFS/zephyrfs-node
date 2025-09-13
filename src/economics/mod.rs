//! Economics Module
//!
//! Complete economic system for ZephyrFS including token economics, payments, and rewards

pub mod token_model;
pub mod zephyr_coin;
pub mod network_health_minter;
pub mod market_maker;
pub mod earnings_calculator;
pub mod payment_processor;
pub mod payout_scheduler;
pub mod performance_rewards;

pub use token_model::{TokenEconomicsManager, TokenEconomics, NetworkHealthMetrics, RewardReason};
pub use zephyr_coin::{ZephyrCoin, TokenEvent};
pub use network_health_minter::{NetworkHealthController, HealthBasedMinter};
pub use market_maker::{ZephyrCoinAMM, TradingPair, Currency as AMMCurrency};
pub use earnings_calculator::{EarningsCalculator, VolunteerMetrics, EarningsProjection};
pub use payment_processor::{PaymentProcessor, PaymentRequest, Currency, PaymentMethod};
pub use payout_scheduler::{PayoutScheduler, PayoutPreferences, PayoutFrequency};
pub use performance_rewards::{PerformanceRewardsSystem, PerformanceScore, Achievement, RewardTier};