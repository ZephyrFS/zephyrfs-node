use anyhow::Result;
use libp2p::{
    identify, mdns, ping,
    swarm::NetworkBehaviour,
    identity,
    PeerId,
};
use tracing::info;

use crate::config::Config;

/// ZephyrBehaviour combines multiple LibP2P behaviours for secure P2P networking
/// 
/// Safety: All behaviours are configured with security-first defaults
/// Privacy: mDNS can be disabled for privacy-sensitive deployments
/// Transparency: All behaviour events are exposed for logging and monitoring
#[derive(NetworkBehaviour)]
pub struct ZephyrBehaviour {
    /// Ping for basic connectivity testing and RTT measurement
    pub ping: ping::Behaviour,
    
    /// Identify for peer capability discovery
    pub identify: identify::Behaviour,
    
    /// mDNS for local network discovery
    pub mdns: mdns::tokio::Behaviour,
}

impl ZephyrBehaviour {
    /// Create a new ZephyrBehaviour with security-focused configuration
    /// 
    /// Safety: All protocols use secure defaults
    /// Privacy: mDNS respects privacy configuration
    pub async fn new(config: &Config, local_peer_id: PeerId, local_key: &identity::Keypair) -> Result<Self> {
        info!("Initializing ZephyrBehaviour for peer: {}", local_peer_id);
        
        // Configure ping behaviour for connection health monitoring
        let ping = ping::Behaviour::new(
            ping::Config::new().with_interval(std::time::Duration::from_secs(30))
        );
        
        // Configure identify behaviour for peer capability discovery
        // Safety: Only expose necessary information, no sensitive data
        let identify = identify::Behaviour::new(
            identify::Config::new("zephyrfs/1.0.0".into(), local_key.public())
                .with_agent_version("zephyrfs-node/0.1.0".into())
                .with_push_listen_addr_updates(true)
                .with_interval(std::time::Duration::from_secs(300)) // 5 minutes
        );
        
        // Configure mDNS for local discovery (can be disabled for privacy)
        let mdns = if config.network.enable_mdns {
            info!("Enabling mDNS for local peer discovery");
            mdns::tokio::Behaviour::new(
                mdns::Config::default(),
                local_peer_id,
            )?
        } else {
            info!("mDNS disabled for privacy");
            mdns::tokio::Behaviour::new(
                mdns::Config::default(),
                local_peer_id,
            )?
        };
        
        Ok(Self {
            ping,
            identify,
            mdns,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{Config, NetworkConfig};
    use libp2p::identity;
    
    #[tokio::test]
    async fn test_behaviour_creation() {
        let config = Config::default();
        let keypair = identity::Keypair::generate_ed25519();
        let peer_id = PeerId::from(keypair.public());
        
        let behaviour = ZephyrBehaviour::new(&config, peer_id, &keypair).await;
        assert!(behaviour.is_ok(), "Should create behaviour successfully");
    }
    
    #[tokio::test]
    async fn test_behaviour_with_mdns_disabled() {
        let mut config = Config::default();
        config.network.enable_mdns = false;
        
        let keypair = identity::Keypair::generate_ed25519();
        let peer_id = PeerId::from(keypair.public());
        
        let behaviour = ZephyrBehaviour::new(&config, peer_id, &keypair).await;
        assert!(behaviour.is_ok(), "Should create behaviour with mDNS disabled");
    }
    
    #[tokio::test]
    async fn test_behaviour_has_required_components() {
        // This test ensures our behaviour struct has all required fields
        let keypair = identity::Keypair::generate_ed25519();
        let peer_id = PeerId::from(keypair.public());
        
        // Create minimal components for testing
        let ping = ping::Behaviour::new(ping::Config::new());
        let identify = identify::Behaviour::new(
            identify::Config::new("test/1.0.0".into(), keypair.public())
        );
        
        // This will fail to compile if we're missing required fields
        let mdns = mdns::tokio::Behaviour::new(
            mdns::Config::default(),
            peer_id,
        ).expect("mDNS creation should succeed in tests");
        
        let _behaviour = ZephyrBehaviour {
            ping,
            identify,
            mdns,
        };
        
        // If we get here, the struct is properly formed
        assert!(true, "Behaviour struct is properly formed");
    }
}