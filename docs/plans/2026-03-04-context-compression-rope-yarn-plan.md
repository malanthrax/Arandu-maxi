# Context Compression Controls + RoPE/YaRN Expansion Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add top-left context tools (counter + force-compression with 3 modes) and expand chat launch settings to include full llama.cpp-compatible RoPE/YaRN controls.

**Architecture:** Implement UI and logic primarily in `frontend/llama-custom/index.html` where chat controls and launch options already live. Reuse existing `postMessage` bridge to `TerminalManager` for operations that need parent control, and call server slot APIs from iframe where appropriate. Keep all changes additive and backward-compatible.

**Tech Stack:** Vanilla JS/HTML/CSS in `frontend/llama-custom/index.html`, parent bridge in `frontend/modules/terminal-manager.js` as needed.

---

### Task 1: Add top-left context tools UI

**Files:**
- Modify: `frontend/llama-custom/index.html`

**Steps:**
1. Add compact context tools row near the existing System title area.
2. Add context counter readout (`used / total`).
3. Add `Force Context Compression` button.
4. Add dropdown with 3 modes:
   - Slot-level compress
   - Soft reset context
   - Summarize + compress
5. Add styles matching current chat control language.

### Task 2: Add context counter data plumbing

**Files:**
- Modify: `frontend/llama-custom/index.html`

**Steps:**
1. Add polling/update function for context usage from server endpoints.
2. Update counter after each response completion and after compression actions.
3. Add fallback display when usage values are unavailable.

### Task 3: Implement 3 compression modes

**Files:**
- Modify: `frontend/llama-custom/index.html`
- Modify: `frontend/modules/terminal-manager.js` (only if bridge support is required)

**Steps:**
1. Slot-level compress: call slot-level compact/shift path.
2. Soft reset context: clear backend slot context while preserving chat UI list.
3. Summarize + compress: generate short summary from current history, keep as system memory, then reset/compress backend context.
4. Add loading/disable guard and user feedback notifications.

### Task 4: Expand RoPE/Position scaling settings to full compatibility

**Files:**
- Modify: `frontend/llama-custom/index.html`

**Steps:**
1. Keep current RoPE fields and add missing YaRN fields:
   - `launch_yarn_orig_ctx`
   - `launch_yarn_ext_factor`
   - `launch_yarn_attn_factor`
   - `launch_yarn_beta_slow`
   - `launch_yarn_beta_fast`
2. Extend defaults, UI bindings, snapshot comparison, and launch arg parsing/generation.
3. Ensure generated args match llama.cpp naming:
   - `--rope-scaling`, `--rope-scale`, `--rope-freq-base`
   - `--yarn-orig-ctx`, `--yarn-ext-factor`, `--yarn-attn-factor`, `--yarn-beta-slow`, `--yarn-beta-fast`

### Task 5: Subagent implementation

**Files:**
- Primary: `frontend/llama-custom/index.html`
- Secondary if needed: `frontend/modules/terminal-manager.js`

**Steps:**
1. Dispatch expert implementer subagent with strict scope.
2. Require no unrelated refactors.
3. Return changed file list and rationale.

### Task 6: Code review subagent + fixes

**Steps:**
1. Dispatch reviewer for requirements compliance and regressions.
2. Apply minimal fixes for any true defects.
3. Re-run review if fixes are substantial.

### Task 7: Triple-check verification and build

**Steps:**
1. Validate requested features are present via code inspection and checks.
2. Run syntax checks:
   - `node --check frontend/modules/terminal-manager.js`
   - inline script parse check for `frontend/llama-custom/index.html`
3. Build executable:
   - `cargo tauri build --no-bundle`
4. Confirm artifact:
   - `backend/target/release/Arandu.exe`
