# Arandu Development Status - 2025-02-23

## 🟩 MCP Integration (Frontend + Backend)

**Status:** Implemented (connection management phase complete)
**Date:** 2026-02-24
**Context:** MCP server registry now supports CRUD + test + persistence in UI and settings.

### Scope completed
- Added MCP connection management in **Network** area: add/edit/remove/save, test, toggle, and reload.
- Kept phase-one boundaries: connection management only; no MCP tool invocation in chat yet.
- MCP registry is owned by backend settings (`GlobalConfig.mcp_servers`) and surfaced through dedicated Tauri commands.

### ✅ Daily update (2026-02-25)
- Fixed MCP JSON transport save-path behavior so `transport: json` no longer requires a URL in either frontend or backend validation.
- Added backend test coverage for this path: `mcp_validation_accepts_json_without_url`.
- Updated MCP test behavior to provide a clear error when testing JSON transport without URL instead of blocking save.
- Fixed MCP transport dropdown contrast/visibility by styling `#mcp-transport` and option list to dark backgrounds with light text.
- Rebuilt executable to include today’s MCP fixes: `backend/target/release/Arandu.exe`.
- Re-ran backend tests and confirmed all passing (`38 passed` after MCP discovery additions).

### ✅ MCP tools discovery milestone (2026-02-25)
- Implemented backend MCP tool discovery for HTTP/StreamableHttp connections: added `list_mcp_tools` command and persisted discovery metadata fields (`tools`, refresh timing/status/message/error).
- Added shared MCP response parsing for `tools/list` and tests for successful and error responses.
- Updated MCP UI to show cached tool count, status, and a quick preview for each configured connection, plus per-row `MCP Tools` action.
- Preserved tool cache when editing/saving existing MCP settings by carrying cached fields in the save payload.
- Added discovery UX styles in `frontend/css/desktop.css` for status chips and tool preview readability.
- Added dedicated MCP Tools detail window (`openMcpToolsWindow`) that opens from each MCP row, lists each tool name/description, and shows input/output schema summaries with optional raw schema expanders.
- Added smoke/build checks: `cargo test --manifest-path backend/Cargo.toml` passes, and `node --check frontend/desktop.js` passes.
- Added a dedicated MCP action in the start menu (`MCP Connections`) that opens the existing Network popup so users can actually reach MCP management.

### ✅ Build checkpoint (2026-02-25)
- User requested packaged build verification for MCP discovery changes.
- Executed `cargo tauri build` from `backend` and build succeeded.
 - New executable: `backend/target/release/Arandu.exe`
 - Bundles produced:
     - `backend/target/release/bundle/msi/Arandu_0.5.5-1_x64_en-US.msi`
     - `backend/target/release/bundle/nsis/Arandu_0.5.5-1_x64-setup.exe`

### ✅ MCP discoverability build checkpoint (2026-02-25)
- Ran `cargo tauri build --no-bundle` after MCP start menu wiring.
- New executable: `backend/target/release/Arandu.exe`.
- Re-ran `cargo test --manifest-path backend/Cargo.toml -- --nocapture` and confirmed `38 passed`.
- Full runtime MCP smoke checklist still requires manual desktop interaction (open Network widget > MCP panel > add/edit/test actions).

### ✅ MCP visibility fix in taskbar (2026-02-25)
- Moved MCP access to the dock in the same visual style as Settings/Hugging Face/Llama.cpp: added `mcp-dock-icon` with `plug` icon and title `MCP Connections` in `frontend/index.html`.
- Added dock click handling in `frontend/desktop.js` to call `openMcpConfigPanel()` directly.
- Rebuilt executable with `cargo tauri build --no-bundle`; output: `backend/target/release/Arandu.exe`.
- Static checks passed: `node --check frontend/desktop.js`.
- Verification limitation: launch-time visual and interaction verification is still a manual GUI check and requires running the built `.exe` locally.

### ✅ Knowledge memory sync (2026-02-25)
- Confirmed MCP memory locator is configured in `tools.yaml` with local server key `nowledge-mem` at `http://127.0.0.1:14242/mcp`.
- Logged MCP discovery/build outcomes in:
   - `docs/knowledge-base/mcp-tools-build-log-2026-02-25.md`
   - `docs/knowledge-base/mcp-tools-memory-sync-2026-02-25.md`.
- Updated docs index entry for the new KB item in `docs/INDEX.md`.

### ✅ Chat restart-contract hardening (2026-02-25)
- Fixed chat restart message parity in `frontend/modules/terminal-manager.js`: restart-required config updates now always send a `settings-saved` response with `restartTriggered: true`.
- Updated chat iframe handler in `frontend/llama-custom/index.html` to treat this as a completed restart signal: baseline is refreshed and connection status loading state is cleared.
- This prevents stale `needs restart` state after a successful in-chat restart request.
- Rebuilt executable after the restart-contract hardening: `backend/target/release/Arandu.exe`.

### Implementation outcome
- **Backend (`backend/src/models.rs`, `backend/src/config.rs`, `backend/src/lib.rs`)**
  - Added `McpTransport`, `McpServerConfig`, and `McpTestResult` types.
  - Added persistent `mcp_servers` config array with migration-safe defaults.
  - Added Tauri CRUD/test commands and wired them into command handler list.
  - Implemented transport-aware `test_mcp_connection` for stdio and URL transports.
  - Added `JSON` transport mode that sends `Accept: application/json` during MCP initialize tests.
- **Frontend (`frontend/index.html`, `frontend/css/desktop.css`, `frontend/desktop.js`)**
  - Added MCP list + form in Network popup, transport-aware field behavior, row status indicators, and save/test/delete/toggle actions.
  - Added local MCP form and state management, including parse helpers for JSON args/env/headers.
  - Added startup load and post-mutation refresh flow to keep UI aligned with persisted config.

### Latest UI fix (MCP transport dropdown visibility)
- Updated MCP transport select styling in `frontend/css/desktop.css` so the dropdown and option list uses dark surfaces with light text.
- Fixed visibility issue where `JSON/HTTP/...` options were hard to read due to white-on-white contrast.
- Change is scoped to `#mcp-transport` and its `option` items only.

### MCP validation fix (JSON transport)
- Updated MCP transport validation so `json` no longer requires a URL to save or update a connection.
- Kept URL optional for JSON transport in the form layer while retaining clear runtime feedback when a JSON MCP entry is tested without a URL.
- Added backend unit coverage: `mcp_validation_accepts_json_without_url`.

### Current milestone: JSON transport save behavior completed
- `2026-02-24` — `JSON` MCP transport can now be saved/edited without a URL.
- Backend validation now treats `json` as a valid transport without URL requirement.
- Frontend validation now allows `json` to be saved without URL, matching backend behavior.
- Manual test confirmation: JSON MCP connection now connects successfully in app after rebuild.
- Rebuilt executable after fix: `backend\target\release\Arandu.exe`.

### ✅ Half-Context launch option added (2026-02-25)
- Added new context menu launch action: `Load with half context`.
- New one-shot backend command: `launch_model_with_half_context` in `backend/src/lib.rs`.
- Launch flow uses temporary in-memory argument override and restores config after launch.
- `--context-shift` is passed through terminal creation so UI reflects launch arguments via `openServerTerminal(..., launchArgs)`.
- Existing preset launches and external launch flow are unchanged.
- MCP popup and MCP commands remain untouched.
- Files touched: `frontend/desktop.js`, `backend/src/lib.rs`.
- Verification: `cargo check --manifest-path backend/Cargo.toml` and JS syntax checks passed.

### Validation
- Backend checks: `cargo check` and `cargo test` succeeded in `backend`.
- Full packaging now succeeds after version metadata fix (`0.5.5-1`):
  - `cargo tauri build` completed and generated both `.msi` and `.exe` bundles.
- MCP runtime smoke checklist (manual UI path) is not fully executable in this environment because GUI interaction is unavailable.
- Added MCP command payload guard tests in `backend/src/lib.rs` (`mcp_validation_*`), covering name, timeout, and URL/transport validation paths.
- Added explicit `JSON` transport coverage:
  - `McpTransport::Json` deserializes from `"json"`.
  - URL transport validation accepts JSON mode for HTTP-style endpoints.
- Extended MCP validation coverage to cover malformed URL parsing and stdio transport command enforcement, plus explicit `StreamableHttp` acceptance case.
- Per-request runtime smoke checks completed:
  - Executable launch sanity check: `backend\\target\\release\\Arandu.exe` starts and exits cleanly when terminated.
  - MCP-oriented Rust tests with `cargo test --manifest-path backend/Cargo.toml mcp -- --nocapture` passed (1 MCP-related test).
  - New MCP validation tests now pass with `cargo test --manifest-path backend/Cargo.toml mcp_validation -- --nocapture` (9 tests).

### Additional hardening completed
- Added backend-side MCP payload validation for stricter transport consistency (required command/URL checks and URL syntax validation).
- Added focused MCP serde migration tests in `backend/src/models.rs`:
  - legacy config without `mcp_servers` loads with empty list,
  - `streamable_http` transport round-trips to `McpTransport::StreamableHttp`.
- Added frontend safety hardening for model list rendering:
  - `refreshDesktopIcons()` now sorts with null-safe arrays and tolerates missing model fields (name/path/size/arch/quantization) without breaking the desktop icon refresh path.
- Updated MCP/network UI checks to use model-path keyed configs consistently in update-indicator flow.
- Made the network popup content scrollable (`.network-simple-content` with `overflow-y: auto`) so MCP and Server Address sections remain reachable when content extends beyond viewport.

### Next (remaining, non-blocking to this phase)
- Add manual runtime smoke check: create/edit/test entries (stdio + URL transport), then restart app and verify list reload.
  - Checklist added: `docs/plans/2026-02-24-mcp-runtime-smoke-checklist.md`
- Keep known packaging/version blockers documented in separate build-state items.

**MCP phase milestone recorded (2026-02-24)**

## 🟡 In-Flight Fix: Launch Args Persistence for Recovery/Restart

**Status:** Partially complete and verified in frontend terminal state
**Date:** 2026-02-24
**Context:** Speculative drafting reliability and crash recovery still using stale launch parameters

### Current State (What we changed)
- Added a fallback resolver in `openServerTerminal(...)` (`frontend/modules/terminal-manager.js`) to load saved `custom_args` from `get_model_settings` whenever launch args are missing at terminal creation time.
- `launchArgs` is now normalized to a trimmed string and stored as `terminalInfo.launchArgs` before UI creation.
- This directly aligns with existing restart logic in `terminal-manager.js`, which reads `terminalInfo.launchArgs` for:
  - speculative flag detection (`-md`, `--model-draft`)
  - restart request config restoration
  - safer recovery flow when a launch path fails

### Why this matters now
- Normal launch path from `frontend/desktop.js` still passes `openServerTerminal(...)` without an explicit arg in some flows.
- Recovery previously saw empty terminal launch args, so speculative stripping and restart re-launch decisions could use stale/incorrect state.
- With this patch, terminal state now consistently records the real args used for that model launch.

### Validation
- Backend sanity checks from earlier pass were clean in this branch: `cargo check` and `cargo test`.
- No additional code changes were needed in `backend/` for this fix pass.

## 🟢 STATUS: Advanced Parameters & Speculative Drafting Live

**Status:** UI ENHANCED - INTELLIGENT DRAFTING ACTIVE
**Date:** 2025-02-23
**Build Status:** ✅ SUCCESS (Release Build)

### Recent Fix: Advanced Parameters & Intelligence Bridge (2025-02-23)
**Implementation:**
1. **Frontend (Chat UI):** Expanded the professional, scrollable sidebar to include 35+ advanced options.
2. **Advanced Chat Features Added:**
   - **🛑 Stop Button:** Integrated `AbortController` to immediately interrupt LLM generation and clear the queue.
   - **📎 Multi-File Attachments:** Added a `+` button supporting Images (Vision models), PDFs, and Word docs. Includes automatic text extraction for documents using PDF.js and Mammoth.js.
   - **🔹 Draft Highlighting:** Tokens generated by the draft model and accepted by the main model are now colored **Light Blue** in the chat stream.
   - **📊 Performance Stats:** Every response now includes a detailed readout of Total Tokens, Main vs Draft token counts, and Time to First Token (TTFT).
3. **Comprehensive Parameters Added:**
   - **Sampling/Runtime:** System prompt, Temperature, Top P, Min P, Top K, Max Tokens, Repeat Penalty, Repeat Last N, Presence Penalty, Frequency Penalty, Context Window, **XTC (Exclude Top Choices), DRY (Don't Repeat Yourself), Reasoning Budget/Format.**
   - **Launch/Hardware:** Context Size, Context Shift, GPU Layers (-ngl), CPU MoE Layers (-ncmoe), GPU Split Mode, Main GPU Index, Flash Attention, Use MMap, KV cache K/V compression types (9 types supported), **Speculative Drafting (with "Sense Model" auto-architecture detection, Draft P-Min, and Draft Max Tokens), NUMA Optimization, Use Pinned Memory, Embedded Template (Jinja),** Environment Variables.
4. **Speculative Drafting "Sense Model":**
   - Implemented a "Sense Compatible Models" button.
   - The UI queries the main app to detect the current model's architecture (e.g., Llama, Qwen).
   - Dynamically populates a scrollable dropdown of local models matching that architecture.
   - Automatically handles `-md` path resolution in the backend for both dev and release paths.
4. **Restart Bridge & Health Checks:**
   - Enhanced `postMessage` bridge with robust source-window identification.
   - **Robust Health Check:** Implemented 15 retries (30-second window) to account for slow dual-model loading during speculative drafting.
   - **Iframe Sync:** The chat UI reloads ONLY when the server is confirmed "Healthy" (responding with 200 OK).
5. **Flash Attention Fix:** Resolved a critical crash where `-fa` was sent without an explicit `on/off` value.

**Files Modified:**
- `frontend/llama-custom/index.html`
- `frontend/modules/terminal-manager.js`
- `backend/src/process.rs`

### 🟢 Resolved Features & Fixes (2025-02-23)

- **✅ Auto-Recovery System:** Implemented logic to automatically strip failing draft model parameters from the saved configuration. If a speculative restart fails, Arandu now cleans the `custom_args` so the main model remains launchable.
- **✅ Performance Stats Fixed:** Enabled `stream_options: { include_usage: true }` to ensure Total, Main, and Draft token counts are received from the server.
- **✅ Stat Readout Overhaul:** Tripled the font size (30px) of the performance statistics (Total, Main, Draft, TTFT) for high visibility.
- **✅ Draft Highlighting Enhanced:** Improved detection of speculative tokens by checking multiple metadata flags in the JSON stream.
- **✅ Smooth Chat Scrolling:** Fixed the chat container flex logic and scroller to ensure the window automatically stays pinned to the bottom during generation.
- **✅ Speculative Drafting "Sense Model":** Automated compatibility checking and model selection for speed drafting.
- **✅ Robust Health Check Logic:** Fixed "blank screen on restart" by waiting up to 30s for models to load.
- **✅ Flash Attention Syntax:** Fixed server crash by sending `-fa on/off` explicitly.
- **✅ Portable Build Created:** Assembled `Arandu_v0.5.5-beta_Portable_Sense.zip` containing the latest executable and resources.
- **✅ Chat Tab (Native) Integration:** Fixed iframe loading race conditions and path resolution.
- **✅ Parameter Panel Interaction:** Panel now starts collapsed with a floating toggle button.
- **✅ Chat Input (Enter Key):** Fixed issue where Enter key added a newline instead of sending.
- **✅ Chat UI (Send Button):** Fixed syntax duplication causing script crashes.
- **✅ Splash Screen Hang:** Corrected syntax errors in `desktop.js`.

---

## ⚠️ KNOWN BUGS (URGENT)

*(All current critical UI and logic bugs have been resolved in the latest session)*

---

## Changes Made: Advanced Parameter Integration

### Phase 1: Complete Code Deletion (~2,373 lines removed)
*(Refer to previous logs for details on the destruction of the legacy custom chat module)*

### Phase 2: Llama-Server Custom UI Implementation
*(Refer to previous logs for the creation of the standalone `llama-custom/index.html`)*

### Phase 3: Process Integration
- Modified `backend/src/process.rs` to dynamically resolve UI paths and handle speculative draft models.

### Summary Statistics
- **Total Removed:** ~2,373 lines
- **Total Added:** ~850 lines (UI + Logic + Bridge)
- **Net change:** ~ -1,500 lines

## Build Information
- **Executable:** `H:\Ardanu Fix\Arandu-maxi\backend\target\release\Arandu.exe`
- **Portable Zip:** `H:\Ardanu Fix\Arandu-maxi\Arandu_v0.5.5-beta_Portable_Sense.zip`

---
*Document Updated: 2025-02-23 (Advanced Drafting & Health Check Session)*

## 🔜 FUTURE WORK PLANNED

### Restart Contract + Chat Stream Hardening (2026-02-24)

**Status:** Partially complete, contract parity fixed (UI + parent)
**Date:** 2026-02-24
**Priority:** High

#### What is already done
- Added/validated restart-impact and environment normalization logic in `frontend/modules/terminal-manager.js`.
- Kept authoritative validation on the parent side (`TerminalManager`) with child `request-restart` contract preserved.
- Added a small smoke test for stream parsing in `frontend/chat-stream-smoke.mjs` covering:
  - `[DONE]` handling,
  - trailing partial stream line processing,
  - usage-based draft token preference,
  - speculative signal fallback counting.
- Built `backend/target/release/Arandu.exe` successfully with `cargo tauri build --no-bundle`.

#### Remaining (not yet working)
- Full `cargo tauri build` still fails at bundle stage with:
  - `optional pre-release identifier in app version must be numeric-only and cannot be greater than 65535 for msi target`
- End-to-end chat/iframe restart + stream send path still needs final UI-level smoke confirmation in a real browser-like environment.
- Parent-side restart contract for `settings-saved` messaging in restart-required paths was parity-fixed and validated via source inspection + syntax checks.

#### Planned actions (next)
1. Add a robust JS/DOM test harness for `frontend/llama-custom/index.html` send/restart flows (message contract + stream behavior).
2. Keep a focused smoke log for 4 restart scenarios and 3 stream edge cases before any release packaging.

### Process rule to follow on every follow-up session
- State-saving rule (new): verify and apply save/update discipline at the start and end of each task. Log meaningful milestones immediately to `THIS-PROJECTS-CURRENT-STATE.md`, and persist stable facts into `docs/knowledge-base/`.
