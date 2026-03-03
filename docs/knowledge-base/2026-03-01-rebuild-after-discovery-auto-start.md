# Rebuild: Discovery Auto-Start Session

## Date

- Mar 01, 2026

## Objective

- Rebuild a fresh desktop executable after discovery-startup and warning-cleanup work.

## Command

- `cargo tauri build --no-bundle` (executed from `H:\Ardanu Fix\Arandu-maxi\backend`)

## Outcome

- Build succeeded, optimized profile.
- New artifact generated:
  - `H:\Ardanu Fix\Arandu-maxi\backend\target\release\Arandu.exe`
- This is ready for user runtime smoke testing of discovery auto-start behavior.

## Notes

- Previously, `cargo test -- --quiet` fails in this environment with `STATUS_ENTRYPOINT_NOT_FOUND` (`0xc0000139`) after compilation; this rebuild still produced a valid release binary.

## Next Step

- Start the rebuilt app with persisted `discovery_enabled: true` and confirm:
  - OpenAI proxy starts on startup
  - Discovery service is active
  - Remote peer list/remote models refresh automatically
