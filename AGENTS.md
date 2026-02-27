# Arandu - Developer Documentation

> **ðŸ“‹ For current project status, bug fixes, and recent changes:** See [THIS-PROJECTS-CURRENT-STATE.md](THIS-PROJECTS-CURRENT-STATE.md)
> 
> **ðŸ“ For file locations and where to find specific code:** Check the knowledge base memory (Arandu Complete File Location Reference) before using shell commands

## Agent Quick Reference

**Before you start:**
1. **Check THIS-PROJECTS-CURRENT-STATE.md** for recent bugs, fixes, and what's already been done
2. **Check the knowledge base memory** (Arandu Complete File Location Reference) to find files - avoid shell commands when possible
3. **AGENTS.md** (this file) = Architecture, patterns, and how-to guides
4. **Use shell commands as fallback** only when memory doesn't have the answer

**File location priorities:**
1. **Knowledge base memory** (search `H:\Ardanu Fix\Arandu-maxi\docs\knowledge-base`) - **ALWAYS CHECK HERE FIRST**
2. AGENTS.md File Structure section
3. Shell commands (ls, find, grep) - use only as a last resort

**MANDATORY RULE: MEMORY FIRST**
You must always search `H:\Ardanu Fix\Arandu-maxi\docs\knowledge-base` for file locations before using any file system commands. If you find a path, use it. If you change a path or find a new one, you MUST update `docs/knowledge-base` immediately to keep it current.

### NOWLEDGE-MEM REQUIREMENT (CRITICAL)

- Use the nowledge-mem MCP endpoint declared in `tools.yaml` for project memory operations.
- Ensure every significant item is captured in nowledge mem, including:
  - errors and issues
  - bug fixes
  - ideas and decisions
  - plans and implementation steps
  - code changes and file locations
- If required project memory/instruction data is not in `AGENTS.md`, update it immediately.
- When practical, include concise evidence in memory entries (commands run, file paths, and outcomes).

---

## Project Overview

**Arandu** is a Tauri-based desktop application that provides a user-friendly UI for managing llama.cpp models and servers. It eliminates the need to manually handle DLL files or command-line arguments by providing a complete desktop environment with model management, HuggingFace integration, automatic llama.cpp backend downloads, and hardware monitoring.

**Version:** 0.5.5-beta  
**Tech Stack:** Rust (Tauri backend) + Vanilla JavaScript (frontend)  
**Platforms:** Windows only

---

## Quick Start

See [README.md](README.md#quick-start) for prerequisites and setup instructions.

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
| `huggingface_downloader.rs` | HF Direct Link Download | `parse_model_id()`, `fetch_model_info()`, `fetch_model_files()` |
| `downloader.rs` | Download management | `DownloadManager`, pause/resume/extract |
| `llamacpp_manager.rs` | GitHub releases | `fetch_llamacpp_releases()`, release caching |
| `process.rs` | Process management | `launch_model_internal()`, `launch_model_external()`, `ProcessHandle` |
| `system_monitor.rs` | Hardware monitoring | `SystemMonitor`, RAM/VRAM tracking |
| `gguf_parser.rs` | GGUF metadata parsing | `parse_gguf_metadata()`, `get_file_modification_date()` |
| `update_checker.rs` | HF update checking | `check_huggingface_updates()`, `link_model_to_hf()`, `extract_hf_model_id_from_path()` |
| `tracker_scraper.rs` | HF trending models | `fetch_trending_models()`, `fetch_model_details()`, `fetch_model_files()` |
| `tracker_manager.rs` | Local tracker storage | `TrackerManager`, `save_models()`, `get_models()`, `get_stats()` |

**Key Dependencies:**
```toml
tauri = { version = "2.10.2", features = ["tray-icon", "devtools"] }
tokio = { version = "1.49.0", features = ["full"] }
reqwest = { version = "0.13.2", features = ["json", "stream"] }
sysinfo = { version = "0.38.1", features = ["serde"] }
nvml-wrapper = "0.11.0"  # NVIDIA GPU monitoring
zip = "7.4.0"            # Backend extraction
gguf-rs-lib = "0.2"      # GGUF metadata parsing
```

### Frontend (`frontend/`)

**Structure:**
```
frontend/
â”œâ”€â”€ index.html           # Main HTML shell
â”œâ”€â”€ desktop.js           # Core desktop logic, window management
â”œâ”€â”€ css/
â”‚   â”œâ”€â”€ main.css         # Base styles, theme CSS vars
â”‚   â”œâ”€â”€ desktop.css      # Desktop icons, dock, taskbar
â”‚   â”œâ”€â”€ windows.css      # Window system
â”‚   â”œâ”€â”€ properties.css   # Model properties panel
â”‚   â”œâ”€â”€ huggingface.css  # HF search UI
â”‚   â”œâ”€â”€ llama-manager.css # Backend manager UI
â”‚   â””â”€â”€ ...
â””â”€â”€ modules/
    â”œâ”€â”€ theme-definitions.js      # Theme color palettes
    â”œâ”€â”€ huggingface-app.js        # HF search & download
    â”œâ”€â”€ terminal-manager.js       # Process terminals
    â”œâ”€â”€ properties-manager.js     # Model settings UI
    â”œâ”€â”€ download-manager.js       # Download progress UI
    â”œâ”€â”€ llamacpp-manager.js       # Backend manager
    â”œâ”€â”€ module-manager.js         # Dynamic module loader
    â”œâ”€â”€ modal-dialog.js           # Dialog system
    â””â”€â”€ search-history.js         # Search history
```

---

## Configuration

**Config Location:** `%USERPROFILE%\.Arandu\config.json`

**GlobalConfig Structure:**
```rust
struct GlobalConfig {
    models_directory: String,        // Primary directory for .gguf files
    additional_models_directories: Vec<String>, // Up to 2 extra directories
    executable_folder: String,       // Where llama-server is installed
    active_executable_folder: Option<String>, // Currently active backend
    active_executable_version: Option<String>, // Version of active backend
    theme_color: String,             // UI theme
    background_color: String,        // Desktop background
    theme_is_synced: bool,           // UI/bg sync toggle
}
```

**Default Paths:**
- Primary Models: `~/.Arandu/models`
- Additional Models: User-defined (up to 2)
- Executables: `~/.Arandu/llama.cpp`
- Downloads: Always go to primary `models_directory`

**Multiple Directories:**
- Models are scanned from all directories and merged
- Duplicate files (same path) automatically deduplicated
- File operations (delete, check exists) work across all directories
- Backward compatible - existing configs continue to work

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
    // cuda, cudart â†’ "cuda"
    // rocm, hip â†’ "rocm"
    // vulkan â†’ "vulkan"
    // opencl â†’ "opencl"
    // metal â†’ "metal"
    // cpu or unknown â†’ "cpu"
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
   - UI served at `http://127.0.0.1:8080` (or dynamic port)
   - Serves custom UI from `frontend/llama-custom` via `--path`
   
2. **External** - Opens in separate window
   - Same command but opens llama.cpp's native web UI

**Parameter Bridge (iframe -> Parent):**
The custom chat UI communicates with Arandu via `window.parent.postMessage`.
- `type: 'request-restart'`: Triggers a server restart with updated `custom_args` and `env_vars`.
- Handled by `TerminalManager.handleRestartRequest()` in `frontend/modules/terminal-manager.js`.

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

**Quantization Color Bars:**
Visual indicators on desktop icons showing quantization bit-level:

| Bit Level | Color | Hex | Example Quantizations |
|-----------|-------|-----|----------------------|
| 1-bit | Deep Red | `#DC2626` | IQ1_S, IQ1_M |
| 2-bit | Orange-Red | `#EA580C` | IQ2_XS, Q2_0, Q2_K |
| 3-bit | Orange | `#F97316` | IQ3_XS, Q3_0, Q3_K |
| 4-bit | Yellow-Orange | `#F59E0B` | Q4_K_M, Q4_0, IQ4_XS |
| 5-bit | Yellow | `#EAB308` | IQ5_K |
| 6-bit | Lime | `#84CC16` | IQ6_K, Q6_0, Q6_K |
| 7-bit | Green | `#22C55E` | IQ7_K |
| 8-bit | Teal | `#14B8A6` | Q8_0, Q8_K, IQ8_K, MXFP8 |
| 16-bit | Blue | `#3B82F6` | F16, BF16 |
| 32-bit | Purple | `#8B5CF6` | F32 |
| Unknown | Gray | `#6B7280` | - |

**Implementation:**
- Location: Bottom of icon-image div
- Height: 4px, border-radius: 0 0 8px 8px
- Extracted from `data-quantization` attribute
- Same bit-level = same color (e.g., Q4_K_M and Q4_0 both yellow-orange)

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
- `get_config()` â†’ `GlobalConfig`
- `save_config(models_dir, exec_folder, theme, bg, synced)` â†’ Result

### Models
- `scan_models_command()` â†’ `{success, models: [ModelInfo]}`
- `get_model_settings(model_path)` â†’ `ModelConfig`
- `update_model_settings(model_path, config)` â†’ Result
- `save_model_preset(model_path, preset)` â†’ Result
- `delete_model_preset(model_path, preset_id)` â†’ Result

### Process Management  
- `launch_model_internal(model_path, config, window_label)` â†’ `LaunchResult`
- `launch_model_external(model_path, config, window_label)` â†’ `LaunchResult`
- `stop_process(process_id)` â†’ Result
- `get_process_status(process_id)` â†’ `ProcessOutput`
- `kill_all_processes()` â†’ Result

### HuggingFace
- `search_huggingface_models(query, limit)` â†’ `SearchResult`
- `fetch_model_details(model_id)` â†’ `ModelDetails`
- `get_hf_suggestions()` â†’ `[{id, name, author}]`

### Downloads
- `start_download(url, filename, destination)` â†’ `DownloadStartResult`
- `start_llamacpp_download(url, filename, version, backend_type)` â†’ Result
- `pause_download(download_id)` â†’ Result
- `resume_download(download_id)` â†’ Result
- `cancel_download(download_id)` â†’ Result
- `get_download_status(download_id)` â†’ `DownloadStatus`
- `get_all_downloads()` â†’ `[DownloadStatus]`
- `clear_completed_downloads()` â†’ Result

### Llama.cpp Releases
- `get_llamacpp_releases()` â†’ `[LlamaCppReleaseFrontend]`
- `refresh_llamacpp_releases()` â†’ Result (force cache refresh)
- `fetch_commit_info(tag_name)` â†’ `CommitInfo`

### System
- `browse_folder(input_id)` â†’ Opens native folder dialog
- `get_app_version()` â†’ String
- `open_external_link(url)` â†’ Opens browser

---

## File Structure

> **ðŸ“ For complete file location reference:** Check knowledge base memory "Arandu Complete File Location Reference" 
> > This memory has exact paths for every file and quick reference by task (e.g., "right-click menu = desktop.js")

```
Arandu/
â”œâ”€â”€ backend/
â”‚   â”œâ”€â”€ Cargo.toml
â”‚   â”œâ”€â”€ tauri.conf.json        # Tauri config
â”‚   â”œâ”€â”€ build.rs
â”‚   â”œâ”€â”€ icons/
â”‚   â””â”€â”€ src/
â”‚       â”œâ”€â”€ main.rs
â”‚       â”œâ”€â”€ lib.rs             # Main entry + commands
â”‚       â”œâ”€â”€ models.rs          # All data structures
â”‚       â”œâ”€â”€ config.rs          # Config I/O
â”‚       â”œâ”€â”€ scanner.rs         # Model scanning
â”‚       â”œâ”€â”€ huggingface.rs     # HF API client
â”‚       â”œâ”€â”€ downloader.rs      # Download manager
â”‚       â”œâ”€â”€ llamacpp_manager.rs # GitHub releases
â”‚       â”œâ”€â”€ process.rs         # Process spawning
â”‚       â””â”€â”€ system_monitor.rs  # Hardware monitoring
â”œâ”€â”€ frontend/
â”‚   â”œâ”€â”€ index.html
â”‚   â”œâ”€â”€ desktop.js
â”‚   â”œâ”€â”€ css/                   # All stylesheets
â”‚   â”œâ”€â”€ assets/                # Logo, icons
â”‚   â”œâ”€â”€ modules/               # JS modules
â”‚   â””â”€â”€ *.json                 # Config/data files
â””â”€â”€ README.md
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

**âš ï¸ CRITICAL: Working Directory Mismatch:**
- **Problem:** Agent may work in wrong directory (e.g., C: drive instead of H: drive)
- **Impact:** Changes don't appear in user's actual project, builds succeed but don't update user's executable
- **Detection:** Check `pwd` command output vs expected project path
- **Solution:** 
  - ALWAYS verify working directory before making changes
  - Use absolute paths when copying files between locations
  - Confirm with user: "Current directory is X, should be Y?"
- **Prevention:** 
  - Add directory verification step at start of every session
  - Ask user to confirm project location before major changes
  - Use explicit path parameters in all file operations

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

## Recent Changes

> **For complete list of bug fixes and features:** See [THIS-PROJECTS-CURRENT-STATE.md](THIS-PROJECTS-CURRENT-STATE.md)

### 2026-02-24 - Model View Toggle + GGUF Suffix
- **feat:** Added view toggle for model display (icon grid vs vertical list)
- **feat:** List view shows models as horizontal bars sorted by size (largest first)
- **feat:** List view includes: quantization bar, model name, file path, size, quantization badge, update indicator
- **feat:** View preference saved to localStorage
- **fix:** CSS conflict causing architecture label to show in list view (duplicate rule)
- **feat:** Added "GGUF" suffix to model names in both icon and list view
- **docs:** Updated AGENTS.md with all changes

### 2026-02-24 - Qwen2.5 Speculative Models Downloaded
- **download:** Qwen2.5-14B-Instruct-Q4_K_M (8.99 GB) - merged from 3 split files
- **download:** Qwen2.5-0.5B-Instruct-Q4_K_M (0.49 GB) - draft model
- **location:** C:\Users\Gordprime\AppData\Roaming\Msty\models\blobs\Qwen2.5\
- **purpose:** Test speculative decoding (draft token) feature

### 2025-02-21 - Major Updates
- **feat:** Added "Open in File Explorer" right-click option for GGUF files
- **feat:** Added total disk space monitor (GB/TB) in top right corner
- **fix:** Search now queries HF API (not just local database)
- **fix:** All tracker filters working correctly
- **fix:** Models loading instantly when tracker opens
- **fix:** File counts updating properly on refresh

### 2025-02-20 - AI Model Tracker Hybrid Search
- **feat:** Hybrid local + live search with badge counts
- **feat:** Live search toggle with rate limiting
- **feat:** File type filters (GGUF, MLX, SafeTensors, etc.)
- **fix:** Tracker button and page loading issues

### 2025-02-18 - Phase 2: HuggingFace Direct Link Download âœ… COMPLETE
- **feat:** Tabbed HF interface with Paste Link functionality
- **feat:** URL parsing for all HF URL formats

---

## Known Issues

> **See [THIS-PROJECTS-CURRENT-STATE.md](THIS-PROJECTS-CURRENT-STATE.md) for current bug status**

---

## Known Issues

### Active (Not Fixed)

- **ðŸ›‘ Chat Tab Not Loading - REGRESSION (2025-02-23)**
  - **Status:** CRITICAL - REGRESSION FROM FIX ATTEMPT
  - **Issue:** Custom Chat tab no longer loads/renders after CSS layout fix attempt
  - **Location:** `frontend/modules/chat-app.js`, `frontend/css/chat-app.css`
  - **Symptoms:** Tab exists but clicking it shows nothing or breaks the interface
  - **Cause:** CSS changes to `.chat-app-wrapper` and `.chat-app-container` broke rendering
  - **Next Steps:** REVERT CSS changes, then apply fixes one at a time with testing
  - **See:** THIS-PROJECTS-CURRENT-STATE.md for full details and what to revert

- **âš ï¸ Chat Input Layout Incorrect (2025-02-23)**
  - **Status:** NOT FIXED - Superseded by regression above
  - **Issue:** Chat input area sits too high and does not show 4 visible lines of text
  - **Location:** `frontend/modules/chat-app.js`, `frontend/css/chat-app.css`
  - **Symptoms:** Input is not pinned to the bottom; message history visibility is reduced
  - **Root Cause:** Missing wrapper CSS, broken height chain, media query override
  - **Note:** Fix attempt caused regression - needs careful re-approach

### Resolved

- ~~**HF Search Model ID Display (2025-02-20)**~~ - **FIXED in commit `fe05539`**
  - Simple copy dialog now works when clicking "?" indicator
  - Copy button copies HF model ID to clipboard
  - Open HF Search button opens search with model pre-filled
  
- ~~**Commit `d7ecc6a`** (GGUF update checker)~~ - **FIXED**
  - ~~Application hangs on loading screen~~
  - **Solution:** Rebuilt feature on stable checkpoint `0df8e33` with proper testing
  - **Status:** Working implementation merged

## Features

### GGUF Update Checker
Monitors local GGUF models for updates on HuggingFace.

**How it works:**
1. **Auto-detection:** If model is in `models/author/model-name/file.gguf` structure, automatically extracts HF model ID
2. **Manual linking:** Link model using context menu or properties panel
3. **Update check:** Compares local file modification date with HF commit date

**Visual indicators:**
- **?** (gray): Not linked to HF - click to link or view model ID
- **âœ“** (green): Up to date
- **âœ—** (red): Update available on HF
- **!** (black/red): Error occurred
- **âŸ³** (spinning): Checking in progress

**Note:** Click the "?" indicator to see the linked HF model ID, copy it, or open HF search.

**Backend modules:**
- `gguf_parser.rs` - Parse GGUF binary format metadata
- `update_checker.rs` - HF API integration and comparison logic

**Tauri commands:**
- `get_model_metadata(path)` â†’ GgufMetadata
- `check_model_update(path)` â†’ UpdateCheckResult
- `link_model_to_hf(path, model_id, filename)` â†’ HfMetadata

---

## AI Model Tracker

Browse and track trending AI models from HuggingFace directly in the app.

**How it works:**
1. Click the Tracker button in the dock (robot icon)
2. Click "Refresh" or "Fetch Models" to load trending models from HuggingFace
3. Use filters to narrow down models by category, quantization, VRAM, etc.
4. View statistics about loaded models

**Features:**
- Trending models from HuggingFace API
- Filter by: category (text, image, video, audio, coding, multimodal), Chinese models only, GGUF only, VRAM limit, backends (CUDA, Vulkan, ROCm, CPU, Intel), quantization
- Sort by: downloads, likes, date, name, size
- Statistics panel showing total models, Chinese models, GGUF models, categories
- Export models to JSON

**Backend modules:**
- `tracker_scraper.rs` - HuggingFace API integration for trending models
- `tracker_manager.rs` - Local storage and filtering of tracker data

**Tauri commands:**
- `get_tracker_models(...)` â†’ Vec<TrackerModel>
- `refresh_tracker_data()` â†’ TrackerStats
- `get_tracker_stats()` â†’ TrackerStats
- `get_tracker_config()` â†’ TrackerConfig
- `save_tracker_config(config)` â†’ Result
- `export_tracker_models(models)` â†’ Result

---

## Skills Applied

- **superpowers:** Agentic development workflow framework
  - `brainstorming` - Design refinement before implementation
  - `writing-plans` - Detailed implementation planning
  - `test-driven-development` - RED-GREEN-REFACTOR cycle
  - `using-git-worktrees` - Isolated feature development
  - `requesting-code-review` - Code quality verification
- **vercel-react-best-practices:** Frontend performance patterns
- **plan:** Implementation planning

---

## Current Status

> **For complete current status, test results, and build information:** See [THIS-PROJECTS-CURRENT-STATE.md](THIS-PROJECTS-CURRENT-STATE.md)

### Quick Overview

**Last Build:** 2025-02-23 - âœ… SUCCESS (Release Build)  
**Version:** 0.5.5-beta  
**Location:** `H:\Ardanu Fix\Arandu-maxi\backend\target\release\Arandu.exe`

### âœ… Working Features
- AI Model Tracker with hybrid search (local + live HF)
- HuggingFace Direct Link Download (Phase 2 complete)
- "Open in File Explorer" right-click option
- Total disk space monitor (top right)
- GGUF Update Checker with visual indicators
- Multiple model directories support
- Quantization color bars on icons
- **Advanced Parameter Sidebar** in Chat UI
- **Restart Bridge** for hardware parameter updates

### âš ï¸ Active Bugs
- **ðŸ›‘ Parameter Panel Interaction Bugs (2025-02-23)**
  - Parameter panel starts visible and close button is unresponsive.
- **ðŸ›‘ Chat Input: Enter key does not send (2025-02-23)**
  - Enter key inserts newline instead of sending message.
- **ðŸ›‘ Chat UI: Send button unresponsive (2025-02-23)**
  - Send button fails to trigger LLM completion.

---

## Implementation Plans

Detailed implementation plans for upcoming features are stored in:

**`docs/plans/`** - Contains comprehensive step-by-step implementation guides

### Completed Plans:
- âœ… **[Phase 2: HuggingFace Direct Link Download](docs/plans/Phase-2-HF-Direct-Link.md)** - IMPLEMENTED
  - Sequential downloads with resume support

When starting new feature work, check this folder for ready-to-implement plans.

---

## Todo (from README)

- [ ] Cleanup code and organize it better
- [ ] New features (coming soon)

## Contact

- GitHub: https://github.com/fredconex/Arandu
- Releases: https://github.com/fredconex/Arandu/releases

---

## Final Reminder - Knowledge Mem Protocol (Read Every Time)

- Before searching for files, bugs, features, or implementation context, query nowledge mem first.
- During work, continue checking nowledge mem whenever new uncertainty appears.
- After every meaningful action (edit, move, save, discovery, fix, verification), add or update at least one nowledge memory.
- Treat memory updates as mandatory deliverables, not optional notes.

