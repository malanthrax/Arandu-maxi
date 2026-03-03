# Chat History Navigation Continuation - 2026-02-27

- **Result:** Continued legacy chat history work in `frontend/llama-custom/index.html` and `frontend/modules/terminal-manager.js` so list/search/create/load is active by default and round-trips through the existing parent/iframe contract.
- **Bridge protocol:** `chat-logs-request` from the iframe now maps to backend chat log commands and replies via `chat-logs-response` using `request_id` correlation.
- **Behavior updates:**
  - Added client-side message persistence helpers and per-chat persisted message counters.
  - Sidebar list now renders `model · date · N msgs` metadata with fallback labels.
  - Active chat switching now calls `persistUnsavedMessages()` before switching.
  - New chat flow now creates/logs backend chat and keeps previous chat context saved.
- **Hardening:** Added validation in terminal bridge handler for missing/invalid `request_id`, op and required payload fields (`chatId`, `role`, etc.) before invoking Tauri commands.
- **Persistence fix:** `persistUnsavedMessages()` now increments saved counts only for successful message appends to avoid dropping unsaved messages from future switch/save attempts.
- **Verification run:**
  - `node --check frontend/modules/terminal-manager.js` (passed)
  - `cargo check --manifest-path Cargo.toml` in `backend/` (passed)
  - `cargo test --manifest-path Cargo.toml -- --quiet` in `backend/` (39 passed)
  - Manual in-app GUI validation still pending for full chat list/load/new-chat/search behavior and context persistence across switches.
