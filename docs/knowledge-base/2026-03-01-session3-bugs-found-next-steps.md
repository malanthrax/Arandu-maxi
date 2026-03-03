# Session 3 Analysis - Bugs Found & Next Steps
**Date:** 2026-03-01 (Evening session)
**Status:** ANALYSIS COMPLETE — FIXES PENDING (credits exhausted)
**Compiler:** ✅ `cargo check` passes (exit 0, 11.23s dev profile)

---

## What the User Reported

1. "The error continues and didn't get fixed" — pointing to discovery log showing EVERY beacon arriving TWICE
2. "The models didn't even show up once this time" — remote model list is empty
3. Requested: recode the API handshake + display page for scrolled remote models

---

## BUGS FOUND THIS SESSION (Prioritized)

### BUG 1 — CRITICAL: JavaScript ReferenceError in chat window creation
**File:** `frontend/modules/terminal-manager.js`
**Function:** `openNativeChatForServerSuccess`
**Lines:** 1670, 1685
**Problem:** Variable named `port` is referenced but the function parameter is `apiPort`:
```javascript
// BROKEN — 'port' is undefined, crashes or shows "host:undefined"
openNativeChatForServerSuccess(modelName, host, apiPort, processId, chatHost, chatPort) {
    ...
    this.desktop.createWindow(windowId, `Remote Chat - ${modelName} (${host}:${port})`, ...)
    //                                                                        ^^^^ undefined!
    this.desktop.addTaskbarItem(`Remote Chat - ${modelName} (${host}:${port})`, ...)
    //                                                                  ^^^^ undefined!
}
```
**Fix:** Replace both `${port}` with `${apiPort}` on lines 1670 and 1685.

---

### BUG 2 — HIGH: Remote models never appear in the list
**Files:** `frontend/desktop.js`, `backend/src/discovery.rs`
**Root Cause (most likely):** `fetch_peer_models` is called on the backend poll cycle but the
frontend's `pollDiscoveredPeers` calls `invoke('get_discovered_peers')` which returns peers.
The question is whether the peer objects returned include populated `.models` arrays.

**Suspected problem:** The `models` field on a `DiscoveredPeer` only gets populated AFTER
`fetch_peer_models` is called internally. Looking at `discovery.rs`, `fetch_peer_models`
is only called from `refresh_models()` — but `refresh_models()` is only called if
explicitly triggered. It is NOT called automatically on each poll cycle.

**Evidence:** The `get_discovered_peers` Tauri command (in lib.rs) returns peers as-is from the
`peers` HashMap, which may have empty `models` vecs if `fetch_peer_models` never fired.

**Fix needed:** After a peer is discovered (beacon received), schedule a `fetch_peer_models`
call for that peer. OR: call `refresh_models()` in the polling cycle from lib.rs, or from
within the `start_listening` loop after updating peer state.

---

### BUG 3 — HIGH: Duplicate RECV beacon log entries
**File:** `backend/src/discovery.rs`
**Evidence from log:**
```
5:49:27 PM [RECV] 10.0.0.119 Beacon received from NucBox_EVO-X2 (api: http://10.0.0.119:8081)
5:49:27 PM [RECV] 10.0.0.119 Beacon received from NucBox_EVO-X2 (api: http://10.0.0.119:8081)
```
Same source IP, same message, same millisecond.

**Root cause candidates:**
- A. The sender (10.0.0.119) has TWO physical/virtual network interfaces and broadcasts on BOTH,
  so the receiver gets two copies of the broadcast, both appearing as source 10.0.0.119.
- B. The `DiscoveryService` is being `start()`ed TWICE (e.g. auto-start + user toggle), but the
  second start fails at `start_listening()` (due to the `is_some()` guard), yet the broadcast
  socket from the first start is still alive — causing TWO sends per interval on the sender side.
- C. The JavaScript frontend is registering TWO `discovery-debug-log` event listeners, so one
  backend emit appears twice in the UI.

**How to confirm:** Check if the log deduplication is purely cosmetic (frontend double-listener)
or if it actually affects peer processing (which would mean true duplicate UDP packets).

**Fix for C (quickest):** In frontend, when binding `discovery-debug-log` event listener, check
if listener already bound and skip if so.

---

### BUG 4 — MEDIUM: Discovery port default mismatch
**File:** `backend/src/models.rs:303`
**Code:**
```rust
fn default_discovery_port() -> u16 {
    5353  // ← Wrong! Documentation says 5352
}
```
All knowledge base docs say UDP port 5352. The default in code is 5353 (DNS port — potential
conflict with mDNS/Bonjour on some machines).
**Fix:** Change `5353` → `5352` in `default_discovery_port()`.

---

### BUG 5 — MEDIUM: GlobalConfig missing api_port and chat_port
**File:** `backend/src/models.rs`
**Problem:** `GlobalConfig` struct does not have `api_port` or `chat_port` fields.
The previous session's fix (knowledge base: `2026-03-01-critical-fixes-complete-cors-port.md`)
mentioned adding chat_port to structs, but `GlobalConfig` in `models.rs` only has:
- `discovery_port: u16`
- `discovery_broadcast_interval: u64`
- `discovery_instance_name: String`
- `discovery_instance_id: String`

This means api_port (8081) and chat_port (8080) are **not persisted** in config — they revert
to hardcoded defaults on every app restart. Users who change these ports lose their settings.

**Fix:** Add to `GlobalConfig`:
```rust
#[serde(default = "default_api_port")]
pub api_port: u16,
#[serde(default = "default_chat_port")]
pub chat_port: u16,
```
With `default_api_port() -> u16 { 8081 }` and `default_chat_port() -> u16 { 8080 }`.

---

### BUG 6 — MEDIUM: Single-click triggers model launch in remote list
**File:** `frontend/desktop.js:5014`
**Code:**
```javascript
modelElement.addEventListener('click', () => {
    this.handleRemoteModelClick(model, model.peer);  // fires on SINGLE click
});
```
This fires `openNativeChatForServer` on every single click, including accidental clicks
while scrolling or selecting. Desktop icons use double-click to launch.
**Fix:** Change to `dblclick` event OR add a select-first-then-confirm pattern to match
the rest of the desktop UX.

---

### BUG 7 — LOW: launch request sends wrong field name for server port
**File:** `frontend/modules/terminal-manager.js:1591`
**Code:**
```javascript
body: JSON.stringify({
    model_path: modelPath,
    server_host: host,
    server_port: resolvedChatPort  // ← sends chatPort (8080) as server_port
})
```
The backend `launch_model` handler completely ignores `server_host` and `server_port` from
the request — it always binds to `0.0.0.0`. So this is harmless right now, but the field
naming is semantically wrong and would cause confusion during debugging.

---

## CODE LOCATIONS QUICK REFERENCE

| Bug | File | Line(s) |
|-----|------|---------|
| `port` vs `apiPort` variable | `frontend/modules/terminal-manager.js` | 1670, 1685 |
| Models never fetched on beacon | `backend/src/discovery.rs` | ~295-340 (recv loop) |
| Models refresh trigger missing | `backend/src/lib.rs` | get_discovered_peers cmd |
| Duplicate event listener | `frontend/desktop.js` | discovery-debug-log binding |
| Default port 5353 vs 5352 | `backend/src/models.rs` | 303 |
| Missing api_port/chat_port in config | `backend/src/models.rs` | ~33-43 (GlobalConfig) |
| Single-click launch | `frontend/desktop.js` | 5014 |

---

## CURRENT BUILD STATE

- **Binary:** `backend/target/release/Arandu.exe` (last built Mar 1, earlier session)
- **`cargo check`:** ✅ PASSES (verified this session, exit code 0)
- **`cargo build --release`:** Not run this session

---

## WHAT WORKS NOW

- ✅ Discovery beacons are being sent and received (three machines visible: 10.0.0.47, 10.0.0.106, 10.0.0.119)
- ✅ Backend API routes compile and are registered (`/api/models/launch`, `/api/models/stop`, `/api/models/active`)
- ✅ `stop_model` handler compiles (fixed last session)
- ✅ CORS flag added to llama-server launches (fixed last session)
- ✅ `openNativeChatForServer` is async and calls the launch API
- ✅ Frontend has proper remote vs. local icon detection

## WHAT IS BROKEN

- ❌ Remote models list is always empty (models not fetching after peer discovery)
- ❌ Chat window title shows "undefined" (`port` variable bug)
- ❌ Each beacon appears twice in log (duplicate recv)
- ❌ Discovery port default is wrong (5353 not 5352)
- ❌ api_port and chat_port not saved to config

---

## RECOMMENDED FIX ORDER (next session)

**Step 1 — Fix `port` variable bug** (2-line change, no rebuild needed for frontend)
- `terminal-manager.js:1670` change `${port}` → `${apiPort}`
- `terminal-manager.js:1685` change `${port}` → `${apiPort}`

**Step 2 — Fix models not appearing** (backend change, requires rebuild)
- In `backend/src/lib.rs`, find the `get_discovered_peers` Tauri command
- Ensure `refresh_models()` is called (or models are fetched inline) before returning peers
- OR: in `discovery.rs` receive loop (~line 338), after inserting new peer, spawn a
  `fetch_peer_models` task for that peer immediately.

**Step 3 — Fix default discovery port** (1-line backend change)
- `backend/src/models.rs:303` change `5353` → `5352`

**Step 4 — Add api_port/chat_port to GlobalConfig** (backend, requires rebuild)
- Add fields to `GlobalConfig` struct in `models.rs`
- Add `default_api_port()` and `default_chat_port()` functions
- Update lib.rs to read these from config when initializing DiscoveryService

**Step 5 — Fix single-click remote model launch**
- `desktop.js:5014` change `click` → `dblclick`

**Step 6 — Investigate duplicate beacon log**
- Check if frontend event listener is registered twice
- If single click is confirmed duplicate: no code change needed (cosmetic only)
- If actual duplicate UDP packets: the sender likely has 2 interfaces — filter by instance_id
  (already done in the recv loop — check if the dedup is working for the HashMap insert)

---

## KEY FILE REFERENCE

```
frontend/
  desktop.js              — Remote model rendering, icon interaction handlers
  modules/terminal-manager.js  — openNativeChatForServer, chat window creation

backend/src/
  discovery.rs            — UDP beacon send/recv, fetch_peer_models, DiscoveryService
  openai_proxy.rs         — HTTP routes: /api/models/launch, /stop, /active
  models.rs               — Data types: GlobalConfig, RemoteLaunchRequest/Response, ActiveModel
  lib.rs                  — Tauri commands: get_discovered_peers, enable_discovery
  process.rs              — launch_model_server (adds --cors flag at lines 222, 357)
```

---

## NETWORK TOPOLOGY (Test Environment)

```
10.0.0.47   — Machine 1 (this dev machine / primary server)
10.0.0.106  — NucBoxEvoX3
10.0.0.119  — NucBox_EVO-X2

Ports:
  UDP 5352  — Discovery beacons (NOTE: code defaults to 5353 — mismatch!)
  TCP 8081  — Arandu HTTP API (/api/models/*, /v1/models/*)
  TCP 8080  — llama-server chat UI + OpenAI proxy

172.18.224.1 — WSL virtual interface (ignored, "own beacon" correctly filtered)
```

---

## PREVIOUS SESSION COMMIT HISTORY

```
75a7574  fix: correct indentation in process.rs CORS flag additions
348249b  fix: add missing api_port and chat_port parameters to test_discovery_service_lifecycle
e1fd9ba  fix: add CORS flag and chat port configuration to discovery system
6de44a0  feat: add comprehensive RECV logging to discovery debug
8edce12  fix: stabilize llama custom chat history interactions
```

Current HEAD: `75a7574` (main branch)

---
