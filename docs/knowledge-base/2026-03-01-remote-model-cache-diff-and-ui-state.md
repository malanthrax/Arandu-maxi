# 2026-03-01 - Remote Model Cache Diff + UI State Handling

## What changed

Implemented backend + frontend improvements for remote model cache stability and visibility:

1. **Discovery metadata for cache provenance**
   - Added fields to discovered peers:
     - `models_from_cache: bool`
     - `cache_last_updated: Option<DateTime<Utc>>`
   - File: `backend/src/discovery.rs`

2. **Peer cache diffing instead of count-only checks**
   - Added `PeerModelDelta` with:
     - `added`, `removed`, `updated`, `unchanged`
     - `count_changed`, `any_changed`
   - `update_peer_models(...)` now returns `PeerModelDelta`.
   - File: `backend/src/peer_cache.rs`

3. **Fixed chat port persistence bug in cache writes**
   - `fetch_peer_models(...)` now persists the peer's known `chat_port` from runtime peer entry instead of writing API port into `chat_port`.
   - File: `backend/src/discovery.rs`

4. **Merged runtime + cache peers for remote list API**
   - `get_discovered_peers` now returns `get_peers_with_cached_models()` after refresh attempts.
   - Keeps remote list stable with offline cached peers.
   - File: `backend/src/lib.rs`

5. **Frontend remote display and launch safety updates**
   - Replaced `JSON.stringify` full-object change detection with a stable signature based on peer id/reachability/model identifiers.
   - Added remote row state badge: `Live`, `Cached/Live`, `Cached/Offline`.
   - Blocked launching from cached offline peers.
   - Files:
     - `frontend/desktop.js`
     - `frontend/css/desktop.css`

## Validation run

- `pwd` in canonical workspace confirms: `/h/Ardanu Fix/Arandu-maxi`
- `node --check frontend/desktop.js` (pass)
- `node --check frontend/modules/terminal-manager.js` (pass)
- `cargo check --manifest-path backend/Cargo.toml` (pass; command exceeded default timeout after success output)

## Notes

- Existing cache format remains JSON (`peer_models_cache.json`); no markdown model cache is used.
- Tracker persistence remains SQLite (`tracker.db`) and unchanged in this iteration.
