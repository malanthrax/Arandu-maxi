# 2026-03-07 - MCP stdio window suppression on Windows

## Problem

- When MCP tools were invoked over `stdio`, Windows console terminals could pop to foreground and cover the screen.

## Fix

- Updated MCP stdio execution path in `backend/src/lib.rs` (`execute_stdio_mcp_request`) to launch child processes with hidden window flags on Windows.
- Applied `creation_flags(0x08000000)` (`CREATE_NO_WINDOW`) for all stdio spawn branches:
  - direct configured command launch
  - command-style shell fallback (`cmd /C`)
  - Windows `.cmd` shim fallback

## Files changed

- `backend/src/lib.rs`
- `THIS-PROJECTS-CURRENT-STATE.md`
- `docs/CURRENT_WORKING_STATE.md`
- `scratchpad.md`

## Verification

- `cargo check --manifest-path backend/Cargo.toml` passed.

## Impact

- MCP stdio tool calls should run without visible terminal popups on Windows while preserving existing fallback behavior.
