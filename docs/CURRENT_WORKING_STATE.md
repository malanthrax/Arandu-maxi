# Current Working State

**Last updated:** 2026-03-01 (Late session — systematic discovery/cache fix pass)

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
