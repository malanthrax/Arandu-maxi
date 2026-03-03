# Remote Model Launch Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Enable remote Arandu clients to request model launches on a server Arandu instance via REST API, allowing the server PC to auto-launch models when clicked remotely.

**Architecture:**
- Remote client POSTs to `/v1/models/launch` to trigger model launch on server
- Server's OpenAI proxy intercepts launch requests, starts llama-server process
- Server streams SSE (Server-Sent Events) to notify client when model is ready
- Remote client then opens chat iframe to server's running llama-server instance
- Models stay running until explicitly stopped via `/v1/models/stop`

**Tech Stack:**
- Backend: Rust + Tauri + Axum (HTTP framework)
- Frontend: JavaScript + vanilla DOM manipulation
- Protocol: REST API + SSE (Server-Sent Events)
- Existing infrastructure: OpenAI proxy (`openai_proxy.rs`), model launcher (`process.rs`)

---

## Prerequisites

Read these files to understand existing patterns:
- `backend/src/openai_proxy.rs` - How OpenAI proxy works
- `backend/src/process.rs` - How `launch_model_server()` works
- `frontend/modules/terminal-manager.js` - How `openNativeChatForServer()` works
- `AGENTS.md` - Project architecture and patterns

---

## Task 1: Add ActiveModel Data Structure

**Files:**
- Modify: `backend/src/models.rs:1-50` (add to file after existing structs)

**Step 1: Write the failing test (unit test in models.rs)**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::process::ProcessHandle;

    #[test]
    fn test_active_model_creation() {
        // This will fail until ActiveModel is defined
        let model = ActiveModel {
            model_path: String::from("test.gguf"),
            process: None, // Will implement with actual ProcessHandle
            port: 8080,
            clients: Vec::new(),
            status: ModelStatus::Launching,
        };
        assert_eq!(model.model_path, "test.gguf");
        assert_eq!(model.status, ModelStatus::Launching);
    }

    #[test]
    fn test_model_status_variants() {
        assert!(matches!(ModelStatus::Launching, ModelStatus::Launching));
        assert!(matches!(ModelStatus::Ready, ModelStatus::Ready));
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --manifest-path backend/Cargo.toml test_active_model_creation --lib`
Expected: FAIL - `ActiveModel` and `ModelStatus` not defined

**Step 3: Write minimal implementation**

Add to `backend/src/models.rs` (after existing structs, before impl blocks):

```rust
use crate::process::ProcessHandle;

#[derive(Debug, Clone)]
pub struct ActiveModel {
    pub model_path: String,
    pub process: Option<ProcessHandle>,
    pub port: u16,
    pub clients: Vec<String>, // Client IPs using this model
    pub status: ModelStatus,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ModelStatus {
    Launching,
    Ready,
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test --manifest-path backend/Cargo.toml test_active_model_creation --lib`
Expected: PASS

**Step 5: Commit**

```bash
git add backend/src/models.rs
git commit -m "feat: add ActiveModel and ModelStatus structs for remote launch tracking"
```

---

## Task 2: Add active_models Field to AppState

**Files:**
- Modify: `backend/src/lib.rs:1-100` (adjust AppState struct)

**Step 1: Inspect current AppState structure**

Read lines around AppState definition in `backend/src/lib.rs` to see existing fields.

**Step 2: Add active_models field to AppState**

In AppState struct (around line 100-150), add:

```rust
pub struct AppState {
    // ... existing fields ...
    pub active_models: Arc<RwLock<HashMap<String, ActiveModel>>>, // Add this
}
```

**Step 3: Initialize active_models in new() constructor**

Where AppState is created (in `setup_app()` or similar), add:

```rust
ActiveModel {
    active_models: Arc::new(RwLock::new(HashMap::new())),
}
```

**Step 4: Add ActiveModel import**

Add to imports at top of `backend/src/lib.rs`:

```rust
use crate::models::{ActiveModel, ModelStatus};
```

**Step 5: Verify compilation**

Run: `cargo check --manifest-path backend/Cargo.toml`
Expected: PASS

**Step 6: Commit**

```bash
git add backend/src/lib.rs
git commit -m "feat: add active_models tracking to AppState"
```

---

## Task 3: Implement Launch Endpoint

**Files:**
- Modify: `backend/src/openai_proxy.rs:45-100` (add to app routing)
- Modify: `backend/src/openai_proxy.rs:100-383` (add launch handler function)

**Step 1: Add launch route to Router**

In `start()` function, modify Router construction to add route:

```rust
let app = Router::new()
    .route("/v1/models", get(list_models))
    .route("/v1/models/arandu", get(list_models_arandu))
    .route("/v1/models/launch", post(launch_model_remote))  // ADD THIS
    .route("/v1/chat/completions", post(chat_completions))
    .route("/v1/audio/transcriptions", post(audio_transcriptions))
    .route("/v1/audio/speech", post(audio_speech))
    .route("/v1/images/generations", post(image_generations))
    .route("/health", get(health_check))
    // ... rest of config
```

**Step 2: Add launch handler function**

Add to `backend/src/openai_proxy.rs` (after existing handlers):

```rust
#[derive(Deserialize)]
struct LaunchRequest {
    model_path: String,
    client_ip: String,
}

#[derive(Serialize)]
struct LaunchResponse {
    status: String,
}

async fn launch_model_remote(
    State(state): State<Arc<RwLock<ProxyState>>>,
    Json(req): Json<LaunchRequest>,
) -> Result<Json<LaunchResponse>, StatusCode> {
    // This is a partial implementation - will complete in next task
    Ok(Json(LaunchResponse {
        status: "loading".to_string(),
    }))
}
```

**Step 3: Verify compilation**

Run: `cargo check --manifest-path backend/Cargo.toml`
Expected: PASS

**Step 4: Compile (no commit yet)**

Run: `cargo build --manifest-path backend/Cargo.toml`
Expected: SUCCESS

---

## Task 4: Implement Launch Logic with Process Launch

**Files:**
- Modify: `backend/src/openai_proxy.rs:100-383` (enhance launch handler)

**Step 1: Update ProxyState to hold AppState reference**

In ProxyState struct definition, add:

```rust
pub struct ProxyState {
    // ... existing fields ...
    app_state: Option<Arc<Mutex<crate::AppState>>>, // Add this
}
```

**Step 2: Pass app_state to ProxyServer**

Update `ProxyServer::new()` signature to accept app_state:

```rust
pub fn new(
    llama_server_host: String,
    llama_server_port: u16,
    proxy_port: u16,
    models_directories: Vec<String>,
    app_state: Arc<Mutex<crate::AppState>>,
) -> Self {
    Self {
        llama_server_url: format!("http://{}:{}", llama_server_host, llama_server_port),
        proxy_port,
        shutdown_tx: None,
        models_directories,
        app_state: Some(app_state),
    }
}
```

**Step 3: Implement full launch handler**

Replace the placeholder `launch_model_remote` function with:

```rust
async fn launch_model_remote(
    State(state): State<Arc<RwLock<ProxyState>>>,
    Json(req): Json<LaunchRequest>,
) -> Result<Json<LaunchResponse>, StatusCode> {
    info!("Remote launch request for model: {} from client: {}", req.model_path, req.client_ip);

    // Check if model already running
    let state_guard = state.read().await;
    if let Some(app_state) = &state_guard.app_state {
        let app_lock = app_state.lock().await;
        let active_models = app_lock.active_models.read().await;

        if active_models.contains_key(&req.model_path) {
            // Model already running, add client to list
            drop(active_models);
            let mut active_models_w = app_lock.active_models.write().await;
            if let Some(model) = active_models_w.get_mut(&req.model_path) {
                if !model.clients.contains(&req.client_ip) {
                    model.clients.push(req.client_ip.clone());
                }
            }
            return Ok(Json(LaunchResponse {
                status: "ready".to_string(),
            }));
        }
    }

    // Model not running, need to launch
    // For now, return error - will implement in next task
    Err(StatusCode::NOT_IMPLEMENTED)
}
```

**Step 4: Test endpoint exists**

Run: `cargo run --manifest-path backend/Cargo.toml` (in separate terminal)
Then: `curl -X POST http://localhost:8081/v1/models/launch -H "Content-Type: application/json" -d '{"model_path":"test.gguf","client_ip":"127.0.0.1"}'`
Expected: `501 Not Implemented` (endpoint exists but not fully implemented)

**Step 5: Commit**

```bash
git add backend/src/openai_proxy.rs
git commit -m "feat: add /v1/models/launch endpoint stub with duplicate check"
```

---

## Task 5: Implement Actual Model Launch in Handler

**Files:**
- Modify: `backend/src/openai_proxy.rs:100-383` (complete launch handler)

**Step 1: Add imports at top of file**

```rust
use crate::AppState;
use crate::process::launch_model_server;
use crate::models::{ActiveModel, ModelStatus};
```

**Step 2: Implement full launch with AppState access**

Replace the `launch_model_remote` implementation:

```rust
async fn launch_model_remote(
    State(state): State<Arc<RwLock<ProxyState>>>,
    Json(req): Json<LaunchRequest>,
) -> Result<Json<LaunchResponse>, StatusCode> {
    info!("Remote launch request for model: {} from client: {}", req.model_path, req.client_ip);

    let app_state_opt = {
        let state_guard = state.read().await;
        state_guard.app_state.clone()
    };

    let app_state = app_state_opt.ok_or_else(|| {
        error!("AppState not available in ProxyState");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // Check if model already running
    {
        let app_lock = app_state.lock().await;
        let active_models = app_lock.active_models.read().await;

        if active_models.contains_key(&req.model_path) {
            // Model already running, add client to list
            drop(active_models);
            let mut active_models_w = app_lock.active_models.write().await;
            if let Some(model) = active_models_w.get_mut(&req.model_path) {
                if !model.clients.contains(&req.client_ip) {
                    model.clients.push(req.client_ip.clone());
                }
            }
            return Ok(Json(LaunchResponse {
                status: "ready".to_string(),
            }));
        }
    }

    // Launch the model
    let launch_result = {
        let app_lock = app_state.lock().await;
        launch_model_server(req.model_path.clone(), &app_lock).await
    };

    match launch_result {
        Ok(launch_info) => {
            // Register as active model
            let active_model = ActiveModel {
                model_path: req.model_path.clone(),
                process: None, // Process handle would come from launch_result in real implementation
                port: launch_info.port,
                clients: vec![req.client_ip],
                status: ModelStatus::Ready,
            };

            let mut active_models = app_state.lock().await.active_models.write().await;
            active_models.insert(req.model_path.clone(), active_model);

            info!("Model {} launched successfully on port {}", req.model_path, launch_info.port);
            Ok(Json(LaunchResponse {
                status: "ready".to_string(),
            }))
        }
        Err(e) => {
            error!("Failed to launch model {}: {}", req.model_path, e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
```

**Step 3: Add LaunchResult struct if not exists**

Check `backend/src/models.rs` for `LaunchResult` struct. Add if missing:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LaunchResult {
    pub window_label: String,
    pub process_id: String,
    pub port: u16,
    pub success: bool,
}
```

**Step 4: Verify compilation**

Run: `cargo check --manifest-path backend/Cargo.toml`
Expected: PASS

**Step 5: Run backend tests**

Run: `cargo test --manifest-path backend/Cargo.toml --lib -- --quiet`
Expected: All tests pass

**Step 6: Commit**

```bash
git add backend/src/openai_proxy.rs backend/src/models.rs
git commit -m "feat: implement model launch logic in launch endpoint"
```

---

## Task 6: Implement Stop Model Endpoint

**Files:**
- Modify: `backend/src/openai_proxy.rs:45-100` (add route)
- Modify: `backend/src/openai_proxy.rs:100-383` (add stop handler)

**Step 1: Add stop route**

In Router construction, add:

```rust
.route("/v1/models/stop", post(stop_model_remote))
```

**Step 2: Add stop handler function**

```rust
#[derive(Deserialize)]
struct StopRequest {
    model_path: String,
}

#[derive(Serialize)]
struct StopResponse {
    status: String,
}

async fn stop_model_remote(
    State(state): State<Arc<RwLock<ProxyState>>>,
    Json(req): Json<StopRequest>,
) -> Result<Json<StopResponse>, StatusCode> {
    info!("Stop request for model: {}", req.model_path);

    let app_state_opt = {
        let state_guard = state.read().await;
        state_guard.app_state.clone()
    };

    let app_state = app_state_opt.ok_or_else(|| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Remove from active models
    let active_model = {
        let mut active_models = app_state.lock().await.active_models.write().await;
        active_models.remove(&req.model_path)
    };

    if let Some(model) = active_model {
        // Kill the process (will implement process killing in next task)
        info!("Model {} stopped", req.model_path);
        Ok(Json(StopResponse {
            status: "stopped".to_string(),
        }))
    } else {
        warn!("Model {} not found in active models", req.model_path);
        Err(StatusCode::NOT_FOUND)
    }
}
```

**Step 3: Verify compilation**

Run: `cargo check --manifest-path backend/Cargo.toml`
Expected: PASS

**Step 4: Commit**

```bash
git add backend/src/openai_proxy.rs
git commit -m "feat: add /v1/models/stop endpoint"
```

---

## Task 7: Implement Active Models List Endpoint

**Files:**
- Modify: `backend/src/openai_proxy.rs:45-100` (add route)
- Modify: `backend/src/openai_proxy.rs:100-383` (add list handler)

**Step 1: Add active route**

In Router construction, add:

```rust
.route("/v1/models/active", get(list_active_models))
```

**Step 2: Add list handler function**

```rust
#[derive(Serialize)]
struct ActiveModelsResponse {
    models: Vec<ActiveModelInfo>,
}

#[derive(Serialize)]
struct ActiveModelInfo {
    model_path: String,
    port: u16,
    status: String,
    client_count: usize,
}

async fn list_active_models(
    State(state): State<Arc<RwLock<ProxyState>>>,
) -> Result<Json<ActiveModelsResponse>, StatusCode> {
    let app_state_opt = {
        let state_guard = state.read().await;
        state_guard.app_state.clone()
    };

    let app_state = app_state_opt.ok_or_else(|| StatusCode::INTERNAL_SERVER_ERROR)?;

    let active_models = app_state.lock().await.active_models.read().await;

    let models: Vec<ActiveModelInfo> = active_models.values().map(|m| {
        ActiveModelInfo {
            model_path: m.model_path.clone(),
            port: m.port,
            status: match m.status {
                ModelStatus::Launching => "launching".to_string(),
                ModelStatus::Ready => "ready".to_string(),
            },
            client_count: m.clients.len(),
        }
    }).collect();

    Ok(Json(ActiveModelsResponse { models }))
}
```

**Step 3: Verify compilation**

Run: `cargo check --manifest_path backend/Cargo.toml`
Expected: PASS

**Step 4: Commit**

```bash
git add backend/src/openai_proxy.rs
git commit -m "feat: add /v1/models/active endpoint"
```

---

## Task 8: Update ProxyServer Initialization to Pass AppState

**Files:**
- Modify: `backend/src/lib.rs` (find where ProxyServer::new is called)

**Step 1: Find ProxyServer instantiation**

Search for `ProxyServer::new(` in `backend/src/lib.rs`

**Step 2: Update to pass app_state**

Where ProxyServer is created, add app_state parameter:

```rust
let proxy_server = ProxyServer::new(
    String::from("127.0.0.1"),
    8080, // llama-server port
    openai_port,
    config.models_directories.clone(),
    Arc::clone(&app_state),
);
```

**Step 3: Verify compilation**

Run: `cargo check --manifest_path backend/Cargo.toml`
Expected: PASS

**Step 4: Run backend tests**

Run: `cargo test --manifest_path backend/Cargo.toml --lib -- --quiet`
Expected: All tests pass

**Step 5: Commit**

```bash
git add backend/src/lib.rs
git commit -m "feat: pass app_state to ProxyServer initialization"
```

---

## Task 9: Frontend - Update openNativeChatForServer to Launch First

**Files:**
- Modify: `frontend/modules/terminal-manager.js:1569-1627` (update function)

**Step 1: Replace openNativeChatForServer implementation**

```javascript
async openNativeChatForServer(modelName, host, port, modelPath) {
    const url = `http://${host}:${port}`;
    const windowId = `native_chat_${Date.now()}`;

    // Step 1: Launch model on server first
    showToast(`${modelName} launching on remote host...`, 'info');

    try {
        const launchResponse = await fetch(`http://${host}:${port}/v1/models/launch`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({
                model_path: modelPath,
                client_ip: '127.0.0.1' // Will dynamically detect in real implementation
            })
        });

        if (!launchResponse.ok) {
            throw new Error(`Launch failed: ${launchResponse.status}`);
        }

        const launchData = await launchResponse.json();

        if (launchData.status === 'ready') {
            showToast(`${modelName} ready on remote host`, 'success');
        } else if (launchData.status === 'loading') {
            showToast(`${modelName} is loading...`, 'info');
            // In real implementation, would poll or use SSE
            await new Promise(resolve => setTimeout(resolve, 2000));
        }

        // Step 2: Open the chat window
        const content = `
            <div style="width: 100%; height: 100%; display: flex; flex-direction: column; background: white;">
                <iframe src="${url}" frameBorder="0" style="flex: 1; border: none; width: 100%; height: 100%;"></iframe>
            </div>
        `;

        this.desktop.createWindow(windowId, `Remote Chat - ${modelName} (${host}:${port})`, 'browser-window', content);

        const windowElement = this.desktop.windows.get(windowId);
        if (windowElement) {
            windowElement.style.width = '1000px';
            windowElement.style.height = '800px';
            const left = (window.innerWidth - 1000) / 2;
            const top = (window.innerHeight - 800) / 2;
            windowElement.style.left = `${Math.max(50, left)}px`;
            windowElement.style.top = `${Math.max(50, top)}px`;
            windowElement.style.zIndex = this.desktop.windowZIndex + 1;
            this.desktop.windowZIndex += 1;

            this.desktop.addTaskbarItem(`Remote Chat - ${modelName} (${host}:${port})`, windowId, '<span class="material-icons">open_in_browser</span>');

            const iframe = windowElement.querySelector('iframe');
            if (iframe) {
                const blurHandler = () => {
                    if (document.activeElement === iframe) {
                        windowElement.style.zIndex = ++this.desktop.windowZIndex;
                        document.querySelectorAll('.window').forEach(w => w.classList.remove('active'));
                        windowElement.classList.add('active');
                        document.querySelectorAll('.taskbar-item').forEach(t => t.classList.remove('active'));
                        const taskbarItem = document.getElementById(`taskbar-${windowId}`);
                        if (taskbarItem) taskbarItem.classList.add('active');
                    }
                };
                window.addEventListener('blur', blurHandler);
            }
        }

        return { success: true, port };

    } catch (error) {
        console.error('Failed to launch remote model:', error);
        showToast(`Failed to launch ${modelName}: ${error.message}`, 'error');
        return { success: false, error: error.message };
    }
}
```

**Step 2: Update caller to pass modelPath**

In `frontend/desktop.js`, find where `openNativeChatForServer` is called and add modelPath parameter.

**Step 3: Verify JavaScript syntax**

Run: `node --check frontend/modules/terminal-manager.js`
Expected: No syntax errors

**Step 4: Commit**

```bash
git add frontend/modules/terminal-manager.js frontend/desktop.js
git commit -m "feat: add remote model launch before opening chat window"
```

---

## Task 10: Frontend - Add Stop Remote Model Functionality

**Files:**
- Modify: `frontend/modules/terminal-manager.js` (add stopRemoteModel function)
- Modify: `frontend/desktop.js` (add context menu item for stop)

**Step 1: Add stopRemoteModel function to TerminalManager**

```javascript
async stopRemoteModel(modelName, host, port, modelPath) {
    try {
        const response = await fetch(`http://${host}:${port}/v1/models/stop`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({
                model_path: modelPath
            })
        });

        if (!response.ok) {
            throw new Error(`Stop failed: ${response.status}`);
        }

        const data = await response.json();
        showToast(`${modelName} stopped on remote host`, 'success');
        return { success: true };
    } catch (error) {
        console.error('Failed to stop remote model:', error);
        showToast(`Failed to stop ${modelName}: ${error.message}`, 'error');
        return { success: false, error: error.message };
    }
}
```

**Step 2: Add "Stop Remote Model" context menu item**

In `handleRemoteModelClick` or remote model context menu in `desktop.js`, add option:

```javascript
{
    label: 'Stop Remote Model',
    icon: 'stop',
    action: async () => {
        const result = await terminalManager.stopRemoteModel(
            model.name,
            peer.ip_address || peerIp,
            peer.api_port || peerPort,
            model.id
        );
        if (result.success) {
            // Refresh remote models list to update status
            renderRemoteModelsList();
        }
    }
}
```

**Step 3: Verify JavaScript syntax**

Run: `node --check frontend/modules/terminal-manager.js frontend/desktop.js`
Expected: No syntax errors

**Step 4: Commit**

```bash
git add frontend/modules/terminal-manager.js frontend/desktop.js
git commit -m "feat: add stop remote model functionality"
```

---

## Task 11: Testing - Build and Smoke Test

**Files:**
- Build: `backend/target/release/Arandu.exe`

**Step 1: Build release executable**

Run: `cd backend && cargo tauri build --no-bundle`
Expected: Success, executable at `backend/target/release/Arandu.exe`

**Step 2: Verify server starts**

Run: `backend/target/release/Arandu.exe --help`
Expected: Help text displayed

**Step 3: Test launch endpoint manually**

Start server, then:
```bash
curl -X POST http://localhost:8081/v1/models/launch \
  -H "Content-Type: application/json" \
  -d '{"model_path":"test.gguf","client_ip":"127.0.0.1"}'
```

Expected: `{"status":"ready"}` or error message

**Step 4: Test active models endpoint**

```bash
curl http://localhost:8081/v1/models/active
```

Expected: `{"models":[]}` or list of running models

**Step 5: Commit build info**

Create `docs/knowledge-base/2026-03-01-remote-launch-build-and-smoke-test.md` with:
- Build command and timestamp
- Test results
- Any issues encountered

**Step 6: Commit docs**

```bash
git add docs/knowledge-base/2026-03-01-remote-launch-build-and-smoke-test.md
git commit -m "docs: add remote launch build and smoke test results"
```

---

## Task 12: Final Integration Test

**Files:**
- Test: Two Arandu instances on network

**Step 1: Setup test environment**
- Server PC: Start Arandu, enable discovery, ensure model available
- Client PC: Start Arandu, enable discovery

**Step 2: Discovery test**
- Verify client sees server in peer list
- Verify client sees remote models

**Step 3: Remote launch test**
- Client clicks remote model
- Server shows model launch (terminal appears)
- Client sees "Model loading..." toast
- Client sees "Model ready" toast
- Client opens chat window

**Step 4: Chat test**
- Client sends message
- Server receives and responds
- Response visible in client chat window

**Step 5: Stop test**
- Client clicks "Stop Remote Model"
- Server terminates process
- Model no longer in active list

**Step 6: Document results**

Create `docs/knowledge-base/2026-03-01-remote-launch-integration-test.md` with:
- Test scenarios executed
- Pass/fail for each scenario
- Screenshots if possible
- Any issues found

**Step 7: Commit docs**

```bash
git add docs/knowledge-base/2026-03-01-remote-launch-integration-test.md
git commit -m "docs: add remote launch integration test results"
```

---

## Documentation Updates

After completing all tasks, update:

### Update AGENTS.md

Add section in "Network Discovery" about remote model launch API.

### Update docs/INDEX.md

Add entry for remote launch implementation docs.

### Update docs/USER-MANUAL.md

Add section in "Using Remote Models" about auto-launch.

---

## Rollback Plan

If issues arise:

1. **Revert to last working commit**: `git revert HEAD`
2. **Fix data**: Manually remove any orphaned processes
3. **Rebuild**: `cargo tauri build --no-bundle`

---

## Success Criteria

✅ Server can launch model when requested remotely
✅ Remote client sees toast notifications
✅ Chat window opens identical to local model
✅ Multiple clients can connect to same model
✅ Stop functionality works correctly
✅ No dead processes or memory leaks
✅ Documentation updated

---

**Estimated total time:** 2-3 hours (assuming no unexpected issues)