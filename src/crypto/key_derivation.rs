//! Key derivation system for ZephyrFS
//!
//! Implements hierarchical deterministic key derivation following Tahoe-LAFS security model.
//! Uses scrypt for password-based key derivation and HKDF for key hierarchy expansion.

use crate::crypto::{ScryptParams, SecureBytes};
use anyhow::{Context, Result};
use ring::hkdf::{Salt, Prk, HKDF_SHA256};
use ring::rand::{SecureRandom, SystemRandom};
use scrypt::{scrypt, Params};
use zeroize::{Zeroize, ZeroizeOnDrop};
use std::collections::HashMap;

/// Master key derived from user password using scrypt
#[derive(ZeroizeOnDrop)]
pub struct DerivedKey {
    /// Master key material (32 bytes)
    master_key: SecureBytes,
    /// Salt used for derivation (32 bytes)  
    salt: [u8; 32],
    /// Verification hash for password checking (32 bytes)
    verification_hash: [u8; 32],
}

impl DerivedKey {
    /// Create DerivedKey from raw bytes (64 bytes total)
    pub fn from_bytes(bytes: SecureBytes) -> Result<Self> {
        if bytes.len() != 64 {
            anyhow::bail!("DerivedKey requires exactly 64 bytes (32 master + 32 salt, verification derived)");
        }

        let mut master_key = SecureBytes::new(32);
        let mut salt = [0u8; 32];

        master_key.copy_from_slice(&bytes[0..32]);
        salt.copy_from_slice(&bytes[32..64]);

        // Derive verification hash using HKDF from master key and salt
        let salt_obj = Salt::new(HKDF_SHA256, &salt);
        let prk = salt_obj.extract(master_key.as_bytes());
        let mut verification_hash = [0u8; 32];
        prk.expand(&[b"ZephyrFS-verification-hash-v1"], HKDF_SHA256)
            .map_err(|_| anyhow::anyhow!("HKDF expansion failed"))?
            .fill(&mut verification_hash)
            .map_err(|_| anyhow::anyhow!("HKDF fill failed"))?;

        Ok(Self {
            master_key,
            salt,
            verification_hash,
        })
    }

    /// Export key material as bytes (for secure storage/transmission)
    pub fn to_bytes(&self) -> SecureBytes {
        let mut bytes = SecureBytes::new(64);
        bytes[0..32].copy_from_slice(self.master_key.as_bytes());
        bytes[32..64].copy_from_slice(&self.salt);
        // Note: verification_hash is derived, not stored
        bytes
    }

    /// Get master key bytes for HKDF
    pub fn master_key(&self) -> &SecureBytes {
        &self.master_key
    }

    /// Get salt used for derivation
    pub fn salt(&self) -> &[u8; 32] {
        &self.salt
    }

    /// Verify password against stored verification hash
    pub fn verify_password(&self, password: &str, params: &ScryptParams) -> Result<bool> {
        let mut derived = vec![0u8; params.output_len];
        let scrypt_params = Params::new(
            params.log_n, 
            params.r, 
            params.p, 
            params.output_len
        ).context("Invalid scrypt parameters")?;

        scrypt(password.as_bytes(), &self.salt, &scrypt_params, &mut derived)
            .context("Scrypt key derivation failed")?;

        // Extract master key from derived output
        let derived_master_key = &derived[0..32];

        // Derive verification hash using HKDF (same as during creation)
        let salt_obj = Salt::new(HKDF_SHA256, &self.salt);
        let prk = salt_obj.extract(derived_master_key);
        let mut computed_verification = [0u8; 32];
        prk.expand(&[b"ZephyrFS-verification-hash-v1"], HKDF_SHA256)
            .map_err(|_| anyhow::anyhow!("HKDF expansion failed"))?
            .fill(&mut computed_verification)
            .map_err(|_| anyhow::anyhow!("HKDF fill failed"))?;

        // Check verification hash matches
        let verification_ok = constant_time_eq::constant_time_eq(
            &computed_verification, 
            &self.verification_hash
        );

        // Clear derived key material
        derived.zeroize();
        computed_verification.zeroize();

        Ok(verification_ok)
    }
}

/// Password-based key derivation using scrypt
pub struct KeyDerivation {
    params: ScryptParams,
    rng: SystemRandom,
}

impl KeyDerivation {
    pub fn new(params: &ScryptParams) -> Self {
        Self {
            params: params.clone(),
            rng: SystemRandom::new(),
        }
    }

    /// Derive key from password with random salt
    pub fn derive_from_password(&self, password: &str) -> Result<DerivedKey> {
        // Generate random salt
        let mut salt = [0u8; 32];
        self.rng.fill(&mut salt)
            .map_err(|_| anyhow::anyhow!("Failed to generate random salt"))?;

        self.derive_from_password_with_salt(password, &salt)
    }

    /// Derive key from password with specific salt (for key recovery)
    pub fn derive_from_password_with_salt(&self, password: &str, salt: &[u8; 32]) -> Result<DerivedKey> {
        let scrypt_params = Params::new(
            self.params.log_n,
            self.params.r, 
            self.params.p,
            self.params.output_len
        ).context("Invalid scrypt parameters")?;

        let mut derived = vec![0u8; self.params.output_len];
        scrypt(password.as_bytes(), salt, &scrypt_params, &mut derived)
            .context("Scrypt key derivation failed")?;

        // Split derived output: 32 bytes master key (64 bytes total now)
        let mut master_key = SecureBytes::new(32);
        master_key.copy_from_slice(&derived[0..32]);

        // Derive verification hash using HKDF from master key and salt
        let salt_obj = Salt::new(HKDF_SHA256, salt);
        let prk = salt_obj.extract(master_key.as_bytes());
        let mut verification_hash = [0u8; 32];
        prk.expand(&[b"ZephyrFS-verification-hash-v1"], HKDF_SHA256)
            .map_err(|_| anyhow::anyhow!("HKDF expansion failed"))?
            .fill(&mut verification_hash)
            .map_err(|_| anyhow::anyhow!("HKDF fill failed"))?;

        // Clear the derived buffer
        derived.zeroize();

        Ok(DerivedKey {
            master_key,
            salt: *salt,
            verification_hash,
        })
    }
}

/// Hierarchical key system for deriving segment-specific keys
pub struct KeyHierarchy {
    master_prk: Prk,
    key_cache: HashMap<Vec<u32>, SecureBytes>,
}

impl KeyHierarchy {
    /// Create new key hierarchy from derived master key
    pub fn new(derived_key: DerivedKey) -> Result<Self> {
        // Use HKDF to create pseudorandom key from master key
        let salt = Salt::new(HKDF_SHA256, &derived_key.salt);
        let prk = salt.extract(derived_key.master_key.as_bytes());

        Ok(Self {
            master_prk: prk,
            key_cache: HashMap::new(),
        })
    }

    /// Derive segment-specific encryption key
    pub fn derive_segment_key(&self, key_path: &[u32]) -> Result<SecureBytes> {
        // Check cache first
        if let Some(cached_key) = self.key_cache.get(key_path) {
            return Ok(cached_key.clone());
        }

        // Create info string from key path
        let mut info = Vec::new();
        info.extend_from_slice(b"ZephyrFS-segment-key-v1:");
        for &path_element in key_path {
            info.extend_from_slice(&path_element.to_be_bytes());
        }

        // Derive key using HKDF-Expand
        let mut segment_key = SecureBytes::new(32); // AES-256 key size
        self.master_prk.expand(&[&info], ring::hkdf::HKDF_SHA256)
            .map_err(|_| anyhow::anyhow!("HKDF expansion failed"))?
            .fill(segment_key.as_mut())?;

        Ok(segment_key)
    }

    /// Derive file-level key (for file metadata encryption)
    pub fn derive_file_key(&self, file_id: &[u8]) -> Result<SecureBytes> {
        let mut info = Vec::new();
        info.extend_from_slice(b"ZephyrFS-file-key-v1:");
        info.extend_from_slice(file_id);

        let mut file_key = SecureBytes::new(32);
        self.master_prk.expand(&[&info], ring::hkdf::HKDF_SHA256)
            .map_err(|_| anyhow::anyhow!("HKDF expansion failed"))?
            .fill(file_key.as_mut())?;

        Ok(file_key)
    }

    /// Derive capability key for zero-knowledge proofs
    pub fn derive_capability_key(&self, capability_id: &[u8]) -> Result<SecureBytes> {
        let mut info = Vec::new();
        info.extend_from_slice(b"ZephyrFS-capability-key-v1:");
        info.extend_from_slice(capability_id);

        let mut capability_key = SecureBytes::new(32);
        self.master_prk.expand(&[&info], ring::hkdf::HKDF_SHA256)
            .map_err(|_| anyhow::anyhow!("HKDF expansion failed"))?
            .fill(capability_key.as_mut())?;

        Ok(capability_key)
    }

    /// Get public capability for sharing (non-sensitive key material)
    pub fn get_public_capability(&self) -> Result<Vec<u8>> {
        // Derive a public capability that can be shared without compromising security
        let mut info = Vec::new();
        info.extend_from_slice(b"ZephyrFS-public-capability-v1");

        let mut public_cap = vec![0u8; 32];
        self.master_prk.expand(&[&info], ring::hkdf::HKDF_SHA256)
            .map_err(|_| anyhow::anyhow!("HKDF expansion failed"))?
            .fill(&mut public_cap)?;

        Ok(public_cap)
    }

    /// Clear key cache (for security)
    pub fn clear_cache(&mut self) {
        for (_, mut key) in self.key_cache.drain() {
            key.zeroize();
        }
    }
}

impl Drop for KeyHierarchy {
    fn drop(&mut self) {
        self.clear_cache();
    }
}

impl Zeroize for KeyHierarchy {
    fn zeroize(&mut self) {
        self.clear_cache();
        // Note: Cannot zeroize master_prk as it's not under our control
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scrypt_params_validation() -> Result<()> {
        // Test different scrypt parameters to find valid range
        println!("Testing various scrypt parameters...");
        
        // Test with lower log_n
        let result15 = scrypt::Params::new(15, 8, 1, 96);
        println!("Scrypt params (15, 8, 1, 96) result: {:?}", result15);
        
        let result16 = scrypt::Params::new(16, 8, 1, 96);
        println!("Scrypt params (16, 8, 1, 96) result: {:?}", result16);
        
        let result17 = scrypt::Params::new(17, 8, 1, 96);
        println!("Scrypt params (17, 8, 1, 96) result: {:?}", result17);
        
        // Test with smaller output length
        let result17_32 = scrypt::Params::new(17, 8, 1, 32);
        println!("Scrypt params (17, 8, 1, 32) result: {:?}", result17_32);
        
        let result17_64 = scrypt::Params::new(17, 8, 1, 64);
        println!("Scrypt params (17, 8, 1, 64) result: {:?}", result17_64);
        
        // 64-byte versions should work, 96-byte should not
        assert!(result17_32.is_ok(), "17,8,1,32 should be valid");
        assert!(result17_64.is_ok(), "17,8,1,64 should be valid");
        assert!(!result17.is_ok(), "17,8,1,96 should be invalid");
        Ok(())
    }

    #[test]
    fn test_key_derivation_from_password() -> Result<()> {
        let params = ScryptParams {
            log_n: 15,  // Lower for testing (faster)
            r: 8,
            p: 1,
            output_len: 64,
        };

        let key_derivation = KeyDerivation::new(&params);
        let derived_key = key_derivation.derive_from_password("test_password_123")?;

        assert_eq!(derived_key.master_key.len(), 32);
        assert_eq!(derived_key.salt.len(), 32);
        assert_eq!(derived_key.verification_hash.len(), 32);

        Ok(())
    }

    #[test]
    fn test_password_verification() -> Result<()> {
        let params = ScryptParams {
            log_n: 15,
            r: 8, 
            p: 1,
            output_len: 64,
        };

        let key_derivation = KeyDerivation::new(&params);
        let derived_key = key_derivation.derive_from_password("correct_password")?;

        // Correct password should verify
        assert!(derived_key.verify_password("correct_password", &params)?);

        // Wrong password should not verify  
        assert!(!derived_key.verify_password("wrong_password", &params)?);

        Ok(())
    }

    #[test]
    fn test_key_hierarchy_segment_keys() -> Result<()> {
        let params = ScryptParams {
            log_n: 15,
            r: 8,
            p: 1,
            output_len: 96,
        };

        let key_derivation = KeyDerivation::new(&params);
        let derived_key = key_derivation.derive_from_password("test_password")?;
        let hierarchy = KeyHierarchy::new(derived_key)?;

        // Derive keys for different segments
        let key1 = hierarchy.derive_segment_key(&[0])?;
        let key2 = hierarchy.derive_segment_key(&[1])?;
        let key3 = hierarchy.derive_segment_key(&[0, 1])?;

        // Keys should be different
        assert_ne!(key1.as_bytes(), key2.as_bytes());
        assert_ne!(key1.as_bytes(), key3.as_bytes());
        assert_ne!(key2.as_bytes(), key3.as_bytes());

        // Same path should produce same key
        let key1_again = hierarchy.derive_segment_key(&[0])?;
        assert_eq!(key1.as_bytes(), key1_again.as_bytes());

        Ok(())
    }

    #[test]
    fn test_derived_key_serialization() -> Result<()> {
        let params = ScryptParams {
            log_n: 15,
            r: 8,
            p: 1, 
            output_len: 96,
        };

        let key_derivation = KeyDerivation::new(&params);
        let original_key = key_derivation.derive_from_password("serialization_test")?;

        // Serialize and deserialize
        let serialized = original_key.to_bytes();
        let restored_key = DerivedKey::from_bytes(serialized)?;

        // Should be able to verify with restored key
        assert!(restored_key.verify_password("serialization_test", &params)?);
        assert!(!restored_key.verify_password("wrong_password", &params)?);

        Ok(())
    }

    #[test]
    fn test_deterministic_key_derivation() -> Result<()> {
        let params = ScryptParams {
            log_n: 15,
            r: 8,
            p: 1,
            output_len: 96,
        };

        let key_derivation = KeyDerivation::new(&params);
        let password = "deterministic_test";
        let salt = [42u8; 32]; // Fixed salt

        // Derive key twice with same password and salt
        let key1 = key_derivation.derive_from_password_with_salt(password, &salt)?;
        let key2 = key_derivation.derive_from_password_with_salt(password, &salt)?;

        // Should produce identical keys
        assert_eq!(key1.master_key.as_bytes(), key2.master_key.as_bytes());
        assert_eq!(key1.salt, key2.salt);
        assert_eq!(key1.verification_hash, key2.verification_hash);

        Ok(())
    }

    #[test]
    fn test_capability_derivation() -> Result<()> {
        let params = ScryptParams {
            log_n: 15,
            r: 8,
            p: 1,
            output_len: 96,
        };

        let key_derivation = KeyDerivation::new(&params);
        let derived_key = key_derivation.derive_from_password("capability_test")?;
        let hierarchy = KeyHierarchy::new(derived_key)?;

        let file_key = hierarchy.derive_file_key(b"test_file_id")?;
        let cap_key = hierarchy.derive_capability_key(b"test_capability_id")?;
        let public_cap = hierarchy.get_public_capability()?;

        // All should be different
        assert_ne!(file_key.as_bytes(), cap_key.as_bytes());
        assert_ne!(file_key.as_bytes(), &public_cap);
        assert_ne!(cap_key.as_bytes(), &public_cap);

        // Should be deterministic
        let file_key2 = hierarchy.derive_file_key(b"test_file_id")?;
        assert_eq!(file_key.as_bytes(), file_key2.as_bytes());

        Ok(())
    }
}