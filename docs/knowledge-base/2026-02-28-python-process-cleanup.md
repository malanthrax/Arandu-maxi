# Python Process Cleanup on Exit - Implementation

**Date:** 2026-02-28  
**Feature:** Kill Python server processes when app closes  
**Status:** ✅ IMPLEMENTED

## Problem
When closing Arandu, Python server processes (spawned by MCP servers or other features) were left running as orphaned processes, consuming system resources.

## Solution
Added comprehensive cleanup that kills all Python processes when the app exits.

## Changes Made

### 1. Added `kill_all_python_processes()` method in `backend/src/lib.rs`

```rust
pub fn kill_all_python_processes(&self) {
    // On Windows: Uses tasklist to find Python processes, then taskkill to terminate
    // On Unix: Uses pkill to kill all Python processes
}
```

**Windows Implementation:**
- Lists all `python.exe` processes using `tasklist`
- Parses CSV output to extract PIDs
- Kills each process with `taskkill /PID <pid> /F /T`
- Also attempts to kill `python3.exe`

**Unix Implementation:**
- Uses `pkill -9 -f python` and `pkill -9 -f python3`

### 2. Added `comprehensive_cleanup()` method

```rust
pub async fn comprehensive_cleanup(&self) {
    // 1. Kill tracked child processes
    self.cleanup_all_processes().await;
    
    // 2. Force cleanup any remaining
    self.force_cleanup_all_processes();
    
    // 3. Kill Python processes
    self.kill_all_python_processes();
}
```

### 3. Updated Exit Handlers

**Application Exit (`tauri://before-exit`):**
- Changed from `cleanup_all_processes()` to `comprehensive_cleanup()`

**Graceful Exit Command:**
- Changed from `cleanup_all_processes()` to `comprehensive_cleanup()`

**Restart Command:**
- Changed from `cleanup_all_processes()` to `comprehensive_cleanup()`

## Files Modified
- `backend/src/lib.rs` - Added new cleanup methods and updated exit handlers

## Build
```
Command: cargo build --release
Duration: 3m 19s
Output: target/release/Arandu.exe (11MB)
Timestamp: 2026-02-28 14:29
```

## How It Works

1. **Normal Exit:** When app closes (via tray menu or graceful exit), `comprehensive_cleanup()` is called
2. **Process Termination Order:**
   - First, kill tracked llama.cpp processes
   - Second, force kill any remaining tracked processes
   - Third, kill all Python processes system-wide
3. **Force Flag:** Uses `/F` (force) and `/T` (terminate tree) on Windows to ensure processes die

## Notes
- This kills ALL Python processes, not just those spawned by Arandu
- This is intentional to ensure no orphaned servers remain
- If you have other Python applications running, they will also be terminated when Arandu exits

## Testing
To verify:
1. Start Arandu
2. Open Task Manager, note any Python processes
3. Exit Arandu completely (not just hide to tray)
4. Verify Python processes are terminated
