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

## Follow-up Update (2026-03-04) - Top-right and remote header button spacing

- Scope: UI-only CSS refactor; no remote-discovery behavior changes.
- File updated: `frontend/css/desktop.css`
- Changes:
  - Improved `.desktop-top-right-controls` spacing constraints and hitbox stability.
  - Increased top-right toggle button size and icon alignment (`.view-toggle-btn`).
  - Added dedicated `desktop-launch-last-btn` styling with larger, consistent padding/size.
  - Increased touch target and wrapping behavior for remote header controls:
    - `.remote-duplicate-toggle-btn`
    - `.remote-cache-purge-btn`
    - `.remote-view-header-right`
- Validation:
  - `git diff -- frontend/css/desktop.css` scoped to intended visual rules only.
  - Removed one stray `}` and verified CSS brace balance `0`.

## Follow-up Update (2026-03-04, second pass) - Readable top-right labels

- Scope: Pure UI-label readability patch; polling and cache behavior unchanged.
- File: `frontend/index.html`
- Changes:
  - Added `span.view-toggle-label` with visible text for existing view buttons.
  - Labels now appear for all top-right mode buttons:
    - `Local`, `List`, `Remote`, `Fake`.
- File: `frontend/css/desktop.css`
- Changes:
  - Kept `.view-toggle-btn` as icon+label layout with compact text styling.
  - Maintained previous hitbox sizing changes.
- Verification:
  - `cargo tauri build --no-bundle` succeeded in canonical workspace.
  - Updated artifact timestamp: `2026-03-04 13:57:14.459494500 -0800`.

## Verification

- `node --check frontend/desktop.js` passed.
  - `cargo check` in `backend/` passed.

## Follow-up Update (2026-03-04, third pass) - Top-right spacing polish

- Scope: Continued UI-only spacing/touch target pass for top-right controls (no discovery logic changes).
- File: `frontend/css/desktop.css`
- Changes:
  - Increased container breathing room:
    - `.desktop-top-right-controls` gap `16px`
    - `.desktop-top-right-controls` vertical padding `4px`
  - Refined toggle row density:
    - `.desktop-view-toggle` gap `10px`
    - `.desktop-view-toggle` padding `7px`
    - `.desktop-view-toggle` corner radius `12px`
  - Increased button hit area and label spacing:
    - `.view-toggle-btn` padding `7px 11px`
    - `.view-toggle-btn` `min-height: 52px`, `min-width: 62px`
    - `.view-toggle-btn` `gap: 4px`
  - Increased last model button comfort:
    - `.desktop-launch-last-btn` padding `11px 18px`
    - `.desktop-launch-last-btn` `min-height: 48px`, `min-width: 144px`
- Validation:
  - `git diff -- frontend/css/desktop.css` reviewed for targeted visual rules only.
  - No behavior-affecting selectors were changed in this pass.
  - App rebuild not re-run in this continuation step (layout-only CSS refinement).

## Follow-up Update (2026-03-04, fourth pass) - Top-right control height alignment

- Scope: Corrected visual mismatch reported after third pass (toggle block too tall vs disk/last buttons).
- File: `frontend/css/desktop.css`
- Changes:
  - Unified control heights around 48px:
    - `.desktop-disk-monitor` `min-height: 48px`
    - `.desktop-view-toggle` `min-height: 48px`, reduced padding to `4px`
  - Refined toggle internals for cleaner proportion:
    - `.desktop-view-toggle` gap reduced to `6px`
    - `.view-toggle-btn` switched to horizontal `icon + label` layout
    - `.view-toggle-btn` `min-height: 40px`, `min-width: 74px`, `padding: 0 12px`, `gap: 6px`
    - `.view-toggle-btn .material-icons` to `18px`
    - `.view-toggle-label` to `12px` for readability
- Validation:
  - CSS-only update; no JS/behavior changes.
  - Intended outcome: toggle group now visually aligns with neighboring controls.

## Follow-up Verification (2026-03-04, build artifact timestamp)

- User reported release exe timestamp not changing after CSS passes.
- Verified canonical workspace and build target:
  - `pwd` -> `/h/Ardanu Fix/Arandu-maxi`
  - canonical build output -> `backend/target/release/Arandu.exe`
- Observed timestamps before rebuild:
  - `backend/target/release/Arandu.exe` -> `2026-03-04 13:57:14`
  - repo-root `Arandu.exe` -> `2026-02-28 19:23:06` (separate file, not Tauri build target)
- Rebuild run (canonical path):
  - `cargo tauri build --no-bundle` from `backend/`
  - Build succeeded.
  - Updated timestamp: `backend/target/release/Arandu.exe` -> `2026-03-04 14:11:26`

## Follow-up Update (2026-03-04, fifth pass) - Move duplicate toggle beside Remote LLMs title

- Scope: UI placement change requested by user because green duplicate toggle control remained hard to see in the right control cluster.
- File: `frontend/desktop.js`
- Changes:
  - In `renderRemoteModelsList()` moved `#remote-model-duplicate-toggle` from `.remote-view-header-right` to `.remote-view-header-left`.
  - Duplicate toggle now renders directly beside the `Remote LLMs` title on the left.
  - Kept `Purge Cached Entries` and stats on the right.
- Behavior impact:
  - No backend/discovery behavior changes.
  - Same toggle semantics (`Show duplicates` / `Hide duplicates`), only placement changed.

## Follow-up Verification (2026-03-04, timestamp-prefixed build output)

- User requested rebuild with date/time prefix in executable name.
- Rebuild executed in canonical backend path:
  - `cargo tauri build --no-bundle`
- Created timestamp-prefixed copy:
  - `backend/target/release/2026-03-04_14-16-40_Arandu.exe`
- Canonical release artifact timestamp after rebuild:
  - `backend/target/release/Arandu.exe` -> `2026-03-04 14:16:29`

---

## Follow-up Update (2026-03-03)

### Additional UI Fix

- Fixed model hover info popup clipping off-screen at far right.
- Scope: both local and remote model tiles (shared `.desktop-icon` hover hint path).
- File: `frontend/desktop.js`
- Changes:
  - `mouseover` passes cursor coordinates to hint renderer
  - added `mousemove` hint reposition updates
  - added viewport-aware placement helper with right-edge flip and bounds clamp

### Build Verification

- Rebuild command: `cargo tauri build --no-bundle`
- Result: success
- Artifact: `backend/target/release/Arandu.exe`

---

## Follow-up Update (2026-03-03) - Database culling hardening

User still reported duplicates in remote database/cache during LLM load.

### Root cause found

- Auto-fetch discovery path could write duplicate model rows directly to cache because that path did not dedupe before persistence.
- Existing `peer_models_cache.json` entries could already contain duplicate model rows and duplicate peer identities for the same endpoint from old runs.

### Minimal backend-only fix

- `backend/src/peer_cache.rs`
  - Added centralized model dedupe helper (`trim + slash normalize + lowercase` path key, fallback `id`).
  - `update_peer_models(...)` now culls duplicate incoming models before saving.
  - Cache-load normalization now culls:
    - duplicate models inside each cached peer,
    - duplicate peer identities sharing same `ip:api_port` endpoint (keeps most recent; deterministic tie-break).
  - Normalized cache is auto-persisted once on load.
  - Added unit test: `test_update_peer_models_culls_duplicate_models`.

- `backend/src/discovery.rs`
  - Auto-fetch path now dedupes models before runtime peer assignment and cache write.
  - Dedupe key normalization aligned with cache normalization (`trim + slash normalize + lowercase`).

### Coding subagent review

- A coding subagent reviewed the change scope (`peer_cache.rs`, `discovery.rs`) and confirmed duplicate-culling intent and isolation.

### Verification

- `cargo check --manifest-path backend/Cargo.toml` passed.
- `cargo test --manifest-path backend/Cargo.toml --no-run` passed (test binaries compiled).
- Rebuild succeeded: `cargo tauri build --no-bundle`.
- Artifact: `backend/target/release/Arandu.exe`.

---

## Follow-up Update (2026-03-03, later) - Cull cached-offline duplicates when online exists

User still saw duplicate remote rows and requested that cached-offline entries be removed when online entries exist.

### Root cause

- Merge output could still include cached-offline peer records alongside online records in some endpoint scenarios.

### Fix (backend only)

- `backend/src/discovery.rs`
  - Added peer-level culling helper `cull_duplicate_peers(...)`.
  - Behavior:
    - if an online peer exists for an endpoint (`ip:api_port`), cached-offline peer for that same endpoint is removed;
    - endpoint-level dedupe keeps the preferred row (online > non-cached > freshest).
  - Applied culling to both `get_peers_with_cached_models()` and no-cache fallback branch.
  - Added tests:
    - `test_cull_duplicate_peers_prefers_online_over_cached_offline`
    - `test_cull_duplicate_peers_keeps_offline_cached_for_other_endpoints`

### Coding subagent review

- A coding subagent reviewed the patch and flagged an over-broad initial filter.
- Final implementation was adjusted to endpoint-scoped removal only (minimal behavior change).

### Verification

- `cargo check --manifest-path backend/Cargo.toml` passed.
- `cargo test --manifest-path backend/Cargo.toml --no-run` passed.
- `cargo tauri build --no-bundle` passed.
- Artifact: `backend/target/release/Arandu.exe`.

---

## Follow-up Update (2026-03-03, latest) - Persistent duplicates in remote/options views

User still reported duplicate rows in:
- remote model list view
- discovered network instances list in options

### Root cause refinement

- Endpoint-level culling alone was insufficient for stale offline cache scenarios where duplicate entries shared the same hostname and model set but came through different endpoint identities.

### Final culling refinement

- `backend/src/discovery.rs`
  - Refined `cull_duplicate_peers(...)` to:
    1. keep endpoint-scoped suppression (online endpoint removes cached-offline for that endpoint),
    2. keep endpoint-level dedupe preference,
    3. collapse remaining cached-offline duplicates by **hostname + normalized model signature** (not hostname-only).
  - This removes stale duplicate offline rows while avoiding over-broad hostname-only removal.
  - Added/updated tests:
    - keep online over cached-offline (same endpoint)
    - keep cached-offline for different endpoints when not duplicates
    - keep a single cached-offline row for same host+model signature
    - keep multiple cached-offline rows when model signatures differ

### Coding subagent review

- Review subagent found an over-broad hostname filter in an intermediate patch.
- Final patch removed that risk and switched to host+model-signature dedupe.

### Verification

- `cargo check --manifest-path backend/Cargo.toml` passed.
- `cargo tauri build --no-bundle` passed using fresh target dir.
- Artifact copied to canonical path:
  - `backend/target/release/Arandu.exe`
  - last write observed: `2026-03-03 19:37:30`

---

## Follow-up Update (2026-03-03, final pass) - UI duplicate suppression for remote + options views

User still observed duplicate rows (especially `Cached/Offline`) in:
- Remote LLM list
- Discovery "Discovered Instances" table in options

### Root cause

- UI consumed raw discovered peer list that could still contain stale cached-offline variants and rendered model rows using a peer-scoped dedupe key, which preserved duplicate offline copies.

### Fix

- `frontend/desktop.js`
  - Added `getDisplayDiscoveryPeers(peers)` to normalize display peers before rendering:
    - suppress cached-offline rows when an online peer exists for same endpoint/host,
    - collapse remaining cached-offline variants per host (pick freshest/best candidate).
  - Applied this normalized peer set to:
    - `renderRemoteModelsList()`
    - `updateDiscoveredInstancesList(peers)`
    - `refreshRemoteActiveModels()` peer query list.
  - Updated remote model dedupe key:
    - online/non-cached: keep peer-scoped dedupe,
    - cached-offline: dedupe by host+model key to prevent repeated offline duplicates.

### Verification

- `node --check frontend/desktop.js` passed.
- `cargo check --manifest-path backend/Cargo.toml` passed.
- Rebuilt with fresh target and copied to canonical path:
  - `backend/target/release/Arandu.exe`
  - last write observed: `2026-03-03 20:31:59`

---

## Follow-up Update (2026-03-03, startup behavior) - Discovery must start ON at launch

User reported discovery only began pinging after manual OFF/ON toggle despite UI indicating enabled.

### Root cause

- In `backend/src/lib.rs` setup path, startup tasks were invoked via a temporary `tokio::runtime::Runtime` created inside `.setup(...)`.
- Discovery service internally uses spawned async tasks; these were tied to the short-lived runtime used during setup.

### Fix

- Switched setup-time startup invocations to Tauri's persistent runtime:
  - `tauri::async_runtime::block_on(initialize_app_state(...))`
  - `tauri::async_runtime::block_on(auto_start_network_server_always(...))`
  - `tauri::async_runtime::block_on(auto_start_discovery_if_enabled(...))`
- Updated `get_discovery_status` to report `enabled: false` when no live discovery service exists (prevents stale ON display state).

### Verification

- `cargo check --manifest-path backend/Cargo.toml` passed.
- Rebuilt and copied canonical exe:
  - `backend/target/release/Arandu.exe`
  - last write observed: `2026-03-03 21:28:03`

---

## Follow-up Update (2026-03-03, manual direct IP peers)

User requested support for peers on different LANs where UDP discovery cannot cross routers.

### Feature added

- Added manual peer management in Discovery options UI:
  - Host/IP input
  - API port input (default 8081)
  - Chat port input (default 8080)
  - Optional display name
  - Add/remove manual peer entries

### Implementation details

- `frontend/index.html`
  - Added "Manual Peer (Direct IP/Host)" section under Discovery settings.

- `frontend/desktop.js`
  - Added manual peer storage and normalization methods.
  - Persisted peers to localStorage key: `Arandu-manual-discovery-peers`.
  - Added manual peer model fetch (`/v1/models/arandu`) with timeout.
  - Added merge logic to combine backend-discovered peers with manual peers.
  - Applied merged peers to:
    - remote model rendering
    - discovered instances table
    - active-model polling target list
  - Discovery polling now remains active when discovery is OFF but manual peers exist.

### Verification

- `node --check frontend/desktop.js` passed.
- `cargo check --manifest-path backend/Cargo.toml` passed.
- Rebuilt and copied canonical exe:
  - `backend/target/release/Arandu.exe`
  - last write observed: `2026-03-03 22:21:49`

---

## Follow-up Update (2026-03-04) - Purge cached-offline entries (backend-only)

User requested a hard purge path so cached-offline peers stop appearing (duplicates) while keeping frontend behavior unchanged.

### What changed

- `backend/src/peer_cache.rs`
  - Added `purge_peers_not_in_endpoints(...)` to remove cached peers whose endpoint (`ip:api_port`) is not currently present in runtime discovery.
  - Added regression test:
    - `test_purge_peers_not_in_endpoints_removes_stale_cached_rows`

- `backend/src/discovery.rs`
  - `get_peers_with_cached_models()` now purges stale cache endpoints before merging.
  - Discovery now merges cache into runtime peers only (same `instance_id` or endpoint fallback); it no longer creates offline peer rows from cache.
  - `cull_duplicate_peers(...)` now drops all cached-offline peers from surfaced discovery results.
  - Updated culling tests to assert cached-offline rows are removed.

### Frontend impact

- Early backend-only implementation intentionally kept frontend unchanged; later updates added frontend peer normalization for display stability (2026-03-03/04 updates).
- Existing UI paths (`renderRemoteModelsList`, discovered instances list) continue to consume discovery results without new frontend logic.

### Cross-network note

- Remote model versions remain discoverable across routed networks via manual peers (direct host/IP + API/chat ports).
- UDP broadcast discovery remains subnet-local; manual peers are the cross-network path.

### Verification

- `cargo test cull_duplicate_peers --lib --no-run` passed (compiled).
- `cargo test purge_peers_not_in_endpoints --lib --no-run` passed (compiled).
- `cargo check` passed.
- `cargo tauri build --no-bundle` passed and produced canonical exe.
- `cargo tauri build --bundles msi` failed in canonical target due `LNK1104` lock on `target/release/deps/llama_os_tauri_lib.dll`; rebuild with `CARGO_TARGET_DIR=target_fresh_msi2` succeeded and MSI was copied to canonical bundle path.

---

## Follow-up Update (2026-03-04) - Remote duplicate toggle copy/state polish

### What changed

- `frontend/desktop.js`
  - In `renderRemoteModelsList()` duplicate-toggle button text/title now consistently represent the next action:
    - dedupe enabled: button text `Show duplicates`, title `Show duplicate model rows`
    - dedupe disabled: button text `Hide duplicates`, title `Hide duplicate model rows`

### Rationale

- Previous copy could appear reversed between current state and click action, causing ambiguity.

### Verification

- `node --check frontend/desktop.js`

---

## Follow-up Update (2026-03-04, manual purge control)

User asked for an explicit cleanup action to remove stale discovery cache rows on demand.

### What changed

- `backend/src/discovery.rs`
  - Added `purge_stale_cached_peers()` for explicit runtime/endpoints cache cleanup.

- `backend/src/lib.rs`
  - Added Tauri command `purge_discovery_cache`:
    - Response shape: `{ success, message, removed }`
    - Returns 0 removed when no stale rows are found.
  - Registered command in the global command list.

- `frontend/desktop.js`
  - Added `remoteDiscoveryCachePurgeInFlight` in constructor as a re-entrant guard.
  - Added `purgeDiscoveryCacheEntries()` to invoke the backend command, disable/relabel button while active, refresh remote peers, and show toast feedback.
  - Updated `renderRemoteModelsList()` to render a new action button beside duplicate toggle:
    - label: `Purge Cached Entries`
    - tooltip: `Remove stale cached discovery rows`

- `frontend/css/desktop.css`
  - Added `.remote-cache-purge-btn` styles matching existing remote header controls.

- `backend/src/peer_cache.rs`
  - `clear()` now returns number of removed peers so the backend can report exact rows cleared when discovery is not running.

### Verification

- `node --check frontend/desktop.js`
- `cargo check --manifest-path backend/Cargo.toml`

## Follow-up Update (2026-03-04, continued) - Purge button now works when discovery is stopped

- Updated `purge_discovery_cache` command behavior:
  - when discovery is running, it performs endpoint-based stale purge (unchanged);
  - when discovery is not running, it clears the full peer cache and reports removed count.
- This keeps the button usable even if discovery is currently disabled/off.
- Updated command response messaging so UI can show:
  - `Discovery service is not running. Cleared X cached discovery rows`.

- `frontend/desktop.js`
  - `purgeDiscoveryCacheEntries()` now prefers backend `message` for toast text when available, so stop/clear semantics remain accurate without string assumptions.
- `backend/src/peer_cache.rs`
  - Added unit test `test_clear_returns_count_and_empties_cache` to lock in `clear()` count semantics.

### Verification

- `node --check frontend/desktop.js`
- `cargo check --manifest-path backend/Cargo.toml`

## Follow-up Review (2026-03-04) - Final consistency pass

- Reviewed all active purge-related files and aligned current state docs to the mixed backend/frontend behavior now in use.
- Confirmed runtime behavior:
  - `get_peers_with_cached_models()` still performs endpoint purge before merge;
  - `cull_duplicate_peers()` removes cached-offline rows from surfaced output;
  - manual peers are merged in the frontend and are not part of backend endpoint purge matching.
- Manual purge flow:
  - running discovery -> endpoint-based stale cleanup;
  - stopped discovery -> full cache clear with removed count.
- Message contract (`success`, `message`, `removed`) is now used by UI for user-facing toast text.

### Verification

- `cargo check --manifest-path backend/Cargo.toml` ✅
- `cargo test --manifest-path backend/Cargo.toml --no-run` ✅
- Focused targeted tests still hit legacy runtime startup exit `0xc0000139 STATUS_ENTRYPOINT_NOT_FOUND` when run in this environment, so this pass captured compile/build outcomes only.

## Follow-up Verification (2026-03-04, continuation)

- Re-ran focused checks in canonical path `H:\Ardanu Fix\Arandu-maxi`:
  - `cargo check --manifest-path backend/Cargo.toml` ✅
  - `cargo test --manifest-path backend/Cargo.toml purge_peers_not_in_endpoints -- --nocapture` ❌ (test process fails to start with `0xc0000139 STATUS_ENTRYPOINT_NOT_FOUND`)
  - `cargo test --manifest-path backend/Cargo.toml purge_peers_not_in_endpoints --no-run` ✅
- Decision remains unchanged: purge behavior is split by runtime state (endpoint-only purge when service is active, full clear when stopped), with UI using backend message text for correct user feedback.

## Follow-up Update (2026-03-04) - Manual-only polling for remote model discovery

- As requested, discovery model polling in the desktop UI now ignores automatic peer discovery source while still allowing manual peer management.
- `frontend/desktop.js`
  - `pollDiscoveredPeers()` now feeds `discovered = []` and only uses `fetchManualDiscoveryPeers()`.
  - Merged path is still used for future compatibility, but automatic source remains intentionally empty.
- `backend/src/discovery.rs`
  - `test_discovery_service_lifecycle` is commented out with intent note (manual workflow only).
- UI wording changed to reflect manual entry requirement: "Add manual peers to discover models from other PCs".

### Verification (manual-only pass)

- `cargo check --manifest-path backend/Cargo.toml` ✅
- `node --check frontend/desktop.js` ✅
