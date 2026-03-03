# Network Discovery - Root Cause Analysis & Fix

**Date:** Feb 28, 2026
**Status:** ✅ FIXED

## Bugs Found & Fixed

### Bug #1: Port Extraction Broken (CRITICAL)
**File:** `backend/src/discovery.rs` line 292-298

**Problem:**
```rust
// OLD BROKEN CODE:
api_endpoint.split(':').last()
// For "http://192.168.1.5:8081/v1/models" returns "8081/v1/models"
// parse::<u16>() fails → returns default 8081
```

**Impact:**
- Peer discovered with wrong `api_port` stored in cache
- When fetching models, lookup by (IP, port) fails
- Models never stored → always shows "0 models"

**Fix:**
```rust
// NEW FIXED CODE:
api_endpoint.split('/').nth(2)  // Get "host:port"
    .and_then(|host_port| host_port.split(':').nth(1))  // Get port
    .and_then(|port_str| port_str.parse::<u16>().ok())
    .unwrap_or(8081)
```

### Bug #2: Property Name Mismatches
**File:** `frontend/desktop.js` lines 4782, 4801, 4868

**Problem:**
- Frontend expected: `peer.ip`, `peer.online`
- Backend sends: `peer.ip_address`, `peer.is_reachable`

**Impact:**
- IP shows as "Unknown IP"
- Online status detection broken

**Fix:** Changed all references to use correct property names

## Testing Instructions

1. **Install new version** on both PCs
2. **Clear old peer cache** (optional but recommended):
   - Disable discovery on both
   - Wait 30 seconds
   - Re-enable discovery
3. **Verify**:
   - Both PCs discover each other
   - Model count shows correctly (not "0")
   - Models appear under peer in right panel
   - Can click models to connect

## Files Modified
- `backend/src/discovery.rs` - Fixed port extraction
- `frontend/desktop.js` - Fixed property names

## New Builds
- `Arandu.exe` (11 MB)
- `Arandu_v0.5.5-beta_FINAL.msi` (7.0 MB) ⭐ RECOMMENDED

Location: `H:\Ardanu Fix\Arandu-maxi\`
