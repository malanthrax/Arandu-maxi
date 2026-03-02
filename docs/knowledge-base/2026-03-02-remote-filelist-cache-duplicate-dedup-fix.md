# 2026-03-02 - Remote filelist cache duplicate dedup fix

## Problem

Remote model list duplicated the same files after repeated app opens/loads.

Observed behavior: each subsequent load could add another copy of the same remote model rows.

## Root cause

- Peer cache keys were instance-id based; if a server reused same endpoint but rotated instance identity, stale cached identities could coexist.
- Runtime+cache merge in discovery merged by `instance_id` only, so endpoint-equivalent stale cache entries were still surfaced.
- Frontend flatten pass did not guard against duplicate model rows for same peer endpoint/path.

## Fixes

### Backend

- `backend/src/peer_cache.rs`
  - In `update_peer_models(...)`, remove stale cache entries with same `ip_address + api_port` but different `instance_id` before insert.

- `backend/src/discovery.rs`
  - Added remote model dedupe helper keyed by `path` fallback to `id`.
  - Applied dedupe on fetched model payload before storing.
  - In `get_peers_with_cached_models()`, skip cached peers when runtime already has same endpoint under a different instance id.
  - Applied dedupe to cached model vectors when merging.

### Frontend

- `frontend/desktop.js`
  - In `renderRemoteModelsList()`, added row-level dedupe keyed by `peer_ip:api_port + model.path` (fallback to id/name).

## Verification

- `node --check frontend/desktop.js` passed.
- `cargo check` in `backend/` passed.
