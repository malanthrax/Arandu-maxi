# 2026-03-08 - IK installer ZIP-only flow, installed-list recovery, and benchmark/MCP fixes

## Summary

- Simplified IK installer UX to ZIP-only workflow with no extra install-step popups.
- Restored installed-version visibility to include both normal llama.cpp installs and IK installs.
- Added MCP guidance to reduce placeholder "please wait" responses.
- Enhanced Benchmark Log with TPS column and best/worst color highlights.

## IK installer flow updates

- Main IK install: click `ik_llama.cpp` -> choose ZIP via Windows picker -> auto install.
- CUDA DLL install: click `ik CUDA DLL` -> choose ZIP via Windows picker -> auto install.
- Removed extra source-choice prompts from IK install path.
- CUDA main install now emits guidance toast to run DLL step manually (no blocking confirmation dialog).

## Installed versions visibility

- `list_llamacpp_versions` now scans:
  - configured versions root (`<executable_folder>/versions`)
  - legacy default root (`%USERPROFILE%\.Arandu\llama.cpp\versions`) when different
- Results are merged and de-duplicated by full path so both normal and IK installs appear in Installed list.

## MCP wait-message mitigation

- Added MCP-mode system instruction in chat request assembly:
  - if tools are needed, emit `tool_calls` immediately,
  - do not ask user to wait/queue/follow-up.

## Benchmark log updates

- Added `TPS` column to Benchmark Log.
- Highlight rules:
  - TTFT: lowest = green, highest = red
  - TPS: highest = green, lowest = red

## Files changed

- `frontend/modules/llamacpp-manager.js`
- `backend/src/lib.rs`
- `frontend/llama-custom/index.html`
- `THIS-PROJECTS-CURRENT-STATE.md`
- `docs/CURRENT_WORKING_STATE.md`
- `scratchpad.md`

## Verification

- `node --check frontend/modules/llamacpp-manager.js` passed.
- Inline script compile check for `frontend/llama-custom/index.html` passed (`inline_scripts_ok 1`).
- `cargo check --manifest-path backend/Cargo.toml` passed.
