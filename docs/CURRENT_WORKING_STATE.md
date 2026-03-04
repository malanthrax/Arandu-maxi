# Current Working State

**Last updated:** 2026-03-04 (Rebuild complete: 13:57 PDT)

## Latest UI Verification (2026-03-04) - Label readability in top-right controls

- Added explicit text labels to top-right view toggle buttons for faster recognition:
  - `Local`, `List`, `Remote`, `Fake`
- Rebuilt canonical release artifact after final label patch:
  - `backend/target/release/Arandu.exe`
  - built at `2026-03-04 13:57:14.459494500 -0800`
- Next step for confidence: run the app and visually verify that labels are readable on target display sizes.

## Latest Discovery Policy Update (2026-03-04) - Manual peers only

- Remote model polling is now manual-peer-only in `frontend/desktop.js::pollDiscoveredPeers()`:
  - `get_discovered_peers` is bypassed there and only manually added peers are fetched.
  - The merged-discovery path remains for compatibility, but the discovered side is intentionally empty.
- Backend discovery lifecycle test is disabled as requested:
  - `backend/src/discovery.rs` now has `test_discovery_service_lifecycle` commented out with context.
- Updated remote-empty message to avoid implying UDP discovery is required:
  - "Add manual peers to discover models from other PCs"
- Verification in canonical workspace:
  - `cargo check` (backend) ✅
  - `node --check frontend/desktop.js` ✅

## Latest UI/Cache Update (2026-03-04) - Manual purge action for stale cached peers

- Added a manual purge action in Remote LLMs header beside the duplicate-toggle control.
- New button: `Purge Cached Entries` (`id=remote-model-cache-purge`).
- Backend Tauri command exposed: `purge_discovery_cache` in `backend/src/lib.rs`.
- Command path: `backend/src/discovery.rs::purge_stale_cached_peers()` (manual equivalent of auto purge).
- On success, remote list refreshes automatically and the button is disabled during in-flight call.
- Response includes removed stale-row count and message for user feedback.
- Purge action now works even when discovery service is not running:
  - with no running discovery, it clears cache entries and reports how many were removed.
- Frontend toast now uses backend `message` when present, and always refreshes peers after completion.
- Files changed:
  - `frontend/desktop.js`
  - `frontend/css/desktop.css`
  - `backend/src/discovery.rs`
  - `backend/src/lib.rs`

## Latest UI Spacing Update (2026-03-04) - Top-right and remote button hit areas

- Updated `frontend/css/desktop.css` to improve clickability and spacing in top-right controls:
  - tightened top-right container layout and added explicit width/height targets,
  - enlarged `view-toggle` button hitbox to `40x40` with clearer icon sizing,
  - added dedicated `desktop-launch-last-btn` styling (pill style, larger min-height/min-width),
  - improved remote header action controls (`remote-duplicate-toggle-btn`, `remote-cache-purge-btn`) with larger min-height and wrapping behavior.
- Kept manual peer and cache-purge workflows unchanged.
- Also removed one unmatched closing brace in the same stylesheet, restoring CSS brace balance to zero.
- Verification:
  - `git diff -- frontend/css/desktop.css` shows only intended UI style updates.
  - quick brace-balance check on CSS file returns `0`.

- Runtime verification (canonical workspace):
  - `cargo check --manifest-path backend/Cargo.toml` ✅
  - `cargo test --manifest-path backend/Cargo.toml purge_peers_not_in_endpoints -- --nocapture` ❌ (environment startup exit `0xc0000139 STATUS_ENTRYPOINT_NOT_FOUND`)
  - `cargo test --manifest-path backend/Cargo.toml purge_peers_not_in_endpoints --no-run` ✅

## Latest Backend Fix (2026-03-04) - Cached-offline purge for discovery

- Discovery now supports cross-network model discovery through manual peers (direct host/IP + API/chat ports), even when UDP broadcast cannot traverse routers.
- Added cache purge behavior so stale cached endpoints not present in runtime discovery are removed from persisted cache.
- Discovery merge no longer creates/surfaces cached-offline peers; cached models are used only to backfill runtime-visible peers.
- Scope includes both backend merge/filtering and frontend display normalization:
  - `backend/src/discovery.rs`
  - `backend/src/peer_cache.rs`
  - `frontend/desktop.js`

## Latest Feature (2026-03-03, manual cross-LAN peers)

- Added manual direct IP/host peer support in Discovery options so remote peers can be added without UDP broadcast discovery.
- New UI controls in Discovery settings:
  - host/IP, API port, chat port, optional display name, add/remove list.
- Manual peers are persisted in local storage (`Arandu-manual-discovery-peers`).
- Polling now merges discovered peers with manual peers so both remote model view and discovered instances table include manual entries.
- Discovery polling also runs when discovery is OFF but manual peers exist.
- Rebuilt canonical exe: `backend/target/release/Arandu.exe` (last write observed: `2026-03-03 22:21:49`).

## Latest Startup Fix (2026-03-03, discovery autostart)

- Fixed discovery not actually running on app launch even when settings showed it enabled.
- Root cause: discovery startup used a temporary Tokio runtime in `setup()`, and spawned discovery tasks were tied to that short-lived runtime.
- Fix: switched startup async calls in `setup()` to `tauri::async_runtime::block_on(...)` so discovery/network autostart uses Tauri's persistent runtime.
- Also adjusted discovery status reporting: if no live discovery service exists, status now reports stopped (`enabled: false`) instead of reflecting stale config intent.
- File: `backend/src/lib.rs`
- Rebuilt canonical exe: `backend/target/release/Arandu.exe` (last write observed: `2026-03-03 21:28:03`).

## Latest Backend Fix (2026-03-03, newest) - Remote/options duplicate suppression

- Refined peer culling to remove persistent duplicate cached-offline rows in discovery merge output.
- Frontend list rendering (`frontend/desktop.js`) applies additional normalization and display dedupe for cached-offline entries.
- Files: `backend/src/discovery.rs`, `frontend/desktop.js`
- Coding subagent review applied during patch refinement.
- Rebuilt and copied canonical exe:
  - `backend/target/release/Arandu.exe`
  - last write observed: `2026-03-03 19:37:30`

## Latest UI Fix (2026-03-03, final) - Cached/offline duplicate rows in remote + options views

- Added UI-side discovery peer normalization and stale duplicate suppression for display paths.
- Updated `frontend/desktop.js` to normalize display peers and dedupe cached-offline model rows by host+model key.
- Applied to both remote model list and discovered instances table.
- Latest rebuilt canonical exe timestamp observed: `2026-03-03 20:31:59` at `backend/target/release/Arandu.exe`.

## Latest Backend Fix (2026-03-03) - Remote DB duplicate culling

- Fixed duplicate remote model rows being persisted in peer cache during LLM load.
- Backend-only scope:
  - `backend/src/peer_cache.rs`: centralized duplicate culling on update + on cache load normalization.
  - `backend/src/discovery.rs`: auto-fetch path dedupes before runtime/cache updates.
- Added unit test for cache duplicate culling (`test_update_peer_models_culls_duplicate_models`).
- Coding subagent reviewed change scope prior to rebuild.
- Rebuild succeeded: `backend/target/release/Arandu.exe`.

## Latest Backend Fix (2026-03-03, later) - Cached-offline duplicate suppression

- Added endpoint-scoped suppression so cached-offline peers are culled when online peers exist for the same endpoint.
- Scope: `backend/src/discovery.rs` only (peer merge/culling path + tests).
- Verification: `cargo check`, `cargo test --no-run`, and `cargo tauri build --no-bundle` all passed.
- Artifact confirmed: `backend/target/release/Arandu.exe`.

## Latest UI Bugfix (2026-03-02)

- Fixed model hover hint clipping off-screen at right edge for both local and remote model icons.
- Hint now positions near cursor and clamps to viewport bounds (with left-side flip near right edge).
- Updated file: `frontend/desktop.js`
- Verification: `node --check frontend/desktop.js` passed.

## Latest Build Status (2026-03-03)

- Rebuilt executable successfully with `cargo tauri build --no-bundle`.
- Output artifact: `backend/target/release/Arandu.exe`.
- Build executed in canonical workspace: `H:\Ardanu Fix\Arandu-maxi\backend`.

## Latest Reliability Pass (2026-03-01)

- Completed a slow, step-by-step discovery reliability pass across timing, locks, persistence, and UI polling.
- Added bounded auto-fetch retries and cooldown for peers missing models.
- Prefer beacon `api_port` during peer registration; endpoint parsing now fallback only.
- Improved cache persistence safety with serialized writes + unique temp file strategy.
- Discovery enable now fails fast if API server cannot start (prevents dead advertised endpoints).
- Frontend polling now has in-flight guard and signature reset on disable/re-enable.
- Full notes: `docs/knowledge-base/2026-03-01-discovery-systematic-step-by-step-fix-pass.md`

## Canonical Baseline (LOCKED)

- This workspace state is the canonical working baseline by explicit user instruction.
- Treat current local code in `H:\Ardanu Fix\Arandu-maxi` as source of truth.
- Ignore old merge relevance unless explicitly requested by the user.
- Do not prioritize historical branch divergence over current working behavior.

---

## STOP — READ THIS FIRST NEXT SESSION

Full bug analysis with exact line numbers:
`docs/knowledge-base/2026-03-01-session3-bugs-found-next-steps.md`

---

## Current Branch
- **Branch:** `main`
- **HEAD commit:** `75a7574` (indentation fix in process.rs)
- **Working directory:** `H:\Ardanu Fix\Arandu-maxi`

## Build Status
- **`cargo check`:** ✅ PASSES (verified this session)
- **Release binary:** `backend/target/release/Arandu.exe` (built earlier today, Mar 1)
- **Installers:** `backend/target/release/bundle/` (MSI + NSIS, earlier build)

---

## ✅ COMPLETED: Peer Model Cache Implementation (2026-03-01)

### What Was Implemented
Implemented persistent local cache for peer model data to solve the "random" model display issues:

**New File:** `backend/src/peer_cache.rs` (400+ lines)
- Persistent JSON cache at `~/.Arandu/peer_models_cache.json`
- Tracks model count changes (20 → 4 → 22)
- Only updates UI when count actually changes
- Survives app restarts

**Auto-Fetch on Discovery:**
- When new peer discovered, immediately spawns task to fetch models
- 500ms delay to let peer stabilize
- Saves to cache automatically
- Logs success/failure

**Cache Benefits:**
- No more empty model lists
- Stable counts prevent UI flickering
- Offline resilience (shows cached models)
- Reduced network traffic

**Modified Files:**
- `backend/src/lib.rs` - Added cache to AppState, initialization
- `backend/src/discovery.rs` - Auto-fetch logic, cache integration

### How It Solves the Problems
1. **"Remote models never appear"** → ✅ Auto-fetches immediately on discovery
2. **"Random model display"** → ✅ Cache provides stable data, only updates on real changes
3. **Network hiccups** → ✅ Shows cached models when peer temporarily unreachable

**Documentation:** `docs/knowledge-base/2026-03-01-peer-model-cache-implementation.md`

---

## What Is Broken Right Now

### 1. Remote models never appear (HIGH)
- Peers ARE discovered (beacons received from 10.0.0.106 and 10.0.0.119)
- BUT `peer.models` array is always empty
- Root cause: `fetch_peer_models` is never triggered after a beacon is received
- The backend stores peers without models until `refresh_models()` is explicitly called
- `get_discovered_peers` Tauri command returns peers immediately without fetching models

### 2. Chat window title is broken (CRITICAL JS bug)
- `terminal-manager.js:1670` — `${port}` should be `${apiPort}`
- `terminal-manager.js:1685` — `${port}` should be `${apiPort}`
- Results in "Remote Chat - ModelName (host:undefined)" as window title

### 3. Each beacon logged twice (MEDIUM)
- Cosmetic issue but confusing
- Every `[RECV]` log line appears exactly twice with same timestamp
- Likely: frontend registers `discovery-debug-log` event listener twice

### 4. Discovery port default is wrong in code (MEDIUM)
- `backend/src/models.rs:303` returns `5353` but should be `5352`
- DNS conflicts possible on port 5353

### 5. api_port and chat_port not persisted (MEDIUM)
- `GlobalConfig` in `models.rs` is missing `api_port` and `chat_port` fields
- These reset to 8081/8080 hardcoded defaults on every app restart

---

## What Works
- UDP beacons broadcast/receive — discovery system running
- All three test machines are visible (10.0.0.47, 10.0.0.106, 10.0.0.119)
- Backend API routes registered and compiling: `/api/models/launch`, `/api/models/stop`, `/api/models/active`
- CORS flag on llama-server (--cors added)
- Remote vs local icon detection in frontend
- Local model launch unaffected

---

## Fix Priority Order (Next Session)

1. **Fix `port` → `apiPort` in terminal-manager.js** (2-line JS change, no rebuild)
   - Line 1670: `${port}` → `${apiPort}`
   - Line 1685: `${port}` → `${apiPort}`

2. **Fix models not fetching** (Rust change, needs rebuild)
   - In `discovery.rs` receive loop (~line 338), after inserting peer into HashMap,
     spawn `fetch_peer_models(peer_ip, api_port)` as a background tokio task
   - OR in `lib.rs` `get_discovered_peers` command, call `refresh_models()` first

3. **Fix discovery port default** (1-line Rust change)
   - `models.rs:303` change `5353` → `5352`

4. **Add api_port/chat_port to GlobalConfig** (Rust change)
   - Add fields with serde defaults to `GlobalConfig` in `models.rs`
   - Pass from config into `DiscoveryService::new()` in lib.rs

5. **Fix single-click remote model launch** (1-line JS change)
   - `desktop.js:5014` change `click` → `dblclick`

6. **Investigate duplicate beacon log** (JS or Rust)
   - Check if `discovery-debug-log` listener is added more than once in frontend

---

## Key File Locations

| Purpose | File | Lines |
|---------|------|-------|
| Chat window title bug | `frontend/modules/terminal-manager.js` | 1670, 1685 |
| Remote model list render | `frontend/desktop.js` | 4896–5019 |
| Remote click launch | `frontend/desktop.js` | 5014 |
| API launch/stop/active handlers | `backend/src/openai_proxy.rs` | 399–501 |
| fetch_peer_models + beacon recv | `backend/src/discovery.rs` | 259–367, 417–555 |
| GlobalConfig struct | `backend/src/models.rs` | 7–43 |
| Discovery port default | `backend/src/models.rs` | 303 |
| llama-server --cors flag | `backend/src/process.rs` | 222, 357 |
