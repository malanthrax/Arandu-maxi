# Implementation Plan: Replace Custom Chat with Native llama.cpp UI + Parameter Controls

## Overview
Remove all custom chat components and enhance the native llama.cpp chat interface with a floating parameter control panel.

**REVISED APPROACH:** Based on code review findings:
1. **Phase 0 must be completed first** - Verify llama.cpp can handle runtime parameters
2. **Method A (Custom llama-server build)** is the primary strategy - avoids iframe/CORS issues
3. **Incremental verification** after each deletion step
4. **CSS cleanup includes desktop.js and main.css imports**

---

# Phase 0: CRITICAL Investigation & Verification (DO THIS FIRST!)

## 0.1 Does Llama.cpp Support Runtime Parameter Updates?

**Test the completion API:**
```bash
# Start llama-server
llama-server.exe -m model.gguf --host 127.0.0.1 --port 8080

# Test with different temperatures
curl -X POST http://127.0.0.1:8080/completion \
  -H "Content-Type: application/json" \
  -d '{"prompt": "write a poem", "temperature": 0.0}'

curl -X POST http://127.0.0.1:8080/completion \
  -H "Content-Type: application/json" \
  -d '{"prompt": "write a poem", "temperature": 2.0}'
```

**Test Results:**
- [ ] Responses are different → Runtime parameters work → PROCEED
- [ ] Responses are identical → Runtime parameters don't work → STOP, need alternative

## 0.2 Find Llama-server UI Files

```bash
# Check Arandu config for location
cat ~/.Arandu/config.json | grep executable_folder

# Navigate and investigate
cd [executable_folder]

# Find web UI files
find . -name "index.html" -type f
find . -path "*/server/public/*" -type f
```

**Document:**
- UI files exist: yes/no
- File location: ________________
- Editable: yes/no
- If embedded: must use custom build approach

## 0.3 Create Backup

```bash
git checkout -b backup-before-remove-custom-chat
git add -A
git commit -m "Backup: Custom chat still working"
git checkout main
```

## 0.4 Document Current State

**Custom chat features that work:**
- [ ] Tab appears and is clickable
- [ ] Can create new chats
- [ ] Can send messages
- [ ] Can view chat history
- [ ] Settings panel works

**These must be replicated in native UI:**
- [ ] Parameter controls
- [ ] Good send button behavior
- [ ] Auto-resize textarea
- [ ] Enter key support

---

# Phase 1: Remove All Custom Chat Components

## 1.1 Delete Frontend Files

**Delete these files:**
- `frontend/modules/chat-app.js` (~1,117 lines)
- `frontend/css/chat-app.css` (~889 lines)

**Edit: `frontend/index.html`**
- Remove line: `<script src="modules/chat-app.js" defer></script>`
- Remove if exists: `<link rel="stylesheet" href="css/chat-app.css">`

## 1.2 Remove Backend Chat Modules

**Delete these files:**
- `backend/src/chat_manager.rs`
- `backend/src/chat_models.rs` (if exists)
- `backend/src/parameter_controller.rs` (unused, but verify first)

**Edit: `backend/src/lib.rs`**

**Remove import:**
```rust
mod chat_manager;
mod chat_models;
mod parameter_controller;
use chat_manager::ChatManager;

// Also need to import ChatSession, ChatMessage, ChatSummary, ChatParameters from chat_models
```

**Remove these Tauri commands (search for each command signature):**
```rust
#[tauri::command]
async fn create_chat(model_path: String, model_name: String) -> Result<ChatSession, String>

#[tauri::command]
async fn load_chat(chat_id: String) -> Result<ChatSession, String>

#[tauri::command]
async fn list_chats(model_path: String) -> Result<Vec<ChatSummary>, String>

#[tauri::command]
async fn send_message(chat_id: String, content: String, stream: bool) -> Result<MessageResponse, String>

#[tauri::command]
async fn update_chat_parameters(chat_id: String, parameters: ChatParameters) -> Result<(), String>

#[tauri::command]
async fn delete_chat(chat_id: String) -> Result<(), String>

#[tauri::command]
async fn generate_chat_title(chat_id: String) -> Result<String, String>
```

**Remove from command registration:**
- Remove all 7 commands from `invoke_handler` list
- Search for `.invoke_handler([...])` and remove chat-related commands

## 1.3 Remove Terminal Manager References

**Edit: `frontend/modules/terminal-manager.js`**

**Remove custom chat tab (search for tab-custom):**
```html
<!-- Remove this entire tab element -->
<div class="server-tab" id="tab-custom-${windowId}" onclick="terminalManager.switchTab('${windowId}', 'custom')" title="Custom Chat" style="opacity: 0.5; pointer-events: none;">
    <span class="material-icons">smart_toy</span>
</div>
```

**Remove chat container:**
```html
<!-- Remove this div -->
<div id="chat-app-${windowId}" class="chat-app-wrapper"></div>
```

**Switch tab logic:**
- Remove case for `'custom'` from `switchTab()` function
- Keep only: `'terminal'` and `'native_chat'` cases

**Remove initializeChatApp function:**
```javascript
// Delete entire function starting at line ~744
initializeChatApp(windowId, processId, modelPath, modelName) {
    // ... entire function ...
}
```

**Remove call to initializeChatApp:**
```javascript
// In createTerminalWindow(), remove this call:
// this.initializeChatApp(windowId, processId, modelPath, modelName);
```

**Remove chat chatApps Map:**
```javascript
// Remove: this.chatApps = new Map();
// Remove any references to this.chatApps
```

**Remove custom tab state management:**
```javascript
// Remove from updateServerStatus():
// "Custom tab is enabled when ChatApp initialization is complete"
// Remove enabling of tab-custom elements
```

**Cleanup:**
- Search for remaining `ChatApp`, `chatApp`, `initializeChatApp` references
- Remove all unused code
- Verify no more `{chat}` tab logic remains

---

# Phase 2: Locate Llama.cpp Native UI

## 2.1 Find Llama-server Installation

**Step 1: Read Arandu config to find executable folder**
- Config location: `%USERPROFILE%\.Arandu\config.json`
- Look for: `executable_folder` field
- Or `active_executable_folder` if set

**Step 2: Navigate to llama-server location**
- Typical path: `~/.Arandu/llama.cpp/` or user-defined location
- Verify: `llama-server.exe` exists

**Step 3: Check for web UI files**
Llama.cpp stores web UI in one of these ways:
- **Embedded:** Compiled into exe as resource
- **Separate:** Files in `examples/server/public/` folder
- **Sibling:** `public/` or `web/` folder next to exe

**Search for:**
```bash
# In llama-server directory:
ls -la examples/server/public/
ls -la public/
ls -la web/
ls -la *.html

# Or in the llama-server build directory:
find . -name "index.html" -type f
find . -name "*.js" -path "*/server/*"
```

**Files we're looking for:**
- `index.html` - Main chat UI
- `index.js` - Chat functionality
- `index.css` - Styling
- `completion.js` - API integration

## 2.2 Determine UI Embedding Method

**Test if files are separate:**
- If `index.html` exists and is editable → **Method A** (Edit directly)
- If no separate files → UI is embedded → Need **Method B** (Injection)

**How to verify:**
```bash
# Open index.html and check
cat examples/server/public/index.html

# If it has llama.cpp chat UI (chat box, send button, etc.), it's separate files
# If file doesn't exist or is minimal binary, it's embedded
```

**Document findings:**
- UI location (path to files)
- Is editable? (yes/no)
- File sizes (indicate if real files or stubs)
- Last modified dates

---

# Phase 3: Add Parameter Control Panel

**Depending on Phase 2 findings:**

## Method A: Edit Llama-server HTML/JS directly (if separate files)

### 3.1 Modify llama-server index.html

**Add parameter panel to index.html:**
```html
<!-- After <body> tag, at top of document -->
<div id="parameter-panel" class="parameter-panel">
    <div class="panel-header">
        <h3>Parameters</h3>
        <button id="toggle-panel" class="toggle-btn">−</button>
    </div>
    <div class="panel-content" id="panel-content">
        <div class="parameter-row">
            <label for="temp-slider">
                Temperature: <span id="temp-value">0.7</span>
            </label>
            <input type="range" id="temp-slider" min="0" max="2" step="0.1" value="0.7">
        </div>
        <div class="parameter-row">
            <label for="top-p-slider">
                Top P: <span id="top-p-value">0.9</span>
            </label>
            <input type="range" id="top-p-slider" min="0" max="1" step="0.05" value="0.9">
        </div>
        <div class="parameter-row">
            <label for="top-k-slider">
                Top K: <span id="top-k-value">40</span>
            </label>
            <input type="range" id="top-k-slider" min="1" max="100" step="1" value="40">
        </div>
        <div class="parameter-row">
            <label for="max-tokens-slider">
                Max Tokens: <span id="max-tokens-value">2048</span>
            </label>
            <input type="range" id="max-tokens-slider" min="256" max="8192" step="256" value="2048">
        </div>
        <div class="parameter-row">
            <label for="repeat-penalty-slider">
                Repeat Penalty: <span id="repeat-penalty-value">1.1</span>
            </label>
            <input type="range" id="repeat-penalty-slider" min="0" max="2" step="0.1" value="1.1">
        </div>
        <div class="panel-buttons">
            <button id="apply-params" class="apply-btn">Apply</button>
            <button id="reset-params" class="reset-btn">Reset</button>
        </div>
    </div>
</div>
```

### 3.2 Add CSS to index.html or separate CSS

```css
/* Add to <style> tag or index.css */
.parameter-panel {
    position: fixed;
    top: 20px;
    right: 20px;
    width: 320px;
    background: rgba(30, 30, 35, 0.95);
    border: 1px solid rgba(255, 255, 255, 0.1);
    border-radius: 12px;
    box-shadow: 0 8px 32px rgba(0, 0, 0, 0.4);
    z-index: 1000;
    max-height: 80vh;
    overflow: hidden;
    color: #e0e0e0;
    font-family: 'Segoe UI', system-ui, sans-serif;
}

.panel-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 16px;
    border-bottom: 1px solid rgba(255, 255, 255, 0.1);
    cursor: move;
    user-select: none;
}

.panel-header h3 {
    margin: 0;
    font-size: 16px;
    font-weight: 600;
}

.toggle-btn {
    background: none;
    border: none;
    color: #e0e0e0;
    font-size: 20px;
    cursor: pointer;
    width: 28px;
    height: 28px;
    border-radius: 4px;
    transition: background 0.2s;
}

.toggle-btn:hover {
    background: rgba(255, 255, 255, 0.1);
}

.panel-content {
    padding: 16px;
    max-height: calc(80vh - 60px);
    overflow-y: auto;
    transition: max-height 0.3s ease, opacity 0.3s ease;
}

.panel-content.collapsed {
    max-height: 0;
    opacity: 0;
    padding: 0 16px;
}

.parameter-row {
    margin-bottom: 20px;
}

.parameter-row label {
    display: block;
    margin-bottom: 8px;
    font-size: 13px;
    color: #a0a0a0;
}

.parameter-row input[type="range"] {
    width: 100%;
    height: 6px;
    background: rgba(255, 255, 255, 0.1);
    border-radius: 3px;
    outline: none;
    -webkit-appearance: none;
}

.parameter-row input[type="range"]::-webkit-slider-thumb {
    -webkit-appearance: none;
    width: 18px;
    height: 18px;
    background: #4a9eff;
    border-radius: 50%;
    cursor: pointer;
    box-shadow: 0 2px 6px rgba(74, 158, 255, 0.4);
}

.parameter-row input[type="range"]::-webkit-slider-thumb:hover {
    background: #5fa8ff;
    transform: scale(1.1);
}

.panel-buttons {
    display: flex;
    gap: 10px;
    margin-top: 24px;
}

.apply-btn, .reset-btn {
    flex: 1;
    padding: 12px;
    border: none;
    border-radius: 8px;
    font-size: 14px;
    font-weight: 500;
    cursor: pointer;
    transition: background 0.2s;
}

.apply-btn {
    background: #4a9eff;
    color: white;
}

.apply-btn:hover {
    background: #5fa8ff;
}

.reset-btn {
    background: rgba(255, 255, 255, 0.1);
    color: #e0e0e0;
}

.reset-btn:hover {
    background: rgba(255, 255, 255, 0.15);
}
```

### 3.3 Add JavaScript functionality

**Add to index.js or inline in index.html:**
```javascript
// Parameter panel management
(function() {
    const panel = {
        params: {
            temperature: 0.7,
            top_p: 0.9,
            top_k: 40,
            max_tokens: 2048,
            repeat_penalty: 1.1
        },
        defaultParams: {
            temperature: 0.7,
            top_p: 0.9,
            top_k: 40,
            max_tokens: 2048,
            repeat_penalty: 1.1
        }
    };

    // Load saved params from localStorage
    function loadParams() {
        const saved = localStorage.getItem('llama-chat-params');
        if (saved) {
            try {
                panel.params = JSON.parse(saved);
                updateSliders();
            } catch (e) {
                console.error('Failed to load params:', e);
            }
        }
    }

    // Save params to localStorage
    function saveParams() {
        localStorage.setItem('llama-chat-params', JSON.stringify(panel.params));
    }

    // Update slider displays
    function updateSliders() {
        document.getElementById('temp-value').textContent = panel.params.temperature;
        document.getElementById('temp-slider').value = panel.params.temperature;

        document.getElementById('top-p-value').textContent = panel.params.top_p;
        document.getElementById('top-p-slider').value = panel.params.top_p;

        document.getElementById('top-k-value').textContent = panel.params.top_k;
        document.getElementById('top-k-slider').value = panel.params.top_k;

        document.getElementById('max-tokens-value').textContent = panel.params.max_tokens;
        document.getElementById('max-tokens-slider').value = panel.params.max_tokens;

        document.getElementById('repeat-penalty-value').textContent = panel.params.repeat_penalty;
        document.getElementById('repeat-penalty-slider').value = panel.params.repeat_penalty;
    }

    // Setup slider listeners
    function setupSliders() {
        document.getElementById('temp-slider').addEventListener('input', (e) => {
            panel.params.temperature = parseFloat(e.target.value);
            document.getElementById('temp-value').textContent = panel.params.temperature;
        });

        document.getElementById('top-p-slider').addEventListener('input', (e) => {
            panel.params.top_p = parseFloat(e.target.value);
            document.getElementById('top-p-value').textContent = panel.params.top_p;
        });

        document.getElementById('top-k-slider').addEventListener('input', (e) => {
            panel.params.top_k = parseInt(e.target.value);
            document.getElementById('top-k-value').textContent = panel.params.top_k;
        });

        document.getElementById('max-tokens-slider').addEventListener('input', (e) => {
            panel.params.max_tokens = parseInt(e.target.value);
            document.getElementById('max-tokens-value').textContent = panel.params.max_tokens;
        });

        document.getElementById('repeat-penalty-slider').addEventListener('input', (e) => {
            panel.params.repeat_penalty = parseFloat(e.target.value);
            document.getElementById('repeat-penalty-value').textContent = panel.params.repeat_penalty;
        });
    }

    // Apply params to current completion
    function applyParams() {
        saveParams();
        showNotification('Parameters applied', 'success');
    }

    // Reset to defaults
    function resetParams() {
        panel.params = {...panel.defaultParams};
        updateSliders();
        saveParams();
        showNotification('Parameters reset', 'info');
    }

    // Toggle panel collapse
    function togglePanel() {
        const content = document.getElementById('panel-content');
        const toggleBtn = document.getElementById('toggle-panel');
        content.classList.toggle('collapsed');
        toggleBtn.textContent = content.classList.contains('collapsed') ? '+' : '−';
    }

    // Simple notification
    function showNotification(message, type) {
        const notification = document.createElement('div');
        notification.className = `notification ${type}`;
        notification.textContent = message;
        notification.style.cssText = `
            position: fixed;
            top: 80px;
            right: 20px;
            background: ${type === 'success' ? '#4caf50' : '#2196f3'};
            color: white;
            padding: 12px 16px;
            border-radius: 8px;
            box-shadow: 0 4px 12px rgba(0,0,0,0.3);
            z-index: 1001;
            animation: slideIn 0.3s ease;
        `;
        document.body.appendChild(notification);
        setTimeout(() => notification.remove(), 2000);
    }

    // Initialize
    document.addEventListener('DOMContentLoaded', () => {
        loadParams();
        updateSliders();
        setupSliders();
        document.getElementById('apply-params').addEventListener('click', applyParams);
        document.getElementById('reset-params').addEventListener('click', resetParams);
        document.getElementById('toggle-panel').addEventListener('click', togglePanel);
    });
})();

// Hook into llama.cpp's completion API
// This requires finding where llama.js calls the completion API
// and modifying it to include our params

// Placeholder - will need to inspect llama.cpp's completion.js to find exact integration point
(function() {
    // Find the function that sends completion requests
    // Modify it to include our panel.params
    // Example: if llama.js has a sendCompletion() function
    // We need to wrap it or extend it
})();
```

## Method B: UI Overlay/Injection (if UI is embedded)

**Create new files in Arandu:**

### 3.1 Create `frontend/modules/parameter-panel.js`

```javascript
class ParameterPanel {
    constructor() {
        this.params = {
            temperature: 0.7,
            top_p: 0.9,
            top_k: 40,
            max_tokens: 2048,
            repeat_penalty: 1.1
        };
        this.isCollapsed = false;
    }

    createPanel() {
        // Create panel HTML (same as Method A)
        const panel = document.createElement('div');
        panel.id = 'arandu-parameter-panel';
        panel.innerHTML = `...same HTML...`;
        document.body.appendChild(panel);
        this.attachListeners();
    }

    // ... remaining methods same as Method A ...
}

// Auto-inject into native chat iframe
function injectIntoNativeChat() {
    const iframes = document.querySelectorAll('iframe');
    iframes.forEach(iframe => {
        iframe.addEventListener('load', () => {
            const panel = new ParameterPanel();
            iframe.contentWindow.aranduPanel = panel;
            panel.createPanel();
        });
    });
}
```

### 3.2 Create `frontend/css/parameter-panel.css`

Use same CSS as Method A.

### 3.3 Integrate with terminal-manager.js

```javascript
// After native chat iframe is loaded
iframe.addEventListener('load', () => {
    injectParameterPanel(iframe);
});

function injectParameterPanel(iframe) {
    // Inject CSS
    const styleLink = document.createElement('link');
    styleLink.rel = 'stylesheet';
    styleLink.href = 'css/parameter-panel.css';
    iframe.contentDocument.head.appendChild(styleLink);

    // Inject JS
    const script = document.createElement('script');
    script.src = 'modules/parameter-panel.js';
    iframe.contentDocument.body.appendChild(script);
}
```

---

# Phase 4: Modify Native Chat Behavior

## 4.1 Fix Send Button & Enter Key

**Find llama.cpp's send button handler:**
- Look in `index.js` or similar for input field and send button
- Search for: `addEventListener('click')` on send button
- Search for: form submission or input handling

**Issues to fix:**
- Send button might not be bound to API call
- Enter key might submit form instead of sending message

**Fix approach:**
- Ensure send button has click listener that calls completion API
- Add Enter key listener to textarea (prevent default, send message on Enter)
- Shift+Enter should allow newline

**Example fix (if we can edit files):**
```javascript
// In llama.cpp's index.js
const textarea = document.getElementById('chat-input');
const sendBtn = document.getElementById('send-btn');

textarea.addEventListener('keydown', (e) => {
    if (e.key === 'Enter' && !e.shiftKey) {
        e.preventDefault();
        sendBtn.click();
    }
});

sendBtn.addEventListener('click', () => {
    const message = textarea.value.trim();
    if (message) {
        sendCompletionRequest(message);
    }
});
```

## 4.2 Add Auto-resize Input

**Find textarea in llama.cpp UI**
- Locate the chat input textarea element
- Check current height constraints

**Add auto-resize behavior:**
```javascript
textarea.style.minHeight = '128px';
textarea.style.maxHeight = '300px';

textarea.addEventListener('input', function() {
    this.style.height = 'auto';
    const newHeight = Math.min(
        Math.max(this.scrollHeight, 128),
        300
    );
    this.style.height = newHeight + 'px';
});
```

---

# Phase 5: Testing & Verification

## 5.1 Parameter Panel Test

**Test checklist:**
- ✓ Open terminal with loaded model
- ✓ Parameter panel appears in top-right
- ✓ All sliders display correct values
- ✓ Sliders can be dragged and update values
- ✓ Toggle button collapses/expands panel
- ✓ "Apply" button saves to localStorage
- ✓ "Reset" button restores defaults
- ✓ Refresh page - params still saved
- ✓ Panel doesn't interfere with chat input

## 5.2 Chat Functionality Test

**Test checklist:**
- ✓ Type message in textarea
- ✓ Click Send button - message is sent
- ✓ Response appears in chat
- ✓ Press Enter - message is sent
- ✓ Press Shift+Enter - newline is added
- ✓ Multiple messages work in conversation
- ✓ Streaming responses appear incrementally
- ✓ Auto-resize works as you type more text

## 5.3 Integration Test

**Test checklist:**
- ✓ Change temperature to 2.0, send creative prompt - response should be more random
- ✓ Change temperature to 0, send prompt - response should be more deterministic
- ✓ Change max_tokens - verify response length limit
- ✓ Repeat penalty works (less repetition)
- ✓ Parameters persist between page refreshes
- ✓ Panel works in both Terminal and Native Chat tabs

## 5.4 Edge Cases

**Test checklist:**
- ✓ Very long messages (>1000 chars)
- ✓ Special characters in messages
- ✓ Slider at min/max values
- ✓ Rapid message sending
- ✓ Network errors handled gracefully
- ✓ Panel visibility at different window sizes

---

# Phase 6: Fallback Subplan

**Trigger Condition:**
1. llama-server files cannot be edited (read-only, permissions, embedded)
2. Modifications don't persist after llama-server restart
3. Changes break llama-server functionality

## 6.1 Custom Llama-server Build

### Step 1: Clone llama.cpp repository
```bash
git clone https://github.com/ggerganov/llama.cpp.git
cd llama.cpp
```

### Step 2: Modify web UI
```bash
cd examples/server/public
# Edit index.html, index.js, index.css
# Add our parameter panel and fixes
```

### Step 3: Build llama-server
```bash
cd ../../..
# Windows build (requires CMake, Visual Studio)
cmake -B build -DLLAMA_CURL=ON
cmake --build build --config Release -j

# Output: build/bin/Release/llama-server.exe
```

### Step 4: Integrate with Arandu
- Copy custom llama-server.exe to Arandu's executable folder
- Update backend to use our custom build
- Version it as `llama-server-arandu-v1.exe`
- Update download logic to use custom build

### Step 5: Deployment
- Package custom llama-server with Arandu installer
- Add version checking (auto-update custom build)
- Document differences from upstream llama.cpp

### Step 6: Maintenance
- Track llama.cpp upstream releases
- Rebase our changes on new versions
- Test compatibility with Arandu

---

# Summary: Files Changed

## Files to Delete
- `frontend/modules/chat-app.js` (1,117 lines)
- `frontend/css/chat-app.css` (889 lines)
- `backend/src/chat_manager.rs` (~400 lines)
- `backend/src/chat_models.rs` (~100 lines)

## Files to Modify
- `frontend/index.html` - Remove chat-app import
- `backend/src/lib.rs` - Remove 7 chat commands, remove imports
- `frontend/modules/terminal-manager.js` - Remove custom chat tab, ChatApp initialization

## Files to Create (Method A - Edit llama-server directly)
- Edit: `llama-server/examples/server/public/index.html` - Add parameter panel HTML
- Edit: `llama-server/examples/server/public/index.js` - Add parameter panel JS, fix Enter key
- Edit: `llama-server/examples/server/public/index.css` - Add parameter panel CSS

## Files to Create (Method B - Overlay)
- `frontend/modules/parameter-panel.js` (~400 lines)
- `frontend/css/parameter-panel.css` (~200 lines)
- `frontend/modules/native-chat-patcher.js` (~100 lines) - Enter key, auto-resize

## Files to Modify (Method B)
- `frontend/modules/terminal-manager.js` - Add injection logic

## Total Change Summary
- **Deleted:** ~2,506 lines
- **Modified:** ~500 lines (removing references)
- **Added:** ~600 lines (Method A) or ~700 lines (Method B)
- **Net Change:** Methods need ~1,000 lines removed overall

## Estimated Time
- Phase 1 (Cleanup): 3-4 hours
- Phase 2 (Locate): 1-2 hours
- Phase 3 (Panel): 4-5 hours
- Phase 4 (Fix chat): 2-3 hours
- Phase 5 (Testing): 3-4 hours
- Phase 6 (Fallback): 6-8 hours (if needed)
- **Total:** 16-23 hours (depending on method needed)

## Risk Assessment
- **Low Risk:** Phase 1 (deletion), Phase 2 (investigation)
- **Medium Risk:** Phase 3 (panel integration - depends on file access)
- **High Risk:** Phase 6 (custom build - may have compatibility issues)
