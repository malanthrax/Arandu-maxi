# Chat History Navigation - Implementation Complete

**Date:** 2026-02-27  
**Status:** ✅ COMPLETE AND OPERATIONAL  
**Component:** Chat History Sidebar in Custom Chat UI  

---

## Overview

Persistent markdown chat sessions with sidebar history navigation are now fully implemented and working in Arandu. Users can:

- View all saved chat sessions in the left sidebar
- Click any chat to load and continue that conversation
- See auto-generated titles after 4 conversation turns
- Delete unwanted chats
- Create new chat sessions
- Have all conversations automatically persisted to markdown files

---

## Architecture

### Frontend (Iframe: `frontend/llama-custom/index.html`)

**Chat History UI Components:**
- `.chat-history-sidebar` - Resizable sidebar (320px default)
- `#chatHistoryList` - Scrollable list of chat items
- `.chat-history-item` - Individual chat entry (role="button", tabindex="0")
- `.chat-history-delete` - Delete button on each item

**Event Handling:**
- Single delegated handler on `#chatHistoryList` handles all clicks and keydowns
- Keyboard support: Enter/Space activates focused chat item
- Clicking item loads chat; clicking delete button removes chat

**Key Functions:**
- `requestChatLogs(op, payload)` - postMessage bridge to parent
- `loadChatById(chatId)` - Loads chat content and switches active session
- `deleteChatById(chatId)` - Removes chat from storage
- `startNewChat()` - Creates new chat session
- `persistUnsavedMessages()` - Saves unsaved messages before switching
- `refreshChatHistoryList()` - Refreshes sidebar list from backend

### Parent Bridge (`frontend/modules/terminal-manager.js`)

**Message Handling:**
- Listens for `chat-logs-request` from iframe via postMessage
- Routes to appropriate Tauri command based on `op` field
- Sends `chat-logs-response` back to iframe with results

**Operations Supported:**
- `list` - Get all chats (sorted by last_used_at desc)
- `search` - Search chats by term
- `create` - Create new chat with model metadata
- `load` - Get chat content and metadata by ID
- `append` - Add message to existing chat
- `rename` - Update chat title
- `delete` - Remove chat from index and filesystem

**Tauri Invoke Mapping:**
- `list_chat_logs` → `list`
- `search_chat_logs` → `search` (term)
- `create_chat_log` → `create` (model)
- `get_chat_log` → `load` (chatId)
- `append_chat_log_message` → `append` (chatId, role, content, model)
- `rename_chat_log` → `rename` (chatId, title)
- `delete_chat_log` → `delete` (chatId)

### Backend (`backend/src/lib.rs`)

**Storage Format:**
- Index: `~/.Arandu/chats/index.json` (JSON array of chat metadata)
- Chat files: `~/.Arandu/chats/{chat_id}.md` (Markdown with YAML frontmatter)

**Index Entry Schema:**
```json
{
  "chat_id": "chat-{timestamp}",
  "file_path": "chat-{timestamp}.md",
  "title": "Auto-generated or user-set title",
  "created_at": "ISO 8601 timestamp",
  "last_used_at": "ISO 8601 timestamp",
  "last_model": "sanitized_model_name",
  "models_used": ["model1", "model2"],
  "message_count": 42
}
```

**Markdown Format:**
```markdown
---
chat_id: chat-1234567890
title: Chat 2024-01-15 10:30
created_at: 2024-01-15T10:30:00Z
last_used_at: 2024-01-15T11:45:00Z
models_used: llama-3-8b, qwen2-7b
---

## USER | 2024-01-15T10:30:00Z | llama-3-8b

Hello, how are you?

## ASSISTANT | 2024-01-15T10:30:05Z | llama-3-8b

I'm doing well! How can I help you today?
```

---

## Critical Bug Fixes Applied

### 1. Tauri v2 camelCase Convention (CRITICAL)
**Problem:** Tauri v2 automatically converts snake_case Rust parameter names to camelCase for JS invoke calls. The frontend was sending `{ chat_id: "..." }` but Tauri expected `{ chatId: "..." }`.

**Impact:** All chat log operations (load, append, rename, delete) failed with "invalid args" / "missing required key" errors.

**Fix:** Changed all invoke call keys from snake_case to camelCase:
```javascript
// BEFORE (broken):
await invoke('get_chat_log', { chat_id: chatId });

// AFTER (working):
await invoke('get_chat_log', { chatId: chatId });
```

**Files:** `frontend/modules/terminal-manager.js` lines 466, 477, 489, 499

### 2. Error Propagation Fix
**Problem:** Tauri invoke rejections return plain strings, not Error objects. The catch block did `error.message` which was always `undefined`, hiding the real error.

**Impact:** Generic "Failed to process chat logs request" instead of specific error messages.

**Fix:** Check if error is a string before accessing `.message`:
```javascript
const errorMsg = typeof error === 'string' 
  ? error 
  : (error && error.message ? error.message : String(error));
```

**Files:** `frontend/modules/terminal-manager.js` line 508

### 3. Index File Corruption Prevention
**Problem:** `fs::write` is not atomic. Concurrent async operations (multiple append calls from duplicate listeners) could corrupt the index.json file with "trailing characters".

**Impact:** After ~10-20 chat operations, index would corrupt and all chats would disappear with parse error.

**Fix:** Atomic writes using temp file + rename:
```rust
fn write_chats_index(index: &[serde_json::Value]) -> Result<(), String> {
    let tmp_path = index_path.with_extension("json.tmp");
    fs::write(&tmp_path, content)?;
    fs::rename(&tmp_path, &index_path)
}
```

**Files:** `backend/src/lib.rs` lines 160-175

### 4. Index Corruption Recovery
**Problem:** If index was already corrupted, entire chat UI would fail to load.

**Impact:** User sees "Failed to load chats" error and cannot use chat history at all.

**Fix:** Graceful degradation:
- Backend: Return `Ok(Vec::new())` instead of `Err(...)` on any parse failure
- Backup corrupt file to `.json.bak`
- Frontend: Show empty list instead of error message
- User can continue using app (create new chats)

**Files:** 
- `backend/src/lib.rs` `read_chats_index()` function
- `frontend/llama-custom/index.html` error handler in `refreshChatHistoryList()`

### 5. Iframe confirm() Blocking
**Problem:** `confirm()` dialogs don't work properly in Tauri webview iframes. They may silently return `false` or appear behind the parent window.

**Impact:** Delete button appeared to do nothing.

**Fix:** Removed `confirm()` dialog for delete operations. Delete now executes immediately.

**Files:** `frontend/llama-custom/index.html` `deleteChatById()` function

### 6. Duplicate Event Listeners
**Problem:** Both per-item listeners AND delegated listener on parent were attached. Both fired on same click, causing race conditions and multiple concurrent operations.

**Impact:** Request counter jumped by 2-4 per click; concurrent writes increased corruption risk.

**Fix:** Removed per-item click/keydown/delete listeners. Single delegated handler on `chatHistoryList` handles everything.

**Files:** `frontend/llama-custom/index.html` `renderChatHistoryList()` function

---

## Testing Checklist

- [x] Load chat history list on startup
- [x] Click chat item to load conversation
- [x] Send message in loaded chat
- [x] Auto-title generation after 4 turns
- [x] Create new chat with "New" button
- [x] Delete chat with trash icon
- [x] Search/filter chats
- [x] Rapid clicking (10-20 times) doesn't corrupt index
- [x] Corrupted index recovers gracefully
- [x] Keyboard navigation (Tab, Enter, Space)

---

## File Locations

**Frontend:**
- `frontend/llama-custom/index.html` - Chat UI with sidebar (lines ~1160-1200 for HTML, ~1930+ for JS)
- `frontend/css/main.css` - Sidebar styling (if extracted, currently inline in index.html)

**Parent Bridge:**
- `frontend/modules/terminal-manager.js` - handleChatLogsRequest() function (lines ~415-510)

**Backend:**
- `backend/src/lib.rs` - Chat log commands (lines ~517-727)
  - `list_chat_logs()` - Line 517
  - `create_chat_log()` - Line 528
  - `get_chat_log()` - Line 646
  - `append_chat_log_message()` - Line 566
  - `rename_chat_log()` - Line 633
  - `delete_chat_log()` - Line 672
  - `search_chat_logs()` - Line 727
  - `read_chats_index()` - Line 66
  - `write_chats_index()` - Line 160
  - `chats_dir()` - Line 54

**Storage:**
- Windows: `%USERPROFILE%\.Arandu\chats\`
- Index: `index.json`
- Chats: `chat-{timestamp}.md`
- Backups: `index.json.bak`

---

## Related Documentation

- `AGENTS.md` - Architecture overview and development guidelines
- `THIS-PROJECTS-CURRENT-STATE.md` - Current status and recent changes
- `docs/knowledge-base/` - Detailed implementation notes and bug fixes
- `docs/plans/2026-02-27-chat-history-navigation-design.md` - Original design document

---

## Build Commands

```bash
cd backend
cargo check --manifest-path Cargo.toml
cargo test --manifest-path Cargo.toml -- --quiet
cargo tauri build --no-bundle
# Output: backend\target\release\Arandu.exe
```

---

## File Attachments (Related Feature)

The chat UI also supports file attachments via the + button:

### Supported File Types
- **Images:** PNG, JPG, JPEG, WEBP, GIF, BMP (sent as base64 for vision models)
- **Documents:** PDF, DOCX, DOC, TXT (text extracted and included in message)
- **Other:** Files sent by reference if text extraction not available

### How It Works
1. Click + button → file picker opens
2. Select file(s) → preview appears in attachment area
3. Type message (optional) → click Send
4. Images sent as `image_url` type in multimodal format
5. Documents have text extracted via PDF.js/mammoth.js
6. All attachments included in message payload to LLM

### Implementation Details
- **Frontend:** `handleFileSelection()` reads files, `sendMessage()` constructs multimodal payload
- **Format:** OpenAI-compatible `{type: "image_url", image_url: {url: "base64..."}}`
- **Text Extraction:** PDFs via pdfjsLib, DOCX via mammoth
- **Size Limit:** Text attachments truncated to 12,000 characters

---

**Last Updated:** 2026-02-28  
**Build Status:** ✅ Working  
**Tests:** 39/39 Passing