use anyhow::{Context, Result};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn, error};

use crate::storage::{
    chunk_store::{ChunkStore, StorageStats},
    metadata_store::{MetadataStore, FileMetadata as FileMetaData},
    file_chunker::{FileChunker, FileMetadata as ChunkerFileMetadata, ChunkInfo},
};

/// Storage capacity configuration and limits
/// 
/// Safety: Enforces storage limits to prevent disk exhaustion
#[derive(Debug, Clone)]
pub struct StorageConfig {
    /// Maximum storage capacity in bytes
    pub max_capacity: u64,
    
    /// Warning threshold (% of capacity)
    pub warning_threshold: f64,
    
    /// Critical threshold (% of capacity) - stop accepting new data
    pub critical_threshold: f64,
    
    /// Default chunk size for file splitting
    pub default_chunk_size: usize,
    
    /// Maximum file size to accept
    pub max_file_size: u64,
    
    /// Enable automatic garbage collection
    pub enable_gc: bool,
    
    /// Garbage collection interval in seconds
    pub gc_interval: u64,
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            max_capacity: 10 * 1024 * 1024 * 1024, // 10GB default
            warning_threshold: 0.8,  // 80%
            critical_threshold: 0.95, // 95%
            default_chunk_size: 1024 * 1024, // 1MB
            max_file_size: 1024 * 1024 * 1024, // 1GB max file
            enable_gc: true,
            gc_interval: 3600, // 1 hour
        }
    }
}

/// Comprehensive storage capacity metrics
/// 
/// Transparency: Detailed capacity tracking for monitoring
#[derive(Debug, Clone)]
pub struct CapacityInfo {
    /// Total configured capacity
    pub total_capacity: u64,
    
    /// Currently used space
    pub used_space: u64,
    
    /// Available space
    pub available_space: u64,
    
    /// Usage percentage (0.0 to 1.0)
    pub usage_percentage: f64,
    
    /// Number of stored files
    pub file_count: u64,
    
    /// Number of stored chunks
    pub chunk_count: u64,
    
    /// Average chunk size
    pub avg_chunk_size: u64,
    
    /// Storage efficiency (deduplication ratio)
    pub efficiency_ratio: f64,
}

/// Main storage manager coordinating all storage operations
/// 
/// Safety: Enforces capacity limits and coordinates atomic operations
/// Transparency: Comprehensive logging and metrics collection
/// Privacy: Handles encrypted storage and secure deletion
pub struct StorageManager {
    /// Chunk storage backend
    chunk_store: Arc<ChunkStore>,
    
    /// Metadata storage backend  
    metadata_store: Arc<MetadataStore>,
    
    /// File chunking system
    file_chunker: Arc<FileChunker>,
    
    /// Storage configuration
    config: StorageConfig,
    
    /// Base storage path
    base_path: PathBuf,
    
    /// Capacity tracking
    capacity_info: Arc<RwLock<CapacityInfo>>,
}

impl StorageManager {
    /// Create a new StorageManager with specified configuration
    /// 
    /// Safety: Initializes all storage backends with security settings
    pub async fn new<P: AsRef<Path>>(base_path: P, config: StorageConfig) -> Result<Self> {
        let base_path = base_path.as_ref().to_path_buf();
        info!("Initializing StorageManager at: {:?}", base_path);
        
        // Create subdirectories for different storage types
        let chunk_path = base_path.join("chunks");
        let metadata_path = base_path.join("metadata");
        
        std::fs::create_dir_all(&chunk_path)
            .context("Failed to create chunk storage directory")?;
        std::fs::create_dir_all(&metadata_path)
            .context("Failed to create metadata storage directory")?;
        
        // Initialize storage backends
        let chunk_store = Arc::new(ChunkStore::new(&chunk_path)
            .context("Failed to initialize chunk store")?);
        
        let metadata_store = Arc::new(MetadataStore::new(&metadata_path)
            .context("Failed to initialize metadata store")?);
        
        let file_chunker = Arc::new(FileChunker::new(Some(config.default_chunk_size))?);
        
        // Initialize capacity tracking
        let capacity_info = Arc::new(RwLock::new(CapacityInfo {
            total_capacity: config.max_capacity,
            used_space: 0,
            available_space: config.max_capacity,
            usage_percentage: 0.0,
            file_count: 0,
            chunk_count: 0,
            avg_chunk_size: 0,
            efficiency_ratio: 1.0,
        }));
        
        let manager = Self {
            chunk_store,
            metadata_store,
            file_chunker,
            config,
            base_path,
            capacity_info,
        };
        
        // Update capacity information from existing storage
        manager.refresh_capacity_info().await?;
        
        info!("StorageManager initialized successfully");
        Ok(manager)
    }
    
    /// Store a file with automatic chunking and deduplication
    /// 
    /// Safety: Enforces capacity limits and validates file integrity
    /// Transparency: Logs all storage operations with file hashes
    pub async fn store_file(&self, file_id: &str, data: &[u8], filename: &str) -> Result<String> {
        info!("Storing file: {} ({} bytes) as {}", filename, data.len(), file_id);
        
        // Check capacity limits before storing
        self.check_capacity_limits(data.len() as u64).await?;
        
        // Check file size limit
        if data.len() as u64 > self.config.max_file_size {
            return Err(anyhow::anyhow!(
                "File size ({} bytes) exceeds maximum allowed size ({} bytes)",
                data.len(),
                self.config.max_file_size
            ));
        }
        
        // Chunk the file
        let metadata = self.file_chunker.chunk_bytes(data, file_id.to_string(), filename.to_string())?;
        let file_hash = metadata.file_hash.clone();
        
        debug!("File chunked into {} pieces", metadata.chunks.len());
        
        // Store all chunks with deduplication
        let mut chunk_ids = Vec::new();
        let mut stored_chunks = 0;
        
        // We need to get the actual chunk data from the original data
        for chunk_info in &metadata.chunks {
            let start = chunk_info.offset as usize;
            let end = start + chunk_info.size as usize;
            let chunk_data = &data[start..end];
            
            let chunk_hash = self.chunk_store.store_chunk(&chunk_info.chunk_id, chunk_data).await?;
            chunk_ids.push(chunk_info.chunk_id.clone());
            stored_chunks += 1;
            
            debug!("Stored chunk {}/{}: {} (hash: {})", 
                   stored_chunks, metadata.chunks.len(), chunk_info.chunk_id, chunk_hash);
        }
        
        // Create file metadata for storage
        let storage_metadata = FileMetaData {
            name: filename.to_string(),
            size: data.len() as u64,
            mime_type: self.detect_mime_type(data, filename),
            file_hash: file_hash.clone(),
            chunk_ids,
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)?
                .as_secs(),
            modified_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)?
                .as_secs(),
            permissions: 0o644, // Default read-write for owner, read for others
            checksum: String::new(), // Will be calculated by metadata store
        };
        
        // Store metadata
        self.metadata_store.store_metadata(file_id, storage_metadata).await?;
        
        // Update capacity information
        self.refresh_capacity_info().await?;
        
        info!("Successfully stored file: {} with hash: {}", file_id, file_hash);
        Ok(file_hash)
    }
    
    /// Retrieve a complete file by reconstructing from chunks
    /// 
    /// Safety: Verifies integrity of all chunks before reconstruction
    /// Transparency: Logs retrieval operations and verification steps
    pub async fn retrieve_file(&self, file_id: &str) -> Result<Option<Vec<u8>>> {
        debug!("Retrieving file: {}", file_id);
        
        // Get file metadata
        let metadata = match self.metadata_store.get_metadata(file_id).await? {
            Some(meta) => meta,
            None => {
                debug!("File metadata not found: {}", file_id);
                return Ok(None);
            }
        };
        
        info!("Retrieving file: {} ({} bytes, {} chunks)", 
              metadata.name, metadata.size, metadata.chunk_ids.len());
        
        // Retrieve all chunks
        let mut chunk_data = Vec::new();
        for chunk_id in &metadata.chunk_ids {
            match self.chunk_store.retrieve_chunk(chunk_id).await? {
                Some(data) => chunk_data.push(data),
                None => {
                    error!("Missing chunk {} for file {}", chunk_id, file_id);
                    return Err(anyhow::anyhow!(
                        "File reconstruction failed: missing chunk {}", chunk_id
                    ));
                }
            }
        }
        
        // Create chunker metadata for reconstruction
        let chunker_metadata = ChunkerFileMetadata {
            file_id: file_id.to_string(),
            filename: metadata.name.clone(),
            total_size: metadata.size,
            file_hash: metadata.file_hash.clone(),
            chunks: {
                let mut offset = 0u64;
                metadata.chunk_ids.iter().enumerate().map(|(i, chunk_id)| {
                    // Calculate the actual chunk hash
                    let mut hasher = Sha256::new();
                    hasher.update(&chunk_data[i]);
                    let chunk_hash = hex::encode(hasher.finalize());
                    
                    let chunk_info = ChunkInfo {
                        chunk_id: chunk_id.clone(),
                        hash: chunk_hash,
                        size: chunk_data[i].len() as u64,
                        index: i as u32,
                        offset,
                    };
                    
                    offset += chunk_data[i].len() as u64;
                    chunk_info
                }).collect()
            },
            chunk_size: self.config.default_chunk_size,
            mime_type: metadata.mime_type.clone(),
            created_at: metadata.created_at,
        };
        
        // Reconstruct file
        let reconstructed = self.file_chunker.reconstruct_file(&chunker_metadata, chunk_data)?;
        
        // Verify file integrity
        let mut hasher = Sha256::new();
        hasher.update(&reconstructed);
        let computed_hash = hex::encode(hasher.finalize());
        if computed_hash != metadata.file_hash {
            error!("File integrity verification failed for {}: hash mismatch", file_id);
            return Err(anyhow::anyhow!(
                "File integrity verification failed: hash mismatch"
            ));
        }
        
        info!("Successfully retrieved and verified file: {}", file_id);
        Ok(Some(reconstructed))
    }
    
    /// Delete a file and its associated chunks (with reference counting)
    /// 
    /// Safety: Uses atomic operations and reference counting
    /// Transparency: Comprehensive logging of deletion process
    pub async fn delete_file(&self, file_id: &str) -> Result<bool> {
        info!("Deleting file: {}", file_id);
        
        // Get file metadata first
        let metadata = match self.metadata_store.get_metadata(file_id).await? {
            Some(meta) => meta,
            None => {
                debug!("Cannot delete non-existent file: {}", file_id);
                return Ok(false);
            }
        };
        
        // Delete all associated chunks (with reference counting)
        let mut deleted_chunks = 0;
        for chunk_id in &metadata.chunk_ids {
            if self.chunk_store.delete_chunk(chunk_id).await? {
                deleted_chunks += 1;
                debug!("Deleted chunk: {}", chunk_id);
            } else {
                debug!("Chunk {} still has references, not deleted", chunk_id);
            }
        }
        
        // Delete metadata
        let metadata_deleted = self.metadata_store.delete_metadata(file_id).await?;
        
        // Update capacity information
        self.refresh_capacity_info().await?;
        
        info!("Successfully deleted file: {} (deleted {} chunks)", 
              file_id, deleted_chunks);
        Ok(metadata_deleted)
    }
    
    /// List all stored files with metadata
    /// 
    /// Transparency: Provides comprehensive file listing for audit
    pub async fn list_files(&self, limit: Option<usize>) -> Result<Vec<(String, FileMetaData)>> {
        self.metadata_store.list_files(limit).await
    }
    
    /// Check if a file exists
    pub async fn file_exists(&self, file_id: &str) -> Result<bool> {
        self.metadata_store.file_exists(file_id).await
    }
    
    /// Retrieve a specific chunk by ID (for P2P sharing)
    /// 
    /// Safety: Verifies chunk integrity before returning
    /// Transparency: Logs all chunk access attempts
    pub async fn retrieve_chunk(&self, chunk_id: &str) -> Result<Option<Vec<u8>>> {
        debug!("Retrieving chunk for P2P sharing: {}", chunk_id);
        self.chunk_store.retrieve_chunk(chunk_id).await
    }
    
    /// Store a chunk directly (for P2P receiving)
    /// 
    /// Safety: Includes full integrity verification
    /// Transparency: Logs all chunk storage operations
    pub async fn store_chunk(&self, chunk_id: &str, data: &[u8]) -> Result<String> {
        debug!("Storing chunk from P2P: {} ({} bytes)", chunk_id, data.len());
        self.chunk_store.store_chunk(chunk_id, data).await
    }
    
    /// Get current storage capacity information
    /// 
    /// Transparency: Real-time capacity metrics for monitoring
    pub async fn get_capacity_info(&self) -> CapacityInfo {
        let info = self.capacity_info.read().await;
        info.clone()
    }
    
    /// Get detailed storage statistics
    /// 
    /// Transparency: Comprehensive storage metrics
    pub async fn get_storage_stats(&self) -> Result<StorageStats> {
        Ok(self.chunk_store.get_stats().await)
    }
    
    /// Check if storage has enough capacity for new data
    /// 
    /// Safety: Prevents storage exhaustion
    async fn check_capacity_limits(&self, required_space: u64) -> Result<()> {
        let info = self.capacity_info.read().await;
        
        // Check if we have enough space
        if info.available_space < required_space {
            return Err(anyhow::anyhow!(
                "Insufficient storage space: required {} bytes, available {} bytes",
                required_space, info.available_space
            ));
        }
        
        // Check if we're approaching critical threshold
        let projected_usage = (info.used_space + required_space) as f64 / info.total_capacity as f64;
        
        if projected_usage > self.config.critical_threshold {
            return Err(anyhow::anyhow!(
                "Storage usage would exceed critical threshold: {:.1}% > {:.1}%",
                projected_usage * 100.0,
                self.config.critical_threshold * 100.0
            ));
        }
        
        // Warn if approaching warning threshold
        if projected_usage > self.config.warning_threshold {
            warn!("Storage usage approaching warning threshold: {:.1}%", 
                  projected_usage * 100.0);
        }
        
        Ok(())
    }
    
    /// Refresh capacity information from storage backends
    /// 
    /// Transparency: Accurate real-time capacity tracking
    async fn refresh_capacity_info(&self) -> Result<()> {
        let chunk_stats = self.chunk_store.get_stats().await;
        let files = self.metadata_store.list_files(None).await?;
        
        let used_space = chunk_stats.total_size;
        let available_space = self.config.max_capacity.saturating_sub(used_space);
        let usage_percentage = used_space as f64 / self.config.max_capacity as f64;
        
        // Calculate efficiency ratio (deduplication benefits)
        let logical_size: u64 = files.iter().map(|(_, meta)| meta.size).sum();
        let efficiency_ratio = if used_space > 0 {
            logical_size as f64 / used_space as f64
        } else {
            1.0
        };
        
        let avg_chunk_size = if chunk_stats.total_chunks > 0 {
            chunk_stats.total_size / chunk_stats.total_chunks
        } else {
            0
        };
        
        let mut info = self.capacity_info.write().await;
        *info = CapacityInfo {
            total_capacity: self.config.max_capacity,
            used_space,
            available_space,
            usage_percentage,
            file_count: files.len() as u64,
            chunk_count: chunk_stats.total_chunks,
            avg_chunk_size,
            efficiency_ratio,
        };
        
        debug!("Capacity info updated: {:.1}% used ({} files, {} chunks)", 
               usage_percentage * 100.0, info.file_count, info.chunk_count);
        
        Ok(())
    }
    
    /// Simple MIME type detection based on file extension and content
    fn detect_mime_type(&self, data: &[u8], filename: &str) -> Option<String> {
        // Check magic bytes for common formats
        if data.len() >= 4 {
            match &data[0..4] {
                [0xFF, 0xD8, 0xFF, _] => return Some("image/jpeg".to_string()),
                [0x89, 0x50, 0x4E, 0x47] => return Some("image/png".to_string()),
                [0x47, 0x49, 0x46, _] => return Some("image/gif".to_string()),
                [0x25, 0x50, 0x44, 0x46] => return Some("application/pdf".to_string()),
                _ => {}
            }
        }
        
        // Fallback to extension-based detection
        if let Some(extension) = Path::new(filename).extension() {
            match extension.to_str()?.to_lowercase().as_str() {
                "txt" => Some("text/plain".to_string()),
                "json" => Some("application/json".to_string()),
                "xml" => Some("application/xml".to_string()),
                "html" => Some("text/html".to_string()),
                "css" => Some("text/css".to_string()),
                "js" => Some("application/javascript".to_string()),
                "mp4" => Some("video/mp4".to_string()),
                "mp3" => Some("audio/mpeg".to_string()),
                "zip" => Some("application/zip".to_string()),
                _ => None,
            }
        } else {
            None
        }
    }

    /// Get chunk IDs for a file (needed for coordinator integration)
    ///
    /// Returns the list of chunk IDs that comprise the given file
    pub async fn get_file_chunks(&self, file_id: &str) -> Result<Option<Vec<String>>> {
        debug!("Getting chunk list for file: {}", file_id);

        match self.metadata_store.get_file(file_id).await? {
            Some(metadata) => {
                Ok(Some(metadata.chunk_ids))
            }
            None => {
                debug!("File not found: {}", file_id);
                Ok(None)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    
    #[tokio::test]
    async fn test_storage_manager_creation() {
        let temp_dir = tempdir().unwrap();
        let config = StorageConfig::default();
        
        let manager = StorageManager::new(temp_dir.path(), config).await.unwrap();
        let capacity = manager.get_capacity_info().await;
        
        assert_eq!(capacity.used_space, 0);
        assert_eq!(capacity.file_count, 0);
        assert_eq!(capacity.chunk_count, 0);
    }
    
    #[tokio::test]
    async fn test_store_and_retrieve_file() {
        let temp_dir = tempdir().unwrap();
        let config = StorageConfig::default();
        let manager = StorageManager::new(temp_dir.path(), config).await.unwrap();
        
        let file_id = "test-file-1";
        let filename = "test.txt";
        let data = b"Hello, ZephyrFS! This is a test file with some content.";
        
        // Store file
        let hash = manager.store_file(file_id, data, filename).await.unwrap();
        assert!(!hash.is_empty());
        
        // Verify existence
        assert!(manager.file_exists(file_id).await.unwrap());
        
        // Retrieve file
        let retrieved = manager.retrieve_file(file_id).await.unwrap().unwrap();
        assert_eq!(retrieved, data);
        
        // Check capacity info
        let capacity = manager.get_capacity_info().await;
        assert_eq!(capacity.file_count, 1);
        assert!(capacity.used_space > 0);
        assert!(capacity.usage_percentage > 0.0);
    }
    
    #[tokio::test]
    async fn test_capacity_limits() {
        let temp_dir = tempdir().unwrap();
        let mut config = StorageConfig::default();
        config.max_capacity = 100; // Very small capacity
        config.critical_threshold = 0.5; // 50%
        
        let manager = StorageManager::new(temp_dir.path(), config).await.unwrap();
        
        let file_id = "large-file";
        let filename = "large.txt";
        let data = vec![0u8; 200]; // Larger than capacity
        
        // Should fail due to capacity limits
        let result = manager.store_file(file_id, &data, filename).await;
        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("Insufficient storage space") || error_msg.contains("Storage usage would exceed"));
    }
    
    #[tokio::test]
    async fn test_file_deletion() {
        let temp_dir = tempdir().unwrap();
        let config = StorageConfig::default();
        let manager = StorageManager::new(temp_dir.path(), config).await.unwrap();
        
        let file_id = "delete-test";
        let filename = "delete.txt";
        let data = b"File to be deleted";
        
        // Store file
        manager.store_file(file_id, data, filename).await.unwrap();
        assert!(manager.file_exists(file_id).await.unwrap());
        
        // Delete file
        let deleted = manager.delete_file(file_id).await.unwrap();
        assert!(deleted);
        assert!(!manager.file_exists(file_id).await.unwrap());
        
        // Verify capacity updated
        let capacity = manager.get_capacity_info().await;
        assert_eq!(capacity.file_count, 0);
    }
    
    #[tokio::test]
    async fn test_mime_type_detection() {
        let temp_dir = tempdir().unwrap();
        let config = StorageConfig::default();
        let manager = StorageManager::new(temp_dir.path(), config).await.unwrap();
        
        // Test JPEG magic bytes
        let jpeg_data = vec![0xFF, 0xD8, 0xFF, 0xE0, 0x00, 0x10];
        let mime = manager.detect_mime_type(&jpeg_data, "test.jpg");
        assert_eq!(mime, Some("image/jpeg".to_string()));
        
        // Test extension-based detection
        let text_data = b"plain text content";
        let mime = manager.detect_mime_type(text_data, "test.txt");
        assert_eq!(mime, Some("text/plain".to_string()));
    }
}