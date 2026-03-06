# 2026-03-06 MCP Context Injection Disabled

## What changed

- Disabled MCP system-context injection in chat request assembly.
- File: frontend/llama-custom/index.html
- Removed buildMcpSystemContextMessage() push into request messages.

## Why

- Keep chat startup context equivalent to normal chat behavior.
- Avoid context pollution for non-MCP discussions.
- Ensure MCP functionality is validated through true tool-calling flow only.

## Current behavior

- Chat starts with normal context only (user options/system prompt/session memory behavior).
- MCP tool availability is exposed via standard tools payload.
- Tool execution occurs only when model emits tool_calls, routed through backend call_mcp_tool.
