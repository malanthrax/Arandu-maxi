# MCP Runtime Tool Usage Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Make configured MCP tools actually executable during chat (not only visible in injected context), with the smallest shippable architecture.

**Architecture:** Keep llama-custom as the orchestrator for a bounded tool-calling loop, add one parent bridge message for MCP tool execution, and add one backend Tauri command that performs MCP `tools/call` against enabled connections. Keep scope intentionally narrow: JSON/HTTP/streamable_http + stdio, single-threaded loop, bounded retries, explicit unsupported handling for SSE.

**Tech Stack:** Tauri (Rust), vanilla JS iframe/parent messaging, llama.cpp OpenAI-compatible `/v1/chat/completions`, MCP JSON-RPC.

---

## Phase 0 - Checkpoint (Already Done)

Status:
- Baseline MCP visibility path exists and is confirmed: iframe `request-mcp-context` -> parent `mcp-context` -> system message injection.
- Existing checkpoint planning artifact exists at `docs/plans/2026-03-06-mcp-runtime-tool-usage-plan.md`.

No additional code work in this phase.

---

## Phase 1 - Investigation Findings (Concrete Inputs for Build)

1. Current gap
- `frontend/llama-custom/index.html` injects MCP metadata, but does not send `tools[]` definitions in chat requests and does not execute `tool_calls` returned by the model.

2. Existing reusable pieces
- `frontend/modules/terminal-manager.js` already handles iframe <-> parent request/response patterns (`request-current-config`, `request-mcp-context`, etc.), so MCP tool execution can reuse this same message contract style.
- `backend/src/lib.rs` already has MCP transport-aware helpers for initialize/tool listing and per-connection validation; this can be extended for `tools/call` instead of building a separate service.
- `backend/src/models.rs` already stores tool schemas (`McpToolInfo.input_schema`) needed to generate OpenAI-style `tools` definitions.

3. Runtime constraints to honor
- Keep tool loop bounded (max iterations + timeout) to avoid runaway calls.
- Keep streaming behavior simple for the first shippable release: when MCP tools are enabled for a request, run non-streaming completion for deterministic `tool_calls` handling.
- Preserve existing no-MCP path (current streaming UX and metrics) unchanged.

---

## Phase 2 - Implementation Tasks (Minimal Shippable Scope)

### Task 1: Add backend MCP tool-call command

**Files likely to change:**
- Modify: `backend/src/models.rs`
- Modify: `backend/src/lib.rs`

**Work:**
- Add request/response structs for tool execution (example naming):
  - `McpToolCallRequest` (`connection_id`, `tool_name`, `arguments` as `serde_json::Value`)
  - `McpToolCallResult` (`success`, `content`, `is_error`, `raw_result`, `error`, `latency_ms`)
- Add a new `#[tauri::command]` in `backend/src/lib.rs` (example: `call_mcp_tool`) that:
  - Locates connection by `id` and confirms `enabled`.
  - Executes MCP initialize + `tools/call` using connection transport settings.
  - Supports: `stdio`, `http`, `json`, `streamable_http`.
  - Returns clear unsupported error for `sse` in this phase.
  - Applies configured headers/env/args/timeouts.
- Register command in Tauri invoke handler list in `backend/src/lib.rs`.

### Task 2: Add parent bridge for iframe MCP tool execution

**Files likely to change:**
- Modify: `frontend/modules/terminal-manager.js`

**Work:**
- Add new message op handling:
  - Request: `request-mcp-tool-call`
  - Response: `mcp-tool-call-result`
- Bridge payload to backend via `invoke('call_mcp_tool', ...)`.
- Validate required fields (`connectionId`, `toolName`, `arguments`) and return structured errors without throwing UI-breaking exceptions.
- Include `request_id` passthrough so iframe can match async responses deterministically.

### Task 3: Add tool-calling loop in chat iframe

**Files likely to change:**
- Modify: `frontend/llama-custom/index.html`

**Work:**
- Build OpenAI-style `tools` array from enabled MCP connections + discovered tools from `mcpContextState.connections`.
- Add tool name mapping strategy (stable and reversible), e.g. `mcp__{connectionId}__{toolName}`.
- Add bounded execution loop in `sendMessage()` MCP path:
  1. Build messages + `tools`.
  2. Call `/v1/chat/completions` with `stream: false` when tools are present.
  3. If assistant returns `tool_calls`, execute each call via parent bridge.
  4. Append assistant tool-call message + `role: tool` results back into messages.
  5. Repeat until assistant returns normal content or max loop reached.
- Keep existing streaming path untouched when no MCP tools are active.
- Add clear system-message fallback on tool failure (`tool unavailable`, `invalid args`, timeout, unsupported transport).

### Task 4: Lightweight observability for acceptance

**Files likely to change:**
- Modify: `frontend/llama-custom/index.html`
- Modify: `frontend/modules/terminal-manager.js`

**Work:**
- Add concise debug logs/status updates for:
  - tools sent to model
  - tool_calls received
  - tool execution success/failure
  - loop stop reason (content complete, max loop, error)
- Keep logs low-noise and avoid secret/header/env value leakage.

Out-of-scope in this plan:
- Parallel tool execution optimizations
- Streaming tool-call delta reassembly
- SSE transport tool execution
- Refactoring MCP logic into new backend modules

---

## Phase 3 - Verification Tasks

### A. Static/build verification

Run from repo root:

```bash
cargo check --manifest-path backend/Cargo.toml
```

```bash
node --check frontend/modules/terminal-manager.js
```

### B. Targeted backend tests (add and run with implementation)

Run:

```bash
cargo test --manifest-path backend/Cargo.toml mcp
```

Expected:
- New tests cover tool-call request validation and transport routing behavior.
- Existing MCP tests remain passing.

### C. Manual runtime verification (critical)

1. Configure one enabled MCP connection with at least one callable tool.
2. Open chat window (`frontend/llama-custom/index.html` embedded flow).
3. Prompt model with an instruction that requires that tool.
4. Confirm sequence in logs/UI:
   - MCP tools included in request
   - model emits `tool_calls`
   - bridge executes backend `call_mcp_tool`
   - follow-up completion uses tool result
5. Confirm assistant response materially reflects tool output.
6. Repeat with a forced failure case (bad args or disabled connection) and confirm graceful error message without chat lockup.

### D. Regression check (no MCP)

1. Disable all MCP connections.
2. Send normal prompt.
3. Confirm existing streaming response path and token stats still work.

---

## Phase 4 - Acceptance Criteria

Ship is acceptable only when all are true:

1. Functional tool usage
- At least one enabled MCP tool is actually invoked during chat through model `tool_calls` + bridge + backend execution.

2. Minimal architecture preserved
- Implementation is limited to:
  - `frontend/llama-custom/index.html` (loop orchestration)
  - `frontend/modules/terminal-manager.js` (bridge)
  - `backend/src/lib.rs` + `backend/src/models.rs` (command + types)

3. Safe fallback behavior
- Unsupported transport or tool execution errors do not crash chat; user gets a clear failure result and can continue conversation.

4. Verification evidence
- Commands in Phase 3 pass.
- Manual verification demonstrates one successful MCP-assisted answer and one failure-path fallback.

5. No scope creep
- No large refactors, no new protocol layers, no unrelated UI redesigns.
