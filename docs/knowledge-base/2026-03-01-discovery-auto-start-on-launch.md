# Discovery Auto-Start on App Launch

## Date

- Mar 01, 2026

## Objective

- Ensure discovery restores automatically when the app starts if `discovery_enabled` is saved in config.

## Scope

- Backend startup wiring in `backend/src/lib.rs`.

## What Changed

- Updated `auto_start_discovery_if_enabled` to accept an optional `tauri::AppHandle` argument.
- Wired startup entrypoint to call:
  - `auto_start_discovery_if_enabled(&state, Some(app_handle))` inside `run()` setup.
- The startup helper now starts the OpenAI proxy (if needed) first, then the discovery service using persisted config values.
- Cleaned a low-priority Rust warning by removing unused `tracing::error` import in `backend/src/openai_proxy.rs` (not functional, just warning cleanup).

## File-level Evidence

- `backend/src/lib.rs`
  - `start_discovery_service(...)` call updated to accept `Option<tauri::AppHandle>` from startup.
  - `auto_start_discovery_if_enabled(...)` signature updated to accept app handle.
  - `run()` setup now executes startup auto-start after `app.manage(state)`.

## Validation

- `node --check frontend/desktop.js` (pass)
- `cargo check --manifest-path backend/Cargo.toml` (pass)
- `cargo test --manifest-path backend/Cargo.toml -- --quiet` now runs far enough to compile tests but still terminates at runtime in this environment:
  - `STATUS_ENTRYPOINT_NOT_FOUND` (`0xc0000139`)
- `cargo check --manifest-path backend/Cargo.toml` now succeeds after the `discovery.rs`/`Uuid` borrow and import issues were already resolved.

## Notes

- App startup now attempts discovery restoration without waiting for user action, improving persistence continuity across restarts.
- This does not currently adjust any user-facing settings or UI toggle behavior; it only follows persisted config state.

## Follow-Up Notes

- Full startup end-to-end validation still requires an interactive desktop run on a Windows GUI session:
  - set `discovery_enabled: true` in persisted config,
  - restart app,
  - confirm OpenAI proxy and discovery services auto-start and peer data appears.
- All compile-time checks are green after this pass; only the runtime test harness issue remains.
