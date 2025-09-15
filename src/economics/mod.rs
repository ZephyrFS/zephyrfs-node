//! Economics Module
//!
//! Contribution-based resource allocation and management system for ZephyrFS.
//! Provides fair, cooperative resource sharing based on storage contributions
//! rather than monetary payments.

pub mod contribution_tracker;
pub mod contribution_manager;
pub mod earnings_calculator;

// Core contribution-based resource management exports
pub use contribution_tracker::{
    ContributionTracker, UserContribution, NetworkContributionStats, ContributionConfig,
    PriorityLevel, AccountStatus
};
pub use contribution_manager::{
    ContributionEconomicManager, SimpleReferralTracker
};

// Legacy compatibility exports (for geographic regions and volunteer metrics)
pub use earnings_calculator::{VolunteerMetrics, NetworkHealthMetrics};