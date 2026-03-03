# Remote Interaction Hardening - 2026-03-01

**Date:** Mar 01, 2026

## Issue

Remote model entries could be listed, but interaction handling could still miss remote-only launch behavior or attempt local launch flows. Remote models needed a strict interaction guard.

## Fix Implemented

Hardening in `frontend/desktop.js` now separates remote and local model flows end-to-end:

1. Added remote model detection helpers:
   - `isRemoteModelIcon(icon)`
   - `getRemoteModelInfoFromIcon(icon)`
   - `launchRemoteModelFromIcon(icon)`
2. Routed remote interactions to the remote chat flow:
   - double-click on remote model launches `terminalManager.openNativeChatForServer(modelName, host, port)`
   - Enter key on selected remote model uses the same remote launcher
   - remote context menu only shows and executes `Open Remote Chat`
3. Prevented non-remote metadata from local launch invocation:
   - `launchModel`, `launchModelWithHalfContext`, `launchModelExternal`, `launchModelWithPreset`, `launchModelWithPresetExternal`
   - each now validates `icon.dataset.path` before local launch calls
4. Kept and validated remote click path:
   - host from `peer.ip_address`/`peerIp`
   - API port from `peer.peer_api_port`, `peer.api_port`, discovery status fallback, then `8081`
5. Improved remote metadata shape in list rendering:
   - `createRemoteModelListElement()` stores `peer_api_port`
   - keeps compatibility fields (`name`, `peer`, `peer_ip`, quantization, size, etc.)
6. Added clearer user feedback for remote connection failures/success.

## Files Changed

- `frontend/desktop.js`

## Evidence

- `node --check frontend/desktop.js` (no syntax errors)
- Static checks confirm updated handlers around:
  - `isRemoteModelIcon`/`getRemoteModelInfoFromIcon`/`launchRemoteModelFromIcon`
  - double-click handler (~`890`)
  - Enter key launch handler (~`1296`)
  - context menu handler/building (~`939-1368`)
  - local launch guards (~`2508`, `2677`, `2935`, `2959`, `3084`)
  - remote list rendering (`createRemoteModelListElement`) and click handler (`handleRemoteModelClick`)

## Result

Remote interactions are now explicitly isolated from local launch paths; local launch commands are blocked when model-path metadata is missing and remote models open through the native remote chat bridge only.

**Canonical Location:** `H:\Ardanu Fix\Arandu-maxi`
