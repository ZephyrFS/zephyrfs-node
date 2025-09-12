use anyhow::{Context, Result};
use futures::StreamExt;
use libp2p::{
    core::upgrade,
    identity,
    mdns,
    noise,
    swarm::{SwarmEvent, Swarm, Config as SwarmConfig},
    tcp,
    yamux,
    Multiaddr,
    PeerId,
    Transport,
};
use std::time::Duration;
use tokio::sync::mpsc;
use tracing::{debug, info, warn, error};

use crate::config::Config;

pub mod behaviour;
pub mod peer_manager;
pub mod message_handler;

use behaviour::ZephyrBehaviour;
use peer_manager::PeerManager;
use message_handler::MessageHandler;

/// NetworkManager handles all P2P networking for ZephyrFS
/// 
/// Safety: All connections use encrypted transport with Noise protocol
/// Privacy: Peer discovery respects privacy settings and reputation
/// Transparency: All network events are logged for auditability
pub struct NetworkManager {
    swarm: Swarm<ZephyrBehaviour>,
    peer_manager: PeerManager,
    message_handler: MessageHandler,
    config: Config,
    shutdown_tx: Option<mpsc::Sender<()>>,
}

impl NetworkManager {
    /// Create a new NetworkManager instance
    /// 
    /// Safety: Generates cryptographically secure keypair for node identity
    pub async fn new(config: Config) -> Result<Self> {
        info!("Initializing NetworkManager with security-first design");
        
        // Generate or load node keypair (Privacy: never logged)
        let keypair = identity::Keypair::generate_ed25519();
        let peer_id = PeerId::from(keypair.public());
        info!("Node PeerID: {}", peer_id);
        
        // Create authenticated and encrypted transport
        // Safety: All communications use Noise encryption + TLS-like security
        let transport = tcp::tokio::Transport::default()
            .upgrade(upgrade::Version::V1)
            .authenticate(noise::Config::new(&keypair)?)
            .multiplex(yamux::Config::default())
            .timeout(Duration::from_secs(30))
            .boxed();
        
        // Initialize network behaviour
        let behaviour = ZephyrBehaviour::new(&config, peer_id, &keypair).await?;
        
        // Create swarm with secure transport
        let swarm = Swarm::new(transport, behaviour, peer_id, SwarmConfig::with_tokio_executor());
        
        // Initialize managers
        let peer_manager = PeerManager::new(&config);
        let message_handler = MessageHandler::new(&config);
        
        Ok(Self {
            swarm,
            peer_manager,
            message_handler,
            config,
            shutdown_tx: None,
        })
    }
    
    /// Start the network manager
    /// 
    /// Transparency: All startup steps are logged
    pub async fn start(&mut self) -> Result<()> {
        info!("Starting ZephyrFS networking with privacy and security protections");
        
        // Start listening on configured port
        let listen_addr = format!("/ip4/0.0.0.0/tcp/{}", self.config.network.p2p_port);
        self.swarm.listen_on(listen_addr.parse()?)
            .context("Failed to start listening")?;
        
        info!("Listening on port {}", self.config.network.p2p_port);
        
        // Connect to bootstrap peers if configured
        self.connect_bootstrap_peers().await?;
        
        // Setup shutdown channel
        let (shutdown_tx, mut shutdown_rx) = mpsc::channel(1);
        self.shutdown_tx = Some(shutdown_tx);
        
        // Main event loop
        loop {
            tokio::select! {
                event = self.swarm.select_next_some() => {
                    if let Err(e) = self.handle_swarm_event(event).await {
                        error!("Error handling swarm event: {}", e);
                    }
                }
                
                _ = shutdown_rx.recv() => {
                    info!("Shutdown signal received, stopping network manager");
                    break;
                }
            }
        }
        
        Ok(())
    }
    
    /// Connect to bootstrap peers for initial network discovery
    /// 
    /// Safety: Only connects to explicitly configured peers
    async fn connect_bootstrap_peers(&mut self) -> Result<()> {
        if self.config.network.bootstrap_peers.is_empty() {
            info!("No bootstrap peers configured, relying on mDNS discovery");
            return Ok(());
        }
        
        info!("Connecting to {} bootstrap peers", self.config.network.bootstrap_peers.len());
        
        for peer_addr in &self.config.network.bootstrap_peers {
            match peer_addr.parse::<Multiaddr>() {
                Ok(addr) => {
                    info!("Connecting to bootstrap peer: {}", peer_addr);
                    if let Err(e) = self.swarm.dial(addr) {
                        warn!("Failed to dial bootstrap peer {}: {}", peer_addr, e);
                    }
                }
                Err(e) => {
                    warn!("Invalid bootstrap peer address {}: {}", peer_addr, e);
                }
            }
        }
        
        Ok(())
    }
    
    /// Handle swarm events with comprehensive logging
    /// 
    /// Transparency: All network events are logged for audit trail
    async fn handle_swarm_event(&mut self, event: SwarmEvent<behaviour::ZephyrBehaviourEvent>) -> Result<()> {
        match event {
            SwarmEvent::NewListenAddr { address, .. } => {
                info!("Listening on address: {}", address);
            }
            
            SwarmEvent::ConnectionEstablished { peer_id, endpoint, .. } => {
                info!("Connection established with peer: {} via {}", peer_id, endpoint.get_remote_address());
                self.peer_manager.add_peer(peer_id, endpoint.get_remote_address().clone()).await;
            }
            
            SwarmEvent::ConnectionClosed { peer_id, cause, .. } => {
                match cause {
                    Some(error) => warn!("Connection closed with peer {}: {}", peer_id, error),
                    None => info!("Connection closed with peer {}", peer_id),
                }
                self.peer_manager.remove_peer(&peer_id).await;
            }
            
            SwarmEvent::IncomingConnection { local_addr, send_back_addr, .. } => {
                debug!("Incoming connection from {} to {}", send_back_addr, local_addr);
                // Safety: Let behaviour handle connection acceptance based on security rules
            }
            
            SwarmEvent::IncomingConnectionError { local_addr, send_back_addr, error, .. } => {
                warn!("Incoming connection error from {} to {}: {}", send_back_addr, local_addr, error);
            }
            
            SwarmEvent::OutgoingConnectionError { peer_id, error, .. } => {
                match peer_id {
                    Some(peer) => warn!("Outgoing connection error to {}: {}", peer, error),
                    None => warn!("Outgoing connection error: {}", error),
                }
            }
            
            SwarmEvent::Behaviour(event) => {
                self.handle_behaviour_event(event).await?;
            }
            
            _ => {
                debug!("Unhandled swarm event: {:?}", event);
            }
        }
        
        Ok(())
    }
    
    /// Handle behaviour-specific events
    /// 
    /// Privacy: Peer discovery events respect reputation and trust settings
    async fn handle_behaviour_event(&mut self, event: behaviour::ZephyrBehaviourEvent) -> Result<()> {
        match event {
            behaviour::ZephyrBehaviourEvent::Mdns(mdns::Event::Discovered(list)) => {
                for (peer_id, multiaddr) in list {
                    info!("Discovered peer via mDNS: {} at {}", peer_id, multiaddr);
                    
                    // Safety: Apply peer reputation check before connecting
                    if self.peer_manager.should_connect_to_peer(&peer_id).await {
                        if let Err(e) = self.swarm.dial(multiaddr.clone()) {
                            warn!("Failed to dial discovered peer {}: {}", peer_id, e);
                        }
                    } else {
                        debug!("Skipping connection to peer {} due to security policy", peer_id);
                    }
                }
            }
            
            behaviour::ZephyrBehaviourEvent::Mdns(mdns::Event::Expired(list)) => {
                for (peer_id, multiaddr) in list {
                    debug!("mDNS entry expired for peer: {} at {}", peer_id, multiaddr);
                }
            }
            
            behaviour::ZephyrBehaviourEvent::Ping(ping_event) => {
                match ping_event {
                    libp2p::ping::Event {
                        peer,
                        result: Ok(rtt),
                        ..
                    } => {
                        debug!("Ping to {} successful: {:?}", peer, rtt);
                        self.peer_manager.update_peer_stats(&peer, rtt).await;
                    }
                    
                    libp2p::ping::Event {
                        peer,
                        result: Err(error),
                        ..
                    } => {
                        warn!("Ping to {} failed: {}", peer, error);
                        self.peer_manager.handle_peer_failure(&peer).await;
                    }
                }
            }
            
            behaviour::ZephyrBehaviourEvent::Identify(identify_event) => {
                match identify_event {
                    libp2p::identify::Event::Received { peer_id, info, .. } => {
                        info!("Identified peer {}: protocol_version={}, agent_version={}", 
                              peer_id, info.protocol_version, info.agent_version);
                        
                        // Safety: Validate peer compatibility before full trust
                        self.peer_manager.validate_peer_identity(&peer_id, &info).await?;
                    }
                    
                    libp2p::identify::Event::Sent { peer_id, .. } => {
                        debug!("Sent identification to peer: {}", peer_id);
                    }
                    
                    libp2p::identify::Event::Pushed { peer_id, .. } => {
                        debug!("Pushed identification to peer: {}", peer_id);
                    }
                    
                    libp2p::identify::Event::Error { peer_id, error, .. } => {
                        warn!("Identify error with peer {}: {}", peer_id, error);
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// Shutdown the network manager gracefully
    /// 
    /// Transparency: Shutdown process is fully logged
    pub async fn shutdown(&mut self) -> Result<()> {
        info!("Shutting down NetworkManager gracefully");
        
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(()).await;
        }
        
        // Close all connections cleanly
        let connected_peers: Vec<_> = self.swarm.connected_peers().cloned().collect();
        for peer in connected_peers {
            info!("Disconnecting from peer: {}", peer);
            let _ = self.swarm.disconnect_peer_id(peer);
        }
        
        info!("NetworkManager shutdown complete");
        Ok(())
    }
    
    /// Get current network statistics for monitoring
    /// 
    /// Transparency: Network stats available for health monitoring
    pub fn get_network_stats(&self) -> NetworkStats {
        NetworkStats {
            connected_peers: self.swarm.connected_peers().count(),
            listening_addresses: self.swarm.listeners().count(),
            pending_connections: self.swarm.network_info().num_peers(),
        }
    }
}

/// Network statistics for monitoring and transparency
#[derive(Debug, Clone)]
pub struct NetworkStats {
    pub connected_peers: usize,
    pub listening_addresses: usize,
    pub pending_connections: usize,
}