# 2026-03-01 - Discovery cache reliability + always-on network server

## Scope

Implemented reliability fixes across discovery/cache polling paths and enabled auto-start behavior for the local network API server at app startup.

## Files changed

- `backend/src/discovery.rs`
  - Added bounded auto-fetch retry cooldown state to runtime peer entries (`last_fetch_attempt`, `fetch_in_progress`)
  - Auto-fetch now triggers for peers that are still model-empty (not only first-seen peers)
  - Prevented overlapping auto-fetch jobs for the same peer while one is in progress
  - Preserved fetched models when beacons arrive
  - Added cache provenance fields to peers (`models_from_cache`, `cache_last_updated`)
  - Marked runtime peers as cache-backed when cache merge fills empty model list

- `backend/src/peer_cache.rs`
  - Kept serialized persist path with `persist_lock`
  - Kept unique temp cache file path on persist
  - Replaced Windows delete-then-rename fallback with atomic replace via `MoveFileExW` (`MOVEFILE_REPLACE_EXISTING | MOVEFILE_WRITE_THROUGH`)

- `backend/src/lib.rs`
  - Added `auto_start_network_server_always()` and invoked it during app setup
  - Discovery enable path now fails fast if the API server cannot be started

- `frontend/desktop.js`
  - Added/kept poll in-flight guard to avoid overlapping `get_discovered_peers` requests
  - Signature-based re-render now includes endpoint fields (`ip_address`, `api_port`, `chat_port`) to avoid stale launch target UI
  - Disabling discovery no longer auto-deactivates network server

## Why

- Reduce discovery regressions where model lists stay empty after initial fetch failures
- Improve cache persistence safety on Windows by avoiding non-atomic delete windows
- Prevent polling overlap and stale peer endpoint rendering
- Support "API just up" behavior by starting network server automatically at startup

## Verification evidence

- Compile check:
  - Command: `cargo check` (run from `backend/`)
  - Result: success

- Unit test attempt:
  - Command: `cargo test test_discovery_service_lifecycle` (run from `backend/`)
  - Result: failed to execute test binary in this environment (`STATUS_ENTRYPOINT_NOT_FOUND`, exit `0xc0000139`)
  - Note: build pipeline still completed successfully via release build below

- Release build + installers:
  - Command: `cargo tauri build` (run from `backend/`)
  - Result artifacts:
    - `backend/target/release/Arandu.exe`
    - `backend/target/release/bundle/nsis/Arandu_0.5.5-1_x64-setup.exe`
    - `backend/target/release/bundle/msi/Arandu_0.5.5-1_x64_en-US.msi`
