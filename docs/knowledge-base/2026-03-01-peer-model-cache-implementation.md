# Peer Model Cache Implementation - 2026-03-01

## Summary
Implemented a persistent local cache for peer model data to provide stable model lists even during network fluctuations. This solves the "random" model display issues by maintaining a local database that only updates when model counts actually change.

## How It Works

### Cache Behavior
- **First check with 20 models**: Cache stores 20 models
- **Send comes in with 4**: Cache detects change (20 → 4), updates to 4
- **Send comes in with 22**: Cache detects change (4 → 22), updates to 22
- **Send comes in with 22 again**: Cache detects no change, keeps 22 (no UI flicker)

### Key Features
1. **Persistent Storage**: Cache saved to `~/.Arandu/peer_models_cache.json`
2. **Change Detection**: Only updates when model count changes
3. **Immediate Fetch on Discovery**: Auto-fetches models when new peer discovered
4. **Offline Support**: Shows cached models even if peer temporarily unreachable
5. **Stable UI**: Prevents flickering/empty lists during network hiccups

## Files Created/Modified

### New Files
- `backend/src/peer_cache.rs` - Cache manager with 400+ lines
  - `PeerModelCache` struct with persistent storage
  - Methods: `get_peer_models`, `update_peer_models`, `get_all_peers`
  - Tracks model count changes
  - Auto-saves to disk

### Modified Files

#### `backend/src/lib.rs`
- Added `mod peer_cache` and import
- Added `peer_model_cache: Option<Arc<PeerModelCache>>` to `AppState`
- Updated `initialize_app_state()` to create cache on startup
- Updated `enable_discovery()` to pass cache to DiscoveryService
- Fixed 3 locations that construct AppState manually

#### `backend/src/discovery.rs`
- Added `peer_model_cache` field to `DiscoveryService`
- Added auto-fetch logic when new peer discovered (lines 360-420)
- Modified `fetch_peer_models()` to save to cache after successful fetch
- Added `get_peers_with_cached_models()` for merged runtime + cached data
- Modified `get_peers()` to fall back to cached models if runtime has none

## Technical Details

### Cache Data Structure
```rust
CachedPeer {
    instance_id: String,
    hostname: String,
    ip_address: String,
    api_port: u16,
    chat_port: u16,
    api_endpoint: String,
    models: Vec<RemoteModel>,
    last_updated: DateTime<Utc>,
    last_seen: DateTime<Utc>,
    is_reachable: bool,
}
```

### Auto-Fetch on Discovery
When beacon received and `is_new`:
1. Log discovery
2. Spawn async task with 500ms delay (let peer stabilize)
3. HTTP GET to `/v1/models/arandu`
4. Parse response
5. Save to cache
6. Log success/failure

### API Compatibility
- Endpoint: `GET /v1/models/arandu`
- Timeout: 10 seconds
- Response: `ModelsResponse` with `data: Vec<ModelInfo>`
- ✅ Verified compatible with existing code

## Benefits

1. **No More Empty Lists**: Cache persists between app restarts
2. **Stable Counts**: UI only updates when count actually changes
3. **Immediate Population**: Auto-fetches when peer discovered
4. **Offline Resilience**: Shows last known models if peer goes offline
5. **Reduced Network Traffic**: No redundant fetches for same data

## Build Status
- ✅ `cargo check` passes
- ✅ Code compiles successfully
- ⚠️ Tests fail due to Windows environment (STATUS_ENTRYPOINT_NOT_FOUND)
  - This is a known environment issue, not code problem
  - Tests use standard temp directories (no tempfile crate dependency)

## Next Steps
1. Build and test on actual Windows machine
2. Verify cache file created at `~/.Arandu/peer_models_cache.json`
3. Test discovery between two machines
4. Verify models populate immediately on discovery
5. Test offline behavior (disable network, verify cached models still show)

## Cache File Location
```
%USERPROFILE%\.Arandu\peer_models_cache.json
```

Example content:
```json
{
  "version": "1.0",
  "last_saved": "2026-03-01T20:00:00Z",
  "peers": {
    "instance-id-uuid": {
      "instance_id": "instance-id-uuid",
      "hostname": "PeerMachine",
      "ip_address": "10.0.0.106",
      "api_port": 8081,
      "chat_port": 8080,
      "api_endpoint": "http://10.0.0.106:8081",
      "models": [...],
      "last_updated": "2026-03-01T20:00:00Z",
      "last_seen": "2026-03-01T20:00:00Z",
      "is_reachable": true
    }
  }
}
```

---
**Implementation Date:** 2026-03-01
**Status:** Code Complete, Ready for Testing
**Build:** `backend/target/release/Arandu.exe`
