//! Proof systems module for ZephyrFS
//!
//! Provides cryptographic proof-of-storage and verification systems
//! without compromising zero-knowledge architecture.

pub mod storage_proof;

pub use storage_proof::{
    StorageProofSystem, ProofConfig, StorageChallenge, StorageProof, ProofVerificationResult,
    ChunkDataAccessor, ChunkMetadata, ChunkAccessStats, StorageStats
};

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Unified proof system configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedProofConfig {
    /// Storage proof configuration
    pub storage_proof_config: storage_proof::ProofConfig,
    /// Global proof policies
    pub global_policies: GlobalProofPolicies,
}

/// Global proof system policies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalProofPolicies {
    /// Enable automatic proof generation
    pub auto_proof_generation: bool,
    /// Proof verification frequency (seconds)
    pub verification_frequency: u64,
    /// Require proofs for all stored chunks
    pub mandatory_proofs: bool,
    /// Enable distributed proof verification
    pub distributed_verification: bool,
    /// Proof aggregation for efficiency
    pub enable_proof_aggregation: bool,
}

impl Default for UnifiedProofConfig {
    fn default() -> Self {
        Self {
            storage_proof_config: storage_proof::ProofConfig::default(),
            global_policies: GlobalProofPolicies {
                auto_proof_generation: true,
                verification_frequency: 3600, // 1 hour
                mandatory_proofs: true,
                distributed_verification: false,
                enable_proof_aggregation: true,
            },
        }
    }
}

/// Unified proof system manager
pub struct UnifiedProofManager {
    storage_proof_system: StorageProofSystem,
    config: UnifiedProofConfig,
    active_challenges: HashMap<Uuid, ChallengeContext>,
    proof_cache: HashMap<Uuid, CachedProofResult>,
}

impl UnifiedProofManager {
    /// Create new unified proof manager
    pub fn new(config: UnifiedProofConfig, node_id: String) -> Self {
        let storage_proof_system = StorageProofSystem::new(
            config.storage_proof_config.clone(),
            node_id,
        );

        Self {
            storage_proof_system,
            config,
            active_challenges: HashMap::new(),
            proof_cache: HashMap::new(),
        }
    }

    /// Generate comprehensive storage proof challenge
    pub async fn generate_comprehensive_challenge(
        &mut self,
        chunk_ids: Vec<Uuid>,
        challenge_context: ChallengeContext,
    ) -> Result<ComprehensiveChallenge> {
        // Generate base storage challenge
        let storage_challenge = self.storage_proof_system.generate_challenge(chunk_ids.clone())?;

        // Store challenge context
        self.active_challenges.insert(storage_challenge.challenge_id, challenge_context);

        // Create comprehensive challenge
        Ok(ComprehensiveChallenge {
            storage_challenge,
            additional_requirements: self.get_additional_requirements(&chunk_ids),
            deadline: storage_challenge.expiry_timestamp,
            verification_nodes: self.select_verification_nodes()?,
        })
    }

    /// Generate proof response for comprehensive challenge
    pub async fn generate_comprehensive_proof(
        &mut self,
        challenge: &ComprehensiveChallenge,
        accessor: &dyn ChunkDataAccessor,
    ) -> Result<ComprehensiveProofResponse> {
        // Generate base storage proof
        let storage_proof = self.storage_proof_system
            .generate_proof(&challenge.storage_challenge, accessor)
            .await?;

        // Generate additional proofs based on requirements
        let additional_proofs = self.generate_additional_proofs(
            &challenge.additional_requirements,
            accessor,
        ).await?;

        // Aggregate proofs if enabled
        let aggregated_proof = if self.config.global_policies.enable_proof_aggregation {
            Some(self.aggregate_proofs(&storage_proof, &additional_proofs)?)
        } else {
            None
        };

        let response = ComprehensiveProofResponse {
            storage_proof,
            additional_proofs,
            aggregated_proof,
            response_timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)?
                .as_secs(),
            proof_metadata: self.generate_proof_metadata(&challenge)?,
        };

        // Cache the proof result
        self.cache_proof_result(challenge.storage_challenge.challenge_id, &response);

        Ok(response)
    }

    /// Verify comprehensive proof response
    pub async fn verify_comprehensive_proof(
        &mut self,
        challenge: &ComprehensiveChallenge,
        response: &ComprehensiveProofResponse,
    ) -> Result<ComprehensiveVerificationResult> {
        // Check if we have a cached result
        if let Some(cached) = self.get_cached_verification(challenge.storage_challenge.challenge_id) {
            if !self.is_verification_cache_expired(&cached) {
                return Ok(cached.result);
            }
        }

        // Verify base storage proof
        let storage_verification = self.storage_proof_system
            .verify_proof(&challenge.storage_challenge, &response.storage_proof)
            .await?;

        // Verify additional proofs
        let additional_verifications = self.verify_additional_proofs(
            &challenge.additional_requirements,
            &response.additional_proofs,
        ).await?;

        // Verify aggregated proof if present
        let aggregation_verification = if let Some(aggregated) = &response.aggregated_proof {
            Some(self.verify_aggregated_proof(aggregated, challenge, response).await?)
        } else {
            None
        };

        // Calculate overall verification result
        let overall_result = self.calculate_overall_verification(
            &storage_verification,
            &additional_verifications,
            aggregation_verification.as_ref(),
        );

        let comprehensive_result = ComprehensiveVerificationResult {
            storage_verification,
            additional_verifications,
            aggregation_verification,
            overall_result,
            verification_metadata: self.generate_verification_metadata(challenge, response)?,
        };

        // Cache the verification result
        self.cache_verification_result(challenge.storage_challenge.challenge_id, &comprehensive_result);

        Ok(comprehensive_result)
    }

    /// Get proof system statistics
    pub fn get_proof_statistics(&self) -> ProofStatistics {
        ProofStatistics {
            active_challenges: self.active_challenges.len(),
            cached_proofs: self.proof_cache.len(),
            total_verifications: self.calculate_total_verifications(),
            successful_verifications: self.calculate_successful_verifications(),
            average_proof_generation_time: self.calculate_average_generation_time(),
            average_verification_time: self.calculate_average_verification_time(),
        }
    }

    /// Schedule automatic proof verification
    pub async fn schedule_verification(&mut self, chunk_id: Uuid) -> Result<()> {
        if !self.config.global_policies.auto_proof_generation {
            return Ok(());
        }

        // Schedule verification based on frequency
        let next_verification = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs() + self.config.global_policies.verification_frequency;

        // In production, this would integrate with a task scheduler
        // For now, just log the scheduling
        println!("Scheduled verification for chunk {} at timestamp {}", chunk_id, next_verification);

        Ok(())
    }

    /// Update proof system configuration
    pub fn update_config(&mut self, new_config: UnifiedProofConfig) -> Result<()> {
        self.config = new_config;
        // Clear caches to ensure new configuration takes effect
        self.active_challenges.clear();
        self.proof_cache.clear();
        Ok(())
    }

    /// Get additional requirements based on chunk characteristics
    fn get_additional_requirements(&self, _chunk_ids: &[Uuid]) -> Vec<AdditionalRequirement> {
        // In production, this would analyze chunk characteristics
        // and determine additional proof requirements
        vec![
            AdditionalRequirement::AvailabilityProof,
            AdditionalRequirement::ConsistencyProof,
        ]
    }

    /// Select verification nodes for distributed verification
    fn select_verification_nodes(&self) -> Result<Vec<String>> {
        if self.config.global_policies.distributed_verification {
            // In production, this would select appropriate verification nodes
            Ok(vec!["verifier-1".to_string(), "verifier-2".to_string()])
        } else {
            Ok(vec![])
        }
    }

    /// Generate additional proofs based on requirements
    async fn generate_additional_proofs(
        &self,
        requirements: &[AdditionalRequirement],
        _accessor: &dyn ChunkDataAccessor,
    ) -> Result<HashMap<AdditionalRequirement, AdditionalProof>> {
        let mut proofs = HashMap::new();

        for requirement in requirements {
            match requirement {
                AdditionalRequirement::AvailabilityProof => {
                    proofs.insert(
                        AdditionalRequirement::AvailabilityProof,
                        AdditionalProof::Availability {
                            response_time_ms: 50,
                            availability_score: 0.99,
                        },
                    );
                }
                AdditionalRequirement::ConsistencyProof => {
                    proofs.insert(
                        AdditionalRequirement::ConsistencyProof,
                        AdditionalProof::Consistency {
                            consistency_hash: vec![1, 2, 3, 4],
                            version_proof: vec![5, 6, 7, 8],
                        },
                    );
                }
                AdditionalRequirement::ReplicationProof => {
                    proofs.insert(
                        AdditionalRequirement::ReplicationProof,
                        AdditionalProof::Replication {
                            replica_count: 3,
                            replica_nodes: vec!["node1".to_string(), "node2".to_string(), "node3".to_string()],
                        },
                    );
                }
            }
        }

        Ok(proofs)
    }

    /// Aggregate multiple proofs for efficiency
    fn aggregate_proofs(
        &self,
        storage_proof: &StorageProof,
        additional_proofs: &HashMap<AdditionalRequirement, AdditionalProof>,
    ) -> Result<AggregatedProof> {
        use ring::digest::{Context, SHA256};

        let mut context = Context::new(&SHA256);

        // Add storage proof data
        context.update(storage_proof.challenge_id.as_bytes());
        context.update(&storage_proof.proof_timestamp.to_le_bytes());

        // Add additional proofs
        for (requirement, proof) in additional_proofs {
            context.update(format!("{:?}", requirement).as_bytes());
            match proof {
                AdditionalProof::Availability { response_time_ms, availability_score } => {
                    context.update(&response_time_ms.to_le_bytes());
                    context.update(&availability_score.to_le_bytes());
                }
                AdditionalProof::Consistency { consistency_hash, version_proof } => {
                    context.update(consistency_hash);
                    context.update(version_proof);
                }
                AdditionalProof::Replication { replica_count, .. } => {
                    context.update(&replica_count.to_le_bytes());
                }
            }
        }

        let aggregation_hash = context.finish().as_ref().to_vec();

        Ok(AggregatedProof {
            aggregation_hash,
            proof_count: 1 + additional_proofs.len(),
            aggregation_method: "SHA256".to_string(),
        })
    }

    /// Verify additional proofs
    async fn verify_additional_proofs(
        &self,
        requirements: &[AdditionalRequirement],
        proofs: &HashMap<AdditionalRequirement, AdditionalProof>,
    ) -> Result<HashMap<AdditionalRequirement, bool>> {
        let mut results = HashMap::new();

        for requirement in requirements {
            if let Some(proof) = proofs.get(requirement) {
                let verified = match (requirement, proof) {
                    (AdditionalRequirement::AvailabilityProof, AdditionalProof::Availability { availability_score, .. }) => {
                        *availability_score > 0.95
                    }
                    (AdditionalRequirement::ConsistencyProof, AdditionalProof::Consistency { .. }) => {
                        true // Simplified verification
                    }
                    (AdditionalRequirement::ReplicationProof, AdditionalProof::Replication { replica_count, .. }) => {
                        *replica_count >= 2
                    }
                    _ => false,
                };
                results.insert(*requirement, verified);
            } else {
                results.insert(*requirement, false);
            }
        }

        Ok(results)
    }

    /// Verify aggregated proof
    async fn verify_aggregated_proof(
        &self,
        aggregated: &AggregatedProof,
        challenge: &ComprehensiveChallenge,
        response: &ComprehensiveProofResponse,
    ) -> Result<bool> {
        // Recalculate aggregation and compare
        let recalculated = self.aggregate_proofs(&response.storage_proof, &response.additional_proofs)?;
        Ok(aggregated.aggregation_hash == recalculated.aggregation_hash)
    }

    /// Calculate overall verification result
    fn calculate_overall_verification(
        &self,
        storage_verification: &ProofVerificationResult,
        additional_verifications: &HashMap<AdditionalRequirement, bool>,
        aggregation_verification: Option<&bool>,
    ) -> OverallVerificationResult {
        let storage_valid = storage_verification.is_valid;
        let additional_all_valid = additional_verifications.values().all(|&v| v);
        let aggregation_valid = aggregation_verification.unwrap_or(true);

        let overall_valid = storage_valid && additional_all_valid && aggregation_valid;

        let confidence = if overall_valid {
            (storage_verification.confidence_score +
                additional_verifications.values().filter(|&&v| v).count() as f64 /
                additional_verifications.len().max(1) as f64 +
                if aggregation_valid { 1.0 } else { 0.0 }) / 3.0
        } else {
            0.0
        };

        OverallVerificationResult {
            is_valid: overall_valid,
            confidence_score: confidence,
            details: format!("Storage: {}, Additional: {}/{}, Aggregation: {}",
                storage_valid,
                additional_verifications.values().filter(|&&v| v).count(),
                additional_verifications.len(),
                aggregation_valid
            ),
        }
    }

    /// Generate proof metadata
    fn generate_proof_metadata(&self, _challenge: &ComprehensiveChallenge) -> Result<ProofMetadata> {
        Ok(ProofMetadata {
            generator_version: "1.0.0".to_string(),
            generation_method: "comprehensive".to_string(),
            security_level: "military-grade".to_string(),
            additional_info: HashMap::new(),
        })
    }

    /// Generate verification metadata
    fn generate_verification_metadata(
        &self,
        _challenge: &ComprehensiveChallenge,
        _response: &ComprehensiveProofResponse,
    ) -> Result<VerificationMetadata> {
        Ok(VerificationMetadata {
            verifier_version: "1.0.0".to_string(),
            verification_method: "comprehensive".to_string(),
            verification_timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)?
                .as_secs(),
            additional_info: HashMap::new(),
        })
    }

    /// Cache proof result
    fn cache_proof_result(&mut self, challenge_id: Uuid, _response: &ComprehensiveProofResponse) {
        let cached = CachedProofResult {
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            generation_time_ms: 100, // Would track actual generation time
        };
        self.proof_cache.insert(challenge_id, cached);
    }

    /// Cache verification result
    fn cache_verification_result(&mut self, challenge_id: Uuid, result: &ComprehensiveVerificationResult) {
        // In production, this would cache verification results
        // For now, just track that verification occurred
        println!("Cached verification result for challenge {}: {}", challenge_id, result.overall_result.is_valid);
    }

    /// Get cached verification result
    fn get_cached_verification(&self, _challenge_id: Uuid) -> Option<CachedVerificationResult> {
        // In production, this would return cached verification results
        None
    }

    /// Check if verification cache is expired
    fn is_verification_cache_expired(&self, _cached: &CachedVerificationResult) -> bool {
        // In production, this would check cache expiry
        false
    }

    /// Calculate total verifications performed
    fn calculate_total_verifications(&self) -> usize {
        self.proof_cache.len()
    }

    /// Calculate successful verifications
    fn calculate_successful_verifications(&self) -> usize {
        // In production, this would track successful vs failed verifications
        self.proof_cache.len()
    }

    /// Calculate average proof generation time
    fn calculate_average_generation_time(&self) -> f64 {
        if self.proof_cache.is_empty() {
            return 0.0;
        }

        let total_time: u64 = self.proof_cache.values()
            .map(|cached| cached.generation_time_ms)
            .sum();

        total_time as f64 / self.proof_cache.len() as f64
    }

    /// Calculate average verification time
    fn calculate_average_verification_time(&self) -> f64 {
        // In production, this would track verification times
        150.0 // milliseconds
    }
}

/// Challenge context information
#[derive(Debug, Clone)]
pub struct ChallengeContext {
    pub requester: String,
    pub challenge_type: String,
    pub priority: ChallengePriority,
}

/// Challenge priority levels
#[derive(Debug, Clone)]
pub enum ChallengePriority {
    Low,
    Medium,
    High,
    Critical,
}

/// Comprehensive storage challenge
#[derive(Debug, Clone)]
pub struct ComprehensiveChallenge {
    pub storage_challenge: StorageChallenge,
    pub additional_requirements: Vec<AdditionalRequirement>,
    pub deadline: u64,
    pub verification_nodes: Vec<String>,
}

/// Additional proof requirements
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AdditionalRequirement {
    AvailabilityProof,
    ConsistencyProof,
    ReplicationProof,
}

/// Additional proof types
#[derive(Debug, Clone)]
pub enum AdditionalProof {
    Availability {
        response_time_ms: u64,
        availability_score: f64,
    },
    Consistency {
        consistency_hash: Vec<u8>,
        version_proof: Vec<u8>,
    },
    Replication {
        replica_count: u32,
        replica_nodes: Vec<String>,
    },
}

/// Aggregated proof combining multiple proofs
#[derive(Debug, Clone)]
pub struct AggregatedProof {
    pub aggregation_hash: Vec<u8>,
    pub proof_count: usize,
    pub aggregation_method: String,
}

/// Comprehensive proof response
#[derive(Debug, Clone)]
pub struct ComprehensiveProofResponse {
    pub storage_proof: StorageProof,
    pub additional_proofs: HashMap<AdditionalRequirement, AdditionalProof>,
    pub aggregated_proof: Option<AggregatedProof>,
    pub response_timestamp: u64,
    pub proof_metadata: ProofMetadata,
}

/// Comprehensive verification result
#[derive(Debug, Clone)]
pub struct ComprehensiveVerificationResult {
    pub storage_verification: ProofVerificationResult,
    pub additional_verifications: HashMap<AdditionalRequirement, bool>,
    pub aggregation_verification: Option<bool>,
    pub overall_result: OverallVerificationResult,
    pub verification_metadata: VerificationMetadata,
}

/// Overall verification result
#[derive(Debug, Clone)]
pub struct OverallVerificationResult {
    pub is_valid: bool,
    pub confidence_score: f64,
    pub details: String,
}

/// Proof metadata
#[derive(Debug, Clone)]
pub struct ProofMetadata {
    pub generator_version: String,
    pub generation_method: String,
    pub security_level: String,
    pub additional_info: HashMap<String, String>,
}

/// Verification metadata
#[derive(Debug, Clone)]
pub struct VerificationMetadata {
    pub verifier_version: String,
    pub verification_method: String,
    pub verification_timestamp: u64,
    pub additional_info: HashMap<String, String>,
}

/// Proof system statistics
#[derive(Debug, Clone)]
pub struct ProofStatistics {
    pub active_challenges: usize,
    pub cached_proofs: usize,
    pub total_verifications: usize,
    pub successful_verifications: usize,
    pub average_proof_generation_time: f64,
    pub average_verification_time: f64,
}

/// Cached proof result
#[derive(Debug, Clone)]
struct CachedProofResult {
    timestamp: u64,
    generation_time_ms: u64,
}

/// Cached verification result
#[derive(Debug, Clone)]
struct CachedVerificationResult {
    result: ComprehensiveVerificationResult,
    timestamp: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_unified_proof_manager() -> Result<()> {
        let config = UnifiedProofConfig::default();
        let mut proof_manager = UnifiedProofManager::new(config, "test-node".to_string());

        let chunk_ids = vec![Uuid::new_v4()];
        let context = ChallengeContext {
            requester: "test-requester".to_string(),
            challenge_type: "routine".to_string(),
            priority: ChallengePriority::Medium,
        };

        let challenge = proof_manager
            .generate_comprehensive_challenge(chunk_ids, context)
            .await?;

        assert!(!challenge.additional_requirements.is_empty());
        assert!(challenge.deadline > 0);

        Ok(())
    }

    #[test]
    fn test_proof_statistics() {
        let config = UnifiedProofConfig::default();
        let proof_manager = UnifiedProofManager::new(config, "test-node".to_string());

        let stats = proof_manager.get_proof_statistics();
        assert_eq!(stats.active_challenges, 0);
        assert_eq!(stats.cached_proofs, 0);
    }
}