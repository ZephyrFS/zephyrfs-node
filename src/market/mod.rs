//! Market Dynamics Module
//!
//! Economic system for fair pricing and efficient resource allocation

pub mod dynamic_pricing;
pub mod quality_service;
pub mod regional_optimizer;
pub mod auction_system;
pub mod sla_manager;
pub mod pricing_oracles;
pub mod load_balancer;
pub mod bandwidth_market;

pub use dynamic_pricing::{
    DynamicPricingEngine, MarketPrice, PriceHistory,
    SupplyDemandMetrics, PricingModel
};
pub use quality_service::{
    QualityOfServiceManager, ServiceTier, ServiceLevel,
    QoSMetrics, TierConfiguration
};
pub use regional_optimizer::{
    RegionalPriceOptimizer, RegionalMarket, PriceAdjustment,
    MarketConditions, GeographicPricing
};
pub use auction_system::{
    ResourceAuctionSystem, StorageAuction, BandwidthAuction,
    AuctionResult, BidSubmission
};
pub use sla_manager::{
    SLAManager, ServiceLevelAgreement, SLAMetrics,
    ComplianceStatus, SLAViolation
};
pub use pricing_oracles::{
    PricingOracleNetwork, PriceOracle, MarketData,
    ExternalPriceSource, OracleConsensus
};
pub use load_balancer::{
    EconomicLoadBalancer, LoadBalancingStrategy, ResourceWeight,
    CostOptimizedRouting, PerformancePricing
};
pub use bandwidth_market::{
    BandwidthMarketplace, BandwidthContract, TrafficShaping,
    QoSPrioritizer, NetworkResourceAllocator
};