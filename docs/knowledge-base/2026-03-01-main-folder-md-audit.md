# 2026-03-01 Main Folder Markdown Audit (H:\Ardanu Fix\Arandu-maxi)

## Scope

- Read the markdown corpus in the canonical H:\Arandu Fix\Arandu-maxi location to capture project-relevant, non-redundant facts.
- Focus on items that should influence future work and decision continuity.

## Notable Findings

- `AGENTS.md` remains the top-level operational anchor and enforces:
  - **Mandatory memory-first workflow** via `docs/knowledge-base`
  - mandatory check for file locations before shell usage
  - strict docs location rule under `docs/knowledge-base`, `docs/plans`, and root docs
  - debug logging appearance standards and mandatory style for discovery logs.
- `WORKING_DIRECTORY_WARNING.md` documents a critical historical issue:
  - Wrong-path work occurred on `C:` (opencode worktree) versus canonical `H:` repo.
  - `C:` location is not the canonical git target for final changes and may not merge into user target.
  - Required branch merge notes are listed for `opencode/silent-comet` commits.
- `README.md` at repo root is currently **empty** (0 lines), so entrypoint guidance should come from `docs/USER-MANUAL.md` and `docs/INDEX.md`.
- `THIS-PROJECTS-CURRENT-STATE.md` and `docs/CURRENT_WORKING_STATE.md` are both present; the latter is in `docs/` and confirms the chat-history fix is fully operational.
- `docs/INDEX.md` is maintained as the documentation pointer index and includes `THIS-PROJECTS-CURRENT-STATE`, user manual, client guides, and key KB links.
- `docs/ROADMAP.md` is largely historical (last updated 2025-02-23), mostly marking task 7/8 as planned.
- Client connectivity docs were found and should be treated as active references:
  - `docs/LAN-CONNECTION-GUIDE.md`
  - `docs/OPENAI_PROXY_CLIENT_GUIDE.md`
  - both emphasize exact host/port/`/v1` formatting differences across clients.
- `docs/USER-MANUAL.md` contains full user-facing feature coverage and includes remote desktop and chat history behavior.
- ` .cursor/rules.md` reinforces mandatory operational constraints (dir checks, docs-first, memory-first, build/test before completion, documentation consolidation), matching AGENTS guidance.
- `Extra skills/memory-bank-setup-skill/*.md` files were also reviewed; they describe general Memory Bank MCP patterns and a `python_picotool`-oriented example set.
- The files indicate MCP configuration options and JSON/tool contracts but do not provide Arandu-specific runtime configuration details.

## Network Discovery Memory Anchors Already Present

- `docs/knowledge-base/2026-02-28-network-discovery-root-cause-fix.md`
- `docs/knowledge-base/2026-02-28-network-discovery-models-fix.md`
- `docs/knowledge-base/2026-02-28-network-discovery-bugs-fixed.md`
- `docs/knowledge-base/2026-02-28-network-discovery-complete-fix.md`

## Chat History Memory Anchors Already Present

- `docs/knowledge-base/2026-02-28-chat-history-freezing-fix.md`
- `docs/knowledge-base/2026-02-28-chat-history-button-fix.md`
- `docs/knowledge-base/chat-history-implementation-complete.md`
- `docs/knowledge-base/2026-02-28-chat-history-verification-run.md`

## Immediate Follow-up Actions (Tracked)

1. Keep using this path and logging model for all subsequent bug/fix entries.
2. Continue maintaining `THIS-PROJECTS-CURRENT-STATE.md` after each milestone.
3. Keep new KB entries in `docs/knowledge-base/` with date-prefixed filenames and evidence paths.
