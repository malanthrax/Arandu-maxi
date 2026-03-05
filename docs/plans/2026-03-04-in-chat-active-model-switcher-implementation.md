# In-Chat Active Model Label + Live Model Switcher Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add an in-chat active model indicator near the Send button and a live switcher that can switch local or reachable remote models immediately while keeping the same chat thread visible.

**Architecture:** Keep the chat iframe as a thin UI layer and let TerminalManager own model inventory + switch execution (parent-orchestrated). The iframe requests inventory and submits switch intents via postMessage; the parent validates, performs local restart or remote launch handoff, and returns explicit success/error events.

**Tech Stack:** Vanilla JS, HTML/CSS in frontend/llama-custom/index.html, parent orchestration in frontend/modules/terminal-manager.js, existing desktop discovery state in frontend/desktop.js.

---

### Task 1: Add Chat UI shell for active model indicator + switcher panel

**Files:**
- Modify: frontend/llama-custom/index.html

**Step 1: Write failing UI test condition (manual assertion target)**

Expected failure before implementation: Elements do not exist in DOM:
- #activeModelSwitcherButton
- #activeModelSwitcherPanel
- #activeModelListLocal
- #activeModelListRemote

**Step 2: Implement minimal UI shell**

Insert into .input-container near Send button:
- compact active model button with truncated model label
- hidden dropdown panel with scrollable local + remote sections
- empty-state rows and loading row

Add CSS for:
- compact label style (small font, clickable)
- panel positioning above input area
- list row styles and selected/loading/error states
- max-height + overflow for scrollable content

**Step 3: Verify DOM integration manually by readback**

Confirm markup and style blocks are present and IDs/classes are unique.

---

### Task 2: Add iframe-side message protocol for inventory + switch actions

**Files:**
- Modify: frontend/llama-custom/index.html

**Step 1: Write failing behavior test (manual/console target)**

Before implementation, clicking the new label should do nothing.

Expected failure: no postMessage request is sent to parent for inventory.

**Step 2: Implement request/response handlers**

Add iframe JS logic to:
- request inventory (type: request-chat-model-switcher-data)
- receive inventory (type: chat-model-switcher-data) and render local/remote sections
- send switch request (type: request-chat-model-switch) with selected target
- receive switch result (type: chat-model-switch-result) and show system message + status

State rules:
- disable switcher while request in-flight
- keep panel open on failures
- close panel on success
- keep chat history arrays untouched

**Step 3: Add keyboard/UX polish**

Implement:
- click outside to close panel
- Escape key closes panel
- preserve current active label until parent confirms success

---

### Task 3: Add parent-side inventory provider and switch executor

**Files:**
- Modify: frontend/modules/terminal-manager.js
- Read-only reference: frontend/desktop.js

**Step 1: Write failing behavior test (runtime expectation)**

Expected failure before implementation:
- parent ignores request-chat-model-switcher-data
- parent ignores request-chat-model-switch

**Step 2: Extend message listener dispatch**

In initTauriAPI() add handling for:
- request-chat-model-switcher-data
- request-chat-model-switch

**Step 3: Implement inventory collection helper**

Add helper methods in TerminalManager:
- identify source terminal/window by sourceWindow
- fetch local models via invoke('scan_models_command')
- fetch reachable remote models from this.desktop.discoveredPeers
- normalize payload fields (id, name, path, sourceType, peer host/api/chat ports)
- include current active model metadata for selected-row highlighting

**Step 4: Implement local switch execution**

For local target:
- update terminalInfo modelPath + modelName
- call existing restartServer(windowId, newModelPath, newModelName)
- refresh server header label + chat section model label
- return structured result to iframe

**Step 5: Implement remote switch execution**

For remote target:
- use existing openNativeChatForServer(...)
- keep current window/thread visible
- send success result to iframe if launch request accepted
- on failure, return error and preserve current model label

**Step 6: Verify syntax**

Run: node --check frontend/modules/terminal-manager.js

Expected: PASS

---

### Task 4: Keep model label synchronized with lifecycle events

**Files:**
- Modify: frontend/modules/terminal-manager.js
- Modify: frontend/llama-custom/index.html

**Step 1: Add failing sync expectation**

Expected failure before implementation: label may become stale after restart/launch completion.

**Step 2: Implement label sync events**

Parent emits to iframe on:
- server terminal opened
- restart success
- model switch success

Event type:
- chat-active-model-changed with { modelName, modelPath, sourceType }

**Step 3: Child consumes sync event**

Update active label immediately and selected-row highlight.

---

### Task 5: Subagent code review + fixes

**Step 1:** Dispatch review subagent for changed files.

**Step 2:** Apply only valid defect fixes.

**Step 3:** Re-run syntax checks.

---

### Task 6: Validation and production build

**Step 1:** Run checks
- node --check frontend/modules/terminal-manager.js
- cargo check --manifest-path backend/Cargo.toml

**Step 2:** Build executable from backend/
- cargo tauri build --no-bundle

**Step 3:** Capture artifact timestamp for backend/target/release/Arandu.exe.

---

### Task 7: Documentation updates for current state

**Files:**
- Add KB entry for this feature implementation
- Update THIS-PROJECTS-CURRENT-STATE.md
- Update docs/CURRENT_WORKING_STATE.md

Include:
- protocol messages
- files touched
- switch semantics
- verification/build evidence
