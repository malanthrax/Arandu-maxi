console.log("Loading tracker-app.js...");
class TrackerApp {
    constructor(desktop) {
        this.desktop = desktop;
        this.windowId = 'tracker-window';
        this.isInitialized = false;
        this.currentFilters = {
            search: '',
            category: '',
            chineseOnly: false,
            ggufOnly: false,
            fileTypes: [],
            quant: '',
            vramLimit: 24,
            sortBy: 'downloads',
            sortDesc: true
        };
        this.liveSearchEnabled = true;
        this.liveFetchTimeout = null;
        this.currentLiveResults = [];
        this.apiCallCount = 0;
        this.apiCallResetTime = Date.now();
        this.rateLimitWarningShown = false;
    }

    async init() {
        if (this.isInitialized) return;
        console.log('TrackerApp initializing...');
        this.isInitialized = true;
    }

    async openTracker() {
        await this.init();
        
        // Check if window already exists
        const existingWindow = document.getElementById(this.windowId);
        if (existingWindow) {
            this.desktop.bringToFront(this.windowId);
            return;
        }

        const content = this.render();
        this.desktop.createWindow(this.windowId, 'AI Model Tracker', 'tracker-window', content);
        
        // Set up event listeners after window is created
        setTimeout(() => this.setupEventListeners(), 100);
        
        // Load initial data - check if we have cached models
        setTimeout(async () => {
            try {
                const stats = await window.__TAURI__.core.invoke('get_tracker_stats');
                const loadingEl = document.getElementById('tracker-loading');
                const contentEl = document.getElementById('tracker-content');
                
                if (stats && stats.total_models > 0) {
                    // We have cached data - show it immediately
                    if (loadingEl) loadingEl.classList.add('hidden');
                    if (contentEl) contentEl.classList.remove('hidden');
                    await this.applyFilters();
                }
                // If no cached data, keep the loading screen with "Fetch Models" button
            } catch (error) {
                console.error('Error checking for cached models:', error);
            }
        }, 150);
    }

    render() {
        return `
            <div class="tracker-container">
                <div class="tracker-header">
                    <h2>🤖 AI Model Tracker</h2>
                    <div class="tracker-actions">
                        <button class="tracker-btn" id="tracker-refresh">
                            <span class="material-icons">refresh</span> Refresh
                        </button>
                        <button class="tracker-btn" id="tracker-export">
                            <span class="material-icons">download</span> Export
                        </button>
                    </div>
                </div>
                <div class="tracker-loading" id="tracker-loading">
                    <div class="loading-spinner"></div>
                    <p>Loading models from HuggingFace...</p>
                    <p style="font-size: 0.8rem; color: #888;">Click Refresh to fetch latest models</p>
                    <button class="tracker-btn" id="tracker-initial-refresh">Fetch Models</button>
                </div>
                <div class="tracker-content hidden" id="tracker-content">
                    <div class="tracker-stats-panel">
                        <h3>Statistics</h3>
                        <div class="stat-item">
                            <span class="stat-value" id="stat-total">0</span>
                            <span class="stat-label">Total Models</span>
                        </div>
                        <div class="stat-item">
                            <span class="stat-value" id="stat-chinese">0</span>
                            <span class="stat-label">Chinese Models</span>
                        </div>
                        <div class="stat-item">
                            <span class="stat-value" id="stat-gguf">0</span>
                            <span class="stat-label">GGUF Models</span>
                        </div>
                        <div class="stat-item">
                            <span class="stat-value" id="stat-categories">0</span>
                            <span class="stat-label">Categories</span>
                        </div>
                    </div>
                    <div class="tracker-filters-panel">
                        <h3>Filters</h3>
                        <div class="filter-group">
                            <label>Search</label>
                            <input type="text" id="filter-search" placeholder="Search models..." class="filter-input">
                        </div>
                        <div class="filter-group">
                            <label>Category</label>
                            <select id="filter-category" class="filter-select">
                                <option value="">All Categories</option>
                                <option value="text">Text</option>
                                <option value="image">Image</option>
                                <option value="video">Video</option>
                                <option value="audio">Audio</option>
                                <option value="coding">Coding</option>
                                <option value="multimodal">Multimodal</option>
                            </select>
                        </div>
                        <div class="filter-group">
                            <label class="checkbox-label">
                                <input type="checkbox" id="filter-chinese"> Chinese Only
                            </label>
                        </div>
                        <div class="filter-group">
                            <label class="checkbox-label">
                                <input type="checkbox" id="filter-gguf"> GGUF Only
                            </label>
                        </div>
                        <div class="filter-group">
                            <label>VRAM Limit (GB)</label>
                            <input type="number" id="filter-vram" value="24" min="1" max="128" class="filter-input">
                        </div>
                        <div class="filter-group">
                            <label>File Types</label>
                            <div class="checkbox-group">
                                <label class="checkbox-label"><input type="checkbox" class="filetype-checkbox" value="gguf"> GGUF</label>
                                <label class="checkbox-label"><input type="checkbox" class="filetype-checkbox" value="mlx"> MLX</label>
                                <label class="checkbox-label"><input type="checkbox" class="filetype-checkbox" value="safetensors"> SafeTensors</label>
                                <label class="checkbox-label"><input type="checkbox" class="filetype-checkbox" value="bin"> .bin</label>
                                <label class="checkbox-label"><input type="checkbox" class="filetype-checkbox" value="pt"> PyTorch</label>
                            </div>
                        </div>
                        <div class="filter-group">
                            <label>Quantization</label>
                            <input type="text" id="filter-quant" placeholder="e.g., Q4, Q8, IQ2" class="filter-input">
                        </div>
                        <div class="filter-group">
                            <label>Sort By</label>
                            <select id="filter-sort" class="filter-select">
                                <option value="downloads">Downloads</option>
                                <option value="likes">Likes</option>
                                <option value="date">Date</option>
                                <option value="name">Name</option>
                                <option value="size">Size</option>
                            </select>
                        </div>
                        <div class="filter-group">
                            <label class="checkbox-label">
                                <input type="checkbox" id="filter-sort-desc" checked> Descending
                            </label>
                        </div>
                    </div>
                    <div class="tracker-models-panel">
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
                        <div class="models-grid" id="tracker-models-grid">
                            <!-- Model cards will be rendered here -->
                        </div>
                    </div>
                </div>
            </div>
        `;
    }

    setupEventListeners() {
        // Initial refresh button
        const initialRefresh = document.getElementById('tracker-initial-refresh');
        if (initialRefresh) {
            initialRefresh.addEventListener('click', () => this.refreshData());
        }

        // Refresh button
        const refreshBtn = document.getElementById('tracker-refresh');
        if (refreshBtn) {
            refreshBtn.addEventListener('click', () => this.refreshData());
        }

        // Export button
        const exportBtn = document.getElementById('tracker-export');
        if (exportBtn) {
            exportBtn.addEventListener('click', () => this.exportJson());
        }

        // Filter inputs
        const searchInput = document.getElementById('filter-search');
        if (searchInput) {
            searchInput.addEventListener('input', (e) => {
                this.currentFilters.search = e.target.value;
                this.applyFilters();
            });
        }

        const categorySelect = document.getElementById('filter-category');
        if (categorySelect) {
            categorySelect.addEventListener('change', (e) => {
                this.currentFilters.category = e.target.value;
                this.applyFilters();
            });
        }

        const chineseCheck = document.getElementById('filter-chinese');
        if (chineseCheck) {
            chineseCheck.addEventListener('change', (e) => {
                this.currentFilters.chineseOnly = e.target.checked;
                this.applyFilters();
            });
        }

        const ggufCheck = document.getElementById('filter-gguf');
        if (ggufCheck) {
            ggufCheck.addEventListener('change', (e) => {
                this.currentFilters.ggufOnly = e.target.checked;
                this.applyFilters();
            });
        }

        const vramInput = document.getElementById('filter-vram');
        if (vramInput) {
            vramInput.addEventListener('change', (e) => {
                this.currentFilters.vramLimit = parseFloat(e.target.value) || 24;
                this.applyFilters();
            });
        }

        const quantInput = document.getElementById('filter-quant');
        if (quantInput) {
            quantInput.addEventListener('input', (e) => {
                this.currentFilters.quant = e.target.value;
                this.applyFilters();
            });
        }

        // File type checkboxes
        const fileTypeCheckboxes = document.querySelectorAll('.filetype-checkbox');
        fileTypeCheckboxes.forEach(cb => {
            cb.addEventListener('change', () => {
                this.currentFilters.fileTypes = Array.from(fileTypeCheckboxes)
                    .filter(c => c.checked)
                    .map(c => c.value);
                this.applyFilters();
            });
        });

        const sortSelect = document.getElementById('filter-sort');
        if (sortSelect) {
            sortSelect.addEventListener('change', (e) => {
                this.currentFilters.sortBy = e.target.value;
                this.applyFilters();
            });
        }

        const sortDescCheck = document.getElementById('filter-sort-desc');
        if (sortDescCheck) {
            sortDescCheck.addEventListener('change', (e) => {
                this.currentFilters.sortDesc = e.target.checked;
                this.applyFilters();
            });
        }

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
    }

    async refreshData() {
        const loadingEl = document.getElementById('tracker-loading');
        const contentEl = document.getElementById('tracker-content');
        
        loadingEl.innerHTML = `
            <div class="loading-spinner"></div>
            <p>Fetching models from HuggingFace...</p>
        `;

        try {
            await window.__TAURI__.core.invoke('refresh_tracker_data');
            await this.applyFilters();
            
            loadingEl.classList.add('hidden');
            contentEl.classList.remove('hidden');
        } catch (error) {
            console.error('Error refreshing tracker data:', error);
            loadingEl.innerHTML = `
                <p style="color: #ff6b6b;">Error: ${error}</p>
                <button class="tracker-btn" id="tracker-retry">Retry</button>
            `;
            document.getElementById('tracker-retry')?.addEventListener('click', () => this.refreshData());
        }
    }

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

    hideLiveResults() {
        // Hide the live badge section
        const liveSection = document.getElementById('badge-live-section');
        if (liveSection) {
            liveSection.style.display = 'none';
        }
        
        // Remove any new/updated badges from model cards
        const newBadges = document.querySelectorAll('.badge.new, .badge.updated-badge');
        newBadges.forEach(badge => badge.remove());
    }

    async fetchLiveResults() {
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

    mergeLiveResults(liveModels) {
        const grid = document.getElementById('tracker-models-grid');
        if (!grid) return;
        
        // Clear any "Loading..." or "No models" messages
        const placeholder = grid.querySelector('.loading-placeholder, .no-models');
        if (placeholder) {
            placeholder.remove();
        }
        
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
        
        // Add new cards (prepend if existing cards, append if grid is empty)
        if (fragment.childNodes.length > 0) {
            if (existingCards.length > 0) {
                grid.insertBefore(fragment, grid.firstChild);
            } else {
                grid.appendChild(fragment);
            }
        }
        
        console.log(`Merged ${newCount} new models, updated ${updatedCount} existing`);
    }

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

    async applyFilters() {
        const grid = document.getElementById('tracker-models-grid');
        if (!grid) return;
        
        grid.innerHTML = '<div class="loading-placeholder">Loading...</div>';

        try {
            const filters = this.currentFilters;
            
            const models = await window.__TAURI__.core.invoke('get_tracker_models', {
                vramLimit: filters.vramLimit,
                categories: filters.category ? [filters.category] : null,
                chineseOnly: filters.chineseOnly,
                ggufOnly: filters.ggufOnly,
                fileTypes: filters.fileTypes.length > 0 ? filters.fileTypes : null,
                quantizations: filters.quant ? [filters.quant] : null,
                search: filters.search || null,
                sortBy: filters.sortBy,
                sortDesc: filters.sortDesc
            });
            
            // Get stats
            const stats = await window.__TAURI__.core.invoke('get_tracker_stats');
            
            this.renderStats(stats);
            this.renderModelCards(models);
            
            // Trigger live search from HF API to supplement local results
            if (this.liveSearchEnabled) {
                this.triggerLiveFetch();
            }
            
        } catch (error) {
            console.error('Error applying filters:', error);
            grid.innerHTML = `<div class="no-models">Error: ${error}</div>`;
        }
    }

    async exportJson() {
        try {
            const json = await window.__TAURI__.core.invoke('export_tracker_json');
            
            // Create and download file
            const blob = new Blob([json], { type: 'application/json' });
            const url = URL.createObjectURL(blob);
            const a = document.createElement('a');
            a.href = url;
            a.download = 'ai-models-export.json';
            a.click();
            URL.revokeObjectURL(url);
            
            this.desktop.showNotification('Models exported successfully!', 'success');
        } catch (error) {
            console.error('Error exporting:', error);
            this.desktop.showNotification('Error exporting models', 'error');
        }
    }

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

    renderStats(stats) {
        document.getElementById('stat-total').textContent = (stats.total_models || 0).toLocaleString();
        document.getElementById('stat-chinese').textContent = (stats.chinese_models || 0).toLocaleString();
        document.getElementById('stat-gguf').textContent = (stats.gguf_models || 0).toLocaleString();
        document.getElementById('stat-categories').textContent = stats.categories ? Object.keys(stats.categories).length : 0;
        
        // Update badge counts
        this.updateBadgeCounts(stats.total_models || 0, null, false);
    }

    renderModelCards(models) {
        const grid = document.getElementById('tracker-models-grid');
        
        if (!models || models.length === 0) {
            grid.innerHTML = '<div class="no-models">No models found matching your filters. Try clicking Refresh to fetch models.</div>';
            return;
        }

        grid.innerHTML = models.map(model => this.renderModelCard(model)).join('');
    }

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

    async viewOnHF(modelId) {
        try {
            const url = `https://huggingface.co/${modelId}`;
            await window.__TAURI__.core.invoke('open_url', { url });
        } catch (error) {
            console.error('Error opening HF link:', error);
            // Fallback: open in new tab
            window.open(`https://huggingface.co/${modelId}`, '_blank');
        }
    }

    escapeHtml(text) {
        if (!text) return '';
        const div = document.createElement('div');
        div.textContent = text;
        return div.innerHTML;
    }
}

// Make globally accessible
window.TrackerApp = TrackerApp;
