# Tracker Hybrid Search Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Implement hybrid local + live search for AI Model Tracker showing cached results immediately while fetching fresh HF data in background.

**Architecture:** Extend existing tracker with live fetch capability that runs parallel to cached data display. Live results merge into UI without overwriting database until user syncs.

**Tech Stack:** Rust (Tauri backend), JavaScript (frontend), SQLite (caching), HuggingFace API

---

## Task 1: Add Total Models Badge to UI

**Files:**
- Modify: `frontend/modules/tracker-app.js:145-150`
- Modify: `frontend/css/tracker.css`

**Step 1: Add badge HTML to tracker render**

In `frontend/modules/tracker-app.js` around line 145, add before the models-grid:

```javascript
<div class="tracker-badge-bar" id="tracker-badge-bar">
    <div class="badge-section cached">
        <span class="badge-icon">📦</span>
        <span class="badge-label">Cached:</span>
        <span class="badge-count" id="badge-cached-count">0</span>
    </div>
    <div class="badge-section live" id="badge-live-section" style="display: none;">
        <span class="badge-icon">🔍</span>
        <span class="badge-label">Live:</span>
        <span class="badge-count" id="badge-live-count">0</span>
        <span class="loading-spinner-small" id="live-loading-spinner"></span>
    </div>
    <div class="badge-toggle">
        <label class="toggle-label">
            <input type="checkbox" id="live-search-toggle" checked>
            <span class="toggle-text">Live Search</span>
        </label>
    </div>
</div>
```

**Step 2: Add CSS styles for badge bar**

In `frontend/css/tracker.css`, add:

```css
.tracker-badge-bar {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 10px 15px;
    background: var(--theme-surface);
    border-bottom: 1px solid var(--theme-border);
    margin-bottom: 10px;
}

.badge-section {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 5px 12px;
    border-radius: 20px;
    background: var(--theme-bg);
}

.badge-section.cached {
    border: 1px solid var(--theme-primary);
}

.badge-section.live {
    border: 1px solid #4ade80;
}

.badge-count {
    font-weight: bold;
    font-size: 1.1em;
}

.loading-spinner-small {
    width: 14px;
    height: 14px;
    border: 2px solid #4ade80;
    border-top-color: transparent;
    border-radius: 50%;
    animation: spin 1s linear infinite;
}

@keyframes spin {
    to { transform: rotate(360deg); }
}

.toggle-label {
    display: flex;
    align-items: center;
    gap: 8px;
    cursor: pointer;
    font-size: 0.9em;
}
```

**Step 3: Add badge update method**

In `frontend/modules/tracker-app.js` after line 333, add:

```javascript
updateBadgeCounts(cachedCount, liveCount, isLoading) {
    const cachedEl = document.getElementById('badge-cached-count');
    const liveSection = document.getElementById('badge-live-section');
    const liveCountEl = document.getElementById('badge-live-count');
    const spinner = document.getElementById('live-loading-spinner');
    
    if (cachedEl) cachedEl.textContent = cachedCount.toLocaleString();
    
    if (liveSection) {
        if (liveCount !== null) {
            liveSection.style.display = 'flex';
            if (liveCountEl) liveCountEl.textContent = liveCount.toLocaleString();
            if (spinner) spinner.style.display = isLoading ? 'inline-block' : 'none';
        } else {
            liveSection.style.display = 'none';
        }
    }
}
```

**Step 4: Call badge update in renderStats**

Modify `renderStats` method (around line 329) to also call badge update:

```javascript
renderStats(stats) {
    document.getElementById('stat-total').textContent = (stats.total_models || 0).toLocaleString();
    document.getElementById('stat-chinese').textContent = (stats.chinese_models || 0).toLocaleString();
    document.getElementById('stat-gguf').textContent = (stats.gguf_models || 0).toLocaleString();
    document.getElementById('stat-categories').textContent = stats.categories ? Object.keys(stats.categories).length : 0;
    
    // Update badge counts
    this.updateBadgeCounts(stats.total_models || 0, null, false);
}
```

**Step 5: Test badge renders**

Run app and open tracker window.
Expected: Badge bar appears at top with "📦 Cached: 0" and toggle switch visible.

**Step 6: Commit**

```bash
git add frontend/modules/tracker-app.js frontend/css/tracker.css
git commit -m "feat: add total models badge with cached/live counts and toggle"
```

---

## Task 2: Add Live Search Toggle Event Handler

**Files:**
- Modify: `frontend/modules/tracker-app.js:155-248`

**Step 1: Add toggle event listener in setupEventListeners**

After line 248, add:

```javascript
// Live search toggle
const liveToggle = document.getElementById('live-search-toggle');
if (liveToggle) {
    liveToggle.addEventListener('change', (e) => {
        this.liveSearchEnabled = e.target.checked;
        if (this.liveSearchEnabled) {
            this.triggerLiveFetch();
        } else {
            this.hideLiveResults();
        }
    });
}
```

**Step 2: Add live search state to constructor**

In constructor (around line 7), add:

```javascript
this.liveSearchEnabled = true;
this.liveFetchTimeout = null;
this.currentLiveResults = [];
```

**Step 3: Add triggerLiveFetch method**

After line 307, add:

```javascript
async triggerLiveFetch() {
    if (!this.liveSearchEnabled) return;
    
    // Debounce: clear existing timeout
    if (this.liveFetchTimeout) {
        clearTimeout(this.liveFetchTimeout);
    }
    
    // Show loading state
    this.updateBadgeCounts(
        document.getElementById('badge-cached-count')?.textContent || 0,
        0,
        true
    );
    
    // Wait 500ms after filters stabilize
    this.liveFetchTimeout = setTimeout(() => {
        this.fetchLiveResults();
    }, 500);
}
```

**Step 4: Test toggle functionality**

Run app, open tracker, toggle live search off/on.
Expected: Toggle works and triggers console log (fetch not yet implemented).

**Step 5: Commit**

```bash
git add frontend/modules/tracker-app.js
git commit -m "feat: add live search toggle with debounce"
```

---

## Task 3: Create Backend Live Fetch Command

**Files:**
- Modify: `backend/src/tracker_scraper.rs`
- Modify: `backend/src/lib.rs`

**Step 1: Add live fetch method to TrackerScraper**

In `backend/src/tracker_scraper.rs` after line 153, add:

```rust
pub async fn fetch_live_results(
    &self,
    query: Option<String>,
    categories: Option<Vec<String>>,
    chinese_only: bool,
    gguf_only: bool,
    limit: u32,
) -> Result<Vec<TrackerModel>, String> {
    let mut url = format!(
        "https://huggingface.co/api/models?sort=downloads&direction=-1&limit={}&full=true",
        limit.min(200) // Cap at 200 for speed
    );
    
    // Add search query if provided
    if let Some(q) = query {
        if !q.is_empty() {
            url.push_str(&format!("&search={}", urlencoding::encode(&q)));
        }
    }
    
    // Add filter tags based on categories
    if let Some(cats) = categories {
        for cat in cats {
            let tag = match cat.as_str() {
                "text" => "text-generation",
                "image" => "image-generation",
                "audio" => "audio-processing",
                "video" => "video-processing",
                "coding" => "code-generation",
                "multimodal" => "multimodal",
                _ => continue,
            };
            url.push_str(&format!("&filter={}", tag));
        }
    }
    
    let response = self.client
        .get(&url)
        .header("Accept", "application/json")
        .send()
        .await
        .map_err(|e| format!("Failed to fetch live results: {}", e))?;
    
    let models: Vec<HFSearchResponse> = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse live results: {}", e))?;
    
    let mut tracker_models = Vec::new();
    
    for model in models {
        let model_id = if model.model_id.is_empty() {
            format!("{}/{}", model.author, model.id)
        } else {
            model.model_id.clone()
        };
        
        // Apply filters
        if chinese_only && !Self::is_chinese_model(&model_id, &model.tags) {
            continue;
        }
        
        // Get detailed info for GGUF detection
        let details = self.fetch_model_details(&model_id).await;
        let files = self.fetch_model_files(&model_id).await.unwrap_or_default();
        let quants = Self::detect_quantizations(&files);
        let is_gguf = !quants.is_empty();
        
        if gguf_only && !is_gguf {
            continue;
        }
        
        let category = Self::categorize_model(
            &details.as_ref().ok().map(|d| d.tags.clone()).unwrap_or_default(),
            "",
        );
        
        let size_gb = files.iter().map(|f| f.size as f64).sum::<f64>() / 1_000_000_000.0;
        
        tracker_models.push(TrackerModel {
            id: model_id,
            name: model.id.clone(),
            author: model.author.clone(),
            description: details.as_ref()
                .ok()
                .and_then(|d| d.pipeline_tag.clone())
                .unwrap_or_else(|| "No description available".to_string())
                .chars().take(200).collect(),
            source: "huggingface".to_string(),
            category,
            is_chinese: Self::is_chinese_model(&model_id, &model.tags),
            is_gguf,
            quantizations: quants,
            backends: details.as_ref()
                .map(|d| Self::detect_backends(&d.tags))
                .unwrap_or_default(),
            estimated_size_gb: size_gb,
            vram_requirement_gb: None,
            context_length: None,
            downloads: model.downloads,
            likes: model.likes,
            last_updated: Some(model.last_modified),
            created_at: Utc::now().to_rfc3339(),
        });
    }
    
    Ok(tracker_models)
}
```

**Step 2: Add Tauri command in lib.rs**

In `backend/src/lib.rs`, find the tauri::Builder section and add command:

```rust
#[tauri::command]
async fn get_tracker_live_results(
    query: Option<String>,
    categories: Option<Vec<String>>,
    chinese_only: bool,
    gguf_only: bool,
    limit: u32,
) -> Result<Vec<TrackerModel>, String> {
    let scraper = TrackerScraper::new();
    scraper.fetch_live_results(query, categories, chinese_only, gguf_only, limit).await
}
```

**Step 3: Register command in tauri::Builder**

Find where commands are registered (look for `.invoke_handler`) and add:

```rust
.invoke_handler(tauri::generate_handler![
    // ... existing commands ...
    get_tracker_live_results,
])
```

**Step 4: Test backend command**

Build backend:
```bash
cd backend && cargo build
```

Expected: Compiles without errors.

**Step 5: Commit**

```bash
git add backend/src/tracker_scraper.rs backend/src/lib.rs
git commit -m "feat: add get_tracker_live_results Tauri command"
```

---

## Task 4: Implement Frontend Live Fetch and Merge

**Files:**
- Modify: `frontend/modules/tracker-app.js`

**Step 1: Add fetchLiveResults method**

After line 274, add:

```javascript
async fetchLiveResults() {
    if (!this.liveSearchEnabled) return;
    
    const filters = this.currentFilters;
    
    try {
        this.updateBadgeCounts(
            document.getElementById('badge-cached-count')?.textContent || 0,
            0,
            true
        );
        
        const liveModels = await window.__TAURI__.core.invoke('get_tracker_live_results', {
            query: filters.search || null,
            categories: filters.category ? [filters.category] : null,
            chineseOnly: filters.chineseOnly,
            ggufOnly: filters.ggufOnly,
            limit: 100
        });
        
        this.currentLiveResults = liveModels;
        this.mergeLiveResults(liveModels);
        
        this.updateBadgeCounts(
            document.getElementById('badge-cached-count')?.textContent || 0,
            liveModels.length,
            false
        );
        
    } catch (error) {
        console.error('Error fetching live results:', error);
        this.desktop.showNotification('Live search failed: ' + error, 'error');
        this.updateBadgeCounts(
            document.getElementById('badge-cached-count')?.textContent || 0,
            null,
            false
        );
    }
}
```

**Step 2: Add mergeLiveResults method**

```javascript
mergeLiveResults(liveModels) {
    const grid = document.getElementById('tracker-models-grid');
    if (!grid) return;
    
    // Get current cached models from grid
    const existingCards = grid.querySelectorAll('.model-card');
    const existingIds = new Set();
    existingCards.forEach(card => {
        const id = card.getAttribute('data-model-id');
        if (id) existingIds.add(id);
    });
    
    // Create document fragment for new cards
    const fragment = document.createDocumentFragment();
    let newCount = 0;
    let updatedCount = 0;
    
    liveModels.forEach(model => {
        if (existingIds.has(model.id)) {
            // Update existing card
            const card = grid.querySelector(`[data-model-id="${model.id}"]`);
            if (card) {
                this.updateModelCard(card, model);
                updatedCount++;
            }
        } else {
            // Create new card with "New" badge
            const cardHtml = this.renderModelCard({
                ...model,
                is_new: true
            });
            const tempDiv = document.createElement('div');
            tempDiv.innerHTML = cardHtml;
            fragment.appendChild(tempDiv.firstElementChild);
            newCount++;
        }
    });
    
    // Prepend new cards to top
    if (fragment.childNodes.length > 0) {
        grid.insertBefore(fragment, grid.firstChild);
    }
    
    console.log(`Merged ${newCount} new models, updated ${updatedCount} existing`);
}
```

**Step 3: Add updateModelCard method**

```javascript
updateModelCard(card, model) {
    // Update stats
    const statsEl = card.querySelector('.model-card-stats');
    if (statsEl) {
        const downloadsStr = model.downloads >= 1000 
            ? `${(model.downloads / 1000).toFixed(1)}K` 
            : model.downloads.toString();
        const likesStr = model.likes >= 1000 
            ? `${(model.likes / 1000).toFixed(1)}K` 
            : model.likes.toString();
        
        statsEl.innerHTML = `
            <span>⬇️ ${downloadsStr}</span>
            <span>❤️ ${likesStr}</span>
            <span>📅 ${model.last_updated ? new Date(model.last_updated).toLocaleDateString() : ''}</span>
        `;
    }
    
    // Add "updated" indicator
    if (!card.querySelector('.updated-badge')) {
        const header = card.querySelector('.model-card-header');
        if (header) {
            const badge = document.createElement('span');
            badge.className = 'badge updated-badge';
            badge.textContent = '↻';
            badge.title = 'Updated from live search';
            header.appendChild(badge);
        }
    }
}
```

**Step 4: Modify renderModelCard to support is_new flag**

Update method signature and add badge:

```javascript
renderModelCard(model) {
    const quantBadges = (model.quantizations || []).slice(0, 3).map(q => 
        `<span class="badge quant">${q}</span>`
    ).join('');
    
    const backendBadges = (model.backends || []).slice(0, 4).map(b => 
        `<span class="badge backend">${b.toUpperCase()}</span>`
    ).join('');
    
    const sizeStr = model.estimated_size_gb ? `${model.estimated_size_gb.toFixed(1)} GB` : '? GB';
    const vramStr = model.vram_requirement_gb ? `~${model.vram_requirement_gb.toFixed(0)}GB VRAM` : '';
    const ctxStr = model.context_length ? `${(model.context_length / 1024).toFixed(0)}K ctx` : '';
    
    const downloadsStr = model.downloads >= 1000 
        ? `${(model.downloads / 1000).toFixed(1)}K` 
        : model.downloads.toString();
    const likesStr = model.likes >= 1000 
        ? `${(model.likes / 1000).toFixed(1)}K` 
        : model.likes.toString();
    
    // NEW: Add is_new badge
    const newBadge = model.is_new ? '<span class="badge new">🆕 New</span>' : '';
    
    return `
        <div class="model-card" data-model-id="${this.escapeHtml(model.id)}">
            <div class="model-card-header">
                <span class="model-name">${this.escapeHtml(model.name)}</span>
                <span class="source-badge">HF</span>
                ${newBadge}
            </div>
            <div class="model-card-desc">${this.escapeHtml(model.description || 'No description')}</div>
            <div class="model-card-meta">
                <span>📦 ${sizeStr}</span>
                ${vramStr ? `<span>🎮 ${vramStr}</span>` : ''}
                ${ctxStr ? `<span>📝 ${ctxStr}</span>` : ''}
            </div>
            <div class="model-card-badges">
                ${quantBadges}
                ${backendBadges}
                ${model.is_chinese ? '<span class="badge chinese">🇨🇳</span>' : ''}
            </div>
            <div class="model-card-stats">
                <span>⬇️ ${downloadsStr}</span>
                <span>❤️ ${likesStr}</span>
                <span>📅 ${model.last_updated ? new Date(model.last_updated).toLocaleDateString() : ''}</span>
            </div>
            <div class="model-card-actions">
                <button class="btn-small" onclick="trackerApp.viewOnHF('${this.escapeHtml(model.id)}')">View on HF</button>
            </div>
        </div>
    `;
}
```

**Step 5: Add CSS for new badges**

In `frontend/css/tracker.css`:

```css
.badge.new {
    background: linear-gradient(135deg, #4ade80, #22c55e);
    color: white;
    font-size: 0.7em;
    padding: 2px 8px;
    animation: pulse 2s ease-in-out infinite;
}

.badge.updated-badge {
    background: #3b82f6;
    color: white;
    font-size: 0.8em;
    padding: 2px 6px;
    margin-left: 8px;
}

@keyframes pulse {
    0%, 100% { opacity: 1; }
    50% { opacity: 0.7; }
}
```

**Step 6: Test live fetch**

Run full app build, open tracker, enable live search.
Expected: Badge shows loading spinner, then updates with live count and new models appear with 🆕 badge.

**Step 7: Commit**

```bash
git add frontend/modules/tracker-app.js frontend/css/tracker.css
git commit -m "feat: implement live fetch and merge with new/updated badges"
```

---

## Task 5: Trigger Live Fetch on Filter Changes

**Files:**
- Modify: `frontend/modules/tracker-app.js:276-307`

**Step 1: Modify applyFilters to trigger live fetch**

Update `applyFilters` method (around line 276):

```javascript
async applyFilters() {
    const grid = document.getElementById('tracker-models-grid');
    if (!grid) return;
    
    grid.innerHTML = '<div class="loading-placeholder">Loading...</div>';

    try {
        const filters = this.currentFilters;
        
        // 1. Get cached results first (instant)
        const cachedModels = await window.__TAURI__.core.invoke('get_tracker_models', {
            vramLimit: filters.vramLimit,
            categories: filters.category ? [filters.category] : null,
            chineseOnly: filters.chineseOnly,
            ggufOnly: filters.ggufOnly,
            backends: filters.backends.length > 0 ? filters.backends : null,
            quantizations: filters.quant ? [filters.quant] : null,
            search: filters.search || null,
            sortBy: filters.sortBy,
            sortDesc: filters.sortDesc
        });
        
        // Get stats for badge
        const stats = await window.__TAURI__.core.invoke('get_tracker_stats');
        
        // Render cached results immediately
        this.renderStats(stats);
        this.renderModelCards(cachedModels);
        this.updateBadgeCounts(stats.total_models || 0, null, false);
        
        // 2. Trigger live fetch in background (if enabled)
        if (this.liveSearchEnabled) {
            this.triggerLiveFetch();
        }
        
    } catch (error) {
        console.error('Error applying filters:', error);
        grid.innerHTML = `<div class="no-models">Error: ${error}</div>`;
    }
}
```

**Step 2: Test filter changes trigger live fetch**

Run app, change filters (category, search text, etc.).
Expected: Cached results appear instantly, then live badge shows loading, then updates with live count.

**Step 3: Commit**

```bash
git add frontend/modules/tracker-app.js
git commit -m "feat: trigger live fetch on filter changes with instant cached display"
```

---

## Task 6: Add Rate Limiting and Error Handling

**Files:**
- Modify: `frontend/modules/tracker-app.js`

**Step 1: Add rate limiting state**

In constructor, add:

```javascript
this.apiCallCount = 0;
this.apiCallResetTime = Date.now();
this.rateLimitWarningShown = false;
```

**Step 2: Add checkRateLimit method**

```javascript
checkRateLimit() {
    const now = Date.now();
    const oneMinute = 60 * 1000;
    
    // Reset counter every minute
    if (now - this.apiCallResetTime > oneMinute) {
        this.apiCallCount = 0;
        this.apiCallResetTime = now;
        this.rateLimitWarningShown = false;
    }
    
    // Check if approaching limit (50 calls/minute)
    if (this.apiCallCount >= 50 && !this.rateLimitWarningShown) {
        this.desktop.showNotification(
            '⚠️ Approaching API limit. Slowing down live search.',
            'warning'
        );
        this.rateLimitWarningShown = true;
        return false; // Don't allow more calls
    }
    
    // Hard limit at 60
    if (this.apiCallCount >= 60) {
        console.log('Rate limit reached, skipping live fetch');
        return false;
    }
    
    this.apiCallCount++;
    return true;
}
```

**Step 3: Update triggerLiveFetch to check rate limit**

```javascript
async triggerLiveFetch() {
    if (!this.liveSearchEnabled) return;
    
    // Check rate limit
    if (!this.checkRateLimit()) {
        this.updateBadgeCounts(
            document.getElementById('badge-cached-count')?.textContent || 0,
            null,
            false
        );
        return;
    }
    
    // ... rest of method
}
```

**Step 4: Update fetchLiveResults with better error handling**

```javascript
async fetchLiveResults() {
    if (!this.liveSearchEnabled) return;
    
    const filters = this.currentFilters;
    
    try {
        this.updateBadgeCounts(
            document.getElementById('badge-cached-count')?.textContent || 0,
            0,
            true
        );
        
        const liveModels = await window.__TAURI__.core.invoke('get_tracker_live_results', {
            query: filters.search || null,
            categories: filters.category ? [filters.category] : null,
            chineseOnly: filters.chineseOnly,
            ggufOnly: filters.ggufOnly,
            limit: 100
        });
        
        this.currentLiveResults = liveModels;
        this.mergeLiveResults(liveModels);
        
        this.updateBadgeCounts(
            document.getElementById('badge-cached-count')?.textContent || 0,
            liveModels.length,
            false
        );
        
    } catch (error) {
        console.error('Error fetching live results:', error);
        
        // Check if it's a rate limit error
        const errorStr = error.toString().toLowerCase();
        if (errorStr.includes('rate') || errorStr.includes('limit') || errorStr.includes('429')) {
            this.desktop.showNotification(
                '⚠️ HuggingFace API rate limit reached. Using cached results only.',
                'warning'
            );
            this.liveSearchEnabled = false;
            const toggle = document.getElementById('live-search-toggle');
            if (toggle) toggle.checked = false;
        } else {
            this.desktop.showNotification('Live search failed: ' + error, 'error');
        }
        
        this.updateBadgeCounts(
            document.getElementById('badge-cached-count')?.textContent || 0,
            null,
            false
        );
    }
}
```

**Step 5: Test rate limiting**

Rapidly change filters multiple times.
Expected: After 50 changes, warning appears. After 60, live search stops with notification.

**Step 6: Commit**

```bash
git add frontend/modules/tracker-app.js
git commit -m "feat: add API rate limiting with warnings and auto-disable"
```

---

## Task 7: Build and Verify

**Files:**
- All modified files

**Step 1: Full build**

```bash
cd backend
cargo tauri build
```

**Step 2: Test all features**

1. Open tracker window
2. Verify badge shows "📦 Cached: X" with toggle
3. Toggle live search on
4. Change filters - verify cached loads instantly, then live updates
5. Verify new models have 🆕 badge
6. Verify updated models show ↻ badge
7. Rapidly change filters 60+ times - verify rate limiting kicks in

**Step 3: Final commit**

```bash
git add -A
git commit -m "feat: complete tracker hybrid search with live/cached merge"
```

---

## Summary

This implementation adds:
1. **Total models badge** showing cached and live counts
2. **Live search toggle** with debounce (500ms)
3. **Instant cached display** with background live fetch
4. **Smart merge** showing 🆕 for new models, ↻ for updates
5. **Rate limiting** (60 calls/minute) with warnings
6. **Graceful fallback** to cached-only when API limits hit

**Files Modified:**
- `frontend/modules/tracker-app.js` - Core logic
- `frontend/css/tracker.css` - Styling
- `backend/src/tracker_scraper.rs` - Live fetch method
- `backend/src/lib.rs` - Tauri command registration

**New Tauri Commands:**
- `get_tracker_live_results` - Fetches live HF data without DB save
