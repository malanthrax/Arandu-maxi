# MCP Runtime Tool Usage Implementation Plan

> **Status:** Active planning/execution phase (2026-03-06)

## Goal

Make user-configured MCP connections not only visible to chat models, but actually usable by model workflows during chat completion.

## Phase Timeline

1. Checkpoint commit before any new MCP runtime behavior changes.
2. Deep investigation of current MCP flow and root-cause gap between metadata visibility and executable tool usage.
3. External research/validation against MCP and OpenAI-compatible tool-calling patterns.
4. Expert-style review of candidate architecture and simplicity trade-offs.
5. Subagent implementation of approved minimal architecture.
6. Independent subagent verification (behavioral + technical checks).
7. Final acceptance verification against user requirements.

## Current Baseline

- Chat requests MCP context from parent via `request-mcp-context`.
- Parent replies with MCP connection/tool metadata via `mcp-context`.
- Chat injects MCP summary into system messages before `/v1/chat/completions`.
- No confirmed execution bridge yet that runs MCP tools based on model tool calls.

## Acceptance Criteria

- A connected/enabled MCP tool can be selected/used by the model in normal chat flow.
- Tool invocation path is observable in logs/debug status.
- Failure paths degrade clearly without breaking normal chat.
- Change remains minimal and maintainable.
