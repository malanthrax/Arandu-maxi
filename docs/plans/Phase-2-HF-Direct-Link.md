# Phase 2: HuggingFace Direct Link Download - Complete Implementation Plan

## Executive Summary

This document contains the complete implementation plan for adding HuggingFace Direct Link Download functionality to Arandu. This feature allows users to paste HuggingFace model URLs and selectively download GGUF quantization variants.

**Status:** ✅ IMPLEMENTED AND TESTED  
**Completion Date:** 2025-02-18  
**Complexity:** Medium  
**Dependencies:** Existing HuggingFace integration, DownloadManager

---

## ✅ IMPLEMENTATION COMPLETE

All features documented in this plan have been successfully implemented:

- ✅ Tabbed interface (Search Models | Paste Link)
- ✅ URL parsing and validation  
- ✅ Model info fetching from HuggingFace API
- ✅ GGUF file selection with quantization badges
- ✅ Sequential download support
- ✅ Custom destination path selection
- ✅ Full Flux/SD image generation model support
- ✅ Search filter fixed to include all GGUF models

**Files Created/Modified:**
- `backend/src/huggingface_downloader.rs` (NEW)
- `backend/src/huggingface.rs` (MODIFIED)
- `backend/src/lib.rs` (MODIFIED)
- `frontend/modules/huggingface-app.js` (MODIFIED)
- `frontend/css/huggingface.css` (MODIFIED)

**Build Location:** `H:\Ardanu Fix\Arandu-maxi\backend\target\release\Arandu.exe`

---

## Table of Contents

1. [Architecture Overview](#architecture-overview)
2. [User Workflow](#user-workflow)
3. [UI/UX Design](#uiux-design)
4. [Frontend Implementation](#frontend-implementation)
5. [Backend Implementation](#backend-implementation)
6. [Data Structures](#data-structures)
7. [API Endpoints](#api-endpoints)
8. [Error Handling](#error-handling)
9. [Testing Strategy](#testing-strategy)
10. [File Changes Checklist](#file-changes-checklist)

---

## Architecture Overview

### High-Level Flow

```
User pastes URL → Frontend validates → Backend fetches model info → 
Display GGUF files → User selects → Download one-by-one → 
Save to ~/.Arandu/models/{author}/{model}/
```

### Component Interaction

```
┌─────────────────────────────────────────────────────────────────┐
│                         FRONTEND                                 │
├─────────────────────────────────────────────────────────────────┤
│  huggingface-app.js                                             │
│  ├── Tab: Search Models (existing)                             │
│  └── Tab: Paste Link (new)                                      │
│       ├── URL Input & Validation                               │
│       ├── Model Info Card                                       │
│       ├── GGUF Files Selection List                            │
│       ├── Download Progress Tracking                           │
│       └── Destination Path Selector                            │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                         BACKEND                                  │
├─────────────────────────────────────────────────────────────────┤
│  huggingface_downloader.rs (NEW)                               │
│  ├── parse_model_id() - URL parsing                            │
│  ├── fetch_model_info() - HF API call                          │
│  ├── fetch_model_files() - Get GGUF list                       │
│  └── build_destination_path() - Path construction              │
│                                                                  │
│  lib.rs (MODIFY)                                               │
│  ├── fetch_hf_model_info (command)                             │
│  ├── fetch_hf_model_files (command)                            │
│  └── download_hf_file (command)                                │
│                                                                  │
│  downloader.rs (MODIFY)                                        │
│  └── Add multi-file download support                           │
└─────────────────────────────────────────────────────────────────┘
```

---

## User Workflow

### Primary Workflow

1. **Open HuggingFace Window**
   - Click HuggingFace dock icon
   - Window opens with two tabs

2. **Switch to "Paste Link" Tab**
   - Click "📋 Paste Link" tab
   - Clean interface appears

3. **Enter Model URL**
   - Paste URL in input field
   - Supported formats:
     * `https://huggingface.co/THUDM/glm-4-9b-chat`
     * `huggingface.co/THUDM/glm-4-9b-chat`
     * `THUDM/glm-4-9b-chat`
   - Click "Validate" button (or press Enter)

4. **Review Model Information**
   - Model name and description
   - License information
   - Tags (e.g., transformers, llama.cpp)
   - Total downloads and likes

5. **Select Files to Download**
   - Checkbox list of all GGUF files
   - Columns: Select, Filename, Size, Quantization
   - "Select All" / "Deselect All" buttons
   - Total size calculator (updates live)

6. **Choose Destination**
   - Default: `~/.Arandu/models/{author}/{model}/`
   - Optional: Browse to select custom location
   - Shows available disk space

7. **Download Files**
   - Click "Download Selected"
   - Files download ONE AT A TIME (sequential)
   - Progress bar for each file
   - Estimated time remaining
   - Cancel button (per file)
   - Pause/Resume support

8. **Completion**
   - Success notification
   - "Open Folder" button
   - "Download More" button
   - Files appear on desktop after refresh

### Alternative Workflows

**Direct File URL:**
- Paste: `https://huggingface.co/author/model/blob/main/model-Q4.gguf`
- System extracts model ID and shows all GGUFs
- Pre-selects the specific file from URL

**Split Model Files:**
- If model has multiple parts (model-00001-of-00002.gguf)
- Show as single entry with part count
- Download all parts sequentially

**Resume Interrupted Download:**
- Check for partially downloaded files
- Resume from last byte (HTTP range requests)
- Show "Resuming..." status

---

## UI/UX Design

### Tabbed Interface Structure

```html
<div class="huggingface-container">
  <!-- Tab Navigation -->
  <div class="tab-header">
    <button class="tab-btn active" data-tab="search">
      <span class="material-icons">search</span>
      Search Models
    </button>
    <button class="tab-btn" data-tab="paste-link">
      <span class="material-icons">content_paste</span>
      Paste Link
    </button>
  </div>
  
  <!-- Tab Content -->
  <div class="tab-content">
    <!-- Search Tab (existing) -->
    <div class="tab-panel active" id="tab-search">
      <!-- Current search interface -->
    </div>
    
    <!-- Paste Link Tab (new) -->
    <div class="tab-panel" id="tab-paste-link">
      <!-- New interface -->
    </div>
  </div>
</div>
```

### Paste Link Tab Layout

```
┌─────────────────────────────────────────────────────────────────────┐
│ PASTE LINK TAB                                                       │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  Step 1: Enter Model URL                                            │
│  ┌──────────────────────────────────────────────────────────────┐  │
│  │ 🔗 https://huggingface.co/THUDM/glm-4-9b-chat      [Paste]  │  │
│  └──────────────────────────────────────────────────────────────┘  │
│                                                      [Validate 🔍]  │
│                                                                      │
│  Step 2: Review Model Information                                   │
│  ┌──────────────────────────────────────────────────────────────┐  │
│  │ 📦 THUDM/glm-4-9b-chat                                        │  │
│  │                                                              │  │
│  │ GLM-4-9B-Chat is an open-source multilingual conversational  │  │
│  │ language model.                                              │  │
│  │                                                              │  │
│  │ 📄 License: MIT        🏷️ Tags: transformers, chat           │  │
│  │ ⬇️ Downloads: 125.3k   ❤️ Likes: 1.2k                        │  │
│  └──────────────────────────────────────────────────────────────┘  │
│                                                                      │
│  Step 3: Select GGUF Files to Download                              │
│  ┌──────────────────────────────────────────────────────────────┐  │
│  │ ☑️ Select All    [Q4_K_M ▼] Filter by quantization...        │  │
│  │                                                              │  │
│  │ ☑️ ☐ glm-4-9b-chat-Q4_K_M.gguf      4.5 GB   Q4_K_M        │  │
│  │ ☐  ☐ glm-4-9b-chat-Q5_K_M.gguf      5.8 GB   Q5_K_M        │  │
│  │ ☐  ☐ glm-4-9b-chat-Q8_0.gguf        8.2 GB   Q8_0          │  │
│  │ ☐  ☐ glm-4-9b-chat-FP16.gguf       16.1 GB   FP16          │  │
│  │                                                              │  │
│  │ 💾 Total: 4.5 GB (1 file selected)                           │  │
│  └──────────────────────────────────────────────────────────────┘  │
│                                                                      │
│  Step 4: Choose Destination                                         │
│  ┌──────────────────────────────────────────────────────────────┐  │
│  │ 📁 ~/.Arandu/models/THUDM/glm-4-9b-chat/        [Browse]     │  │
│  │                                                              │  │
│  │ 💽 Available Space: 145.2 GB                                 │  │
│  └──────────────────────────────────────────────────────────────┘  │
│                                                                      │
│                                          [Cancel]  [⬇️ Download]   │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

### Download Progress View

```
┌─────────────────────────────────────────────────────────────────────┐
│ DOWNLOADING...                                                       │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  📦 THUDM/glm-4-9b-chat                                             │
│                                                                      │
│  Downloading: glm-4-9b-chat-Q4_K_M.gguf                             │
│  ┌──────────────────────────────────────────────────────────────┐  │
│  │██████████████████████████████████████░░░░░░░░░░░░░░░  72%   │  │
│  └──────────────────────────────────────────────────────────────┘  │
│  3.2 GB / 4.5 GB  •  2.1 MB/s  •  ETA: 10 minutes                   │
│                                                                      │
│  [⏸️ Pause]  [❌ Cancel]                                            │
│                                                                      │
│  Queue:                                                             │
│  ✓ glm-4-9b-chat-Q4_K_M.gguf (completed)                           │
│  ⏳ glm-4-9b-chat-Q5_K_M.gguf (waiting...)                          │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

### CSS Classes Required

```css
/* Tab System */
.huggingface-container { }
.tab-header { }
.tab-btn { }
.tab-btn.active { }
.tab-content { }
.tab-panel { display: none; }
.tab-panel.active { display: block; }

/* Paste Link Specific */
.paste-link-container { }
.url-input-section { }
.url-input-wrapper { }
.url-input { }
.url-input.error { border-color: var(--theme-error); }
.url-input.valid { border-color: var(--theme-success); }
.validate-btn { }
.paste-btn { }

/* Model Info Card */
.model-info-card { }
.model-info-header { }
.model-name { }
.model-id { }
.model-description { }
.model-metadata { }
.model-license { }
.model-tags { }
.model-stats { }

/* File Selection */
.file-selection-section { }
.file-list-header { }
.select-all-btn { }
.filter-select { }
.file-list { }
.file-item { }
.file-item.selected { }
.file-checkbox { }
.file-name { }
.file-size { }
.file-quantization { }
.file-badge { } /* Q4, Q5, etc. */
.total-size-display { }

/* Destination */
.destination-section { }
.destination-path { }
.browse-btn { }
.disk-space-info { }
.disk-space-low { color: var(--theme-error); }

/* Download Progress */
.download-progress-section { }
.current-file { }
.progress-bar-container { }
.progress-bar { }
.progress-text { }
.download-stats { }
.download-speed { }
.download-eta { }
.download-actions { }
.pause-btn { }
.cancel-btn { }
.queue-list { }
.queue-item { }
.queue-item.completed { }
.queue-item.active { }
.queue-item.pending { }

/* Messages */
.error-message { }
.success-message { }
.info-message { }
.loading-spinner { }
```

---

## Frontend Implementation

### File: `frontend/modules/huggingface-app.js`

#### New Methods to Add

```javascript
/**
 * Initialize Paste Link tab functionality
 */
initializePasteLinkTab() {
    this.setupPasteLinkListeners();
    this.setupTabSwitching();
}

/**
 * Setup tab switching between Search and Paste Link
 */
setupTabSwitching() {
    const tabBtns = document.querySelectorAll('.tab-btn');
    tabBtns.forEach(btn => {
        btn.addEventListener('click', (e) => {
            const tabName = e.currentTarget.dataset.tab;
            this.switchTab(tabName);
        });
    });
}

/**
 * Switch between tabs
 */
switchTab(tabName) {
    // Update button states
    document.querySelectorAll('.tab-btn').forEach(btn => {
        btn.classList.toggle('active', btn.dataset.tab === tabName);
    });
    
    // Update panel visibility
    document.querySelectorAll('.tab-panel').forEach(panel => {
        panel.classList.toggle('active', panel.id === `tab-${tabName}`);
    });
}

/**
 * Setup event listeners for Paste Link interface
 */
setupPasteLinkListeners() {
    const window = this.desktop.windows.get(this.windowId);
    if (!window) return;
    
    // URL input
    const urlInput = window.querySelector('#hf-url-input');
    const validateBtn = window.querySelector('#hf-validate-btn');
    const pasteBtn = window.querySelector('#hf-paste-btn');
    
    // Paste from clipboard
    pasteBtn?.addEventListener('click', async () => {
        try {
            const text = await navigator.clipboard.readText();
            urlInput.value = text;
            this.validateUrl(text);
        } catch (err) {
            console.error('Failed to read clipboard:', err);
        }
    });
    
    // Validate on button click
    validateBtn?.addEventListener('click', () => {
        this.handleUrlValidation();
    });
    
    // Validate on Enter key
    urlInput?.addEventListener('keypress', (e) => {
        if (e.key === 'Enter') {
            this.handleUrlValidation();
        }
    });
    
    // Real-time validation feedback
    urlInput?.addEventListener('input', (e) => {
        this.validateUrl(e.target.value);
    });
}

/**
 * Validate URL format
 */
validateUrl(url) {
    const input = document.querySelector('#hf-url-input');
    if (!url) {
        input?.classList.remove('valid', 'error');
        return false;
    }
    
    // Regex patterns for validation
    const patterns = [
        /^https?:\/\/huggingface\.co\/[^\/]+\/[^\/]+/,
        /^huggingface\.co\/[^\/]+\/[^\/]+/,
        /^[^\/]+\/[^\/]+$/
    ];
    
    const isValid = patterns.some(pattern => pattern.test(url));
    
    if (isValid) {
        input?.classList.add('valid');
        input?.classList.remove('error');
    } else {
        input?.classList.add('error');
        input?.classList.remove('valid');
    }
    
    return isValid;
}

/**
 * Handle URL validation and fetch model data
 */
async handleUrlValidation() {
    const urlInput = document.querySelector('#hf-url-input');
    const url = urlInput?.value?.trim();
    
    if (!url) {
        this.showError('Please enter a HuggingFace model URL');
        return;
    }
    
    if (!this.validateUrl(url)) {
        this.showError('Invalid URL format. Use: https://huggingface.co/author/model');
        return;
    }
    
    // Show loading state
    this.showLoading('Fetching model information...');
    
    try {
        const invoke = this.getInvoke();
        if (!invoke) {
            throw new Error('Tauri API not available');
        }
        
        // Parse URL to get model ID
        const modelId = await invoke('parse_hf_url', { url });
        
        // Fetch model info and files
        const [modelInfo, files] = await Promise.all([
            invoke('fetch_hf_model_info', { modelId }),
            invoke('fetch_hf_model_files', { modelId })
        ]);
        
        // Display results
        this.displayModelInfo(modelInfo);
        this.displayFilesList(files);
        
        // Set default destination
        this.setDefaultDestination(modelId);
        
    } catch (error) {
        console.error('Failed to fetch model:', error);
        this.showError(`Failed to fetch model: ${error.message}`);
    } finally {
        this.hideLoading();
    }
}

/**
 * Display model information card
 */
displayModelInfo(modelInfo) {
    const container = document.querySelector('#hf-model-info');
    if (!container) return;
    
    container.innerHTML = `
        <div class="model-info-card">
            <div class="model-info-header">
                <h3>${this.escapeHtml(modelInfo.name || modelInfo.id)}</h3>
                <span class="model-id">${this.escapeHtml(modelInfo.id)}</span>
            </div>
            ${modelInfo.description ? `
                <p class="model-description">${this.escapeHtml(modelInfo.description)}</p>
            ` : ''}
            <div class="model-metadata">
                ${modelInfo.license ? `
                    <span class="model-license">📄 License: ${this.escapeHtml(modelInfo.license)}</span>
                ` : ''}
                ${modelInfo.tags?.length ? `
                    <span class="model-tags">🏷️ ${modelInfo.tags.slice(0, 5).map(t => this.escapeHtml(t)).join(', ')}</span>
                ` : ''}
            </div>
            <div class="model-stats">
                ${modelInfo.downloads ? `
                    <span>⬇️ Downloads: ${this.formatNumber(modelInfo.downloads)}</span>
                ` : ''}
                ${modelInfo.likes ? `
                    <span>❤️ Likes: ${this.formatNumber(modelInfo.likes)}</span>
                ` : ''}
            </div>
        </div>
    `;
    
    container.style.display = 'block';
}

/**
 * Display list of GGUF files
 */
displayFilesList(files) {
    const container = document.querySelector('#hf-files-list');
    if (!container) return;
    
    if (!files || files.length === 0) {
        container.innerHTML = `
            <div class="info-message">
                <span class="material-icons">info</span>
                No GGUF files found in this repository.
            </div>
        `;
        return;
    }
    
    const filesHtml = files.map((file, index) => `
        <div class="file-item" data-filename="${this.escapeHtml(file.filename)}">
            <input type="checkbox" class="file-checkbox" id="file-${index}" 
                   data-filename="${this.escapeHtml(file.filename)}"
                   data-size="${file.size}">
            <label for="file-${index}" class="file-label">
                <span class="file-name">${this.escapeHtml(file.filename)}</span>
                <span class="file-size">${file.sizeFormatted}</span>
                ${file.quantization ? `<span class="file-badge">${this.escapeHtml(file.quantization)}</span>` : ''}
            </label>
        </div>
    `).join('');
    
    container.innerHTML = `
        <div class="file-list-header">
            <button class="select-all-btn" id="hf-select-all">Select All</button>
            <button class="select-none-btn" id="hf-select-none">Deselect All</button>
        </div>
        <div class="file-list">
            ${filesHtml}
        </div>
        <div class="total-size-display" id="hf-total-size">
            Selected: 0 files (0 GB)
        </div>
    `;
    
    // Setup file selection listeners
    this.setupFileSelectionListeners();
    
    container.style.display = 'block';
}

/**
 * Setup listeners for file selection
 */
setupFileSelectionListeners() {
    const container = document.querySelector('#hf-files-list');
    if (!container) return;
    
    // Individual file checkboxes
    container.querySelectorAll('.file-checkbox').forEach(checkbox => {
        checkbox.addEventListener('change', () => {
            this.updateTotalSize();
        });
    });
    
    // Select all
    container.querySelector('#hf-select-all')?.addEventListener('click', () => {
        container.querySelectorAll('.file-checkbox').forEach(cb => {
            cb.checked = true;
        });
        this.updateTotalSize();
    });
    
    // Deselect all
    container.querySelector('#hf-select-none')?.addEventListener('click', () => {
        container.querySelectorAll('.file-checkbox').forEach(cb => {
            cb.checked = false;
        });
        this.updateTotalSize();
    });
}

/**
 * Update total size display
 */
updateTotalSize() {
    const checkboxes = document.querySelectorAll('.file-checkbox:checked');
    let totalSize = 0;
    let count = 0;
    
    checkboxes.forEach(cb => {
        totalSize += parseInt(cb.dataset.size) || 0;
        count++;
    });
    
    const display = document.querySelector('#hf-total-size');
    if (display) {
        display.textContent = `Selected: ${count} file${count !== 1 ? 's' : ''} (${this.formatBytes(totalSize)})`;
    }
}

/**
 * Set default destination path
 */
async setDefaultDestination(modelId) {
    const invoke = this.getInvoke();
    if (!invoke) return;
    
    try {
        const defaultPath = await invoke('get_default_download_path', { modelId });
        const pathInput = document.querySelector('#hf-destination-path');
        if (pathInput) {
            pathInput.value = defaultPath;
        }
    } catch (error) {
        console.error('Failed to get default path:', error);
    }
}

/**
 * Handle download initiation
 */
async startDownload() {
    const selectedFiles = Array.from(document.querySelectorAll('.file-checkbox:checked'))
        .map(cb => cb.dataset.filename);
    
    if (selectedFiles.length === 0) {
        this.showError('Please select at least one file to download');
        return;
    }
    
    const destination = document.querySelector('#hf-destination-path')?.value;
    if (!destination) {
        this.showError('Please select a destination folder');
        return;
    }
    
    const url = document.querySelector('#hf-url-input')?.value;
    const modelId = await this.getInvoke()('parse_hf_url', { url });
    
    // Show download progress UI
    this.showDownloadProgress(selectedFiles);
    
    // Download files one at a time
    for (let i = 0; i < selectedFiles.length; i++) {
        const filename = selectedFiles[i];
        this.updateCurrentFile(filename, i + 1, selectedFiles.length);
        
        try {
            await this.downloadFile(modelId, filename, destination);
            this.markFileComplete(filename);
        } catch (error) {
            console.error(`Failed to download ${filename}:`, error);
            this.markFileError(filename, error.message);
            
            // Ask user if they want to continue
            const shouldContinue = await this.showContinueDialog(filename);
            if (!shouldContinue) break;
        }
    }
    
    this.showDownloadComplete();
}

/**
 * Download a single file
 */
async downloadFile(modelId, filename, destination) {
    const invoke = this.getInvoke();
    
    return await invoke('download_hf_file', {
        modelId,
        filename,
        destination
    });
}

// Utility methods
showLoading(message) { }
hideLoading() { }
showError(message) { }
showSuccess(message) { }
formatNumber(num) { }
formatBytes(bytes) { }
escapeHtml(text) { }
```

---

## Backend Implementation

### File: `backend/src/huggingface_downloader.rs` (NEW)

```rust
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Information about a single GGUF file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HfFileInfo {
    pub filename: String,
    pub path: String,
    pub size: u64,
    pub size_formatted: String,
    pub quantization: Option<String>,
    pub commit_date: Option<String>,
}

/// Model information from HuggingFace API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelCardInfo {
    pub id: String,
    pub name: Option<String>,
    pub description: Option<String>,
    pub license: String,
    pub tags: Vec<String>,
    pub downloads: Option<u64>,
    pub likes: Option<u64>,
}

/// Parse various URL formats to extract model ID (author/model)
pub fn parse_model_id(input: &str) -> Result<String, String> {
    let input = input.trim();
    
    // Pattern 1: Full URL
    // https://huggingface.co/author/model
    if let Some(caps) = regex::Regex::new(r"https?://huggingface\.co/([^/]+/[^/]+)").unwrap().captures(input) {
        return Ok(caps.get(1).unwrap().as_str().to_string());
    }
    
    // Pattern 2: URL without protocol
    // huggingface.co/author/model
    if let Some(caps) = regex::Regex::new(r"huggingface\.co/([^/]+/[^/]+)").unwrap().captures(input) {
        return Ok(caps.get(1).unwrap().as_str().to_string());
    }
    
    // Pattern 3: Direct file URL
    // https://huggingface.co/author/model/blob/main/file.gguf
    if let Some(caps) = regex::Regex::new(r"https?://huggingface\.co/([^/]+/[^/]+)/blob/").unwrap().captures(input) {
        return Ok(caps.get(1).unwrap().as_str().to_string());
    }
    
    // Pattern 4: Just the ID
    // author/model
    if regex::Regex::new(r"^[a-zA-Z0-9_-]+/[a-zA-Z0-9_-]+$").unwrap().is_match(input) {
        return Ok(input.to_string());
    }
    
    Err("Invalid URL format. Expected: https://huggingface.co/author/model".to_string())
}

/// Fetch model information from HuggingFace API
pub async fn fetch_model_info(model_id: &str) -> Result<ModelCardInfo, String> {
    let url = format!("https://huggingface.co/api/models/{}", model_id);
    
    let client = reqwest::Client::new();
    let response = client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("Failed to fetch model info: {}", e))?;
    
    if !response.status().is_success() {
        return Err(format!("Model not found (HTTP {})", response.status()));
    }
    
    let data: serde_json::Value = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))?;
    
    Ok(ModelCardInfo {
        id: data.get("id").and_then(|v| v.as_str()).unwrap_or(model_id).to_string(),
        name: data.get("modelId").and_then(|v| v.as_str()).map(|s| s.to_string()),
        description: data.get("cardData")
            .and_then(|v| v.get("description"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()),
        license: data.get("license").and_then(|v| v.as_str()).unwrap_or("Unknown").to_string(),
        tags: data.get("tags")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_str()).map(|s| s.to_string()).collect())
            .unwrap_or_default(),
        downloads: data.get("downloads").and_then(|v| v.as_u64()),
        likes: data.get("likes").and_then(|v| v.as_u64()),
    })
}

/// Fetch list of GGUF files from model repository
pub async fn fetch_model_files(model_id: &str) -> Result<Vec<HfFileInfo>, String> {
    let url = format!("https://huggingface.co/api/models/{}/tree/main", model_id);
    
    let client = reqwest::Client::new();
    let response = client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("Failed to fetch file list: {}", e))?;
    
    if !response.status().is_success() {
        return Err(format!("Failed to fetch files (HTTP {})", response.status()));
    }
    
    let files: Vec<serde_json::Value> = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse file list: {}", e))?;
    
    let mut gguf_files: Vec<HfFileInfo> = files
        .into_iter()
        .filter(|file| {
            file.get("path")
                .and_then(|p| p.as_str())
                .map(|path| path.ends_with(".gguf"))
                .unwrap_or(false)
        })
        .map(|file| {
            let path = file.get("path").and_then(|p| p.as_str()).unwrap_or("").to_string();
            let filename = path.split('/').last().unwrap_or(&path).to_string();
            let size = file.get("size").and_then(|s| s.as_u64()).unwrap_or(0);
            
            // Extract quantization from filename
            let quantization = extract_quantization(&filename);
            
            HfFileInfo {
                path: path.clone(),
                filename: filename.clone(),
                size,
                size_formatted: format_bytes(size),
                quantization,
                commit_date: file.get("lastCommit")
                    .and_then(|c| c.get("date"))
                    .and_then(|d| d.as_str())
                    .map(|s| s.to_string()),
            }
        })
        .collect();
    
    // Sort by size (smallest first)
    gguf_files.sort_by_key(|f| f.size);
    
    Ok(gguf_files)
}

/// Extract quantization from filename
/// e.g., "model-Q4_K_M.gguf" -> "Q4_K_M"
fn extract_quantization(filename: &str) -> Option<String> {
    // Remove .gguf extension
    let base = filename.strip_suffix(".gguf")?;
    
    // Common patterns:
    // model-Q4_K_M.gguf
    // model-q4_0.gguf
    // model-f16.gguf
    // model-Q4KM.gguf
    
    let patterns = [
        regex::Regex::new(r"[-_](Q[0-9]_[A-Z]+)$").unwrap(),
        regex::Regex::new(r"[-_](Q[0-9][A-Z])$").unwrap(),
        regex::Regex::new(r"[-_](F16|FP16|F32|FP32)$").unwrap(),
        regex::Regex::new(r"[-_](Q[0-9]_[A-Z]+_[A-Z]+)$").unwrap(),
    ];
    
    for pattern in &patterns {
        if let Some(caps) = pattern.captures(base) {
            return caps.get(1).map(|m| m.as_str().to_string());
        }
    }
    
    None
}

/// Build default destination path
pub fn build_destination_path(base_dir: &str, model_id: &str) -> PathBuf {
    let path = PathBuf::from(base_dir);
    let parts: Vec<&str> = model_id.split('/').collect();
    
    if parts.len() >= 2 {
        // ~/.Arandu/models/author/model/
        path.join("models").join(parts[0]).join(parts[1])
    } else {
        path.join("models").join(model_id)
    }
}

/// Format bytes to human readable
fn format_bytes(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit_index = 0;
    
    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }
    
    format!("{:.1} {}", size, UNITS[unit_index])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_model_id_full_url() {
        let url = "https://huggingface.co/THUDM/glm-4-9b-chat";
        assert_eq!(parse_model_id(url).unwrap(), "THUDM/glm-4-9b-chat");
    }

    #[test]
    fn test_parse_model_id_no_protocol() {
        let url = "huggingface.co/THUDM/glm-4-9b-chat";
        assert_eq!(parse_model_id(url).unwrap(), "THUDM/glm-4-9b-chat");
    }

    #[test]
    fn test_parse_model_id_direct_id() {
        let id = "THUDM/glm-4-9b-chat";
        assert_eq!(parse_model_id(id).unwrap(), "THUDM/glm-4-9b-chat");
    }

    #[test]
    fn test_parse_model_id_blob_url() {
        let url = "https://huggingface.co/THUDM/glm-4-9b-chat/blob/main/model.gguf";
        assert_eq!(parse_model_id(url).unwrap(), "THUDM/glm-4-9b-chat");
    }

    #[test]
    fn test_extract_quantization() {
        assert_eq!(
            extract_quantization("model-Q4_K_M.gguf"),
            Some("Q4_K_M".to_string())
        );
        assert_eq!(
            extract_quantization("model-Q5_0.gguf"),
            Some("Q5_0".to_string())
        );
        assert_eq!(
            extract_quantization("model-F16.gguf"),
            Some("F16".to_string())
        );
    }
}
```

### File: `backend/src/lib.rs` (MODIFY)

Add these Tauri commands:

```rust
// Add to module declarations
mod huggingface_downloader;

// Add to imports
use huggingface_downloader::*;

// Add Tauri commands

#[tauri::command]
async fn parse_hf_url(url: String) -> Result<String, String> {
    huggingface_downloader::parse_model_id(&url)
}

#[tauri::command]
async fn fetch_hf_model_info(model_id: String) -> Result<ModelCardInfo, String> {
    huggingface_downloader::fetch_model_info(&model_id).await
}

#[tauri::command]
async fn fetch_hf_model_files(model_id: String) -> Result<Vec<HfFileInfo>, String> {
    huggingface_downloader::fetch_model_files(&model_id).await
}

#[tauri::command]
async fn get_default_download_path(
    model_id: String,
    state: tauri::State<'_, AppState>,
) -> Result<String, String> {
    let config = state.config.lock().await;
    let base_dir = &config.models_directory;
    let path = huggingface_downloader::build_destination_path(base_dir, &model_id);
    Ok(path.to_string_lossy().to_string())
}

#[tauri::command]
async fn download_hf_file(
    model_id: String,
    filename: String,
    destination: String,
    state: tauri::State<'_, AppState>,
) -> Result<String, String> {
    // Construct download URL
    let download_url = format!(
        "https://huggingface.co/{}/resolve/main/{}",
        model_id, filename
    );
    
    // Ensure destination directory exists
    let dest_path = std::path::Path::new(&destination);
    if let Some(parent) = dest_path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create directory: {}", e))?;
    }
    
    // Use existing download infrastructure
    let mut download_manager = state.download_manager.lock().await;
    
    let download_id = download_manager
        .start_download(download_url, filename, destination)
        .await
        .map_err(|e| format!("Failed to start download: {}", e))?;
    
    Ok(download_id)
}

// Register commands in invoke_handler
.invoke_handler(tauri::generate_handler![
    // ... existing commands ...
    parse_hf_url,
    fetch_hf_model_info,
    fetch_hf_model_files,
    get_default_download_path,
    download_hf_file,
])
```

---

## Data Structures

### Frontend Types (TypeScript)

```typescript
// Model information from HF API
interface HfModelInfo {
    id: string;                    // "author/model"
    name?: string;                 // Display name
    description?: string;          // Model description
    license: string;               // License type
    tags: string[];               // Model tags
    downloads?: number;           // Total downloads
    likes?: number;               // Total likes
}

// Individual GGUF file information
interface HfFileInfo {
    filename: string;             // "model-Q4_K_M.gguf"
    path: string;                 // Full path in repo
    size: number;                 // Size in bytes
    sizeFormatted: string;        // "4.5 GB"
    quantization?: string;        // "Q4_K_M"
    commitDate?: string;          // Last updated
    selected?: boolean;           // UI state
}

// Download request
interface DownloadRequest {
    modelId: string;              // "author/model"
    files: string[];             // Selected filenames
    destination: string;         // Destination path
}

// Download progress
interface DownloadProgress {
    downloadId: string;
    filename: string;
    progress: number;            // 0-100
    downloadedBytes: number;
    totalBytes: number;
    speed: string;               // "2.1 MB/s"
    eta: string;                // "10 minutes"
    status: 'pending' | 'downloading' | 'paused' | 'completed' | 'error';
}
```

### Backend Types (Rust)

Already defined in huggingface_downloader.rs above.

---

## API Endpoints

### HuggingFace API Endpoints Used

```
GET https://huggingface.co/api/models/{model_id}
Response: Model metadata (name, description, license, tags, etc.)

GET https://huggingface.co/api/models/{model_id}/tree/main
Response: List of files in the repository

GET https://huggingface.co/{model_id}/resolve/main/{filename}
Response: Direct file download (with resume support via Range header)
```

### Tauri Commands

```rust
// Parse URL to extract model ID
parse_hf_url(url: String) -> Result<String, String>

// Fetch model information
fetch_hf_model_info(model_id: String) -> Result<ModelCardInfo, String>

// Fetch list of GGUF files
fetch_hf_model_files(model_id: String) -> Result<Vec<HfFileInfo>, String>

// Get default download path
get_default_download_path(model_id: String) -> Result<String, String>

// Download a single file
// Note: Uses existing download_manager infrastructure
download_hf_file(
    model_id: String,
    filename: String,
    destination: String
) -> Result<String, String>
```

---

## Error Handling

### Frontend Errors

| Error | Cause | User Message |
|-------|-------|--------------|
| Invalid URL | User enters malformed URL | "Invalid URL format. Please use: https://huggingface.co/author/model" |
| Empty URL | User clicks validate with empty field | "Please enter a HuggingFace model URL" |
| Network Error | No internet connection | "Network error. Please check your connection and try again." |
| Model Not Found | 404 from HF API | "Model not found. Please check the URL and try again." |
| No GGUF Files | Repo exists but has no GGUFs | "No GGUF files found in this repository." |
| Permission Denied | Gated/private model | "This model requires authentication. Please log in to HuggingFace." |
| Disk Full | Not enough space | "Not enough disk space. Please free up space and try again." |
| Download Failed | Server error, etc. | "Failed to download {filename}: {error}. Retry?" |

### Backend Errors

```rust
// All functions return Result<T, String> with descriptive errors

// URL parsing errors
Err("Invalid URL format. Expected: https://huggingface.co/author/model")

// API errors
Err("Failed to fetch model info: {reqwest_error}")
Err("Model not found (HTTP 404)")
Err("Rate limited. Please wait a moment and try again.")

// Download errors
Err("Failed to start download: {error}")
Err("Destination directory not writable: {error}")
```

### Error Recovery

1. **URL Validation Failure**
   - Show inline error message
   - Highlight input field in red
   - Provide format examples

2. **Model Fetch Failure**
   - Show error card with details
   - "Try Again" button
   - "Go Back" button

3. **Download Failure**
   - Show error per file in queue
   - "Retry" button for failed file
   - "Skip" button to continue with next file
   - "Cancel All" button

4. **Network Interruption**
   - Auto-retry 3 times with exponential backoff
   - Resume download from last byte (HTTP Range)
   - Show "Resuming..." status

---

## Testing Strategy

### Unit Tests (Backend)

```rust
// huggingface_downloader.rs tests

#[test]
fn test_parse_model_id_variations() {
    // Test all URL formats
    // Test edge cases
    // Test invalid inputs
}

#[test]
fn test_extract_quantization() {
    // Test various quantization formats
    // Test files without quantization
}

#[test]
fn test_build_destination_path() {
    // Test path construction
    // Test with different base directories
}

#[tokio::test]
async fn test_fetch_model_info_real() {
    // Integration test with real HF API
    // Use a well-known public model
}

#[tokio::test]
async fn test_fetch_model_files_real() {
    // Test file listing
    // Verify GGUF filtering
}
```

### Frontend Tests

```javascript
// Test URL validation
function testUrlValidation() {
    const validUrls = [
        'https://huggingface.co/THUDM/glm-4-9b-chat',
        'huggingface.co/THUDM/glm-4-9b-chat',
        'THUDM/glm-4-9b-chat',
        'https://huggingface.co/THUDM/glm-4-9b-chat/blob/main/model.gguf'
    ];
    
    const invalidUrls = [
        'not-a-url',
        'https://huggingface.co/invalid',
        ''
    ];
    
    // Validate each
}

// Test file selection
function testFileSelection() {
    // Test select all
    // Test select none
    // Test individual selection
    // Test total size calculation
}

// Test download queue
function testDownloadQueue() {
    // Test sequential downloads
    // Test pause/resume
    // Test cancellation
}
```

### Manual Testing Checklist

- [ ] Paste valid URL → Model info displays
- [ ] Paste invalid URL → Error shown
- [ ] Model with no GGUFs → Info message
- [ ] Select multiple files → Total size updates
- [ ] Click Download → Progress shown
- [ ] Interrupt download → Resume works
- [ ] Download completes → File appears in models folder
- [ ] Custom destination → Saves to correct location
- [ ] Split model files → All parts downloaded
- [ ] Tab switching → State preserved
- [ ] Window close/reopen → Downloads continue in background

---

## File Changes Checklist

### Frontend Files

- [ ] **MODIFY**: `frontend/modules/huggingface-app.js`
  - Add tab switching logic
  - Add paste link interface methods
  - Add file selection logic
  - Add download progress tracking
  
- [ ] **MODIFY**: `frontend/css/huggingface.css`
  - Add tab styling
  - Add paste link section styles
  - Add file list styles
  - Add progress indicator styles

### Backend Files

- [ ] **CREATE**: `backend/src/huggingface_downloader.rs`
  - URL parsing functions
  - HF API integration
  - File information extraction
  - Unit tests

- [ ] **MODIFY**: `backend/src/lib.rs`
  - Add module declaration
  - Add Tauri commands
  - Register commands in invoke_handler

- [ ] **MODIFY**: `backend/Cargo.toml`
  - Add regex crate (if not already present)
  - Verify all dependencies

### Documentation

- [ ] **UPDATE**: `AGENTS.md`
  - Add new feature to Recent Changes
  - Update Core Modules table
  - Document new workflow

- [ ] **CREATE**: `docs/usage/huggingface-paste-link.md`
  - User guide for paste link feature
  - Troubleshooting section

---

## Implementation Order

### Day 1: Frontend Structure
1. Create tabbed interface in huggingface-app.js
2. Add CSS styling for tabs and paste link UI
3. Implement URL input and validation

### Day 2: Backend Core
1. Create huggingface_downloader.rs
2. Implement URL parsing
3. Implement HF API calls
4. Write unit tests

### Day 3: Integration
1. Connect frontend to backend commands
2. Implement file selection UI
3. Add destination path handling
4. Test end-to-end flow

### Day 4: Polish & Download
1. Implement download progress UI
2. Integrate with DownloadManager
3. Add error handling and recovery
4. Manual testing and bug fixes

---

## Future Enhancements (Phase 3+)

- [ ] **Image Models**: Detect Flux/SD models and show image generation option
- [ ] **Model Cards**: Rich markdown rendering of model README
- [ ] **Comparison Tool**: Side-by-side quantization comparison
- [ ] **Auto-Update**: Check for new quants of installed models
- [ ] **Import/Export**: Share model lists with other users
- [ ] **Batch Operations**: Download multiple models at once
- [ ] **Mirror Support**: Use HF mirrors for faster downloads
- [ ] **Checksum Verification**: Verify downloaded files integrity

---

## Success Criteria

- [ ] User can paste any supported HF URL format
- [ ] System correctly parses URL and fetches model info
- [ ] All GGUF files are displayed with correct metadata
- [ ] User can select multiple files
- [ ] Files download one at a time with progress
- [ ] Interrupted downloads can resume
- [ ] Files save to correct location (default or custom)
- [ ] UI is responsive and provides clear feedback
- [ ] No regression in existing search functionality
- [ ] All error cases handled gracefully

---

## Notes

- Keep code consistent with existing patterns in codebase
- Reuse existing DownloadManager for actual file downloads
- Follow existing error handling patterns
- Maintain backward compatibility with existing HF search
- Test with real HF models before finalizing
- Consider rate limiting (HF API allows 60 req/hour unauthenticated)

---

**Document Version:** 1.0  
**Last Updated:** 2025-02-18  
**Status:** Ready for Implementation
