# 2026-03-08 - GLM/IK readiness, template-order, and payload compatibility fixes

## Issues reported

- IK/GLM runs showed `usage:` + process exit, then UI still showed `Server is responding! Ready for chat` with white chat panel.
- MCP tool loop failed with HTTP 500: `System message must be at the beginning`.
- GLM-family runs crashed with sampler/assert instability in IK builds.

## Fixes implemented

### 1) False-ready guard in terminal lifecycle

- File: `frontend/modules/terminal-manager.js`
- Added `launchFailed` tracking per terminal.
- Detects usage-dump failure signature and non-zero process exit.
- Health check now refuses to mark terminal ready if process is stopped/failed.
- Prevents white-screen "ready" state after launch failure.

### 2) System-message ordering normalization

- File: `frontend/llama-custom/index.html`
- Added `normalizeTemplateMessages(messages)`.
- Before each chat completion request, payload messages are normalized so all `role: system` messages are placed at the beginning.
- Resolves jinja template strict-order error: `System message must be at the beginning`.

### 3) GLM compatibility payload mode

- File: `frontend/llama-custom/index.html`
- Added GLM-family detection (`isGlmFamilyActiveModel()`).
- For GLM names, request payload builder uses minimal safe sampling fields (avoids unstable advanced sampler set).
- Reduces GLM/IK crash risk from sampler incompatibility.

### 4) IK pre-launch unsupported-arg sanitizer

- File: `backend/src/process.rs`
- When launching with an `ik_llama.cpp` executable path, launch code now:
  - executes `llama-server --help`,
  - extracts supported flags,
  - removes unsupported custom args before process spawn.
- This prevents startup failures from incompatible custom launch options.

## Additional context from same bundle

- IK installer flow remains ZIP-only with no extra install-step popups.
- Installed versions listing merges configured and legacy versions roots so normal and IK installs appear together.
- Benchmark Log includes TPS and best/worst coloring.

## Verification

- `node --check frontend/modules/terminal-manager.js` passed.
- Inline script compile check for `frontend/llama-custom/index.html` passed (`inline_scripts_ok 1`).
- `cargo check --manifest-path backend/Cargo.toml` passed.
