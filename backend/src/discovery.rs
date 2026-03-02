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

use crate::peer_cache::PeerModelCache;

const DISCOVERY_PROTOCOL: &str = "arandu-discovery";
const DISCOVERY_VERSION: &str = "1.0";
const PEER_TIMEOUT_SECS: i64 = 30;
const AUTO_FETCH_COOLDOWN_SECS: i64 = 10;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryBeacon {
    pub protocol: String,
    pub version: String,
    pub instance_id: String,
    pub hostname: String,
    pub api_endpoint: String,
    pub api_port: u16,
    pub chat_port: u16,
    pub timestamp: String,
}

impl DiscoveryBeacon {
    pub fn new(instance_id: String, hostname: String, api_endpoint: String, api_port: u16, chat_port: u16) -> Self {
        Self {
            protocol: DISCOVERY_PROTOCOL.to_string(),
            version: DISCOVERY_VERSION.to_string(),
            instance_id,
            hostname,
            api_endpoint,
            api_port,
            chat_port,
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
    pub chat_port: u16,
    pub api_endpoint: String,
    pub last_seen: DateTime<Utc>,
    pub is_reachable: bool,
    pub models_from_cache: bool,
    pub cache_last_updated: Option<DateTime<Utc>>,
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
    pub path: Option<String>,
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
            path: model.path,
        }
    }
}

#[derive(Debug, Clone)]
struct PeerEntry {
    peer: DiscoveredPeer,
    last_seen: DateTime<Utc>,
    last_fetch_attempt: Option<DateTime<Utc>>,
    fetch_in_progress: bool,
}

type PeerCache = Arc<Mutex<HashMap<String, PeerEntry>>>;

#[derive(Debug)]
pub struct DiscoveryService {
    port: u16,
    instance_id: String,
    hostname: String,
    api_endpoint: String,
    api_port: u16,
    chat_port: u16,
    broadcast_interval_secs: u64,
    broadcast_socket: Option<Arc<UdpSocket>>,
    listen_socket: Option<Arc<UdpSocket>>,
    peers: PeerCache,
    peer_model_cache: Option<Arc<PeerModelCache>>, // NEW: Persistent cache
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
        api_port: u16,
        chat_port: u16,
        broadcast_interval_secs: u64,
        app_handle: Option<tauri::AppHandle>,
        peer_model_cache: Option<Arc<PeerModelCache>>,
    ) -> Self {
        Self {
            port,
            instance_id,
            hostname,
            api_endpoint,
            api_port,
            chat_port,
            broadcast_interval_secs,
            broadcast_socket: None,
            listen_socket: None,
            peers: Arc::new(Mutex::new(HashMap::new())),
            peer_model_cache,
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
            self.api_port,
            self.chat_port,
        );

        let beacon_json =
            serde_json::to_vec(&beacon).map_err(|e| format!("Failed to serialize beacon: {}", e))?;

        let interval_secs = self.broadcast_interval_secs.max(1);
        let handle = tokio::spawn(async move {
            let mut ticker = interval(Duration::from_secs(interval_secs));
            
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
            &format!("Started broadcasting on port {} (every {}s)", self.port, interval_secs),
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
        let peer_model_cache_opt = self.peer_model_cache.clone();

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
                                let peer_port = if beacon.api_port != 0 {
                                    beacon.api_port
                                } else {
                                    Self::extract_port(&beacon.api_endpoint)
                                };

                                let now = Utc::now();
                                let mut peers_guard = peers.lock().await;
                                let existing_entry = peers_guard.get(&beacon.instance_id).cloned();
                                let is_new = existing_entry.is_none();

                                // Preserve previously-fetched models so beacons don't wipe them
                                let preserved_models = existing_entry
                                    .as_ref()
                                    .map(|entry| entry.peer.models.clone())
                                    .unwrap_or_default();

                                let prev_fetch_in_progress = existing_entry
                                    .as_ref()
                                    .map(|entry| entry.fetch_in_progress)
                                    .unwrap_or(false);

                                let prev_last_fetch_attempt = existing_entry
                                    .as_ref()
                                    .and_then(|entry| entry.last_fetch_attempt.clone());

                                let cooldown_elapsed = prev_last_fetch_attempt
                                    .as_ref()
                                    .map(|last| {
                                        now.signed_duration_since(last.clone()).num_seconds()
                                            >= AUTO_FETCH_COOLDOWN_SECS
                                    })
                                    .unwrap_or(true);

                                let should_autofetch = (is_new || preserved_models.is_empty())
                                    && !prev_fetch_in_progress
                                    && cooldown_elapsed;

                                let peer = DiscoveredPeer {
                                    instance_id: beacon.instance_id.clone(),
                                    hostname: beacon.hostname.clone(),
                                    ip_address: peer_ip.clone(),
                                    api_port: peer_port,
                                    chat_port: beacon.chat_port,
                                    api_endpoint: beacon.api_endpoint.clone(),
                                    last_seen: Utc::now(),
                                    is_reachable: true,
                                    models_from_cache: false,
                                    cache_last_updated: None,
                                    models: preserved_models,
                                };

                                peers_guard.insert(
                                    beacon.instance_id.clone(),
                                    PeerEntry {
                                        peer: peer.clone(),
                                        last_seen: now,
                                        last_fetch_attempt: if should_autofetch {
                                            Some(now)
                                        } else {
                                            prev_last_fetch_attempt
                                        },
                                        fetch_in_progress: should_autofetch,
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

                                if should_autofetch {
                                    // Trigger model fetch for peers that are new or still missing models.
                                    let peer_ip_clone = peer_ip.clone();
                                    let peer_port_clone = peer_port;
                                    let cache_clone = peer_model_cache_opt.clone();
                                    let peers_clone = peers.clone();
                                    let beacon_instance_id = beacon.instance_id.clone();
                                    let beacon_hostname = beacon.hostname.clone();
                                    let beacon_endpoint = beacon.api_endpoint.clone();
                                    let beacon_chat_port = beacon.chat_port;

                                    tokio::spawn(async move {
                                        let max_attempts = 4u8;
                                        let mut backoff_ms = 500u64;
                                        let url = format!(
                                            "http://{}:{}/v1/models/arandu",
                                            peer_ip_clone, peer_port_clone
                                        );

                                        let client = match reqwest::Client::builder()
                                            .timeout(Duration::from_secs(10))
                                            .build()
                                        {
                                            Ok(c) => c,
                                            Err(e) => {
                                                warn!("Failed to create HTTP client for auto-fetch: {}", e);
                                                let mut peers_guard = peers_clone.lock().await;
                                                if let Some(entry) = peers_guard.get_mut(&beacon_instance_id) {
                                                    entry.fetch_in_progress = false;
                                                    entry.last_fetch_attempt = Some(Utc::now());
                                                }
                                                return;
                                            }
                                        };

                                        for attempt in 1..=max_attempts {
                                            if attempt > 1 {
                                                tokio::time::sleep(Duration::from_millis(backoff_ms)).await;
                                                backoff_ms = (backoff_ms.saturating_mul(2)).min(8_000);
                                            }

                                            info!(
                                                "Auto-fetch attempt {}/{} for peer {} at {}:{}",
                                                attempt,
                                                max_attempts,
                                                beacon_hostname,
                                                peer_ip_clone,
                                                peer_port_clone
                                            );

                                            let response = match client.get(&url).send().await {
                                                Ok(resp) => resp,
                                                Err(e) => {
                                                    warn!(
                                                        "Auto-fetch request failed on attempt {}/{} for {}: {}",
                                                        attempt,
                                                        max_attempts,
                                                        beacon_hostname,
                                                        e
                                                    );
                                                    continue;
                                                }
                                            };

                                            if !response.status().is_success() {
                                                warn!(
                                                    "Auto-fetch non-success status on attempt {}/{} for {}: {}",
                                                    attempt,
                                                    max_attempts,
                                                    beacon_hostname,
                                                    response.status()
                                                );
                                                continue;
                                            }

                                            let models_response = match response
                                                .json::<crate::openai_types::ModelsResponse>()
                                                .await
                                            {
                                                Ok(payload) => payload,
                                                Err(e) => {
                                                    warn!(
                                                        "Auto-fetch parse failure on attempt {}/{} for {}: {}",
                                                        attempt,
                                                        max_attempts,
                                                        beacon_hostname,
                                                        e
                                                    );
                                                    continue;
                                                }
                                            };

                                            let models: Vec<RemoteModel> = models_response
                                                .data
                                                .into_iter()
                                                .map(|model| {
                                                    RemoteModel::from_openai_model(
                                                        model,
                                                        beacon_instance_id.clone(),
                                                        beacon_hostname.clone(),
                                                        beacon_endpoint.clone(),
                                                    )
                                                })
                                                .collect();

                                            info!(
                                                "Auto-fetched {} models from new peer {} on attempt {}/{}",
                                                models.len(),
                                                beacon_hostname,
                                                attempt,
                                                max_attempts
                                            );

                                            {
                                                let mut peers_guard = peers_clone.lock().await;
                                                if let Some(entry) = peers_guard.get_mut(&beacon_instance_id) {
                                                    entry.peer.models = models.clone();
                                                    entry.peer.is_reachable = true;
                                                    entry.fetch_in_progress = false;
                                                    entry.last_fetch_attempt = Some(Utc::now());
                                                }
                                            }

                                            if let Some(cache) = &cache_clone {
                                                let _ = cache
                                                    .update_peer_models(
                                                        beacon_instance_id.clone(),
                                                        beacon_hostname.clone(),
                                                        peer_ip_clone.clone(),
                                                        peer_port_clone,
                                                        beacon_chat_port,
                                                        beacon_endpoint.clone(),
                                                        models,
                                                        true,
                                                    )
                                                    .await;
                                            }

                                            return;
                                        }

                                        warn!(
                                            "Auto-fetch exhausted all {} attempts for peer {} at {}:{}",
                                            max_attempts,
                                            beacon_hostname,
                                            peer_ip_clone,
                                            peer_port_clone
                                        );

                                        let mut peers_guard = peers_clone.lock().await;
                                        if let Some(entry) = peers_guard.get_mut(&beacon_instance_id) {
                                            entry.fetch_in_progress = false;
                                            entry.last_fetch_attempt = Some(Utc::now());
                                        }
                                    });
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

        // Save to persistent cache
        if let Some(cache) = &self.peer_model_cache {
            let delta = cache.update_peer_models(
                instance_id.clone(),
                instance_hostname.clone(),
                ip.to_string(),
                port,
                {
                    let peers_guard = self.peers.lock().await;
                    peers_guard
                        .get(&instance_id)
                        .map(|entry| entry.peer.chat_port)
                        .unwrap_or(port)
                },
                format!("http://{}:{}", ip, port),
                remote_models.clone(),
                true,
            ).await;
            
            if delta.any_changed {
                info!(
                    "Peer {} models changed (added: {}, removed: {}, updated: {})",
                    instance_id,
                    delta.added,
                    delta.removed,
                    delta.updated
                );
            }
        }

        Ok(remote_models)
    }

    pub async fn get_peers(&self) -> Vec<DiscoveredPeer> {
        let peers_guard = self.peers.lock().await;
        let mut peers: Vec<DiscoveredPeer> = peers_guard
            .values()
            .map(|entry| entry.peer.clone())
            .collect();
        drop(peers_guard);

        // Merge with cached models for stability
        if let Some(cache) = &self.peer_model_cache {
            for peer in &mut peers {
                // If runtime peer has no models but cache has them, use cached models
                if peer.models.is_empty() {
                    if let Some(cached_peer) = cache.get_peer(&peer.instance_id).await {
                        let cached_models = cached_peer.models;
                        if !cached_models.is_empty() {
                        debug!(
                            "Using {} cached models for peer {}",
                            cached_models.len(),
                            peer.hostname
                        );
                        peer.models = cached_models;
                        peer.models_from_cache = true;
                        peer.cache_last_updated = Some(cached_peer.last_updated);
                        }
                    }
                }
            }
        }

        peers
    }

    /// Get peers with model count from cache (for stable UI)
    pub async fn get_peers_with_cached_models(&self) -> Vec<DiscoveredPeer> {
        let peers_guard = self.peers.lock().await;
        let runtime_peers: HashMap<String, DiscoveredPeer> = peers_guard
            .values()
            .map(|entry| (entry.peer.instance_id.clone(), entry.peer.clone()))
            .collect();
        drop(peers_guard);

        if let Some(cache) = &self.peer_model_cache {
            let cached_peers = cache.get_all_peers().await;
            let mut merged_peers: HashMap<String, DiscoveredPeer> = runtime_peers;

            // Add cached peers that aren't in runtime (but mark as unreachable)
            for cached in cached_peers {
                if let Some(peer) = merged_peers.get_mut(&cached.instance_id) {
                    // Merge: use cached models if runtime has none
                    if peer.models.is_empty() && !cached.models.is_empty() {
                        peer.models = cached.models;
                        peer.models_from_cache = true;
                        peer.cache_last_updated = Some(cached.last_updated);
                    }
                } else {
                    // Add cached peer as offline
                    let offline_peer = DiscoveredPeer {
                        instance_id: cached.instance_id,
                        hostname: cached.hostname,
                        ip_address: cached.ip_address,
                        api_port: cached.api_port,
                        chat_port: cached.chat_port,
                        api_endpoint: cached.api_endpoint,
                        last_seen: cached.last_seen,
                        is_reachable: false, // Mark as offline
                        models_from_cache: true,
                        cache_last_updated: Some(cached.last_updated),
                        models: cached.models,
                    };
                    merged_peers.insert(offline_peer.instance_id.clone(), offline_peer);
                }
            }

            merged_peers.into_values().collect()
        } else {
            runtime_peers.into_values().collect()
        }
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

    pub fn get_api_port(&self) -> u16 {
        self.api_port
    }

    pub fn get_chat_port(&self) -> u16 {
        self.chat_port
    }

    pub fn get_instance_id(&self) -> &str {
        &self.instance_id
    }

    pub fn get_hostname(&self) -> &str {
        &self.hostname
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
    pub port: u16,
    pub api_port: u16,
    pub chat_port: u16,
    pub broadcast_interval: u64,
    pub instance_name: String,
    pub instance_id: String,
}

impl Default for DiscoveryStatus {
    fn default() -> Self {
        Self {
            enabled: false,
            instance_id: String::new(),
            instance_name: String::new(),
            port: 0,
            api_port: 8081,
            chat_port: 8080,
            broadcast_interval: 5,
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
            8081,
            8080,
        );

        let json = serde_json::to_string(&beacon).unwrap();
        let deserialized: DiscoveryBeacon = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.protocol, DISCOVERY_PROTOCOL);
        assert_eq!(deserialized.version, DISCOVERY_VERSION);
        assert_eq!(deserialized.instance_id, "test-instance-123");
        assert_eq!(deserialized.hostname, "TestHost");
        assert_eq!(deserialized.api_endpoint, "http://192.168.1.100:8081");
        assert_eq!(deserialized.api_port, 8081);
        assert_eq!(deserialized.chat_port, 8080);
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
            8081,
            8080,
            5,
            None,
            None, // peer_model_cache
        );

        assert!(service.start_broadcasting().await.is_ok());
        assert!(service.start_listening().await.is_ok());

        tokio::time::sleep(Duration::from_millis(100)).await;

        let peers = service.get_peers().await;
        assert!(peers.is_empty());

        service.stop();
    }
}
