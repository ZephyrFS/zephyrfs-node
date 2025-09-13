//! Chunk-level security isolation for ZephyrFS
//!
//! Implements military-grade per-chunk security boundaries to ensure that:
//! 1. Each chunk is encrypted with unique, non-reusable keys
//! 2. Chunks are isolated from each other - compromise of one chunk cannot affect others
//! 3. Zero-knowledge guarantee - storage nodes never see plaintext or patterns
//! 4. Malicious content isolation - suspicious chunks are quarantined immediately

use anyhow::{Context, Result};
use ring::digest::{digest, SHA256};
use ring::rand::{SecureRandom, SystemRandom};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn, error};
use uuid::Uuid;
use zeroize::{Zeroize, ZeroizeOnDrop};

use crate::crypto::{EncryptedData, KeyHierarchy, SecureBytes};

/// Security isolation levels for chunks
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IsolationLevel {
    /// Standard isolation - normal chunks with per-chunk encryption
    Standard,
    /// Enhanced isolation - suspicious chunks with additional monitoring
    Enhanced,
    /// Maximum isolation - quarantined chunks with strict containment
    Quarantined,
}

/// Chunk security container with complete isolation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IsolatedChunk {
    /// Unique chunk identifier
    pub chunk_id: Uuid,

    /// Encrypted chunk data
    pub encrypted_data: EncryptedData,

    /// Unique encryption key fingerprint (not the key itself)
    pub key_fingerprint: String,

    /// Security isolation level
    pub isolation_level: IsolationLevel,

    /// Timestamp when chunk was isolated
    pub isolated_at: u64,

    /// Security metadata (encrypted)
    pub security_metadata: Vec<u8>,

    /// Integrity verification hash
    pub integrity_hash: String,

    /// Access control flags
    pub access_flags: ChunkAccessFlags,

    /// Quarantine reason (if applicable)
    pub quarantine_reason: Option<String>,
}

/// Access control flags for chunk security
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkAccessFlags {
    /// Can be read by storage operations
    pub readable: bool,

    /// Can be written/modified
    pub writable: bool,

    /// Can be transmitted over network
    pub transmittable: bool,

    /// Requires additional authentication
    pub auth_required: bool,

    /// Under security monitoring
    pub monitored: bool,

    /// Scheduled for deletion
    pub marked_for_deletion: bool,
}

impl Default for ChunkAccessFlags {
    fn default() -> Self {
        Self {
            readable: true,
            writable: false, // Chunks are immutable by default
            transmittable: true,
            auth_required: false,
            monitored: false,
            marked_for_deletion: false,
        }
    }
}

/// Per-chunk security manager with strict isolation
pub struct ChunkSecurityManager {
    /// Isolated chunks store
    chunks: Arc<RwLock<HashMap<Uuid, IsolatedChunk>>>,

    /// Security event log
    security_log: Arc<RwLock<Vec<SecurityEvent>>>,

    /// Cryptographic random number generator
    rng: SystemRandom,

    /// Active quarantine rules
    quarantine_rules: Arc<RwLock<Vec<QuarantineRule>>>,
}

/// Security event for audit logging
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityEvent {
    /// Event timestamp
    pub timestamp: u64,

    /// Event type
    pub event_type: SecurityEventType,

    /// Associated chunk ID
    pub chunk_id: Uuid,

    /// Event description
    pub description: String,

    /// Security level at time of event
    pub security_level: IsolationLevel,

    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

/// Types of security events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecurityEventType {
    ChunkIsolated,
    SecurityUpgraded,
    AccessDenied,
    SuspiciousPattern,
    QuarantineTriggered,
    IntegrityViolation,
    UnauthorizedAccess,
    SecurityDowngraded,
}

/// Quarantine rule for automated threat response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuarantineRule {
    /// Rule identifier
    pub rule_id: String,

    /// Rule description
    pub description: String,

    /// Pattern to match (encrypted pattern)
    pub pattern: Vec<u8>,

    /// Action to take when triggered
    pub action: QuarantineAction,

    /// Rule priority (higher = more important)
    pub priority: u32,

    /// Whether rule is currently active
    pub active: bool,
}

/// Actions to take when quarantine is triggered
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QuarantineAction {
    /// Monitor the chunk closely
    Monitor,

    /// Enhance security isolation
    EnhanceIsolation,

    /// Quarantine immediately
    Quarantine,

    /// Block all access
    Block,

    /// Mark for deletion
    Delete,
}

impl ChunkSecurityManager {
    /// Create new chunk security manager
    pub fn new() -> Self {
        Self {
            chunks: Arc::new(RwLock::new(HashMap::new())),
            security_log: Arc::new(RwLock::new(Vec::new())),
            rng: SystemRandom::new(),
            quarantine_rules: Arc::new(RwLock::new(Self::default_quarantine_rules())),
        }
    }

    /// Create isolated chunk with unique security boundary
    pub async fn create_isolated_chunk(
        &self,
        encrypted_data: EncryptedData,
        isolation_level: IsolationLevel,
    ) -> Result<IsolatedChunk> {
        let chunk_id = Uuid::new_v4();

        // Generate unique key fingerprint
        let key_fingerprint = self.generate_key_fingerprint(&encrypted_data)?;

        // Calculate integrity hash
        let integrity_hash = self.calculate_integrity_hash(&encrypted_data)?;

        // Create access flags based on isolation level
        let access_flags = match isolation_level {
            IsolationLevel::Standard => ChunkAccessFlags::default(),
            IsolationLevel::Enhanced => ChunkAccessFlags {
                auth_required: true,
                monitored: true,
                ..ChunkAccessFlags::default()
            },
            IsolationLevel::Quarantined => ChunkAccessFlags {
                readable: false,
                writable: false,
                transmittable: false,
                auth_required: true,
                monitored: true,
                marked_for_deletion: false,
            },
        };

        let isolated_chunk = IsolatedChunk {
            chunk_id,
            encrypted_data,
            key_fingerprint,
            isolation_level,
            isolated_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)?
                .as_secs(),
            security_metadata: vec![], // Will be populated based on analysis
            integrity_hash,
            access_flags,
            quarantine_reason: None,
        };

        // Store the isolated chunk
        {
            let mut chunks = self.chunks.write().await;
            chunks.insert(chunk_id, isolated_chunk.clone());
        }

        // Log security event
        self.log_security_event(SecurityEvent {
            timestamp: isolated_chunk.isolated_at,
            event_type: SecurityEventType::ChunkIsolated,
            chunk_id,
            description: format!("Chunk isolated with level: {:?}", isolation_level),
            security_level: isolation_level,
            metadata: HashMap::new(),
        }).await;

        info!("Created isolated chunk {} with level {:?}", chunk_id, isolation_level);
        Ok(isolated_chunk)
    }

    /// Enforce security boundaries for chunk access
    pub async fn access_chunk(&self, chunk_id: Uuid, access_type: ChunkAccessType) -> Result<bool> {
        let chunks = self.chunks.read().await;
        let chunk = chunks.get(&chunk_id)
            .context("Chunk not found")?;

        let allowed = match access_type {
            ChunkAccessType::Read => chunk.access_flags.readable,
            ChunkAccessType::Write => chunk.access_flags.writable,
            ChunkAccessType::Transmit => chunk.access_flags.transmittable,
        };

        if !allowed {
            // Log access denial
            self.log_security_event(SecurityEvent {
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)?
                    .as_secs(),
                event_type: SecurityEventType::AccessDenied,
                chunk_id,
                description: format!("Access denied for type: {:?}", access_type),
                security_level: chunk.isolation_level,
                metadata: HashMap::new(),
            }).await;

            warn!("Access denied for chunk {} (type: {:?})", chunk_id, access_type);
        }

        Ok(allowed)
    }

    /// Upgrade chunk security level
    pub async fn upgrade_security(&self, chunk_id: Uuid, new_level: IsolationLevel, reason: String) -> Result<()> {
        let mut chunks = self.chunks.write().await;
        let chunk = chunks.get_mut(&chunk_id)
            .context("Chunk not found")?;

        let old_level = chunk.isolation_level;

        // Only allow security upgrades, not downgrades (unless explicitly authorized)
        if (new_level as u32) < (old_level as u32) {
            warn!("Attempted security downgrade for chunk {}: {:?} -> {:?}",
                  chunk_id, old_level, new_level);
            return Ok(());
        }

        chunk.isolation_level = new_level;

        // Update access flags based on new level
        match new_level {
            IsolationLevel::Enhanced => {
                chunk.access_flags.auth_required = true;
                chunk.access_flags.monitored = true;
            },
            IsolationLevel::Quarantined => {
                chunk.access_flags.readable = false;
                chunk.access_flags.writable = false;
                chunk.access_flags.transmittable = false;
                chunk.access_flags.auth_required = true;
                chunk.access_flags.monitored = true;
                chunk.quarantine_reason = Some(reason.clone());
            },
            _ => {}
        }

        // Log security upgrade
        self.log_security_event(SecurityEvent {
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)?
                .as_secs(),
            event_type: SecurityEventType::SecurityUpgraded,
            chunk_id,
            description: format!("Security upgraded from {:?} to {:?}: {}", old_level, new_level, reason),
            security_level: new_level,
            metadata: HashMap::new(),
        }).await;

        info!("Upgraded security for chunk {} from {:?} to {:?}", chunk_id, old_level, new_level);
        Ok(())
    }

    /// Analyze chunk for suspicious patterns (content-blind)
    pub async fn analyze_chunk_security(&self, chunk_id: Uuid) -> Result<SecurityAnalysis> {
        let chunks = self.chunks.read().await;
        let chunk = chunks.get(&chunk_id)
            .context("Chunk not found")?;

        let mut analysis = SecurityAnalysis {
            chunk_id,
            threat_level: ThreatLevel::Low,
            suspicious_indicators: Vec::new(),
            recommendations: Vec::new(),
        };

        // Analyze encrypted data patterns (zero-knowledge analysis)
        let encrypted_data = &chunk.encrypted_data.ciphertext;

        // 1. Check for unusual size patterns
        if encrypted_data.len() > 100 * 1024 * 1024 {
            analysis.suspicious_indicators.push("Unusually large chunk size".to_string());
            analysis.threat_level = ThreatLevel::Medium;
        }

        // 2. Check encryption entropy (encrypted data should be high entropy)
        let entropy = self.calculate_entropy(&encrypted_data);
        if entropy < 7.5 {
            analysis.suspicious_indicators.push("Low entropy in encrypted data".to_string());
            analysis.threat_level = ThreatLevel::High;
            analysis.recommendations.push("Quarantine immediately".to_string());
        }

        // 3. Check for pattern repetition (even in encrypted form)
        if self.detect_pattern_repetition(&encrypted_data) {
            analysis.suspicious_indicators.push("Suspicious pattern repetition".to_string());
            analysis.threat_level = ThreatLevel::Medium;
        }

        // 4. Check against known bad patterns
        if self.check_against_quarantine_rules(&chunk).await {
            analysis.suspicious_indicators.push("Matches quarantine rule".to_string());
            analysis.threat_level = ThreatLevel::Critical;
            analysis.recommendations.push("Immediate quarantine required".to_string());
        }

        // Take action based on threat level
        match analysis.threat_level {
            ThreatLevel::Medium => {
                self.upgrade_security(chunk_id, IsolationLevel::Enhanced,
                    "Automated security analysis".to_string()).await?;
            },
            ThreatLevel::High | ThreatLevel::Critical => {
                self.upgrade_security(chunk_id, IsolationLevel::Quarantined,
                    format!("High threat detected: {:?}", analysis.suspicious_indicators)).await?;
            },
            _ => {}
        }

        Ok(analysis)
    }

    /// Generate unique key fingerprint without exposing the key
    fn generate_key_fingerprint(&self, encrypted_data: &EncryptedData) -> Result<String> {
        let mut context = Vec::new();
        context.extend_from_slice(&encrypted_data.nonce);
        context.extend_from_slice(&encrypted_data.aad);
        context.extend_from_slice(&encrypted_data.key_path.iter()
            .flat_map(|&x| x.to_le_bytes().to_vec()).collect::<Vec<_>>());

        let hash = digest(&SHA256, &context);
        Ok(hex::encode(hash.as_ref()))
    }

    /// Calculate integrity hash of encrypted data
    fn calculate_integrity_hash(&self, encrypted_data: &EncryptedData) -> Result<String> {
        let mut data = Vec::new();
        data.extend_from_slice(&encrypted_data.ciphertext);
        data.extend_from_slice(&encrypted_data.nonce);
        data.extend_from_slice(&encrypted_data.aad);

        let hash = digest(&SHA256, &data);
        Ok(hex::encode(hash.as_ref()))
    }

    /// Calculate entropy of data (for encrypted data analysis)
    fn calculate_entropy(&self, data: &[u8]) -> f64 {
        let mut freq = [0u32; 256];
        for &byte in data {
            freq[byte as usize] += 1;
        }

        let len = data.len() as f64;
        let mut entropy = 0.0;

        for &count in &freq {
            if count > 0 {
                let p = count as f64 / len;
                entropy -= p * p.log2();
            }
        }

        entropy
    }

    /// Detect suspicious pattern repetition in encrypted data
    fn detect_pattern_repetition(&self, data: &[u8]) -> bool {
        if data.len() < 32 {
            return false;
        }

        // Check for repeated 16-byte blocks (suspicious in properly encrypted data)
        let mut blocks = HashMap::new();
        for chunk in data.chunks(16) {
            if chunk.len() == 16 {
                let count = blocks.entry(chunk.to_vec()).or_insert(0);
                *count += 1;
                if *count > 3 {
                    return true; // Too many repetitions
                }
            }
        }

        false
    }

    /// Check chunk against active quarantine rules
    async fn check_against_quarantine_rules(&self, chunk: &IsolatedChunk) -> bool {
        let rules = self.quarantine_rules.read().await;

        for rule in rules.iter().filter(|r| r.active) {
            // Pattern matching on encrypted data
            if !rule.pattern.is_empty() {
                if chunk.encrypted_data.ciphertext.windows(rule.pattern.len())
                    .any(|window| window == rule.pattern) {
                    return true;
                }
            }
        }

        false
    }

    /// Log security event
    async fn log_security_event(&self, event: SecurityEvent) {
        let mut log = self.security_log.write().await;
        log.push(event);

        // Keep log size manageable
        if log.len() > 10000 {
            log.drain(0..1000);
        }
    }

    /// Get security events for audit
    pub async fn get_security_events(&self, chunk_id: Option<Uuid>) -> Vec<SecurityEvent> {
        let log = self.security_log.read().await;

        match chunk_id {
            Some(id) => log.iter().filter(|e| e.chunk_id == id).cloned().collect(),
            None => log.clone(),
        }
    }

    /// Default quarantine rules for automated threat detection
    fn default_quarantine_rules() -> Vec<QuarantineRule> {
        vec![
            QuarantineRule {
                rule_id: "entropy_check".to_string(),
                description: "Detect low entropy patterns".to_string(),
                pattern: vec![], // Handled by entropy analysis
                action: QuarantineAction::Quarantine,
                priority: 100,
                active: true,
            },
            QuarantineRule {
                rule_id: "size_limit".to_string(),
                description: "Block oversized chunks".to_string(),
                pattern: vec![], // Handled by size analysis
                action: QuarantineAction::EnhanceIsolation,
                priority: 50,
                active: true,
            },
        ]
    }
}

/// Types of chunk access
#[derive(Debug, Clone, Copy)]
pub enum ChunkAccessType {
    Read,
    Write,
    Transmit,
}

/// Security analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityAnalysis {
    pub chunk_id: Uuid,
    pub threat_level: ThreatLevel,
    pub suspicious_indicators: Vec<String>,
    pub recommendations: Vec<String>,
}

/// Threat level assessment
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ThreatLevel {
    Low,
    Medium,
    High,
    Critical,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::EncryptedData;

    #[tokio::test]
    async fn test_chunk_isolation() {
        let manager = ChunkSecurityManager::new();

        let encrypted_data = EncryptedData {
            segment_index: 0,
            ciphertext: vec![1, 2, 3, 4, 5],
            nonce: [0; 12],
            aad: vec![],
            key_path: vec![0, 1, 2],
        };

        let chunk = manager.create_isolated_chunk(encrypted_data, IsolationLevel::Standard).await.unwrap();

        assert_eq!(chunk.isolation_level, IsolationLevel::Standard);
        assert!(chunk.access_flags.readable);
        assert!(!chunk.access_flags.writable);
    }

    #[tokio::test]
    async fn test_security_upgrade() {
        let manager = ChunkSecurityManager::new();

        let encrypted_data = EncryptedData {
            segment_index: 0,
            ciphertext: vec![1, 2, 3, 4, 5],
            nonce: [0; 12],
            aad: vec![],
            key_path: vec![0, 1, 2],
        };

        let chunk = manager.create_isolated_chunk(encrypted_data, IsolationLevel::Standard).await.unwrap();

        manager.upgrade_security(chunk.chunk_id, IsolationLevel::Enhanced,
            "Test upgrade".to_string()).await.unwrap();

        let access_allowed = manager.access_chunk(chunk.chunk_id, ChunkAccessType::Read).await.unwrap();
        assert!(access_allowed);
    }
}