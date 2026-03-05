# 2026-03-04 - In-chat active model label and live model switcher

## Scope

Implemented in-chat model awareness and switching in the custom chat iframe, with parent-orchestrated execution in TerminalManager.

## Files changed

- `frontend/llama-custom/index.html`
- `frontend/modules/terminal-manager.js`
- `docs/plans/2026-03-04-in-chat-active-model-switcher-implementation.md`
- `docs/CURRENT_WORKING_STATE.md`
- `THIS-PROJECTS-CURRENT-STATE.md`

## Feature behavior

### Chat UI

- Added active model button near Send:
  - `#activeModelSwitcherButton`
  - label element `#activeModelSwitcherLabel`
- Added switcher panel:
  - `#activeModelSwitcherPanel`
  - local list `#activeModelListLocal`
  - remote list `#activeModelListRemote`

### Message protocol

- Child -> Parent:
  - `request-chat-model-switcher-data`
  - `request-chat-model-switch`
- Parent -> Child:
  - `chat-model-switcher-data`
  - `chat-model-switch-result`
  - `chat-active-model-changed`

### Switching semantics

- Local target:
  - Updates terminal model identity
  - Calls restart path in the same server terminal
  - Emits active-model sync event back to iframe
  - Updates chat persistence model path (`currentModelPath`)
- Remote target:
  - Uses existing `openNativeChatForServer(...)`
  - Opens remote chat through remote launch flow
  - Keeps current embedded chat active and unchanged

## Reliability updates

- Switch request resolution now requires exact iframe-source terminal match for switching.
- Added request watchdog timeouts in chat UI:
  - inventory load timeout (8s)
  - switch timeout (20s)
- Timeout failures reset state and post user-facing system messages.

## Subagent review + fixes

A code-review subagent was run and flagged correctness risks.
Applied fixes:

- Prevented remote success from overriding embedded chat active-model identity.
- Updated `currentModelPath` on local model changes.
- Removed switch fallback to active-window context (exact iframe source required).
- Added timeout recovery for loading/switch states.

## Verification

Commands run in canonical workspace `H:\Ardanu Fix\Arandu-maxi`:

- `node --check frontend/modules/terminal-manager.js` -> pass
- `cargo check --manifest-path backend/Cargo.toml` -> pass
- `cargo tauri build --no-bundle` (from `backend/`) -> pass
- `stat backend/target/release/Arandu.exe`
  - Modify: `2026-03-04 16:11:21 -0800`

## Artifact

- `backend/target/release/Arandu.exe` rebuilt successfully.
