# Chat Input Layout (Pinned Bottom) Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Dock the chat input to the bottom with a 4-line minimum textarea and preserve visibility of the most recent messages.

**Architecture:** Use a flex column layout for `.chat-main` with `.chat-messages` as the scrollable area and `.chat-input-area` at the bottom. Update bottom padding dynamically so the last message stays visible above the composer as it grows.

**Tech Stack:** Vanilla JS, CSS (Arandu frontend)

---

### Task 1: Ensure layout anchors input to bottom

**Files:**
- Modify: `frontend/css/chat-app.css`

**Step 1: Update layout styles**

Add or adjust these rules (ensure they exist and are not overridden):

```css
.chat-main {
  display: flex;
  flex-direction: column;
  height: 100%;
}

.chat-messages {
  flex: 1;
  overflow-y: auto;
}

.chat-input-area {
  flex: 0 0 auto;
}
```

**Step 2: Run a quick UI check**

Launch the app and confirm the input bar sits at the bottom edge.

---

### Task 2: Set 4-line minimum textarea height

**Files:**
- Modify: `frontend/css/chat-app.css`
- Modify: `frontend/modules/chat-app.js`

**Step 1: Adjust textarea min-height**

In `.chat-input`, set a min-height that fits 4 lines (e.g., 120px) and ensure padding/line-height match.

**Step 2: Enforce min height in auto-resize**

Update `autoResizeTextarea` to respect the minimum height:

```javascript
autoResizeTextarea(textarea) {
  const minHeight = 120;
  textarea.style.height = 'auto';
  textarea.style.height = `${Math.max(minHeight, textarea.scrollHeight)}px`;
}
```

**Step 3: Verify**

Type multiple lines and ensure the textarea starts at 4 lines and grows smoothly.

---

### Task 3: Keep latest message visible above the composer

**Files:**
- Modify: `frontend/modules/chat-app.js`
- Modify: `frontend/css/chat-app.css`

**Step 1: Add bottom padding to message list**

Set an initial CSS variable default in `.chat-messages`:

```css
.chat-messages {
  padding-bottom: var(--composer-padding, 160px);
}
```

**Step 2: Dynamically update padding**

Add a method to `ChatApp`:

```javascript
updateComposerPadding() {
  const inputArea = document.querySelector(`#chat-input-area-${this.processId}`);
  const messages = document.querySelector(`#chat-messages-${this.processId}`);
  if (!inputArea || !messages) return;
  const padding = inputArea.offsetHeight + 16;
  messages.style.setProperty('--composer-padding', `${padding}px`);
}
```

Call it after render, on textarea input, and after auto-resize.

**Step 3: Verify**

Scroll to bottom; the last message should always be visible above the input.

---

### Task 4: Manual verification

**Step 1: Run the app**

`cd backend && cargo tauri dev`

**Step 2: Check behavior**

- Input pinned to bottom edge
- 4 lines visible by default
- Message history not hidden behind composer

---

## Notes

- If selectors are different, use the actual IDs/classes in `chat-app.js`.
- Keep the solution CSS-first; JS should only handle dynamic padding and min height.
