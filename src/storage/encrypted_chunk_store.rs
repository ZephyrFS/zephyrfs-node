//! Encrypted chunk storage for ZephyrFS
//!
//! Provides storage layer functionality for encrypted chunks while maintaining
//! zero-knowledge security. Storage nodes never see plaintext data.

use anyhow::{Context, Result};
use rocksdb::{DB, Options, WriteBatch};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

use crate::crypto::{EncryptedData, ContentId};

/// Metadata for encrypted chunks stored in the system
/// 
/// Zero-knowledge: Only stores encrypted data and hashes, no plaintext metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedChunkMetadata {
    /// Content hash of the encrypted chunk (for deduplication)
    pub encrypted_hash: String,
    
    /// Size of the encrypted chunk in bytes
    pub encrypted_size: u64,
    
    /// Timestamp when chunk was stored
    pub stored_at: u64,
    
    /// Reference count (how many encrypted files reference this chunk)
    pub ref_count: u32,
    
    /// Verification checksum for integrity (of encrypted data)
    pub checksum: String,
    
    /// Content addressing hash (encrypted, for lookup)
    pub content_id: Option<String>,
    
    /// Encryption nonce (safe to store)
    pub nonce: [u8; 12],
    
    /// Additional authenticated data (encrypted metadata)
    pub aad: Vec<u8>,
    
    /// Key derivation path (safe to store, no keys)
    pub key_path: Vec<u32>,
}

/// Enhanced file metadata that includes encryption information
/// 
/// Zero-knowledge: Stores encrypted metadata and access patterns
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedFileMetadata {
    /// Original filename (encrypted)
    pub encrypted_name: Vec<u8>,
    
    /// Encrypted file size info
    pub encrypted_size_info: Vec<u8>,
    
    /// File hash of encrypted data
    pub encrypted_file_hash: String,
    
    /// List of encrypted chunk IDs
    pub encrypted_chunk_ids: Vec<String>,
    
    /// Timestamp (can be plaintext for sorting)
    pub created_at: u64,
    pub modified_at: u64,
    
    /// Encryption metadata
    pub encryption_metadata: EncryptionMetadata,
    
    /// Access control capabilities (encrypted)
    pub capabilities: Vec<u8>,
}

/// Encryption-specific metadata stored with files
/// 
/// Zero-knowledge: No sensitive key material stored
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptionMetadata {
    /// Encryption algorithm version
    pub version: u32,
    
    /// Number of encrypted segments
    pub segment_count: u32,
    
    /// Chunk size used for encryption (MB)
    pub chunk_size_mb: u32,
    
    /// Content addressing algorithm
    pub content_hash_algorithm: String,
    
    /// Verification hash algorithm
    pub verification_hash_algorithm: String,
    
    /// Master nonce for file-level operations
    pub master_nonce: [u8; 12],
    
    /// Encrypted content verification data
    pub encrypted_content_verification: Vec<u8>,
}

/// Capability token for secure file access
/// 
/// Zero-knowledge: Contains encrypted access permissions and keys
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileCapability {
    /// Unique capability ID
    pub capability_id: String,
    
    /// File ID this capability grants access to
    pub file_id: String,
    
    /// Encrypted access permissions (read, write, share, etc.)
    pub encrypted_permissions: Vec<u8>,
    
    /// Encrypted key material for this capability
    pub encrypted_key_material: Vec<u8>,
    
    /// Capability expiration timestamp (optional)
    pub expires_at: Option<u64>,
    
    /// Created timestamp
    pub created_at: u64,
    
    /// Capability signature (for verification)
    pub signature: Vec<u8>,
}

/// Enhanced chunk store that handles encrypted chunks
/// 
/// Zero-knowledge: Never processes or sees plaintext data
pub struct EncryptedChunkStore {
    /// Underlying chunk storage
    db: Arc<DB>,
    
    /// Metadata cache for encrypted chunks
    metadata_cache: Arc<RwLock<HashMap<String, EncryptedChunkMetadata>>>,
    
    /// File metadata storage
    file_metadata_cache: Arc<RwLock<HashMap<String, EncryptedFileMetadata>>>,
    
    /// Capability storage
    capability_cache: Arc<RwLock<HashMap<String, FileCapability>>>,
    
    /// Storage statistics
    stats: Arc<RwLock<EncryptedStorageStats>>,
}

#[derive(Debug, Default, Clone)]
pub struct EncryptedStorageStats {
    pub total_encrypted_chunks: u64,
    pub total_encrypted_size: u64,
    pub total_encrypted_files: u64,
    pub active_capabilities: u64,
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub deduplication_savings: u64,
}

impl EncryptedChunkStore {
    /// Create a new encrypted chunk store
    /// 
    /// Zero-knowledge: Configures storage to handle only encrypted data
    pub fn new<P: AsRef<Path>>(db_path: P) -> Result<Self> {
        info!("Initializing EncryptedChunkStore for zero-knowledge storage");
        
        let mut opts = Options::default();
        opts.create_if_missing(true);
        opts.set_paranoid_checks(true);
        opts.set_use_fsync(true);
        
        let db = DB::open(&opts, db_path)
            .context("Failed to open encrypted chunk database")?;
        
        let store = Self {
            db: Arc::new(db),
            metadata_cache: Arc::new(RwLock::new(HashMap::new())),
            file_metadata_cache: Arc::new(RwLock::new(HashMap::new())),
            capability_cache: Arc::new(RwLock::new(HashMap::new())),
            stats: Arc::new(RwLock::new(EncryptedStorageStats::default())),
        };
        
        store.load_stats_from_db()?;
        info!("EncryptedChunkStore initialized with zero-knowledge architecture");
        Ok(store)
    }
    
    /// Store an encrypted chunk with deduplication
    /// 
    /// Zero-knowledge: Only handles encrypted data, maintains content-based deduplication
    pub async fn store_encrypted_chunk(&self, chunk_id: &str, encrypted_data: &EncryptedData) -> Result<String> {
        debug!("Storing encrypted chunk: {} ({} bytes)", chunk_id, encrypted_data.ciphertext.len());
        
        // Calculate hash of encrypted content for deduplication
        let mut hasher = Sha256::new();
        hasher.update(&encrypted_data.ciphertext);
        hasher.update(&encrypted_data.nonce);
        hasher.update(&encrypted_data.aad);
        let encrypted_hash = hex::encode(hasher.finalize());
        
        // Check if this encrypted chunk already exists (deduplication)
        if let Some(existing_metadata) = self.get_encrypted_chunk_metadata(&encrypted_hash).await? {
            // Increment reference count
            let mut metadata = existing_metadata;
            metadata.ref_count += 1;
            self.update_encrypted_chunk_metadata(&encrypted_hash, &metadata).await?;
            
            debug!("Deduplicated encrypted chunk: {} (ref_count: {})", encrypted_hash, metadata.ref_count);
            return Ok(encrypted_hash);
        }
        
        // Create checksum for integrity verification
        let checksum = self.calculate_encrypted_checksum(&encrypted_data.ciphertext, &encrypted_hash);
        
        let metadata = EncryptedChunkMetadata {
            encrypted_hash: encrypted_hash.clone(),
            encrypted_size: encrypted_data.ciphertext.len() as u64,
            stored_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)?
                .as_secs(),
            ref_count: 1,
            checksum,
            content_id: None, // Set by caller if needed
            nonce: encrypted_data.nonce,
            aad: encrypted_data.aad.clone(),
            key_path: encrypted_data.key_path.clone(),
        };
        
        // Store encrypted chunk data and metadata atomically
        let mut batch = WriteBatch::default();
        
        // Store the encrypted ciphertext
        let chunk_key = format!("chunk:{}", encrypted_hash);
        batch.put(&chunk_key, &encrypted_data.ciphertext);
        
        // Store metadata
        let metadata_key = format!("meta:{}", encrypted_hash);
        let metadata_bytes = bincode::serialize(&metadata)
            .context("Failed to serialize encrypted chunk metadata")?;
        batch.put(&metadata_key, &metadata_bytes);
        
        self.db.write(batch)
            .context("Failed to store encrypted chunk atomically")?;
        
        // Update cache and stats
        {
            let mut cache = self.metadata_cache.write().await;
            cache.insert(encrypted_hash.clone(), metadata);
        }
        
        {
            let mut stats = self.stats.write().await;
            stats.total_encrypted_chunks += 1;
            stats.total_encrypted_size += encrypted_data.ciphertext.len() as u64;
        }
        
        info!("Stored new encrypted chunk: {}", encrypted_hash);
        Ok(encrypted_hash)
    }
    
    /// Retrieve an encrypted chunk by hash
    /// 
    /// Zero-knowledge: Returns encrypted data without any decryption
    pub async fn retrieve_encrypted_chunk(&self, encrypted_hash: &str) -> Result<Option<EncryptedData>> {
        debug!("Retrieving encrypted chunk: {}", encrypted_hash);
        
        // Get metadata first
        let metadata = match self.get_encrypted_chunk_metadata(encrypted_hash).await? {
            Some(meta) => meta,
            None => {
                debug!("Encrypted chunk not found: {}", encrypted_hash);
                return Ok(None);
            }
        };
        
        // Retrieve encrypted ciphertext
        let chunk_key = format!("chunk:{}", encrypted_hash);
        let ciphertext = match self.db.get(&chunk_key)
            .context("Failed to read encrypted chunk from database")? {
            Some(data) => data,
            None => {
                warn!("Encrypted chunk data missing for hash: {}", encrypted_hash);
                return Ok(None);
            }
        };
        
        // Verify integrity
        let computed_checksum = self.calculate_encrypted_checksum(&ciphertext, encrypted_hash);
        if computed_checksum != metadata.checksum {
            return Err(anyhow::anyhow!(
                "Encrypted chunk integrity verification failed for {}", encrypted_hash
            ));
        }
        
        // Reconstruct EncryptedData
        let encrypted_data = EncryptedData {
            segment_index: 0, // Will be set by caller
            ciphertext,
            nonce: metadata.nonce,
            aad: metadata.aad,
            key_path: metadata.key_path,
        };
        
        {
            let mut stats = self.stats.write().await;
            stats.cache_hits += 1;
        }
        
        Ok(Some(encrypted_data))
    }
    
    /// Store encrypted file metadata
    /// 
    /// Zero-knowledge: All sensitive metadata is encrypted
    pub async fn store_encrypted_file_metadata(&self, file_id: &str, metadata: &EncryptedFileMetadata) -> Result<()> {
        debug!("Storing encrypted file metadata: {}", file_id);
        
        let metadata_key = format!("file:{}", file_id);
        let metadata_bytes = bincode::serialize(metadata)
            .context("Failed to serialize encrypted file metadata")?;
        
        self.db.put(&metadata_key, &metadata_bytes)
            .context("Failed to store encrypted file metadata")?;
        
        // Update cache
        {
            let mut cache = self.file_metadata_cache.write().await;
            cache.insert(file_id.to_string(), metadata.clone());
        }
        
        {
            let mut stats = self.stats.write().await;
            stats.total_encrypted_files += 1;
        }
        
        Ok(())
    }
    
    /// Retrieve encrypted file metadata
    /// 
    /// Zero-knowledge: Returns encrypted metadata without decryption
    pub async fn get_encrypted_file_metadata(&self, file_id: &str) -> Result<Option<EncryptedFileMetadata>> {
        // Check cache first
        {
            let cache = self.file_metadata_cache.read().await;
            if let Some(metadata) = cache.get(file_id) {
                return Ok(Some(metadata.clone()));
            }
        }
        
        let metadata_key = format!("file:{}", file_id);
        let metadata_bytes = match self.db.get(&metadata_key)
            .context("Failed to read encrypted file metadata")? {
            Some(data) => data,
            None => return Ok(None),
        };
        
        let metadata: EncryptedFileMetadata = bincode::deserialize(&metadata_bytes)
            .context("Failed to deserialize encrypted file metadata")?;
        
        // Update cache
        {
            let mut cache = self.file_metadata_cache.write().await;
            cache.insert(file_id.to_string(), metadata.clone());
        }
        
        Ok(Some(metadata))
    }
    
    /// Store a file capability for secure access control
    /// 
    /// Zero-knowledge: Capability contains encrypted permissions and keys
    pub async fn store_capability(&self, capability: &FileCapability) -> Result<()> {
        debug!("Storing file capability: {}", capability.capability_id);
        
        let cap_key = format!("cap:{}", capability.capability_id);
        let cap_bytes = bincode::serialize(capability)
            .context("Failed to serialize capability")?;
        
        self.db.put(&cap_key, &cap_bytes)
            .context("Failed to store capability")?;
        
        // Update cache and stats
        {
            let mut cache = self.capability_cache.write().await;
            cache.insert(capability.capability_id.clone(), capability.clone());
        }
        
        {
            let mut stats = self.stats.write().await;
            stats.active_capabilities += 1;
        }
        
        Ok(())
    }
    
    /// Retrieve a file capability
    pub async fn get_capability(&self, capability_id: &str) -> Result<Option<FileCapability>> {
        // Check cache first
        {
            let cache = self.capability_cache.read().await;
            if let Some(capability) = cache.get(capability_id) {
                return Ok(Some(capability.clone()));
            }
        }
        
        let cap_key = format!("cap:{}", capability_id);
        let cap_bytes = match self.db.get(&cap_key)
            .context("Failed to read capability")? {
            Some(data) => data,
            None => return Ok(None),
        };
        
        let capability: FileCapability = bincode::deserialize(&cap_bytes)
            .context("Failed to deserialize capability")?;
        
        // Check expiration
        if let Some(expires_at) = capability.expires_at {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)?
                .as_secs();
            if now > expires_at {
                debug!("Capability expired: {}", capability_id);
                return Ok(None);
            }
        }
        
        // Update cache
        {
            let mut cache = self.capability_cache.write().await;
            cache.insert(capability_id.to_string(), capability.clone());
        }
        
        Ok(Some(capability))
    }
    
    /// Get storage statistics
    pub async fn get_encrypted_stats(&self) -> EncryptedStorageStats {
        let stats = self.stats.read().await;
        (*stats).clone()
    }
    
    /// Helper methods
    async fn get_encrypted_chunk_metadata(&self, encrypted_hash: &str) -> Result<Option<EncryptedChunkMetadata>> {
        // Check cache first
        {
            let cache = self.metadata_cache.read().await;
            if let Some(metadata) = cache.get(encrypted_hash) {
                return Ok(Some(metadata.clone()));
            }
        }
        
        let metadata_key = format!("meta:{}", encrypted_hash);
        let metadata_bytes = match self.db.get(&metadata_key)? {
            Some(data) => data,
            None => return Ok(None),
        };
        
        let metadata: EncryptedChunkMetadata = bincode::deserialize(&metadata_bytes)
            .context("Failed to deserialize encrypted chunk metadata")?;
        
        Ok(Some(metadata))
    }
    
    async fn update_encrypted_chunk_metadata(&self, encrypted_hash: &str, metadata: &EncryptedChunkMetadata) -> Result<()> {
        let metadata_key = format!("meta:{}", encrypted_hash);
        let metadata_bytes = bincode::serialize(metadata)?;
        
        self.db.put(&metadata_key, &metadata_bytes)?;
        
        // Update cache
        {
            let mut cache = self.metadata_cache.write().await;
            cache.insert(encrypted_hash.to_string(), metadata.clone());
        }
        
        Ok(())
    }
    
    fn calculate_encrypted_checksum(&self, ciphertext: &[u8], hash: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(ciphertext);
        hasher.update(hash.as_bytes());
        hasher.update(b"zephyrfs-encrypted-chunk-v1");
        hex::encode(hasher.finalize())
    }
    
    fn load_stats_from_db(&self) -> Result<()> {
        // Implementation would scan database to calculate stats
        // For now, we'll initialize with defaults
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    
    #[tokio::test]
    async fn test_encrypted_chunk_store_creation() {
        let temp_dir = tempdir().unwrap();
        let store = EncryptedChunkStore::new(temp_dir.path()).unwrap();
        let stats = store.get_encrypted_stats().await;
        
        assert_eq!(stats.total_encrypted_chunks, 0);
        assert_eq!(stats.total_encrypted_files, 0);
        assert_eq!(stats.active_capabilities, 0);
    }
    
    #[tokio::test]
    async fn test_encrypted_chunk_deduplication() {
        let temp_dir = tempdir().unwrap();
        let store = EncryptedChunkStore::new(temp_dir.path()).unwrap();
        
        let encrypted_data = EncryptedData {
            segment_index: 0,
            ciphertext: vec![1, 2, 3, 4, 5],
            nonce: [0u8; 12],
            aad: vec![],
            key_path: vec![0, 1],
        };
        
        // Store same encrypted chunk twice
        let hash1 = store.store_encrypted_chunk("chunk1", &encrypted_data).await.unwrap();
        let hash2 = store.store_encrypted_chunk("chunk2", &encrypted_data).await.unwrap();
        
        // Should be deduplicated (same hash)
        assert_eq!(hash1, hash2);
        
        // Should have reference count of 2
        let metadata = store.get_encrypted_chunk_metadata(&hash1).await.unwrap().unwrap();
        assert_eq!(metadata.ref_count, 2);
    }
}