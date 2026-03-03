Remote Model Launch - Testing Checklist
========================================

DATE: 2026-03-01
BUILT: backend/target/release/Arandu.exe (Mar 1, 15:38)

## Prerequisites

1. **Server Setup**:
   - Launch Arandu on Machine A with Discovery enabled
   - Note Server IP (e.g., 192.168.1.100)
   - Note API port (default: 8081, or custom from settings)
   - Ensure at least one GGUF model exists on server
   - Server's models_directory path will be used for remote launches

2. **Client Setup**:
   - Launch Arandu on Machine B with Discovery enabled
   - Wait for server to appear in discovered peers
   - Click Tracker button → Remote Models (if using list view)

## Test 1: End-to-End Remote Launch (CRITICAL)

**Steps**:
1. On Client Machine B, wait for Machine A to appear in discovery
2. Expand Machine A to see available models
3. Click on a remote model (e.g., "Llama-3-8B-instruct-Q4.gguf")
4. Watch toast notifications:
   - ✅ "Requesting model launch: [model name]..."
   - ✅ "Model [model name] is ready!" (or error message)
5. Verify chat window opens with:
   - Title: "Remote Chat - [model name] ([ip:port])"
   - Header showing Process ID
   - "Stop Model" button visible
   - Iframe loading llama-server web UI
6. Try sending a message in the chat

**Expected Results**:
- Toast sequence shows proper state transitions
- Model launches on Server Machine A
- Chat window opens on Client Machine B
- Messages can be sent successfully
- Taskbar shows cloud icon for remote session

**Failure Indicators**:
- ❌ "Failed to launch model: [reason]" toast
- ❌ Error window with connection message
- ❌ Chat window opens but iframe shows "Connection refused"
- ❌ No process_id displayed in header

## Test 2: Concurrent Remote Access

**Steps**:
1. Complete Test 1 on Client Machine B (model running)
2. Launch Arandu on Machine C (third machine on LAN)
3. Enable Discovery on Machine C
4. Wait for Machine A to appear in discovery
5. Click the SAME model that Machine B is using
6. Watch toast notifications on Machine C
7. Try sending messages from both Machine B and Machine C simultaneously

**Expected Results**:
- Machine C shows "Model [model name] is ready!" (should NOT launch new model if already running)
- Both clients can send messages to same model
- Responses alternate between clients
- Server shows only one llama-server process for that model

**Failure Indicators**:
- ❌ Machine C launches duplicate llama-server
- ❌ Clients cannot connect simultaneously
- ❌ Messages from one client not received by other

## Test 3: Stop Model Functionality

**Steps** (with model running from Test 1):
1. On Client Machine B, with remote chat window open
2. Click "Stop Model" button in window header
3. Watch toast notification: "Stopping model..." → "Model stopped successfully"
4. Verify chat window remains open but shows "Connection refused" in iframe
5. Try to send a message (should fail)

**Expected Results**:
- Toast shows stop confirmation
- llama-server process terminates on Server Machine A
- Chat window remains open but cannot send messages
- If client tries to open model again, it re-launches

**Failure Indicators**:
- ❌ Toast shows "Failed to stop model: [reason]"
- ❌ Model continues running on server
- ❌ Multiple stop attempts create errors

## Test 4: Error Handling - Invalid Model

**Steps**:
1. Modify client code to request non-existent model path (or test via Postman):
   ```bash
   curl -X POST http://192.168.1.100:8081/api/models/launch \
     -H "Content-Type: application/json" \
     -d '{"model_path": "/invalid/path/model.gguf"}'
   ```
2. Observe error response

**Expected Results**:
- Server returns `{success: false, message: "Failed to launch model: ...", process_id: null}`
- No llama-server process is created
- Client shows error toast and error window

**Failure Indicators**:
- ❌ Server crashes or hangs
- ❌ Returns 500 error instead of structured JSON

## Test 5: Error Handling - Server Offline

**Steps**:
1. On Client Machine A, with no server running
2. Manually trigger launch to unreachable server via Postman:
   ```bash
   curl -X POST http://192.168.1.999:8081/api/models/launch \
     -H "Content-Type: application/json" \
     -d '{"model_path": "/path/to/model.gguf"}'
   ```
   (note: use unreachable IP)

**Expected Results**:
- Fetch API throws network error on client
- Toast shows "Error launching model: Failed to fetch" or similar
- Error window shows "Launch request failed: [network error]"

**Failure Indicators**:
- ❌ Client hangs indefinitely
- ❌ No error shown to user

## Test 6: List Active Models API

**Steps** (after Test 1, with model running):
1. Via Postman: `GET http://192.168.1.100:8081/api/models/active`
2. Inspect response

**Expected Results**:
```json
{
  "success": true,
  "models": [
    {
      "process_id": "uuid-string",
      "model_path": "/path/to/model.gguf",
      "model_name": "model.gguf",
      "host": "",
      "port": 0,
      "server_host": "127.0.0.1",
      "server_port": 8080,
      "status": "Ready",
      "launched_at": "2026-03-01T12:34:56Z"
    }
  ]
}
```
- model_path matches launched model
- status is "Ready" (not "Starting" or "Failed")

**Failure Indicators**:
- ❌ Empty models array when model is running
- ❌ Incorrect model_path
- ❌ status stuck in "Starting"

## Test 7: Multiple Sequential Launches

**Steps**:
1. Launch model A → stop model A
2. Launch model B → stop model B
3. Launch model A again
4. Check `/api/models/active` after each step

**Expected Results**:
- Only one model active at a time
- Stopped models disappear from active list
- Re-launching same model works correctly

**Failure Indicators**:
- ❌ Multiple models accumulate in active list
- ❌ Cannot re-launch stopped model
- ❌ Port conflicts on sequential launches

## Test 8: Window Management

**Steps**:
1. Launch remote model (chat window opens)
2. Open a local model (different chat window)
3. Open a second remote model (third window)
4. Switch between windows using desktop/alt-tab
5. Close remote chat window by clicking X

**Expected Results**:
- Each window has unique ID
- Taskbar shows all three items
- Windows layer correctly (clicked one comes to front)
- Closing remote window does not stop model (only stop button does)

**Failure Indicators**:
- ❌ Windows overlap or disappear
- ❌ Closing window stops model unexpectedly
- ❌ Taskbar items confuse different sessions

## API Testing Commands

### Test Launch Endpoint
```bash
curl -X POST http://192.168.1.100:8081/api/models/launch \
  -H "Content-Type: application/json" \
  -d '{
    "model_path": "C:/Users/Server/.Arandu/models/llama-3-8b-instruct-q4.gguf",
    "server_host": "192.168.1.100",
    "server_port": 8081
  }'
```

### Test Stop Endpoint
```bash
curl -X POST http://192.168.1.100:8081/api/models/stop \
  -H "Content-Type: application/json" \
  -d '{
    "process_id": "uuid-from-launch-response"
  }'
```

### Test Active Models Endpoint
```bash
curl -X GET http://192.168.1.100:8081/api/models/active
```

## Success Criteria

**Implementation is complete when**:
- ✅ Test 1 passes (basic remote launch works)
- ✅ Test 3 passes (model stops correctly)
- ✅ Tests 4-5 pass (error handling works)
- ✅ At least one of Test 2 or Test 6 passes (optional for MVP)

**For production release**:
- All 8 tests must pass
- Documentation updated in AGENTS.md and docs/USER-MANUAL.md

## Notes for Tester

- Check `/api/models/active` before/after each test to verify server state
- Monitor Server Machine A's Task Manager to verify llama-server.exe processes
- Check browser console on client for any errors
- Use different browser tabs/windows to simulate concurrent clients
- Test with small models first (Q4 quantization) for faster launches
- Document any non-standard network configurations (firewall, antivirus)

## Known Issues to Watch For

1. **Model Path Issues**: Server uses relative or absolute paths inconsistently
   - Workaround: Use full absolute path in POST body

2. **Process ID Tracking**: stop_model may fail if process_id doesn't match
   - Workaround: Server auto-cleans active_models on restart

3. **Concurrent Launch Race**: Two clients launching same model simultaneously
   - Workaround: Not critical for MVP, would need distributed lock