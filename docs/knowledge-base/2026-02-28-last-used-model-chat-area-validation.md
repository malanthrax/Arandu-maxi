# 2026-02-28 - Last-Used Model + Chat Area Validation

## Outcome

- The `Launch Last Used Model` feature is now functioning with launch-mode persistence across:
  - default launch
  - half-context launch
  - preset launch
  - external launch
  - preset-external launch
- Stale preset IDs are handled gracefully:
  - preset mode falls back to default launch when preset is missing,
  - preset-external mode falls back to external launch when preset is missing.
- Chat area LLM execution is currently confirmed as working in the built app after the latest changes.

## What changed

- Updated `frontend/desktop.js` launch flow to persist and reuse launch mode (`launchMode`, `presetId`) via existing `setLastUsedModel` state.
- Updated button availability refresh behavior to recalc after desktop icon refresh.

## Verification commands

- `node --check frontend/desktop.js` (syntax)
- `cargo tauri build --no-bundle` from `backend/`
- Artifact check: `backend\\target\\release\\Arandu.exe`

## Notes

- Manual in-app verification was done via built executable interaction and reported as working for both last-used launch mode replay and chat LLM usage.
