//! Dynamic Pricing Engine
//!
//! Real-time storage and bandwidth pricing based on supply and demand

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use tokio::time::{Duration, Instant};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketPrice {
    pub resource_type: ResourceType,
    pub current_price: f64, // ZEPH per unit per hour
    pub base_price: f64,
    pub demand_multiplier: f64,
    pub supply_multiplier: f64,
    pub quality_premium: f64,
    pub regional_adjustment: f64,
    pub timestamp: Instant,
    pub confidence_score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResourceType {
    Storage { size_gb: u64 },
    Bandwidth { mbps: u64 },
    Compute { cpu_cores: u32 },
    NetworkLatency { max_ms: u32 },
    Redundancy { level: u8 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupplyDemandMetrics {
    pub resource_type: ResourceType,
    pub total_supply: f64,
    pub available_supply: f64,
    pub current_demand: f64,
    pub projected_demand: f64,
    pub utilization_rate: f64,
    pub supply_demand_ratio: f64,
    pub market_tension: f64, // 0.0 = oversupply, 1.0 = undersupply
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceHistory {
    pub resource_type: ResourceType,
    pub price_points: VecDeque<PricePoint>,
    pub moving_averages: MovingAverages,
    pub volatility_index: f64,
    pub trend_direction: PriceTrend,
    pub price_elasticity: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PricePoint {
    pub timestamp: Instant,
    pub price: f64,
    pub volume: f64,
    pub supply: f64,
    pub demand: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MovingAverages {
    pub ma_5min: f64,
    pub ma_15min: f64,
    pub ma_1hour: f64,
    pub ma_24hour: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PriceTrend {
    StrongBull,    // Prices rising rapidly
    Bull,          // Prices rising
    Sideways,      // Prices stable
    Bear,          // Prices falling
    StrongBear,    // Prices falling rapidly
}

#[derive(Debug, Clone)]
pub enum PricingModel {
    SupplyDemand,      // Basic supply/demand curves
    Dutch,             // Dutch auction pricing
    Vickrey,           // Second-price sealed-bid
    Continuous,        // Continuous double auction
    Algorithmic,       // ML-based algorithmic pricing
    Hybrid,            // Combination of multiple models
}

pub struct DynamicPricingEngine {
    market_prices: HashMap<String, MarketPrice>,
    price_history: HashMap<String, PriceHistory>,
    supply_demand_cache: HashMap<String, SupplyDemandMetrics>,
    pricing_models: HashMap<ResourceType, PricingModel>,
    base_rates: BaseRateConfiguration,
    market_makers: Vec<MarketMaker>,
    price_bounds: PriceBounds,
    update_frequency: Duration,
}

#[derive(Debug, Clone)]
struct BaseRateConfiguration {
    storage_per_gb_per_hour: f64,     // 0.001 ZEPH/GB/hour
    bandwidth_per_mbps_per_hour: f64, // 0.01 ZEPH/Mbps/hour
    compute_per_core_per_hour: f64,   // 0.1 ZEPH/core/hour
    latency_premium_per_ms: f64,      // 0.0001 ZEPH/ms reduction
    redundancy_multiplier: f64,       // 1.5x per redundancy level
}

struct MarketMaker {
    node_id: String,
    resource_capacity: HashMap<ResourceType, f64>,
    current_utilization: HashMap<ResourceType, f64>,
    pricing_strategy: MarketMakingStrategy,
    profit_margin: f64,
    minimum_price: f64,
    maximum_price: f64,
}

#[derive(Debug, Clone)]
enum MarketMakingStrategy {
    Conservative,  // Stable pricing, low risk
    Aggressive,    // Dynamic pricing, high profit potential
    Balanced,      // Moderate pricing adjustments
    Opportunistic, // Price based on market conditions
}

#[derive(Debug, Clone)]
struct PriceBounds {
    min_multiplier: f64, // 0.1x base price minimum
    max_multiplier: f64, // 10x base price maximum
    volatility_limit: f64, // Max 50% price change per update
    emergency_ceiling: f64, // Hard cap for crisis situations
}

impl DynamicPricingEngine {
    pub fn new() -> Self {
        Self {
            market_prices: HashMap::new(),
            price_history: HashMap::new(),
            supply_demand_cache: HashMap::new(),
            pricing_models: Self::initialize_pricing_models(),
            base_rates: BaseRateConfiguration::default(),
            market_makers: Vec::new(),
            price_bounds: PriceBounds::default(),
            update_frequency: Duration::from_secs(60), // 1-minute updates
        }
    }

    pub async fn update_market_prices(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let resource_types = self.get_active_resource_types();

        for resource_type in resource_types {
            let supply_demand = self.calculate_supply_demand(&resource_type).await?;
            let new_price = self.calculate_optimal_price(&resource_type, &supply_demand).await?;

            self.update_price_history(&resource_type, &new_price);
            self.market_prices.insert(
                self.resource_type_key(&resource_type),
                new_price,
            );
            self.supply_demand_cache.insert(
                self.resource_type_key(&resource_type),
                supply_demand,
            );
        }

        self.rebalance_market_makers().await?;

        Ok(())
    }

    pub fn get_current_price(&self, resource_type: &ResourceType) -> Option<&MarketPrice> {
        self.market_prices.get(&self.resource_type_key(resource_type))
    }

    pub fn get_price_history(&self, resource_type: &ResourceType) -> Option<&PriceHistory> {
        self.price_history.get(&self.resource_type_key(resource_type))
    }

    pub async fn quote_storage_price(
        &self,
        size_gb: u64,
        duration_hours: u64,
        quality_tier: QualityTier,
        region: &str,
    ) -> Result<PriceQuote, Box<dyn std::error::Error>> {
        let resource_type = ResourceType::Storage { size_gb };
        let base_price = self.get_current_price(&resource_type)
            .map(|p| p.current_price)
            .unwrap_or(self.base_rates.storage_per_gb_per_hour);

        let quality_multiplier = match quality_tier {
            QualityTier::Economy => 0.8,
            QualityTier::Standard => 1.0,
            QualityTier::Premium => 1.5,
            QualityTier::Enterprise => 2.0,
        };

        let regional_multiplier = self.get_regional_price_multiplier(region).await;
        let volume_discount = self.calculate_volume_discount(size_gb as f64);

        let unit_price = base_price * quality_multiplier * regional_multiplier * volume_discount;
        let total_price = unit_price * size_gb as f64 * duration_hours as f64;

        Ok(PriceQuote {
            resource_type,
            unit_price,
            total_price,
            currency: "ZEPH".to_string(),
            valid_until: Instant::now() + Duration::from_secs(300), // 5-minute validity
            breakdown: PriceBreakdown {
                base_price,
                quality_multiplier,
                regional_multiplier,
                volume_discount,
                estimated_fees: total_price * 0.02, // 2% network fees
            },
        })
    }

    pub async fn quote_bandwidth_price(
        &self,
        mbps: u64,
        duration_hours: u64,
        latency_requirement: Option<u32>,
        region: &str,
    ) -> Result<PriceQuote, Box<dyn std::error::Error>> {
        let resource_type = ResourceType::Bandwidth { mbps };
        let base_price = self.get_current_price(&resource_type)
            .map(|p| p.current_price)
            .unwrap_or(self.base_rates.bandwidth_per_mbps_per_hour);

        let latency_premium = if let Some(max_ms) = latency_requirement {
            let premium_rate = self.base_rates.latency_premium_per_ms;
            let base_latency = 100.0; // 100ms baseline
            if (max_ms as f64) < base_latency {
                (base_latency - max_ms as f64) * premium_rate
            } else {
                0.0
            }
        } else {
            0.0
        };

        let regional_multiplier = self.get_regional_price_multiplier(region).await;
        let peak_time_multiplier = self.get_peak_time_multiplier().await;

        let unit_price = (base_price + latency_premium) * regional_multiplier * peak_time_multiplier;
        let total_price = unit_price * mbps as f64 * duration_hours as f64;

        Ok(PriceQuote {
            resource_type,
            unit_price,
            total_price,
            currency: "ZEPH".to_string(),
            valid_until: Instant::now() + Duration::from_secs(300),
            breakdown: PriceBreakdown {
                base_price,
                quality_multiplier: peak_time_multiplier,
                regional_multiplier,
                volume_discount: 1.0,
                estimated_fees: total_price * 0.02,
            },
        })
    }

    async fn calculate_supply_demand(&mut self, resource_type: &ResourceType) -> Result<SupplyDemandMetrics, Box<dyn std::error::Error>> {
        // Aggregate supply from all market makers
        let total_supply = self.market_makers.iter()
            .filter_map(|mm| mm.resource_capacity.get(resource_type))
            .sum();

        let current_utilization: f64 = self.market_makers.iter()
            .filter_map(|mm| mm.current_utilization.get(resource_type))
            .sum();

        let available_supply = total_supply - current_utilization;

        // Calculate demand metrics (placeholder - would use real demand data)
        let current_demand = current_utilization * 1.2; // 20% buffer
        let projected_demand = self.project_future_demand(resource_type).await;

        let utilization_rate = if total_supply > 0.0 { current_utilization / total_supply } else { 0.0 };
        let supply_demand_ratio = if current_demand > 0.0 { available_supply / current_demand } else { f64::INFINITY };

        // Market tension: 0.0 = oversupply, 1.0 = undersupply
        let market_tension = if supply_demand_ratio > 2.0 { 0.0 }
                            else if supply_demand_ratio > 1.5 { 0.2 }
                            else if supply_demand_ratio > 1.0 { 0.5 }
                            else if supply_demand_ratio > 0.5 { 0.8 }
                            else { 1.0 };

        Ok(SupplyDemandMetrics {
            resource_type: resource_type.clone(),
            total_supply,
            available_supply,
            current_demand,
            projected_demand,
            utilization_rate,
            supply_demand_ratio,
            market_tension,
        })
    }

    async fn calculate_optimal_price(
        &self,
        resource_type: &ResourceType,
        metrics: &SupplyDemandMetrics,
    ) -> Result<MarketPrice, Box<dyn std::error::Error>> {
        let base_price = self.get_base_price_for_resource(resource_type);

        // Supply multiplier: lower supply = higher prices
        let supply_multiplier = if metrics.supply_demand_ratio < 0.5 { 2.0 }
                               else if metrics.supply_demand_ratio < 1.0 { 1.5 }
                               else if metrics.supply_demand_ratio < 2.0 { 1.0 }
                               else { 0.8 };

        // Demand multiplier: higher utilization = higher prices
        let demand_multiplier = 1.0 + (metrics.utilization_rate * 1.5);

        // Market tension adjustment
        let tension_adjustment = 1.0 + (metrics.market_tension * 0.5);

        let calculated_price = base_price * supply_multiplier * demand_multiplier * tension_adjustment;

        // Apply price bounds
        let bounded_price = calculated_price
            .max(base_price * self.price_bounds.min_multiplier)
            .min(base_price * self.price_bounds.max_multiplier);

        // Check for volatility limits
        let final_price = if let Some(previous_price) = self.get_current_price(resource_type) {
            let max_change = previous_price.current_price * self.price_bounds.volatility_limit;
            bounded_price
                .max(previous_price.current_price - max_change)
                .min(previous_price.current_price + max_change)
        } else {
            bounded_price
        };

        Ok(MarketPrice {
            resource_type: resource_type.clone(),
            current_price: final_price,
            base_price,
            demand_multiplier,
            supply_multiplier,
            quality_premium: 0.0,
            regional_adjustment: 1.0,
            timestamp: Instant::now(),
            confidence_score: self.calculate_price_confidence(metrics),
        })
    }

    fn calculate_price_confidence(&self, metrics: &SupplyDemandMetrics) -> f64 {
        let supply_confidence = if metrics.total_supply > 1000.0 { 0.9 }
                               else if metrics.total_supply > 100.0 { 0.7 }
                               else { 0.4 };

        let demand_stability = if metrics.current_demand > 0.0 {
            let demand_variance = (metrics.projected_demand - metrics.current_demand).abs() / metrics.current_demand;
            1.0 - demand_variance.min(1.0)
        } else {
            0.5
        };

        let market_maturity = if metrics.utilization_rate > 0.1 && metrics.utilization_rate < 0.9 { 0.8 } else { 0.6 };

        (supply_confidence + demand_stability + market_maturity) / 3.0
    }

    async fn project_future_demand(&self, resource_type: &ResourceType) -> f64 {
        // Simple trend projection based on historical data
        if let Some(history) = self.price_history.get(&self.resource_type_key(resource_type)) {
            let recent_volumes: Vec<f64> = history.price_points.iter()
                .rev()
                .take(10)
                .map(|point| point.volume)
                .collect();

            if recent_volumes.len() >= 3 {
                let trend = (recent_volumes[0] - recent_volumes[recent_volumes.len() - 1]) / recent_volumes.len() as f64;
                let current_volume = recent_volumes[0];
                return current_volume + trend * 5.0; // Project 5 periods ahead
            }
        }

        // Fallback to current demand * growth factor
        self.supply_demand_cache.get(&self.resource_type_key(resource_type))
            .map(|metrics| metrics.current_demand * 1.1)
            .unwrap_or(100.0)
    }

    fn update_price_history(&mut self, resource_type: &ResourceType, price: &MarketPrice) {
        let key = self.resource_type_key(resource_type);
        let history = self.price_history.entry(key.clone()).or_insert_with(|| PriceHistory {
            resource_type: resource_type.clone(),
            price_points: VecDeque::new(),
            moving_averages: MovingAverages { ma_5min: 0.0, ma_15min: 0.0, ma_1hour: 0.0, ma_24hour: 0.0 },
            volatility_index: 0.0,
            trend_direction: PriceTrend::Sideways,
            price_elasticity: 0.5,
        });

        // Add new price point
        let price_point = PricePoint {
            timestamp: price.timestamp,
            price: price.current_price,
            volume: self.supply_demand_cache.get(&key)
                .map(|metrics| metrics.current_demand)
                .unwrap_or(0.0),
            supply: self.supply_demand_cache.get(&key)
                .map(|metrics| metrics.available_supply)
                .unwrap_or(0.0),
            demand: self.supply_demand_cache.get(&key)
                .map(|metrics| metrics.current_demand)
                .unwrap_or(0.0),
        };

        history.price_points.push_back(price_point);

        // Keep only last 24 hours of data (1440 minutes)
        if history.price_points.len() > 1440 {
            history.price_points.pop_front();
        }

        // Update moving averages
        self.update_moving_averages(history);
        self.update_trend_analysis(history);
    }

    fn update_moving_averages(&self, history: &mut PriceHistory) {
        let prices: Vec<f64> = history.price_points.iter().map(|p| p.price).collect();

        if prices.len() >= 5 {
            history.moving_averages.ma_5min = prices.iter().rev().take(5).sum::<f64>() / 5.0;
        }
        if prices.len() >= 15 {
            history.moving_averages.ma_15min = prices.iter().rev().take(15).sum::<f64>() / 15.0;
        }
        if prices.len() >= 60 {
            history.moving_averages.ma_1hour = prices.iter().rev().take(60).sum::<f64>() / 60.0;
        }
        if prices.len() >= 1440 {
            history.moving_averages.ma_24hour = prices.iter().rev().take(1440).sum::<f64>() / 1440.0;
        }
    }

    fn update_trend_analysis(&self, history: &mut PriceHistory) {
        if history.price_points.len() < 10 {
            return;
        }

        let recent_prices: Vec<f64> = history.price_points.iter()
            .rev()
            .take(20)
            .map(|p| p.price)
            .collect();

        // Calculate price change over the period
        let price_change = (recent_prices[0] - recent_prices[recent_prices.len() - 1]) / recent_prices[recent_prices.len() - 1];

        history.trend_direction = if price_change > 0.1 { PriceTrend::StrongBull }
                                 else if price_change > 0.02 { PriceTrend::Bull }
                                 else if price_change > -0.02 { PriceTrend::Sideways }
                                 else if price_change > -0.1 { PriceTrend::Bear }
                                 else { PriceTrend::StrongBear };

        // Calculate volatility index
        let mean_price = recent_prices.iter().sum::<f64>() / recent_prices.len() as f64;
        let variance = recent_prices.iter()
            .map(|&price| (price - mean_price).powi(2))
            .sum::<f64>() / recent_prices.len() as f64;
        history.volatility_index = variance.sqrt() / mean_price;
    }

    async fn rebalance_market_makers(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        for market_maker in &mut self.market_makers {
            match market_maker.pricing_strategy {
                MarketMakingStrategy::Conservative => {
                    // Adjust prices slowly, maintain stability
                    self.apply_conservative_pricing_updates(market_maker).await?;
                }
                MarketMakingStrategy::Aggressive => {
                    // Quick price adjustments for maximum profit
                    self.apply_aggressive_pricing_updates(market_maker).await?;
                }
                MarketMakingStrategy::Balanced => {
                    // Moderate price adjustments
                    self.apply_balanced_pricing_updates(market_maker).await?;
                }
                MarketMakingStrategy::Opportunistic => {
                    // Price based on market opportunities
                    self.apply_opportunistic_pricing_updates(market_maker).await?;
                }
            }
        }
        Ok(())
    }

    async fn apply_conservative_pricing_updates(&self, _market_maker: &mut MarketMaker) -> Result<(), Box<dyn std::error::Error>> {
        // Conservative pricing: small, stable adjustments
        Ok(())
    }

    async fn apply_aggressive_pricing_updates(&self, _market_maker: &mut MarketMaker) -> Result<(), Box<dyn std::error::Error>> {
        // Aggressive pricing: larger adjustments for profit maximization
        Ok(())
    }

    async fn apply_balanced_pricing_updates(&self, _market_maker: &mut MarketMaker) -> Result<(), Box<dyn std::error::Error>> {
        // Balanced pricing: moderate adjustments
        Ok(())
    }

    async fn apply_opportunistic_pricing_updates(&self, _market_maker: &mut MarketMaker) -> Result<(), Box<dyn std::error::Error>> {
        // Opportunistic pricing: based on market conditions
        Ok(())
    }

    fn get_base_price_for_resource(&self, resource_type: &ResourceType) -> f64 {
        match resource_type {
            ResourceType::Storage { .. } => self.base_rates.storage_per_gb_per_hour,
            ResourceType::Bandwidth { .. } => self.base_rates.bandwidth_per_mbps_per_hour,
            ResourceType::Compute { .. } => self.base_rates.compute_per_core_per_hour,
            ResourceType::NetworkLatency { .. } => self.base_rates.latency_premium_per_ms,
            ResourceType::Redundancy { level } => {
                self.base_rates.storage_per_gb_per_hour * (self.base_rates.redundancy_multiplier * *level as f64)
            }
        }
    }

    async fn get_regional_price_multiplier(&self, region: &str) -> f64 {
        match region {
            "us-east" | "us-west" => 1.0,
            "europe" => 1.1,
            "asia-pacific" => 0.9,
            "south-america" => 0.8,
            "africa" => 0.7,
            "middle-east" => 1.2,
            _ => 1.0,
        }
    }

    async fn get_peak_time_multiplier(&self) -> f64 {
        // Placeholder: would use real time-of-day analysis
        1.0
    }

    fn calculate_volume_discount(&self, volume: f64) -> f64 {
        if volume > 1000.0 { 0.8 }      // 20% discount for > 1TB
        else if volume > 100.0 { 0.9 }  // 10% discount for > 100GB
        else { 1.0 }                    // No discount
    }

    fn get_active_resource_types(&self) -> Vec<ResourceType> {
        vec![
            ResourceType::Storage { size_gb: 1 },
            ResourceType::Bandwidth { mbps: 1 },
            ResourceType::Compute { cpu_cores: 1 },
            ResourceType::NetworkLatency { max_ms: 100 },
            ResourceType::Redundancy { level: 2 },
        ]
    }

    fn resource_type_key(&self, resource_type: &ResourceType) -> String {
        match resource_type {
            ResourceType::Storage { .. } => "storage".to_string(),
            ResourceType::Bandwidth { .. } => "bandwidth".to_string(),
            ResourceType::Compute { .. } => "compute".to_string(),
            ResourceType::NetworkLatency { .. } => "latency".to_string(),
            ResourceType::Redundancy { .. } => "redundancy".to_string(),
        }
    }

    fn initialize_pricing_models() -> HashMap<ResourceType, PricingModel> {
        let mut models = HashMap::new();
        models.insert(ResourceType::Storage { size_gb: 0 }, PricingModel::SupplyDemand);
        models.insert(ResourceType::Bandwidth { mbps: 0 }, PricingModel::Continuous);
        models.insert(ResourceType::Compute { cpu_cores: 0 }, PricingModel::Dutch);
        models.insert(ResourceType::NetworkLatency { max_ms: 0 }, PricingModel::Vickrey);
        models.insert(ResourceType::Redundancy { level: 0 }, PricingModel::Hybrid);
        models
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QualityTier {
    Economy,    // Basic service, lower reliability
    Standard,   // Standard service, good reliability
    Premium,    // High-quality service, high reliability
    Enterprise, // Maximum quality, SLA guarantees
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceQuote {
    pub resource_type: ResourceType,
    pub unit_price: f64,
    pub total_price: f64,
    pub currency: String,
    pub valid_until: Instant,
    pub breakdown: PriceBreakdown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceBreakdown {
    pub base_price: f64,
    pub quality_multiplier: f64,
    pub regional_multiplier: f64,
    pub volume_discount: f64,
    pub estimated_fees: f64,
}

impl Default for BaseRateConfiguration {
    fn default() -> Self {
        Self {
            storage_per_gb_per_hour: 0.001,
            bandwidth_per_mbps_per_hour: 0.01,
            compute_per_core_per_hour: 0.1,
            latency_premium_per_ms: 0.0001,
            redundancy_multiplier: 1.5,
        }
    }
}

impl Default for PriceBounds {
    fn default() -> Self {
        Self {
            min_multiplier: 0.1,
            max_multiplier: 10.0,
            volatility_limit: 0.5,
            emergency_ceiling: 100.0,
        }
    }
}