# Network Discovery Models Bug Fix - 2026-02-28

## Problem
Network discovery was finding peers but showing "0" models in:
1. Settings → Network Discovery table
2. Main desktop scroll view (right panel empty)

## Root Cause
Frontend expected wrong data structure from backend.

**Frontend expected:**
```javascript
{ peers: [...] }
```

**Backend returned:**
```javascript
[...]  // Direct array
```

## Fix
Changed `pollDiscoveredPeers()` in `frontend/desktop.js` line 4523:

**Before:**
```javascript
if (result && Array.isArray(result.peers)) {
    this.discoveredPeers = result.peers;
```

**After:**
```javascript
if (result && Array.isArray(result)) {
    this.discoveredPeers = result;
```

## Files Modified
- `frontend/desktop.js` - Fixed data structure handling

## New Installers
Both rebuilt with fix:
- `Arandu_v0.5.5-beta_Installer_Fixed.msi` (7.0 MB)
- `Arandu_v0.5.5-beta_Setup_Fixed.exe` (4.4 MB)

## Testing
Remote models should now:
1. Show model count in Settings table
2. Appear in right panel of scroll view
3. Be clickable to connect

## Location
`H:\Ardanu Fix\Arandu-maxi\`
