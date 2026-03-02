use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
#[cfg(windows)]
use std::io;
use std::path::PathBuf;
use std::sync::Arc;
#[cfg(windows)]
use std::os::windows::ffi::OsStrExt;
use tokio::sync::Mutex;
use tracing::{debug, error, info, warn};

use crate::discovery::RemoteModel;

const CACHE_FILE_NAME: &str = "peer_models_cache.json";
const CACHE_VERSION: &str = "1.0";

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PeerModelDelta {
    pub added: usize,
    pub removed: usize,
    pub updated: usize,
    pub unchanged: usize,
    pub count_changed: bool,
    pub any_changed: bool,
}

/// Cached peer data with model information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedPeer {
    pub instance_id: String,
    pub hostname: String,
    pub ip_address: String,
    pub api_port: u16,
    pub chat_port: u16,
    pub api_endpoint: String,
    pub models: Vec<RemoteModel>,
    pub last_updated: DateTime<Utc>,
    pub last_seen: DateTime<Utc>,
    pub is_reachable: bool,
    pub model_count: usize,
}

impl CachedPeer {
    pub fn model_count(&self) -> usize {
        self.models.len()
    }
}

/// Cache structure for persistent storage
#[derive(Debug, Clone, Serialize, Deserialize)]
struct PeerCacheData {
    version: String,
    last_saved: DateTime<Utc>,
    peers: HashMap<String, CachedPeer>, // Key: instance_id
}

impl Default for PeerCacheData {
    fn default() -> Self {
        Self {
            version: CACHE_VERSION.to_string(),
            last_saved: Utc::now(),
            peers: HashMap::new(),
        }
    }
}

/// Manages persistent caching of peer model data
#[derive(Clone)]
pub struct PeerModelCache {
    cache: Arc<Mutex<PeerCacheData>>,
    cache_file_path: PathBuf,
    persist_lock: Arc<Mutex<()>>,
}

impl std::fmt::Debug for PeerModelCache {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PeerModelCache")
            .field("cache_file_path", &self.cache_file_path)
            .finish_non_exhaustive()
    }
}

impl PeerModelCache {
    /// Create a new cache manager with the given app data directory
    pub async fn new(app_data_dir: PathBuf) -> Self {
        if let Err(e) = tokio::fs::create_dir_all(&app_data_dir).await {
            warn!("Failed to ensure app data dir exists {:?}: {}", app_data_dir, e);
        }
        let cache_file_path = app_data_dir.join(CACHE_FILE_NAME);
        info!("Peer model cache file path: {:?}", cache_file_path);
        
        // Load existing cache or create new
        let cache_data = if cache_file_path.exists() {
            match tokio::fs::read_to_string(&cache_file_path).await {
                Ok(content) => {
                    match serde_json::from_str::<PeerCacheData>(&content) {
                        Ok(data) => {
                            info!("Loaded peer model cache with {} peers", data.peers.len());
                            data
                        }
                        Err(e) => {
                            warn!("Failed to parse cache file: {}. Creating new cache.", e);
                            PeerCacheData::default()
                        }
                    }
                }
                Err(e) => {
                    warn!("Failed to read cache file: {}. Creating new cache.", e);
                    PeerCacheData::default()
                }
            }
        } else {
            info!("No existing peer model cache found. Creating new cache.");
            PeerCacheData::default()
        };

        Self {
            cache: Arc::new(Mutex::new(cache_data)),
            cache_file_path,
            persist_lock: Arc::new(Mutex::new(())),
        }
    }

    /// Get all cached peers
    pub async fn get_all_peers(&self) -> Vec<CachedPeer> {
        let cache = self.cache.lock().await;
        cache.peers.values().cloned().collect()
    }

    /// Get a specific peer by instance_id
    pub async fn get_peer(&self, instance_id: &str) -> Option<CachedPeer> {
        let cache = self.cache.lock().await;
        cache.peers.get(instance_id).cloned()
    }

    /// Get cached models for a specific peer
    pub async fn get_peer_models(&self, instance_id: &str) -> Vec<RemoteModel> {
        let cache = self.cache.lock().await;
        cache
            .peers
            .get(instance_id)
            .map(|p| p.models.clone())
            .unwrap_or_default()
    }

    /// Update or insert peer data with new models
    /// Returns model delta details for UI and logging
    pub async fn update_peer_models(
        &self,
        instance_id: String,
        hostname: String,
        ip_address: String,
        api_port: u16,
        chat_port: u16,
        api_endpoint: String,
        new_models: Vec<RemoteModel>,
        is_reachable: bool,
    ) -> PeerModelDelta {
        let mut cache = self.cache.lock().await;
        let old_models = cache
            .peers
            .get(&instance_id)
            .map(|p| p.models.clone())
            .unwrap_or_default();
        let old_count = old_models.len();
        
        let new_count = new_models.len();
        let count_changed = old_count != new_count;

        let mut old_index: HashMap<String, RemoteModel> = HashMap::new();
        for model in old_models {
            old_index.insert(model.id.clone(), model);
        }

        let mut added = 0usize;
        let mut updated = 0usize;
        let mut unchanged = 0usize;

        for model in &new_models {
            match old_index.get(&model.id) {
                None => added += 1,
                Some(existing) => {
                    if existing.path == model.path
                        && existing.size_gb == model.size_gb
                        && existing.quantization == model.quantization
                        && existing.architecture == model.architecture
                        && existing.date == model.date
                    {
                        unchanged += 1;
                    } else {
                        updated += 1;
                    }
                }
            }
        }

        let mut new_ids: HashMap<String, ()> = HashMap::new();
        for model in &new_models {
            new_ids.insert(model.id.clone(), ());
        }
        let removed = old_index
            .keys()
            .filter(|id| !new_ids.contains_key(*id))
            .count();

        let any_changed = added > 0 || removed > 0 || updated > 0;

        let cached_peer = CachedPeer {
            instance_id: instance_id.clone(),
            hostname,
            ip_address,
            api_port,
            chat_port,
            api_endpoint,
            models: new_models,
            last_updated: Utc::now(),
            last_seen: Utc::now(),
            is_reachable,
            model_count: new_count,
        };

        debug!(
            "Updating peer {}: {} -> {} models (added: {}, removed: {}, updated: {})",
            instance_id, old_count, new_count, added, removed, updated
        );

        cache.peers.insert(instance_id, cached_peer);
        cache.last_saved = Utc::now();

        // Persist to disk
        drop(cache); // Release lock before async file operation
        self.persist().await;

        PeerModelDelta {
            added,
            removed,
            updated,
            unchanged,
            count_changed,
            any_changed,
        }
    }

    /// Update peer reachability status without changing models
    pub async fn update_peer_status(&self, instance_id: &str, is_reachable: bool) {
        let mut cache = self.cache.lock().await;
        
        if let Some(peer) = cache.peers.get_mut(instance_id) {
            peer.is_reachable = is_reachable;
            peer.last_seen = Utc::now();
            debug!("Updated peer {} reachability: {}", instance_id, is_reachable);
        }
        
        drop(cache);
        self.persist().await;
    }

    /// Remove a peer from the cache
    pub async fn remove_peer(&self, instance_id: &str) -> bool {
        let mut cache = self.cache.lock().await;
        let removed = cache.peers.remove(instance_id).is_some();
        
        if removed {
            info!("Removed peer {} from cache", instance_id);
            cache.last_saved = Utc::now();
        }
        
        drop(cache);
        self.persist().await;
        
        removed
    }

    /// Clear all peers from cache
    pub async fn clear(&self) {
        let mut cache = self.cache.lock().await;
        let count = cache.peers.len();
        cache.peers.clear();
        cache.last_saved = Utc::now();
        info!("Cleared {} peers from cache", count);
        
        drop(cache);
        self.persist().await;
    }

    /// Get cache statistics
    pub async fn get_stats(&self) -> (usize, DateTime<Utc>) {
        let cache = self.cache.lock().await;
        (cache.peers.len(), cache.last_saved)
    }

    /// Persist cache to disk
    async fn persist(&self) {
        let _persist_guard = self.persist_lock.lock().await;
        let cache = self.cache.lock().await;
        
        let json = match serde_json::to_string_pretty(&*cache) {
            Ok(j) => j,
            Err(e) => {
                error!("Failed to serialize cache: {}", e);
                return;
            }
        };
        
        drop(cache); // Release lock before file operation

        let temp_path = self.cache_file_path.with_extension(format!(
            "json.tmp.{}.{}",
            std::process::id(),
            Utc::now().timestamp_nanos_opt().unwrap_or_default()
        ));
        if let Err(e) = tokio::fs::write(&temp_path, json).await {
            error!("Failed to write cache temp file: {}", e);
            return;
        }

        #[cfg(windows)]
        {
            if let Err(e) = replace_file_atomic_windows(&temp_path, &self.cache_file_path).await {
                error!("Failed to persist cache file atomically on Windows: {}", e);
                let _ = tokio::fs::remove_file(&temp_path).await;
                return;
            }
        }

        #[cfg(not(windows))]
        {
            if let Err(e) = tokio::fs::rename(&temp_path, &self.cache_file_path).await {
                error!("Failed to persist cache file: {}", e);
                let _ = tokio::fs::remove_file(&temp_path).await;
                return;
            }
        }

        debug!("Peer model cache persisted to disk");
    }

    /// Mark peers as stale if not seen for longer than timeout
    pub async fn mark_stale_peers(&self, timeout_secs: i64) -> Vec<String> {
        let mut cache = self.cache.lock().await;
        let now = Utc::now();
        let mut stale_ids = Vec::new();

        for (id, peer) in cache.peers.iter_mut() {
            let elapsed = now.signed_duration_since(peer.last_seen);
            if elapsed.num_seconds() > timeout_secs && peer.is_reachable {
                peer.is_reachable = false;
                stale_ids.push(id.clone());
                info!("Marked peer {} as stale (last seen {}s ago)", id, elapsed.num_seconds());
            }
        }

        if !stale_ids.is_empty() {
            cache.last_saved = Utc::now();
            drop(cache);
            self.persist().await;
        }

        stale_ids
    }
}

#[cfg(windows)]
async fn replace_file_atomic_windows(from: &PathBuf, to: &PathBuf) -> io::Result<()> {
    let from_clone = from.clone();
    let to_clone = to.clone();

    tokio::task::spawn_blocking(move || {
        const MOVEFILE_REPLACE_EXISTING: u32 = 0x00000001;
        const MOVEFILE_WRITE_THROUGH: u32 = 0x00000008;

        #[link(name = "kernel32")]
        extern "system" {
            fn MoveFileExW(
                lpExistingFileName: *const u16,
                lpNewFileName: *const u16,
                dwFlags: u32,
            ) -> i32;
        }

        fn to_wide(path: &std::path::Path) -> Vec<u16> {
            path.as_os_str()
                .encode_wide()
                .chain(std::iter::once(0))
                .collect()
        }

        let from_wide = to_wide(from_clone.as_path());
        let to_wide = to_wide(to_clone.as_path());

        // Atomic replace on Windows when source/target are on same volume.
        let rc = unsafe {
            MoveFileExW(
                from_wide.as_ptr(),
                to_wide.as_ptr(),
                MOVEFILE_REPLACE_EXISTING | MOVEFILE_WRITE_THROUGH,
            )
        };

        if rc == 0 {
            Err(io::Error::last_os_error())
        } else {
            Ok(())
        }
    })
    .await
    .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("Join error: {}", e)))?
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_remote_model(id: &str) -> RemoteModel {
        RemoteModel {
            id: id.to_string(),
            name: format!("model_{}", id),
            object: "model".to_string(),
            owned_by: "test".to_string(),
            instance_id: "test-instance".to_string(),
            instance_hostname: "test-host".to_string(),
            api_endpoint: "http://test:8081".to_string(),
            size_gb: Some(4.5),
            quantization: Some("Q4_K_M".to_string()),
            architecture: Some("llama".to_string()),
            date: Some(1234567890),
            path: Some("/path/to/model.gguf".to_string()),
        }
    }

    fn create_test_temp_dir() -> std::path::PathBuf {
        let temp_dir = std::env::temp_dir().join(format!("arandu_test_{}", std::process::id()));
        let _ = std::fs::create_dir_all(&temp_dir);
        temp_dir
    }

    #[tokio::test]
    async fn test_cache_update_tracks_count_changes() {
        let temp_dir = create_test_temp_dir();
        let cache = PeerModelCache::new(temp_dir.clone()).await;

        let models_20: Vec<RemoteModel> = (0..20)
            .map(|i| create_test_remote_model(&format!("model_{}", i)))
            .collect();

        // First update with 20 models
        let delta = cache
            .update_peer_models(
                "peer-1".to_string(),
                "host-1".to_string(),
                "10.0.0.1".to_string(),
                8081,
                8080,
                "http://10.0.0.1:8081".to_string(),
                models_20.clone(),
                true,
            )
            .await;

        assert!(delta.count_changed, "First update should report changed (0 -> 20)");

        // Update with same 20 models - should not report changed
        let delta = cache
            .update_peer_models(
                "peer-1".to_string(),
                "host-1".to_string(),
                "10.0.0.1".to_string(),
                8081,
                8080,
                "http://10.0.0.1:8081".to_string(),
                models_20.clone(),
                true,
            )
            .await;

        assert!(!delta.count_changed, "Update with same count should not report changed");
        assert!(!delta.any_changed, "No field changes should be detected");

        // Update with 22 models - should report changed
        let models_22: Vec<RemoteModel> = (0..22)
            .map(|i| create_test_remote_model(&format!("model_{}", i)))
            .collect();

        let delta = cache
            .update_peer_models(
                "peer-1".to_string(),
                "host-1".to_string(),
                "10.0.0.1".to_string(),
                8081,
                8080,
                "http://10.0.0.1:8081".to_string(),
                models_22,
                true,
            )
            .await;

        assert!(delta.count_changed, "Update with different count should report changed (20 -> 22)");

        // Update with 4 models - should report changed
        let models_4: Vec<RemoteModel> = (0..4)
            .map(|i| create_test_remote_model(&format!("model_{}", i)))
            .collect();

        let delta = cache
            .update_peer_models(
                "peer-1".to_string(),
                "host-1".to_string(),
                "10.0.0.1".to_string(),
                8081,
                8080,
                "http://10.0.0.1:8081".to_string(),
                models_4,
                true,
            )
            .await;

        assert!(delta.count_changed, "Update with different count should report changed (22 -> 4)");
    }

    #[tokio::test]
    async fn test_cache_persistence() {
        let temp_dir = create_test_temp_dir();
        let cache_path = temp_dir.clone();

        // Create cache and add data
        {
            let cache = PeerModelCache::new(cache_path.clone()).await;
            let models = vec![create_test_remote_model("test")];
            
            cache
                .update_peer_models(
                    "peer-1".to_string(),
                    "host-1".to_string(),
                    "10.0.0.1".to_string(),
                    8081,
                    8080,
                    "http://10.0.0.1:8081".to_string(),
                    models,
                    true,
                )
                .await;
        }

        // Recreate cache and verify data persisted
        {
            let cache = PeerModelCache::new(cache_path).await;
            let peer = cache.get_peer("peer-1").await;
            assert!(peer.is_some(), "Peer should be loaded from cache");
            assert_eq!(peer.unwrap().hostname, "host-1");
        }
    }
}
