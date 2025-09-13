//! ZephyrFS Node Library
//!
//! Core library for ZephyrFS distributed P2P storage system.
//! Provides cryptographic primitives, storage management, and network protocols.

pub mod config;
pub mod network;
pub mod storage;
pub mod protocol;
pub mod node_manager;
pub mod crypto;

pub use crypto::{
    ZephyrCrypto, CryptoParams, ScryptParams, AesParams, HashParams,
    ContentHasher, VerificationHasher, EncryptedData, ContentId, HashAlgorithm
};

pub use config::Config;
pub use node_manager::{NodeManager, DistributionStrategy};
pub use storage::{StorageManager, StorageConfig};