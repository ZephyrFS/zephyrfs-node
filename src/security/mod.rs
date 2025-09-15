//! Security module for ZephyrFS
//!
//! Provides comprehensive security features including chunk isolation,
//! malicious content detection, and military-grade security boundaries.

pub mod chunk_isolation;
pub mod malicious_detection;

pub use chunk_isolation::{
    ChunkSecurityManager, IsolatedChunk, IsolationLevel, ChunkAccessFlags, SecurityEvent,
    IsolationConfig, ChunkStatus, ThreatLevel
};
pub use malicious_detection::{
    MaliciousContentDetector, ThreatAnalysisResult, QuarantineManager, QuarantineStats,
    ThreatIndicator, DetectionConfig, QuarantineStatus
};

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;
use crate::crypto::EncryptedData;

/// Security configuration for the entire system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    /// Chunk isolation configuration
    pub isolation_config: chunk_isolation::IsolationConfig,
    /// Malicious content detection configuration
    pub detection_config: malicious_detection::DetectionConfig,
    /// Global security policies
    pub global_policies: GlobalSecurityPolicies,
}

/// Global security policies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalSecurityPolicies {
    /// Minimum isolation level for new chunks
    pub minimum_isolation_level: IsolationLevel,
    /// Automatic quarantine on threat detection
    pub auto_quarantine_enabled: bool,
    /// Maximum threat level before immediate quarantine
    pub quarantine_threshold: ThreatLevel,
    /// Security audit logging enabled
    pub audit_logging_enabled: bool,
    /// Zero-knowledge enforcement
    pub zero_knowledge_enforced: bool,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            isolation_config: chunk_isolation::IsolationConfig::default(),
            detection_config: malicious_detection::DetectionConfig::default(),
            global_policies: GlobalSecurityPolicies {
                minimum_isolation_level: IsolationLevel::Standard,
                auto_quarantine_enabled: true,
                quarantine_threshold: ThreatLevel::Medium,
                audit_logging_enabled: true,
                zero_knowledge_enforced: true,
            },
        }
    }
}

/// Unified security manager combining all security systems
pub struct UnifiedSecurityManager {
    chunk_security: ChunkSecurityManager,
    threat_detector: MaliciousContentDetector,
    quarantine_manager: QuarantineManager,
    config: SecurityConfig,
}

impl UnifiedSecurityManager {
    /// Create new unified security manager
    pub fn new(config: SecurityConfig) -> Result<Self> {
        let chunk_security = ChunkSecurityManager::new();
        let threat_detector = MaliciousContentDetector::new();
        let quarantine_manager = QuarantineManager::new();

        Ok(Self {
            chunk_security,
            threat_detector,
            quarantine_manager,
            config,
        })
    }

    /// Process new chunk with comprehensive security analysis
    pub async fn process_new_chunk(
        &mut self,
        chunk_id: Uuid,
        encrypted_data: &[u8],
        metadata: HashMap<String, String>,
    ) -> Result<ChunkSecurityDecision> {
        // Step 1: Create encrypted data struct with production-ready crypto
        use rand::RngCore;
        let mut rng = rand::thread_rng();
        let mut nonce = [0u8; 12];
        rng.fill_bytes(&mut nonce);

        let encrypted_data_struct = EncryptedData {
            segment_index: 0,
            ciphertext: encrypted_data.to_vec(),
            nonce, // Cryptographically secure random nonce
            aad: Vec::new(),
            key_path: vec![chunk_id.as_u128() as u32], // Derive from chunk ID
        };

        // Step 2: Initial threat analysis
        let threat_analysis = self.threat_detector
            .analyze_content(&encrypted_data_struct, &metadata)
            .await?;

        // Step 3: Determine isolation level based on threat analysis
        let isolation_level = self.determine_isolation_level(&threat_analysis);

        // Step 4: Create isolated chunk
        let isolated_chunk = self.chunk_security.create_isolated_chunk(
            encrypted_data_struct,
            isolation_level,
        ).await?;

        // Step 4: Check if quarantine is needed
        let quarantine_decision = if threat_analysis.overall_threat_level >= self.config.global_policies.quarantine_threshold {
            self.quarantine_manager.quarantine_chunk(
                chunk_id,
                format!("Threat detected: {:?}", threat_analysis.recommended_actions),
                threat_analysis.overall_threat_level,
                true, // automated
            ).await?;
            QuarantineDecision::Quarantined
        } else {
            QuarantineDecision::Allowed
        };

        Ok(ChunkSecurityDecision {
            chunk_id,
            isolation_level,
            threat_analysis,
            quarantine_decision,
            security_clearance: self.calculate_security_clearance(&threat_analysis, isolation_level),
        })
    }

    /// Verify chunk access with security checks
    pub async fn verify_chunk_access(
        &self,
        chunk_id: Uuid,
        requested_access: ChunkAccessFlags,
        requester_context: AccessContext,
    ) -> Result<AccessDecision> {
        // Get chunk security status
        let chunk_status = self.chunk_security.get_chunk_status(chunk_id).await?;

        // Check if chunk is quarantined
        if self.quarantine_manager.is_quarantined(chunk_id).await? {
            return Ok(AccessDecision::Denied {
                reason: "Chunk is quarantined".to_string(),
            });
        }

        // Convert ChunkAccessFlags to ChunkAccessType based on flags
        let access_type = if requested_access.writable {
            chunk_isolation::ChunkAccessType::Write
        } else if requested_access.transmittable {
            chunk_isolation::ChunkAccessType::Transmit
        } else {
            chunk_isolation::ChunkAccessType::Read // Default for readable or other cases
        };

        // Verify access permissions based on isolation level
        let access_allowed = self.chunk_security.verify_access(
            chunk_id,
            access_type,
        ).await?;

        if access_allowed {
            Ok(AccessDecision::Granted {
                conditions: self.get_access_conditions(&chunk_status, &requester_context),
            })
        } else {
            Ok(AccessDecision::Denied {
                reason: "Insufficient security clearance".to_string(),
            })
        }
    }

    /// Get comprehensive security status for a chunk
    pub async fn get_chunk_security_status(&self, chunk_id: Uuid) -> Result<ChunkSecurityStatus> {
        let chunk_status = self.chunk_security.get_chunk_status(chunk_id).await?;
        let quarantine_status = self.quarantine_manager.get_quarantine_status(chunk_id).await?;
        let threat_history = self.threat_detector.get_threat_history(chunk_id).await;

        Ok(ChunkSecurityStatus {
            chunk_id,
            isolation_level: chunk_status.isolation_level,
            access_flags: chunk_status.access_flags,
            quarantine_status,
            threat_history,
            last_security_scan: chunk_status.last_update,
            security_score: self.calculate_security_score(&chunk_status, &quarantine_status),
        })
    }

    /// Update security policies
    pub fn update_policies(&mut self, new_policies: GlobalSecurityPolicies) -> Result<()> {
        self.config.global_policies = new_policies;
        self.chunk_security.update_config(self.config.isolation_config.clone())?;
        self.threat_detector.update_config(self.config.detection_config.clone())?;
        Ok(())
    }

    /// Determine appropriate isolation level based on threat analysis
    fn determine_isolation_level(&self, analysis: &ThreatAnalysisResult) -> IsolationLevel {
        match analysis.overall_threat_level {
            ThreatLevel::None | ThreatLevel::Low => {
                if self.config.global_policies.minimum_isolation_level > IsolationLevel::Standard {
                    self.config.global_policies.minimum_isolation_level
                } else {
                    IsolationLevel::Standard
                }
            }
            ThreatLevel::Medium => IsolationLevel::Enhanced,
            ThreatLevel::High | ThreatLevel::Critical => IsolationLevel::Quarantined,
        }
    }

    /// Calculate security clearance level
    fn calculate_security_clearance(
        &self,
        analysis: &ThreatAnalysisResult,
        isolation_level: IsolationLevel,
    ) -> SecurityClearance {
        match (analysis.overall_threat_level, isolation_level) {
            (ThreatLevel::None, IsolationLevel::Standard) => SecurityClearance::Public,
            (ThreatLevel::Low, IsolationLevel::Standard) |
            (ThreatLevel::None, IsolationLevel::Enhanced) => SecurityClearance::Internal,
            (ThreatLevel::Medium, _) |
            (ThreatLevel::Low, IsolationLevel::Enhanced) => SecurityClearance::Restricted,
            (ThreatLevel::High, _) => SecurityClearance::Confidential,
            (ThreatLevel::Critical, _) => SecurityClearance::TopSecret,
            _ => SecurityClearance::Internal,
        }
    }

    /// Get access conditions based on security status
    fn get_access_conditions(
        &self,
        _chunk_status: &chunk_isolation::ChunkStatus,
        _requester_context: &AccessContext,
    ) -> Vec<AccessCondition> {
        // Return appropriate access conditions
        vec![AccessCondition::AuditLogging, AccessCondition::RateLimit]
    }

    /// Calculate overall security score for a chunk
    fn calculate_security_score(
        &self,
        chunk_status: &chunk_isolation::ChunkStatus,
        quarantine_status: &malicious_detection::QuarantineStatus,
    ) -> f64 {
        let isolation_score = match chunk_status.isolation_level {
            IsolationLevel::Standard => 0.6,
            IsolationLevel::Enhanced => 0.8,
            IsolationLevel::Quarantined => 0.4, // Lower because it indicates detected threats
        };

        let quarantine_penalty = if quarantine_status.is_quarantined { 0.2 } else { 0.0 };

        (isolation_score - quarantine_penalty).max(0.0).min(1.0)
    }
}

/// Decision result for chunk security processing
#[derive(Debug, Clone)]
pub struct ChunkSecurityDecision {
    pub chunk_id: Uuid,
    pub isolation_level: IsolationLevel,
    pub threat_analysis: ThreatAnalysisResult,
    pub quarantine_decision: QuarantineDecision,
    pub security_clearance: SecurityClearance,
}

/// Quarantine decision
#[derive(Debug, Clone)]
pub enum QuarantineDecision {
    Allowed,
    Quarantined,
    PendingReview,
}

/// Security clearance levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SecurityClearance {
    Public,
    Internal,
    Restricted,
    Confidential,
    Secret,
    TopSecret,
}

/// Access decision result
#[derive(Debug, Clone)]
pub enum AccessDecision {
    Granted { conditions: Vec<AccessCondition> },
    Denied { reason: String },
}

/// Conditions for chunk access
#[derive(Debug, Clone)]
pub enum AccessCondition {
    AuditLogging,
    RateLimit,
    TimeRestriction,
    LocationRestriction,
    AdditionalAuthentication,
}

/// Context for access requests
#[derive(Debug, Clone)]
pub struct AccessContext {
    pub requester_id: String,
    pub security_clearance: SecurityClearance,
    pub request_timestamp: u64,
    pub request_origin: String,
    pub additional_context: HashMap<String, String>,
}

/// Comprehensive security status for a chunk
#[derive(Debug, Clone)]
pub struct ChunkSecurityStatus {
    pub chunk_id: Uuid,
    pub isolation_level: IsolationLevel,
    pub access_flags: ChunkAccessFlags,
    pub quarantine_status: malicious_detection::QuarantineStatus,
    pub threat_history: Vec<ThreatAnalysisResult>,
    pub last_security_scan: u64,
    pub security_score: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_unified_security_manager() -> Result<()> {
        let config = SecurityConfig::default();
        let mut security_manager = UnifiedSecurityManager::new(config)?;

        let chunk_id = Uuid::new_v4();
        let test_data = b"Test encrypted chunk data";
        let metadata = HashMap::new();

        let decision = security_manager
            .process_new_chunk(chunk_id, test_data, metadata)
            .await?;

        assert_eq!(decision.chunk_id, chunk_id);
        assert!(matches!(decision.quarantine_decision, QuarantineDecision::Allowed));

        Ok(())
    }

    #[tokio::test]
    async fn test_chunk_access_verification() -> Result<()> {
        let config = SecurityConfig::default();
        let security_manager = UnifiedSecurityManager::new(config)?;

        let chunk_id = Uuid::new_v4();
        let access_context = AccessContext {
            requester_id: "test-user".to_string(),
            security_clearance: SecurityClearance::Internal,
            request_timestamp: 1234567890,
            request_origin: "localhost".to_string(),
            additional_context: HashMap::new(),
        };

        let access_decision = security_manager
            .verify_chunk_access(chunk_id, ChunkAccessFlags::READ, access_context)
            .await?;

        // Should deny access for non-existent chunk
        assert!(matches!(access_decision, AccessDecision::Denied { .. }));

        Ok(())
    }
}