# Memory Bank Activation & Tracking Protocol

**Date:** Mar 01, 2026  
**Status:** ✅ Active

## Objective

- Continue all work using the project knowledge base under `docs/knowledge-base` as the canonical long-term memory.
- Record every meaningful bug, fix, issue, decision, file path, command, and code change in dated entries.
- Use short, searchable naming with clear context for future retrieval.

## Applied in Session

- Confirmed canonical workspace is `H:\Ardanu Fix\Arandu-maxi` and will avoid edits in the C:\ worktree for functional changes.
- Confirmed existing knowledge base location and rules at:
  - `docs/knowledge-base/`
  - `docs/knowledge-base/documentation-rules.md`

## Session Memory Update Rules (next actions)

1. Create or append dated files in `docs/knowledge-base/` for:
   - Regression notes
   - Discovery or network model issues
   - UI rendering/click flow fixes
   - Verification command output
2. Add supporting updates to `THIS-PROJECTS-CURRENT-STATE.md` after each milestone.
3. Keep file names with date + topic, e.g. `2026-03-01-remote-model-regression.md`.
4. Include exact file paths and command evidence whenever possible.

## Tracking Baseline (Current)

- This session completed the remaining Remote LLMs click-path fix and then rebuilt release artifacts.
- Remote model click now opens the native remote chat window through existing launcher path instead of only showing a toast.
- Evidence and validation are logged in `docs/knowledge-base/2026-03-01-remote-click-launch-fix.md`.
- Follow-up note added for discovery diagnostics (`2026-03-01-discovery-log-no-recv-notice.md`) after user reported logs showing only SEND activity.
- Additional follow-up: added explicit receive-side logging for ignored/accepted discovery beacons in `backend/src/discovery.rs` and documented in `2026-03-01-discovery-recv-logging-enhancement.md` so you can distinguish "no packets" vs "packets ignored".
- Next concrete work target is the remote model visibility and launch regression in:
  - `frontend/desktop.js`
  - `frontend/css/desktop.css`
  - related render/interaction paths in the desktop UI layer.

(End of file)
