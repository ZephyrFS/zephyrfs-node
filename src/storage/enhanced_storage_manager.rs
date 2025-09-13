//! Enhanced storage manager with encryption support
//!
//! Integrates encrypted chunk storage with existing storage operations
//! while maintaining backward compatibility and zero-knowledge security.

use anyhow::{Context, Result};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn, error};

use crate::crypto::{ZephyrCrypto, EncryptedData, ContentId};
use crate::storage::{
    StorageManager, StorageConfig, CapacityInfo,
    EncryptedChunkStore, EncryptedChunkMetadata, EncryptedFileMetadata,
    EncryptionMetadata, FileCapability, EncryptedStorageStats,
};

/// Enhanced storage manager supporting both encrypted and plaintext operations
/// 
/// Zero-knowledge: Encrypted files never expose plaintext to storage nodes
/// Backward compatibility: Supports existing plaintext storage operations
pub struct EnhancedStorageManager {
    /// Traditional plaintext storage manager
    plaintext_storage: Arc<StorageManager>,
    
    /// Encrypted chunk storage
    encrypted_storage: Arc<EncryptedChunkStore>,
    
    /// Storage configuration
    config: StorageConfig,
    
    /// Base storage path
    base_path: PathBuf,
    
    /// Combined capacity tracking
    combined_capacity: Arc<RwLock<CombinedCapacityInfo>>,
}

/// Combined capacity information for both encrypted and plaintext storage
#[derive(Debug, Clone)]
pub struct CombinedCapacityInfo {
    /// Traditional storage info
    pub plaintext_capacity: CapacityInfo,
    
    /// Encrypted storage info  
    pub encrypted_stats: EncryptedStorageStats,
    
    /// Combined totals
    pub total_used_space: u64,
    pub total_files: u64,
    pub total_chunks: u64,
    pub combined_efficiency: f64,
}

/// File storage result containing information about how the file was stored
#[derive(Debug)]
pub struct FileStorageResult {
    pub file_hash: String,
    pub is_encrypted: bool,
    pub chunk_count: usize,
    pub content_id: Option<String>,
    pub capability_id: Option<String>,
}

impl EnhancedStorageManager {
    /// Create a new enhanced storage manager
    /// 
    /// Zero-knowledge: Initializes both plaintext and encrypted storage backends
    pub async fn new<P: AsRef<Path>>(base_path: P, config: StorageConfig) -> Result<Self> {
        let base_path = base_path.as_ref().to_path_buf();
        info!("Initializing EnhancedStorageManager with encryption support at: {:?}", base_path);
        
        // Create subdirectories
        let plaintext_path = base_path.join("plaintext");
        let encrypted_path = base_path.join("encrypted");
        
        std::fs::create_dir_all(&plaintext_path)
            .context("Failed to create plaintext storage directory")?;
        std::fs::create_dir_all(&encrypted_path)
            .context("Failed to create encrypted storage directory")?;
        
        // Initialize storage backends
        let plaintext_storage = Arc::new(StorageManager::new(&plaintext_path, config.clone()).await
            .context("Failed to initialize plaintext storage manager")?);
        
        let encrypted_storage = Arc::new(EncryptedChunkStore::new(&encrypted_path)
            .context("Failed to initialize encrypted chunk store")?);
        
        // Initialize combined capacity tracking
        let combined_capacity = Arc::new(RwLock::new(CombinedCapacityInfo {
            plaintext_capacity: plaintext_storage.get_capacity_info().await,
            encrypted_stats: encrypted_storage.get_encrypted_stats().await,
            total_used_space: 0,
            total_files: 0,
            total_chunks: 0,
            combined_efficiency: 1.0,
        }));
        
        let manager = Self {
            plaintext_storage,
            encrypted_storage,
            config,
            base_path,
            combined_capacity,
        };
        
        // Update combined capacity information
        manager.refresh_combined_capacity().await?;
        
        info!("EnhancedStorageManager initialized with zero-knowledge encryption support");
        Ok(manager)
    }
    
    /// Store a file with automatic encryption (if crypto context provided)
    /// 
    /// Zero-knowledge: When crypto is provided, file is encrypted before storage
    pub async fn store_file_with_crypto(
        &self, 
        file_id: &str, 
        data: &[u8], 
        filename: &str,
        crypto: Option<&ZephyrCrypto>
    ) -> Result<FileStorageResult> {
        info!("Storing file: {} ({} bytes) with encryption: {}", 
              filename, data.len(), crypto.is_some());
        
        match crypto {
            Some(crypto_ctx) => {
                self.store_encrypted_file(file_id, data, filename, crypto_ctx).await
            }
            None => {
                self.store_plaintext_file(file_id, data, filename).await
            }
        }
    }
    
    /// Store an encrypted file using zero-knowledge encryption
    /// 
    /// Zero-knowledge: File is encrypted before storage, storage nodes never see plaintext
    async fn store_encrypted_file(
        &self,
        file_id: &str,
        data: &[u8],
        filename: &str,
        crypto: &ZephyrCrypto,
    ) -> Result<FileStorageResult> {
        debug!("Storing encrypted file: {}", file_id);
        
        // Encrypt file data into segments
        let encrypted_segments = crypto.encrypt_file(data)
            .context("Failed to encrypt file data")?;
        
        // Generate content ID for encrypted file
        let content_id = crypto.content_id(data);
        
        // Store encrypted chunks
        let mut chunk_hashes = Vec::new();
        for (index, encrypted_data) in encrypted_segments.iter().enumerate() {
            let chunk_id = format!("{}:{}", file_id, index);
            let chunk_hash = self.encrypted_storage.store_encrypted_chunk(&chunk_id, encrypted_data).await
                .context("Failed to store encrypted chunk")?;
            chunk_hashes.push(chunk_hash);
        }
        
        // Create encrypted file metadata
        let encrypted_metadata = EncryptedFileMetadata {
            encrypted_name: self.encrypt_filename(filename, crypto)?,
            encrypted_size_info: self.encrypt_size_info(data.len(), crypto)?,
            encrypted_file_hash: content_id.to_hex(),
            encrypted_chunk_ids: chunk_hashes,
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)?
                .as_secs(),
            modified_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)?
                .as_secs(),
            encryption_metadata: EncryptionMetadata {
                version: 1,
                segment_count: encrypted_segments.len() as u32,
                chunk_size_mb: 1, // TODO: Get from config
                content_hash_algorithm: "blake3".to_string(),
                verification_hash_algorithm: "blake3".to_string(),
                master_nonce: [0u8; 12], // TODO: Generate proper nonce
                encrypted_content_verification: self.encrypt_content_verification(&content_id, crypto)?,
            },
            capabilities: vec![], // TODO: Generate initial capability
        };
        
        // Store encrypted file metadata
        self.encrypted_storage.store_encrypted_file_metadata(file_id, &encrypted_metadata).await
            .context("Failed to store encrypted file metadata")?;
        
        // Create file capability for access control
        let capability = self.create_file_capability(file_id, crypto).await?;
        let capability_id = capability.capability_id.clone();
        
        self.encrypted_storage.store_capability(&capability).await
            .context("Failed to store file capability")?;
        
        // Update capacity tracking
        self.refresh_combined_capacity().await?;
        
        info!("Successfully stored encrypted file: {} with capability: {}", file_id, capability_id);
        Ok(FileStorageResult {
            file_hash: content_id.to_hex(),
            is_encrypted: true,
            chunk_count: encrypted_segments.len(),
            content_id: Some(content_id.to_hex()),
            capability_id: Some(capability_id),
        })
    }
    
    /// Store a plaintext file using existing storage manager
    async fn store_plaintext_file(
        &self,
        file_id: &str,
        data: &[u8],
        filename: &str,
    ) -> Result<FileStorageResult> {
        debug!("Storing plaintext file: {}", file_id);
        
        let file_hash = self.plaintext_storage.store_file(file_id, data, filename).await
            .context("Failed to store plaintext file")?;
        
        // Update capacity tracking
        self.refresh_combined_capacity().await?;
        
        info!("Successfully stored plaintext file: {}", file_id);
        Ok(FileStorageResult {
            file_hash,
            is_encrypted: false,
            chunk_count: 1, // Plaintext files are stored as single chunks
            content_id: None,
            capability_id: None,
        })
    }
    
    /// Retrieve a file with automatic detection of encryption
    /// 
    /// Zero-knowledge: Encrypted files are decrypted using provided crypto context
    pub async fn retrieve_file_with_crypto(
        &self,
        file_id: &str,
        crypto: Option<&ZephyrCrypto>,
    ) -> Result<Option<(Vec<u8>, bool)>> {
        debug!("Retrieving file: {} with crypto: {}", file_id, crypto.is_some());
        
        // First try encrypted storage
        if let Some(encrypted_metadata) = self.encrypted_storage.get_encrypted_file_metadata(file_id).await? {
            if let Some(crypto_ctx) = crypto {
                let data = self.retrieve_encrypted_file(file_id, &encrypted_metadata, crypto_ctx).await?;
                return Ok(Some((data, true)));
            } else {
                return Err(anyhow::anyhow!(
                    "File {} is encrypted but no crypto context provided for decryption", file_id
                ));
            }
        }
        
        // Try plaintext storage
        if let Some(data) = self.plaintext_storage.retrieve_file(file_id).await? {
            return Ok(Some((data, false)));
        }
        
        Ok(None)
    }
    
    /// Retrieve an encrypted file using zero-knowledge decryption
    /// 
    /// Zero-knowledge: File is decrypted client-side, storage nodes never see plaintext
    async fn retrieve_encrypted_file(
        &self,
        file_id: &str,
        metadata: &EncryptedFileMetadata,
        crypto: &ZephyrCrypto,
    ) -> Result<Vec<u8>> {
        debug!("Retrieving encrypted file: {}", file_id);
        
        // Retrieve all encrypted chunks
        let mut encrypted_segments = Vec::new();
        for (index, chunk_hash) in metadata.encrypted_chunk_ids.iter().enumerate() {
            let mut encrypted_data = self.encrypted_storage.retrieve_encrypted_chunk(chunk_hash).await?
                .ok_or_else(|| anyhow::anyhow!("Missing encrypted chunk: {}", chunk_hash))?;
            
            // Set correct segment index
            encrypted_data.segment_index = index as u64;
            encrypted_segments.push(encrypted_data);
        }
        
        // Decrypt file
        let decrypted_data = crypto.decrypt_file(&encrypted_segments)
            .context("Failed to decrypt file")?;
        
        // Verify content integrity
        let computed_content_id = crypto.content_id(&decrypted_data);
        let expected_content_id = ContentId::from_hex(
            crate::crypto::HashAlgorithm::Blake3,
            &metadata.encrypted_file_hash
        ).context("Invalid content ID in encrypted file metadata")?;
        
        if !crypto.verify_content(&decrypted_data, &expected_content_id) {
            return Err(anyhow::anyhow!(
                "Content verification failed for encrypted file: {}", file_id
            ));
        }
        
        info!("Successfully retrieved and decrypted file: {}", file_id);
        Ok(decrypted_data)
    }
    
    /// Check if a file exists in either storage backend
    pub async fn file_exists(&self, file_id: &str) -> Result<(bool, bool)> {
        let encrypted_exists = self.encrypted_storage.get_encrypted_file_metadata(file_id).await?.is_some();
        let plaintext_exists = self.plaintext_storage.file_exists(file_id).await?;
        Ok((plaintext_exists, encrypted_exists))
    }
    
    /// Get combined storage capacity information
    pub async fn get_combined_capacity(&self) -> CombinedCapacityInfo {
        let capacity = self.combined_capacity.read().await;
        capacity.clone()
    }
    
    /// Get file capability for encrypted files
    pub async fn get_file_capability(&self, capability_id: &str) -> Result<Option<FileCapability>> {
        self.encrypted_storage.get_capability(capability_id).await
    }
    
    /// Grant access to an encrypted file by creating a new capability
    pub async fn grant_file_access(
        &self,
        file_id: &str,
        permissions: &[u8],
        crypto: &ZephyrCrypto,
    ) -> Result<String> {
        let capability = FileCapability {
            capability_id: self.generate_capability_id(),
            file_id: file_id.to_string(),
            encrypted_permissions: permissions.to_vec(),
            encrypted_key_material: self.encrypt_key_material_for_capability(crypto)?,
            expires_at: None,
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)?
                .as_secs(),
            signature: vec![], // TODO: Generate proper signature
        };
        
        let capability_id = capability.capability_id.clone();
        self.encrypted_storage.store_capability(&capability).await?;
        
        info!("Created file capability: {} for file: {}", capability_id, file_id);
        Ok(capability_id)
    }
    
    /// Helper methods for encryption operations
    fn encrypt_filename(&self, filename: &str, crypto: &ZephyrCrypto) -> Result<Vec<u8>> {
        // TODO: Implement filename encryption
        Ok(filename.as_bytes().to_vec())
    }
    
    fn encrypt_size_info(&self, size: usize, crypto: &ZephyrCrypto) -> Result<Vec<u8>> {
        // TODO: Implement size info encryption
        Ok(size.to_le_bytes().to_vec())
    }
    
    fn encrypt_content_verification(&self, content_id: &ContentId, crypto: &ZephyrCrypto) -> Result<Vec<u8>> {
        // TODO: Implement content verification encryption
        Ok(content_id.to_hex().as_bytes().to_vec())
    }
    
    async fn create_file_capability(&self, file_id: &str, crypto: &ZephyrCrypto) -> Result<FileCapability> {
        let capability_id = self.generate_capability_id();
        
        Ok(FileCapability {
            capability_id,
            file_id: file_id.to_string(),
            encrypted_permissions: vec![0x01], // Basic read permission
            encrypted_key_material: self.encrypt_key_material_for_capability(crypto)?,
            expires_at: None,
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)?
                .as_secs(),
            signature: vec![], // TODO: Generate proper signature
        })
    }
    
    fn encrypt_key_material_for_capability(&self, crypto: &ZephyrCrypto) -> Result<Vec<u8>> {
        // TODO: Implement proper key material encryption for capabilities
        crypto.get_capability().context("Failed to get capability key material")
    }
    
    fn generate_capability_id(&self) -> String {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let bytes: [u8; 16] = rng.gen();
        hex::encode(bytes)
    }
    
    async fn refresh_combined_capacity(&self) -> Result<()> {
        let plaintext_capacity = self.plaintext_storage.get_capacity_info().await;
        let encrypted_stats = self.encrypted_storage.get_encrypted_stats().await;
        
        let total_used_space = plaintext_capacity.used_space + encrypted_stats.total_encrypted_size;
        let total_files = plaintext_capacity.file_count + encrypted_stats.total_encrypted_files;
        let total_chunks = plaintext_capacity.chunk_count + encrypted_stats.total_encrypted_chunks;
        
        // Calculate combined efficiency (deduplication benefits)
        let combined_efficiency = if total_used_space > 0 {
            // This is a simplified calculation - in practice, we'd need more sophisticated metrics
            (plaintext_capacity.efficiency_ratio + 1.0) / 2.0
        } else {
            1.0
        };
        
        let mut capacity = self.combined_capacity.write().await;
        *capacity = CombinedCapacityInfo {
            plaintext_capacity,
            encrypted_stats,
            total_used_space,
            total_files,
            total_chunks,
            combined_efficiency,
        };
        
        debug!("Updated combined capacity: {} bytes used, {} files, {} chunks", 
               total_used_space, total_files, total_chunks);
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use crate::crypto::{CryptoParams, ZephyrCrypto};
    
    #[tokio::test]
    async fn test_enhanced_storage_manager_creation() {
        let temp_dir = tempdir().unwrap();
        let config = StorageConfig::default();
        
        let manager = EnhancedStorageManager::new(temp_dir.path(), config).await.unwrap();
        let capacity = manager.get_combined_capacity().await;
        
        assert_eq!(capacity.total_used_space, 0);
        assert_eq!(capacity.total_files, 0);
        assert_eq!(capacity.total_chunks, 0);
    }
    
    #[tokio::test]
    async fn test_plaintext_file_storage() {
        let temp_dir = tempdir().unwrap();
        let config = StorageConfig::default();
        let manager = EnhancedStorageManager::new(temp_dir.path(), config).await.unwrap();
        
        let file_id = "test-file-1";
        let filename = "test.txt";
        let data = b"Hello, ZephyrFS! This is a test file.";
        
        // Store plaintext file
        let result = manager.store_file_with_crypto(file_id, data, filename, None).await.unwrap();
        assert!(!result.is_encrypted);
        assert!(result.capability_id.is_none());
        
        // Retrieve plaintext file
        let (retrieved_data, is_encrypted) = manager.retrieve_file_with_crypto(file_id, None).await.unwrap().unwrap();
        assert_eq!(retrieved_data, data);
        assert!(!is_encrypted);
    }
    
    #[tokio::test]
    async fn test_encrypted_file_storage() {
        let temp_dir = tempdir().unwrap();
        let config = StorageConfig::default();
        let manager = EnhancedStorageManager::new(temp_dir.path(), config).await.unwrap();
        
        // Create crypto context
        let mut crypto = ZephyrCrypto::new();
        crypto.init_from_password("test_password_123").unwrap();
        
        let file_id = "encrypted-file-1";
        let filename = "secret.txt";
        let data = b"This is confidential data that should be encrypted.";
        
        // Store encrypted file
        let result = manager.store_file_with_crypto(file_id, data, filename, Some(&crypto)).await.unwrap();
        assert!(result.is_encrypted);
        assert!(result.capability_id.is_some());
        
        // Retrieve encrypted file
        let (retrieved_data, is_encrypted) = manager.retrieve_file_with_crypto(file_id, Some(&crypto)).await.unwrap().unwrap();
        assert_eq!(retrieved_data, data);
        assert!(is_encrypted);
    }
}