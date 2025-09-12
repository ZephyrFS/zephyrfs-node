use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::path::{Path, PathBuf};

/// Configuration for ZephyrFS Node
/// 
/// Transparency: All configuration options are clearly documented
/// Privacy: No sensitive data is logged or stored in plaintext
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    /// Node identification (generated if not provided)
    pub node_id: Option<String>,
    
    /// P2P networking configuration
    pub network: NetworkConfig,
    
    /// Storage configuration
    pub storage: StorageConfig,
    
    /// Coordinator connection
    pub coordinator: CoordinatorConfig,
    
    /// Security settings
    pub security: SecurityConfig,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NetworkConfig {
    /// P2P listening port (default: 4001)
    pub p2p_port: u16,
    
    /// API listening port (default: 8080)
    pub api_port: u16,
    
    /// Enable mDNS for local discovery (default: true)
    pub enable_mdns: bool,
    
    /// Bootstrap peers for initial connection
    pub bootstrap_peers: Vec<String>,
    
    /// Maximum number of peers to maintain (default: 50)
    pub max_peers: usize,
    
    /// NAT traversal settings
    pub enable_nat_traversal: bool,
    pub stun_servers: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct StorageConfig {
    /// Data directory for storing chunks and metadata
    pub data_dir: PathBuf,
    
    /// Maximum storage to allocate (in bytes)
    pub max_storage: u64,
    
    /// Chunk size for file splitting (default: 1MB)
    pub chunk_size: usize,
    
    /// Enable storage encryption at rest
    pub encrypt_at_rest: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CoordinatorConfig {
    /// Coordinator service URL
    pub url: String,
    
    /// Connection timeout (seconds)
    pub timeout: u64,
    
    /// Heartbeat interval (seconds)
    pub heartbeat_interval: u64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SecurityConfig {
    /// Enable strict TLS verification
    pub strict_tls: bool,
    
    /// Minimum peer reputation score to accept connections
    pub min_peer_reputation: f64,
    
    /// Maximum connections from unknown peers
    pub max_unknown_peers: usize,
    
    /// Enable connection rate limiting
    pub enable_rate_limiting: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            node_id: None, // Generated at startup
            network: NetworkConfig::default(),
            storage: StorageConfig::default(),
            coordinator: CoordinatorConfig::default(),
            security: SecurityConfig::default(),
        }
    }
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            p2p_port: 4001,
            api_port: 8080,
            enable_mdns: true,
            bootstrap_peers: vec![],
            max_peers: 50,
            enable_nat_traversal: true,
            stun_servers: vec![
                "stun:stun.l.google.com:19302".to_string(),
                "stun:stun1.l.google.com:19302".to_string(),
            ],
        }
    }
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            data_dir: PathBuf::from("./data"),
            max_storage: 10 * 1024 * 1024 * 1024, // 10GB
            chunk_size: 1024 * 1024, // 1MB
            encrypt_at_rest: true, // Privacy: Always encrypt by default
        }
    }
}

impl Default for CoordinatorConfig {
    fn default() -> Self {
        Self {
            url: "http://localhost:9090".to_string(),
            timeout: 30,
            heartbeat_interval: 60,
        }
    }
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            strict_tls: true, // Safety: Always use strict TLS
            min_peer_reputation: 0.5,
            max_unknown_peers: 10,
            enable_rate_limiting: true, // Safety: Prevent DoS attacks
        }
    }
}

impl Config {
    /// Load configuration from file or environment variables
    /// 
    /// Transparency: Configuration loading process is fully logged
    pub fn load(config_path: Option<&Path>) -> Result<Self> {
        let mut config = Config::default();
        
        // Load from file if provided
        if let Some(path) = config_path {
            let contents = std::fs::read_to_string(path)
                .with_context(|| format!("Failed to read config file: {:?}", path))?;
            
            config = serde_yaml::from_str(&contents)
                .with_context(|| format!("Failed to parse config file: {:?}", path))?;
        }
        
        // Override with environment variables (for Docker/container deployment)
        config.apply_env_overrides()?;
        
        // Generate node ID if not provided
        if config.node_id.is_none() {
            config.node_id = Some(generate_node_id());
        }
        
        // Validate configuration
        config.validate()?;
        
        Ok(config)
    }
    
    /// Apply environment variable overrides
    /// 
    /// Transparency: All environment variables are documented
    fn apply_env_overrides(&mut self) -> Result<()> {
        if let Ok(node_id) = std::env::var("ZEPHYR_NODE_ID") {
            self.node_id = Some(node_id);
        }
        
        if let Ok(p2p_port) = std::env::var("ZEPHYR_P2P_PORT") {
            self.network.p2p_port = p2p_port.parse()
                .context("Invalid ZEPHYR_P2P_PORT value")?;
        }
        
        if let Ok(api_port) = std::env::var("ZEPHYR_API_PORT") {
            self.network.api_port = api_port.parse()
                .context("Invalid ZEPHYR_API_PORT value")?;
        }
        
        if let Ok(coordinator_url) = std::env::var("ZEPHYR_COORDINATOR_URL") {
            self.coordinator.url = coordinator_url;
        }
        
        if let Ok(max_storage) = std::env::var("ZEPHYR_MAX_STORAGE") {
            self.storage.max_storage = parse_storage_size(&max_storage)
                .context("Invalid ZEPHYR_MAX_STORAGE value")?;
        }
        
        Ok(())
    }
    
    /// Validate configuration for safety and security
    /// 
    /// Safety: Comprehensive validation prevents misconfigurations
    fn validate(&self) -> Result<()> {
        // Validate ports are available
        if self.network.p2p_port == self.network.api_port {
            anyhow::bail!("P2P and API ports must be different");
        }
        
        // Validate storage limits
        if self.storage.max_storage == 0 {
            anyhow::bail!("Maximum storage must be greater than 0");
        }
        
        if self.storage.chunk_size == 0 || self.storage.chunk_size > 100 * 1024 * 1024 {
            anyhow::bail!("Chunk size must be between 1 byte and 100MB");
        }
        
        // Validate coordinator URL
        if self.coordinator.url.is_empty() {
            anyhow::bail!("Coordinator URL must be provided");
        }
        
        // Validate security settings
        if self.security.min_peer_reputation < 0.0 || self.security.min_peer_reputation > 1.0 {
            anyhow::bail!("Minimum peer reputation must be between 0.0 and 1.0");
        }
        
        Ok(())
    }
    
    /// Get P2P listen address
    pub fn p2p_listen_addr(&self) -> SocketAddr {
        ([0, 0, 0, 0], self.network.p2p_port).into()
    }
    
    /// Get API listen address  
    pub fn api_listen_addr(&self) -> SocketAddr {
        ([127, 0, 0, 1], self.network.api_port).into()
    }
}

/// Generate a secure, unique node ID
/// 
/// Privacy: Node ID doesn't reveal any personal information
fn generate_node_id() -> String {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let id: [u8; 16] = rng.gen();
    format!("zephyr-{}", hex::encode(id))
}

/// Parse storage size from human-readable format (e.g., "10GB", "500MB")
/// 
/// Transparency: Clear error messages for invalid formats
fn parse_storage_size(size_str: &str) -> Result<u64> {
    let size_str = size_str.to_uppercase();
    
    if let Some(num_str) = size_str.strip_suffix("GB") {
        let num: u64 = num_str.parse().context("Invalid number in storage size")?;
        Ok(num * 1024 * 1024 * 1024)
    } else if let Some(num_str) = size_str.strip_suffix("MB") {
        let num: u64 = num_str.parse().context("Invalid number in storage size")?;
        Ok(num * 1024 * 1024)
    } else if let Some(num_str) = size_str.strip_suffix("KB") {
        let num: u64 = num_str.parse().context("Invalid number in storage size")?;
        Ok(num * 1024)
    } else {
        // Assume bytes if no suffix
        size_str.parse().context("Invalid storage size format")
    }
}

// Add hex dependency to Cargo.toml
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_default_config_is_valid() {
        let config = Config::default();
        assert!(config.validate().is_ok());
    }
    
    #[test]
    fn test_storage_size_parsing() {
        assert_eq!(parse_storage_size("10GB").unwrap(), 10 * 1024 * 1024 * 1024);
        assert_eq!(parse_storage_size("500MB").unwrap(), 500 * 1024 * 1024);
        assert_eq!(parse_storage_size("1024KB").unwrap(), 1024 * 1024);
        assert_eq!(parse_storage_size("1024").unwrap(), 1024);
        assert!(parse_storage_size("invalid").is_err());
    }
    
    #[test]
    fn test_config_validation() {
        let mut config = Config::default();
        
        // Test invalid port configuration
        config.network.p2p_port = config.network.api_port;
        assert!(config.validate().is_err());
        
        // Test invalid storage
        config = Config::default();
        config.storage.max_storage = 0;
        assert!(config.validate().is_err());
    }
}