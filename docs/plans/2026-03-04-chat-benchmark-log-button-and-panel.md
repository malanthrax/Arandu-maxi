# Chat Benchmark Log Button + Rolling Stats Panel Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add a book icon under the chat gear button that opens a settings-style panel showing the last 100 response benchmark entries (model, token counts, TTFT) for quick model speed comparisons.

**Architecture:** Extend the existing chat iframe UI in `frontend/llama-custom/index.html` only, reusing current assistant response stats extraction and panel toggle patterns. Add a compact rolling in-memory log with localStorage persistence and strict formatting rules (aligned columns, truncated model label with full hover tooltip).

**Tech Stack:** Vanilla HTML/CSS/JS in single-file chat UI (`frontend/llama-custom/index.html`).

---

### Task 1: Add UI controls and panel shell

**Files:**
- Modify: `frontend/llama-custom/index.html`

**Step 1: Add failing UI expectation**

Expected missing before implementation:
- Book icon button under existing floating gear toggle
- Benchmark panel container and scrollable list area

**Step 2: Add markup for book button and panel**

Add:
- Floating benchmark toggle button directly below `#floatingToggle`
- Panel container opened/closed by toggle function
- Header, clear button, and scrollable entries list

**Step 3: Match visual style to settings button behavior**

Use same button dimensions/colors/shadow/positioning pattern as gear button and same open/close interaction pattern.

**Step 4: Verify placement/readability**

Manual check: book icon visually aligns under gear and does not overlap chat or parameter panel.

---

### Task 2: Add benchmark data model, rolling history, and formatting

**Files:**
- Modify: `frontend/llama-custom/index.html`

**Step 1: Add failing data expectation**

Expected missing before implementation:
- No benchmark entries are stored after responses
- No cap at 100 entries

**Step 2: Add benchmark state and persistence**

Implement JS state:
- `benchmarkLogEntries` array
- `benchmarkLogStorageKey`
- `MAX_BENCHMARK_ENTRIES = 100`

Functions:
- load from localStorage
- save to localStorage
- push new entry with rolling trim (keep latest 100)

**Step 3: Add normalized benchmark entry schema**

Each entry stores only:
- `modelNameDisplay`
- `modelNameFull`
- `totalTokens`
- `mainTokens`
- `draftTokens`
- `ttftMs`
- `timestamp`

No prompt/question/chat-name data stored.

**Step 4: Add fixed-width/truncated display formatting**

Render rows with:
- Truncated model column for consistent spacing
- Monospace + fixed-width column formatting
- Hover tooltip (`title`) on model name showing full model string/path context

---

### Task 3: Hook benchmark capture into assistant response stats

**Files:**
- Modify: `frontend/llama-custom/index.html`

**Step 1: Add failing capture expectation**

Expected missing before implementation:
- Completing assistant response does not add benchmark row.

**Step 2: Identify existing finalized stats source**

Use the same already-computed response stats used for `message-stats` readout (tokens, TTFT, model metadata).

**Step 3: Capture and append on response completion only**

On successful assistant completion:
- Extract model + total/main/draft + TTFT
- Append benchmark entry once per completed response
- Re-render benchmark panel if open

**Step 4: Ensure no extra data leaks**

Confirm no chat text/question text is read into benchmark log.

---

### Task 4: Implement settings-like open/close behavior and UX details

**Files:**
- Modify: `frontend/llama-custom/index.html`

**Step 1: Add panel toggle behavior**

Implement open/close in same style as settings panel toggle mechanics:
- Book toggle opens/closes benchmark panel
- Escape/click-outside behavior follows existing panel conventions

**Step 2: Add utility actions**

- Clear log button (empties entries + storage)
- Empty-state message when no entries

**Step 3: Keep usability intact**

Verify:
- Existing gear/settings toggle still behaves the same
- Existing chat send/input/history interactions unchanged
- No overlap with active model switcher panel

---

### Task 5: Subcoder implementation via subagent

**Files:**
- Modify: `frontend/llama-custom/index.html`

**Step 1: Dispatch subcoder subagent**

Provide the exact requirements and constraints:
- single-file change target
- preserve all existing behavior
- no unrelated refactors

**Step 2: Validate subcoder diff scope**

Ensure only intended areas changed:
- button/panel markup
- benchmark CSS
- benchmark JS logic

---

### Task 6: Code reviewer subagent and defect fixes

**Files:**
- Review: `frontend/llama-custom/index.html`

**Step 1: Dispatch code reviewer subagent**

Review for:
- requirement compliance
- UI regressions
- duplicate entries/race conditions
- formatting/tooltip consistency

**Step 2: Apply only necessary fixes**

Patch defects found by reviewer. Avoid feature creep.

**Step 3: Re-run reviewer if substantial fixes were required**

Confirm ready status.

---

### Task 7: Integration safety check (no unrelated behavior changes)

**Files:**
- Verify: `frontend/llama-custom/index.html`

**Step 1: Static syntax check**

Run:
- `node --check frontend/modules/terminal-manager.js`
- (iframe file is HTML; use targeted grep/read checks + runtime validation)

**Step 2: Behavioral smoke checklist**

Confirm unchanged behavior:
- chat send via Enter/button
- parameter panel toggle
- active model switcher
- attachments and history list

**Step 3: Build for test**

Run:
- `cargo tauri build --no-bundle`

Expected:
- updated `backend/target/release/Arandu.exe`

---

### Task 8: Documentation

**Files:**
- Modify: `docs/CURRENT_WORKING_STATE.md`
- Modify: `THIS-PROJECTS-CURRENT-STATE.md`
- Add: `docs/knowledge-base/2026-03-04-chat-benchmark-log-panel.md`

**Step 1: Document feature behavior and constraints**

Include:
- icon placement
- panel behavior
- 100-entry rolling cap
- exact tracked fields

**Step 2: Document verification evidence**

Include command outputs and artifact path.
