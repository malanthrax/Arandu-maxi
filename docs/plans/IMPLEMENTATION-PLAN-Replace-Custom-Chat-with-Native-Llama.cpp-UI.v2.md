# Native Llama.cpp Chat Integration Implementation Plan

## Overview

Replace the custom chat interface with llama.cpp's native web UI while adding parameter control for temperature, top_p, top_k, max_tokens, and repeat_penalty.

---

# Phase 1: Remove All Custom Chat Components

## 1.1 Delete Frontend Files

### Frontend Files to Delete:
- `frontend/modules/chat-app.js` (1117 lines)

### Index.html Modifications:
**File:** `frontend/index.html`
**Line 65:** Remove the script import:

```html
<!-- DELETE THIS LINE -->
<script src="modules/chat-app.js" defer></script>
```

---

## 1.2 Remove Backend Chat Modules

### Backend Files to Delete:
- `backend/src/chat_models.rs` (119 lines)
- `backend/src/chat_manager.rs` (394 lines)

### lib.rs Tauri Commands to Remove:
**File:** `backend/src/lib.rs`
**Lines to remove from command registration:**

```rust
// DELETE THESE COMMANDS (lines ~2603-2608)
create_chat,
load_chat,
list_chats,
send_message,
generate_chat_title,
delete_chat,
```

### lib.rs Command Function Implementations to Remove:
**Lines ~2264-2383:** Remove the following async functions:
- `create_chat(...)` - Creates new chat session
- `load_chat(...)` - Loads existing chat
- `list_chats(...)` - Lists all chats
- `send_message(...)` - Sends message with streaming
- `generate_chat_title(...)` - Auto-generates chat title
- `delete_chat(...)` - Deletes chat session

### lib.rs Imports to Remove:
```rust
// DELETE THESE IMPORTS if they exist
use crate::chat_models::*;
use crate::chat_manager::ChatManager;
```

### AppState Modifications:
**File:** `backend/src/lib.rs`
**Line ~65:** Remove from AppState struct:

```rust
// DELETE THIS FIELD
chat_manager: Arc<Mutex<ChatManager>>,
```

**Line ~110:** Remove from AppState::new():

```rust
// DELETE THIS INITIALIZATION
chat_manager: Arc::new(Mutex::new(ChatManager::default())),
```

---

## 1.3 Remove Terminal Manager References

### File: `frontend/modules/terminal-manager.js`

#### 1.3.1 Remove Custom Chat Tab Element
**Line 84:** Delete the custom chat tab HTML:

```html
<!-- DELETE THIS LINE -->
<div class="server-tab" id="tab-custom-${windowId}" onclick="terminalManager.switchTab('${windowId}', 'custom')" title="Custom Chat" style="opacity: 0.5; pointer-events: none;">
    <span class="material-icons">smart_toy</span>
</div>
```

#### 1.3.2 Remove Custom Chat Panel Container
**Lines 60-62:** Delete the custom chat panel HTML:

```html
<!-- DELETE THESE LINES -->
<div class="server-tab-panel" id="panel-custom-${windowId}">
    <div id="chat-app-${windowId}" class="chat-app-wrapper"></div>
</div>
```

#### 1.3.3 Remove initializeChatApp() Function Call
**Line 200:** Locate and remove:

```javascript
// DELETE THIS FUNCTION CALL
this.initializeChatApp(windowId, processId, modelPath, modelName);
```

#### 1.3.4 Remove initializeChatApp() Function Definition
**Lines 744-785:** Delete the entire function:

```javascript
// DELETE THIS ENTIRE FUNCTION
initializeChatApp(windowId, processId, modelPath, modelName) {
    // ... all function contents
}
```

#### 1.3.5 Remove ChatApp Class Instantiation/Usage
**Lines 745-785:** Within initializeChatApp(), remove:

```javascript
// DELETE THIS BLOCK
const chatApp = new ChatApp(this, windowId, modelPath, modelName);

// ... all chatApp.method() calls
```

#### 1.3.6 Remove Chat Instance References
Check for any remaining `chatApp` references in:
- `this.chatApps` Map (if exists)
- Any event handlers referencing chatApp
- Any cleanup functions

#### 1.3.7 Remove Chat Tab Activation Logic
**Lines 707+ and 479+:** Remove code that enables the custom chat tab when ChatApp initializes, such as:

```javascript
// DELETE THESE LINES
const customTab = document.getElementById(`tab-custom-${windowId}`);
if (customTab) {
    customTab.style.opacity = '1';
    customTab.style.pointerEvents = 'auto';
}
```

---

# Phase 2: Locate Llama.cpp Native UI

## 2.1 Find Llama-server Installation

### Known Location (from settings.json):
```
Path: C:\Users\Gordprime\.Arandu\llama.cpp\versions\b7972\cuda\
Executable: llama-server.exe (10,414,080 bytes)
Port: 8080 (default)
```

### Verification Steps:
1. Run llama-server and examine process startup logs for web UI path
2. Check if llama.cpp provides public folder or embedded HTML
3. Search for `index.html` or similar in llama-server directory

### Commands:
```bash
# Check llama-server directory for HTML/CSS files
ls ~/.Arandu/llama.cpp/versions/b7972/cuda/*.html
ls ~/.Arandu/llama.cpp/versions/b7972/cuda/*.css

# Run llama-server with --help to see if there's a web-ui flag
~/.Arandu/llama.cpp/versions/b7972/cuda/llama-server.exe --help

# Start llama-server and check for log messages about serving files
~/.Arandu/llama.cpp/versions/b7972/cuda/llama-server.exe -m <model> --host 127.0.0.1 --port 8080
# Watch output for messages like "Serving files from: [...]"
```

---

## 2.2 Determine UI Embedding Method

### Investigate Web UI Structure:

**Option A: Embedded in Executable**
- No HTML files found in directory
- HTML served from compiled-in resources
- Need to use browser tools to view source

**Option B: Separate Files**
- Check for common paths:
  - `public/` folder alongside exe
  - `web/` folder
  - Parent directory (e.g., `~/.Arandu/llama.cpp/versions/b7972/`)

**Investigation Commands:**
```bash
# Check entire llama.cpp tree for HTML files
find ~/.Arandu/llama.cpp -name "*.html" -o -name "*.css"

# Check for llama.cpp source if available (search locally or clone)
```

### Browser Investigation:
1. Start llama-server with a model
2. Open `http://127.0.0.1:8080` in browser
3. Right-click → View Page Source
4. Analyze HTML structure for:
   - Input elements (textarea for user input)
   - Send button
   - Parameter controls (if any)
   - Message display area

### Document Expected API Endpoints:
From llama.cpp documentation, likely endpoints:
- `POST /completion` - Send message and get response
- `GET /` - Serve web UI
- `POST /tokenize` - Token counting
- `GET /props` - Model properties

---

# Phase 3: Add Parameter Control Panel

## 3.1 Create Floating Panel HTML

### File: `frontend/modules/parameter-panel.js` (NEW)

```javascript
class ParameterPanel {
    constructor(host, port) {
        this.host = host;
        this.port = port;
        this.baseUrl = `http://${host}:${port}`;
        this.parameters = this.loadParameters();

        this.panelId = 'llama-params-panel';
        this.container = null;
        this.isCollapsed = false;
    }

    loadParameters() {
        const saved = localStorage.getItem('llamaChatParameters');
        if (saved) {
            return JSON.parse(saved);
        }
        return {
            temperature: 0.7,
            top_p: 0.9,
            top_k: 40,
            max_tokens: 2048,
            repeat_penalty: 1.1
        };
    }

    saveParameters() {
        localStorage.setItem('llamaChatParameters', JSON.stringify(this.parameters));
    }

    render() {
        const panel = document.createElement('div');
        panel.id = this.panelId;
        panel.className = 'parameter-panel';
        panel.innerHTML = `
            <div class="panel-header" onclick="this.closest('.parameter-panel').classList.toggle('collapsed')">
                <span class="material-icons">tune</span>
                <span>Parameters</span>
                <span class="collapse-indicator material-icons">expand_less</span>
            </div>
            <div class="panel-body">
                <div class="param-control">
                    <label>
                        Temperature: <span class="param-value" id="param-temp-value">${this.parameters.temperature}</span>
                    </label>
                    <input type="range" id="param-temperature" min="0" max="2" step="0.1" value="${this.parameters.temperature}">
                </div>

                <div class="param-control">
                    <label>
                        Top P: <span class="param-value" id="param-topp-value">${this.parameters.top_p}</span>
                    </label>
                    <input type="range" id="param-top-p" min="0" max="1" step="0.05" value="${this.parameters.top_p}">
                </div>

                <div class="param-control">
                    <label>
                        Top K: <span class="param-value" id="param-topk-value">${this.parameters.top_k}</span>
                    </label>
                    <input type="range" id="param-top-k" min="1" max="100" step="1" value="${this.parameters.top_k}">
                </div>

                <div class="param-control">
                    <label>
                        Max Tokens: <span class="param-value" id="param-maxt-value">${this.parameters.max_tokens}</span>
                    </label>
                    <input type="range" id="param-max-tokens" min="256" max="8192" step="256" value="${this.parameters.max_tokens}">
                </div>

                <div class="param-control">
                    <label>
                        Repeat Penalty: <span class="param-value" id="param-rp-value">${this.parameters.repeat_penalty}</span>
                    </label>
                    <input type="range" id="param-repeat-penalty" min="1" max="2" step="0.1" value="${this.parameters.repeat_penalty}">
                </div>

                <div class="panel-actions">
                    <button class="btn btn-primary" id="btn-apply-params">Apply</button>
                    <button class="btn btn-secondary" id="btn-reset-params">Reset</button>
                </div>
            </div>
        `;
        this.container = panel;
        this.attachEventListeners();
        return panel;
    }

    attachEventListeners() {
        const updateValue = (id, param) => {
            const el = document.getElementById(id);
            if (el) {
                el.addEventListener('input', (e) => {
                    const valueEl = document.getElementById(id + '-value');
                    if (valueEl) {
                        valueEl.textContent = e.target.value;
                    }
                });
            }
        };

        updateValue('param-temperature', 'temperature');
        updateValue('param-top-p', 'top_p');
        updateValue('param-top-k', 'top_k');
        updateValue('param-max-tokens', 'max_tokens');
        updateValue('param-repeat-penalty', 'repeat_penalty');

        const applyBtn = document.getElementById('btn-apply-params');
        if (applyBtn) {
            applyBtn.addEventListener('click', () => this.applyParameters());
        }

        const resetBtn = document.getElementById('btn-reset-params');
        if (resetBtn) {
            resetBtn.addEventListener('click', () => this.resetParameters());
        }
    }

    applyParameters() {
        this.parameters.temperature = parseFloat(document.getElementById('param-temperature').value);
        this.parameters.top_p = parseFloat(document.getElementById('param-top-p').value);
        this.parameters.top_k = parseInt(document.getElementById('param-top-k').value);
        this.parameters.max_tokens = parseInt(document.getElementById('param-max-tokens').value);
        this.parameters.repeat_penalty = parseFloat(document.getElementById('param-repeat-penalty').value);

        this.saveParameters();

        // Send update to llama.cpp via API
        // Note: llama.cpp may need restarting for some params, or may have an update endpoint
        // This will be determined in Phase 2 investigation
        console.log('Parameters applied:', this.parameters);

        // Show success notification
        this.showNotification('Parameters saved');
    }

    resetParameters() {
        this.parameters = {
            temperature: 0.7,
            top_p: 0.9,
            top_k: 40,
            max_tokens: 2048,
            repeat_penalty: 1.1
        };

        // Update UI
        document.getElementById('param-temperature').value = this.parameters.temperature;
        document.getElementById('param-top-p').value = this.parameters.top_p;
        document.getElementById('param-top-k').value = this.parameters.top_k;
        document.getElementById('param-max-tokens').value = this.parameters.max_tokens;
        document.getElementById('param-repeat-penalty').value = this.parameters.repeat_penalty;

        document.getElementById('param-temp-value').textContent = this.parameters.temperature;
        document.getElementById('param-topp-value').textContent = this.parameters.top_p;
        document.getElementById('param-topk-value').textContent = this.parameters.top_k;
        document.getElementById('param-maxt-value').textContent = this.parameters.max_tokens;
        document.getElementById('param-rp-value').textContent = this.parameters.repeat_penalty;

        this.saveParameters();
        this.showNotification('Parameters reset to defaults');
    }

    showNotification(message) {
        const notification = document.createElement('div');
        notification.className = 'parameter-notification';
        notification.textContent = message;
        document.body.appendChild(notification);

        setTimeout(() => {
            notification.remove();
        }, 2000);
    }

    inject() {
        // Wait for iframe to load
        const checkIframe = () => {
            const iframe = document.querySelector('iframe[src^="http"]');
            if (iframe) {
                // Append panel to parent window, not iframe
                // The panel should float above the iframe
                const windowEl = iframe.closest('.window-content');
                if (windowEl) {
                    windowEl.style.position = 'relative';
                    windowEl.appendChild(this.render());
                }
            } else {
                setTimeout(checkIframe, 100);
            }
        };
        checkIframe();
    }
}
```

---

## 3.2 Create CSS for Parameter Panel

### File: `frontend/css/parameter-panel.css` (NEW)

```css
.parameter-panel {
    position: absolute;
    top: 10px;
    right: 10px;
    width: 300px;
    background: var(--theme-surface);
    border: 1px solid var(--theme-border);
    border-radius: 8px;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
    z-index: 9999;
    color: var(--theme-text);
    transition: width 0.3s ease, max-height 0.3s ease;
    overflow: hidden;
}

.parameter-panel.collapsed {
    width: auto;
}

.parameter-panel.collapsed .panel-body {
    max-height: 0;
    opacity: 0;
    overflow: hidden;
}

.panel-header {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 12px 16px;
    background: var(--theme-primary);
    color: white;
    cursor: pointer;
    user-select: none;
    transition: background 0.2s ease;
}

.panel-header:hover {
    background: var(--theme-hover);
}

.panel-header .material-icons {
    font-size: 20px;
}

.collapse-indicator {
    margin-left: auto;
    transition: transform 0.3s ease;
}

.parameter-panel.collapsed .collapse-indicator {
    transform: rotate(-90deg);
}

.panel-body {
    padding: 16px;
    max-height: 500px;
    opacity: 1;
    transition: max-height 0.3s ease, opacity 0.3s ease;
    overflow-y: auto;
}

.param-control {
    margin-bottom: 16px;
}

.param-control:last-child {
    margin-bottom: 20px;
}

.param-control label {
    display: block;
    margin-bottom: 8px;
    font-size: 13px;
    color: var(--theme-text);
    font-weight: 500;
}

.param-control .param-value {
    display: inline-block;
    min-width: 40px;
    text-align: right;
    font-family: monospace;
    font-weight: 600;
    color: var(--theme-primary);
}

.param-control input[type="range"] {
    width: 100%;
    height: 6px;
    background: var(--theme-bg-medium);
    border-radius: 3px;
    outline: none;
    -webkit-appearance: none;
    appearance: none;
}

.param-control input[type="range"]::-webkit-slider-thumb {
    -webkit-appearance: none;
    appearance: none;
    width: 18px;
    height: 18px;
    background: var(--theme-primary);
    border-radius: 50%;
    cursor: pointer;
    transition: background 0.2s ease, transform 0.2s ease;
}

.param-control input[type="range"]::-webkit-slider-thumb:hover {
    background: var(--theme-hover);
    transform: scale(1.1);
}

.param-control input[type="range"]::-moz-range-thumb {
    width: 18px;
    height: 18px;
    background: var(--theme-primary);
    border: none;
    border-radius: 50%;
    cursor: pointer;
    transition: background 0.2s ease, transform 0.2s ease;
}

.param-control input[type="range"]::-moz-range-thumb:hover {
    background: var(--theme-hover);
    transform: scale(1.1);
}

.panel-actions {
    display: flex;
    gap: 8px;
    margin-top: 20px;
    padding-top: 16px;
    border-top: 1px solid var(--theme-border);
}

.btn {
    flex: 1;
    padding: 10px 16px;
    border: none;
    border-radius: 6px;
    font-size: 13px;
    font-weight: 600;
    cursor: pointer;
    transition: background 0.2s ease, transform 0.1s ease;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 8px;
}

.btn:active {
    transform: scale(0.98);
}

.btn-primary {
    background: var(--theme-primary);
    color: white;
}

.btn-primary:hover {
    background: var(--theme-hover);
}

.btn-secondary {
    background: var(--theme-bg-medium);
    color: var(--theme-text);
}

.btn-secondary:hover {
    background: var(--theme-bg-strong);
}

.parameter-notification {
    position: fixed;
    bottom: 20px;
    right: 20px;
    padding: 12px 20px;
    background: var(--theme-success);
    color: white;
    border-radius: 6px;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.2);
    z-index: 10000;
    font-size: 14px;
    animation: slideIn 0.3s ease, fadeOut 0.3s ease 1.7s forwards;
}

@keyframes slideIn {
    from {
        transform: translateX(100%);
        opacity: 0;
    }
    to {
        transform: translateX(0);
        opacity: 1;
    }
}

@keyframes fadeOut {
    from {
        opacity: 1;
    }
    to {
        opacity: 0;
    }
}

/* Light theme adjustments */
[data-theme="light"] .parameter-panel {
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.15);
}

[data-theme="light"] .panel-header {
    color: var(--theme-bg);
}

/* Dark theme adjustments (default) */
[data-theme="dark"] .param-control input[type="range"] {
    background: var(--theme-bg-medium);
}

[data-theme="dark"] .parameter-notification {
    background: var(--theme-success-bg);
    color: var(--theme-success);
}
```

---

## 3.3 Import CSS in Index.html

### File: `frontend/index.html`
**Insert after line 17:**

```html
<link rel="stylesheet" href="css/parameter-panel.css">
```

---

## 3.4 Inject Panel into Native Chat

### Modify: `frontend/modules/terminal-manager.js`

#### Add Import Statement
**After line ~2, add:**

```javascript
import ParameterPanel from './parameter-panel.js';
```

**OR** if using script tags (after modifying lib.rs), add to `index.html`:

```html
<script src="modules/parameter-panel.js" defer></script>
```

#### Modify openServerTerminal() Function
**After window creation (around line ~100), add:**

```javascript
// After rendering window content
window.classList.remove('hidden');
window.style.display = 'block';

// NEW: Initialize parameter panel
setTimeout(() => {
    const paramPanel = new ParameterPanel(host, port);
    window.desktop.paramPanels.set(windowId, paramPanel);
    paramPanel.inject();
}, 500);
```

#### Add to TerminalManager Constructor
**Line ~8, add:**

```javascript
this.paramPanels = new Map();
```

#### Add Cleanup Function
**Add to TerminalManager class:**

```javascript
cleanupParamPanel(windowId) {
    const panel = this.paramPanels.get(windowId);
    if (panel && panel.container) {
        panel.container.remove();
    }
    this.paramPanels.delete(windowId);
}
```

#### Call Cleanup on Window Close
**In the window close handler (modify existing):**

```javascript
stopTerminal(windowId) {
    // Existing cleanup code...

    // NEW: Clean up parameter panel
    this.cleanupParamPanel(windowId);
}
```

---

# Phase 4: Modify Native Chat Behavior

## 4.1 Investigate Native UI Behavior

### Open native chat and analyze:
1. Check if Enter key sends message
2. Check if Shift+Enter adds newline
3. Check send button functionality
4. Test auto-resize input (if any)
5. Identify DOM elements for:
   - Chat message input (textarea/input)
   - Send button
   - Message display area

### Use Browser DevTools:
```javascript
// In browser console on http://127.0.0.1:8080:
document.querySelector('textarea');
document.querySelector('button[type="submit"]');
document.querySelectorAll('input, textarea, button');
```

---

## 4.2 Apply Behavior Improvements

### NOTE: Modifications may require injecting JavaScript into iframe

#### 4.2.1 Create Injection Script

### File: `frontend/modules/native-chat-patcher.js` (NEW)

```javascript
class NativeChatPatcher {
    constructor(iframe) {
        this.iframe = iframe;
        this.patched = false;
    }

    patch() {
        if (this.patched) return;

        const iframeWindow = this.iframe.contentWindow;
        const iframeDoc = this.iframe.contentDocument;

        if (!iframeWindow || !iframeDoc) {
            console.warn('Cannot access iframe content');
            return;
        }

        // Wait for document to fully load
        if (iframeDoc.readyState !== 'complete') {
            this.iframe.addEventListener('load', () => this.patch());
            return;
        }

        this.patchTextareaBehavior(iframeDoc);
        this.patchSendButton(iframeDoc, iframeWindow);
        this.addAutoResize(iframeDoc);

        this.patched = true;
        console.log('Native chat UI patched');
    }

    patchTextareaBehavior(doc) {
        // Find textarea elements
        const textareas = doc.querySelectorAll('textarea');

        textareas.forEach(textarea => {
            // Set initial styles
            textarea.style.minHeight = '128px';
            textarea.style.maxHeight = '300px';
            textarea.style.resize = 'none';
            textarea.style.overflowY = 'auto';

            // Add Enter key support
            textarea.addEventListener('keydown', (e) => {
                if (e.key === 'Enter' && !e.shiftKey) {
                    e.preventDefault();
                    // Find and click send button
                    const sendBtn = doc.querySelector('button[type="submit"], .send-btn, #send');
                    if (sendBtn) {
                        sendBtn.click();
                    } else {
                        // Try to find form and submit
                        const form = textarea.closest('form');
                        if (form) {
                            form.dispatchEvent(new Event('submit', { bubbles: true, cancelable: true }));
                        }
                    }
                }
            });

            // Set rows to 4 initially
            textarea.setAttribute('rows', '4');
        });
    }

    patchSendButton(doc, win) {
        // Ensure send button has click handler
        const sendBtns = doc.querySelectorAll('button[type="submit"], .send-btn, #send');

        sendBtns.forEach(btn => {
            // Make sure it's visible and enabled
            btn.style.cursor = 'pointer';
            btn.removeAttribute('disabled');
        });
    }

    addAutoResize(doc) {
        const textareas = doc.querySelectorAll('textarea');

        textareas.forEach(textarea => {
            const autoResize = () => {
                textarea.style.height = 'auto';
                const newHeight = Math.min(
                    Math.max(textarea.scrollHeight, 128),
                    300
                );
                textarea.style.height = newHeight + 'px';
            };

            textarea.addEventListener('input', autoResize);
            textarea.addEventListener('focus', autoResize);

            // Initial resize
            setTimeout(autoResize, 100);
        });
    }
}
```

---

## 4.3 Integrate Patcher with Terminal Manager

### Modify: `frontend/modules/terminal-manager.js`

#### Import Patcher
```javascript
import NativeChatPatcher from './native-chat-patcher.js';
```

#### Store Patcher Instances
**In constructor, add:**
```javascript
this.chatPatchers = new Map();
```

#### Initialize Patcher When Chat Tab Opens
**Modify switchTab() function:**

```javascript
switchTab(windowId, tabName) {
    // Existing tab switching logic...

    if (tabName === 'chat') {
        // Wait for iframe to load, then patch
        setTimeout(() => {
            const iframe = document.querySelector(`#panel-chat-${windowId} iframe`);
            if (iframe) {
                if (!this.chatPatchers.has(windowId)) {
                    const patcher = new NativeChatPatcher(iframe);
                    this.chatPatchers.set(windowId, patcher);
                }
                this.chatPatchers.get(windowId).patch();
            }
        }, 1000);
    }
}
```

#### Clean Up Patcher
**Add to stopTerminal():**
```javascriptcleanupChatPatcher(windowId) {
    const patcher = this.chatPatchers.get(windowId);
    if (patcher) {
        this.chatPatchers.delete(windowId);
    }
}
```

---

# Phase 5: Testing & Verification

## 5.1 Parameter Panel Test

### Test Cases:

#### Test 1: Panel Rendering
- [ ] Start llama-server with a model
- [ ] Open server terminal window
- [ ] Navigate to "Native Chat" tab
- [ ] Verify parameter panel appears in top-right corner
- [ ] Verify panel has all 5 sliders (temperature, top_p, top_k, max_tokens, repeat_penalty)
- [ ] Verify "Apply" and "Reset" buttons are visible

#### Test 2: Panel Collapsing
- [ ] Click panel header
- [ ] Verify panel collapses (only header visible)
- [ ] Click header again
- [ ] Verify panel expands

#### Test 3: Slider Interaction
- [ ] Adjust temperature slider
- [ ] Verify value display updates
- [ ] Adjust each slider to various values
- [ ] Verify all values display correctly
- [ ] Verify sliders stay within min/max bounds

#### Test 4: Apply Parameters
- [ ] Modify several parameters
- [ ] Click "Apply"
- [ ] Verify notification appears ("Parameters saved")
- [ ] Reload page/browser
- [ ] Open parameter panel again
- [ ] Verify values persist (loaded from localStorage)

#### Test 5: Reset Parameters
- [ ] Modify parameters
- [ ] Click "Reset"
- [ ] Verify all values revert to defaults
- [ ] Apply new parameters
- [ ] Reload page
- [ ] Verify defaults are restored

---

## 5.2 Chat Functionality Test

### Test Cases:

#### Test 1: Send Message
- [ ] Type message in native chat input
- [ ] Click Send button
- [ ] Verify message appears in chat

#### Test 2: Enter Key Support
- [ ] Type message
- [ ] Press Enter (without Shift)
- [ ] Verify message sends
- [ ] Type message
- [ ] Press Shift+Enter
- [ ] Verify newline is added (message not sent)

#### Test 3: Auto-resize Input
- [ ] Start typing long message
- [ ] Verify textarea grows up to 300px
- [ ] Delete text
- [ ] Verify textarea shrinks to 128px minimum
- [ ] Verify input never exceeds 300px

#### Test 4: Streaming Responses
- [ ] Send a message
- [ ] Verify response streams in real-time
- [ ] Verify no lag or freeze
- [ ] Verify response completes fully

---

## 5.3 Integration Test

### Test Cases:

#### Test 1: Parameter Effect on Output
- [ ] Set temperature to 0.0
- [ ] Send the same prompt twice
- [ ] Verify outputs are identical/similar
- [ ] Set temperature to 1.5
- [ ] Send the same prompt
- [ ] Verify output is different/creative

#### Test 2: Panel UI Over Native Chat
- [ ] Open parameter panel
- [ ] Type message
- [ ] Verify panel doesn't interfere with typing
- [ ] Verify panel stays on top (z-index)
- [ ] Verify panel doesn't block chat elements

#### Test 3: Multiple Instances
- [ ] Start two models simultaneously
- [ ] Open parameter panels for both
- [ ] Verify each has independent settings
- [ ] Apply different parameters to each
- [ ] Verify parameters apply to correct model

---

# Phase 6: Fallback Subplan

## 6.1 Trigger Conditions

Implement fallback if:
- [ ] Cannot access iframe content (CORS/security restrictions)
- [ ] Cannot query/manipulate native UI elements
- [ ] Patching causes native UI to break
- [ ] Tests fail consistently

---

## 6.2 Custom Llama-server Build

### Backup Plan: Compile Custom llama-server

#### Step 1: Clone llama.cpp Repository

```bash
cd ~/src  # or preferred source directory
git clone https://github.com/ggerganov/llama.cpp.git
cd llama.cpp
git checkout <commit-hash>  # Match Arandu's version if known
```

#### Step 2: Locate Web UI Files

```bash
# Find server source
find . -name "*server*" -type f

# Look for HTML/CSS/JS files in examples/server/
ls -la examples/server/
ls -la examples/server/public/  # If exists
```

#### Step 3: Modify Web UI

If files found in `examples/server/public/`:
1. Edit `index.html` to add parameter controls
2. Add CSS file in `public/css/`
3. Add JS file in `public/js/`
4. Test locally

If files are embedded:
1. Search for HTML strings in source
2. Modify template HTML in C++ code
3. Recompile

#### Step 4: Compile Custom Build

```bash
# Build with CUDA support (Arandu uses CUDA)
cmake -B build -DLLAMA_CUBLAS=ON -DLLAMA_CUDA_F16=ON
cmake --build build --config Release

# Find compiled executable
find build -name "llama-server.exe"
```

#### Step 5: Integrate with Arandu

Test custom build:
```bash
# Run with a test model
build/Release/llama-server.exe -m <model-path> -h 127.0.0.1 -p 8080

# Open in browser and verify parameter panel appears
```

Replace Arandu executable:
```bash
# Backup original
cp ~/.Arandu/llama.cpp/versions/b7972/cuda/llama-server.exe ~/.Arandu/llama.cpp/versions/b7972/cuda/llama-server.exe.backup

# Copy custom build
cp build/Release/llama-server.exe ~/.Arandu/llama.cpp/versions/b7972/cuda/

# Set as active in Arandu settings or use default executable folder
```

#### Step 6: Update Arandu Backend (Optional)

If custom build version need special handling:
- Add version detection in `llamacpp_manager.rs`
- Update version mapping
- Document custom build in AGENTS.md

---

## 6.3 Alternative: Standalone Parameter Manager UI

If custom build not feasible:

### Create External Parameter Dialog

1. Add "Configure Parameters" button to terminal window
2. Open modal dialog with sliders
3. Apply parameters by:
   - Sending API call to update llama-server
   - Updating localStorage
   - Showing restart prompt if needed

### Update Process Launch Command

Modify `process.rs` to add parameter flags:
```rust
let command = format!(
    "llama-server.exe -m {} --temp {} --top_p {} --top_k {} -ctx {}",
    model_path,
    config.temperature,
    config.top_p,
    config.top_k,
    config.context_length
);
```

---

# Summary

## Files Created

| File | Description |
|------|-------------|
| `frontend/modules/parameter-panel.js` | Floating parameter control panel class |
| `frontend/css/parameter-panel.css` | Styles for parameter panel |
| `frontend/modules/native-chat-patcher.js` | Patches native chat UI behavior |

## Files Modified

| File | Changes |
|------|---------|
| `frontend/index.html` | Remove chat-app.js import, add parameter-panel.css |
| `backend/src/lib.rs` | Remove chat commands, imports, and ChatManager |
| `frontend/modules/terminal-manager.js` | Remove custom chat tab/panel, add parameter panel integration |

## Files Deleted

| File | Lines | Description |
|------|-------|-------------|
| `frontend/modules/chat-app.js` | 1117 | Custom chat application |
| `backend/src/chat_models.rs` | 119 | Chat data structures |
| `backend/src/chat_manager.rs` | 394 | Chat session management |

## Total Lines Changed

- **Deleted:** ~1,630 lines (custom chat code)
- **Added:** ~400 lines (parameter panel, patching)
- **Modified:** ~150 lines (terminal manager integration)
- **Net Change:** -1,080 lines (code reduction + feature addition)

---

# Risk Assessment

| Risk | Probability | Impact | Mitigation |
|------|------------|--------|------------|
| Native UI inaccessible from iframe | Medium | High | Use Fallback Plan B (custom build) |
| Parameter panel breaks native UI | Low | Medium | Test thoroughly, use z-index management |
| llama-server API doesn't support runtime parameter updates | Medium | Medium | Document restart requirement, consider custom build |
| Cross-origin restrictions prevent iframe patching | Medium | High | Try postMessage, fall back to custom build |
| Browser security blocks localStorage in iframe | Low | Low | Store in parent window localStorage |

---

# Verification Checklist

Before considering implementation complete:

- [ ] All custom chat files deleted
- [ ] All backend chat commands removed
- [ ] Terminal manager no longer references custom chat
- [ ] Parameter panel renders correctly
- [ ] Parameters save/load from localStorage
- [ ] Native chat tabs work
- [ ] Enter key sends messages
- [ ] Shift+Enter adds newlines
- [ ] Auto-resize works on native input
- [ ] Parameter changes affect model output
- [ ] Multiple windows have independent parameters
- [ ] All tests in Phase 5 passing
- [ ] Existing features not broken
- [ ] No console errors
- [ ] Build succeeds without warnings
- [ ] AGENTS.md updated (optional, for documentation)

---

# Implementation Notes

### Important Considerations:

1. **CROSS-ORIGIN RESTRICTIONS:** Browsers may block iframe access from different origins. Need to verify `http://127.0.0.1:8080` is accessible from the Tauri window.

2. **PARAMETER PERSISTENCE:** Parameters saved to localStorage but may not affect llama-server until restart. Check if llama.cpp supports runtime parameter updates.

3. **THEME COMPATIBILITY:** Parameter panel uses Arandu CSS variables. Ensure all themes have these defined.

4. **PERFORMANCE:** Parameter panel should not impact chat performance. Use event delegation and efficient DOM updates.

5. **ACCESSIBILITY:** Add ARIA labels, keyboard navigation, and focus management to parameter panel.

6. **FALLBACK READINESS:** Have custom build ready if iframe patching doesn't work. Test llama.cpp compilation early.

---

# Timeline Estimate

| Phase | Estimated Time |
|-------|---------------|
| Phase 1: Remove Custom Chat | 2-3 hours |
| Phase 2: Locate Native UI | 1-2 hours |
| Phase 3: Parameter Panel | 4-5 hours |
| Phase 4: Modify Native Chat | 2-3 hours |
| Phase 5: Testing | 3-4 hours |
| Phase 6: Fallback (if needed) | 4-6 hours |
| **Total** | **16-23 hours** |

---

# Success Criteria

Project considered successful when:

1. ✅ Custom chat completely removed (no references remaining)
2. ✅ Native llama.cpp chat works in terminal window
3. ✅ Parameter panel controls temperature, top_p, top_k, max_tokens, repeat_penalty
4. ✅ Parameters persist across sessions
5. ✅ Native chat supports Enter to send and Shift+Enter for newline
6. ✅ Native chat input auto-resizes (128px - 300px)
7. ✅ No regression in existing features
8. ✅ Build succeeds and runs without errors
