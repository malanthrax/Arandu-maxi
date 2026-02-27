# MCP Integration Design - 2026-02-24

## Objective

Add first-phase MCP server management to Arandu so users can save and test MCP connections directly from the desktop UI near the existing Network Serve area.

## Non-functional constraints

- Scope stays to connection management only in this phase:
  - add connection entry
  - edit connection entry
  - delete connection entry
  - test connection
  - persist list in app settings
- Do not implement MCP tool invocation into chat yet.
- Follow existing save/load patterns used by network config and model settings.
- Keep build blocker work (`cargo tauri build` version issue) tracked in `THIS-PROJECTS-CURRENT-STATE.md`.

## Chosen approach

### Recommendation (approved)

Use a backend-owned MCP registry in `GlobalConfig` with Tauri CRUD commands and a dedicated desktop UI panel integrated with the Network widget.

### Why this approach

- Consistent with existing app persistence model.
- Easy to reuse restart-safe serialization and config migration patterns.
- Gives a stable API surface for future phases (tool discovery/execution).

## Architecture

### Data model

Add a new MCP config type in `backend/src/models.rs`:

- `McpTransport`: enum with `Stdio`, `SSE`, `HTTP`, `StreamableHttp`
- `McpConnection`: struct for one entry with
  - `id: String`
  - `name: String`
  - `transport: McpTransport`
  - `enabled: bool`
  - `command: String` (for stdio launch)
  - `args: Vec<String>`
  - `url: String` (for SSE/HTTP)
  - `env_vars: HashMap<String, String>`
  - `headers: HashMap<String, String>`
  - `timeout_seconds: u64`
  - `last_test_at: Option<String>`
  - `last_test_status: Option<String>`

Add `mcp_servers: Vec<McpConnection>` to `GlobalConfig` with default `Vec::new()` for backwards compatibility.

### Backend commands

Add Tauri commands in `backend/src/lib.rs`:

1. `get_mcp_connections() -> Vec<McpConnection>`
2. `save_mcp_connection(connection: McpConnection) -> Result<String, String>`
   - insert/update by `id`
3. `delete_mcp_connection(id: String) -> Result<(), String>`
4. `toggle_mcp_connection(id: String, enabled: bool) -> Result<(), String>`
5. `test_mcp_connection(id: String) -> serde_json::Value`
   - stdio: spawn command short-lived, capture process start within timeout
   - http/sse: connect probe to configured URL and verify HTTP response path reachable
   - return `{ success, message, latency_ms, status_code?, error? }`

Command pattern should mirror existing `save_network_config`/`get_network_config`:

- mutate shared config state under lock
- persist via `save_settings(&state).await`
- return explicit errors

### Test behavior

- StdI/O transport test:
  - run process with parsed args
  - wait for process start + optional short grace period
  - if process exits immediately, include exit code in diagnostics
  - terminate process after success test window
- SSE/HTTP/Streamable tests:
  - perform async request to target URL
  - timeout using configured seconds
  - surface status/message details for UI

### UI integration (frontend)

Keep all MCP controls inside `frontend/index.html` and `frontend/desktop.js`, reusing the Network widget container pattern:

- Add a second section under the existing Network panel:
  - MCP server list
  - per-server status dot / last test text
  - buttons: Test, Edit, Delete
  - Add New + fields for transport, name, URL/command, args, env, headers
- `frontend/modules` additions:
  - either keep logic inside `desktop.js` or add new `frontend/modules/mcp-manager.js` if cleaner.
- Add small validation in UI before calling backend:
  - URL required for HTTP/SSE
  - command required for stdio
  - port/address sanity checks for URL fields

### State and persistence

- Store MCP entries in config settings JSON under same `%USERPROFILE%\.Arandu\settings.json` path.
- Load MCP entries once during desktop initialization and refresh after save/delete/test updates.
- Use config defaults for migration: missing field should be `[]`.

### User experience

- `network-widget-popup` should show MCP section by default collapsed/expandable.
- Save + test operations should show toast + inline row status.
- Failed connection tests should retain the existing entry and show actionable message.

### Error handling

- All backend errors return clear `success: false` and `message` for `test`.
- Frontend must handle command invocation failures consistently with existing pattern in `activateNetworkServer`.
- Ensure no UI hard-crash if process spawn fails.

## File list (expected)

- `backend/src/models.rs` (new MCP types + config field)
- `backend/src/lib.rs` (new commands + invoke registration)
- `frontend/index.html` (MCP section markup)
- `frontend/css/desktop.css` (new MCP styles)
- `frontend/desktop.js` (MCP state + handlers + backend invocation)
- `docs/knowledge-base` + `THIS-PROJECTS-CURRENT-STATE.md` updates when milestones complete

## Milestones for implementation

1. Backend schema + commands
2. Frontend MCP list/create/edit/delete flow
3. Test action + result rendering
4. Persisted load/reload path verification
5. Manual pass: create + edit + test + restart app to ensure persistence
6. Update `THIS-PROJECTS-CURRENT-STATE.md` with implemented milestone

## Out of scope

- MCP tool discovery and runtime execution
- OpenAI/Chat pipeline integration in this phase

## Open questions

- Should env/headers editing support raw JSON or key-value grid first?
- Should MCP servers be auto-disabled when their test fails repeatedly?
- Should network widget show one combined “Service is active” state for both proxy and MCP?
