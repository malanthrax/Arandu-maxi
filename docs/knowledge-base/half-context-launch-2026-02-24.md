# Half-Context Launch Launch-Time Override (2026-02-24)

## Session context

- Added a temporary one-shot launch path that starts a model with `--context-shift` without writing it to persisted `ModelConfig.custom_args`.
- Implemented on both frontend and backend while preserving existing launch paths and preset behavior.
- Kept the MCP popup and existing MCP command flow unchanged.

## Verified changes

- Backend: added `launch_model_with_half_context` in `backend/src/lib.rs`.
  - New helper: `append_half_context_arg` avoids duplicating `--context-shift` if already present.
  - Command temporarily overrides in-memory `custom_args`, launches via `launch_model_server`, restores original args.
  - Registered command in Tauri invoke handler list.
- Frontend: updated `frontend/desktop.js`.
  - Context menu now includes `Launch with half context` action (`data-action="launch-half-context"`, text: `Load with half context`).
  - Added `launchModelWithHalfContext` dispatch path that:
    - computes launch args as saved args + `--context-shift`,
    - checks terminal availability,
    - calls new backend command,
    - opens terminal window with per-launch `launchArgs` via `openServerTerminal(..., launchArgs)`.
  - Existing `open` / `launch` / `launch-external` / preset actions remain untouched.

## Evidence

- Backend check: `cargo check --manifest-path backend/Cargo.toml` passed.
- Frontend syntax checks: `node --check frontend/desktop.js` and `node --check frontend/modules/terminal-manager.js` passed.
