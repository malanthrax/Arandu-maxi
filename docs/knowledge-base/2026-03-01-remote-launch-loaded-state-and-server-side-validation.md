# 2026-03-01 - Remote launch hardening + loaded-state indicator

## Goals completed

1. Ensure remote launch request carries valid launch target and server can launch with normal local-launch semantics.
2. Add detection/handling for "model already loaded" on remote server.
3. Improve remote display state so users can see loaded models (`L` badge).
4. Keep remote display resilient when peers are discovered before model lists fully hydrate.

## Implementation

### Backend: launch validation + loaded model reuse metadata

- File: `backend/src/openai_proxy.rs`

Changes:
- Added path normalization helper for robust comparison (`normalize_model_path`).
- In `/api/models/launch`:
  - Enumerates current server model library via scanner.
  - Validates requested path exists in configured model library.
  - Validates file readability before launch.
  - Checks `active_models` + `running_processes` for already-running same model and returns success without relaunch when already loaded.
  - Launch path still goes through `process::launch_model_server(..., Some("0.0.0.0"))`, i.e. same backend launch path used for local process management (equivalent server-side launch behavior, not shell command injection).
  - Stores launched model in `active_models` with process id from actual launch result.
- In `/api/models/active`:
  - Reconciles statuses against `running_processes` (`Starting`/`Ready`).
  - Fills missing host/port from process info.
  - Removes stale/terminated entries.

### Frontend: active-model reuse + loaded badge

- File: `frontend/modules/terminal-manager.js`

Changes:
- Before calling `/api/models/launch`, requests `/api/models/active` and reuses an already-loaded model if available (same normalized model path).
- Uses launch response `server_host/server_port` for readiness polling and iframe URL (handles dynamic port selection correctly).
- After launch/reuse/stop, triggers refresh of remote loaded-state map via desktop manager.

- File: `frontend/desktop.js`

Changes:
- Added periodic bounded fetch of `/api/models/active` per reachable peer (`refreshRemoteActiveModels`) with cooldown/in-flight guards.
- Added remote loaded-state map keyed by `peer_ip:api_port` and normalized model path.
- Remote items now carry `peer_model_loaded` and render an `L` badge when loaded.
- Added empty-model recovery UX:
  - If peers are discovered but model list is empty, auto-trigger bounded `refresh_remote_models`.
  - Shows explicit waiting state and manual "Refresh Remote Models" button.

- File: `frontend/css/desktop.css`

Changes:
- Added `.remote-loaded-badge` styling (gold circular `L` badge).

## Verification

- `node --check frontend/desktop.js` ✅
- `node --check frontend/modules/terminal-manager.js` ✅
- `cargo check` (backend) ✅

## Build artifacts

Canonical `backend/target/release/Arandu.exe` was locked by a running process in this environment.
To complete release packaging reliably, build was executed with alternate target dir:

- Command:
  - `CARGO_TARGET_DIR="H:\Ardanu Fix\Arandu-maxi\backend\target_fresh" cargo tauri build`

- Artifacts generated:
  - `backend/target_fresh/release/Arandu.exe`
  - `backend/target_fresh/release/bundle/msi/Arandu_0.5.5-1_x64_en-US.msi`
  - `backend/target_fresh/release/bundle/nsis/Arandu_0.5.5-1_x64-setup.exe`
