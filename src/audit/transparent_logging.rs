//! Transparency and audit logging system for ZephyrFS
//!
//! Provides comprehensive audit trails and transparency features while maintaining
//! zero-knowledge architecture and protecting user privacy.

use anyhow::{Context, Result};
use ring::digest::{Context as DigestContext, SHA256};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

/// Audit logging configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditConfig {
    /// Enable operation logging
    pub enable_operation_logging: bool,
    /// Enable access logging
    pub enable_access_logging: bool,
    /// Enable security event logging
    pub enable_security_logging: bool,
    /// Enable performance metrics logging
    pub enable_performance_logging: bool,
    /// Maximum log retention period (days)
    pub log_retention_days: u32,
    /// Log rotation size limit (bytes)
    pub log_rotation_size: u64,
    /// Enable log integrity verification
    pub enable_log_integrity: bool,
}

impl Default for AuditConfig {
    fn default() -> Self {
        Self {
            enable_operation_logging: true,
            enable_access_logging: true,
            enable_security_logging: true,
            enable_performance_logging: true,
            log_retention_days: 90,
            log_rotation_size: 100 * 1024 * 1024, // 100MB
            enable_log_integrity: true,
        }
    }
}

/// Types of audit events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuditEventType {
    /// Storage operations
    Storage(StorageEvent),
    /// Access control events
    Access(AccessEvent),
    /// Security-related events
    Security(SecurityEvent),
    /// Performance metrics
    Performance(PerformanceEvent),
    /// System health events
    SystemHealth(SystemHealthEvent),
}

/// Storage operation events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StorageEvent {
    ChunkStored { chunk_id: Uuid, size: u64, node_id: String },
    ChunkRetrieved { chunk_id: Uuid, node_id: String },
    ChunkDeleted { chunk_id: Uuid, node_id: String },
    ChunkReplicated { chunk_id: Uuid, source_node: String, target_node: String },
    ChunkCorrupted { chunk_id: Uuid, node_id: String, error_type: String },
}

/// Access control events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AccessEvent {
    ChunkAccessed { chunk_id: Uuid, requester: String, access_type: String },
    AccessDenied { chunk_id: Uuid, requester: String, reason: String },
    AuthenticationAttempt { user_id: String, success: bool, method: String },
    PermissionGranted { user_id: String, permission: String, resource: String },
    PermissionRevoked { user_id: String, permission: String, resource: String },
}

/// Security-related events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecurityEvent {
    ThreatDetected { threat_type: String, severity: String, details: String },
    ChunkQuarantined { chunk_id: Uuid, reason: String, threat_level: u8 },
    IntegrityViolation { resource: String, violation_type: String },
    EncryptionKeyRotation { key_id: String, rotation_reason: String },
    SecurityPolicyViolation { policy: String, violation: String },
}

/// Performance metrics events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PerformanceEvent {
    OperationTiming { operation: String, duration_ms: u64, success: bool },
    ThroughputMeasurement { operation: String, bytes_per_second: u64 },
    ResourceUtilization { cpu_percent: f32, memory_bytes: u64, disk_bytes: u64 },
    NetworkLatency { target: String, latency_ms: u64 },
}

/// System health events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SystemHealthEvent {
    NodeJoined { node_id: String, node_type: String },
    NodeLeft { node_id: String, reason: String },
    NetworkPartition { affected_nodes: Vec<String> },
    StorageThresholdReached { threshold_type: String, current_value: f64, limit: f64 },
    SystemMaintenanceScheduled { maintenance_type: String, scheduled_time: u64 },
}

/// Complete audit log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogEntry {
    /// Unique entry identifier
    pub entry_id: Uuid,
    /// Timestamp when event occurred
    pub timestamp: u64,
    /// Type of audit event
    pub event_type: AuditEventType,
    /// Node that generated this event
    pub source_node: String,
    /// Event severity level
    pub severity: AuditSeverity,
    /// Additional contextual metadata
    pub metadata: HashMap<String, String>,
    /// Cryptographic hash for integrity
    pub integrity_hash: Option<String>,
}

/// Severity levels for audit events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuditSeverity {
    Info,
    Warning,
    Error,
    Critical,
}

/// Audit log query parameters
#[derive(Debug, Clone)]
pub struct AuditQuery {
    /// Filter by time range
    pub time_range: Option<(u64, u64)>,
    /// Filter by event types
    pub event_types: Option<Vec<String>>,
    /// Filter by severity
    pub severity: Option<AuditSeverity>,
    /// Filter by source node
    pub source_node: Option<String>,
    /// Maximum number of results
    pub limit: Option<usize>,
    /// Include integrity verification
    pub verify_integrity: bool,
}

/// Audit query results
#[derive(Debug, Clone)]
pub struct AuditQueryResult {
    /// Matching log entries
    pub entries: Vec<AuditLogEntry>,
    /// Total number of matching entries
    pub total_count: usize,
    /// Query execution time
    pub execution_time_ms: u64,
    /// Integrity verification results
    pub integrity_status: IntegrityStatus,
}

/// Integrity verification status for audit logs
#[derive(Debug, Clone)]
pub enum IntegrityStatus {
    Verified,
    Compromised { corrupted_entries: Vec<Uuid> },
    UnknownNotChecked,
}

/// Transparency reporting metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransparencyReport {
    /// Reporting period start
    pub period_start: u64,
    /// Reporting period end
    pub period_end: u64,
    /// Total storage operations
    pub storage_operations: StorageMetrics,
    /// Access control statistics
    pub access_statistics: AccessMetrics,
    /// Security events summary
    pub security_summary: SecurityMetrics,
    /// Performance overview
    pub performance_overview: PerformanceMetrics,
    /// Node participation metrics
    pub node_metrics: NodeMetrics,
}

/// Storage operation metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageMetrics {
    pub total_chunks_stored: u64,
    pub total_chunks_retrieved: u64,
    pub total_chunks_deleted: u64,
    pub total_bytes_stored: u64,
    pub replication_events: u64,
    pub corruption_events: u64,
}

/// Access control metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessMetrics {
    pub total_access_attempts: u64,
    pub successful_accesses: u64,
    pub denied_accesses: u64,
    pub authentication_attempts: u64,
    pub successful_authentications: u64,
}

/// Security event metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityMetrics {
    pub threats_detected: u64,
    pub chunks_quarantined: u64,
    pub integrity_violations: u64,
    pub policy_violations: u64,
    pub key_rotations: u64,
}

/// Performance metrics summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub average_operation_time_ms: f64,
    pub peak_throughput_bps: u64,
    pub average_cpu_usage: f32,
    pub peak_memory_usage: u64,
    pub network_latency_p95_ms: u64,
}

/// Node participation metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeMetrics {
    pub active_nodes: u64,
    pub nodes_joined: u64,
    pub nodes_left: u64,
    pub network_partitions: u64,
    pub storage_threshold_breaches: u64,
}

/// Transparent audit logging system
pub struct TransparentAuditor {
    config: AuditConfig,
    node_id: String,
    log_storage: Box<dyn AuditStorage>,
}

impl TransparentAuditor {
    /// Create new transparent auditor
    pub fn new(config: AuditConfig, node_id: String, log_storage: Box<dyn AuditStorage>) -> Self {
        Self {
            config,
            node_id,
            log_storage,
        }
    }

    /// Log a storage operation event
    pub async fn log_storage_event(&self, event: StorageEvent) -> Result<()> {
        if !self.config.enable_operation_logging {
            return Ok(());
        }

        let entry = self.create_audit_entry(
            AuditEventType::Storage(event),
            AuditSeverity::Info,
            HashMap::new(),
        )?;

        self.log_storage.store_entry(entry).await?;
        Ok(())
    }

    /// Log an access control event
    pub async fn log_access_event(&self, event: AccessEvent) -> Result<()> {
        if !self.config.enable_access_logging {
            return Ok(());
        }

        let severity = match &event {
            AccessEvent::AccessDenied { .. } => AuditSeverity::Warning,
            AccessEvent::AuthenticationAttempt { success: false, .. } => AuditSeverity::Warning,
            _ => AuditSeverity::Info,
        };

        let entry = self.create_audit_entry(
            AuditEventType::Access(event),
            severity,
            HashMap::new(),
        )?;

        self.log_storage.store_entry(entry).await?;
        Ok(())
    }

    /// Log a security event
    pub async fn log_security_event(&self, event: SecurityEvent) -> Result<()> {
        if !self.config.enable_security_logging {
            return Ok(());
        }

        let severity = match &event {
            SecurityEvent::ThreatDetected { severity, .. } => match severity.as_str() {
                "critical" => AuditSeverity::Critical,
                "high" => AuditSeverity::Error,
                "medium" => AuditSeverity::Warning,
                _ => AuditSeverity::Info,
            },
            SecurityEvent::IntegrityViolation { .. } => AuditSeverity::Error,
            SecurityEvent::SecurityPolicyViolation { .. } => AuditSeverity::Warning,
            _ => AuditSeverity::Info,
        };

        let entry = self.create_audit_entry(
            AuditEventType::Security(event),
            severity,
            HashMap::new(),
        )?;

        self.log_storage.store_entry(entry).await?;
        Ok(())
    }

    /// Log a performance event
    pub async fn log_performance_event(&self, event: PerformanceEvent) -> Result<()> {
        if !self.config.enable_performance_logging {
            return Ok(());
        }

        let entry = self.create_audit_entry(
            AuditEventType::Performance(event),
            AuditSeverity::Info,
            HashMap::new(),
        )?;

        self.log_storage.store_entry(entry).await?;
        Ok(())
    }

    /// Log a system health event
    pub async fn log_system_health_event(&self, event: SystemHealthEvent) -> Result<()> {
        let severity = match &event {
            SystemHealthEvent::NetworkPartition { .. } => AuditSeverity::Error,
            SystemHealthEvent::StorageThresholdReached { .. } => AuditSeverity::Warning,
            _ => AuditSeverity::Info,
        };

        let entry = self.create_audit_entry(
            AuditEventType::SystemHealth(event),
            severity,
            HashMap::new(),
        )?;

        self.log_storage.store_entry(entry).await?;
        Ok(())
    }

    /// Query audit logs with filtering
    pub async fn query_logs(&self, query: AuditQuery) -> Result<AuditQueryResult> {
        let start_time = SystemTime::now();

        let entries = self.log_storage.query_entries(query.clone()).await?;

        let execution_time = SystemTime::now()
            .duration_since(start_time)
            .unwrap_or_default()
            .as_millis() as u64;

        let integrity_status = if query.verify_integrity {
            self.verify_log_integrity(&entries).await?
        } else {
            IntegrityStatus::UnknownNotChecked
        };

        Ok(AuditQueryResult {
            total_count: entries.len(),
            entries,
            execution_time_ms: execution_time,
            integrity_status,
        })
    }

    /// Generate transparency report for a time period
    pub async fn generate_transparency_report(
        &self,
        period_start: u64,
        period_end: u64,
    ) -> Result<TransparencyReport> {
        let query = AuditQuery {
            time_range: Some((period_start, period_end)),
            event_types: None,
            severity: None,
            source_node: None,
            limit: None,
            verify_integrity: false,
        };

        let query_result = self.query_logs(query).await?;
        let entries = query_result.entries;

        // Aggregate metrics from log entries
        let storage_operations = self.aggregate_storage_metrics(&entries);
        let access_statistics = self.aggregate_access_metrics(&entries);
        let security_summary = self.aggregate_security_metrics(&entries);
        let performance_overview = self.aggregate_performance_metrics(&entries);
        let node_metrics = self.aggregate_node_metrics(&entries);

        Ok(TransparencyReport {
            period_start,
            period_end,
            storage_operations,
            access_statistics,
            security_summary,
            performance_overview,
            node_metrics,
        })
    }

    /// Create audit log entry with proper formatting
    fn create_audit_entry(
        &self,
        event_type: AuditEventType,
        severity: AuditSeverity,
        metadata: HashMap<String, String>,
    ) -> Result<AuditLogEntry> {
        let entry_id = Uuid::new_v4();
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .context("Failed to get timestamp")?
            .as_secs();

        let integrity_hash = if self.config.enable_log_integrity {
            Some(self.compute_entry_hash(&entry_id, timestamp, &event_type)?)
        } else {
            None
        };

        Ok(AuditLogEntry {
            entry_id,
            timestamp,
            event_type,
            source_node: self.node_id.clone(),
            severity,
            metadata,
            integrity_hash,
        })
    }

    /// Compute cryptographic hash for log entry integrity
    fn compute_entry_hash(
        &self,
        entry_id: &Uuid,
        timestamp: u64,
        event_type: &AuditEventType,
    ) -> Result<String> {
        let mut context = DigestContext::new(&SHA256);

        // Hash entry components
        context.update(entry_id.as_bytes());
        context.update(&timestamp.to_le_bytes());
        context.update(self.node_id.as_bytes());

        // Hash event type (serialized)
        let event_bytes = serde_json::to_vec(event_type)
            .context("Failed to serialize event type")?;
        context.update(&event_bytes);

        let hash = context.finish();
        Ok(hex::encode(hash.as_ref()))
    }

    /// Verify integrity of log entries
    async fn verify_log_integrity(&self, entries: &[AuditLogEntry]) -> Result<IntegrityStatus> {
        if !self.config.enable_log_integrity {
            return Ok(IntegrityStatus::UnknownNotChecked);
        }

        let mut corrupted_entries = Vec::new();

        for entry in entries {
            if let Some(stored_hash) = &entry.integrity_hash {
                let computed_hash = self.compute_entry_hash(
                    &entry.entry_id,
                    entry.timestamp,
                    &entry.event_type,
                )?;

                if stored_hash != &computed_hash {
                    corrupted_entries.push(entry.entry_id);
                }
            }
        }

        if corrupted_entries.is_empty() {
            Ok(IntegrityStatus::Verified)
        } else {
            Ok(IntegrityStatus::Compromised { corrupted_entries })
        }
    }

    /// Aggregate storage metrics from log entries
    fn aggregate_storage_metrics(&self, entries: &[AuditLogEntry]) -> StorageMetrics {
        let mut metrics = StorageMetrics {
            total_chunks_stored: 0,
            total_chunks_retrieved: 0,
            total_chunks_deleted: 0,
            total_bytes_stored: 0,
            replication_events: 0,
            corruption_events: 0,
        };

        for entry in entries {
            if let AuditEventType::Storage(storage_event) = &entry.event_type {
                match storage_event {
                    StorageEvent::ChunkStored { size, .. } => {
                        metrics.total_chunks_stored += 1;
                        metrics.total_bytes_stored += size;
                    }
                    StorageEvent::ChunkRetrieved { .. } => {
                        metrics.total_chunks_retrieved += 1;
                    }
                    StorageEvent::ChunkDeleted { .. } => {
                        metrics.total_chunks_deleted += 1;
                    }
                    StorageEvent::ChunkReplicated { .. } => {
                        metrics.replication_events += 1;
                    }
                    StorageEvent::ChunkCorrupted { .. } => {
                        metrics.corruption_events += 1;
                    }
                }
            }
        }

        metrics
    }

    /// Aggregate access metrics from log entries
    fn aggregate_access_metrics(&self, entries: &[AuditLogEntry]) -> AccessMetrics {
        let mut metrics = AccessMetrics {
            total_access_attempts: 0,
            successful_accesses: 0,
            denied_accesses: 0,
            authentication_attempts: 0,
            successful_authentications: 0,
        };

        for entry in entries {
            if let AuditEventType::Access(access_event) = &entry.event_type {
                match access_event {
                    AccessEvent::ChunkAccessed { .. } => {
                        metrics.total_access_attempts += 1;
                        metrics.successful_accesses += 1;
                    }
                    AccessEvent::AccessDenied { .. } => {
                        metrics.total_access_attempts += 1;
                        metrics.denied_accesses += 1;
                    }
                    AccessEvent::AuthenticationAttempt { success, .. } => {
                        metrics.authentication_attempts += 1;
                        if *success {
                            metrics.successful_authentications += 1;
                        }
                    }
                    _ => {}
                }
            }
        }

        metrics
    }

    /// Aggregate security metrics from log entries
    fn aggregate_security_metrics(&self, entries: &[AuditLogEntry]) -> SecurityMetrics {
        let mut metrics = SecurityMetrics {
            threats_detected: 0,
            chunks_quarantined: 0,
            integrity_violations: 0,
            policy_violations: 0,
            key_rotations: 0,
        };

        for entry in entries {
            if let AuditEventType::Security(security_event) = &entry.event_type {
                match security_event {
                    SecurityEvent::ThreatDetected { .. } => {
                        metrics.threats_detected += 1;
                    }
                    SecurityEvent::ChunkQuarantined { .. } => {
                        metrics.chunks_quarantined += 1;
                    }
                    SecurityEvent::IntegrityViolation { .. } => {
                        metrics.integrity_violations += 1;
                    }
                    SecurityEvent::SecurityPolicyViolation { .. } => {
                        metrics.policy_violations += 1;
                    }
                    SecurityEvent::EncryptionKeyRotation { .. } => {
                        metrics.key_rotations += 1;
                    }
                }
            }
        }

        metrics
    }

    /// Aggregate performance metrics from log entries
    fn aggregate_performance_metrics(&self, entries: &[AuditLogEntry]) -> PerformanceMetrics {
        let mut operation_times = Vec::new();
        let mut throughputs = Vec::new();
        let mut cpu_usages = Vec::new();
        let mut memory_usages = Vec::new();
        let mut latencies = Vec::new();

        for entry in entries {
            if let AuditEventType::Performance(perf_event) = &entry.event_type {
                match perf_event {
                    PerformanceEvent::OperationTiming { duration_ms, .. } => {
                        operation_times.push(*duration_ms as f64);
                    }
                    PerformanceEvent::ThroughputMeasurement { bytes_per_second, .. } => {
                        throughputs.push(*bytes_per_second);
                    }
                    PerformanceEvent::ResourceUtilization { cpu_percent, memory_bytes, .. } => {
                        cpu_usages.push(*cpu_percent);
                        memory_usages.push(*memory_bytes);
                    }
                    PerformanceEvent::NetworkLatency { latency_ms, .. } => {
                        latencies.push(*latency_ms);
                    }
                }
            }
        }

        let average_operation_time_ms = if operation_times.is_empty() {
            0.0
        } else {
            operation_times.iter().sum::<f64>() / operation_times.len() as f64
        };

        let peak_throughput_bps = throughputs.iter().max().copied().unwrap_or(0);

        let average_cpu_usage = if cpu_usages.is_empty() {
            0.0
        } else {
            cpu_usages.iter().sum::<f32>() / cpu_usages.len() as f32
        };

        let peak_memory_usage = memory_usages.iter().max().copied().unwrap_or(0);

        // Calculate 95th percentile latency
        let mut sorted_latencies = latencies;
        sorted_latencies.sort_unstable();
        let p95_index = (sorted_latencies.len() as f64 * 0.95) as usize;
        let network_latency_p95_ms = sorted_latencies.get(p95_index).copied().unwrap_or(0);

        PerformanceMetrics {
            average_operation_time_ms,
            peak_throughput_bps,
            average_cpu_usage,
            peak_memory_usage,
            network_latency_p95_ms,
        }
    }

    /// Aggregate node metrics from log entries
    fn aggregate_node_metrics(&self, entries: &[AuditLogEntry]) -> NodeMetrics {
        let mut metrics = NodeMetrics {
            active_nodes: 0, // This would need to be calculated differently
            nodes_joined: 0,
            nodes_left: 0,
            network_partitions: 0,
            storage_threshold_breaches: 0,
        };

        for entry in entries {
            if let AuditEventType::SystemHealth(health_event) = &entry.event_type {
                match health_event {
                    SystemHealthEvent::NodeJoined { .. } => {
                        metrics.nodes_joined += 1;
                    }
                    SystemHealthEvent::NodeLeft { .. } => {
                        metrics.nodes_left += 1;
                    }
                    SystemHealthEvent::NetworkPartition { .. } => {
                        metrics.network_partitions += 1;
                    }
                    SystemHealthEvent::StorageThresholdReached { .. } => {
                        metrics.storage_threshold_breaches += 1;
                    }
                    _ => {}
                }
            }
        }

        metrics
    }
}

/// Trait for audit log storage backends
#[async_trait::async_trait]
pub trait AuditStorage: Send + Sync {
    /// Store a new audit log entry
    async fn store_entry(&self, entry: AuditLogEntry) -> Result<()>;

    /// Query audit log entries with filters
    async fn query_entries(&self, query: AuditQuery) -> Result<Vec<AuditLogEntry>>;

    /// Rotate old log files
    async fn rotate_logs(&self, retention_days: u32) -> Result<()>;

    /// Get storage statistics
    async fn get_storage_stats(&self) -> Result<(u64, u64)>; // (total_entries, total_size_bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    /// In-memory audit storage for testing
    struct MemoryAuditStorage {
        entries: Arc<Mutex<Vec<AuditLogEntry>>>,
    }

    impl MemoryAuditStorage {
        fn new() -> Self {
            Self {
                entries: Arc::new(Mutex::new(Vec::new())),
            }
        }
    }

    #[async_trait::async_trait]
    impl AuditStorage for MemoryAuditStorage {
        async fn store_entry(&self, entry: AuditLogEntry) -> Result<()> {
            let mut entries = self.entries.lock().unwrap();
            entries.push(entry);
            Ok(())
        }

        async fn query_entries(&self, query: AuditQuery) -> Result<Vec<AuditLogEntry>> {
            let entries = self.entries.lock().unwrap();
            let mut filtered: Vec<_> = entries.clone();

            // Apply time range filter
            if let Some((start, end)) = query.time_range {
                filtered.retain(|entry| entry.timestamp >= start && entry.timestamp <= end);
            }

            // Apply limit
            if let Some(limit) = query.limit {
                filtered.truncate(limit);
            }

            Ok(filtered)
        }

        async fn rotate_logs(&self, _retention_days: u32) -> Result<()> {
            // No-op for in-memory storage
            Ok(())
        }

        async fn get_storage_stats(&self) -> Result<(u64, u64)> {
            let entries = self.entries.lock().unwrap();
            Ok((entries.len() as u64, 0)) // Size calculation would be more complex
        }
    }

    #[tokio::test]
    async fn test_audit_logging() -> Result<()> {
        let config = AuditConfig::default();
        let storage = Box::new(MemoryAuditStorage::new());
        let auditor = TransparentAuditor::new(config, "test-node".to_string(), storage);

        // Log some events
        auditor.log_storage_event(StorageEvent::ChunkStored {
            chunk_id: Uuid::new_v4(),
            size: 1024,
            node_id: "node-1".to_string(),
        }).await?;

        auditor.log_security_event(SecurityEvent::ThreatDetected {
            threat_type: "malware".to_string(),
            severity: "medium".to_string(),
            details: "Test threat".to_string(),
        }).await?;

        // Query logs
        let query = AuditQuery {
            time_range: None,
            event_types: None,
            severity: None,
            source_node: None,
            limit: None,
            verify_integrity: true,
        };

        let result = auditor.query_logs(query).await?;
        assert_eq!(result.entries.len(), 2);
        assert!(matches!(result.integrity_status, IntegrityStatus::Verified));

        Ok(())
    }

    #[tokio::test]
    async fn test_transparency_report() -> Result<()> {
        let config = AuditConfig::default();
        let storage = Box::new(MemoryAuditStorage::new());
        let auditor = TransparentAuditor::new(config, "test-node".to_string(), storage);

        // Log various events
        for i in 0..5 {
            auditor.log_storage_event(StorageEvent::ChunkStored {
                chunk_id: Uuid::new_v4(),
                size: (i + 1) * 1000,
                node_id: format!("node-{}", i),
            }).await?;
        }

        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let report = auditor.generate_transparency_report(
            current_time - 3600, // 1 hour ago
            current_time,
        ).await?;

        assert_eq!(report.storage_operations.total_chunks_stored, 5);
        assert_eq!(report.storage_operations.total_bytes_stored, 15000);

        Ok(())
    }
}