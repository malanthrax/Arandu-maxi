# 2026-03-01 - Remote chat window centering + direct launch reliability fix

## Issues addressed

1. Remote chat windows opened in upper-left instead of true centered placement.
2. Remote model double-click launch path was inconsistent and could fail while right-click "Open Remote Chat" worked.
3. Remote chat popup size felt too small for modern usage.

## Root causes

- `createWindow()` centers windows using `left: 50%`, `top: 50%`, `transform: translate(-50%, -50%)`.
- `openNativeChatForServerSuccess()` later overwrote `left/top` with pixel values but did not clear transform, causing offset-to-upper-left behavior.
- Remote model items had a second per-item `dblclick` handler using a different launch path (`handleRemoteModelClick`) than the main icon-container path (`launchRemoteModelFromIcon`), creating inconsistent behavior.

## Fixes

### 1) Centered, movable remote popup with larger default size

File: `frontend/modules/terminal-manager.js`

- `openNativeChatForServerSuccess(...)`
  - Uses `createWindow(...)` returned element directly.
  - Sets dynamic modern size:
    - width: `max(1100, min(90vw, 1700))`
    - height: `max(760, min(88vh, 1200))`
  - Clears center transform before explicit positioning: `windowElement.style.transform = 'none'`
  - Centers using computed px left/top and stores saved dimensions.

- `openNativeChatForServerError(...)`
  - Also clears transform before explicit centering.

### 2) Unified remote double-click launch path

File: `frontend/desktop.js`

- Removed per-item remote `dblclick` handler in `createRemoteModelListElement(...)`.
- Keeps a single authoritative remote launch path via the existing `#desktop-icons` container `dblclick` handler:
  - `launchRemoteModelFromIcon(...)`
  - This path ensures terminal manager readiness and avoids inconsistent failures.

## Verification

- `node --check frontend/desktop.js` ✅
- `node --check frontend/modules/terminal-manager.js` ✅
- `cargo check` (backend) ✅
