# Documentation Folder - README

## Purpose

This folder contains complete documentation for the Arandu project, including user guides, technical documentation, implementation plans, and a knowledge base of fixes and improvements.

---

## Folder Structure

```
docs/
├── INDEX.md                    # Main documentation index
├── USER-MANUAL.md              # Complete user guide ⭐
├── OPENAI_PROXY_CLIENT_GUIDE.md # Client configuration guide
├── LAN-CONNECTION-GUIDE.md     # LAN troubleshooting guide
├── CHANGELOG.md                # Project changelog
├── knowledge-base/             # Session logs, fixes, and memory
│   ├── 2026-03-01-*.md        # Latest fixes (CORS and Port Config) ⭐
│   ├── 2026-02-*.md            # February 2026 fixes
│   ├── 2025-*.md               # Older fixes
│   └── *.md                    # General documentation
├── plans/                      # Implementation plans
└── README.md                   # This file
```

---

## Quick Start

### For Users
1. Read **USER-MANUAL.md** - Complete usage instructions
2. Check **LAN-CONNECTION-GUIDE.md** - If experiencing connection issues
3. Review **OPENAI_PROXY_CLIENT_GUIDE.md** - For client integration

### For Developers
1. Start with **INDEX.md** - Complete documentation overview
2. Check **THIS-PROJECTS-CURRENT-STATE.md** (in project root) - Current build status
3. Review **AGENTS.md** (in project root) - Architecture and patterns
4. Check **knowledge-base/** - Latest fixes and session logs

---

## Knowledge Base Organization

### Session Organization
All significant sessions are documented with dated entries:
- Format: `YYYY-MM-DD-description.md`
- Example: `2026-03-01-port-config-and-cors-fix.md`

### Entry Types

#### Fix Documentation (`*-fix.md`)
- Problem description
- Root cause analysis
- Solution with file locations
- Verification steps
- Impact notes

#### Session Summaries (`session-summary-*.md`)
- Task execution flow
- Subagent performance notes
- Files modified
- Commits created
- Success criteria

#### Reference Documentation (`*-reference*.md`)
- File location guides
- Quick reference tables
- Configuration examples

#### Memory Entries (`*-complete.md`)
- Session statistics
- Critical issue resolutions
- Next steps
- Key learnings

---

## Latest Critical Fixes (2026-03-01) ⭐⭐⭐

### CORS and Port Configuration Fixes
**Issue:** Remote chat windows showing white screens, port configuration incomplete

**Documentation:**
1. **2026-03-01-port-config-and-cors-fix.md** - Complete fix documentation
2. **2026-03-01-session-summary-cors-port-fixes.md** - Session execution flow
3. **2026-03-01-file-location-reference-cors-port-fixes.md** - File locations
4. **2026-03-01-critical-fixes-complete-cors-port.md** - Memory entry

**Status:** ✅ All issues resolved, ready for testing

---

## Documentation Conventions

### File Naming
- Use lowercase with hyphens
- Include date in session files: `YYYY-MM-DD`
- Use descriptive name: `feature-name` or `fix-description`

### Markdown Formatting
- Use H1 (`#`) for file title only
- Use H2 (`##`) for main sections
- Use H3 (`###`) for subsections
- Use **bold** for emphasis
- Use `code` for file paths and inline code
- Use ``` for code blocks

### Code References
Always include:
- File path (relative from project root)
- Line numbers when applicable
- Function/method names
- Commit references when relevant

**Example:**
```
**Location:** `backend/src/process.rs:222`
**Function:** `launch_model_server()`
**Commit:** e1fd9ba
```

---

## Updating Documentation

### When to Update
- After every significant fix or feature
- When changing file structure or API
- When updating project status
- After completing a major implementation phase

### Required Updates (Per User Policy)
1. AGENTS.md (if architecture changed)
2. THIS-PROJECTS-CURRENT-STATE.md (if status changed)
3. Knowledge base entry (dated)
4. INDEX.md (if new major entry)
5. CHANGELOG.md (if user-facing change)

### Process
1. Create dated knowledge base entry
2. Update root documentation files (AGENTS.md, THIS-PROJECTS-CURRENT-STATE.md)
3. Update INDEX.md if new major section
4. Update CHANGELOG.md if user-facing
5. Commit with clear message

---

## Navigation Aids

### Finding Specific Information

**By Topic:**
- Check **INDEX.md** - Categorized listings
- Search knowledge base folder for keywords
- Check THIS-PROJECTS-CURRENT-STATE.md for recent issues

**By Date:**
- Sort `knowledge-base/` by date
- Latest fixes in `2026-03-01-*.md` files
- Search `git log` for commit references

**By File:**
- See `*-reference*.md` files for file location guides
- Check AGENTS.md for architecture overview
- Use `grep` in knowledge base for specific files

---

## Documentation Quality Checklist

- [ ] Clear problem statement (for fixes)
- [ ] Root cause analysis included (for fixes)
- [ ] Solution with exact file locations and line numbers
- [ ] Verification steps or test cases
- [ ] Impact notes (what changes, what breaks, backwards compatibility)
- [ ] Date-stamped
- [ ] References to commits or git history
- [ ] Links to related documentation

---

## Contributing Guidelines

### Adding New Entries

1. **Create dated entry:** `YYYY-MM-DD-description.md`
2. **Follow conventions:**
   - H1 for title
   - H2 for main sections
   - Code blocks for all code
   - File paths in backticks
3. **Include required sections:**
   - Date and session status
   - Problem/Goal description
   - Solution/Implementation details
   - File locations and line numbers
   - Verification steps
   - Impact and next steps
4. **Update INDEX.md:**
   - Add to appropriate section
   - Mark with ⭐ for critical entries
   - Keep sorted by date

### Updating Existing Entries

1. Add update note at top with date
2. Preserve original content (don't delete, append)
3. Use bold to highlight latest changes
4. Update THIS-PROJECTS-CURRENT-STATE.md if status changed

---

## Emergency Documentation

If a critical issue is discovered and needs immediate documentation:

1. Create entry with `CRITICAL` or `EMERGENCY` in filename
2. Include brief summary at top file
3. Document all known workarounds
4. Link to any temporary fixes
5. Update THIS-PROJECTS-CURRENT-STATE.md immediately
6. Commit with `[EMERGENCY]` prefix

---

## Documentation Maintenance

### Regular Reviews
- Monthly: Check for outdated information
- Per Release: Update changelog, review all entries
- Per Major Version: Archive old knowledge-base entries

### Archiving Strategy
- Older than 6 months: Move to `knowledge-base/archive/`
- Keep active fixes: In main `knowledge-base/` folder
- Reference-only: Keep in `reference/` subfolder

---

## Related Files (Project Root)

- **README.md** - Main project overview
- **AGENTS.md** - Architecture and agent guidelines
- **THIS-PROJECTS-CURRENT-STATE.md** - Current project status
- **WORKING_DIRECTORY_WARNING.md** - Build warnings
- **CHANGELOG.md** - User-facing changelog

---

## Support

For questions about documentation:
1. Check INDEX.md first
2. Search knowledge base for similar issues
3. Review THIS-PROJECTS-CURRENT-STATE.md
4. Check git history for recent changes

---

**Last Updated:** 2026-03-01
**Maintainer:** Development Team
**Status:** Active ✅