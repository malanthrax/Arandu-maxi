use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::UdpSocket;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;
use tokio::time::interval;
use tracing::{debug, error, info, warn};
use tauri::Emitter;

#[cfg(test)]
use uuid::Uuid;

const DISCOVERY_PROTOCOL: &str = "arandu-discovery";
const DISCOVERY_VERSION: &str = "1.0";
const BROADCAST_INTERVAL_SECS: u64 = 5;
const PEER_TIMEOUT_SECS: i64 = 30;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryBeacon {
    pub protocol: String,
    pub version: String,
    pub instance_id: String,
    pub hostname: String,
    pub api_endpoint: String,
    pub timestamp: String,
}

impl DiscoveryBeacon {
    pub fn new(instance_id: String, hostname: String, api_endpoint: String) -> Self {
        Self {
            protocol: DISCOVERY_PROTOCOL.to_string(),
            version: DISCOVERY_VERSION.to_string(),
            instance_id,
            hostname,
            api_endpoint,
            timestamp: Utc::now().to_rfc3339(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveredPeer {
    pub instance_id: String,
    pub hostname: String,
    pub ip_address: String,
    pub api_port: u16,
    pub api_endpoint: String,
    pub last_seen: DateTime<Utc>,
    pub is_reachable: bool,
    pub models: Vec<RemoteModel>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteModel {
    pub id: String,
    pub name: String,
    pub object: String,
    pub owned_by: String,
    pub instance_id: String,
    pub instance_hostname: String,
    pub api_endpoint: String,
    pub size_gb: Option<f64>,
    pub quantization: Option<String>,
    pub architecture: Option<String>,
    pub date: Option<i64>,
}

impl RemoteModel {
    fn from_openai_model(
        model: crate::openai_types::ModelInfo,
        instance_id: String,
        instance_hostname: String,
        api_endpoint: String,
    ) -> Self {
        let model_id = model.id.clone();
        Self {
            id: model_id.clone(),
            name: model_id,
            object: model.object,
            owned_by: model.owned_by,
            instance_id,
            instance_hostname,
            api_endpoint,
            size_gb: model.size_gb,
            quantization: model.quantization,
            architecture: model.architecture,
            date: model.date,
        }
    }
}

#[derive(Debug)]
struct PeerEntry {
    peer: DiscoveredPeer,
    last_seen: DateTime<Utc>,
}

type PeerCache = Arc<Mutex<HashMap<String, PeerEntry>>>;

#[derive(Debug)]
pub struct DiscoveryService {
    port: u16,
    instance_id: String,
    hostname: String,
    api_endpoint: String,
    broadcast_socket: Option<Arc<UdpSocket>>,
    listen_socket: Option<Arc<UdpSocket>>,
    peers: PeerCache,
    broadcast_handle: Option<JoinHandle<()>>,
    listen_handle: Option<JoinHandle<()>>,
    cleanup_handle: Option<JoinHandle<()>>,
    app_handle: Option<tauri::AppHandle>,
}

impl DiscoveryService {
    pub fn new(
        port: u16,
        instance_id: String,
        hostname: String,
        api_endpoint: String,
        app_handle: Option<tauri::AppHandle>,
    ) -> Self {
        Self {
            port,
            instance_id,
            hostname,
            api_endpoint,
            broadcast_socket: None,
            listen_socket: None,
            peers: Arc::new(Mutex::new(HashMap::new())),
            broadcast_handle: None,
            listen_handle: None,
            cleanup_handle: None,
            app_handle,
        }
    }

    fn log_event(&self, direction: &str, ip: &str, data: &str, event_type: &str) {
        if let Some(app) = &self.app_handle {
            let payload = serde_json::json!({
                "direction": direction,
                "ip": ip,
                "data": data,
                "type": event_type,
                "timestamp": Utc::now().to_rfc3339(),
            });
            let _ = app.emit("discovery-debug-log", payload);
        }
    }

    pub async fn start_broadcasting(&mut self) -> Result<(), String> {
        if self.broadcast_handle.is_some() {
            return Err("Broadcasting already started".to_string());
        }

        let broadcast_addr = SocketAddr::from(([255, 255, 255, 255], self.port));
        let local_addr = SocketAddr::from(([0, 0, 0, 0], 0));

        let socket = UdpSocket::bind(local_addr)
            .await
            .map_err(|e| format!("Failed to bind broadcast socket: {}", e))?;

        socket
            .set_broadcast(true)
            .map_err(|e| format!("Failed to enable broadcast: {}", e))?;

        let socket = Arc::new(socket);
        self.broadcast_socket = Some(socket.clone());

        let beacon = DiscoveryBeacon::new(
            self.instance_id.clone(),
            self.hostname.clone(),
            self.api_endpoint.clone(),
        );

        let beacon_json =
            serde_json::to_vec(&beacon).map_err(|e| format!("Failed to serialize beacon: {}", e))?;

        let handle = tokio::spawn(async move {
            let mut ticker = interval(Duration::from_secs(BROADCAST_INTERVAL_SECS));
            
            loop {
                ticker.tick().await;
                
                debug!("Broadcasting discovery beacon to {}", broadcast_addr);
                
                if let Err(e) = socket.send_to(&beacon_json, broadcast_addr).await {
                    warn!("Failed to send discovery beacon: {}", e);
                } else {
                    debug!("Discovery beacon sent successfully");
                }
            }
        });
        
        self.log_event(
            "SEND",
            "255.255.255.255",
            &format!("Started broadcasting on port {} (every {}s)", self.port, BROADCAST_INTERVAL_SECS),
            "info"
        );

        self.broadcast_handle = Some(handle);
        info!("Started broadcasting discovery beacons on port {}", self.port);

        Ok(())
    }

pub async fn start_listening(&mut self) -> Result<(), String> {
        if self.listen_handle.is_some() {
            return Err("Listening already started".to_string());
        }

        let addr = SocketAddr::from(([0, 0, 0, 0], self.port));
        
        let socket = UdpSocket::bind(&addr)
            .await
            .map_err(|e| format!("Failed to bind listen socket to {}: {}", addr, e))?;

        let socket = Arc::new(socket);
        self.listen_socket = Some(socket.clone());

        let peers = self.peers.clone();
        let own_instance_id = self.instance_id.clone();
        let app_handle_opt = self.app_handle.clone();
        let discovery_port = self.port;

        let emit_log = move |direction: &str, ip: &str, data: &str, event_type: &str| {
            if let Some(app) = app_handle_opt.clone() {
                let payload = serde_json::json!({
                    "direction": direction,
                    "ip": ip,
                    "data": data,
                    "type": event_type,
                    "timestamp": Utc::now().to_rfc3339(),
                });
                let _ = app.emit("discovery-debug-log", payload);
            }
        };

        let handle = tokio::spawn(async move {
            let mut buf = vec![0u8; 4096];

            loop {
                match socket.recv_from(&mut buf).await {
                    Ok((len, src)) => {
                        debug!("Received {} bytes from {}", len, src);

                            match serde_json::from_slice::<DiscoveryBeacon>(&buf[..len]) {
                            Ok(beacon) => {
                                if beacon.protocol != DISCOVERY_PROTOCOL {
                                    debug!("Ignoring beacon with unknown protocol: {}", beacon.protocol);
                                    emit_log(
                                        "RECV",
                                        &src.ip().to_string(),
                                        &format!(
                                            "Received packet with unknown protocol {} from {}",
                                            beacon.protocol,
                                            src
                                        ),
                                        "info"
                                    );
                                    continue;
                                }

                                if beacon.instance_id == own_instance_id {
                                    debug!("Ignoring own beacon");
                                    emit_log(
                                        "RECV",
                                        &src.ip().to_string(),
                                        "Ignoring own discovery beacon",
                                        "info"
                                    );
                                    continue;
                                }

                                debug!("Received beacon from {} at {}", beacon.hostname, src);
                                emit_log(
                                    "RECV",
                                    &src.ip().to_string(),
                                    &format!(
                                        "Beacon received from {} (api: {})",
                                        beacon.hostname,
                                        beacon.api_endpoint
                                    ),
                                    "receive"
                                );
                                
                                let peer_ip = src.ip().to_string();
                                let peer_port = Self::extract_port(&beacon.api_endpoint);

                                let peer = DiscoveredPeer {
                                    instance_id: beacon.instance_id.clone(),
                                    hostname: beacon.hostname.clone(),
                                    ip_address: peer_ip.clone(),
                                    api_port: peer_port,
                                    api_endpoint: beacon.api_endpoint.clone(),
                                    last_seen: Utc::now(),
                                    is_reachable: true,
                                    models: Vec::new(),
                                };

                                let mut peers_guard = peers.lock().await;
                                let is_new = !peers_guard.contains_key(&beacon.instance_id);
                                peers_guard.insert(
                                    beacon.instance_id.clone(),
                                    PeerEntry {
                                        peer: peer.clone(),
                                        last_seen: Utc::now(),
                                    },
                                );
                                drop(peers_guard);

                                if is_new {
                                    info!(
                                        "Discovered peer: {} ({}) at {}",
                                        beacon.hostname, beacon.instance_id, beacon.api_endpoint
                                    );
                                    emit_log(
                                        "RECV",
                                        &peer_ip,
                                        &format!(
                                            "Discovered new peer {} ({})",
                                            beacon.hostname,
                                            beacon.api_endpoint
                                        ),
                                        "receive"
                                    );
                                }
                            }
                            Err(e) => {
                                debug!("Failed to parse beacon from {}: {}", src, e);
                            }
                        }
                    }
                    Err(e) => {
                        error!("Error receiving UDP packet: {}", e);
                    }
                }
            }
        });

        self.listen_handle = Some(handle);

        let peers = self.peers.clone();
        let cleanup_handle = tokio::spawn(async move {
            let mut ticker = interval(Duration::from_secs(5));
            
            loop {
                ticker.tick().await;
                
                let mut peers_guard = peers.lock().await;
                let now = Utc::now();
                let to_remove: Vec<String> = peers_guard
                    .iter()
                    .filter(|(_, entry)| {
                        let elapsed = now.signed_duration_since(entry.last_seen);
                        elapsed.num_seconds() > PEER_TIMEOUT_SECS
                    })
                    .map(|(id, _)| id.clone())
                    .collect();

                for id in to_remove {
                    if let Some(entry) = peers_guard.remove(&id) {
                        info!("Peer {} timed out and was removed", entry.peer.hostname);
                    }
                }
                drop(peers_guard);
            }
        });

        self.cleanup_handle = Some(cleanup_handle);
        info!("Started listening for discovery beacons on port {}", discovery_port);

        Ok(())
    }

    fn extract_port(api_endpoint: &str) -> u16 {
        // Parse URL like "http://192.168.1.100:8081" or "http://192.168.1.100:8081/v1/models"
        // Extract the port after the host:port combination
        api_endpoint
            .split('/')
            .nth(2) // Get "host:port" part (index 2 after splitting by /)
            .and_then(|host_port| {
                host_port.split(':').nth(1) // Get port after ":"
            })
            .and_then(|port_str| port_str.parse::<u16>().ok())
            .unwrap_or(8081)
    }

    pub async fn fetch_peer_models(
        &self,
        ip: &str,
        port: u16,
    ) -> Result<Vec<RemoteModel>, String> {
        // Use the Arandu-specific endpoint that returns full model metadata
        let url = format!("http://{}:{}/v1/models/arandu", ip, port);
        
        debug!("Fetching models from peer at {}", url);
        
        // Log the request
        self.log_event(
            "SEND",
            &format!("{}:{}", ip, port),
            "GET /v1/models/arandu",
            "send"
        );

        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

        let response = match client
            .get(&url)
            .send()
            .await {
                Ok(resp) => resp,
                Err(e) => {
                    let err_msg = format!("Failed to fetch models from {}: {}", url, e);
                    self.log_event(
                        "ERROR",
                        &format!("{}:{}", ip, port),
                        &err_msg,
                        "error"
                    );
                    return Err(err_msg);
                }
            };

        if !response.status().is_success() {
            let err_msg = format!("Peer returned error status: {}", response.status());
            self.log_event(
                "ERROR",
                &format!("{}:{}", ip, port),
                &err_msg,
                "error"
            );
            return Err(err_msg);
        }

        let models_response: crate::openai_types::ModelsResponse = match response
            .json()
            .await {
                Ok(data) => data,
                Err(e) => {
                    let err_msg = format!("Failed to parse models response: {}", e);
                    self.log_event(
                        "ERROR",
                        &format!("{}:{}", ip, port),
                        &err_msg,
                        "error"
                    );
                    return Err(err_msg);
                }
            };

        let peer_info = {
            let peers_guard = self.peers.lock().await;
            peers_guard
                .values()
                .find(|entry| {
                    entry.peer.ip_address == ip && entry.peer.api_port == port
                })
                .map(|entry| {
                    (
                        entry.peer.instance_id.clone(),
                        entry.peer.hostname.clone(),
                        entry.peer.api_endpoint.clone(),
                    )
                })
        };

        let (instance_id, instance_hostname, api_endpoint) = match peer_info {
            Some(info) => info,
            None => {
                return Err(format!("Peer {}:{} not found in cache", ip, port));
            }
        };

        let remote_models: Vec<RemoteModel> = models_response
            .data
            .into_iter()
            .map(|model| {
                RemoteModel::from_openai_model(
                    model,
                    instance_id.clone(),
                    instance_hostname.clone(),
                    api_endpoint.clone(),
                )
            })
            .collect();

        // Log successful response
        self.log_event(
            "RECV",
            &format!("{}:{}", ip, port),
            &format!("HTTP 200 OK - {} models received", remote_models.len()),
            "receive"
        );

        {
            let mut peers_guard = self.peers.lock().await;
            if let Some(entry) = peers_guard.get_mut(&instance_id) {
                entry.peer.models = remote_models.clone();
                entry.peer.is_reachable = true;
            }
        }

        info!(
            "Fetched {} models from peer {} at {}:{}",
            remote_models.len(),
            instance_hostname,
            ip,
            port
        );
        
        // Log the response
        let model_names: Vec<String> = remote_models.iter()
            .map(|m| format!("{} ({:.2} GB)", m.name, m.size_gb.unwrap_or(0.0)))
            .collect();
        self.log_event(
            "RECV",
            &format!("{}:{}", ip, port),
            &format!("Got {} models: {}", remote_models.len(), model_names.join(", ")),
            "receive"
        );

        Ok(remote_models)
    }

    pub async fn get_peers(&self) -> Vec<DiscoveredPeer> {
        let peers_guard = self.peers.lock().await;
        peers_guard
            .values()
            .map(|entry| entry.peer.clone())
            .collect()
    }

    pub fn stop(&mut self) {
        info!("Stopping discovery service");

        if let Some(handle) = self.broadcast_handle.take() {
            handle.abort();
            info!("Broadcasting stopped");
        }

        if let Some(handle) = self.listen_handle.take() {
            handle.abort();
            info!("Listening stopped");
        }

        if let Some(handle) = self.cleanup_handle.take() {
            handle.abort();
            info!("Cleanup task stopped");
        }

        self.broadcast_socket = None;
        self.listen_socket = None;
    }
}

impl Drop for DiscoveryService {
    fn drop(&mut self) {
        self.stop();
    }
}

// Additional methods for Tauri command compatibility
impl DiscoveryService {
    /// Check if the service is currently running (broadcasting and/or listening)
    pub fn is_running(&self) -> bool {
        self.broadcast_handle.is_some() || self.listen_handle.is_some()
    }
    
    /// Get the port used by this discovery service
    pub fn get_port(&self) -> u16 {
        self.port
    }
    
    /// Get the instance ID
    pub fn get_instance_id(&self) -> &str {
        &self.instance_id
    }
    
    /// Get the hostname/instance name
    pub fn get_hostname(&self) -> &str {
        &self.hostname
    }

    /// Get the API port from the api_endpoint
    pub fn get_api_port(&self) -> u16 {
        self.api_endpoint
            .split(':')
            .last()
            .and_then(|p| p.parse().ok())
            .unwrap_or(8081)
    }

    /// Start both broadcasting and listening
    pub async fn start(&mut self) -> Result<(), String> {
        if !self.is_running() {
            self.start_broadcasting().await?;
            self.start_listening().await?;
        }
        Ok(())
    }

    /// Refresh models from all discovered peers
    pub async fn refresh_models(&self) -> Result<usize, String> {
        let peers = self.get_peers().await;
        let mut total_models = 0;

        for peer in &peers {
            match self.fetch_peer_models(&peer.ip_address, peer.api_port).await {
                Ok(models) => {
                    total_models += models.len();
                }
                Err(e) => {
                    warn!("Failed to fetch models from {}: {}", peer.hostname, e);
                    self.log_event(
                        "ERROR",
                        &format!("{}:{}", peer.ip_address, peer.api_port),
                        &format!("Failed to fetch models: {}", e),
                        "error"
                    );
                }
            }
        }

        Ok(total_models)
    }
}

/// Status information for the discovery service
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryStatus {
    pub enabled: bool,
    pub instance_id: String,
    pub instance_name: String,
    pub port: u16,
    pub api_port: u16,
    pub broadcast_interval_secs: u64,
}

impl Default for DiscoveryStatus {
    fn default() -> Self {
        Self {
            enabled: false,
            instance_id: String::new(),
            instance_name: String::new(),
            port: 0,
            api_port: 8081,
            broadcast_interval_secs: 30,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_discovery_beacon_serialization() {
        let beacon = DiscoveryBeacon::new(
            "test-instance-123".to_string(),
            "TestHost".to_string(),
            "http://192.168.1.100:8081".to_string(),
        );

        let json = serde_json::to_string(&beacon).unwrap();
        let deserialized: DiscoveryBeacon = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.protocol, DISCOVERY_PROTOCOL);
        assert_eq!(deserialized.version, DISCOVERY_VERSION);
        assert_eq!(deserialized.instance_id, "test-instance-123");
        assert_eq!(deserialized.hostname, "TestHost");
        assert_eq!(deserialized.api_endpoint, "http://192.168.1.100:8081");
    }

    #[test]
    fn test_extract_port() {
        assert_eq!(DiscoveryService::extract_port("http://10.0.0.1:8081"), 8081);
        assert_eq!(DiscoveryService::extract_port("http://localhost:3000"), 3000);
        assert_eq!(DiscoveryService::extract_port("http://example.com"), 8081); // Default
    }

    #[tokio::test]
    async fn test_discovery_service_lifecycle() {
        let instance_id = Uuid::new_v4().to_string();
        let mut service = DiscoveryService::new(
            35000,
            instance_id.clone(),
            "TestInstance".to_string(),
            "http://127.0.0.1:8081".to_string(),
            None,
        );

        assert!(service.start_broadcasting().await.is_ok());
        assert!(service.start_listening().await.is_ok());

        tokio::time::sleep(Duration::from_millis(100)).await;

        let peers = service.get_peers().await;
        assert!(peers.is_empty());

        service.stop();
    }
}
