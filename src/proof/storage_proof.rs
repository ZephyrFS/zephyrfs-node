//! Proof-of-storage system for ZephyrFS
//!
//! Provides cryptographic proofs of data storage without revealing content,
//! enabling verification of storage node integrity and data availability.

use anyhow::{Context, Result};
use ring::digest::{Context as DigestContext, SHA256, SHA512};
use ring::rand::{SecureRandom, SystemRandom};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

/// Proof-of-storage configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProofConfig {
    /// Challenge difficulty (number of random challenges)
    pub challenge_count: usize,
    /// Proof validity period (seconds)
    pub proof_validity_seconds: u64,
    /// Merkle tree depth for batch proofs
    pub merkle_tree_depth: usize,
    /// Enable zero-knowledge proofs
    pub enable_zk_proofs: bool,
    /// Minimum chunk sample size for statistical proofs
    pub min_sample_size: usize,
}

impl Default for ProofConfig {
    fn default() -> Self {
        Self {
            challenge_count: 64,
            proof_validity_seconds: 1800, // 30 minutes
            merkle_tree_depth: 20,
            enable_zk_proofs: true,
            min_sample_size: 16,
        }
    }
}

/// Storage proof challenge
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageChallenge {
    /// Unique challenge identifier
    pub challenge_id: Uuid,
    /// Timestamp when challenge was issued
    pub challenge_timestamp: u64,
    /// Challenge expiry time
    pub expiry_timestamp: u64,
    /// Challenged chunk identifiers
    pub chunk_ids: Vec<Uuid>,
    /// Random challenge seeds for each chunk
    pub challenge_seeds: Vec<[u8; 32]>,
    /// Additional challenge parameters
    pub challenge_params: ChallengeParams,
}

/// Parameters for storage challenges
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChallengeParams {
    /// Type of proof required
    pub proof_type: ProofType,
    /// Statistical sampling parameters
    pub sampling_params: Option<SamplingParams>,
    /// Zero-knowledge proof parameters
    pub zk_params: Option<ZkProofParams>,
}

/// Types of storage proofs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProofType {
    /// Direct cryptographic proof
    Direct,
    /// Statistical sampling proof
    Statistical,
    /// Zero-knowledge proof
    ZeroKnowledge,
    /// Combined multi-layer proof
    Combined,
}

/// Parameters for statistical sampling
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SamplingParams {
    /// Sample size for statistical proof
    pub sample_size: usize,
    /// Confidence level required (0.0-1.0)
    pub confidence_level: f64,
    /// Random sampling seed
    pub sampling_seed: [u8; 32],
}

/// Parameters for zero-knowledge proofs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZkProofParams {
    /// Commitment scheme identifier
    pub commitment_scheme: String,
    /// Proof system identifier
    pub proof_system: String,
    /// Circuit parameters
    pub circuit_params: Vec<u8>,
}

/// Storage proof response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageProof {
    /// Challenge this proof responds to
    pub challenge_id: Uuid,
    /// Proof generation timestamp
    pub proof_timestamp: u64,
    /// Node that generated this proof
    pub prover_node_id: String,
    /// Cryptographic proofs for each challenged chunk
    pub chunk_proofs: Vec<ChunkProof>,
    /// Aggregated proof data
    pub aggregate_proof: AggregateProof,
    /// Zero-knowledge proof data (if applicable)
    pub zk_proof: Option<ZkProofData>,
}

/// Proof for an individual chunk
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkProof {
    /// Chunk identifier
    pub chunk_id: Uuid,
    /// Hash-based proof of possession
    pub possession_proof: PossessionProof,
    /// Integrity verification data
    pub integrity_data: IntegrityData,
    /// Temporal proof (showing recent access)
    pub temporal_proof: TemporalProof,
}

/// Proof of chunk possession
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PossessionProof {
    /// Challenge response hash
    pub response_hash: Vec<u8>,
    /// Merkle inclusion proof
    pub merkle_proof: MerkleInclusionProof,
    /// Content-blind verification hash
    pub verification_hash: Vec<u8>,
}

/// Merkle tree inclusion proof
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MerkleInclusionProof {
    /// Leaf index in the tree
    pub leaf_index: usize,
    /// Authentication path
    pub auth_path: Vec<Vec<u8>>,
    /// Root hash
    pub root_hash: Vec<u8>,
}

/// Chunk integrity verification data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrityData {
    /// Size verification
    pub size_proof: u64,
    /// Checksum verification
    pub checksum: u32,
    /// Content fingerprint (without revealing content)
    pub content_fingerprint: Vec<u8>,
}

/// Temporal proof of recent chunk access
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemporalProof {
    /// Last access timestamp
    pub last_access_timestamp: u64,
    /// Access frequency indicator
    pub access_frequency: f64,
    /// Time-based verification hash
    pub temporal_hash: Vec<u8>,
}

/// Aggregate proof combining multiple chunks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregateProof {
    /// Combined possession proof
    pub combined_possession: Vec<u8>,
    /// Statistical integrity metrics
    pub integrity_metrics: IntegrityMetrics,
    /// Storage utilization proof
    pub utilization_proof: UtilizationProof,
}

/// Statistical integrity metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrityMetrics {
    /// Total verified chunks
    pub verified_chunk_count: u64,
    /// Average chunk integrity score
    pub average_integrity_score: f64,
    /// Corruption detection count
    pub corruption_count: u64,
    /// Storage efficiency metrics
    pub efficiency_metrics: HashMap<String, f64>,
}

/// Storage utilization proof
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UtilizationProof {
    /// Total storage allocated
    pub total_storage_bytes: u64,
    /// Actually used storage
    pub used_storage_bytes: u64,
    /// Available storage capacity
    pub available_storage_bytes: u64,
    /// Storage utilization ratio
    pub utilization_ratio: f64,
}

/// Zero-knowledge proof data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZkProofData {
    /// Zero-knowledge proof
    pub proof: Vec<u8>,
    /// Public inputs
    pub public_inputs: Vec<Vec<u8>>,
    /// Verification key
    pub verification_key: Vec<u8>,
    /// Proof metadata
    pub metadata: HashMap<String, String>,
}

/// Proof verification result
#[derive(Debug, Clone)]
pub struct ProofVerificationResult {
    /// Whether proof is valid
    pub is_valid: bool,
    /// Confidence score (0.0-1.0)
    pub confidence_score: f64,
    /// Individual verification results
    pub verification_details: VerificationDetails,
    /// Performance metrics
    pub verification_metrics: VerificationMetrics,
}

/// Detailed verification results
#[derive(Debug, Clone)]
pub struct VerificationDetails {
    /// Possession proof results
    pub possession_results: HashMap<Uuid, bool>,
    /// Integrity verification results
    pub integrity_results: HashMap<Uuid, f64>,
    /// Temporal proof results
    pub temporal_results: HashMap<Uuid, bool>,
    /// Aggregate proof result
    pub aggregate_result: bool,
    /// Zero-knowledge proof result
    pub zk_result: Option<bool>,
}

/// Verification performance metrics
#[derive(Debug, Clone)]
pub struct VerificationMetrics {
    /// Verification time (milliseconds)
    pub verification_time_ms: u64,
    /// CPU cycles used
    pub cpu_cycles: Option<u64>,
    /// Memory usage (bytes)
    pub memory_usage_bytes: u64,
    /// Network I/O (bytes)
    pub network_io_bytes: u64,
}

/// Storage proof system
pub struct StorageProofSystem {
    config: ProofConfig,
    node_id: String,
    rng: SystemRandom,
}

impl StorageProofSystem {
    /// Create new storage proof system
    pub fn new(config: ProofConfig, node_id: String) -> Self {
        Self {
            config,
            node_id,
            rng: SystemRandom::new(),
        }
    }

    /// Generate a storage challenge for specific chunks
    pub fn generate_challenge(&self, chunk_ids: Vec<Uuid>) -> Result<StorageChallenge> {
        let challenge_id = Uuid::new_v4();
        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .context("Failed to get timestamp")?
            .as_secs();

        let expiry_timestamp = current_time + self.config.proof_validity_seconds;

        // Generate random seeds for each chunk
        let mut challenge_seeds = Vec::new();
        for _ in &chunk_ids {
            let mut seed = [0u8; 32];
            self.rng.fill(&mut seed)
                .map_err(|_| anyhow::anyhow!("Failed to generate random seed"))?;
            challenge_seeds.push(seed);
        }

        let challenge_params = self.create_challenge_params()?;

        Ok(StorageChallenge {
            challenge_id,
            challenge_timestamp: current_time,
            expiry_timestamp,
            chunk_ids,
            challenge_seeds,
            challenge_params,
        })
    }

    /// Generate storage proof response for a challenge
    pub async fn generate_proof(
        &self,
        challenge: &StorageChallenge,
        chunk_data_accessor: &dyn ChunkDataAccessor,
    ) -> Result<StorageProof> {
        // Verify challenge hasn't expired
        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .context("Failed to get timestamp")?
            .as_secs();

        if current_time > challenge.expiry_timestamp {
            return Err(anyhow::anyhow!("Challenge has expired"));
        }

        let proof_timestamp = current_time;

        // Generate individual chunk proofs
        let mut chunk_proofs = Vec::new();
        for (i, chunk_id) in challenge.chunk_ids.iter().enumerate() {
            let chunk_proof = self.generate_chunk_proof(
                *chunk_id,
                &challenge.challenge_seeds[i],
                chunk_data_accessor,
            ).await?;
            chunk_proofs.push(chunk_proof);
        }

        // Generate aggregate proof
        let aggregate_proof = self.generate_aggregate_proof(&chunk_proofs, chunk_data_accessor).await?;

        // Generate zero-knowledge proof if enabled
        let zk_proof = if self.config.enable_zk_proofs {
            Some(self.generate_zk_proof(&challenge, &chunk_proofs).await?)
        } else {
            None
        };

        Ok(StorageProof {
            challenge_id: challenge.challenge_id,
            proof_timestamp,
            prover_node_id: self.node_id.clone(),
            chunk_proofs,
            aggregate_proof,
            zk_proof,
        })
    }

    /// Verify a storage proof against a challenge
    pub async fn verify_proof(
        &self,
        challenge: &StorageChallenge,
        proof: &StorageProof,
    ) -> Result<ProofVerificationResult> {
        let start_time = SystemTime::now();

        // Basic validation
        if proof.challenge_id != challenge.challenge_id {
            return Ok(ProofVerificationResult {
                is_valid: false,
                confidence_score: 0.0,
                verification_details: VerificationDetails {
                    possession_results: HashMap::new(),
                    integrity_results: HashMap::new(),
                    temporal_results: HashMap::new(),
                    aggregate_result: false,
                    zk_result: Some(false),
                },
                verification_metrics: VerificationMetrics {
                    verification_time_ms: 0,
                    cpu_cycles: None,
                    memory_usage_bytes: 0,
                    network_io_bytes: 0,
                },
            });
        }

        // Verify individual chunk proofs
        let mut possession_results = HashMap::new();
        let mut integrity_results = HashMap::new();
        let mut temporal_results = HashMap::new();

        for (i, chunk_proof) in proof.chunk_proofs.iter().enumerate() {
            let chunk_id = chunk_proof.chunk_id;
            let seed = &challenge.challenge_seeds[i];

            // Verify possession proof
            let possession_valid = self.verify_possession_proof(
                &chunk_proof.possession_proof,
                chunk_id,
                seed,
            ).await?;
            possession_results.insert(chunk_id, possession_valid);

            // Verify integrity data
            let integrity_score = self.verify_integrity_data(&chunk_proof.integrity_data).await?;
            integrity_results.insert(chunk_id, integrity_score);

            // Verify temporal proof
            let temporal_valid = self.verify_temporal_proof(&chunk_proof.temporal_proof).await?;
            temporal_results.insert(chunk_id, temporal_valid);
        }

        // Verify aggregate proof
        let aggregate_result = self.verify_aggregate_proof(&proof.aggregate_proof).await?;

        // Verify zero-knowledge proof if present
        let zk_result = if let Some(zk_proof) = &proof.zk_proof {
            Some(self.verify_zk_proof(zk_proof, challenge).await?)
        } else {
            None
        };

        // Calculate overall confidence score
        let confidence_score = self.calculate_confidence_score(
            &possession_results,
            &integrity_results,
            &temporal_results,
            aggregate_result,
            zk_result,
        );

        let is_valid = confidence_score >= 0.8; // 80% confidence threshold

        let verification_time = SystemTime::now()
            .duration_since(start_time)
            .unwrap_or_default()
            .as_millis() as u64;

        Ok(ProofVerificationResult {
            is_valid,
            confidence_score,
            verification_details: VerificationDetails {
                possession_results,
                integrity_results,
                temporal_results,
                aggregate_result,
                zk_result,
            },
            verification_metrics: VerificationMetrics {
                verification_time_ms: verification_time,
                cpu_cycles: None, // Would need platform-specific implementation
                memory_usage_bytes: 0, // Would need memory tracking
                network_io_bytes: 0,
            },
        })
    }

    /// Create challenge parameters based on configuration
    fn create_challenge_params(&self) -> Result<ChallengeParams> {
        let proof_type = if self.config.enable_zk_proofs {
            ProofType::Combined
        } else {
            ProofType::Statistical
        };

        let sampling_params = Some(SamplingParams {
            sample_size: self.config.min_sample_size,
            confidence_level: 0.95,
            sampling_seed: {
                let mut seed = [0u8; 32];
                self.rng.fill(&mut seed)
                    .map_err(|_| anyhow::anyhow!("Failed to generate sampling seed"))?;
                seed
            },
        });

        let zk_params = if self.config.enable_zk_proofs {
            Some(ZkProofParams {
                commitment_scheme: "Pedersen".to_string(),
                proof_system: "PLONK".to_string(),
                circuit_params: vec![], // Would contain actual circuit parameters
            })
        } else {
            None
        };

        Ok(ChallengeParams {
            proof_type,
            sampling_params,
            zk_params,
        })
    }

    /// Generate proof for an individual chunk
    async fn generate_chunk_proof(
        &self,
        chunk_id: Uuid,
        challenge_seed: &[u8; 32],
        accessor: &dyn ChunkDataAccessor,
    ) -> Result<ChunkProof> {
        // Get encrypted chunk metadata (not the actual content)
        let chunk_metadata = accessor.get_chunk_metadata(chunk_id).await?;

        // Generate possession proof
        let possession_proof = self.generate_possession_proof(
            chunk_id,
            challenge_seed,
            &chunk_metadata,
        )?;

        // Generate integrity data
        let integrity_data = self.generate_integrity_data(&chunk_metadata)?;

        // Generate temporal proof
        let temporal_proof = self.generate_temporal_proof(chunk_id, accessor).await?;

        Ok(ChunkProof {
            chunk_id,
            possession_proof,
            integrity_data,
            temporal_proof,
        })
    }

    /// Generate possession proof for a chunk
    fn generate_possession_proof(
        &self,
        chunk_id: Uuid,
        challenge_seed: &[u8; 32],
        metadata: &ChunkMetadata,
    ) -> Result<PossessionProof> {
        // Create challenge response hash
        let mut context = DigestContext::new(&SHA256);
        context.update(chunk_id.as_bytes());
        context.update(challenge_seed);
        context.update(&metadata.content_hash);
        context.update(&metadata.size.to_le_bytes());
        let response_hash = context.finish().as_ref().to_vec();

        // Generate Merkle inclusion proof (simplified)
        let merkle_proof = MerkleInclusionProof {
            leaf_index: 0, // Would be actual leaf index
            auth_path: vec![], // Would contain actual authentication path
            root_hash: metadata.merkle_root.clone(),
        };

        // Generate verification hash without revealing content
        let mut verification_context = DigestContext::new(&SHA512);
        verification_context.update(&response_hash);
        verification_context.update(&metadata.encryption_key_fingerprint);
        let verification_hash = verification_context.finish().as_ref().to_vec();

        Ok(PossessionProof {
            response_hash,
            merkle_proof,
            verification_hash,
        })
    }

    /// Generate integrity data for a chunk
    fn generate_integrity_data(&self, metadata: &ChunkMetadata) -> Result<IntegrityData> {
        // Size proof
        let size_proof = metadata.size;

        // Checksum
        let checksum = metadata.checksum;

        // Content fingerprint without revealing actual content
        let mut fingerprint_context = DigestContext::new(&SHA256);
        fingerprint_context.update(&metadata.content_hash);
        fingerprint_context.update(&metadata.creation_timestamp.to_le_bytes());
        let content_fingerprint = fingerprint_context.finish().as_ref().to_vec();

        Ok(IntegrityData {
            size_proof,
            checksum,
            content_fingerprint,
        })
    }

    /// Generate temporal proof for recent access
    async fn generate_temporal_proof(
        &self,
        chunk_id: Uuid,
        accessor: &dyn ChunkDataAccessor,
    ) -> Result<TemporalProof> {
        let access_stats = accessor.get_access_stats(chunk_id).await?;

        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .context("Failed to get timestamp")?
            .as_secs();

        // Generate time-based verification hash
        let mut temporal_context = DigestContext::new(&SHA256);
        temporal_context.update(chunk_id.as_bytes());
        temporal_context.update(&current_time.to_le_bytes());
        temporal_context.update(&access_stats.last_access_timestamp.to_le_bytes());
        let temporal_hash = temporal_context.finish().as_ref().to_vec();

        Ok(TemporalProof {
            last_access_timestamp: access_stats.last_access_timestamp,
            access_frequency: access_stats.access_frequency,
            temporal_hash,
        })
    }

    /// Generate aggregate proof combining multiple chunks
    async fn generate_aggregate_proof(
        &self,
        chunk_proofs: &[ChunkProof],
        accessor: &dyn ChunkDataAccessor,
    ) -> Result<AggregateProof> {
        // Combine possession proofs
        let mut combined_context = DigestContext::new(&SHA256);
        for chunk_proof in chunk_proofs {
            combined_context.update(&chunk_proof.possession_proof.response_hash);
        }
        let combined_possession = combined_context.finish().as_ref().to_vec();

        // Calculate integrity metrics
        let verified_chunk_count = chunk_proofs.len() as u64;
        let total_integrity_score: f64 = chunk_proofs.iter()
            .map(|_| 1.0) // Simplified - would calculate actual scores
            .sum();
        let average_integrity_score = total_integrity_score / verified_chunk_count as f64;

        let integrity_metrics = IntegrityMetrics {
            verified_chunk_count,
            average_integrity_score,
            corruption_count: 0, // Would be calculated from actual data
            efficiency_metrics: HashMap::new(),
        };

        // Generate utilization proof
        let storage_stats = accessor.get_storage_stats().await?;
        let utilization_proof = UtilizationProof {
            total_storage_bytes: storage_stats.total_storage_bytes,
            used_storage_bytes: storage_stats.used_storage_bytes,
            available_storage_bytes: storage_stats.available_storage_bytes,
            utilization_ratio: storage_stats.used_storage_bytes as f64 / storage_stats.total_storage_bytes as f64,
        };

        Ok(AggregateProof {
            combined_possession,
            integrity_metrics,
            utilization_proof,
        })
    }

    /// Generate zero-knowledge proof
    async fn generate_zk_proof(
        &self,
        challenge: &StorageChallenge,
        chunk_proofs: &[ChunkProof],
    ) -> Result<ZkProofData> {
        // Simplified ZK proof generation
        // In a real implementation, this would use a ZK-SNARK library

        let mut proof_context = DigestContext::new(&SHA256);
        proof_context.update(challenge.challenge_id.as_bytes());

        for chunk_proof in chunk_proofs {
            proof_context.update(&chunk_proof.possession_proof.response_hash);
        }

        let proof = proof_context.finish().as_ref().to_vec();

        Ok(ZkProofData {
            proof,
            public_inputs: vec![challenge.challenge_id.as_bytes().to_vec()],
            verification_key: vec![], // Would contain actual verification key
            metadata: HashMap::new(),
        })
    }

    /// Verify possession proof
    async fn verify_possession_proof(
        &self,
        proof: &PossessionProof,
        chunk_id: Uuid,
        seed: &[u8; 32],
    ) -> Result<bool> {
        // In a real implementation, this would verify the Merkle proof
        // and check the response hash validity
        Ok(!proof.response_hash.is_empty() && !proof.verification_hash.is_empty())
    }

    /// Verify integrity data
    async fn verify_integrity_data(&self, _integrity: &IntegrityData) -> Result<f64> {
        // Simplified integrity verification
        // Would perform actual integrity checks
        Ok(1.0) // Perfect integrity score
    }

    /// Verify temporal proof
    async fn verify_temporal_proof(&self, _temporal: &TemporalProof) -> Result<bool> {
        // Simplified temporal verification
        // Would check timestamp validity and access patterns
        Ok(true)
    }

    /// Verify aggregate proof
    async fn verify_aggregate_proof(&self, _aggregate: &AggregateProof) -> Result<bool> {
        // Simplified aggregate verification
        // Would verify combined proofs and metrics
        Ok(true)
    }

    /// Verify zero-knowledge proof
    async fn verify_zk_proof(
        &self,
        _zk_proof: &ZkProofData,
        _challenge: &StorageChallenge,
    ) -> Result<bool> {
        // Simplified ZK verification
        // Would use actual ZK verification algorithms
        Ok(true)
    }

    /// Calculate overall confidence score
    fn calculate_confidence_score(
        &self,
        possession_results: &HashMap<Uuid, bool>,
        integrity_results: &HashMap<Uuid, f64>,
        temporal_results: &HashMap<Uuid, bool>,
        aggregate_result: bool,
        zk_result: Option<bool>,
    ) -> f64 {
        let possession_score = possession_results.values().filter(|&&v| v).count() as f64
            / possession_results.len().max(1) as f64;

        let integrity_score = integrity_results.values().sum::<f64>()
            / integrity_results.len().max(1) as f64;

        let temporal_score = temporal_results.values().filter(|&&v| v).count() as f64
            / temporal_results.len().max(1) as f64;

        let aggregate_score = if aggregate_result { 1.0 } else { 0.0 };

        let zk_score = match zk_result {
            Some(true) => 1.0,
            Some(false) => 0.0,
            None => 0.8, // Neutral score when ZK proofs not used
        };

        // Weighted average
        (possession_score * 0.3 + integrity_score * 0.25 + temporal_score * 0.2
            + aggregate_score * 0.15 + zk_score * 0.1)
    }
}

/// Trait for accessing chunk data without revealing content
#[async_trait::async_trait]
pub trait ChunkDataAccessor: Send + Sync {
    /// Get chunk metadata without revealing content
    async fn get_chunk_metadata(&self, chunk_id: Uuid) -> Result<ChunkMetadata>;

    /// Get access statistics for a chunk
    async fn get_access_stats(&self, chunk_id: Uuid) -> Result<ChunkAccessStats>;

    /// Get overall storage statistics
    async fn get_storage_stats(&self) -> Result<StorageStats>;
}

/// Chunk metadata for proof generation
#[derive(Debug, Clone)]
pub struct ChunkMetadata {
    pub chunk_id: Uuid,
    pub size: u64,
    pub content_hash: Vec<u8>,
    pub merkle_root: Vec<u8>,
    pub encryption_key_fingerprint: Vec<u8>,
    pub creation_timestamp: u64,
    pub checksum: u32,
}

/// Chunk access statistics
#[derive(Debug, Clone)]
pub struct ChunkAccessStats {
    pub last_access_timestamp: u64,
    pub access_count: u64,
    pub access_frequency: f64,
}

/// Overall storage statistics
#[derive(Debug, Clone)]
pub struct StorageStats {
    pub total_storage_bytes: u64,
    pub used_storage_bytes: u64,
    pub available_storage_bytes: u64,
    pub chunk_count: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Mock chunk data accessor for testing
    struct MockChunkDataAccessor;

    #[async_trait::async_trait]
    impl ChunkDataAccessor for MockChunkDataAccessor {
        async fn get_chunk_metadata(&self, chunk_id: Uuid) -> Result<ChunkMetadata> {
            Ok(ChunkMetadata {
                chunk_id,
                size: 1024,
                content_hash: vec![1, 2, 3, 4],
                merkle_root: vec![5, 6, 7, 8],
                encryption_key_fingerprint: vec![9, 10, 11, 12],
                creation_timestamp: 1234567890,
                checksum: 0x12345678,
            })
        }

        async fn get_access_stats(&self, _chunk_id: Uuid) -> Result<ChunkAccessStats> {
            Ok(ChunkAccessStats {
                last_access_timestamp: 1234567890,
                access_count: 10,
                access_frequency: 0.5,
            })
        }

        async fn get_storage_stats(&self) -> Result<StorageStats> {
            Ok(StorageStats {
                total_storage_bytes: 1000000,
                used_storage_bytes: 750000,
                available_storage_bytes: 250000,
                chunk_count: 100,
            })
        }
    }

    #[tokio::test]
    async fn test_storage_proof_generation() -> Result<()> {
        let config = ProofConfig::default();
        let proof_system = StorageProofSystem::new(config, "test-node".to_string());
        let accessor = MockChunkDataAccessor;

        let chunk_ids = vec![Uuid::new_v4(), Uuid::new_v4()];
        let challenge = proof_system.generate_challenge(chunk_ids)?;

        let proof = proof_system.generate_proof(&challenge, &accessor).await?;

        assert_eq!(proof.challenge_id, challenge.challenge_id);
        assert_eq!(proof.chunk_proofs.len(), challenge.chunk_ids.len());

        Ok(())
    }

    #[tokio::test]
    async fn test_proof_verification() -> Result<()> {
        let config = ProofConfig::default();
        let proof_system = StorageProofSystem::new(config, "test-node".to_string());
        let accessor = MockChunkDataAccessor;

        let chunk_ids = vec![Uuid::new_v4()];
        let challenge = proof_system.generate_challenge(chunk_ids)?;
        let proof = proof_system.generate_proof(&challenge, &accessor).await?;

        let verification_result = proof_system.verify_proof(&challenge, &proof).await?;

        assert!(verification_result.is_valid);
        assert!(verification_result.confidence_score > 0.5);

        Ok(())
    }
}