// Storage module - Phase 1.2 implementation
// 
// Safety: Storage implements encryption at rest by default
// Privacy: All data stored is encrypted with user-controlled keys  
// Transparency: Storage operations are logged for audit trail

pub mod chunk_store;
pub mod metadata_store;
pub mod file_chunker;
pub mod storage_manager;
pub mod encrypted_chunk_store;
pub mod enhanced_storage_manager;

// Re-export main storage interfaces
pub use chunk_store::{ChunkStore, ChunkMetadata, StorageStats};
pub use metadata_store::{MetadataStore, FileMetadata};
pub use file_chunker::{FileChunker, ChunkInfo};
pub use storage_manager::{StorageManager, StorageConfig, CapacityInfo};
pub use encrypted_chunk_store::{
    EncryptedChunkStore, EncryptedChunkMetadata, EncryptedFileMetadata, 
    EncryptionMetadata, FileCapability, EncryptedStorageStats
};
pub use enhanced_storage_manager::{
    EnhancedStorageManager, CombinedCapacityInfo, FileStorageResult
};