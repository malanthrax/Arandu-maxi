# Arandu - Mandatory AI Agent Rules

âš ï¸ **CRITICAL: ALL AI agents working on this project MUST follow these rules.**

Failure to follow these rules results in:
- Lost work (working in wrong directory)
- Breaking existing features (missing context)
- Wasted time (searching for files manually)
- Failed builds (not testing properly)

---

## Rule 1: VERIFY WORKING DIRECTORY FIRST

**Before ANY other action, verify working directory:**

```bash
pwd
```

**MUST return:** `H:
Arandu Fix\Arandu-maxi` or `/h/Ardanu Fix/Arandu-maxi`

**If on C drive, STOP and switch:**
```bash
cd "H:
Ardanu Fix\Arandu-maxi"
```

**Why:** C drive has a diverged repository that is NOT synced with H drive. Changes on C drive won't appear in the actual project.

---

## Rule 2: READ ALL DOCUMENTATION FIRST

**You MUST read these files BEFORE making any changes:**

1. **WORKING_DIRECTORY_WARNING.md** (2 minutes)
   - Critical warnings about directory issues
   - Location: `H:
Ardanu Fix\Arandu-maxi\WORKING_DIRECTORY_WARNING.md`

2. **AGENTS.md** (5 minutes)
   - Architecture and patterns
   - How-to guides
   - Common issues
   - Location: `H:
Ardanu Fix\Arandu-maxi\AGENTS.md`

3. **THIS-PROJECTS-CURRENT-STATE.md** (3 minutes)
   - Current status and build info
   - Recent bug fixes
   - What's already been done
   - Location: `H:
Ardanu Fix\Arandu-maxi\THIS-PROJECTS-CURRENT-STATE.md`

**Time investment:** ~10 minutes  
**Time saved:** Hours of mistakes and rework

---

## Rule 3: USE KNOWLEDGE BASE MEMORY FIRST

**Before using shell commands (grep, find, ls) to locate files:**

1. Search memory for file locations:
   ```
   Search: "Arandu Complete File Location Reference"
   ```

2. If not found, search for the specific feature:
   ```
   Search: "Arandu [feature-name]"
   ```

3. Only use shell commands as FALLBACK

**Why memory is better:**
- Faster (no filesystem access)
- Has task-based quick reference (e.g., "right-click menu = desktop.js")
- Always up to date with latest file locations
- Saves context window tokens

---

## Rule 4: BUILD AND TEST BEFORE CLAIMING SUCCESS

**After making changes, you MUST:**

1. **Build the application:**
   ```bash
   cd "H:
Ardanu Fix\Arandu-maxi\backend"
   cargo tauri build
   ```

2. **Verify build succeeds:**
   - âœ… 0 errors (warnings OK)
   - Output: `backend/target/release/Arandu.exe`

3. **Test the feature:**
   - Run the application
   - Verify your changes work
   - Check for console errors

**DO NOT claim work is complete without building and testing.**

---

## Rule 5: SAVE TO MEMORY AFTER COMPLETING WORK

**When you finish a feature or fix:**

1. **Update existing memories** if you modified files they reference
2. **Create new memories** for new features with:
   - Clear title (e.g., "Arandu Network Widget Implementation")
   - Complete file paths and line numbers
   - What methods/functions were added
   - How it integrates with existing code
   - Importance: 0.8-1.0 for critical features
   - Labels: arandu, frontend/backend, feature-name

3. **Update THIS-PROJECTS-CURRENT-STATE.md** with:
   - What was implemented
   - Files modified
   - Build status
   - Any testing notes

---

## Rule 6: FOLLOW EXISTING PATTERNS

**When adding new code:**

1. **UI Elements:**
   - Use glassmorphism CSS pattern
   - Use theme variables (never hardcode colors)
   - Follow z-index hierarchy
   - Position with CSS classes, not inline styles

2. **JavaScript:**
   - Use existing class methods as templates
   - Always use `this.getInvoke()` for Tauri API
   - Always wrap in try-catch
   - Always await promises

3. **Rust:**
   - Use `Result<T, String>` for commands
   - Register commands in invoke_handler
   - Use platform-agnostic paths (PathBuf)

4. **Error Handling:**
   - Show user-friendly notifications
   - Log detailed errors to console
   - Never crash silently

---

## Rule 7: UPDATE DOCUMENTATION

**After completing work:**

1. **Add to THIS-PROJECTS-CURRENT-STATE.md:**
   ```markdown
   ## New Feature: [Feature Name]
   **Status:** COMPLETE âœ…
   **Date:** [YYYY-MM-DD]
   
   **Description:** What it does
   
   **Files Modified:**
   - `frontend/...` - What changed
   - `backend/...` - What changed
   
   **Build Status:** âœ… SUCCESS
   ```

2. **Update relevant sections in AGENTS.md** if you:
   - Added new commands
   - Changed architecture
   - Fixed common issues

3. **Update README.md** if user-facing features changed

---

## Quick Reference

### Project Location
```
H:
Ardanu Fix\Arandu-maxi\
```

### Build Command
```bash
cd backend
cargo tauri build
```

### Key Files
- Main logic: `frontend/desktop.js`
- Backend commands: `backend/src/lib.rs`
- Current status: `THIS-PROJECTS-CURRENT-STATE.md`
- Architecture: `AGENTS.md`

### Memory Search Keywords
- "Arandu Complete File Location Reference" - All file paths
- "Arandu Common Development Patterns" - How-to guides
- "Arandu Critical Gotchas" - Common mistakes
- "Arandu Agent Onboarding" - Complete workflow
- "Arandu [feature-name]" - Specific features

---

## Consequences of Breaking Rules

| Rule Broken | Consequence |
|-------------|-------------|
| Rule 1 (Wrong directory) | Lost work, changes don't appear in build |
| Rule 2 (Skip docs) | Break existing features, duplicate work |
| Rule 3 (No memory) | Wasted time searching, wasted tokens |
| Rule 4 (No testing) | Broken builds, failed releases |
| Rule 5 (No memory save) | Next agent wastes time rediscovering |
| Rule 6 (Wrong patterns) | Inconsistent code, maintenance issues |
| Rule 7 (No docs) | Outdated docs, confused users/agents |

---

## Self-Check Before Starting

Ask yourself:
- [ ] Am I on H drive (`H:
Arandu Fix\Arandu-maxi`)?
- [ ] Did I read WORKING_DIRECTORY_WARNING.md?
- [ ] Did I read AGENTS.md?
- [ ] Did I read THIS-PROJECTS-CURRENT-STATE.md?
- [ ] Did I check memory for file locations?
- [ ] Do I understand what I'm building?
- [ ] Will I build and test before finishing?
- [ ] Will I save to memory and update docs?

**If any answer is NO, go back and do it.**

---

## Emergency Contacts

If you're stuck:
1. Check AGENTS.md Architecture section
2. Search memory for similar patterns
3. Read the code comments
4. Look at how similar features are implemented
5. Ask for help with specific error messages

**Do NOT guess or assume. This codebase has specific patterns for a reason.**

---

*These rules exist because previous agents learned the hard way. Follow them.*


---

## Rule 8: CONSOLIDATED DOCUMENTATION

**All documentation MUST be kept within the Arandu-maxi folder.**

- **Knowledge Base:** docs/knowledge-base/
- **Implementation Plans:** docs/plans/
- **Root Docs:** README.md, AGENTS.md, THIS-PROJECTS-CURRENT-STATE.md

**NEVER** create documentation in the parent H:\Ardanu Fix\ directory or other sibling folders. This prevents fragmentation and ensures all agents have the same context.
