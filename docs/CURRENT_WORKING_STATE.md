# Current Working State

**Last updated:** 2026-03-08

## IK Installer UX Clarity Update (2026-03-08)

- Replaced the old source-type `confirm()` wording with explicit source choices for both IK install actions:
  - `ZIP file`
  - `Extracted folder`
  - `Cancel`
- If `ModalDialog` is available, source choice uses a clean modal; otherwise fallback prompt accepts `ZIP` / `FOLDER` / `CANCEL`.
- Success notifications now include destination path (`install_path`) for:
  - main `ik_llama.cpp` install
  - `ik CUDA DLL` install

## IK Readme + Gold Star Indicator (2026-03-08)

- Added top-right `Readme` button (`view-ik-readme-btn`) near existing model view controls.
- Clicking it opens `ik_llama.cpp Supported Families` window with the full IK-supported families list.
- Added reusable IK family constants + normalized matching helpers in `frontend/desktop.js`.
- Local list/card titles and remote model titles now show gold `★` when model name/path matches IK-supported families.
- Saved same canonical list in repo root file: `ikllama.cpp supported models.md`.

## ik_llama.cpp Dual ZIP Install Update (2026-03-08)

- Added separate IK install actions in Llama.cpp manager top controls:
  - `ik_llama.cpp` for main archive install
  - `ik CUDA DLL` for CUDA DLL package install
- CPU ik installs remain one-step and do not require DLL package.
- CUDA ik main installs now immediately offer DLL ZIP installation.
- Added backend install command to unpack DLL ZIP and place discovered `.dll` files into the selected ik CUDA backend folder.
- Manual DLL flow resolves target path from last install/active CUDA backend and prompts for selection when multiple ik CUDA backends exist.

## ik_llama.cpp Loose Folder + Workspace Update (2026-03-08)

- IK installer source selection now supports both:
  - ZIP file source
  - extracted folder source
- This applies to both main ik install and CUDA DLL package install.
- Added managed workspace root for IK install operations:
  - `<executable_folder>/versions/ik_llama.cpp/_installer_workspace`
  - subfolders: `main_drop`, `dll_drop`, `temp`
- Frontend installer now prompts whether to pick folder source or ZIP source before each IK install action.
- Llama.cpp manager now includes visible `IK Installer Workspace` panel with copy buttons for:
  - `main_drop`
  - `dll_drop`

## IK Installed Readout (2026-03-08)

- Added explicit `ik` installed-status chip in Release Manager top controls (`ik-installed-status`).
- Readout now shows:
  - `ik: not installed` when none exist,
  - installed count when present,
  - active backend when one is active (e.g., `active CUDA`).

## H-drive Runtime Path Policy (2026-03-08)

- Default runtime base now prefers `H:\Ardanu Fix\Arandu-maxi\.Arandu` in this development phase.
- `load_settings()` migration now remaps legacy default-style `.Arandu` paths loaded from C/home base to H base for model/executable directories and active backend folder.
- Migration is conservative and does not rewrite arbitrary custom paths.

## IK Installer UX + Installed List Recovery (2026-03-08)

- IK installer interaction is now ZIP-only for both actions:
  - `ik_llama.cpp` (main archive)
  - `ik CUDA DLL` (DLL archive)
- Removed extra install-time source-type prompts from IK flow.
- Main CUDA install now gives a follow-up guidance toast instead of a blocking confirmation dialog.
- Backend installed-version listing now scans both configured versions root and legacy default versions root, so normal llama.cpp versions and IK versions are visible together in Installed list.
- Backend install commands enforce ZIP-only source inputs (no directory-source fallback).

## MCP Wait-message + Benchmark Coloring (2026-03-08)

- Added MCP mode system guidance to prefer immediate `tool_calls` and avoid "please wait/queue" placeholder replies.
- Benchmark Log enhanced with TPS tracking column.
- Benchmark best/worst highlighting:
  - TTFT best (lowest) = green, worst (highest) = red
  - TPS best (highest) = green, worst (lowest) = red

## GLM/IK Crash and Template-order Fixes (2026-03-08)

- Added outgoing message normalization to keep all `system` messages at the beginning of `/v1/chat/completions` payloads.
- Added GLM-family payload compatibility mode to avoid unstable sampler combinations in IK + GLM runs.
- Added terminal readiness guard so health-check success does not mark running state after launch failure/usage-exit.
- Added pre-launch IK arg sanitizer in backend launch path that removes unsupported custom flags based on `llama-server --help` before model start.

## Mandatory Process Policy (2026-03-07)

- Scratchpad-first: new requests are recorded in `scratchpad.md` before implementation.
- Completion protocol per item: update docs, update nowledge memory, update supermemory memory (when available), then remove item from `scratchpad.md`.
- Coding protocol: use coding subagent -> review subagent -> resolve findings -> final requirement verification.

## Current Active Workflow (2026-03-07)

- MCP tools in chat are now refreshed at context request-time from backend (`list_mcp_tools`) instead of relying only on cached connection tools.
- Chat shows explicit MCP visibility UI:
  - Wrench count chip beside active model (callable tools seen by model).
  - Clickable MCP tool panel listing exact `Connection :: ToolName` entries sent to model `tools[]`.
- MCP status line includes refresh failure telemetry (`... / X refresh failed`) to separate "no tools" from "refresh failed".
- Chat Model Options now include runtime `MCP Tool Loops` (default `6`, range `1..20`); MCP execution loop cap now reads this setting with defensive clamp enforcement.
- Chat history reliability hardening complete for repeated operations:
  - delete/load/new actions share centralized lock lifecycle,
  - search input disabled while busy,
  - pending debounce cleared at lock-start and callback guarded,
  - stale-lock watchdog avoids releasing lock while requests are still pending.

## Llama.cpp Local ZIP Install + Model Warning (2026-03-07)

- Added `ik_llama.cpp` button in Llama.cpp Release Manager Installed controls.
- Added backend zip picker command: `pick_llamacpp_zip_file`.
- Added backend local install command: `install_local_llamacpp_zip`.
  - Target path: `<executable_folder>/versions/ik_llama.cpp/<detected-backend>/...`
  - Backend detection by zip filename: `cuda/rocm/vulkan/opencl/metal/cpu`.
  - Safe extraction enforced using zip enclosed paths (path traversal blocked).
  - Installer locates nested `llama-server(.exe)` root and flattens it into final backend folder.
- Installed list behavior remains standard (Activate/Delete/Active badge/etc.) because install path matches existing version scan format.
- Added persistent desktop reminder:
  - `Do not use quantized models from Unsloth that have _XL in their name.`

## System Prompt Override Update (2026-03-07)

- Added top-right light-blue `System Prompt Override` button in desktop controls.
- Added override manager window to save named system prompts and select active prompt.
- Dropdown always includes `Default` as first option; selecting it injects nothing and removes active launch override.
- Persisted prompt entries + selected prompt in localStorage.
- Wired parent->iframe sync for live override updates.
- Effective precedence in chat request assembly:
  - typed chat Model Options system prompt
  - selected global system prompt override
  - default/no injection
- Best-effort launch path integration now applies/removes `--system-prompt` in terminal-manager launch/restart arg handling.
- Follow-up UX update: added `New` button in override manager action row (`Clear Editor` | `New` | `Save Prompt`) for quick creation of additional prompts.
  - `New` sets selected option to `Default`, clears editor fields, and focuses prompt name input.

## Date-Awareness Hardening (2026-03-07)

- Added always-on current date/time system prompt injection in main chat request assembly.
- Source of truth is local system clock (Tauri webview runtime `Date` from host environment).
- Injected data includes local date, local time, timezone + UTC offset, and UTC ISO timestamp.
- Preserved existing optional system prompt precedence (`typed > global override > none`).

## MCP stdio Window Suppression (2026-03-07)

- Updated backend stdio MCP process spawning to avoid popping visible Windows console windows.
- Added hidden-window creation flags for all MCP stdio spawn branches:
  - direct configured command,
  - cmd-shell fallback for command-style launchers,
  - `.cmd` shim fallback path.
- Verification: `cargo check --manifest-path backend/Cargo.toml` passed.

## MCP Log Panel (2026-03-07)

- Added floating `MCP Log` toggle in `frontend/llama-custom/index.html` directly under `Benchmark Log`.
- Added MCP log panel with `Clear` + `Close` controls and benchmark/debug-coherent styling.
- MCP tool-call loop now logs:
  - outbound model -> MCP calls (mapped connection/tool + argument snapshot),
  - inbound MCP -> model responses (success/failure + result snippet),
  - timestamp per entry.
- Log is bounded (150 entries), persisted in localStorage (`aranduMcpLogV1`), and rendered newest-first.
- Follow-up fix: log now includes LLM MCP-loop request lifecycle entries (`__chat_completion__`) so HTTP 500 / pre-tool failures are visible instead of appearing empty.

## Multimodal Image Upload 500 Fix (2026-03-07)

- Added `buildChatCompletionPayload()` in chat path to enforce compatibility-safe payloads when images are present.
- For image-attached requests:
  - force non-stream completion mode,
  - send core completion fields only,
  - omit advanced sampler/reasoning/speculative fields.
- For text-only requests, existing full parameter payload remains intact.

## Apriel Template Options (2026-03-07)

- Added Apriel template selections to model tile `Chat Template` dropdown in `frontend/model-settings-config.json`:
  - `Apriel 1.6 (Fixed)` (`Apriel-1.6-15b-Thinker-fixed`)
  - `Apriel (Unsloth 1.5)` (`unsloth-Apriel-1.5`)
- Upstream template availability cross-checked from llama.cpp template inventory.

### Verification (System Prompt Override)

- `node --check frontend/desktop.js` ✅
- `node --check frontend/modules/terminal-manager.js` ✅
- Inline script parse sanity for `frontend/llama-custom/index.html` (Node VM compile) ✅ (`inline_scripts_ok 1`)

## Build + Verification (2026-03-07)

- `node --check frontend/modules/terminal-manager.js` ✅
- Inline script parse check for `frontend/llama-custom/index.html` (Node VM compile) ✅
- `cargo check --manifest-path backend/Cargo.toml` ✅
- `cargo tauri build --no-bundle` ✅
- Artifact: `backend/target/release/Arandu.exe`

## Security Hardening Addendum (2026-03-07)

- `frontend/llama-custom/index.html` assistant model-output rendering no longer inserts untrusted model text with raw `innerHTML`.
- Added centralized safe rendering helpers that use text nodes/DOM APIs and controlled `<br>` insertion for line breaks.
- Streaming and non-streaming assistant output paths now share this safe rendering approach.
- Assistant stats rendering moved to DOM construction (no string-based HTML concatenation for model text paths).

### Verification (addendum)

- Inline script parse sanity (Node VM compile of extracted inline script) ✅
  - Result: `inline_scripts_ok 1`

## Current Active Workflow (2026-03-06)

- Primary objective now is MCP enablement in chat inference flow: models must not only read MCP connection metadata, but be able to actually use MCP-connected tooling.
- Confirmed current baseline:
  - local model list tile update to show size in normal list view is already done,
  - MCP panel fixes were already completed in prior sessions,
  - chat currently receives/injects MCP context metadata via parent bridge.
- Active investigation question: why MCP-enabled chat sessions still fail to produce practical MCP tool usage, and what minimal architecture change is required.
- Execution protocol for this phase: checkpoint -> deep investigation -> expert validation -> subagent implementation -> subagent verification -> final acceptance check.

## MCP Runtime Execution Status (2026-03-06)

- MCP chat flow now has a real tool-calling loop (model tool_calls -> MCP execution -> role: tool -> final response).
- Backend MCP tool execution now supports transports: `http`, `json`, `streamable_http`, `sse`, `stdio`.
- SSE response parsing is now handled for MCP JSON-RPC over event-stream bodies.
- Stdio path now executes MCP requests with initialize + initialized notification and bounded response read.
- Frontend callable-tool gating includes all supported transports above.
- MCP metadata prompt injection is disabled; startup chat context remains normal (no MCP text injection before user/tool flow).

## Latest Chat UX Feature (2026-03-04) - In-chat active model label + live model switcher

- Added active model indicator beside Send in `frontend/llama-custom/index.html`.
- Added in-chat switcher panel with two sections:
  - Local models (from `scan_models_command`)
  - Reachable remote models (from discovery/manual peers)
- Added parent/iframe message protocol:
  - `request-chat-model-switcher-data` -> `chat-model-switcher-data`
  - `request-chat-model-switch` -> `chat-model-switch-result`
  - `chat-active-model-changed` for parent-driven sync events
- Switch behavior:
  - Local model: immediate restart in the same terminal window and active label updates.
  - Remote model: launches remote chat via existing remote launch flow; current embedded chat remains unchanged.
- Safety/UX updates:
  - Exact iframe-source matching for switch requests (no active-window fallback for switching).
  - Timeout recovery for inventory/switch operations to avoid stuck loading/switching state.
  - `currentModelPath` now updates on confirmed local model changes to keep chat-log model metadata aligned.

### Verification

- `node --check frontend/modules/terminal-manager.js` ✅
- `cargo check --manifest-path backend/Cargo.toml` ✅
- `cargo tauri build --no-bundle` (from `backend/`) ✅
- Artifact: `backend/target/release/Arandu.exe`
  - Modify time: `2026-03-04 16:11:21 -0800`

## Follow-up Fix (2026-03-04) - Manual IP peer entry restored

- Root cause: manual peer UI block and manual peer polling/merge logic were missing, so only UDP discovery controls were visible.
- Restored manual peer settings UI in `frontend/index.html`:
  - `manual-discovery-ip`
  - `manual-discovery-api-port`
  - `manual-discovery-chat-port`
  - `manual-discovery-name`
  - `manual-discovery-peer-list`
- Restored manual peer logic in `frontend/desktop.js`:
  - storage key/state initialization
  - load/save/render helpers
  - add/remove manual peer handlers
  - fetch/merge manual peers with discovered peers
  - polling now runs when manual peers exist, even if discovery toggle is off

### Verification (follow-up)

- `node --check frontend/desktop.js` ✅
- `cargo check --manifest-path backend/Cargo.toml` ✅
- `cargo tauri build` ✅
  - MSI: `backend/target/release/bundle/msi/Arandu_0.5.5-1_x64_en-US.msi`
  - NSIS: `backend/target/release/bundle/nsis/Arandu_0.5.5-1_x64-setup.exe`
  - EXE modify time: `2026-03-04 18:50:50 -0800`

## Launch UX Update (2026-03-04) - Targeted restart rules + reload action

- Restart warning logic is now aligned to true launch-time options.
- `launch_draft_p_min` and `launch_draft_max` no longer trigger restart warnings by themselves; they are applied at request time.
- Added request payload mapping for speculative runtime tuning:
  - `speculative.n_max`
  - `speculative.p_min`
- Added small red `R` marker beside options that require model/server restart.
- Added `Reload with Changes` button near the Context & Launch section title to force immediate reload with current launch settings.

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
## 2026-03-07 Backup Addendum (Post-Previous Update)

### Runtime and MCP updates

- MCP context refresh before chat handoff is active.
- Chat shows wrench count + tool inventory panel for model-visible tools.
- MCP status includes refresh-failure diagnostics.
- JSON MCP payload handling expanded to support stdio-style JSON without URL requirement.
- Streamable/SSE response parsing hardened with loss-tolerant decode.
- Windows MCP stdio launches include `.cmd` shim fallback.

### Chat reliability and UX updates

- Chat-history delete/load/new flow lock handling stabilized (no one-time delete failure).
- Search debounce is lock-aware and guarded.
- Stream toggle (`On/Off`) added and persisted (global + per-model).
- Context counter has fallback estimation when slot metrics are unavailable.
- Context compression slot-mode now has summary+reset fallback.

### Supermemory integration status

- Supermemory now supports native tool integration (no mcp-remote required for tool calls).
- Native Supermemory tool set exposed to model includes 4 tools:
  - `supermemory_search`
  - `supermemory_add_memory`
  - `supermemory_profile`
  - `supermemory_configure_settings`
- Supermemory tool exposure is prioritized in tool ordering.
- MCP tools are currently sent without cap/schema compaction (full payload mode enabled for compatibility testing).
- Model Options now includes API key lifecycle controls:
  - `New Supermemory API Key`
  - `Delete Supermemory API Key`

### Verification and build snapshot

- `cargo check --manifest-path backend/Cargo.toml` passed.
- `node --check frontend/modules/terminal-manager.js` passed.
- `node --check frontend/desktop.js` passed.
- Inline script syntax checks for chat html passed.
- `cargo tauri build --no-bundle` passed.
- Artifact: `backend/target/release/Arandu.exe`.

## 2026-03-07 Addendum - Chat Rename/Colors + MCP Full Payload Mode

- Chat history now supports rename by clicking the chat title.
- Chat history now includes 8-color tags with per-chat palette cycling.
- Chat color mapping persists via localStorage key `aranduChatColorMapV1`.
- MCP tool compaction/cap is currently disabled; tool payloads are sent in full form.

## 2026-03-07 Addendum - Desktop List Layout + Remote Meta Display

- Remote list header is now compact and pinned in the top UI band (not in tile flow).
- Remote list and local list both have top spacing buffers to prevent overlap with controls/stats/view toggles.
- Remote model entries now show size below quantization in the right-side quant block (remote-only display difference retained).
- Latest build artifact remains: `backend/target/release/Arandu.exe`.

## 2026-03-07 Addendum - Model Badges, VRAM AMD Fallback, Settings Actions

- Added subtle tile shadow in list-based model views for clearer separation.
- Added yellow `Custom` badge to indicate non-default launch configuration:
  - remote rows: beside `Live/Cached` state badge,
  - local/list rows: beside model size metadata.
- Backend GPU monitoring now includes Windows fallback detection for non-NVIDIA adapters (AMD dGPU/iGPU) using `Win32_VideoController` (`AdapterRAM`).
- Frontend memory monitor now treats non-NVIDIA GPUs as valid and shows `N/A` instead of `Unknown` when VRAM totals are unavailable.
- Options UI now has separate actions under `Global Options`:
  - `Save Settings`
  - `Scan Models`
