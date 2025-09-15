//! Resource Allocation Module
//!
//! Contribution-based resource allocation system for fair and efficient distribution

pub mod contribution_allocator;
pub mod quality_tiers;
pub mod regional_balancer;
pub mod load_optimizer;
pub mod resource_scheduler;

pub use contribution_allocator::{
    ContributionBasedAllocator, AllocationDecision, AllocationRequest,
    AllocationStrategy, AllocationQuality, ResourcePriority
};
pub use quality_tiers::{
    QualityTierManager, QualityTier, ServiceLevel,
    TierRequirements, TierBenefits
};
pub use regional_balancer::{
    RegionalResourceBalancer, RegionalAllocation, RegionalMetrics,
    GeographicDistribution, RegionalPolicy
};
pub use load_optimizer::{
    ContributionLoadBalancer, LoadBalancingDecision, ResourceWeight,
    OptimizedRouting, PerformanceAllocation
};
pub use resource_scheduler::{
    ResourceScheduler, ScheduledAllocation, AllocationSchedule,
    SchedulingPolicy, ResourceReservation
};