# 2026-03-01 — OpenCode Project Config Hardening Pass

- Extended `opencode.json` with an explicit `default_agent` value so project sessions
  consistently start with the `build` agent.
- Kept `instructions` pointing at both:
  - `AGENTS.md`
  - `WORKING_DIRECTORY_WARNING.md`
- Validated syntax with a direct JSON parse check via Node in the canonical workspace:
  - `node -e "... JSON.parse(fs.readFileSync(p, 'utf8')) ..."`
- Config is now expected to provide deterministic project-level behavior independent of
  stale alternate worktrees.
