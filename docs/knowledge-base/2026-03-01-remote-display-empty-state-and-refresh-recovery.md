# 2026-03-01 - Remote display empty-state + refresh recovery

## Problem addressed

Remote view could appear visually broken (header only / blank list) when peers were discovered but model arrays were still empty due handshake timing or temporary fetch misses.

## Changes made

- `frontend/desktop.js`
  - Added bounded auto-recovery refresh path in discovery polling:
    - If peers are reachable but total remote model count is `0`, trigger `refresh_remote_models` with cooldown.
  - Added explicit method: `refreshRemoteModelsNow(force)`.
    - Cooldown for auto mode: 15 seconds.
    - Prevents overlap with `remoteModelRefreshInFlight` guard.
  - Added remote view empty-model state UI:
    - Shows "Peers discovered, waiting for model list" message.
    - Displays peer/reachability counts.
    - Adds `Refresh Remote Models` button for manual recovery.

## Why

- Makes "no models yet" an explicit state instead of looking like a rendering failure.
- Adds a safe recovery handshake when discovery is live but model fetch timing lags.
- Gives users deterministic manual refresh control from the remote view.

## Verification

- Command: `node --check frontend/desktop.js`
- Result: pass (no syntax errors)
