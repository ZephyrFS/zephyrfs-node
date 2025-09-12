// Storage module - placeholder for Phase 1.2 implementation
// 
// Safety: Storage will implement encryption at rest by default
// Privacy: All data stored is encrypted with user-controlled keys
// Transparency: Storage operations are logged for audit trail

pub mod chunk_store;
pub mod metadata_store;

// Re-export main storage interface
pub use chunk_store::ChunkStore;
pub use metadata_store::MetadataStore;