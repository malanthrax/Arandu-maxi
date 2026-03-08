# 2026-03-08 - H-Drive Runtime Path Policy Migration

## Request
- Ensure runtime/config defaults and loaded defaults prefer `H:\Ardanu Fix\Arandu-maxi\.Arandu` in current dev phase instead of legacy C-drive `.Arandu` defaults.

## Implementation
- Updated `backend/src/models.rs`:
  - Added `preferred_arandu_base_dir()`.
  - `GlobalConfig::default()` now uses `H:\Ardanu Fix\Arandu-maxi\.Arandu` when that parent path exists.
  - Falls back to `home\.Arandu` when the preferred H-drive location is unavailable.

- Updated `backend/src/config.rs` (`load_settings()` path migration):
  - Added targeted migration helpers to remap only legacy default-style paths.
  - Remaps:
    - `models_directory`
    - `executable_folder`
    - `additional_models_directories[]`
    - `active_executable_folder` (when it matches criteria)
  - Leaves `active_executable_version` unchanged.
  - Migration triggers only when path is:
    - under old default base (`home\.Arandu`), or
    - starts with `C:\` and contains `\.Arandu\`.
  - If migration changes values, updated settings are persisted to disk immediately.

## Safety Notes
- Custom non-default paths (e.g., `D:\AI\models`) are not rewritten.
- Migration is minimal and scoped to default-style `.Arandu` paths only.

## Verification
- Command: `cargo check --manifest-path backend/Cargo.toml`
- Result: pass (see current session output)
