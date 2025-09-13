//! # ZephyrFS Cryptography Module
//! 
//! Provides zero-knowledge client-side encryption for ZephyrFS distributed storage.
//! Implements Tahoe-LAFS inspired capability-based security with mandatory encryption.
//!
//! ## Architecture
//! - AES-256-GCM encryption for file segments
//! - Scrypt key derivation from user passwords  
//! - Hierarchical deterministic keys for per-segment encryption
//! - Secure memory handling with automatic cleanup
//! - Content addressing with cryptographic verification

pub mod encryption;
pub mod key_derivation;
pub mod secure_memory;
pub mod content_addressing;

pub use encryption::{FileEncryption, SegmentEncryption, EncryptedData};
pub use key_derivation::{KeyDerivation, DerivedKey, KeyHierarchy};
pub use secure_memory::{SecureBytes, SecureString};
pub use content_addressing::{ContentId, ContentVerifier, HashAlgorithm};

use anyhow::Result;
use zeroize::Zeroize;

/// Core cryptographic parameters following Tahoe-LAFS security model
pub struct CryptoParams {
    /// Scrypt parameters: N=2^17, r=8, p=1, output=96 bytes
    pub scrypt_params: ScryptParams,
    /// AES-256-GCM parameters
    pub aes_params: AesParams,
    /// Content addressing parameters
    pub hash_params: HashParams,
}

#[derive(Debug, Clone)]
pub struct ScryptParams {
    pub log_n: u8,      // 17 (N = 2^17 = 131072)
    pub r: u32,         // 8
    pub p: u32,         // 1  
    pub output_len: usize, // 96 bytes (master key + salt + verification)
}

#[derive(Debug, Clone)]
pub struct AesParams {
    pub key_len: usize,    // 32 bytes (AES-256)
    pub nonce_len: usize,  // 12 bytes (GCM standard)
    pub tag_len: usize,    // 16 bytes (GCM authentication tag)
}

#[derive(Debug, Clone)] 
pub struct HashParams {
    pub content_hasher: ContentHasher,
    pub verification_hasher: VerificationHasher,
}

#[derive(Debug, Clone)]
pub enum ContentHasher {
    Blake3,     // Fast hashing for content IDs
    Sha256,     // SHA-256 for compatibility
}

#[derive(Debug, Clone)]
pub enum VerificationHasher {
    Blake3,     // Fast verification
    Sha256,     // Standard verification  
}

impl Default for CryptoParams {
    fn default() -> Self {
        Self {
            scrypt_params: ScryptParams {
                log_n: 17,          // 2^17 iterations (strong security)
                r: 8,               // Memory cost parameter
                p: 1,               // Parallelization parameter  
                output_len: 64,     // 64 bytes (max supported by scrypt crate)
            },
            aes_params: AesParams {
                key_len: 32,        // AES-256
                nonce_len: 12,      // GCM standard nonce size
                tag_len: 16,        // GCM authentication tag size
            },
            hash_params: HashParams {
                content_hasher: ContentHasher::Blake3,
                verification_hasher: VerificationHasher::Blake3,
            },
        }
    }
}

/// High-level cryptographic operations for ZephyrFS
pub struct ZephyrCrypto {
    params: CryptoParams,
    key_hierarchy: Option<KeyHierarchy>,
}

impl ZephyrCrypto {
    /// Create new crypto instance with default parameters
    pub fn new() -> Self {
        Self {
            params: CryptoParams::default(),
            key_hierarchy: None,
        }
    }

    /// Create crypto instance with custom parameters
    pub fn with_params(params: CryptoParams) -> Self {
        Self {
            params,
            key_hierarchy: None,
        }
    }

    /// Initialize key hierarchy from password
    pub fn init_from_password(&mut self, password: &str) -> Result<()> {
        let key_derivation = KeyDerivation::new(&self.params.scrypt_params);
        let derived_key = key_derivation.derive_from_password(password)?;
        self.key_hierarchy = Some(KeyHierarchy::new(derived_key)?);
        Ok(())
    }

    /// Initialize key hierarchy from existing key material  
    pub fn init_from_key(&mut self, key_material: SecureBytes) -> Result<()> {
        let derived_key = DerivedKey::from_bytes(key_material)?;
        self.key_hierarchy = Some(KeyHierarchy::new(derived_key)?);
        Ok(())
    }

    /// Encrypt file data into segments
    pub fn encrypt_file(&self, file_data: &[u8]) -> Result<Vec<EncryptedData>> {
        let hierarchy = self.key_hierarchy.as_ref()
            .ok_or_else(|| anyhow::anyhow!("Key hierarchy not initialized"))?;

        let file_encryption = FileEncryption::new(&self.params.aes_params);
        file_encryption.encrypt_to_segments(file_data, hierarchy)
    }

    /// Decrypt file segments back to original data
    pub fn decrypt_file(&self, encrypted_segments: &[EncryptedData]) -> Result<Vec<u8>> {
        let hierarchy = self.key_hierarchy.as_ref()
            .ok_or_else(|| anyhow::anyhow!("Key hierarchy not initialized"))?;

        let file_encryption = FileEncryption::new(&self.params.aes_params);
        file_encryption.decrypt_from_segments(encrypted_segments, hierarchy)
    }

    /// Generate content ID for data
    pub fn content_id(&self, data: &[u8]) -> ContentId {
        ContentId::generate(data, &self.params.hash_params.content_hasher)
    }

    /// Verify content integrity
    pub fn verify_content(&self, data: &[u8], expected_id: &ContentId) -> bool {
        ContentVerifier::verify(data, expected_id, &self.params.hash_params.verification_hasher)
    }

    /// Get public key material for capability sharing (zero-knowledge proof)
    pub fn get_capability(&self) -> Result<Vec<u8>> {
        let hierarchy = self.key_hierarchy.as_ref()
            .ok_or_else(|| anyhow::anyhow!("Key hierarchy not initialized"))?;
        
        // Return only public/shareable key material, never private keys
        Ok(hierarchy.get_public_capability()?)
    }
}

impl Drop for ZephyrCrypto {
    fn drop(&mut self) {
        // Ensure all sensitive data is cleared when dropping
        if let Some(ref mut hierarchy) = self.key_hierarchy {
            hierarchy.zeroize();
        }
    }
}

impl Zeroize for ZephyrCrypto {
    fn zeroize(&mut self) {
        if let Some(ref mut hierarchy) = self.key_hierarchy {
            hierarchy.zeroize();
        }
        self.key_hierarchy = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crypto_params_default() {
        let params = CryptoParams::default();
        assert_eq!(params.scrypt_params.log_n, 17);
        assert_eq!(params.scrypt_params.r, 8);
        assert_eq!(params.scrypt_params.p, 1);
        assert_eq!(params.aes_params.key_len, 32);
        assert_eq!(params.aes_params.nonce_len, 12);
        assert_eq!(params.aes_params.tag_len, 16);
    }

    #[test]
    fn test_zephyr_crypto_creation() {
        let crypto = ZephyrCrypto::new();
        assert!(crypto.key_hierarchy.is_none());
    }

    #[test] 
    fn test_crypto_init_from_password() {
        let mut crypto = ZephyrCrypto::new();
        let result = crypto.init_from_password("test_password_123");
        assert!(result.is_ok());
        assert!(crypto.key_hierarchy.is_some());
    }
}