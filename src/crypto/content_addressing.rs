//! Content addressing system for ZephyrFS
//!
//! Provides cryptographic content identifiers and verification using Blake3 and SHA-256.
//! Content IDs are used for integrity verification and deduplication.

use crate::crypto::{ContentHasher, VerificationHasher};
use blake3::Hasher as Blake3Hasher;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fmt;

/// Content identifier - cryptographic hash of data
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ContentId {
    /// Hash algorithm used
    pub algorithm: HashAlgorithm,
    /// Hash bytes
    pub hash: Vec<u8>,
}

/// Supported hash algorithms
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HashAlgorithm {
    Blake3,
    Sha256,
}

impl ContentId {
    /// Generate content ID from data
    pub fn generate(data: &[u8], hasher: &ContentHasher) -> Self {
        match hasher {
            ContentHasher::Blake3 => {
                let mut hasher = Blake3Hasher::new();
                hasher.update(data);
                let hash = hasher.finalize();
                
                Self {
                    algorithm: HashAlgorithm::Blake3,
                    hash: hash.as_bytes().to_vec(),
                }
            }
            ContentHasher::Sha256 => {
                let mut hasher = Sha256::new();
                hasher.update(data);
                let hash = hasher.finalize();
                
                Self {
                    algorithm: HashAlgorithm::Sha256,
                    hash: hash.to_vec(),
                }
            }
        }
    }

    /// Create from existing hash bytes and algorithm
    pub fn from_hash(algorithm: HashAlgorithm, hash: Vec<u8>) -> Self {
        Self { algorithm, hash }
    }

    /// Get hash bytes
    pub fn hash_bytes(&self) -> &[u8] {
        &self.hash
    }

    /// Get algorithm used
    pub fn algorithm(&self) -> &HashAlgorithm {
        &self.algorithm
    }

    /// Convert to hex string representation
    pub fn to_hex(&self) -> String {
        hex::encode(&self.hash)
    }

    /// Create from hex string
    pub fn from_hex(algorithm: HashAlgorithm, hex_str: &str) -> Result<Self, hex::FromHexError> {
        let hash = hex::decode(hex_str)?;
        Ok(Self { algorithm, hash })
    }

    /// Get expected hash length for algorithm
    pub fn expected_length(&self) -> usize {
        match self.algorithm {
            HashAlgorithm::Blake3 => 32,  // Blake3 output is 32 bytes
            HashAlgorithm::Sha256 => 32,  // SHA-256 output is 32 bytes
        }
    }

    /// Validate hash length matches algorithm
    pub fn is_valid(&self) -> bool {
        self.hash.len() == self.expected_length()
    }

    /// Create a multihash-style prefix for the content ID
    pub fn multihash_prefix(&self) -> Vec<u8> {
        let mut prefix = Vec::new();
        
        // Add algorithm identifier
        match self.algorithm {
            HashAlgorithm::Blake3 => {
                prefix.push(0x1e); // Blake3 multicodec
                prefix.push(32);   // Hash length
            }
            HashAlgorithm::Sha256 => {
                prefix.push(0x12); // SHA-256 multicodec
                prefix.push(32);   // Hash length
            }
        }
        
        prefix.extend_from_slice(&self.hash);
        prefix
    }

    /// Parse from multihash format
    pub fn from_multihash(data: &[u8]) -> Result<Self, String> {
        if data.len() < 2 {
            return Err("Multihash too short".to_string());
        }

        let algorithm = match data[0] {
            0x1e => HashAlgorithm::Blake3,
            0x12 => HashAlgorithm::Sha256,
            _ => return Err(format!("Unsupported hash algorithm: {}", data[0])),
        };

        let expected_len = data[1] as usize;
        if data.len() != expected_len + 2 {
            return Err(format!(
                "Invalid multihash length: expected {}, got {}",
                expected_len + 2,
                data.len()
            ));
        }

        let hash = data[2..].to_vec();
        let content_id = Self { algorithm, hash };

        if !content_id.is_valid() {
            return Err("Invalid hash length for algorithm".to_string());
        }

        Ok(content_id)
    }
}

impl fmt::Display for ContentId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", 
            match self.algorithm {
                HashAlgorithm::Blake3 => "blake3",
                HashAlgorithm::Sha256 => "sha256",
            },
            self.to_hex()
        )
    }
}

/// Content verification utilities
pub struct ContentVerifier;

impl ContentVerifier {
    /// Verify data matches expected content ID
    pub fn verify(data: &[u8], expected_id: &ContentId, hasher: &VerificationHasher) -> bool {
        let computed_id = match hasher {
            VerificationHasher::Blake3 => ContentId::generate(data, &ContentHasher::Blake3),
            VerificationHasher::Sha256 => ContentId::generate(data, &ContentHasher::Sha256),
        };

        // Must use same algorithm and hash must match
        computed_id.algorithm == expected_id.algorithm && 
        computed_id.hash == expected_id.hash
    }

    /// Verify data with multiple hash algorithms (for maximum security)
    pub fn verify_multi(data: &[u8], expected_ids: &[ContentId]) -> bool {
        for expected_id in expected_ids {
            let hasher = match expected_id.algorithm {
                HashAlgorithm::Blake3 => VerificationHasher::Blake3,
                HashAlgorithm::Sha256 => VerificationHasher::Sha256,
            };

            if !Self::verify(data, expected_id, &hasher) {
                return false;
            }
        }
        true
    }

    /// Compute multiple hashes for data (Blake3 + SHA-256 for maximum security)
    pub fn compute_multi_hash(data: &[u8]) -> Vec<ContentId> {
        vec![
            ContentId::generate(data, &ContentHasher::Blake3),
            ContentId::generate(data, &ContentHasher::Sha256),
        ]
    }

    /// Create integrity proof for data chunk
    pub fn create_integrity_proof(data: &[u8]) -> IntegrityProof {
        IntegrityProof {
            blake3_hash: ContentId::generate(data, &ContentHasher::Blake3),
            sha256_hash: ContentId::generate(data, &ContentHasher::Sha256),
            data_length: data.len() as u64,
        }
    }

    /// Verify data against integrity proof
    pub fn verify_integrity_proof(data: &[u8], proof: &IntegrityProof) -> bool {
        if data.len() as u64 != proof.data_length {
            return false;
        }

        let blake3_ok = Self::verify(data, &proof.blake3_hash, &VerificationHasher::Blake3);
        let sha256_ok = Self::verify(data, &proof.sha256_hash, &VerificationHasher::Sha256);

        blake3_ok && sha256_ok
    }
}

/// Integrity proof containing multiple hashes for maximum security
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrityProof {
    pub blake3_hash: ContentId,
    pub sha256_hash: ContentId,
    pub data_length: u64,
}

impl IntegrityProof {
    /// Get the primary content ID (Blake3 for performance)
    pub fn primary_id(&self) -> &ContentId {
        &self.blake3_hash
    }

    /// Get the secondary content ID (SHA-256 for compatibility)
    pub fn secondary_id(&self) -> &ContentId {
        &self.sha256_hash
    }

    /// Convert to multihash format (primary hash only)
    pub fn to_multihash(&self) -> Vec<u8> {
        self.blake3_hash.multihash_prefix()
    }
}

/// Content addressing utilities for file chunks
pub struct ChunkAddressing;

impl ChunkAddressing {
    /// Generate content ID for a file chunk with metadata
    pub fn chunk_id(chunk_data: &[u8], chunk_index: u64, file_id: &[u8]) -> ContentId {
        let mut hasher = Blake3Hasher::new();
        
        // Include chunk metadata in hash for uniqueness
        hasher.update(b"ZephyrFS-chunk-v1:");
        hasher.update(&chunk_index.to_be_bytes());
        hasher.update(b":");
        hasher.update(file_id);
        hasher.update(b":");
        hasher.update(chunk_data);
        
        let hash = hasher.finalize();
        ContentId {
            algorithm: HashAlgorithm::Blake3,
            hash: hash.as_bytes().to_vec(),
        }
    }

    /// Generate file-level content ID from chunk IDs
    pub fn file_id_from_chunks(chunk_ids: &[ContentId]) -> ContentId {
        let mut hasher = Blake3Hasher::new();
        hasher.update(b"ZephyrFS-file-v1:");
        
        for chunk_id in chunk_ids {
            hasher.update(&chunk_id.hash);
        }
        
        let hash = hasher.finalize();
        ContentId {
            algorithm: HashAlgorithm::Blake3,
            hash: hash.as_bytes().to_vec(),
        }
    }

    /// Create Merkle tree root from chunk hashes
    pub fn merkle_root(chunk_ids: &[ContentId]) -> ContentId {
        if chunk_ids.is_empty() {
            // Empty file hash
            return ContentId::generate(b"", &ContentHasher::Blake3);
        }

        if chunk_ids.len() == 1 {
            return chunk_ids[0].clone();
        }

        // Build Merkle tree bottom-up
        let mut level: Vec<ContentId> = chunk_ids.to_vec();
        
        while level.len() > 1 {
            let mut next_level = Vec::new();
            
            for pair in level.chunks(2) {
                let mut hasher = Blake3Hasher::new();
                hasher.update(b"ZephyrFS-merkle-v1:");
                hasher.update(&pair[0].hash);
                
                if pair.len() == 2 {
                    hasher.update(&pair[1].hash);
                } else {
                    // Odd number of nodes - hash with itself
                    hasher.update(&pair[0].hash);
                }
                
                let hash = hasher.finalize();
                next_level.push(ContentId {
                    algorithm: HashAlgorithm::Blake3,
                    hash: hash.as_bytes().to_vec(),
                });
            }
            
            level = next_level;
        }
        
        level.into_iter().next().unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_content_id_generation() {
        let data = b"Hello, ZephyrFS!";
        
        let blake3_id = ContentId::generate(data, &ContentHasher::Blake3);
        let sha256_id = ContentId::generate(data, &ContentHasher::Sha256);

        assert_eq!(blake3_id.algorithm, HashAlgorithm::Blake3);
        assert_eq!(sha256_id.algorithm, HashAlgorithm::Sha256);
        assert_eq!(blake3_id.hash.len(), 32);
        assert_eq!(sha256_id.hash.len(), 32);
        assert_ne!(blake3_id.hash, sha256_id.hash);
    }

    #[test]
    fn test_content_verification() {
        let data = b"Test data for verification";
        let content_id = ContentId::generate(data, &ContentHasher::Blake3);

        // Correct data should verify
        assert!(ContentVerifier::verify(data, &content_id, &VerificationHasher::Blake3));

        // Wrong data should not verify
        let wrong_data = b"Wrong data";
        assert!(!ContentVerifier::verify(wrong_data, &content_id, &VerificationHasher::Blake3));
    }

    #[test]
    fn test_content_id_hex_serialization() {
        let data = b"Serialization test";
        let original_id = ContentId::generate(data, &ContentHasher::Blake3);
        
        let hex_str = original_id.to_hex();
        let restored_id = ContentId::from_hex(HashAlgorithm::Blake3, &hex_str).unwrap();
        
        assert_eq!(original_id, restored_id);
    }

    #[test]
    fn test_multihash_format() {
        let data = b"Multihash test data";
        let content_id = ContentId::generate(data, &ContentHasher::Blake3);
        
        let multihash = content_id.multihash_prefix();
        let restored_id = ContentId::from_multihash(&multihash).unwrap();
        
        assert_eq!(content_id, restored_id);
        
        // Test SHA-256 as well
        let sha256_id = ContentId::generate(data, &ContentHasher::Sha256);
        let sha256_multihash = sha256_id.multihash_prefix();
        let restored_sha256 = ContentId::from_multihash(&sha256_multihash).unwrap();
        
        assert_eq!(sha256_id, restored_sha256);
    }

    #[test]
    fn test_integrity_proof() {
        let data = b"Integrity proof test data";
        let proof = ContentVerifier::create_integrity_proof(data);
        
        assert_eq!(proof.data_length, data.len() as u64);
        assert_eq!(proof.blake3_hash.algorithm, HashAlgorithm::Blake3);
        assert_eq!(proof.sha256_hash.algorithm, HashAlgorithm::Sha256);
        
        // Correct data should verify
        assert!(ContentVerifier::verify_integrity_proof(data, &proof));
        
        // Wrong data should not verify
        let wrong_data = b"Wrong data";
        assert!(!ContentVerifier::verify_integrity_proof(wrong_data, &proof));
    }

    #[test]
    fn test_chunk_addressing() {
        let chunk_data = b"Chunk data for addressing test";
        let file_id = b"test_file_12345";
        
        let chunk_id = ChunkAddressing::chunk_id(chunk_data, 0, file_id);
        assert_eq!(chunk_id.algorithm, HashAlgorithm::Blake3);
        assert_eq!(chunk_id.hash.len(), 32);
        
        // Different chunk index should produce different ID
        let chunk_id2 = ChunkAddressing::chunk_id(chunk_data, 1, file_id);
        assert_ne!(chunk_id, chunk_id2);
        
        // Different file ID should produce different ID
        let chunk_id3 = ChunkAddressing::chunk_id(chunk_data, 0, b"different_file");
        assert_ne!(chunk_id, chunk_id3);
    }

    #[test]
    fn test_file_id_from_chunks() {
        let chunk1 = ContentId::generate(b"chunk 1", &ContentHasher::Blake3);
        let chunk2 = ContentId::generate(b"chunk 2", &ContentHasher::Blake3);
        let chunk3 = ContentId::generate(b"chunk 3", &ContentHasher::Blake3);
        
        let file_id = ChunkAddressing::file_id_from_chunks(&[chunk1.clone(), chunk2.clone(), chunk3.clone()]);
        assert_eq!(file_id.algorithm, HashAlgorithm::Blake3);
        
        // Different order should produce different file ID
        let file_id2 = ChunkAddressing::file_id_from_chunks(&[chunk3, chunk1, chunk2]);
        assert_ne!(file_id, file_id2);
    }

    #[test]
    fn test_merkle_root() {
        let chunk1 = ContentId::generate(b"chunk 1", &ContentHasher::Blake3);
        let chunk2 = ContentId::generate(b"chunk 2", &ContentHasher::Blake3);
        let chunk3 = ContentId::generate(b"chunk 3", &ContentHasher::Blake3);
        let chunk4 = ContentId::generate(b"chunk 4", &ContentHasher::Blake3);
        
        // Single chunk
        let root1 = ChunkAddressing::merkle_root(&[chunk1.clone()]);
        assert_eq!(root1, chunk1);
        
        // Multiple chunks
        let root2 = ChunkAddressing::merkle_root(&[chunk1.clone(), chunk2.clone()]);
        let root4 = ChunkAddressing::merkle_root(&[chunk1, chunk2, chunk3, chunk4]);
        
        assert_ne!(root2, root4);
        assert_eq!(root2.algorithm, HashAlgorithm::Blake3);
        assert_eq!(root4.algorithm, HashAlgorithm::Blake3);
        
        // Empty chunks
        let empty_root = ChunkAddressing::merkle_root(&[]);
        assert_eq!(empty_root.algorithm, HashAlgorithm::Blake3);
    }

    #[test]
    fn test_content_id_display() {
        let content_id = ContentId::generate(b"display test", &ContentHasher::Blake3);
        let display_str = format!("{}", content_id);
        
        assert!(display_str.starts_with("blake3:"));
        assert_eq!(display_str.len(), 71); // "blake3:" (7) + hex hash (64)
        
        let sha256_id = ContentId::generate(b"display test", &ContentHasher::Sha256);
        let sha256_str = format!("{}", sha256_id);
        
        assert!(sha256_str.starts_with("sha256:"));
        assert_eq!(sha256_str.len(), 71); // "sha256:" (7) + hex hash (64)
    }

    #[test]
    fn test_multi_hash_verification() {
        let data = b"Multi hash verification test";
        let blake3_id = ContentId::generate(data, &ContentHasher::Blake3);
        let sha256_id = ContentId::generate(data, &ContentHasher::Sha256);
        
        let ids = vec![blake3_id, sha256_id];
        
        // Correct data should verify against all hashes
        assert!(ContentVerifier::verify_multi(data, &ids));
        
        // Wrong data should fail verification
        let wrong_data = b"Wrong data";
        assert!(!ContentVerifier::verify_multi(wrong_data, &ids));
    }
}