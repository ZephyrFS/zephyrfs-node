use anyhow::Result;
use libp2p::{Multiaddr, PeerId};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

use crate::config::Config;

/// Manages peer connections with reputation and security controls
/// 
/// Safety: Implements peer reputation system to prevent malicious connections
/// Privacy: Tracks minimal necessary peer information
/// Transparency: All peer management decisions are logged
pub struct PeerManager {
    peers: RwLock<HashMap<PeerId, PeerInfo>>,
    config: Config,
}

/// Information tracked for each peer
/// 
/// Privacy: Only stores network-relevant information, no personal data
#[derive(Debug, Clone)]
pub struct PeerInfo {
    /// Peer's multiaddresses
    pub addresses: Vec<Multiaddr>,
    
    /// Connection timestamp
    pub connected_at: Instant,
    
    /// Last successful ping timestamp
    pub last_ping: Option<Instant>,
    
    /// Round-trip time measurements
    pub rtt_measurements: Vec<Duration>,
    
    /// Reputation score (0.0 to 1.0)
    pub reputation: f64,
    
    /// Number of failed interactions
    pub failure_count: u32,
    
    /// Peer capabilities from identify protocol
    pub agent_version: Option<String>,
    pub protocol_version: Option<String>,
}

impl PeerInfo {
    fn new(address: Multiaddr) -> Self {
        Self {
            addresses: vec![address],
            connected_at: Instant::now(),
            last_ping: None,
            rtt_measurements: Vec::with_capacity(10), // Keep last 10 measurements
            reputation: 0.5, // Start with neutral reputation
            failure_count: 0,
            agent_version: None,
            protocol_version: None,
        }
    }
    
    /// Calculate average RTT from recent measurements
    pub fn average_rtt(&self) -> Option<Duration> {
        if self.rtt_measurements.is_empty() {
            None
        } else {
            let sum: Duration = self.rtt_measurements.iter().sum();
            Some(sum / self.rtt_measurements.len() as u32)
        }
    }
    
    /// Check if peer is considered healthy
    pub fn is_healthy(&self) -> bool {
        self.reputation >= 0.3 && self.failure_count < 5
    }
}

impl PeerManager {
    /// Create a new PeerManager
    pub fn new(config: &Config) -> Self {
        info!("Initializing PeerManager with security policies");
        
        Self {
            peers: RwLock::new(HashMap::new()),
            config: config.clone(),
        }
    }
    
    /// Add a new peer to management
    /// 
    /// Transparency: Log all peer additions for audit trail
    pub async fn add_peer(&self, peer_id: PeerId, address: Multiaddr) {
        let mut peers = self.peers.write().await;
        
        match peers.get_mut(&peer_id) {
            Some(peer_info) => {
                // Update existing peer info
                if !peer_info.addresses.contains(&address) {
                    peer_info.addresses.push(address.clone());
                    debug!("Added new address {} for existing peer {}", address, peer_id);
                }
            }
            None => {
                // Add new peer
                let peer_info = PeerInfo::new(address.clone());
                peers.insert(peer_id, peer_info);
                info!("Added new peer {} with address {}", peer_id, address);
            }
        }
    }
    
    /// Remove a peer from management
    /// 
    /// Transparency: Log all peer removals
    pub async fn remove_peer(&self, peer_id: &PeerId) {
        let mut peers = self.peers.write().await;
        if peers.remove(peer_id).is_some() {
            info!("Removed peer {}", peer_id);
        }
    }
    
    /// Check if we should connect to a specific peer
    /// 
    /// Safety: Apply security policies before allowing connections
    pub async fn should_connect_to_peer(&self, peer_id: &PeerId) -> bool {
        let peers = self.peers.read().await;
        
        // Check if we've reached max peers
        if peers.len() >= self.config.network.max_peers {
            debug!("Max peers reached, rejecting connection to {}", peer_id);
            return false;
        }
        
        // Check peer reputation if known
        if let Some(peer_info) = peers.get(peer_id) {
            if peer_info.reputation < self.config.security.min_peer_reputation {
                debug!("Peer {} has low reputation: {}, rejecting connection", 
                       peer_id, peer_info.reputation);
                return false;
            }
            
            if !peer_info.is_healthy() {
                debug!("Peer {} is not healthy, rejecting connection", peer_id);
                return false;
            }
        }
        
        true
    }
    
    /// Update peer statistics after successful ping
    /// 
    /// Transparency: All RTT measurements are logged for network health monitoring
    pub async fn update_peer_stats(&self, peer_id: &PeerId, rtt: Duration) {
        let mut peers = self.peers.write().await;
        
        if let Some(peer_info) = peers.get_mut(peer_id) {
            peer_info.last_ping = Some(Instant::now());
            
            // Add RTT measurement, keeping only last 10
            peer_info.rtt_measurements.push(rtt);
            if peer_info.rtt_measurements.len() > 10 {
                peer_info.rtt_measurements.remove(0);
            }
            
            // Improve reputation for successful pings
            peer_info.reputation = (peer_info.reputation + 0.01).min(1.0);
            
            debug!("Updated stats for peer {}: RTT={:?}, reputation={:.3}", 
                   peer_id, rtt, peer_info.reputation);
        }
    }
    
    /// Handle peer failure (failed ping, connection error, etc.)
    /// 
    /// Safety: Degrade reputation and potentially disconnect problematic peers
    pub async fn handle_peer_failure(&self, peer_id: &PeerId) {
        let mut peers = self.peers.write().await;
        
        if let Some(peer_info) = peers.get_mut(peer_id) {
            peer_info.failure_count += 1;
            peer_info.reputation = (peer_info.reputation - 0.1).max(0.0);
            
            warn!("Peer {} failed interaction: failures={}, reputation={:.3}", 
                  peer_id, peer_info.failure_count, peer_info.reputation);
            
            // Remove peer if too many failures
            if peer_info.failure_count >= 10 || peer_info.reputation <= 0.0 {
                warn!("Removing peer {} due to excessive failures", peer_id);
                peers.remove(peer_id);
            }
        }
    }
    
    /// Validate peer identity from identify protocol
    /// 
    /// Safety: Ensure peer is running compatible protocol version
    pub async fn validate_peer_identity(
        &self, 
        peer_id: &PeerId, 
        info: &libp2p::identify::Info
    ) -> Result<()> {
        let mut peers = self.peers.write().await;
        
        if let Some(peer_info) = peers.get_mut(peer_id) {
            peer_info.agent_version = Some(info.agent_version.clone());
            peer_info.protocol_version = Some(info.protocol_version.clone());
            
            // Validate protocol compatibility
            if !info.protocol_version.starts_with("zephyrfs/") {
                warn!("Peer {} running incompatible protocol: {}", 
                      peer_id, info.protocol_version);
                peer_info.reputation = (peer_info.reputation - 0.2).max(0.0);
            } else {
                // Boost reputation for compatible peers
                peer_info.reputation = (peer_info.reputation + 0.05).min(1.0);
                info!("Validated compatible peer {}: {}", peer_id, info.agent_version);
            }
        }
        
        Ok(())
    }
    
    /// Get statistics about managed peers
    /// 
    /// Transparency: Provide comprehensive peer statistics for monitoring
    pub async fn get_peer_stats(&self) -> PeerStats {
        let peers = self.peers.read().await;
        
        let mut healthy_peers = 0;
        let mut total_rtt = Duration::ZERO;
        let mut rtt_count = 0;
        let mut avg_reputation = 0.0;
        
        for peer_info in peers.values() {
            if peer_info.is_healthy() {
                healthy_peers += 1;
            }
            
            if let Some(rtt) = peer_info.average_rtt() {
                total_rtt += rtt;
                rtt_count += 1;
            }
            
            avg_reputation += peer_info.reputation;
        }
        
        PeerStats {
            total_peers: peers.len(),
            healthy_peers,
            average_rtt: if rtt_count > 0 { 
                Some(total_rtt / rtt_count) 
            } else { 
                None 
            },
            average_reputation: if peers.is_empty() { 
                0.0 
            } else { 
                avg_reputation / peers.len() as f64 
            },
        }
    }
}

/// Statistics about peer network health
#[derive(Debug, Clone)]
pub struct PeerStats {
    pub total_peers: usize,
    pub healthy_peers: usize,
    pub average_rtt: Option<Duration>,
    pub average_reputation: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use libp2p::identity;
    
    #[tokio::test]
    async fn test_peer_manager_creation() {
        let config = Config::default();
        let peer_manager = PeerManager::new(&config);
        
        let stats = peer_manager.get_peer_stats().await;
        assert_eq!(stats.total_peers, 0);
        assert_eq!(stats.healthy_peers, 0);
    }
    
    #[tokio::test]
    async fn test_add_and_remove_peer() {
        let config = Config::default();
        let peer_manager = PeerManager::new(&config);
        
        let keypair = identity::Keypair::generate_ed25519();
        let peer_id = PeerId::from(keypair.public());
        let address: Multiaddr = "/ip4/127.0.0.1/tcp/4001".parse().unwrap();
        
        // Add peer
        peer_manager.add_peer(peer_id, address.clone()).await;
        
        let stats = peer_manager.get_peer_stats().await;
        assert_eq!(stats.total_peers, 1);
        assert_eq!(stats.healthy_peers, 1); // New peers start healthy
        
        // Remove peer
        peer_manager.remove_peer(&peer_id).await;
        
        let stats = peer_manager.get_peer_stats().await;
        assert_eq!(stats.total_peers, 0);
    }
    
    #[tokio::test]
    async fn test_peer_reputation_system() {
        let config = Config::default();
        let peer_manager = PeerManager::new(&config);
        
        let keypair = identity::Keypair::generate_ed25519();
        let peer_id = PeerId::from(keypair.public());
        let address: Multiaddr = "/ip4/127.0.0.1/tcp/4001".parse().unwrap();
        
        // Add peer
        peer_manager.add_peer(peer_id, address).await;
        
        // Test successful ping improves reputation
        peer_manager.update_peer_stats(&peer_id, Duration::from_millis(50)).await;
        
        let stats = peer_manager.get_peer_stats().await;
        assert!(stats.average_reputation > 0.5, "Reputation should improve after successful ping");
        
        // Test failure degrades reputation
        peer_manager.handle_peer_failure(&peer_id).await;
        
        let stats = peer_manager.get_peer_stats().await;
        assert!(stats.average_reputation < 0.5, "Reputation should degrade after failure");
    }
    
    #[tokio::test]
    async fn test_should_connect_to_peer() {
        let mut config = Config::default();
        config.network.max_peers = 2;
        config.security.min_peer_reputation = 0.3;
        
        let peer_manager = PeerManager::new(&config);
        
        let keypair1 = identity::Keypair::generate_ed25519();
        let peer_id1 = PeerId::from(keypair1.public());
        
        // Should connect to unknown peer
        assert!(peer_manager.should_connect_to_peer(&peer_id1).await);
        
        // Add peer and degrade reputation
        let address: Multiaddr = "/ip4/127.0.0.1/tcp/4001".parse().unwrap();
        peer_manager.add_peer(peer_id1, address).await;
        
        // Degrade reputation below threshold by causing many failures
        for _ in 0..5 {  
            peer_manager.handle_peer_failure(&peer_id1).await;
        }
        
        // Check if peer was removed or has low reputation
        let should_connect = peer_manager.should_connect_to_peer(&peer_id1).await;
        assert!(!should_connect, "Should not connect to peer with degraded reputation");
    }
    
    #[test]
    fn test_peer_info_health_check() {
        let address: Multiaddr = "/ip4/127.0.0.1/tcp/4001".parse().unwrap();
        let mut peer_info = PeerInfo::new(address);
        
        // Should start healthy
        assert!(peer_info.is_healthy());
        
        // Should become unhealthy with low reputation
        peer_info.reputation = 0.1;
        assert!(!peer_info.is_healthy());
        
        // Should become unhealthy with many failures
        peer_info.reputation = 0.5;
        peer_info.failure_count = 10;
        assert!(!peer_info.is_healthy());
    }
    
    #[test]
    fn test_peer_info_rtt_calculation() {
        let address: Multiaddr = "/ip4/127.0.0.1/tcp/4001".parse().unwrap();
        let mut peer_info = PeerInfo::new(address);
        
        // No measurements initially
        assert!(peer_info.average_rtt().is_none());
        
        // Add measurements
        peer_info.rtt_measurements.push(Duration::from_millis(50));
        peer_info.rtt_measurements.push(Duration::from_millis(100));
        
        let avg_rtt = peer_info.average_rtt().unwrap();
        assert_eq!(avg_rtt, Duration::from_millis(75));
    }
}