# 2026-03-01 - Discovery send rate and peer cache root-cause fix

## User-reported symptoms

- Continuous remote API fetch sends (frequent `GET /v1/models/arandu`) during discovery polling.
- Peer models were not consistently appearing as cached on client machines even though peers were discovered/pinged.

## Root causes identified

1. `get_discovered_peers` performed model-fetch attempts during every frontend poll cycle.
   - Polling in frontend runs repeatedly; backend was re-triggering peer fetch logic from this command.
2. Discovery beacon broadcast interval was hardcoded in service (`5s`) and ignored config value.
3. Auto-fetch for newly discovered peers wrote to persistent cache, but did not update runtime in-memory peer models immediately.
   - This kept some peers in empty-model state until later code paths fetched again.
4. Peer cache persistence path did not proactively ensure app data directory exists before write.

## Fixes applied

- `backend/src/lib.rs`
  - `get_discovered_peers` now returns merged runtime+cache peers directly (`get_peers_with_cached_models`) and no longer fetches models on each UI poll.

- `backend/src/discovery.rs`
  - Added configurable `broadcast_interval_secs` to `DiscoveryService` (constructor + struct field).
  - Replaced hardcoded broadcast ticker interval with configured value.
  - Auto-fetch path for new peers now updates runtime peer entry (`entry.peer.models`) immediately before cache write.

- `backend/src/peer_cache.rs`
  - Added `tokio::fs::create_dir_all(app_data_dir)` in cache initialization to avoid persistence failures when directory is missing.

## Verification

- `cargo check --manifest-path backend/Cargo.toml` passes.

## Expected behavior after fix

- Discovery polling no longer causes continuous model-fetch API traffic.
- New peers should populate model list once auto-fetch succeeds, and cache should persist to `peer_models_cache.json` reliably.
- Broadcast interval now respects configured `discovery_broadcast_interval` value.
