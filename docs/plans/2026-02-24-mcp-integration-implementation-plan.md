# MCP Integration Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add backend-owned MCP connection management (add/edit/delete/test/persist) with UI in the Network area, without wiring tool invocation into chat yet.

**Architecture:** Store MCP connection definitions in `GlobalConfig` and expose a small set of Tauri commands for CRUD + test. The desktop UI will render/manage the list and call backend commands, reusing the existing network panel state patterns.

**Tech Stack:** Rust (Tauri commands, serde, tokio), JavaScript (frontend modules), existing CSS/HTML component pattern.

---

### Task 1: Add MCP data model to backend settings

**Files:**
- Modify: `backend/src/models.rs`
- Modify: `backend/src/config.rs`

**Step 1: Define MCP types and config field**

- Add `McpTransport`, `McpConnection`, `McpTestResult` structs/enums in `models.rs` with serde defaults.
- Add `pub mcp_servers: Vec<McpConnection>` to `GlobalConfig` and default to `Vec::new()` in `Default` impl.

**Step 2: Add migration-safe serde annotations**

- Ensure missing field reads as empty list by using `#[serde(default)]` on the new field.
- Ensure list fields inside structs have `#[serde(default)]` where sensible (env/headers).

**Step 3: Verify settings conversion remains intact**

- Run `cargo test` from `backend` and confirm existing tests still pass.

### Task 2: Add backend Tauri commands for MCP CRUD

**Files:**
- Modify: `backend/src/lib.rs`

**Step 1: Add command handlers**

- Add command functions:
  - `get_mcp_connections() -> Result<Vec<McpConnection>, String>`
  - `save_mcp_connection(connection: McpConnection) -> Result<String, String>`
  - `delete_mcp_connection(id: String) -> Result<(), String>`
  - `toggle_mcp_connection(id: String, enabled: bool) -> Result<(), String>`
  - `test_mcp_connection(id: String) -> Result<McpTestResult, String>`

**Step 2: Persist changes through existing save path**

- For mutations (`save`, `delete`, `toggle`) update `state.config.lock().await.mcp_servers`, then call `save_settings(&state).await`.

**Step 3: Register commands**

- Add new command symbols in the `tauri::generate_handler![]` list near existing network commands.

**Step 4: Compile check**

- Run `cargo check` in `backend`.

### Task 3: Implement MCP connection test behavior

**Files:**
- Modify: `backend/src/lib.rs`

**Step 1: Implement HTTP/SSE test**

- For `Sse`, `Http`, `StreamableHttp`, run a connect/test request with timeout.
- Use lightweight `reqwest` call to target URL; return status/time in `McpTestResult`.

**Step 2: Implement stdio test**

- For `Stdio`, spawn process with timeout, confirm start-up, then terminate.
- Capture and return process-level errors if spawn/initialization fails.

**Step 3: Write explicit errors and durations**

- Return stable keys: `success`, `latency_ms`, `message`, optional `status_code`/`exit_code`.

**Step 4: Run focused tests**

- Add/extend backend unit test in `backend/src/lib.rs` or adjacent tests if feasible.
- Run specific test command and record output.

### Task 4: Add MCP UI section in desktop panel

**Files:**
- Modify: `frontend/index.html`
- Modify: `frontend/css/desktop.css`
- Modify: `frontend/desktop.js`

**Step 1: Add markup**

- In `index.html`, add a compact MCP block inside existing network widget popup (list + controls + form fields).
- Include transport select, name, URL, command, args, env, headers, timeout, Save/Test/Delete buttons.

**Step 2: Add styling**

- Add minimal styles in `desktop.css` following current theme variables.
- Add state classes (`.mcp-entry`, `.mcp-test-ok`, `.mcp-test-fail`, `.mcp-badge`) and keep responsive behavior.

**Step 3: Add JS handlers and state**

- In `desktop.js`, add methods near existing network handlers:
  - `initMcpManager()`
  - `loadMcpConnections()`
  - `renderMcpConnections()`
  - `saveMcpConnection()`
  - `deleteMcpConnection(id)`
  - `testMcpConnection(id)`
  - `populateMcpForm(connection)`

**Step 4: Backend command wiring in JS**

- Use `invoke('get_mcp_connections')`, `invoke('save_mcp_connection', ...)`, `invoke('delete_mcp_connection', ...)`, `invoke('test_mcp_connection', ...)`.

### Task 5: Validate persistence and startup reload

**Files:**
- Modify: `THIS-PROJECTS-CURRENT-STATE.md`
- Modify: `frontend/desktop.js`

**Step 1: Load and render on startup**

- Call `loadMcpConnections()` during desktop initialization after current network config call.

**Step 2: Ensure reload after save/delete/test updates**

- Re-fetch connections after each successful mutation to keep UI consistent.

**Step 3: Manual restart check**

- Save entry → restart Arandu executable → confirm MCP list restores and remains valid.

### Task 6: Documentation and state update

**Files:**
- Modify: `docs/knowledge-base/` reference entries if path/style changed
- Modify: `THIS-PROJECTS-CURRENT-STATE.md`

**Step 1: Update current-state file**

- Add implemented milestone block and date.
- Keep Future Work section for remaining work and current blockers.

**Step 2: Record any known caveats**

- Call out that MCP tool execution is explicitly deferred to phase 2.

### Task 7: Build and integration verification

**Commands:**
- `cd backend && cargo check`
- `cd backend && cargo test`
- `cargo tauri build --no-bundle`
- (optional) `cargo tauri build` after version metadata blocker is resolved

**Step 1: Run backend checks first**

- Execute checks from backend dir and verify no compile breaks from new types/commands.

**Step 2: Run lightweight UI smoke checks**

- Start app, open desktop panel, create one SSE and one stdio connection, test both, then reopen app to validate persistence.

### Task 8: Commit granularity

**Commit sequence (2-5 minutes each):**
1. Backend models/config + commands
2. Test logic
3. Frontend UI + styles
4. End-to-end verification + docs updates

### Execution options

Plan complete and saved to `docs/plans/2026-02-24-mcp-integration-implementation-plan.md`.

**1. Subagent-Driven (this session)**
- Use `superpowers:subagent-driven-development` and execute task-by-task with code review checkpoints.

**2. Parallel Session (separate worktree/session)**
- Open a new execution session and use `superpowers:executing-plans`.
