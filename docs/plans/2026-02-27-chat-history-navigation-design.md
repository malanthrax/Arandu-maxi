# Chat History Navigation and Persistence Design

## Goal
- Enable the custom chat UI history system in the left panel.
- Persist conversations as markdown files under the existing Arandu chat storage.
- Add reliable create/load/search/list operations and a **New Chat** action.

## Scope and Decisions
- **Frontend UI:** `frontend/llama-custom/index.html`
  - Turn on legacy history mode.
  - Add chat history header controls:
    - Search box (`#chatHistorySearch`)
    - New chat button (`#newChatButton`)
  - Render each tile with:
    - Title
    - Model used (`last_model`)
    - Date/time (`last_used_at`)
    - Message count (`message_count`)
  - Keep current `requestChatLogs(...)` contract and resolve via message responses from parent.
  - Save the active chat context when creating a new chat by creating a new chat entry first, preserving the current active chat in the index and continuing with the new chat.

- **Parent bridge:** `frontend/modules/terminal-manager.js`
  - Add `chat-logs-request` listener branch.
  - Implement operation router for:
    - `list` -> `list_chat_logs`
    - `search` -> `search_chat_logs`
    - `create` -> `create_chat_log`
    - `load` -> `get_chat_log`
    - `append` -> `append_chat_log_message`
    - `rename` -> `rename_chat_log`
  - Return responses as `{ type: 'chat-logs-response', request_id, ok, result|error }`.

- **Model context for each message:** always pass current model path when appending messages.

## Validation Plan
- Open chat window and verify:
  - History list loads with metadata.
  - Sending messages appends to markdown-backed chat file.
  - Search filters by title/content.
  - New chat creates a fresh session and keeps prior chats in list.
  - Selecting an item restores its messages.

## 2026-02-28 Execution Update

- Implementation is in place in:
  - `frontend/llama-custom/index.html` (history UI + list/search/new/load flows + persistence helpers)
  - `frontend/modules/terminal-manager.js` (chat logs bridge + command routing + validation)
- Build completed with `cargo tauri build --no-bundle` from the canonical workspace.
- Remaining confirmation items are manual GUI verification in the desktop app.

## 2026-02-28 Follow-up Verification

- Backend verification completed in canonical workspace:
  - `cargo check --manifest-path Cargo.toml`
  - `cargo test --manifest-path Cargo.toml -- --quiet` (39 passed)
  - `cargo tauri build --no-bundle` (release binary regenerated)
- Frontend verification: `node --check frontend/modules/terminal-manager.js` passed.
- Remaining items still require manual runtime checks in the desktop app:
  - history list loads,
  - search filtering,
  - new-chat context persistence,
  - switching/restore behavior.

## Risks
- Missing model path in parent terminal context could leave model metadata as `unknown`; fallback to safe defaults.
- Time formatting should avoid invalid date values and show `Unknown` when missing.
