# Arandu Documentation Consolidation & Rules (2026-02-23)

## Consolidation Summary
All project documentation and "knowledge memory" files have been moved into the main project folder to prevent context fragmentation.

**Moved Locations:**
- Old: `H:\Ardanu Fix\nowledge-mem\` -> New: `H:\Ardanu Fix\Arandu-maxi\docs\knowledge-base\`
- Old: `H:\Ardanu Fix\IMPLEMENTATION-PLAN-...md` -> New: `H:\Ardanu Fix\Arandu-maxi\docs\plans\`
- Old: `H:\Ardanu Fix\frontend\llama-custom` (stray folder) -> Deleted (Duplicate of `Arandu-maxi\frontend\llama-custom`)

## Mandatory Documentation Rule
**ALL documentation, plans, and knowledge base files MUST reside within the `H:\Ardanu Fix\Arandu-maxi\` directory.**

1.  **NO EXTERNAL FILES:** Do not create documentation files in the root `H:\Ardanu Fix\` or any other parent/sibling folders.
2.  **KNOWLEDGE BASE:** Use `docs/knowledge-base/` for permanent facts, network configs, and architectural discoveries.
3.  **PLANS:** Use `docs/plans/` for implementation plans and session logs.
4.  **ROOT DOCS:** Keep `README.md`, `AGENTS.md`, `THIS-PROJECTS-CURRENT-STATE.md`, and `WORKING_DIRECTORY_WARNING.md` in the `Arandu-maxi` root for immediate visibility.

## Save & State Discipline
This is a mandatory operating rule:

1.  **Save progress updates immediately** after each meaningful verification or fix milestone in `THIS-PROJECTS-CURRENT-STATE.md`.
2.  **Persist long-lived facts** in `docs/knowledge-base/` as soon as they become stable.
3.  **Do not defer release-blocker notes**; if a build or runtime blocker is discovered (for example bundle/packaging failures), log it in the “Future Work Planned” section before continuing other work.
4.  **When file locations are discovered or changed, update `docs/knowledge-base` first** before other documentation writes that reference those paths.

5.  **No documentation files may be created outside `H:\Ardanu Fix\Arandu-maxi`**. If any new `.md` must be authored, it must be inside this folder.

## Search Pattern Update
When searching for project facts, always target:
`H:\Ardanu Fix\Arandu-maxi\docs\knowledge-base\`
