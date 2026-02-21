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
| `huggingface_downloader.rs` | HF Direct Link Download | `parse_model_id()`, `fetch_model_info()`, `fetch_model_files()` |
| `downloader.rs` | Download management | `DownloadManager`, pause/resume/extract |
| `llamacpp_manager.rs` | GitHub releases | `fetch_llamacpp_releases()`, release caching |
| `process.rs` | Process management | `launch_model_internal()`, `launch_model_external()`, `ProcessHandle` |
| `system_monitor.rs` | Hardware monitoring | `SystemMonitor`, RAM/VRAM tracking |
| `gguf_parser.rs` | GGUF metadata parsing | `parse_gguf_metadata()`, `get_file_modification_date()` |
| `update_checker.rs` | HF update checking | `check_huggingface_updates()`, `link_model_to_hf()`, `extract_hf_model_id_from_path()` |

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

**⚠️ CRITICAL: Working Directory Mismatch:**
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

### 2025-02-20 - HF Search Model ID Display ❌ BROKEN
- **attempt:** HF Search Model ID Display with Copy Button
- **intended behavior:** Click "?" indicator on model → HF search opens → Model ID bar appears at top with Copy button
- **current status:** NOT WORKING - clicking "?" appears to do nothing
- **attempts made:**
  - Added model ID bar HTML to HF search template
  - Created copy button with clipboard functionality
  - Added CSS styling for model ID bar (gradient background, copy button)
  - Modified `openHuggingFaceSearch(modelId)` to accept and display model ID
  - Fixed early return when model status is 'not_linked'
  - Fixed existing window detection to show model ID bar
  - Multiple rebuilds on H drive (`backend/target/release/`)
- **files modified:**
  - `frontend/desktop.js` - `handleCheckUpdate()` method
  - `frontend/modules/huggingface-app.js` - `openHuggingFaceSearch()` method
  - `frontend/css/huggingface.css` - added ~70 lines of model ID bar styles
- **commits:** `f4763f3`, `0c826f0`, `9d47de0`, `a699121`
- **issue:** Code appears correct in source, builds successfully, but runtime behavior doesn't match
- **see:** "Known Issues" section for detailed investigation notes

### 2025-02-18 - Phase 2: HuggingFace Direct Link Download ✅ COMPLETE
- **feat:** HuggingFace Direct Link Download feature fully implemented and tested
  - **Tabbed interface:** Search Models | Paste Link tabs in HuggingFace window
  - **Paste Link functionality:** Paste any HuggingFace model URL to download GGUF files
  - **URL parsing:** Supports multiple URL formats (full URLs, model IDs, blob/resolve URLs)
  - **Model info display:** Shows name, description, license, downloads, likes, tags
  - **File selection:** Checklist with quantization badges (Q4_K_M, Q8_0, etc.)
  - **Sequential downloads:** Downloads files one at a time with progress tracking
  - **Custom destinations:** Browse and select download location
  - **Flux/SD support:** Now finds all GGUF models including image generation (Flux, Stable Diffusion)
  - **Search fix:** Removed "conversational" filter from HF API search to include all GGUF model types
  - **New backend module:** `huggingface_downloader.rs` with URL parsing and HF API integration
  - **New Tauri commands:** `parse_hf_url`, `fetch_hf_model_info`, `fetch_hf_model_files`, `get_default_download_path`, `download_hf_file`
  - **Frontend updates:** `huggingface-app.js` and `huggingface.css` with tab UI and paste link interface

### 2025-02-15 - GGUF Update Checker
- **feat:** Add GGUF Update Checker feature
  - Parse GGUF metadata (architecture, name, quantization) from file headers
  - Three-tier HF tracking: explicit metadata, path extraction, manual linking
  - Visual update indicators: ✓ (green/up-to-date), ✗ (red/update available), ? (gray/not linked)
  - Click indicator to check for updates on HuggingFace
  - Right-click context menu option "Check for Updates"
  - Link dialog for manual HF model association
  - Compare local file modification date with HF commit date
  - Cache update check results for performance
  - New backend modules: `gguf_parser.rs`, `update_checker.rs`
  - New Tauri commands: `get_model_metadata`, `check_model_update`, `link_model_to_hf`

### 2025-02-15 - Working Baseline
- **checkpoint:** `0df8e33` - Working baseline before GGUF update checker
  - Stable version with multiple model directories and quantization bars

### 2025-02-14
- **feat:** Add support for multiple model directories
  - Primary directory + up to 2 additional directories
  - Automatic deduplication across all directories
  - Downloads still go to primary directory only
  - Full backward compatibility with existing configs
- **feat:** Add quantization color bars to GGUF icons (10 color levels by bit-depth)
- **fix:** Dock button clickability issues (z-index fix)
- **fix:** Icon font sizing to prevent oversized text rendering
- **docs:** Update README and AGENTS.md with accurate feature list

### 2025-02-12  
- **fix:** CSS z-index and pointer-events for dock items
- **docs:** Add AGENTS.md developer documentation
- **docs:** Clarify Windows-only platform support

---

## Known Issues

### Breaking - HF Search Model ID Display (2025-02-20)
**Status:** ❌ **BROKEN**
**Feature:** "?" indicator click to show HF model ID with copy button
**Build:** `a699121` and later
**Description:**
- Clicking the "?" update indicator on a model icon should:
  1. Check for updates
  2. Open HF search window
  3. Show model ID bar at top with Copy button
  4. Allow instant copy of HF model ID (e.g., `THUDM/glm-4-9b-chat`)

**Current Behavior:**
- Clicking "?" appears to do nothing
- No HF search window opens
- No model ID bar appears
- No visual feedback to user

**Attempts Made:**
1. Added model ID bar HTML to HF search window
2. Created copy button with clipboard functionality
3. Added CSS styling for model ID bar
4. Modified `openHuggingFaceSearch(modelId)` to accept model ID parameter
5. Fixed early return when model status is 'not_linked'
6. Fixed existing window detection to show model ID bar
7. Multiple rebuilds on correct H drive location

**Files Modified:**
- `frontend/desktop.js` - `handleCheckUpdate()` method (lines 4661-4744)
- `frontend/modules/huggingface-app.js` - `openHuggingFaceSearch()` method (lines 148-574)
- `frontend/css/huggingface.css` - Model ID bar styles (lines 10-80)

**Expected Code Locations:**
- Model ID bar HTML: `frontend/modules/huggingface-app.js` line 210
- CSS for bar: `frontend/css/huggingface.css` lines 10-80
- Copy handler: `frontend/modules/huggingface-app.js` lines 549-560
- Window open call: `frontend/desktop.js` line 4736

**Investigation Notes:**
- Code appears correct in source files on H drive
- Builds complete successfully without errors
- EXE timestamp updates per build
- FrontendDist path verified as `../frontend`
- Model ID bar HTML exists in template string
- CSS selectors correctly reference class names

**Status:** Requires deeper investigation into why runtime code differs from source

### Resolved
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
- **?** (gray): Not linked to HF - **⚠️ CLICKING THIS IS BROKEN** (see Known Issues)
- **✓** (green): Up to date
- **✗** (red): Update available on HF
- **!** (black/red): Error occurred
- **⟳** (spinning): Checking in progress

**Note:** The click-to-show-model-ID functionality for the "?" indicator is currently broken. See "Known Issues" section for details.

**Backend modules:**
- `gguf_parser.rs` - Parse GGUF binary format metadata
- `update_checker.rs` - HF API integration and comparison logic

**Tauri commands:**
- `get_model_metadata(path)` → GgufMetadata
- `check_model_update(path)` → UpdateCheckResult
- `link_model_to_hf(path, model_id, filename)` → HfMetadata

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

### ✅ Working Features
- Phase 2: HuggingFace Direct Link Download
  - ✅ Tabbed HF interface (Search Models | Paste Link)
  - ✅ URL parsing for all HF URL formats
  - ✅ Model info fetching (description, license, stats)
  - ✅ GGUF file listing with quantization badges
  - ✅ Sequential file downloads with progress
  - ✅ Custom destination folder selection
  - ✅ Support for ALL GGUF models (text, image, video generation)
  - ✅ Search now finds Flux, Stable Diffusion, and other image generation models

### ❌ Known Broken Feature
- **HF Search Model ID Display via "?" Indicator** (2025-02-20)
  - ❌ NOT WORKING - See "Known Issues" section above
  - Clicking "?" should show HF model ID with copy button
  - Currently has no visible effect

**Files Modified:**
- `backend/src/huggingface_downloader.rs` (NEW - 371 lines)
- `backend/src/huggingface.rs` (removed "conversational" filter)
- `backend/src/lib.rs` (added 5 new Tauri commands)
- `frontend/modules/huggingface-app.js` (tab UI + paste link logic)
- `frontend/css/huggingface.css` (tab styles + paste link UI)

**Build Location:** `H:\Ardanu Fix\Arandu-maxi\backend\target\release\Arandu.exe`

---

## Implementation Plans

Detailed implementation plans for upcoming features are stored in:

**`docs/plans/`** - Contains comprehensive step-by-step implementation guides

### Completed Plans:
- ✅ **[Phase 2: HuggingFace Direct Link Download](docs/plans/Phase-2-HF-Direct-Link.md)** - IMPLEMENTED
  - Sequential downloads with resume support

When starting new feature work, check this folder for ready-to-implement plans.

---

## Todo (from README)

- [ ] Cleanup code and organize it better
- [ ] New features (coming soon)

## Contact

- GitHub: https://github.com/fredconex/Arandu
- Releases: https://github.com/fredconex/Arandu/releases
