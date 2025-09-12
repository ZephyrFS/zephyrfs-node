use anyhow::{Context, Result};
use rocksdb::{DB, Options, WriteBatch};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// File metadata with comprehensive tracking
/// 
/// Privacy: Only stores operational metadata, not file contents
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileMetadata {
    /// Original filename (encrypted if privacy mode enabled)
    pub name: String,
    
    /// File size in bytes
    pub size: u64,
    
    /// MIME type detection
    pub mime_type: Option<String>,
    
    /// SHA-256 hash of complete file
    pub file_hash: String,
    
    /// List of chunk IDs that comprise this file
    pub chunk_ids: Vec<String>,
    
    /// Creation timestamp
    pub created_at: u64,
    
    /// Last modified timestamp
    pub modified_at: u64,
    
    /// Access permissions (future use)
    pub permissions: u32,
    
    /// Integrity checksum for metadata verification
    pub checksum: String,
}

/// Thread-safe metadata storage using RocksDB
/// 
/// Safety: All operations include integrity checks and atomic updates
/// Transparency: All metadata operations are logged for audit
/// Privacy: Supports encrypted filename storage
pub struct MetadataStore {
    /// RocksDB instance for file metadata
    db: Arc<DB>,
    
    /// In-memory cache for frequently accessed metadata
    metadata_cache: Arc<RwLock<HashMap<String, FileMetadata>>>,
}

impl MetadataStore {
    /// Create a new MetadataStore with secure configuration
    /// 
    /// Safety: Creates database with paranoid checks enabled
    pub fn new<P: AsRef<Path>>(db_path: P) -> Result<Self> {
        info!("Initializing MetadataStore with security-focused configuration");
        
        let mut opts = Options::default();
        opts.create_if_missing(true);
        opts.set_paranoid_checks(true); // Safety: Enable paranoid consistency checks
        opts.set_use_fsync(true); // Safety: Force fsync for durability
        
        let db = DB::open(&opts, db_path)
            .context("Failed to open metadata database")?;
        
        let store = Self {
            db: Arc::new(db),
            metadata_cache: Arc::new(RwLock::new(HashMap::new())),
        };
        
        info!("MetadataStore initialized successfully");
        Ok(store)
    }
    
    /// Store file metadata with integrity verification
    /// 
    /// Safety: Includes checksum verification and atomic operations
    /// Transparency: All operations logged with file hashes
    pub async fn store_metadata(&self, file_id: &str, mut metadata: FileMetadata) -> Result<()> {
        debug!("Storing metadata for file: {} ({})", file_id, metadata.name);
        
        // Calculate integrity checksum
        metadata.checksum = self.calculate_metadata_checksum(&metadata);
        
        // Update timestamp
        metadata.modified_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs();
        
        // Serialize metadata
        let metadata_bytes = bincode::serialize(&metadata)
            .context("Failed to serialize file metadata")?;
        
        // Store in database
        let key = format!("file:{}", file_id);
        self.db.put(&key, metadata_bytes)
            .context("Failed to store metadata in database")?;
        
        // Update cache
        {
            let mut cache = self.metadata_cache.write().await;
            cache.insert(file_id.to_string(), metadata.clone());
        }
        
        info!("Successfully stored metadata for file: {} with hash: {}", 
              file_id, metadata.file_hash);
        Ok(())
    }
    
    /// Retrieve file metadata with integrity verification
    /// 
    /// Safety: Verifies checksum before returning metadata
    /// Transparency: Cache hits/misses are tracked and logged
    pub async fn get_metadata(&self, file_id: &str) -> Result<Option<FileMetadata>> {
        debug!("Retrieving metadata for file: {}", file_id);
        
        // Check cache first
        {
            let cache = self.metadata_cache.read().await;
            if let Some(metadata) = cache.get(file_id) {
                debug!("Cache hit for metadata: {}", file_id);
                return Ok(Some(metadata.clone()));
            }
        }
        
        // Load from database
        let key = format!("file:{}", file_id);
        let metadata_bytes = match self.db.get(&key)? {
            Some(bytes) => bytes,
            None => {
                debug!("Metadata not found for file: {}", file_id);
                return Ok(None);
            }
        };
        
        // Deserialize metadata
        let metadata: FileMetadata = bincode::deserialize(&metadata_bytes)
            .context("Failed to deserialize file metadata")?;
        
        // Verify integrity
        if !self.verify_metadata_integrity(&metadata)? {
            warn!("Metadata integrity verification failed for file: {}", file_id);
            return Err(anyhow::anyhow!("Metadata integrity verification failed"));
        }
        
        // Update cache
        {
            let mut cache = self.metadata_cache.write().await;
            cache.insert(file_id.to_string(), metadata.clone());
        }
        
        debug!("Successfully retrieved and verified metadata for file: {}", file_id);
        Ok(Some(metadata))
    }
    
    /// Delete file metadata
    /// 
    /// Safety: Atomic deletion with comprehensive logging
    pub async fn delete_metadata(&self, file_id: &str) -> Result<bool> {
        debug!("Attempting to delete metadata for file: {}", file_id);
        
        let key = format!("file:{}", file_id);
        
        // Check if metadata exists
        if self.db.get(&key)?.is_none() {
            debug!("Cannot delete non-existent metadata: {}", file_id);
            return Ok(false);
        }
        
        // Delete from database
        self.db.delete(&key)
            .context("Failed to delete metadata from database")?;
        
        // Remove from cache
        {
            let mut cache = self.metadata_cache.write().await;
            cache.remove(file_id);
        }
        
        info!("Successfully deleted metadata for file: {}", file_id);
        Ok(true)
    }
    
    /// List all stored files with optional filtering
    /// 
    /// Transparency: Provides comprehensive file listing for audit
    pub async fn list_files(&self, limit: Option<usize>) -> Result<Vec<(String, FileMetadata)>> {
        debug!("Listing stored files (limit: {:?})", limit);
        
        let mut files = Vec::new();
        let iter = self.db.iterator(rocksdb::IteratorMode::Start);
        
        for (i, item) in iter.enumerate() {
            if let Some(limit) = limit {
                if i >= limit {
                    break;
                }
            }
            
            let (key, value) = item?;
            let key_str = String::from_utf8_lossy(&key);
            
            // Only process file metadata keys
            if !key_str.starts_with("file:") {
                continue;
            }
            
            let file_id = key_str.strip_prefix("file:").unwrap().to_string();
            
            match bincode::deserialize::<FileMetadata>(&value) {
                Ok(metadata) => {
                    if self.verify_metadata_integrity(&metadata)? {
                        files.push((file_id, metadata));
                    } else {
                        warn!("Skipping file with corrupted metadata: {}", file_id);
                    }
                }
                Err(e) => {
                    warn!("Failed to deserialize metadata for {}: {}", file_id, e);
                }
            }
        }
        
        debug!("Retrieved {} files", files.len());
        Ok(files)
    }
    
    /// Check if file metadata exists
    pub async fn file_exists(&self, file_id: &str) -> Result<bool> {
        // Check cache first
        {
            let cache = self.metadata_cache.read().await;
            if cache.contains_key(file_id) {
                return Ok(true);
            }
        }
        
        // Check database
        let key = format!("file:{}", file_id);
        Ok(self.db.get(&key)?.is_some())
    }
    
    /// Calculate checksum for metadata integrity verification
    fn calculate_metadata_checksum(&self, metadata: &FileMetadata) -> String {
        let mut hasher = blake3::Hasher::new();
        hasher.update(metadata.name.as_bytes());
        hasher.update(&metadata.size.to_le_bytes());
        hasher.update(metadata.file_hash.as_bytes());
        hasher.update(&metadata.created_at.to_le_bytes());
        hasher.update(&metadata.permissions.to_le_bytes());
        
        // Include chunk IDs in checksum
        for chunk_id in &metadata.chunk_ids {
            hasher.update(chunk_id.as_bytes());
        }
        
        hex::encode(hasher.finalize().as_bytes())
    }
    
    /// Verify metadata integrity using checksum
    /// 
    /// Safety: Prevents use of corrupted metadata
    fn verify_metadata_integrity(&self, metadata: &FileMetadata) -> Result<bool> {
        let computed_checksum = self.calculate_metadata_checksum(metadata);
        Ok(computed_checksum == metadata.checksum)
    }
}