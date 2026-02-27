# MCP Runtime Smoke Checklist

## Objective
Validate MCP connection CRUD flow in a running Arandu app after startup persistence is in place.

## Checklist

1. Start Arandu from a fresh launch.
2. Open the **Network** widget and switch to MCP panel.
3. Add a URL transport entry
   - Name: `Local MCP (HTTP)`
   - Transport: `HTTP`
   - URL: `http://127.0.0.1:8080/mcp`
   - Timeout: `10`
   - Save
4. Validate row appears with `Never tested` status.
5. Click **Test** and verify either:
   - Success: status moves to connected, green badge, success toast, or
   - Failure: readable failure message and status updates to error.
6. Add a stdio transport entry
   - Name: `Local MCP (stdio)`
   - Transport: `STDIO`
   - Command: path to executable
   - Args: `[]`
   - Save
7. Test this second entry and verify status updates correctly.
8. Toggle `Enabled` off/on on each entry and confirm state persists in UI.
9. Edit one entry and re-save; verify update in list and no duplicate IDs.
10. Delete one entry and confirm list + storage remove it.
11. Close app and restart.
12. Re-open Network/MCP panel and verify all remaining entries restored with latest metadata.

## Pass criteria
- Add/Edit/Delete/Toggle/Test all work without crash.
- Form validation blocks invalid inputs (missing name, missing command/url).
- Entries survive restart and remain queryable via `get_mcp_connections`.
