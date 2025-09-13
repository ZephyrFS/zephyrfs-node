//! AES-256-GCM encryption implementation for ZephyrFS
//! 
//! Provides segment-level encryption using AES-256 in GCM mode for authenticated encryption.
//! Each file segment is encrypted with its own derived key for maximum security isolation.

use crate::crypto::{AesParams, KeyHierarchy, SecureBytes};
use anyhow::{Context, Result};
use ring::aead::{Aad, BoundKey, Nonce, NonceSequence, OpeningKey, SealingKey, UnboundKey, AES_256_GCM};
use ring::error::Unspecified;
use ring::rand::{SecureRandom, SystemRandom};
use serde::{Deserialize, Serialize};
use zeroize::{Zeroize, ZeroizeOnDrop};

/// Default segment size for file encryption (1MB)
pub const DEFAULT_SEGMENT_SIZE: usize = 1024 * 1024; // 1MB

/// Encrypted data container with all necessary metadata for decryption
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedData {
    /// Segment index in the original file
    pub segment_index: u64,
    /// Encrypted data payload
    pub ciphertext: Vec<u8>,
    /// Nonce used for this encryption
    pub nonce: [u8; 12],
    /// Additional authenticated data (AAD) - segment metadata
    pub aad: Vec<u8>,
    /// Key derivation path for this segment
    pub key_path: Vec<u32>,
}

/// File-level encryption operations
pub struct FileEncryption {
    params: AesParams,
    segment_size: usize,
    rng: SystemRandom,
}

impl FileEncryption {
    pub fn new(params: &AesParams) -> Self {
        Self {
            params: params.clone(),
            segment_size: DEFAULT_SEGMENT_SIZE,
            rng: SystemRandom::new(),
        }
    }

    pub fn with_segment_size(params: &AesParams, segment_size: usize) -> Self {
        Self {
            params: params.clone(),
            segment_size,
            rng: SystemRandom::new(),
        }
    }

    /// Encrypt file data into multiple encrypted segments
    pub fn encrypt_to_segments(
        &self,
        file_data: &[u8],
        key_hierarchy: &KeyHierarchy,
    ) -> Result<Vec<EncryptedData>> {
        let mut encrypted_segments = Vec::new();
        let total_segments = (file_data.len() + self.segment_size - 1) / self.segment_size;

        for (segment_index, chunk) in file_data.chunks(self.segment_size).enumerate() {
            let segment_encryption = SegmentEncryption::new(&self.params, &self.rng);
            let key_path = vec![segment_index as u32]; // Simple path for now
            
            let encrypted_data = segment_encryption.encrypt_segment(
                chunk,
                segment_index as u64,
                key_hierarchy,
                &key_path,
            )?;

            encrypted_segments.push(encrypted_data);
        }

        Ok(encrypted_segments)
    }

    /// Decrypt segments back to original file data
    pub fn decrypt_from_segments(
        &self,
        encrypted_segments: &[EncryptedData],
        key_hierarchy: &KeyHierarchy,
    ) -> Result<Vec<u8>> {
        // Sort segments by index to ensure correct order
        let mut sorted_segments = encrypted_segments.to_vec();
        sorted_segments.sort_by_key(|s| s.segment_index);

        let mut decrypted_data = Vec::new();
        
        for encrypted_segment in sorted_segments {
            let segment_encryption = SegmentEncryption::new(&self.params, &self.rng);
            let decrypted_chunk = segment_encryption.decrypt_segment(
                &encrypted_segment,
                key_hierarchy,
            )?;
            
            decrypted_data.extend_from_slice(&decrypted_chunk);
        }

        Ok(decrypted_data)
    }
}

/// Segment-level encryption operations
pub struct SegmentEncryption {
    params: AesParams,
    rng: SystemRandom,
}

impl SegmentEncryption {
    pub fn new(params: &AesParams, rng: &SystemRandom) -> Self {
        Self {
            params: params.clone(),
            rng: SystemRandom::new(), // Create new instance for thread safety
        }
    }

    /// Encrypt a single file segment
    pub fn encrypt_segment(
        &self,
        segment_data: &[u8],
        segment_index: u64,
        key_hierarchy: &KeyHierarchy,
        key_path: &[u32],
    ) -> Result<EncryptedData> {
        // Derive segment-specific key
        let segment_key = key_hierarchy.derive_segment_key(key_path)
            .context("Failed to derive segment key")?;

        // Generate random nonce
        let mut nonce_bytes = [0u8; 12];
        self.rng.fill(&mut nonce_bytes)
            .map_err(|_| anyhow::anyhow!("Failed to generate random nonce"))?;

        let nonce = Nonce::assume_unique_for_key(nonce_bytes);

        // Prepare AAD (Additional Authenticated Data) with segment metadata
        let aad_data = self.prepare_aad(segment_index, key_path)?;
        let aad = Aad::from(&aad_data);

        // Create sealing key
        let unbound_key = UnboundKey::new(&AES_256_GCM, segment_key.as_bytes())
            .map_err(|_| anyhow::anyhow!("Failed to create encryption key"))?;
        
        let mut sealing_key = SealingKey::new(unbound_key, NonceCounter::new());

        // Encrypt the data
        let mut ciphertext = segment_data.to_vec();
        sealing_key.seal_in_place_append_tag(aad, &mut ciphertext)
            .map_err(|_| anyhow::anyhow!("Failed to encrypt segment"))?;

        Ok(EncryptedData {
            segment_index,
            ciphertext,
            nonce: nonce_bytes,
            aad: aad_data,
            key_path: key_path.to_vec(),
        })
    }

    /// Decrypt a single encrypted segment
    pub fn decrypt_segment(
        &self,
        encrypted_data: &EncryptedData,
        key_hierarchy: &KeyHierarchy,
    ) -> Result<Vec<u8>> {
        // Derive the same segment key used for encryption
        let segment_key = key_hierarchy.derive_segment_key(&encrypted_data.key_path)
            .context("Failed to derive segment key for decryption")?;

        // Recreate nonce
        let nonce = Nonce::assume_unique_for_key(encrypted_data.nonce);
        let aad = Aad::from(&encrypted_data.aad);

        // Create opening key  
        let unbound_key = UnboundKey::new(&AES_256_GCM, segment_key.as_bytes())
            .map_err(|_| anyhow::anyhow!("Failed to create decryption key"))?;
            
        let mut opening_key = OpeningKey::new(unbound_key, NonceCounter::new());

        // Decrypt the data
        let mut ciphertext = encrypted_data.ciphertext.clone();
        let plaintext = opening_key.open_in_place(aad, &mut ciphertext)
            .map_err(|_| anyhow::anyhow!("Failed to decrypt segment - authentication failed"))?;

        Ok(plaintext.to_vec())
    }

    /// Prepare Additional Authenticated Data (AAD) for GCM
    fn prepare_aad(&self, segment_index: u64, key_path: &[u32]) -> Result<Vec<u8>> {
        let mut aad = Vec::new();
        
        // Add segment index to AAD
        aad.extend_from_slice(&segment_index.to_le_bytes());
        
        // Add key path to AAD for additional security
        for path_element in key_path {
            aad.extend_from_slice(&path_element.to_le_bytes());
        }

        // Add version identifier for future compatibility
        aad.extend_from_slice(b"ZephyrFS-v1");

        Ok(aad)
    }
}

/// Nonce counter for GCM operations - ensures nonces are never reused
#[derive(ZeroizeOnDrop)]
struct NonceCounter {
    counter: u64,
}

impl NonceCounter {
    fn new() -> Self {
        Self { counter: 0 }
    }
}

impl NonceSequence for NonceCounter {
    fn advance(&mut self) -> Result<Nonce, Unspecified> {
        self.counter += 1;
        
        if self.counter == u64::MAX {
            return Err(Unspecified);
        }

        let mut nonce_bytes = [0u8; 12];
        nonce_bytes[4..].copy_from_slice(&self.counter.to_be_bytes());
        
        Ok(Nonce::assume_unique_for_key(nonce_bytes))
    }
}

impl Zeroize for NonceCounter {
    fn zeroize(&mut self) {
        self.counter = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::key_derivation::{KeyDerivation, DerivedKey};
    use crate::crypto::CryptoParams;

    fn create_test_hierarchy() -> Result<KeyHierarchy> {
        let params = CryptoParams::default();
        let key_derivation = KeyDerivation::new(&params.scrypt_params);
        let derived_key = key_derivation.derive_from_password("test_password_123")?;
        KeyHierarchy::new(derived_key)
    }

    #[test]
    fn test_segment_encryption_roundtrip() -> Result<()> {
        let params = CryptoParams::default().aes_params;
        let hierarchy = create_test_hierarchy()?;
        let rng = SystemRandom::new();
        
        let segment_encryption = SegmentEncryption::new(&params, &rng);
        let test_data = b"Hello, ZephyrFS! This is test segment data for encryption.";
        let key_path = vec![42];

        // Encrypt
        let encrypted = segment_encryption.encrypt_segment(
            test_data,
            1,
            &hierarchy,
            &key_path,
        )?;

        // Decrypt
        let decrypted = segment_encryption.decrypt_segment(&encrypted, &hierarchy)?;

        assert_eq!(test_data, decrypted.as_slice());
        assert_eq!(encrypted.segment_index, 1);
        assert_eq!(encrypted.key_path, vec![42]);
        
        Ok(())
    }

    #[test]
    fn test_file_encryption_multiple_segments() -> Result<()> {
        let params = CryptoParams::default().aes_params;
        let hierarchy = create_test_hierarchy()?;
        
        let file_encryption = FileEncryption::with_segment_size(&params, 100); // Small segments for testing
        let test_data = b"This is a longer test file that should be split into multiple segments for encryption testing. Each segment will be encrypted independently with its own derived key.";

        // Encrypt file
        let encrypted_segments = file_encryption.encrypt_to_segments(test_data, &hierarchy)?;
        
        // Should have multiple segments
        assert!(encrypted_segments.len() > 1);

        // Decrypt file
        let decrypted_data = file_encryption.decrypt_from_segments(&encrypted_segments, &hierarchy)?;

        assert_eq!(test_data, decrypted_data.as_slice());
        
        Ok(())
    }

    #[test]
    fn test_encryption_authentication_failure() -> Result<()> {
        let params = CryptoParams::default().aes_params;
        let hierarchy = create_test_hierarchy()?;
        let rng = SystemRandom::new();
        
        let segment_encryption = SegmentEncryption::new(&params, &rng);
        let test_data = b"Test data for authentication failure";
        let key_path = vec![1];

        // Encrypt
        let mut encrypted = segment_encryption.encrypt_segment(
            test_data,
            0,
            &hierarchy,
            &key_path,
        )?;

        // Tamper with ciphertext
        encrypted.ciphertext[0] ^= 1;

        // Decrypt should fail due to authentication failure
        let result = segment_encryption.decrypt_segment(&encrypted, &hierarchy);
        assert!(result.is_err());
        
        Ok(())
    }

    #[test]
    fn test_different_segments_different_keys() -> Result<()> {
        let params = CryptoParams::default().aes_params;
        let hierarchy = create_test_hierarchy()?;
        let rng = SystemRandom::new();
        
        let segment_encryption = SegmentEncryption::new(&params, &rng);
        let test_data = b"Same data, different keys";

        // Encrypt with different key paths
        let encrypted1 = segment_encryption.encrypt_segment(
            test_data, 0, &hierarchy, &[1]
        )?;
        
        let encrypted2 = segment_encryption.encrypt_segment(
            test_data, 0, &hierarchy, &[2]
        )?;

        // Ciphertexts should be different even with same plaintext
        assert_ne!(encrypted1.ciphertext, encrypted2.ciphertext);
        assert_ne!(encrypted1.key_path, encrypted2.key_path);
        
        Ok(())
    }
}