# 2026-03-07 - System Prompt Override UI and precedence

## Summary

- Added a new top-right light-blue `System Prompt Override` button in desktop controls.
- Added a `System Prompt Override` manager window with:
  - saved prompt dropdown,
  - prompt name input,
  - prompt text input,
  - save action.
- Added persistence for prompt entries and selected prompt in localStorage.

## Default behavior

- Dropdown first option is always `Default`.
- `Default` injects nothing and makes no system-prompt override changes.

## Precedence model

- Effective system prompt precedence in chat request path:
  1. Typed `System Prompt` in chat Model Options (`currentParams.system_prompt`).
  2. Selected global System Prompt Override prompt.
  3. Default/no system prompt injection.

## Launch-arg integration (best effort)

- Terminal manager now applies/removes `--system-prompt` during launch/restart argument handling based on effective precedence.
- Default/no-override state now strips stale `--system-prompt` flags from args to preserve no-injection semantics.

## Files changed

- `frontend/index.html`
- `frontend/css/desktop.css`
- `frontend/desktop.js`
- `frontend/modules/terminal-manager.js`
- `frontend/llama-custom/index.html`
- `THIS-PROJECTS-CURRENT-STATE.md`
- `docs/CURRENT_WORKING_STATE.md`
- `scratchpad.md`

## Verification

- `node --check frontend/desktop.js` passed.
- `node --check frontend/modules/terminal-manager.js` passed.
- Inline script parse sanity for `frontend/llama-custom/index.html` passed: `inline_scripts_ok 1`.

## Notes

- Live open chat windows receive parent broadcast updates when global override selection changes.
- Existing running servers are not force-restarted by selection change alone; launch arg override applies on launch/restart flow.

## 2026-03-07 Follow-up: `New` button in manager

- Added `New` action button beside `Clear Editor` and `Save Prompt` in System Prompt Override window.
- `New` calls `prepareNewSystemPromptOverrideEntry()` in `frontend/desktop.js`.
- Behavior on click:
  - set selected prompt to `Default`,
  - clear prompt name + prompt text fields,
  - focus prompt name input for fast new-entry creation.
- Existing save/update semantics are preserved (case-insensitive name match updates existing; new names create new entries).
- Styling updated in `frontend/css/desktop.css` with `.system-prompt-override-new` to keep action row visuals consistent.

### Follow-up verification

- `node --check frontend/desktop.js` passed (no output).

## 2026-03-07 Follow-up: always-on current date/time system injection

- Added `buildCurrentDateTimeSystemPrompt()` in `frontend/llama-custom/index.html`.
- Chat request assembly now always injects a system message from the current system clock before all optional system prompts.
- Injected date/time payload includes:
  - local date,
  - local time,
  - timezone (IANA name where available) + UTC offset,
  - UTC ISO timestamp.
- Existing optional system prompt precedence remains unchanged:
  - typed `System Prompt` > selected global override > none.

### Follow-up verification

- Inline script parse/compile check for `frontend/llama-custom/index.html` passed: `inline_scripts_ok 1`.
