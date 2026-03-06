# 2026-03-06 MCP Tool Execution Bridge Implementation

## Scope completed

- Added backend MCP tool-call request/response models in backend/src/models.rs:
  - McpToolCallRequest
  - McpToolCallResult
- Added new Tauri command call_mcp_tool in backend/src/lib.rs and registered it in the invoke handler list.
- Implemented MCP tool execution for http, json, streamable_http, sse, and stdio transports:
  - resolve_mcp_url
  - default_mcp_initialize_payload
  - post_mcp_request
- Added SSE response parsing support (SSE `data:` payload parsing) so tool discovery/calls can parse event-stream responses.
- Added stdio MCP execution path for `tools/list` and `tools/call` with initialize + initialized notification + bounded response wait.

## Frontend bridge

- Added parent bridge handling in frontend/modules/terminal-manager.js:
  - request: request-mcp-tool-call
  - response: mcp-tool-call-result
- Preserves request_id passthrough and validates connectionId, toolName, and arguments.

## Chat loop changes

- Added minimal bounded MCP tool-calling loop in frontend/llama-custom/index.html:
  - Builds OpenAI tools array from enabled MCP context connections/tools.
  - Uses non-streaming /v1/chat/completions path only when MCP tools are present.
  - Executes returned tool_calls through parent bridge and appends role: tool outputs.
  - Loops until final assistant content or max-iteration guard.
  - Provides clear fallback text on loop/tool failures.
- Kept existing no-MCP streaming chat path unchanged.
- Added concise MCP debug status updates for tools sent, tool calls received, and loop stop reason.
- Updated tool transport gating to include callable transports: `http`, `json`, `streamable_http`, `sse`, `stdio`.

## Verification

- cargo check --manifest-path backend/Cargo.toml -> pass
- node --check frontend/modules/terminal-manager.js -> pass
