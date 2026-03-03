# Network Discovery Debug Logging - Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Create a toggle-able debug log window on the main model page that shows all network discovery traffic between PCs

**Architecture:** 
- Backend (Rust): Discovery service emits detailed logs via Tauri events
- Frontend (JS): Toggle button on desktop opens floating log window
- Log window displays timestamped entries with send/receive data, IPs, and model info
- Window can be toggled open/closed with same button

**Tech Stack:** Tauri v2 events, Rust logging, Vanilla JS, CSS

---

## Task 1: Add Debug Toggle Button to Desktop

**Files:**
- Modify: `frontend/desktop.js` (add button to UI)
- Modify: `frontend/css/desktop.css` (button styling)
- Modify: `frontend/index.html` (if needed for container)

**Step 1: Add debug toggle button to desktop toolbar**

In `frontend/desktop.js`, find the dock/taskbar area and add a debug button:

```javascript
// Add to desktop initialization or toolbar creation
createDebugToggleButton() {
    const toolbar = document.getElementById('desktop-toolbar') || document.querySelector('.taskbar');
    if (!toolbar) return;
    
    const debugBtn = document.createElement('button');
    debugBtn.id = 'discovery-debug-toggle';
    debugBtn.className = 'toolbar-btn debug-btn';
    debugBtn.innerHTML = '<span class="material-icons">bug_report</span>';
    debugBtn.title = 'Toggle Discovery Debug Log';
    debugBtn.onclick = () => this.toggleDebugLogWindow();
    
    toolbar.appendChild(debugBtn);
}
```

**Step 2: Add CSS for debug button**

In `frontend/css/desktop.css`:

```css
.debug-btn {
    background: rgba(255, 193, 7, 0.2);
    border: 1px solid rgba(255, 193, 7, 0.5);
    color: #ffc107;
    transition: all 0.2s;
}

.debug-btn:hover {
    background: rgba(255, 193, 7, 0.3);
    box-shadow: 0 0 8px rgba(255, 193, 7, 0.4);
}

.debug-btn.active {
    background: rgba(255, 193, 7, 0.5);
    box-shadow: 0 0 12px rgba(255, 193, 7, 0.6);
}
```

**Step 3: Commit**

```bash
git add frontend/desktop.js frontend/css/desktop.css
git commit -m "feat: add discovery debug toggle button"
```

---

## Task 2: Create Floating Debug Log Window

**Files:**
- Modify: `frontend/desktop.js`
- Create: `frontend/css/debug-log.css` (or add to desktop.css)

**Step 1: Create window toggle function**

```javascript
// Add to DesktopManager class
toggleDebugLogWindow() {
    const existingWindow = document.getElementById('discovery-debug-window');
    const toggleBtn = document.getElementById('discovery-debug-toggle');
    
    if (existingWindow) {
        existingWindow.remove();
        toggleBtn?.classList.remove('active');
        this.debugLogWindowOpen = false;
    } else {
        this.createDebugLogWindow();
        toggleBtn?.classList.add('active');
        this.debugLogWindowOpen = true;
    }
}

createDebugLogWindow() {
    const windowEl = document.createElement('div');
    windowEl.id = 'discovery-debug-window';
    windowEl.className = 'debug-log-window';
    windowEl.innerHTML = `
        <div class="debug-log-header">
            <span class="material-icons">network_check</span>
            <span>Discovery Debug Log</span>
            <button class="close-btn" onclick="desktop.toggleDebugLogWindow()">×</button>
        </div>
        <div class="debug-log-content" id="debug-log-content">
            <div class="log-entry info">
                <span class="timestamp">${new Date().toLocaleTimeString()}</span>
                <span class="message">Debug log started...</span>
            </div>
        </div>
        <div class="debug-log-controls">
            <button onclick="desktop.clearDebugLog()">Clear</button>
            <button onclick="desktop.exportDebugLog()">Export</button>
        </div>
    `;
    
    // Position window
    windowEl.style.cssText = `
        position: fixed;
        top: 80px;
        right: 20px;
        width: 600px;
        height: 400px;
        z-index: 9999;
    `;
    
    document.body.appendChild(windowEl);
}
```

**Step 2: Add CSS for debug window**

```css
.debug-log-window {
    background: var(--theme-surface);
    border: 1px solid var(--theme-border);
    border-radius: 8px;
    box-shadow: 0 4px 20px rgba(0, 0, 0, 0.5);
    display: flex;
    flex-direction: column;
    overflow: hidden;
}

.debug-log-header {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 12px 16px;
    background: var(--theme-surface-light);
    border-bottom: 1px solid var(--theme-border);
    font-weight: 600;
}

.debug-log-header .close-btn {
    margin-left: auto;
    background: none;
    border: none;
    color: var(--theme-text-muted);
    font-size: 20px;
    cursor: pointer;
}

.debug-log-content {
    flex: 1;
    overflow-y: auto;
    padding: 8px;
    font-family: 'Consolas', 'Monaco', monospace;
    font-size: 12px;
    line-height: 1.4;
}

.log-entry {
    padding: 6px 8px;
    margin-bottom: 4px;
    border-radius: 4px;
    border-left: 3px solid transparent;
}

.log-entry.send {
    background: rgba(59, 130, 246, 0.1);
    border-left-color: #3b82f6;
}

.log-entry.receive {
    background: rgba(34, 197, 94, 0.1);
    border-left-color: #22c55e;
}

.log-entry.error {
    background: rgba(239, 68, 68, 0.1);
    border-left-color: #ef4444;
}

.log-entry.info {
    background: rgba(156, 163, 175, 0.1);
    border-left-color: #9ca3af;
}

.log-entry .timestamp {
    color: var(--theme-text-muted);
    margin-right: 8px;
}

.log-entry .direction {
    font-weight: 600;
    margin-right: 8px;
}

.log-entry .ip {
    color: #fbbf24;
    margin-right: 8px;
}

.log-entry .data {
    color: var(--theme-text);
    word-break: break-all;
}

.debug-log-controls {
    display: flex;
    gap: 8px;
    padding: 8px;
    border-top: 1px solid var(--theme-border);
    background: var(--theme-surface-light);
}

.debug-log-controls button {
    padding: 4px 12px;
    font-size: 12px;
}
```

**Step 3: Add log helper methods**

```javascript
addDebugLogEntry(direction, ip, data, type = 'info') {
    const content = document.getElementById('debug-log-content');
    if (!content) return;
    
    const entry = document.createElement('div');
    entry.className = `log-entry ${type}`;
    entry.innerHTML = `
        <span class="timestamp">${new Date().toLocaleTimeString()}</span>
        <span class="direction">[${direction}]</span>
        <span class="ip">${ip}</span>
        <span class="data">${data}</span>
    `;
    
    content.appendChild(entry);
    content.scrollTop = content.scrollHeight;
}

clearDebugLog() {
    const content = document.getElementById('debug-log-content');
    if (content) {
        content.innerHTML = `
            <div class="log-entry info">
                <span class="timestamp">${new Date().toLocaleTimeString()}</span>
                <span class="message">Log cleared...</span>
            </div>
        `;
    }
}

exportDebugLog() {
    const content = document.getElementById('debug-log-content');
    if (!content) return;
    
    const entries = content.querySelectorAll('.log-entry');
    let logText = 'Discovery Debug Log\n';
    logText += '===================\n\n';
    
    entries.forEach(entry => {
        logText += entry.textContent + '\n';
    });
    
    const blob = new Blob([logText], { type: 'text/plain' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `discovery-debug-${Date.now()}.txt`;
    a.click();
    URL.revokeObjectURL(url);
}
```

**Step 4: Commit**

```bash
git add frontend/desktop.js frontend/css/desktop.css
git commit -m "feat: create floating debug log window"
```

---

## Task 3: Add Backend Logging to Discovery Service

**Files:**
- Modify: `backend/src/discovery.rs`
- Modify: `backend/src/lib.rs`

**Step 1: Add logging function to discovery service**

In `backend/src/discovery.rs`, add a logging callback:

```rust
use tauri::Manager;

// Add to DiscoveryService struct
pub struct DiscoveryService {
    // ... existing fields ...
    app_handle: Option<tauri::AppHandle>,
}

// Update new() to accept app_handle
pub fn new(
    port: u16,
    instance_id: String,
    hostname: String,
    api_endpoint: String,
    app_handle: Option<tauri::AppHandle>,
) -> Self {
    Self {
        port,
        instance_id,
        hostname,
        api_endpoint,
        broadcast_socket: None,
        listen_socket: None,
        peers: Arc::new(Mutex::new(HashMap::new())),
        broadcast_handle: None,
        listen_handle: None,
        cleanup_handle: None,
        app_handle,
    }
}

// Add logging method
fn log_event(&self, direction: &str, ip: &str, data: &str, event_type: &str) {
    if let Some(app) = &self.app_handle {
        let payload = serde_json::json!({
            "direction": direction,
            "ip": ip,
            "data": data,
            "type": event_type,
            "timestamp": chrono::Utc::now().to_rfc3339(),
        });
        
        let _ = app.emit("discovery-debug-log", payload);
    }
}
```

**Step 2: Add logging calls throughout discovery**

In `start_broadcasting()`:
```rust
self.log_event("SEND", "255.255.255.255", &format!("Broadcast beacon: {}", self.hostname), "info");
```

In beacon reception (where DiscoveredPeer is created):
```rust
self.log_event(
    "RECV", 
    &src.ip().to_string(), 
    &format!("Beacon from {} at {} (API: {})", beacon.hostname, src.ip(), beacon.api_endpoint),
    "receive"
);
```

In `fetch_peer_models()`:
```rust
self.log_event(
    "SEND",
    &format!("{}:{}", ip, port),
    &format!("GET /v1/models/arandu"),
    "send"
);

// After receiving:
self.log_event(
    "RECV",
    &format!("{}:{}", ip, port),
    &format!("Got {} models: {:?}", remote_models.len(), remote_models.iter().map(|m| &m.name).collect::<Vec<_>>()),
    "receive"
);
```

**Step 3: Update service creation in lib.rs**

In `enable_discovery` command:
```rust
let app_handle = app.app_handle();
let mut discovery_service = DiscoveryService::new(
    port,
    instance_id.clone(),
    instance_name.clone(),
    api_endpoint.clone(),
    Some(app_handle),
);
```

**Step 4: Commit**

```bash
git add backend/src/discovery.rs backend/src/lib.rs
git commit -m "feat: add backend logging to discovery service"
```

---

## Task 4: Frontend Event Listener for Logs

**Files:**
- Modify: `frontend/desktop.js`

**Step 1: Add event listener for debug logs**

In `initDiscovery()` or desktop initialization:

```javascript
initDiscoveryDebugLogging() {
    // Listen for debug log events from backend
    if (window.__TAURI__) {
        window.__TAURI__.event.listen('discovery-debug-log', (event) => {
            const { direction, ip, data, type } = event.payload;
            this.addDebugLogEntry(direction, ip, data, type);
        });
    }
}
```

**Step 2: Add frontend-side logging**

In `pollDiscoveredPeers()`:
```javascript
async pollDiscoveredPeers() {
    this.addDebugLogEntry('INFO', 'LOCAL', 'Polling discovered peers...', 'info');
    
    try {
        const result = await invoke('get_discovered_peers');
        this.addDebugLogEntry('RECV', 'LOCAL', `Got ${result.length} peers`, 'receive');
        
        result.forEach(peer => {
            this.addDebugLogEntry(
                'DATA', 
                peer.ip_address, 
                `Peer: ${peer.hostname}, Models: ${peer.models?.length || 0}`,
                'info'
            );
            
            if (peer.models && peer.models.length > 0) {
                peer.models.forEach(model => {
                    this.addDebugLogEntry(
                        'DATA',
                        peer.ip_address,
                        `  - ${model.name} (${model.size_gb?.toFixed(2) || '?'} GB)`,
                        'info'
                    );
                });
            }
        });
        
        // ... rest of existing code ...
    } catch (error) {
        this.addDebugLogEntry('ERROR', 'LOCAL', `Error: ${error}`, 'error');
    }
}
```

**Step 3: Commit**

```bash
git add frontend/desktop.js
git commit -m "feat: add frontend event listener for discovery logs"
```

---

## Task 5: Testing & Verification

**Step 1: Build and test**

```bash
cd backend
cargo tauri build 2>&1 | tail -20
```

**Step 2: Verify UI elements**

1. Start Arandu
2. Look for debug button (bug icon) in toolbar
3. Click button - debug window should open
4. Click again - window should close
5. Enable discovery
6. Watch logs appear in window

**Step 3: Test logging**

Enable discovery and verify logs show:
- [ ] SEND entries (broadcasts)
- [ ] RECV entries (incoming beacons)
- [ ] Model fetch requests
- [ ] Model fetch responses with data
- [ ] IP addresses are correct
- [ ] Model counts match actual models

**Step 4: Commit**

```bash
git add .
git commit -m "test: verify discovery debug logging works"
```

---

## Task 6: Create Test Builds

**Files:**
- Create final builds

**Step 1: Build release**

```bash
cargo tauri build
```

**Step 2: Copy to main folder**

```bash
cp target/release/Arandu.exe ../Arandu.exe
cp target/release/bundle/msi/*.msi ../
```

**Step 3: Document**

Update knowledge base with debug feature documentation.

---

## Summary

This plan creates:
1. ✅ Toggle button on main desktop (bug icon)
2. ✅ Floating debug log window
3. ✅ Backend logging of all discovery events
4. ✅ Frontend display of network traffic
5. ✅ Send/Receive indicators with IP addresses
6. ✅ Model data logging
7. ✅ Export capability for debugging

**Expected Result:** User can click bug icon, see exactly what network traffic is happening, identify if issue is network or presentation.

**Next Step:** Execute this plan using subagent-driven-development or executing-plans skill.
