# Targeted Python Process Cleanup - Implementation

**Date:** 2026-02-28  
**Feature:** Kill only Python processes spawned by Arandu (not all Python on system)  
**Status:** ✅ IMPLEMENTED

## Problem
Previous implementation killed ALL Python processes on the system, which was too destructive and would interfere with other applications.

## Solution
Track only the Python process PIDs that Arandu spawns, then kill only those specific processes on exit.

## Changes Made

### 1. Added `python_processes` field to AppState

```rust
pub struct AppState {
    // ... other fields ...
    pub python_processes: Arc<Mutex<Vec<u32>>>, // Track Python process PIDs
}
```

### 2. Added tracking methods

**`register_python_process(pid)`** - Call when spawning a Python process
- Adds PID to tracking list
- Located in `test_mcp_connection` for MCP stdio connections

**`unregister_python_process(pid)`** - Call when Python process dies naturally
- Removes PID from tracking list
- Available for future use

**`kill_tracked_python_processes()`** - Kill only tracked PIDs
- Gets list of tracked PIDs
- Kills each one individually with `taskkill /PID <pid> /F /T` (Windows) or `kill -9 <pid>` (Unix)
- Clears tracking list after cleanup
- Only kills Arandu-spawned processes, not all Python on system

### 3. Updated `comprehensive_cleanup()`

Now uses `kill_tracked_python_processes()` instead of killing all Python processes.

### 4. Added tracking to MCP stdio connections

When spawning an MCP stdio connection, if the command contains "python", the PID is registered:

```rust
if cmd_lower.contains("python") {
    if let Some(pid) = child.id() {
        state.register_python_process(pid).await;
    }
}
```

## Behavior

1. When Arandu spawns a Python process (e.g., for MCP server), the PID is tracked
2. When Arandu exits, only those tracked PIDs are killed
3. Other Python processes on the system remain unaffected

## Files Modified
- `backend/src/lib.rs` - Added tracking field, methods, and integration

## Build
```
Command: cargo build --release
Duration: 3m 07s
Output: target/release/Arandu.exe (11MB)
Timestamp: 2026-02-28 14:41
```

## Testing
To verify:
1. Start Arandu
2. Open Task Manager, note Python processes
3. Start an MCP connection that uses Python
4. Note the new Python process PID
5. Exit Arandu
6. Verify only that specific Python process is terminated
7. Verify other Python processes remain running
