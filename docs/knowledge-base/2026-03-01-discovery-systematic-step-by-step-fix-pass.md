# 2026-03-01 - Discovery systematic step-by-step fix pass

## Scope

User requested a slow, step-by-step pass across timing, locks, file persistence, and CSV concerns for discovery/model-cache reliability.

## What was checked

1. **Timing paths**
   - Beacon broadcast interval source and usage
   - New-peer auto-fetch delay/timeout/retry behavior
   - Frontend poll cadence and overlap behavior
2. **Lock/contention risk**
   - Discovery status lock ordering in command handlers
3. **File/cache persistence**
   - `peer_models_cache.json` write path and durability behavior
   - app data dir fallback behavior
4. **CSV**
   - No CSV-based discovery/cache pipeline exists (only attachment-type extension mentions in chat UI).

## Root issues addressed in this pass

1. **Auto-fetch too fragile for model convergence**
   - Previously: one-shot attempts only for brand-new peers.
   - Now: bounded retry loop with backoff and cooldown; also retries for peers still missing models.
   - File: `backend/src/discovery.rs`

2. **Port source for discovery peer model fetch**
   - Prefer beacon `api_port`; fallback to endpoint parsing only when missing.
   - File: `backend/src/discovery.rs`

3. **Cache persistence race/durability improvements**
   - Added serialized persist lock.
   - Switched to unique temp-file writes before replace/rename.
   - Logged explicit cache file path at startup.
   - File: `backend/src/peer_cache.rs`

4. **Discovery enable should fail if API server cannot start**
   - Previously continued with warning, causing advertised-but-dead API port states.
   - Now returns error and aborts discovery enable.
   - File: `backend/src/lib.rs`

5. **Frontend discovery poll reliability**
   - Added in-flight poll guard to prevent overlapping async polls.
   - Enable flow now starts polling immediately and updates local discovery status.
   - Disable flow now resets discovery signature cache to avoid stale no-render on re-enable.
   - File: `frontend/desktop.js`

6. **Cache provenance for merged peers**
   - Runtime peers filled from cache now correctly set `models_from_cache` and `cache_last_updated`.
   - File: `backend/src/discovery.rs`

## Verification

- `pwd` -> `/h/Ardanu Fix/Arandu-maxi`
- `node --check frontend/desktop.js` -> pass
- `cargo check --manifest-path backend/Cargo.toml` -> pass
- Targeted `cargo test` compiles but runtime still blocked in this environment with `STATUS_ENTRYPOINT_NOT_FOUND` (known env issue).

## Agent-assisted process used

- Planning agent: generated conservative phased remediation strategy.
- Coding agent: reviewed retry/cache/poll implementation risks and flagged concrete gaps.
- Review agent: identified and then validated fix for signature-reset regression; final rebuild gate = GO.
