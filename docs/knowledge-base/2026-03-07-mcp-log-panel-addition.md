# 2026-03-07 MCP Log Panel Addition

## Scope

- Added an MCP log entry point in chat UI under Benchmark Log.
- Added a dedicated MCP log panel for tool-call traffic visibility.
- Wired MCP loop logging around tool-call dispatch and tool-call results.

## Files Changed

- `frontend/llama-custom/index.html`
- `scratchpad.md`
- `THIS-PROJECTS-CURRENT-STATE.md`
- `docs/CURRENT_WORKING_STATE.md`

## MCP Log UI

- New floating toggle `mcpLogToggle` under benchmark toggle.
- New panel `mcpLogPanel` with `Clear` and `Close` controls.
- Columns: `Time`, `Direction`, `Tool`, `Status`, `Details`.
- Styling follows existing benchmark/debug panel language.

## Data Captured

- Outbound (MODEL -> MCP): timestamp, mapped connection/tool, arguments snapshot.
- Inbound (MCP -> MODEL): timestamp, mapped connection/tool, success/failure, result snippet.

## Storage + Safety

- Bounded history: `MCP_LOG_LIMIT = 150`.
- localStorage key: `aranduMcpLogV1`.
- Rendered newest-first.
- Rows rendered via DOM + `textContent` (no untrusted raw HTML).

## Integration Points

- Hooked into MCP loop near `toolCalls` handling in `sendMessage()`.
- Outbound logged before `requestMcpToolCall(...)`.
- Inbound logged for success/failure results and mapping failures.

## Verification

- Inline script compile sanity via Node VM extraction passed: `inline_scripts_ok 1`.

## Follow-up Fix: Pre-tool Failures Captured

- Added synthetic MCP log entries for MCP-loop chat-completion lifecycle with tool name `__chat_completion__`.
- Outbound entry now logs each MCP-loop `/v1/chat/completions` request (iteration/message/tool counts).
- Inbound entry now logs HTTP status outcome and failure body snippet for non-OK responses (including 500 errors).
- Added guard to avoid duplicate fallback error entries when a failure was already logged in-loop.

## Follow-up Update: Configurable MCP Loop Limit

- Added runtime `MCP Tool Loops` numeric parameter in `frontend/llama-custom/index.html` Model Options.
- Input range is constrained to `1..20` (default `6`) with explicit UI clamp on change.
- MCP execution path now uses this parameter instead of a hardcoded loop count.
- Execution path also re-clamps the value to `1..20` before each run so devtools/local-state tampering cannot exceed `20`.
