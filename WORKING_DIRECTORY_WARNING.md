# CRITICAL: WORKING DIRECTORY WARNING

## Issue
- Agent may work in wrong directory (e.g., C drive instead of H drive)
- C drive location: `C:\Users\Gordprime\.local\share\opencode\worktree\b4ce4e27cbb87ed1bf7aace61dd65e2524173d4c\silent-comet`
- H drive location: `H:\Ardanu Fix\Arandu-maxi` (CORRECT PROJECT LOCATION - USE THIS)

## When This Happened
1. First occurrence: Unknown/earlier work
2. Second occurrence: 2025-02-20 - HF Search Autofill implementation
3. Third occurrence: Multiple times during HF Search Model ID Display attempts

## Git Worktree Status
```
H:/Ardanu Fix/Arandu-maxi                                                                                 9909581 [main]
C:/Users/Gordprime/.local/share/opencode/worktree/b4ce4e27cbb87ed1bf7aace61dd65e2524173d4c/silent-comet   b054396 [opencode/silent-comet]
```

## CRITICAL - This is NOT a worktree!
The C drive location is NOT a git worktree - it's a git clone that diverged from the H drive repository. Using `workdir` parameter in bash commands does NOT sync changes.

## Impact
- Changes made to C drive files do NOT affect H drive repository
- Builds on H drive do NOT include C drive changes until manually copied/merged
- This caused the HF Search Model ID Display feature to appear to not work

## Resolution
ALWAYS verify working directory before starting work:
```bash
cd "H:\Ardanu Fix\Arandu-maxi"  # Force switch to H drive
pwd                              # Should be: /h/Ardanu Fix/Arandu-maxi
```

**NEVER rely on `workdir` parameter alone.** It sets the shell working directory for a single command, but subsequent operations return to the default C drive path.

## Build Location
Always use: `H:\Ardanu Fix\Arandu-maxi\backend\target\release\Arandu.exe`
H:/Ardanu Fix/Arandu-maxi                                                                                 9909581 [main]
C:/Users/Gordprime/.local/share/opencode/worktree/b4ce4e27cbb87ed1bf7aace61dd65e2524173d4c/silent-comet   b054396 [opencode/silent-comet]
```

## Commits on C Drive Needing Merge
Branch: `opencode/silent-comet` (vs `main`)
- b054396 feat: add CSS styles for date comparison badges
- c13b4f0 feat: clear comparison context on manual search
- 3eafde6 feat: add date comparison badge generation methods
- 5cd3f17 feat: add comparison context methods to HF app
- f40659b fix: correct module access and indentation errors in handleCheckUpdate

## Resolution
ALWAYS verify working directory before starting work:
```bash
pwd  # Should be: /h/Ardanu Fix/Arandu-maxi
```

Build location: `H:\Ardanu Fix\Arandu-maxi\backend\target\release\Arandu.exe`

## Action Required
Merge opencode/silent-comet branch into main if changes haven't been applied:
```bash
cd "H:\Ardanu Fix\Arandu-maxi"
git merge opencode/silent-comet
```