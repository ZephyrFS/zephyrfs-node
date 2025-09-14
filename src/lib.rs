//! ZephyrFS Node Library
//!
//! Core library for ZephyrFS distributed P2P storage system.
//! Provides cryptographic primitives, storage management, network protocols,
//! and military-grade security systems with zero-knowledge architecture.

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

// Phase 5.1: Economic Foundation & Token System
pub mod economics;
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

// Phase 5.1: Economic system exports
pub use economics::{
    TokenEconomicsManager, ZephyrCoin, NetworkHealthController, ZephyrCoinAMM,
    EarningsCalculator, PaymentProcessor, PayoutScheduler, PerformanceRewardsSystem
};
pub use allocation::{
    DemocraticAllocationManager, AllocationStrategy, AllocationQuality
};

// Phase 5.2: Smart redundancy system exports
pub use redundancy::{
    IntelligentReplicationManager, GeographicOptimizer, ChunkHealthMonitor,
    AutoReplicationManager, ReplicationStrategy, GeographicDistribution,
    HealthStatus, ReplicationStatus
};