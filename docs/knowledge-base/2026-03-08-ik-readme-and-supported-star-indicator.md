# IK Readme Button + Supported Star Indicator (2026-03-08)

## Summary

- Added a new top-right `Readme` button in desktop controls to open IK-supported model family reference.
- Added a reusable IK family list constant in `frontend/desktop.js` and normalized matching helpers.
- Local and remote model titles now show a gold star when they match IK-supported families.
- Saved canonical list at repo root in `ikllama.cpp supported models.md`.

## Files Changed

- `frontend/index.html`
  - Added `view-ik-readme-btn` beside existing view toggle buttons.
- `frontend/css/desktop.css`
  - Added `view-ik-readme-btn` styling.
  - Added `ik-supported-models-window` + content/list styles.
  - Added `.ikllama-star` gold star styling for model titles.
- `frontend/desktop.js`
  - Added reusable constants:
    - `IKLLAMA_SUPPORTED_FAMILIES`
    - `IKLLAMA_MATCHER_ALIASES`
  - Added matching helpers:
    - `normalizeIkMatcherText()`
    - `buildIkSupportedFamilyMatchers()`
    - `isIkLlamaSupportedModel()`
    - `buildModelTitleHtml()`
  - Added Readme window action:
    - `openIkSupportedModelsWindow()`
  - Wired button listener in `initViewToggle()`.
  - Applied star-title rendering in:
    - local list view
    - local card view
    - remote list view
- `ikllama.cpp supported models.md`
  - Added exact IK-supported family list content (canonical source copy).
- `THIS-PROJECTS-CURRENT-STATE.md`
- `docs/CURRENT_WORKING_STATE.md`
- `scratchpad.md`

## Verification

- Command run:
  - `node --check frontend/desktop.js`
- Result:
  - pass (no syntax errors)
