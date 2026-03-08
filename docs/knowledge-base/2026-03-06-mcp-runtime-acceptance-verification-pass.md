# 2026-03-06 MCP Runtime Acceptance Verification Pass

## Objective
Verify MCP runtime integration is in place so models can use MCP tools, not only see metadata.

## Verification Performed
- cargo check manifest-path backend/Cargo.toml: PASS
- node check frontend/modules/terminal-manager.js: PASS
- Code-path verified: backend call_mcp_tool command, parent bridge request-mcp-tool-call to mcp-tool-call-result, and chat tool loop in llama-custom index.
- Transport gating verified: only http/json/streamable_http are advertised as callable tools; sse/stdio are skipped and unsupported for execution.
- Live MCP endpoint sanity check on local nowledge MCP (127.0.0.1:14242/mcp): initialize 200, tools/list 200, tools/call 200.

## End-to-End Runtime Demonstration (Completed)

Executed a live tool-calling demonstration in this environment:
- Started temporary llama-server with `--jinja` on `127.0.0.1:8080`.
- First completion request included tools and returned `finish_reason: tool_calls`.
- Executed real MCP `tools/call` against local nowledge endpoint (`memory_search`).
- Sent follow-up completion with assistant tool call + `role: tool` output.
- Second completion returned `finish_reason: stop` with final answer using tool-derived content.

## Current Acceptance Status
- Implementation acceptance: PASS
- Environment runtime acceptance: PASS

## Optional UI Confirmation
1. Run one prompt in Arandu chat UI and visually confirm MCP debug status updates during tool execution.

## 2026-03-07 Addendum - Visibility + Reliability Follow-up

### MCP Visibility Improvements
- Parent bridge now refreshes MCP tools at context handoff by invoking `list_mcp_tools` per enabled connection in `frontend/modules/terminal-manager.js`.
- Chat now shows:
  - wrench chip with callable tool count near active model label,
  - clickable panel listing exact `Connection :: ToolName` entries sent in `tools[]`.
- MCP status text now includes refresh failure count (`... / X refresh failed`) for faster diagnostics.

### Chat History Reliability Improvements
- Fixed the "delete works once then stops" issue by centralizing interaction lock lifecycle in chat history UI.
- Added guarded debounce and lock-aware search behavior to avoid overlapping list/delete/load actions.

### Runtime/Parser Hardening (same phase)
- Streamable/SSE parsing path uses loss-tolerant byte decode before JSON/SSE parse.
- Windows stdio MCP command execution includes npm-style `.cmd` shim fallback when PATH lookup fails.

### Verification (2026-03-07)
- cargo check manifest-path backend/Cargo.toml: PASS
- node check frontend/modules/terminal-manager.js: PASS
- inline script parse sanity for frontend/llama-custom/index.html: PASS
- cargo tauri build no-bundle: PASS
- artifact: backend/target/release/Arandu.exe

## 2026-03-07 Backup Addendum - Native Supermemory + Key Lifecycle + Streaming/Context Updates

### Native Supermemory integration (non-MCP-proxy execution)
- Added backend native command `call_supermemory_native_tool` with 4 tools:
  - `supermemory_search`
  - `supermemory_add_memory`
  - `supermemory_profile`
  - `supermemory_configure_settings`
- Parent bridge now injects synthetic enabled connection `native-supermemory` with those 4 tool schemas.
- Chat routes `native-supermemory` tool calls through native backend command.
- Added 45s timeout for native Supermemory API requests.

### Supermemory operational UX
- Chat retains Supermemory toggle and now uses parent-side storage sync.
- Model Options now include:
  - `New Supermemory API Key`
  - `Delete Supermemory API Key`

### Chat and runtime safeguards added in same phase
- Stream toggle added and persisted (global + per-model).
- Context counter now has fallback estimate when slot metrics unavailable.
- Slot-compression fallback: summary + reset when slot endpoints are unsupported.
- Tool ordering prioritizes `native-supermemory`.
- Update (2026-03-07 later pass): MCP tool cap/schema compaction was rolled back temporarily; tools are currently sent in full form for compatibility testing.

### Security hardening
- Parent message bridge now only handles sensitive iframe requests when source window maps to a known terminal iframe.

### Additional verification snapshot
- node --check frontend/modules/terminal-manager.js: PASS
- node --check frontend/desktop.js: PASS
- inline script parse sanity for frontend/llama-custom/index.html: PASS
- cargo check manifest-path backend/Cargo.toml: PASS
- cargo tauri build no-bundle: PASS
