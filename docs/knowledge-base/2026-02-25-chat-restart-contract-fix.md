# 2026-02-25 Chat restart-contract UX fix

## Scope
- File: `frontend/modules/terminal-manager.js`
- File: `frontend/llama-custom/index.html`

## Change
- In restart-required flows from the chat iframe, `request-restart` handling now always sends a `settings-saved` message back to the iframe with a machine-readable flag `restartTriggered: true`.
- The chat UI now treats this reply as a restart-complete signal:
  - Resets the parameter baseline (`launchConfigBaseline`) so change detection stays in sync.
  - Clears the connection loading state (`connectionStatus` text/class).
  - Continues showing the server status message to confirm the action.

## Why
- Parent-side currently emitted success messages only for non-restart saves, leaving restart-required flows with weaker feedback and stale restart-impact baseline state.
- This patch hardens the restart contract so the UI and terminal state remain aligned after an in-iframe restart request.

## Evidence
- Rebuilt executable after this work:
  - `backend/target/release/Arandu.exe`
- Manual verification command used: `node --check frontend/modules/terminal-manager.js`
- Additional lightweight check: `node frontend/chat-stream-smoke.mjs`
- Files modified in working tree:
  - `frontend/modules/terminal-manager.js`
  - `frontend/llama-custom/index.html`
