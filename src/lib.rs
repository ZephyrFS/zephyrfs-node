//! ZephyrFS Node Library
//!
//! Core library for ZephyrFS distributed P2P storage system.
//! Provides cryptographic primitives, storage management, network protocols,
//! comprehensive security systems, and contribution-based resource allocation
//! with zero-knowledge architecture.

pub mod config;
pub mod network;
pub mod storage;
pub mod protocol;
pub mod node_manager;
pub mod crypto;
pub mod coordinator;

// Phase 4.3: Enhanced Security & Malicious Content Protection
pub mod security;
pub mod verification;
pub mod audit;
pub mod proof;

/// Serializable wrapper for std::time::Instant
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct SerializableInstant {
    inner: std::time::Instant,
}

impl SerializableInstant {
    pub fn now() -> Self {
        Self {
            inner: std::time::Instant::now(),
        }
    }

    pub fn elapsed(&self) -> std::time::Duration {
        self.inner.elapsed()
    }

    pub fn duration_since(&self, earlier: Self) -> std::time::Duration {
        self.inner.duration_since(earlier.inner)
    }
}

impl serde::Serialize for SerializableInstant {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        // Serialize as milliseconds since creation (approximate)
        serializer.serialize_u64(0) // Placeholder - in production use proper epoch handling
    }
}

impl<'de> serde::Deserialize<'de> for SerializableInstant {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let _millis: u64 = serde::Deserialize::deserialize(deserializer)?;
        Ok(Self::now()) // Placeholder - in production reconstruct from epoch
    }
}

impl std::ops::Add<std::time::Duration> for SerializableInstant {
    type Output = Self;

    fn add(self, duration: std::time::Duration) -> Self::Output {
        Self {
            inner: self.inner + duration,
        }
    }
}

impl std::ops::Sub<std::time::Duration> for SerializableInstant {
    type Output = Self;

    fn sub(self, duration: std::time::Duration) -> Self::Output {
        Self {
            inner: self.inner - duration,
        }
    }
}

impl std::ops::Sub<SerializableInstant> for SerializableInstant {
    type Output = std::time::Duration;

    fn sub(self, other: SerializableInstant) -> Self::Output {
        self.inner - other.inner
    }
}

// Phase 6.2: Contribution-Based Resource Management
pub mod economics;

// Phase 6.3: Contribution-Based Resource Allocation
pub mod allocation;

// Phase 5.2: Smart Redundancy & Data Durability
pub mod redundancy;

pub use crypto::{
    ZephyrCrypto, CryptoParams, ScryptParams, AesParams, HashParams,
    ContentHasher, VerificationHasher, EncryptedData, ContentId, HashAlgorithm
};

// Core system exports
pub use config::Config;
pub use node_manager::{NodeManager, DistributionStrategy};
pub use storage::{StorageManager, StorageConfig};

// Phase 4.3: Security system exports
pub use security::{
    UnifiedSecurityManager, SecurityConfig, ChunkSecurityDecision, AccessDecision,
    SecurityClearance, ChunkSecurityStatus
};
pub use verification::{
    UnifiedVerificationManager, VerificationConfig, ComprehensiveVerificationResult,
    VerificationRecommendation
};
pub use audit::{
    UnifiedAuditManager, UnifiedAuditConfig, EnhancedTransparencyReport,
    AuditAlert, AlertSeverity
};
pub use proof::{
    UnifiedProofManager, UnifiedProofConfig, ComprehensiveChallenge,
    ComprehensiveVerificationResult as ProofVerificationResult, ProofStatistics
};

// Phase 6.2: Contribution-based economic system exports
pub use economics::{
    ContributionTracker, UserContribution, NetworkContributionStats, PriorityLevel, AccountStatus,
    ContributionEconomicManager, SimpleReferralTracker, ContributionConfig
};
// Phase 6.3: Contribution-based allocation system exports
pub use allocation::{
    ContributionBasedAllocator, AllocationDecision, AllocationRequest, AllocationStrategy, AllocationQuality,
    QualityTierManager, QualityTier, ServiceLevel, TierRequirements, TierBenefits,
    RegionalResourceBalancer, ContributionLoadBalancer, ResourceScheduler
};

// Phase 6.6: Contribution-Based Smart Redundancy System
pub use redundancy::{
    IntelligentReplicationManager, GeographicOptimizer, ChunkHealthMonitor,
    AutoReplicationManager, ReplicationStrategy, GeographicDistribution,
    HealthStatus, ReplicationStatus,
    ContributionNodeSelector, ContributionReplicationManager, NodeContribution, NodeReliability
};