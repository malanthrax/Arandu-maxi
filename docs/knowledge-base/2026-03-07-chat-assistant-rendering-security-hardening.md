# 2026-03-07 - Chat assistant rendering security hardening

## Summary

- Hardened assistant output rendering in `frontend/llama-custom/index.html` to avoid inserting untrusted model text as raw HTML.
- Introduced centralized safe render utilities for text + newline rendering via DOM APIs.
- Updated both streaming and non-streaming assistant output paths to use safe rendering.
- Updated assistant stats rendering to DOM-based construction for affected message render flow.

## Why this was needed

- Prior rendering path used `innerHTML` for assistant model output.
- Model output is untrusted input and should not be interpreted as executable HTML/JS.

## Files changed

- `frontend/llama-custom/index.html`
- `docs/CURRENT_WORKING_STATE.md`
- `THIS-PROJECTS-CURRENT-STATE.md`
- `scratchpad.md`

## Verification

- Inline script syntax sanity for `frontend/llama-custom/index.html` using extracted `<script>` content compiled in Node VM.
- Result: `inline_scripts_ok 1` (PASS)
- Backend verification not required for this change set (frontend/docs only).

## Notes

- HTML-like text from model responses is now rendered literally as text for safety.
- Existing newline display and draft-token highlighting behavior were preserved.

---

## Follow-up update (2026-03-07) - System Prompt Override (frontend)

### Scope

- Added a global System Prompt Override flow with frontend-only edits.
- New top-right light-blue button opens `System Prompt Override` manager window.
- Manager supports name + prompt text + save + dropdown selection.
- Dropdown default option is always `Default` and injects nothing.

### Files changed

- `frontend/index.html`
- `frontend/css/desktop.css`
- `frontend/desktop.js`
- `frontend/modules/terminal-manager.js`
- `frontend/llama-custom/index.html`
- `scratchpad.md`

### Precedence implemented

1. Typed chat Model Options `System Prompt` value
2. Selected saved global override prompt
3. Existing/default behavior (no injection)

### Integration behavior

- Saved prompts + selected entry are persisted in localStorage.
- Terminal manager now sends global override metadata to chat iframe in `current-config`.
- Chat iframe also listens for live override changes via postMessage broadcast.
- Best-effort launch override applies `--system-prompt` when effective override exists.

### Verification

- `node --check frontend/desktop.js` -> `desktop_js_check:PASS`
- `node --check frontend/modules/terminal-manager.js` -> `terminal_manager_check:PASS`
- Inline script parse/compile for `frontend/llama-custom/index.html` -> `inline_scripts_ok 1`
