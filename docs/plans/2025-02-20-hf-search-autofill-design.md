# Design: Auto-fill HF Search with Date Comparison

**Date:** 2025-02-20
**Status:** Approved
**Priority:** Medium

## Overview

When a user clicks the "?" update indicator next to a model icon, the system should:
1. Perform the existing update check
2. Open the HuggingFace search window
3. Auto-fill the search input with the HF model ID
4. Display search results with visual date comparison badges
5. Allow users to see and download updated versions

## Problem Statement

Currently, clicking the "?" indicator only shows update notifications but provides no direct way to find and download updated models. Users must manually search HuggingFace, which is inefficient and error-prone.

## Solution

Automatically open the HuggingFace search window with:
- Model ID pre-filled
- Visual comparison of local vs. remote model dates
- Clear indication of which versions are newer
- Direct access to download updated files

## Architecture

### Components

| Component | Responsibility |
|-----------|----------------|
| `desktop.js:handleCheckUpdate()` | Entry point, orchestrates update check + HF search |
| `desktop.js` | Stores local model metadata and creation dates |
| `huggingface-app.js` | HF search window, adds comparison context |
| `update_checker.rs` | Backend update check (existing) |

### Data Flow

```
User clicks "?" on model icon
    ↓
handleCheckUpdate(modelPath)
    ↓
1. Get local model metadata:
   - HF model ID (from hf_metadata or path extraction)
   - Local creation/modification date
   - Current quantization
    ↓
2. Store comparison data in desktop state
    ↓
3. Perform update check (existing logic)
    ↓
4. Pass comparison context to HF app
    ↓
5. Open HF search window
    ↓
6. Auto-fill search input with model ID
    ↓
7. Trigger search after 300ms delay
    ↓
8. Display results with date comparison badges
```

## User Interface

### HF Search Results Enhancement

Each model result card includes a date comparison badge:

```
┌─────────────────────────────────────────┐
│ [📂] Qwen/Qwen2.5-7B-Instruct           │
│ Conversational • transformers           │
│ ❤️ 1.2M  ⬇️ 345K                        │
│                                         │
│ [🔴 NEWER  Updated: 2 days ago]         │ ← COMPARE BADGE
│ [Your model: 15 days ago]                │
│                                         │
│ [View Files →]                          │
└─────────────────────────────────────────┘
```

### Badge Types

| Badge Color | Condition | Display |
|-------------|-----------|---------|
| 🟢 Green | Remote date ≤ Local date | "Up to date" |
| 🔴 Red | Remote date > Local date | "NEWER + days difference" |
| 🔵 Blue | Remote date unknown | "Date unknown" |

### Badge Content (Red/NEWER)

```
┌─────────────────────────────────────┐
│ ↑ NEWER                             │
│ Updated: 2 days ago (13 days newer) │
│ Your model: 15 days ago             │
└─────────────────────────────────────┘
```

## Implementation Details

### Frontend: `desktop.js`

**New state property:**
```javascript
this.updateComparisonData = {
    modelId: string,        // HF model ID for search
    localDate: number,      // Unix timestamp
    localPath: string,      // Full path to local model
    localQuantization: string // Current quantization
};
```

**Modified method:**
```javascript
async handleCheckUpdate(modelPath) {
    const model = this.models[modelPath];
    const invoke = window.__TAURI__.core.invoke;

    // Get local model creation date
    const localDate = model.creation_date ||
                     model.hf_metadata?.local_modified ||
                     await invoke('get_file_modification_date', { path: modelPath });

    // Build search query
    let searchQuery = '';
    if (model.hf_metadata?.model_id) {
        searchQuery = model.hf_metadata.model_id;
    } else {
        const parts = modelPath.split(/[\\/]/);
        if (parts.length >= 3) {
            searchQuery = `${parts[parts.length - 3]}/${parts[parts.length - 2]}`;
        } else {
            searchQuery = model.name;
        }
    }

    // Store comparison data
    this.updateComparisonData = {
        modelId: searchQuery,
        localDate: localDate,
        localPath: modelPath,
        localQuantization: model.quantization
    };

    // Perform update check (existing)
    const result = await invoke('check_model_update', { modelPath });
    this.updateUpdateIndicator(modelPath, result);

    // Open HF search with comparison context
    const hfApp = this.desktop.modules.get('huggingface');
    if (hfApp) {
        hfApp.setComparisonContext(this.updateComparisonData);
        hfApp.openHuggingFaceSearch().then(() => {
            setTimeout(() => {
                const searchInput = document.querySelector('#hf-search-input');
                if (searchInput && searchQuery) {
                    searchInput.value = searchQuery;
                    hfApp.performHuggingFaceSearch();
                }
            }, 300);
        });
    }

    if (result.update_available) {
        this.showNotification('Update available! Check HF search window.', 'success');
    }
}
```

### Frontend: `huggingface-app.js`

**New property:**
```javascript
this.comparisonContext = null;
```

**New method:**
```javascript
setComparisonContext(context) {
    this.comparisonContext = context;
}
```

**Modified method: `displayHuggingFaceResults()`:**
```javascript
displayHuggingFaceResults(models, query) {
    const window = this.desktop.windows.get(this.windowId);
    if (!window) return;
    const resultsContainer = window.querySelector('#hf-search-results');

    let modelsHtml = models.map(model => {
        const compareBadge = this.generateDateComparisonBadge(model);

        return `
            <div class="hf-model-item" data-model-id="${this.escapeHtml(model.id)}">
                <div class="model-header">
                    <div class="model-name">${this.escapeHtml(model.id)}</div>
                </div>
                <div class="model-meta">
                    ${this.modelTagsToHtml(model.tags, model.modelId)}
                </div>
                <div class="model-stats">
                    <span class="stat-item">❤️ ${this.formatNumber(model.likes)}</span>
                    <span class="stat-item">⬇️ ${this.formatNumber(model.downloads)}</span>
                </div>
                ${compareBadge}
                <button class="view-files-btn" data-model-id="${this.escapeHtml(model.id)}">
                    <span class="material-icons">folder_open</span>
                    View Files
                </button>
            </div>
        `;
    }).join('');

    resultsContainer.innerHTML = modelsHtml;
}
```

**New method: `generateDateComparisonBadge()`:**
```javascript
generateDateComparisonBadge(model) {
    if (!this.comparisonContext) return '';

    const localDate = this.comparisonContext.localDate;
    const remoteDate = model.lastModified;

    if (!remoteDate) {
        return `
            <div class="date-comparison-badge blue">
                <span class="material-icons">help</span>
                <span>Date unknown</span>
            </div>
        `;
    }

    const localObj = new Date(localDate * 1000);
    const remoteObj = new Date(remoteDate);

    if (remoteObj > localObj) {
        const daysDiff = Math.floor((remoteObj - localObj) / (1000 * 60 * 60 * 24));
        return `
            <div class="date-comparison-badge red">
                <span class="material-icons">arrow_upward</span>
                <span class="badge-title">NEWER</span>
                <span>Updated: ${this.formatRelativeDate(remoteDate)} (${daysDiff} days newer)</span>
                <span class="your-model-date">Your model: ${this.formatRelativeDate(localDate)}</span>
            </div>
        `;
    } else {
        return `
            <div class="date-comparison-badge green">
                <span class="material-icons">check_circle</span>
                <span>Up to date</span>
            </div>
        `;
    }
}
```

**New utility method: `formatRelativeDate()`:**
```javascript
formatRelativeDate(timestamp) {
    const date = new Date(timestamp * 1000);
    const now = new Date();
    const diffDays = Math.floor((now - date) / (1000 * 60 * 60 * 24));

    if (diffDays < 1) return 'today';
    if (diffDays === 1) return 'yesterday';
    if (diffDays < 7) return `${diffDays} days ago`;
    if (diffDays < 30) return `${Math.floor(diffDays / 7)} weeks ago`;
    if (diffDays < 365) return `${Math.floor(diffDays / 30)} months ago`;
    return `${Math.floor(diffDays / 365)} years ago`;
}
```

**Optional: Reset comparison context:**
```javascript
// Call this when user performs manual search not via update checker
clearComparisonContext() {
    this.comparisonContext = null;
}
```

### CSS: `frontend/css/huggingface.css`

```css
/* Date Comparison Badges */
.date-comparison-badge {
    margin-top: 8px;
    padding: 8px 12px;
    border-radius: 6px;
    font-size: 12px;
    display: flex;
    align-items: center;
    gap: 6px;
}

.date-comparison-badge.red {
    background: rgba(239, 68, 68, 0.15);
    border: 1px solid rgba(239, 68, 68, 0.3);
    color: #ef4444;
    flex-direction: column;
    align-items: flex-start;
}

.date-comparison-badge.green {
    background: rgba(34, 197, 94, 0.15);
    border: 1px solid rgba(34, 197, 94, 0.3);
    color: #22c55e;
}

.date-comparison-badge.blue {
    background: rgba(59, 130, 246, 0.15);
    border: 1px solid rgba(59, 130, 246, 0.3);
    color: #3b82f6;
}

.badge-title {
    font-weight: 700;
    text-transform: uppercase;
    font-size: 10px;
    letter-spacing: 0.5px;
}

.your-model-date {
    font-style: italic;
    opacity: 0.8;
    margin-top: 4px;
}
```

## Edge Cases

| Scenario | Behavior |
|----------|----------|
| Model not linked, no HF path structure | Use GGUF metadata name as search term |
| No local creation date available | Fetch via `get_file_modification_date` backend call |
| HF window already open | Update search input, set new comparison context, trigger search |
| No HF app initialized | Show error notification |
| Network error on update check | Still open HF search, show error notification |
| Remote model has no `lastModified` field | Show blue "Date unknown" badge |
| Empty searchQuery | Don't update search field, show error |
| Local and remote dates equal | Show green "Up to date" badge |
| Search returns no results | Show "No results found" message |

## Data Structures

### Comparison Context

```javascript
{
    modelId: string,        // "meta-llama/Llama-3-8B-Instruct"
    localDate: number,      // Unix timestamp (seconds)
    localPath: string,      // "H:/Arandu/models/meta-llama/Llama-3-8B-Instruct/..."
    localQuantization: string // "Q4_K_M"
}
```

### HF Model Result (with lastModified)

```javascript
{
    id: string,             // "meta-llama/Llama-3-8B-Instruct"
    modelId: string,        // Display name
    likes: number,
    downloads: number,
    tags: string[],
    lastModified: number    // Unix timestamp (milliseconds from HF API)
}
```

## Testing Checklist

- [ ] Click "?" on linked model → HF search opens with model ID pre-filled
- [ ] Click "?" on unlinked model with path structure → HF search opens with extracted ID
- [ ] Click "?" on unlinked model without path structure → HF search opens with model name
- [ ] Search results show red "NEWER" badge for remote models newer than local
- [ ] Search results show green "Up to date" badge for equal or older models
- [ ] Search results show blue "Date unknown" badge when remote has no date
- [ ] Badge shows correct days difference (remote - local)
- [ ] Badge shows "Your model: X days ago" text
- [ ] Update check still performs correctly and shows notifications
- [ ] HF search window existing → updates correctly without opening new window
- [ ] User performs manual search → comparison context cleared (no badges)
- [ ] Network error → HF search still opens, error notification shows
- [ ] View Files button works → opens file selection dialog

## Performance Considerations

- Update check and HF search open happen in parallel (async)
- 300ms delay ensures HF window is fully rendered before auto-search
- Comparison context is lightweight JSON object (< 200 bytes)
- Date calculations are client-side, no additional backend calls

## Security Considerations

- No user data sent to external endpoints (HF API already called for search)
- Model paths already accessible via existing desktop state
- No changes to authentication or authorization

## Accessibility

- Badge uses Material Icons for visual context
- Color contrast meets WCAG AA (red #ef4444 on light bg, green #22c55e, blue #3b82f6)
- Screen reader-friendly text: "NEWER Updated 2 days ago. Your model: 15 days ago"

## Future Enhancements

- [ ] Show quantization comparison (e.g., "You have Q4, this has Q8")
- [ ] Filter results to only show models with newer versions
- [ ] Batch download multiple updated quantizations
- [ ] Auto-download latest version with confirmation dialog

## Dependencies

- None (uses existing HF search infrastructure)
- `gguf_parser.rs` already provides `get_file_modification_date()`
- `huggingface-app.js` already exists with search functionality
- `desktop.js` already has model metadata

## Migration Notes

- No breaking changes to existing functionality
- Backward compatible with existing update checker
- Comparison context is optional (null = no badges shown)
- Existing manual search workflow unchanged

## Rollback Plan

If issues arise:
1. Remove `setComparisonContext()` calls
2. Remove `generateDateComparisonBadge()` method
3. Remove date comparison badge CSS
4. Restore original `displayHuggingFaceResults()` (remove badge HTML)
5. Update checker continues to work independently

## Success Criteria

- [ ] User clicks "?" → HF search opens in under 1 second
- [ ] Search input auto-filled with correct model ID (100% accuracy)
- [ ] Date badge appears and shows correct status
- [ ] User can identify which models are newer
- [ ] User can navigate to download updated files
- [ ] No regression in existing update check functionality
- [ ] Works for both linked and unlinked models
