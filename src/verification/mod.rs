//! Verification module for ZephyrFS
//!
//! Provides comprehensive content verification and integrity checking
//! without compromising zero-knowledge architecture.

pub mod integrity_checks;

pub use integrity_checks::{
    IntegrityVerifier, IntegrityConfig, IntegrityProof, VerificationResult, VerificationReport
};

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Unified verification configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationConfig {
    /// Integrity verification configuration
    pub integrity_config: integrity_checks::IntegrityConfig,
    /// Global verification policies
    pub global_policies: GlobalVerificationPolicies,
}

/// Global verification policies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalVerificationPolicies {
    /// Require verification for all chunks
    pub mandatory_verification: bool,
    /// Maximum age for cached verification results (seconds)
    pub verification_cache_ttl: u64,
    /// Minimum confidence threshold for acceptance
    pub acceptance_threshold: f64,
    /// Enable periodic re-verification
    pub periodic_reverification: bool,
    /// Re-verification interval (seconds)
    pub reverification_interval: u64,
}

impl Default for VerificationConfig {
    fn default() -> Self {
        Self {
            integrity_config: integrity_checks::IntegrityConfig::default(),
            global_policies: GlobalVerificationPolicies {
                mandatory_verification: true,
                verification_cache_ttl: 3600, // 1 hour
                acceptance_threshold: 0.95,
                periodic_reverification: true,
                reverification_interval: 86400, // 24 hours
            },
        }
    }
}

/// Unified verification manager
pub struct UnifiedVerificationManager {
    integrity_verifier: IntegrityVerifier,
    config: VerificationConfig,
    verification_cache: HashMap<Uuid, CachedVerificationResult>,
}

impl UnifiedVerificationManager {
    /// Create new unified verification manager
    pub fn new(config: VerificationConfig) -> Self {
        let integrity_verifier = IntegrityVerifier::new(config.integrity_config.clone());

        Self {
            integrity_verifier,
            config,
            verification_cache: HashMap::new(),
        }
    }

    /// Verify chunk with comprehensive checks
    pub async fn verify_chunk(
        &mut self,
        chunk_id: Uuid,
        encrypted_content: &[u8],
        stored_proof: Option<&IntegrityProof>,
    ) -> Result<ComprehensiveVerificationResult> {
        // Check cache first
        if let Some(cached_result) = self.get_cached_verification(chunk_id) {
            if !self.is_cache_expired(&cached_result) {
                return Ok(ComprehensiveVerificationResult {
                    chunk_id,
                    verification_result: cached_result.result.clone(),
                    from_cache: true,
                    recommendations: self.generate_recommendations(&cached_result.result),
                });
            }
        }

        // Perform fresh verification
        let verification_result = if let Some(proof) = stored_proof {
            // Verify against existing proof
            self.integrity_verifier.verify_integrity(encrypted_content, proof)?
        } else {
            // Generate new proof and verify
            let new_proof = self.integrity_verifier.generate_proof(encrypted_content)?;
            self.integrity_verifier.verify_integrity(encrypted_content, &new_proof)?
        };

        // Cache the result
        let cached_result = CachedVerificationResult {
            result: verification_result.clone(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)?
                .as_secs(),
        };
        self.verification_cache.insert(chunk_id, cached_result);

        let recommendations = self.generate_recommendations(&verification_result);

        Ok(ComprehensiveVerificationResult {
            chunk_id,
            verification_result,
            from_cache: false,
            recommendations,
        })
    }

    /// Generate integrity proof for chunk
    pub fn generate_integrity_proof(&self, encrypted_content: &[u8]) -> Result<IntegrityProof> {
        self.integrity_verifier.generate_proof(encrypted_content)
    }

    /// Verify integrity proof
    pub fn verify_integrity_proof(
        &self,
        encrypted_content: &[u8],
        proof: &IntegrityProof,
    ) -> Result<VerificationResult> {
        self.integrity_verifier.verify_integrity(encrypted_content, proof)
    }

    /// Check if chunk needs re-verification
    pub fn needs_reverification(&self, chunk_id: Uuid, last_verification: u64) -> bool {
        if !self.config.global_policies.periodic_reverification {
            return false;
        }

        let current_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        current_time - last_verification >= self.config.global_policies.reverification_interval
    }

    /// Get verification statistics
    pub fn get_verification_stats(&self) -> VerificationStats {
        let total_cached = self.verification_cache.len();
        let valid_results = self.verification_cache.values()
            .filter(|result| result.result.is_valid)
            .count();

        VerificationStats {
            total_verifications: total_cached,
            successful_verifications: valid_results,
            failed_verifications: total_cached - valid_results,
            cache_hit_rate: 0.0, // Would need to track cache hits/misses
            average_confidence: self.calculate_average_confidence(),
        }
    }

    /// Update verification configuration
    pub fn update_config(&mut self, new_config: VerificationConfig) {
        self.config = new_config;
        // Clear cache to ensure new policies take effect
        self.verification_cache.clear();
    }

    /// Get cached verification result if valid
    fn get_cached_verification(&self, chunk_id: Uuid) -> Option<&CachedVerificationResult> {
        self.verification_cache.get(&chunk_id)
    }

    /// Check if cached result is expired
    fn is_cache_expired(&self, cached_result: &CachedVerificationResult) -> bool {
        let current_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        current_time - cached_result.timestamp >= self.config.global_policies.verification_cache_ttl
    }

    /// Generate recommendations based on verification results
    fn generate_recommendations(&self, result: &VerificationResult) -> Vec<VerificationRecommendation> {
        let mut recommendations = Vec::new();

        if !result.is_valid {
            recommendations.push(VerificationRecommendation::RequireReUpload);
            recommendations.push(VerificationRecommendation::InvestigateCorruption);
        } else if result.confidence < self.config.global_policies.acceptance_threshold {
            recommendations.push(VerificationRecommendation::RequireAdditionalVerification);

            if result.confidence < 0.8 {
                recommendations.push(VerificationRecommendation::MarkForReview);
            }
        }

        if result.confidence >= self.config.global_policies.acceptance_threshold {
            recommendations.push(VerificationRecommendation::AcceptAsValid);
        }

        recommendations
    }

    /// Calculate average confidence across all cached results
    fn calculate_average_confidence(&self) -> f64 {
        if self.verification_cache.is_empty() {
            return 0.0;
        }

        let total_confidence: f64 = self.verification_cache.values()
            .map(|result| result.result.confidence)
            .sum();

        total_confidence / self.verification_cache.len() as f64
    }
}

/// Cached verification result
#[derive(Debug, Clone)]
struct CachedVerificationResult {
    result: VerificationResult,
    timestamp: u64,
}

/// Comprehensive verification result
#[derive(Debug, Clone)]
pub struct ComprehensiveVerificationResult {
    pub chunk_id: Uuid,
    pub verification_result: VerificationResult,
    pub from_cache: bool,
    pub recommendations: Vec<VerificationRecommendation>,
}

/// Verification recommendations
#[derive(Debug, Clone)]
pub enum VerificationRecommendation {
    AcceptAsValid,
    RequireAdditionalVerification,
    RequireReUpload,
    MarkForReview,
    InvestigateCorruption,
    ScheduleReverification,
}

/// Verification statistics
#[derive(Debug, Clone)]
pub struct VerificationStats {
    pub total_verifications: usize,
    pub successful_verifications: usize,
    pub failed_verifications: usize,
    pub cache_hit_rate: f64,
    pub average_confidence: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_unified_verification_manager() -> Result<()> {
        let config = VerificationConfig::default();
        let mut verification_manager = UnifiedVerificationManager::new(config);

        let chunk_id = Uuid::new_v4();
        let test_data = b"Test encrypted chunk data for verification";

        let result = verification_manager
            .verify_chunk(chunk_id, test_data, None)
            .await?;

        assert_eq!(result.chunk_id, chunk_id);
        assert!(!result.from_cache); // First verification shouldn't be from cache
        assert!(!result.recommendations.is_empty());

        Ok(())
    }

    #[tokio::test]
    async fn test_verification_caching() -> Result<()> {
        let config = VerificationConfig::default();
        let mut verification_manager = UnifiedVerificationManager::new(config);

        let chunk_id = Uuid::new_v4();
        let test_data = b"Test data for caching verification";

        // First verification
        let result1 = verification_manager
            .verify_chunk(chunk_id, test_data, None)
            .await?;
        assert!(!result1.from_cache);

        // Second verification should use cache
        let result2 = verification_manager
            .verify_chunk(chunk_id, test_data, None)
            .await?;
        assert!(result2.from_cache);

        Ok(())
    }

    #[test]
    fn test_verification_stats() {
        let config = VerificationConfig::default();
        let verification_manager = UnifiedVerificationManager::new(config);

        let stats = verification_manager.get_verification_stats();
        assert_eq!(stats.total_verifications, 0);
        assert_eq!(stats.successful_verifications, 0);
        assert_eq!(stats.failed_verifications, 0);
    }
}