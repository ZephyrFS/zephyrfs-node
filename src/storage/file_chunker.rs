use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::io::{Read, Seek, SeekFrom};
use tracing::{debug, info, warn};

/// File chunking configuration following ZephyrFS architecture
/// 
/// Safety: Chunk sizes are validated and bounded to prevent memory exhaustion
/// Transparency: All chunking operations are logged with metadata
const DEFAULT_CHUNK_SIZE: usize = 1024 * 1024; // 1MB
const MIN_CHUNK_SIZE: usize = 64 * 1024;       // 64KB minimum
const MAX_CHUNK_SIZE: usize = 16 * 1024 * 1024; // 16MB maximum

/// Metadata for a file chunk
/// 
/// Privacy: Contains only structural information, no content
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ChunkInfo {
    /// Unique identifier for the chunk
    pub chunk_id: String,
    
    /// SHA-256 hash of chunk content for integrity verification
    pub hash: String,
    
    /// Size of the chunk in bytes
    pub size: u64,
    
    /// Position of this chunk within the original file
    pub index: u32,
    
    /// Offset within the original file
    pub offset: u64,
}

/// Metadata for a chunked file
/// 
/// Transparency: Complete file reconstruction information available
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileMetadata {
    /// Original file identifier
    pub file_id: String,
    
    /// Original filename (for user convenience)
    pub filename: String,
    
    /// Total size of original file
    pub total_size: u64,
    
    /// SHA-256 hash of complete file for integrity verification
    pub file_hash: String,
    
    /// Ordered list of chunks
    pub chunks: Vec<ChunkInfo>,
    
    /// Chunk size used for this file
    pub chunk_size: usize,
    
    /// MIME type if detected
    pub mime_type: Option<String>,
    
    /// Creation timestamp
    pub created_at: u64,
}

/// File chunking engine with security and integrity focus
/// 
/// Safety: All operations include bounds checking and validation
/// Privacy: Original file content is never stored unencrypted
pub struct FileChunker {
    chunk_size: usize,
}

impl FileChunker {
    /// Create a new FileChunker with specified chunk size
    /// 
    /// Safety: Validates chunk size is within safe bounds
    pub fn new(chunk_size: Option<usize>) -> Result<Self> {
        let chunk_size = chunk_size.unwrap_or(DEFAULT_CHUNK_SIZE);
        
        if chunk_size < MIN_CHUNK_SIZE || chunk_size > MAX_CHUNK_SIZE {
            anyhow::bail!(
                "Chunk size {} is outside safe bounds [{}, {}]",
                chunk_size, MIN_CHUNK_SIZE, MAX_CHUNK_SIZE
            );
        }
        
        info!("Initialized FileChunker with chunk size: {} bytes", chunk_size);
        Ok(Self { chunk_size })
    }
    
    /// Create default FileChunker with 1MB chunks
    pub fn default() -> Self {
        Self::new(None).expect("Default chunk size should always be valid")
    }
    
    /// Chunk a file from a reader
    /// 
    /// Safety: Uses bounded reads to prevent memory exhaustion
    /// Transparency: All chunking steps are logged
    pub fn chunk_file<R: Read + Seek>(
        &self,
        mut reader: R,
        file_id: String,
        filename: String,
    ) -> Result<FileMetadata> {
        info!("Chunking file: {} (ID: {})", filename, file_id);
        
        // Get total file size
        let total_size = reader.seek(SeekFrom::End(0))
            .context("Failed to determine file size")?;
        reader.seek(SeekFrom::Start(0))
            .context("Failed to seek to file start")?;
        
        if total_size == 0 {
            warn!("Attempting to chunk empty file: {}", filename);
            return Ok(FileMetadata {
                file_id,
                filename,
                total_size: 0,
                file_hash: self.calculate_empty_file_hash(),
                chunks: vec![],
                chunk_size: self.chunk_size,
                mime_type: None,
                created_at: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)?
                    .as_secs(),
            });
        }
        
        let mut chunks = Vec::new();
        let mut file_hasher = Sha256::new();
        let mut buffer = vec![0u8; self.chunk_size];
        let mut total_read = 0u64;
        let mut chunk_index = 0u32;
        
        debug!("Starting to read file in {} byte chunks", self.chunk_size);
        
        loop {
            let bytes_read = reader.read(&mut buffer)
                .context("Failed to read from file")?;
            
            if bytes_read == 0 {
                break; // End of file
            }
            
            let chunk_data = &buffer[..bytes_read];
            
            // Update file hash with chunk data
            file_hasher.update(chunk_data);
            
            // Calculate chunk hash
            let chunk_hash = self.calculate_chunk_hash(chunk_data);
            
            // Generate chunk ID (content-addressable)
            let chunk_id = format!("chunk_{}", &chunk_hash[..16]);
            
            let chunk_info = ChunkInfo {
                chunk_id: chunk_id.clone(),
                hash: chunk_hash,
                size: bytes_read as u64,
                index: chunk_index,
                offset: total_read,
            };
            
            chunks.push(chunk_info);
            total_read += bytes_read as u64;
            chunk_index += 1;
            
            debug!(
                "Created chunk {} (index: {}, size: {} bytes, offset: {})",
                chunk_id, chunk_index - 1, bytes_read, total_read - bytes_read as u64
            );
        }
        
        // Calculate final file hash
        let file_hash = hex::encode(file_hasher.finalize());
        
        let metadata = FileMetadata {
            file_id,
            filename: filename.clone(),
            total_size,
            file_hash,
            chunks,
            chunk_size: self.chunk_size,
            mime_type: self.detect_mime_type(&filename),
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)?
                .as_secs(),
        };
        
        info!(
            "Successfully chunked file {} into {} chunks (total: {} bytes)",
            filename, metadata.chunks.len(), total_size
        );
        
        Ok(metadata)
    }
    
    /// Chunk data from a byte slice
    /// 
    /// Safety: Memory-bounded operation suitable for smaller files
    pub fn chunk_bytes(
        &self,
        data: &[u8],
        file_id: String,
        filename: String,
    ) -> Result<FileMetadata> {
        info!("Chunking {} bytes of data for file: {}", data.len(), filename);
        
        if data.is_empty() {
            return Ok(FileMetadata {
                file_id,
                filename,
                total_size: 0,
                file_hash: self.calculate_empty_file_hash(),
                chunks: vec![],
                chunk_size: self.chunk_size,
                mime_type: None,
                created_at: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)?
                    .as_secs(),
            });
        }
        
        let mut chunks = Vec::new();
        let mut file_hasher = Sha256::new();
        file_hasher.update(data);
        
        for (chunk_index, chunk_data) in data.chunks(self.chunk_size).enumerate() {
            let chunk_hash = self.calculate_chunk_hash(chunk_data);
            let chunk_id = format!("chunk_{}", &chunk_hash[..16]);
            let offset = (chunk_index * self.chunk_size) as u64;
            
            let chunk_info = ChunkInfo {
                chunk_id: chunk_id.clone(),
                hash: chunk_hash,
                size: chunk_data.len() as u64,
                index: chunk_index as u32,
                offset,
            };
            
            chunks.push(chunk_info);
            
            debug!(
                "Created chunk {} (index: {}, size: {} bytes)",
                chunk_id, chunk_index, chunk_data.len()
            );
        }
        
        let file_hash = hex::encode(file_hasher.finalize());
        
        let metadata = FileMetadata {
            file_id,
            filename: filename.clone(),
            total_size: data.len() as u64,
            file_hash,
            chunks,
            chunk_size: self.chunk_size,
            mime_type: self.detect_mime_type(&filename),
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)?
                .as_secs(),
        };
        
        info!(
            "Successfully chunked {} bytes into {} chunks",
            data.len(), metadata.chunks.len()
        );
        
        Ok(metadata)
    }
    
    /// Reconstruct file data from chunks
    /// 
    /// Safety: Validates chunk order and integrity before reconstruction
    /// Transparency: Reconstruction process is fully logged
    pub fn reconstruct_file(&self, metadata: &FileMetadata, chunk_data: Vec<Vec<u8>>) -> Result<Vec<u8>> {
        info!("Reconstructing file: {} ({} chunks)", metadata.filename, metadata.chunks.len());
        
        if chunk_data.len() != metadata.chunks.len() {
            anyhow::bail!(
                "Chunk data length {} doesn't match metadata chunks {}",
                chunk_data.len(), metadata.chunks.len()
            );
        }
        
        // Verify all chunks are present and in order
        for (i, (chunk_info, data)) in metadata.chunks.iter().zip(chunk_data.iter()).enumerate() {
            if chunk_info.index as usize != i {
                anyhow::bail!("Chunk {} is out of order (expected index {})", chunk_info.chunk_id, i);
            }
            
            if data.len() as u64 != chunk_info.size {
                anyhow::bail!(
                    "Chunk {} size mismatch: expected {}, got {}",
                    chunk_info.chunk_id, chunk_info.size, data.len()
                );
            }
            
            // Verify chunk hash
            let calculated_hash = self.calculate_chunk_hash(data);
            if calculated_hash != chunk_info.hash {
                anyhow::bail!("Chunk {} hash verification failed", chunk_info.chunk_id);
            }
        }
        
        // Reconstruct file
        let mut reconstructed = Vec::with_capacity(metadata.total_size as usize);
        for data in chunk_data {
            reconstructed.extend_from_slice(&data);
        }
        
        // Verify reconstructed file hash
        let mut file_hasher = Sha256::new();
        file_hasher.update(&reconstructed);
        let calculated_hash = hex::encode(file_hasher.finalize());
        
        if calculated_hash != metadata.file_hash {
            anyhow::bail!("Reconstructed file hash verification failed");
        }
        
        info!("Successfully reconstructed file: {} ({} bytes)", metadata.filename, reconstructed.len());
        Ok(reconstructed)
    }
    
    /// Calculate SHA-256 hash of chunk data
    fn calculate_chunk_hash(&self, data: &[u8]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(data);
        hex::encode(hasher.finalize())
    }
    
    /// Calculate hash for empty file (consistent across all empty files)
    fn calculate_empty_file_hash(&self) -> String {
        let hasher = Sha256::new();
        hex::encode(hasher.finalize())
    }
    
    /// Simple MIME type detection based on file extension
    /// 
    /// Privacy: Only uses filename extension, no content inspection
    fn detect_mime_type(&self, filename: &str) -> Option<String> {
        let extension = std::path::Path::new(filename)
            .extension()?
            .to_str()?
            .to_lowercase();
            
        match extension.as_str() {
            "txt" | "md" => Some("text/plain".to_string()),
            "html" | "htm" => Some("text/html".to_string()),
            "json" => Some("application/json".to_string()),
            "pdf" => Some("application/pdf".to_string()),
            "jpg" | "jpeg" => Some("image/jpeg".to_string()),
            "png" => Some("image/png".to_string()),
            "gif" => Some("image/gif".to_string()),
            "zip" => Some("application/zip".to_string()),
            "tar" => Some("application/x-tar".to_string()),
            "gz" => Some("application/gzip".to_string()),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;
    
    #[test]
    fn test_file_chunker_creation() {
        let chunker = FileChunker::default();
        assert_eq!(chunker.chunk_size, DEFAULT_CHUNK_SIZE);
        
        let custom_chunker = FileChunker::new(Some(512 * 1024)).unwrap();
        assert_eq!(custom_chunker.chunk_size, 512 * 1024);
        
        // Test invalid chunk sizes
        assert!(FileChunker::new(Some(1024)).is_err()); // Too small
        assert!(FileChunker::new(Some(32 * 1024 * 1024)).is_err()); // Too large
    }
    
    #[test]
    fn test_chunk_empty_data() {
        let chunker = FileChunker::default();
        let metadata = chunker.chunk_bytes(
            &[],
            "empty-test".to_string(),
            "empty.txt".to_string(),
        ).unwrap();
        
        assert_eq!(metadata.total_size, 0);
        assert!(metadata.chunks.is_empty());
        assert!(!metadata.file_hash.is_empty());
    }
    
    #[test]
    fn test_chunk_small_data() {
        let chunker = FileChunker::new(Some(128 * 1024)).unwrap(); // 128KB chunks
        let test_data = b"Hello, ZephyrFS! This is a test file for chunking.";
        
        let metadata = chunker.chunk_bytes(
            test_data,
            "small-test".to_string(),
            "test.txt".to_string(),
        ).unwrap();
        
        assert_eq!(metadata.total_size, test_data.len() as u64);
        assert_eq!(metadata.chunks.len(), 1); // Should fit in one chunk
        assert_eq!(metadata.chunks[0].size, test_data.len() as u64);
        assert_eq!(metadata.chunks[0].index, 0);
        assert_eq!(metadata.chunks[0].offset, 0);
        assert_eq!(metadata.mime_type, Some("text/plain".to_string()));
    }
    
    #[test]
    fn test_chunk_large_data() {
        let chunker = FileChunker::new(Some(64 * 1024)).unwrap(); // 64KB chunks for testing
        let test_data = vec![42u8; 200 * 1024]; // 200KB of data
        
        let metadata = chunker.chunk_bytes(
            &test_data,
            "large-test".to_string(),
            "large.bin".to_string(),
        ).unwrap();
        
        assert_eq!(metadata.total_size, 200 * 1024);
        assert_eq!(metadata.chunks.len(), 4); // Should split into 4 chunks
        
        // Verify chunk sizes
        assert_eq!(metadata.chunks[0].size, 64 * 1024);
        assert_eq!(metadata.chunks[1].size, 64 * 1024);
        assert_eq!(metadata.chunks[2].size, 64 * 1024);
        assert_eq!(metadata.chunks[3].size, 8 * 1024); // Remainder
        
        // Verify offsets
        assert_eq!(metadata.chunks[0].offset, 0);
        assert_eq!(metadata.chunks[1].offset, 64 * 1024);
        assert_eq!(metadata.chunks[2].offset, 128 * 1024);
        assert_eq!(metadata.chunks[3].offset, 192 * 1024);
    }
    
    #[test]
    fn test_file_reconstruction() {
        let chunker = FileChunker::new(Some(64 * 1024)).unwrap();
        let original_data = b"The quick brown fox jumps over the lazy dog. ".repeat(50);
        
        // Chunk the data
        let metadata = chunker.chunk_bytes(
            &original_data,
            "reconstruction-test".to_string(),
            "test.txt".to_string(),
        ).unwrap();
        
        // Extract chunk data (simulating retrieval from storage)
        let mut chunk_data = Vec::new();
        let mut offset = 0;
        for chunk_info in &metadata.chunks {
            let end = offset + chunk_info.size as usize;
            chunk_data.push(original_data[offset..end].to_vec());
            offset = end;
        }
        
        // Reconstruct the file
        let reconstructed = chunker.reconstruct_file(&metadata, chunk_data).unwrap();
        
        assert_eq!(reconstructed, original_data);
    }
    
    #[test]
    fn test_chunk_reader() {
        let chunker = FileChunker::new(Some(64 * 1024)).unwrap();
        let test_data = b"This is test data for the reader-based chunking functionality.";
        let mut cursor = Cursor::new(test_data);
        
        let metadata = chunker.chunk_file(
            &mut cursor,
            "reader-test".to_string(),
            "reader.txt".to_string(),
        ).unwrap();
        
        assert_eq!(metadata.total_size, test_data.len() as u64);
        assert_eq!(metadata.chunks.len(), 1); // Small data fits in one chunk
        assert!(!metadata.file_hash.is_empty());
    }
    
    #[test]
    fn test_hash_consistency() {
        let chunker = FileChunker::default();
        let test_data = b"Consistent hashing test data";
        
        // Chunk the same data twice
        let metadata1 = chunker.chunk_bytes(
            test_data,
            "hash-test-1".to_string(),
            "hash.txt".to_string(),
        ).unwrap();
        
        let metadata2 = chunker.chunk_bytes(
            test_data,
            "hash-test-2".to_string(),
            "hash.txt".to_string(),
        ).unwrap();
        
        // File hashes should be identical
        assert_eq!(metadata1.file_hash, metadata2.file_hash);
        assert_eq!(metadata1.chunks[0].hash, metadata2.chunks[0].hash);
    }
    
    #[test]
    fn test_mime_type_detection() {
        let chunker = FileChunker::default();
        
        assert_eq!(chunker.detect_mime_type("test.txt"), Some("text/plain".to_string()));
        assert_eq!(chunker.detect_mime_type("doc.pdf"), Some("application/pdf".to_string()));
        assert_eq!(chunker.detect_mime_type("image.png"), Some("image/png".to_string()));
        assert_eq!(chunker.detect_mime_type("unknown.xyz"), None);
    }
    
    #[test]
    fn test_chunk_integrity_verification() {
        let chunker = FileChunker::new(Some(64 * 1024)).unwrap();
        let test_data = vec![1u8; 2048]; // 2KB data
        
        let metadata = chunker.chunk_bytes(
            &test_data,
            "integrity-test".to_string(),
            "integrity.bin".to_string(),
        ).unwrap();
        
        // With 64KB chunks, 2KB data will be in a single chunk
        let chunk_data = vec![test_data.clone()];
        
        // Should reconstruct successfully
        let reconstructed = chunker.reconstruct_file(&metadata, chunk_data).unwrap();
        assert_eq!(reconstructed, test_data);
        
        // Test with corrupted chunk
        let corrupted_chunk_data = vec![
            vec![0u8; 2048], // Corrupted chunk (same size but different data)
        ];
        
        // Should fail reconstruction
        assert!(chunker.reconstruct_file(&metadata, corrupted_chunk_data).is_err());
    }
}