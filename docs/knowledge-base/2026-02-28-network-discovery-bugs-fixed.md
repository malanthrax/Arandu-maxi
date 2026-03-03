# Network Discovery Bug Fixes - Feb 28, 2026

## Bug 1: Split View Half Screen Issue
**Problem:** Split view only showed half screen, need edge-to-edge with buffer

**Fix:** Updated CSS in `desktop.css`:
- Changed `.desktop-icons.list-view` to use full viewport width
- Set `left: 0; right: 0; width: 100vw;`
- Updated split container in `desktop.js` to use `width: calc(100% - 40px)` with 20px margin

## Bug 2: Remote Models Show 0
**Problem:** Remote PCs discovered but always show 0 models

**Root Cause:** In `get_discovered_peers()`, peers were cloned BEFORE fetching models, so returned peers didn't have the fetched models.

**Fix:** In `lib.rs`:
1. Fetch models for each peer
2. Re-fetch peers from cache AFTER all fetches complete
3. Return the updated peers with models

**Files Modified:**
- `backend/src/lib.rs` - Fixed get_discovered_peers to return updated peers
- `frontend/css/desktop.css` - Fixed list view to use full width
- `frontend/desktop.js` - Added margin to split container

## New Builds
All rebuilt with fixes:
- `Arandu.exe` (11 MB)
- `Arandu_v0.5.5-beta_Fixed.msi` (7.0 MB) ⭐ RECOMMENDED
- `Arandu_v0.5.5-beta_Fixed.exe` (4.4 MB)

Location: `H:\Ardanu Fix\Arandu-maxi\`
