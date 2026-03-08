# Scratchpad

## Active Requests (2026-03-07)

- ~~Hide/minimize MCP stdio terminal windows on Windows so MCP usage does not pop visible console windows.~~
- ~~Verify backend build check and update docs/memory; cross off scratchpad item.~~
- ~~Add MCP Log entry in chat menu under Benchmark Log.~~
- ~~Implement MCP Log window to show model -> MCP tool call payloads and MCP -> model return data.~~
- ~~Verify chat script syntax, update docs/memory, and cross off completed item.~~
- ~~Fix MCP log gaps: capture pre-request failures (including HTTP 500) and full LLM/MCP loop lifecycle so failing models still show diagnostic log entries.~~
- ~~Add user-configurable MCP tool loop limit in chat Model Options (max 20) and use it in MCP execution loop.~~
- ~~Debug multimodal image uploads returning `Chat request failed (500)` and implement a robust fix.~~
- ~~Find a fixed Jinja template for Apriel-1.6/Apriel models online and add it to selectable templates in model tiles.~~
- ~~Add `ik_llama.cpp` install flow in llama.cpp Release Manager (local .zip picker via Explorer) and treat installed backend like normal versions with active-state handling.~~
- ~~Add persistent Unsloth warning banner on model pages: `Do not use quantized models from Unsloth that have _XL in their name.`~~
- ~~Run coding subagent + review subagent workflow, verify, document, update memory, and cross off scratchpad items.~~
- ~~Add `Readme` button near model view controls to open IK support reference window.~~
- ~~Add gold-star badge on model titles for models matching IK-supported families list.~~
- ~~Persist full IK-supported models list in `ikllama.cpp supported models.md`, nowledge mem, and supermemory mirror note.~~
- ~~Force Arandu runtime/config paths to prefer H: workspace base instead of C: defaults for current development phase.~~
- ~~Add dual-file ik_llama.cpp local installer flow: pick main binary archive and pick CUDA DLL archive separately.~~
- ~~Keep CPU-only ik_llama.cpp installs as single-file flow (DLL step optional/skippable).~~
- ~~Add install-time validation/warnings for CUDA builds when DLL package is missing, then update docs and memory.~~
- ~~Support loose source structure for ik installer: allow selecting extracted folder sources (not only zip).~~
- ~~Create/use dedicated IK installer workspace folders for drop files and temporary extraction.~~
- ~~Fix confusing ik_llama installer UX: replace ambiguous source prompt with explicit ZIP/Folder/Cancel choice flow and show install destination on success (main IK + CUDA DLL paths).~~
- ~~Verify IK installer module syntax (`node --check frontend/modules/llamacpp-manager.js`), update state docs, and cross off this scratchpad item.~~
- ~~Add visible IK installer workspace panel in Llama.cpp manager so users can see/copy holding folder paths.~~
- ~~Update all 3 doc locations for workspace visibility change.~~
- ~~Add explicit ik_llama.cpp installed version/readout badge in Release Manager header area.~~
- ~~Fix IK install flow to zero-extra-popups: click button -> pick ZIP -> automatic install only.~~
- ~~Remove any remaining confirm/prompt/modals in IK main and IK CUDA DLL install paths.~~
- ~~Restore Installed Versions list to show both normal llama.cpp backends and IK backends together in scrollable list.~~
- ~~Fix MCP tool-call waiting behavior where model asks user to wait instead of issuing tool_calls.~~
- ~~Benchmark Log enhancements: show TPS column and color fastest/slowest TTFT and TPS (best green, worst red).~~
- ~~Prevent false-ready white-screen state by blocking health-ready transition when llama-server exits with usage/invalid-args.~~
- ~~Fix GLM/IK chat failures: template system-message ordering errors and sampling-payload compatibility crashes.~~
- ~~Add pre-launch IK compatibility sanitizer: when active backend is ik_llama.cpp, auto-disable/remove unsupported launch options before starting model.~~
- Recheck IK-supported star matching and ensure GLM models reliably receive gold star in model list.

## Completed This Cycle

1. Process policy codified in docs:
   - scratchpad-first execution
   - mandatory coding subagent + review subagent workflow
2. UI/features/bugs:
    - model tile shadow updates
    - yellow `Custom` badges (remote + local/list contexts)
    - AMD/iGPU VRAM fallback and frontend handling updates
    - options page split actions under Global Options
3. ~~Security hardening:~~
   - ~~replaced assistant model-output rendering path in `frontend/llama-custom/index.html` to avoid raw HTML insertion~~
    - ~~added safe text render helpers and DOM-based stats rendering~~
    - ~~verified inline script syntax with Node extraction/check (`inline_scripts_ok 1`)~~
4. ~~System Prompt Override feature:~~
     - ~~added top-right light-blue `System Prompt Override` button~~
     - ~~added prompt override management window with save/select/default behavior~~
     - ~~persisted entries + selected prompt in localStorage~~
     - ~~wired precedence: typed chat system prompt > selected global override > existing defaults~~
     - ~~wired best-effort launch arg override (`--system-prompt`) via terminal manager~~
5. ~~Focused review: System Prompt Override implementation~~
    - ~~reviewed scoped files + message contracts and precedence path~~
    - ~~ran parser checks for `desktop.js`, `terminal-manager.js`, and inline scripts in `frontend/llama-custom/index.html`~~
    - ~~captured must-fix/should-fix findings with patch-level suggestions~~
6. ~~System Prompt Override manager UX:~~
     - ~~added `New` button beside Clear/Save in manager actions~~
     - ~~`New` now clears editor fields and resets selection to `Default` to start a fresh save~~
     - ~~verified syntax with `node --check frontend/desktop.js`~~
7. ~~Date-awareness hardening in chat request path:~~
    - ~~added always-on system clock injection (local date/time, timezone, UTC ISO) before optional prompts in `frontend/llama-custom/index.html`~~
    - ~~preserved existing system prompt precedence (typed > global override)~~
    - ~~verified inline script compile with Node VM extraction (`inline_scripts_ok 1`)~~
