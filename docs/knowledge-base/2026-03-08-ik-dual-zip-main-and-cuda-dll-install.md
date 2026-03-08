# ik_llama.cpp Dual ZIP Install (Main + CUDA DLL) — 2026-03-08

## What changed

- Added a two-button ik install flow in Llama.cpp Release Manager:
  - `ik_llama.cpp` (main ZIP installer)
  - `ik CUDA DLL` (DLL ZIP installer)
- CUDA main installs now prompt user to immediately install DLL ZIP.
- CPU main installs remain single-step (no DLL required).
- Installer now supports loose source layouts by accepting extracted folders as install source (not ZIP-only).
- Added dedicated managed workspace root: `<executable_folder>/versions/ik_llama.cpp/_installer_workspace`.

## Frontend

- File: `frontend/modules/llamacpp-manager.js`
  - Added `installIkLlamaCppMainZip()` (main archive flow).
  - Added `installIkLlamaCppCudaDllZip()` (DLL archive flow).
  - Added `pickIkSourcePath()` to select either ZIP source or extracted folder source via prompt + picker.
  - Added `getPreferredIkCudaInstallPath()` to resolve target backend path:
    - prefer last installed CUDA ik path,
    - else active CUDA ik backend,
    - else single CUDA ik backend,
    - else user prompt when multiple are installed.
  - Kept compatibility alias: `installIkLlamaCppFromZip()` forwards to main flow.
  - Added top-tab button markup for `ik CUDA DLL`.
  - Added visible `IK Installer Workspace` panel with `Copy` buttons for `main_drop` and `dll_drop` paths.

- File: `frontend/css/llama-manager.css`
  - Added style variants for `.top-tab-install-ik-dll` (green accent) consistent with existing top-tab language.
  - Added workspace panel styles (`.ik-workspace-panel`, rows, copy buttons).

## Backend

- File: `backend/src/lib.rs`
  - Added command `install_local_llamacpp_cuda_dlls_zip(zip_path, install_path)`.
  - Added helper `collect_files_with_extension(root, "dll")`.
  - Added helper `ensure_ik_installer_workspace(base_exec)`.
  - Main/DLL install commands now accept either `.zip` path or directory path as source.
  - DLL install behavior:
    1. validate selected ZIP,
    2. extract safely using existing path-traversal-safe extraction (`enclosed_name()`),
    3. find all `.dll` files recursively,
    4. copy DLLs into selected ik CUDA backend folder,
    5. return DLL count + install path.
  - Registered new command in Tauri `invoke_handler`.

## Verification

- `node --check frontend/modules/llamacpp-manager.js` ✅
- `node --check frontend/desktop.js` ✅
- `cargo check --manifest-path backend/Cargo.toml` ✅
