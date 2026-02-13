# Arandu - Developer Documentation

## Project Overview

**Arandu** is a Tauri-based desktop application that provides a user-friendly UI for managing llama.cpp models and servers. It eliminates the need to manually handle DLL files or command-line arguments by providing a complete desktop environment with model management, HuggingFace integration, automatic llama.cpp backend downloads, and hardware monitoring.

**Version:** 0.5.5-beta  
**Tech Stack:** Rust (Tauri backend) + Vanilla JavaScript (frontend)  
**Platforms:** Windows only

---

## Quick Start

```bash
# Development
cargo tauri dev

# Build (output: backend/target/release/)
cargo tauri build
```

---

## Architecture

### Backend (`backend/`)

**Entry Points:**
- `src/main.rs` - Application entry
- `src/lib.rs` - Main library with Tauri commands and AppState
- `Cargo.toml` - Dependencies and build config

**Core Modules:**

| File | Purpose | Key Types/Functions |
|------|---------|---------------------|
| `lib.rs` | Main entry, Tauri commands, AppState | `AppState`, Tauri command handlers |
| `models.rs` | Data structures | `GlobalConfig`, `ModelConfig`, `ProcessInfo`, `DownloadStatus` |
| `config.rs` | Config persistence | `load_settings()`, `save_settings()` |
| `scanner.rs` | Model file discovery | `scan_models()`, `scan_mmproj_files()` |
| `huggingface.rs` | HF API integration | `search_huggingface_models()`, `fetch_model_files()` |
| `downloader.rs` | Download management | `DownloadManager`, pause/resume/extract |
| `llamacpp_manager.rs` | GitHub releases | `fetch_llamacpp_releases()`, release caching |
| `process.rs` | Process management | `launch_model_internal()`, `launch_model_external()`, `ProcessHandle` |
| `system_monitor.rs` | Hardware monitoring | `SystemMonitor`, RAM/VRAM tracking |

**Key Dependencies:**
```toml
tauri = { version = "2.10.2", features = ["tray-icon", "devtools"] }
tokio = { version = "1.49.0", features = ["full"] }
reqwest = { version = "0.13.2", features = ["json", "stream"] }
sysinfo = { version = "0.38.1", features = ["serde"] }
nvml-wrapper = "0.11.0"  # NVIDIA GPU monitoring
zip = "7.4.0"            # Backend extraction
```

### Frontend (`frontend/`)

**Structure:**
```
frontend/
├── index.html           # Main HTML shell
├── desktop.js           # Core desktop logic, window management
├── css/
│   ├── main.css         # Base styles, theme CSS vars
│   ├── desktop.css      # Desktop icons, dock, taskbar
│   ├── windows.css      # Window system
│   ├── properties.css   # Model properties panel
│   ├── huggingface.css  # HF search UI
│   ├── llama-manager.css # Backend manager UI
│   └── ...
└── modules/
    ├── theme-definitions.js      # Theme color palettes
    ├── huggingface-app.js        # HF search & download
    ├── terminal-manager.js       # Process terminals
    ├── properties-manager.js     # Model settings UI
    ├── download-manager.js       # Download progress UI
    ├── llamacpp-manager.js       # Backend manager
    ├── module-manager.js         # Dynamic module loader
    ├── modal-dialog.js           # Dialog system
    └── search-history.js         # Search history
```

---

## Configuration

**Config Location:** `%USERPROFILE%\.Arandu\config.json`

**GlobalConfig Structure:**
```rust
struct GlobalConfig {
    models_directory: String,        // Where .gguf files are stored
    executable_folder: String,       // Where llama-server is installed
    active_executable_folder: Option<String>, // Currently active backend
    active_executable_version: Option<String>, // Version of active backend
    theme_color: String,             // UI theme
    background_color: String,        // Desktop background
    theme_is_synced: bool,           // UI/bg sync toggle
}
```

**Default Paths:**
- Models: `~/.Arandu/models`
- Executables: `~/.Arandu/llama.cpp`
- Downloads: Same as models_directory

---

## Backend Management (llama.cpp)

**Download Flow:**
1. `llamacpp_manager.rs::fetch_llamacpp_releases()` - Fetches from GitHub API with 10min cache
2. User selects release/asset in UI
3. `downloader.rs` downloads ZIP to `executable_folder/versions/`
4. Auto-extraction creates `versions/<tag>/<backend_type>/`
5. `detect_backend_type()` parses asset names (cuda, rocm, vulkan, cpu, etc.)

**Backend Type Detection:**
```rust
fn detect_backend_type(asset_name: &str) -> String {
    // cuda, cudart → "cuda"
    // rocm, hip → "rocm"
    // vulkan → "vulkan"
    // opencl → "opencl"
    // metal → "metal"
    // cpu or unknown → "cpu"
}
```

**Path Resolution:**
```rust
// Priority order:
1. active_executable_folder (user preference)
2. Latest version in executable_folder/versions/*/ (newest created time)
3. executable_folder (legacy flat structure)
```

---

## Process Management

**Launch Types:**

1. **Internal** - Uses built-in web UI on port 8080
   - Command: `llama-server.exe -m <model> [args]`
   - UI served at `http://127.0.0.1:8080`
   
2. **External** - Opens in separate window
   - Same command but opens llama.cpp's native web UI

**ProcessHandle Architecture:**
```rust
pub struct ProcessHandle {
    child: Option<Child>,           // tokio::process::Child
    kill_on_drop: bool,             // Ensures cleanup
}

// Drop impl kills process if kill_on_drop=true
```

**Cleanup Strategy:**
- Graceful: `child.kill().await` 
- Force (Windows): `taskkill /PID <pid> /F /T`
- Force (Unix): `kill -9 <pid>`
- App exit: `AppState::cleanup_all_processes()`

---

## Model Management

**Scanning:**
- Recursively scans `models_directory` for `*.gguf` files
- Extracts metadata: architecture, name, quantization from filename patterns
- File naming convention: `<name>-<quant>.gguf` or `<name>-<arch>-<quant>.gguf`

**Model Settings:**
```rust
struct ModelConfig {
    custom_args: String,        // Additional llama-server arguments
    server_host: String,        // Default: 127.0.0.1
    server_port: u16,           // Default: 8080
    model_path: String,
    presets: Vec<ModelPreset>,  // Saved argument presets
    default_preset_id: Option<String>,
}
```

---

## HuggingFace Integration

**API:** Uses HF API v1 with search and model file listing.

**Search:** `huggingface.rs::search_huggingface_models()`
- Endpoint: `https://huggingface.co/api/models`
- Parameters: search query, limit, filters for GGUF

**File Listing:** `fetch_model_files()`
- Gets repository tree via HF API
- Filters for `.gguf` files
- Extracts quantization from filename

**Download:**
- URL pattern: `https://huggingface.co/<model_id>/resolve/main/<filename>`
- Supports resume (if server supports range requests)
- Auto-extracts after download (for archives)
- Cleanup of partial downloads on failure

---

## Download System

**States:**
```rust
enum DownloadState {
    Starting, Downloading, Paused, Extracting, 
    Completed, Failed, Cancelled
}
```

**Features:**
- Pause/resume support
- Progress tracking (bytes/s, ETA)
- Cancellation tokens
- Multi-file downloads
- ZIP extraction
- Progress events via Tauri emit

**DownloadManager:**
```rust
pub struct DownloadManager {
    downloads: HashMap<String, DownloadStatus>,      // Active
    download_history: Vec<DownloadStatus>,            // Completed
    cancellation_tokens: HashMap<String, Arc<Mutex<bool>>>,
}
```

---

## System Monitoring

**SystemMonitor** polls every 2 seconds:
- **RAM:** Used/total via `sysinfo`
- **VRAM:** NVIDIA GPUs via `nvml-wrapper`  
- Events emitted to frontend for real-time bars

**Frontend Display:**
- Desktop overlay with RAM/VRAM bars
- Tooltip with detailed specs
- Color-coded usage (green/yellow/red)

---

## Theme System

**Theme Definitions:** `frontend/modules/theme-definitions.js`
- Navy (default), Dark Gray, Purple, Green, Red, Orange, Blue, Pink
- Each theme has: primary, light, dark, accent, bg, surface, text variants

**CSS Variables:** Applied to `:root`
```css
--theme-primary, --theme-light, --theme-dark
--theme-accent, --theme-bg, --theme-surface
--theme-text, --theme-text-muted, --theme-border
/* ... 25+ variables */
```

**Sync Mode:** When enabled, background follows UI theme

---

## Tauri Commands Reference

### Configuration
- `get_config()` → `GlobalConfig`
- `save_config(models_dir, exec_folder, theme, bg, synced)` → Result

### Models
- `scan_models_command()` → `{success, models: [ModelInfo]}`
- `get_model_settings(model_path)` → `ModelConfig`
- `update_model_settings(model_path, config)` → Result
- `save_model_preset(model_path, preset)` → Result
- `delete_model_preset(model_path, preset_id)` → Result

### Process Management  
- `launch_model_internal(model_path, config, window_label)` → `LaunchResult`
- `launch_model_external(model_path, config, window_label)` → `LaunchResult`
- `stop_process(process_id)` → Result
- `get_process_status(process_id)` → `ProcessOutput`
- `kill_all_processes()` → Result

### HuggingFace
- `search_huggingface_models(query, limit)` → `SearchResult`
- `fetch_model_details(model_id)` → `ModelDetails`
- `get_hf_suggestions()` → `[{id, name, author}]`

### Downloads
- `start_download(url, filename, destination)` → `DownloadStartResult`
- `start_llamacpp_download(url, filename, version, backend_type)` → Result
- `pause_download(download_id)` → Result
- `resume_download(download_id)` → Result
- `cancel_download(download_id)` → Result
- `get_download_status(download_id)` → `DownloadStatus`
- `get_all_downloads()` → `[DownloadStatus]`
- `clear_completed_downloads()` → Result

### Llama.cpp Releases
- `get_llamacpp_releases()` → `[LlamaCppReleaseFrontend]`
- `refresh_llamacpp_releases()` → Result (force cache refresh)
- `fetch_commit_info(tag_name)` → `CommitInfo`

### System
- `browse_folder(input_id)` → Opens native folder dialog
- `get_app_version()` → String
- `open_external_link(url)` → Opens browser

---

## File Structure

```
Arandu/
├── backend/
│   ├── Cargo.toml
│   ├── tauri.conf.json        # Tauri config
│   ├── build.rs
│   ├── icons/
│   └── src/
│       ├── main.rs
│       ├── lib.rs             # Main entry + commands
│       ├── models.rs          # All data structures
│       ├── config.rs          # Config I/O
│       ├── scanner.rs         # Model scanning
│       ├── huggingface.rs     # HF API client
│       ├── downloader.rs      # Download manager
│       ├── llamacpp_manager.rs # GitHub releases
│       ├── process.rs         # Process spawning
│       └── system_monitor.rs  # Hardware monitoring
├── frontend/
│   ├── index.html
│   ├── desktop.js
│   ├── css/                   # All stylesheets
│   ├── assets/                # Logo, icons
│   ├── modules/               # JS modules
│   └── *.json                 # Config/data files
└── README.md
```

---

## Development Guidelines

### Adding New Commands
1. Add command function in `backend/src/lib.rs` with `#[tauri::command]`
2. Register in `tauri::Builder` command list
3. Call from frontend via `window.__TAURI__.invoke('command_name', args)`

### Error Handling
- Use `Result<T, String>` for commands
- Log errors with `eprintln!()` or `tracing`
- Return user-friendly error messages to frontend

### Async Patterns
- All commands are async
- Use `state.config.lock().await` for state access
- Long operations should emit progress events

### Frontend Patterns
- Windows: Use `desktop.createWindow()`, `desktop.minimize/closeWindow()`
- Events: Listen via `window.__TAURI__.event.listen()`
- State: Use `window.desktop` global for desktop state

---

## Common Issues

**GitHub API Rate Limiting:**
- 60 requests/hour unauthenticated
- 10-minute cache in `llamacpp_manager.rs`
- Error 403 when exceeded

**Process Cleanup:**
- Windows: Sometimes requires `taskkill /F`
- Check Task Manager for orphan llama-server processes

**GPU Monitoring:**
- Requires NVIDIA drivers
- Falls back to CPU-only if NVML fails

**Config Not Saving:**
- Check permissions on `%USERPROFILE%\.Arandu\`
- Verify JSON validity in config.json

---

## Build & Release

**Prerequisites:**
- Rust 1.70+
- Node.js (for Tauri CLI)
- Windows: Visual Studio Build Tools

**Build:**
```bash
cd backend
cargo tauri build
# Output: backend/target/release/Arandu.exe
```

**Installer:**
- NSIS installer generated by Tauri
- Icon: `backend/icons/icon.ico`

---

## Skills Applied

- **vercel-react-best-practices:** For frontend performance patterns (though this uses vanilla JS, patterns still apply)
- **plan:** For creating implementation plans for new features

---

## Todo (from README)

- [ ] Cleanup code and organize it better
- [ ] New features (coming soon)

## Contact

- GitHub: https://github.com/fredconex/Arandu
- Releases: https://github.com/fredconex/Arandu/releases
