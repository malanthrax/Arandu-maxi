// Hugging Face Application Module
class HuggingFaceApp {
    constructor(desktop) {
        this.desktop = desktop;
        this.windowId = 'huggingface-search';
        // Initialize Tauri API access - defer until needed
        this.invoke = null;
        this.tauriInitialized = false;
        // Load suggestions from config file
        this.suggestions = [];
        this.loadSuggestions();
        // Don't call initTauriAPI here - wait until first use
        this.setupEventListeners();
        this.comparisonContext = null;
    }

    setComparisonContext(context) {
        this.comparisonContext = context;
    }

    clearComparisonContext() {
        this.comparisonContext = null;
    }

    async loadSuggestions() {
        try {
            const response = await fetch('huggingface-suggestions.json');
            const data = await response.json();
            this.suggestions = data.suggestions || [];
        } catch (error) {
            console.error('Failed to load suggestions:', error);
            // Fallback to default suggestions
            this.suggestions = ['qwen', 'minimax', 'glm', 'mistral'];
        }
    }

    generateSuggestionsHTML() {
        // Split suggestions into two rows: 4 in first row, remaining in second row
        const firstRow = this.suggestions.slice(0, 4);
        const secondRow = this.suggestions.slice(4);
        
        let html = '';
        
        if (firstRow.length > 0) {
            html += '<div class="suggestions-row">\n';
            html += firstRow.map(term => 
                `                        <button class="suggestion-btn" onclick="huggingFaceApp.quickSearch('${term}')">${term}</button>`
            ).join('\n');
            html += '\n                    </div>';
        }
        
        if (secondRow.length > 0) {
            html += '\n                    <div class="suggestions-row">\n';
            html += secondRow.map(term => 
                `                        <button class="suggestion-btn" onclick="huggingFaceApp.quickSearch('${term}')">${term}</button>`
            ).join('\n');
            html += '\n                    </div>';
        }
        
        return html;
    }

    generatePlaceholderHTML() {
        return `
            <div class="search-placeholder">
                <div class="search-placeholder-icon">Search</div>
                <p>Enter a search term and press Enter to find llama.cpp compatible text generation models</p>
                <div class="search-suggestions">
                    <span class="suggestion-label">Popular searches:</span>
                    <div class="suggestions-grid">
                    ${this.generateSuggestionsHTML()}
                    </div>
                </div>
            </div>
        `;
    }

    initTauriAPI() {
        if (this.tauriInitialized) return;

        try {
            if (window.__TAURI__ && window.__TAURI__.core) {
                this.invoke = window.__TAURI__.core.invoke;
                this.tauriInitialized = true;
                console.log('Tauri API initialized in HuggingFaceApp');
            } else {
                console.warn('Tauri API not available yet, will retry when needed');
                this.tauriInitialized = false;
            }
        } catch (error) {
            console.error('Failed to initialize Tauri API:', error);
            this.tauriInitialized = false;
        }
    }

    async waitForTauriAPI(maxAttempts = 10, delay = 500) {
        if (this.tauriInitialized) return true;

        for (let attempt = 1; attempt <= maxAttempts; attempt++) {
            console.log(`Attempting to initialize Tauri API (attempt ${attempt}/${maxAttempts})...`);
            this.initTauriAPI();

            if (this.tauriInitialized) {
                console.log('Tauri API initialized successfully');
                return true;
            }

            if (attempt < maxAttempts) {
                console.log(`Tauri API not ready, waiting ${delay}ms before retry...`);
                await new Promise(resolve => setTimeout(resolve, delay));
            }
        }

        console.error('Failed to initialize Tauri API after maximum attempts');
        return false;
    }

    setupEventListeners() {
        // Listen for file system changes to update download status
        if (window.__TAURI__ && window.__TAURI__.event) {
            // Listen for file deletion events
            window.__TAURI__.event.listen('file-deleted', () => {
                console.log('File deleted event received in HuggingFace app, refreshing download status...');
                this.refreshAllDownloadStatus();
            });

            // Listen for download completion events
            window.__TAURI__.event.listen('download-complete', () => {
                console.log('Download complete event received, refreshing download status...');
                setTimeout(() => {
                    this.refreshAllDownloadStatus();
                }, 1000); // Small delay to ensure file is fully written
            });
        }
    }

    getInvoke() {
        if (!this.tauriInitialized) {
            this.initTauriAPI();
        }
        return this.invoke;
    }

    isInitialized() {
        return this.tauriInitialized && this.invoke !== null;
    }

    async openHuggingFaceSearch() {
        // Check if Tauri API is available
        if (!this.isInitialized()) {
            console.warn('HuggingFace app not fully initialized, attempting to initialize...');

            // Try to wait for Tauri API to be ready
            const apiReady = await this.waitForTauriAPI();
            if (!apiReady) {
                this.desktop.showNotification('HuggingFace app failed to initialize Tauri API', 'error');
                return;
            }
        }

        // Mark dock icon as active
        const huggingfaceDockIcon = document.getElementById('huggingface-dock-icon');

        // Check if window already exists
        const existingWindow = this.desktop.windows.get(this.windowId);
        if (existingWindow) {
            // Toggle window visibility
            if (existingWindow.classList.contains('hidden')) {
                existingWindow.classList.remove('hidden');
                existingWindow.style.zIndex = ++this.desktop.windowZIndex;
                // Update focused state
                this.desktop.updateDockFocusedState(this.windowId);
                if (huggingfaceDockIcon) {
                    huggingfaceDockIcon.classList.add('active');
                }
            } else {
                existingWindow.classList.add('hidden');
                if (huggingfaceDockIcon) {
                    huggingfaceDockIcon.classList.remove('active');
                }
            }
            return;
        }

        // Mark dock icon as active for new window
        if (huggingfaceDockIcon) {
            huggingfaceDockIcon.classList.add('active');
        }

        const content = `
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
                    <!-- Search Tab -->
                    <div class="tab-panel active" id="tab-search">
                        <div class="huggingface-search-container">
                            <div class="search-section">
                                <div class="search-header">
                                    <div class="search-controls" style="position: relative;">
                                        <div style="position: relative; flex: 1; display: flex; align-items: center;">
                                            <input type="text" id="hf-search-input" class="search-input" placeholder="Search for models (e.g., llama, mistral, qwen, codellama)" autocomplete="off" style="flex: 1;">
                                            <button class="search-clear" id="hf-search-clear" title="Clear Search" style="position: absolute; right: 8px;">
                                                <span class="material-icons">close</span>
                                            </button>
                                        </div>
                                        <div class="limit-controls">
                                            <label for="hf-limit">Show:</label>
                                            <select id="hf-limit" class="limit-select">
                                                <option value="50">50</option>
                                                <option value="100" selected>100</option>
                                                <option value="250">250</option>
                                                <option value="500">500</option>
                                            </select>
                                        </div>
                                        <div class="sorting-controls">
                                            <label for="hf-sort-by">Sort by:</label>
                                            <select id="hf-sort-by" class="sort-select">
                                                <option value="relevance">Relevance</option>
                                                <option value="downloads">Most Downloads</option>
                                                <option value="likes">Most Likes</option>
                                                <option value="updated">Recently Updated</option>
                                            </select>
                                        </div>
                                        <div class="search-history-dropdown" id="hf-search-history-dropdown">
                                            <ul class="search-history-list" id="hf-search-history-list">
                                                <!-- History items will be populated here -->
                                            </ul>
                                            <div class="search-history-header">
                                                <span class="search-history-title">Recent Searches</span>
                                                <button class="search-history-clear" id="hf-search-history-clear-all">Clear All</button>
                                            </div>
                                        </div>
                                    </div>
                                </div>
                                
                                <div class="search-results" id="hf-search-results">
                                    ${this.generatePlaceholderHTML()}
                                </div>
                            </div>
                        </div>
                    </div>
                    
                    <!-- Paste Link Tab -->
                    <div class="tab-panel" id="tab-paste-link">
                        <div class="paste-link-container">
                            <!-- Step 1: URL Input -->
                            <div class="url-input-section">
                                <h3>Step 1: Enter Model URL</h3>
                                <div class="url-input-wrapper">
                                    <input type="text" id="hf-url-input" class="url-input" placeholder="https://huggingface.co/author/model" autocomplete="off">
                                    <button class="paste-btn" id="hf-paste-btn" title="Paste from clipboard">
                                        <span class="material-icons">content_paste</span>
                                    </button>
                                </div>
                                <button class="validate-btn" id="hf-validate-btn">
                                    <span class="material-icons">search</span>
                                    Validate & Fetch
                                </button>
                            </div>
                            
                            <!-- Step 2: Model Info -->
                            <div class="model-info-section" id="hf-model-info" style="display: none;">
                                <!-- Model info will be populated here -->
                            </div>
                            
                            <!-- Step 3: File Selection -->
                            <div class="file-selection-section" id="hf-files-section" style="display: none;">
                                <h3>Step 3: Select GGUF Files to Download</h3>
                                <div class="file-list-header">
                                    <button class="select-all-btn" id="hf-select-all">Select All</button>
                                    <button class="select-none-btn" id="hf-select-none">Deselect All</button>
                                </div>
                                <div class="file-list" id="hf-files-list">
                                    <!-- Files will be populated here -->
                                </div>
                                <div class="total-size-display" id="hf-total-size">
                                    Selected: 0 files (0 GB)
                                </div>
                            </div>
                            
                            <!-- Step 4: Destination -->
                            <div class="destination-section" id="hf-destination-section" style="display: none;">
                                <h3>Step 4: Choose Destination</h3>
                                <div class="destination-input-wrapper">
                                    <input type="text" id="hf-destination-path" class="destination-path" readonly>
                                    <button class="browse-btn" id="hf-browse-btn">
                                        <span class="material-icons">folder_open</span>
                                        Browse
                                    </button>
                                </div>
                                <div class="disk-space-info" id="hf-disk-space">
                                    Available space will be shown here
                                </div>
                            </div>
                            
                            <!-- Action Buttons -->
                            <div class="paste-link-actions" id="hf-actions" style="display: none;">
                                <button class="cancel-btn" id="hf-cancel-btn">Cancel</button>
                                <button class="download-btn" id="hf-download-btn">
                                    <span class="material-icons">download</span>
                                    Download Selected
                                </button>
                            </div>
                            
                            <!-- Download Progress -->
                            <div class="download-progress-section" id="hf-download-progress" style="display: none;">
                                <h3>Downloading...</h3>
                                <div class="current-file" id="hf-current-file"></div>
                                <div class="progress-bar-container">
                                    <div class="progress-bar" id="hf-progress-bar">
                                        <div class="progress-fill" id="hf-progress-fill"></div>
                                    </div>
                                    <div class="progress-text" id="hf-progress-text">0%</div>
                                </div>
                                <div class="download-stats" id="hf-download-stats"></div>
                                <div class="download-actions">
                                    <button class="pause-btn" id="hf-pause-btn">Pause</button>
                                    <button class="cancel-btn" id="hf-cancel-download-btn">Cancel</button>
                                </div>
                                <div class="queue-list" id="hf-queue-list"></div>
                            </div>
                        </div>
                    </div>
                </div>
            </div>
        `;

        const window = this.desktop.createWindow(this.windowId, 'Hugging Face Models', 'huggingface-search-window', content);
        window.style.width = '900px';
        window.style.height = '700px';
        // Remove the transform and use absolute positioning for centering
        window.style.transform = '';
        window.style.left = 'calc(50% - 450px)';
        window.style.top = 'calc(50% - 350px)';

        // Add event listeners
        this.setupHuggingFaceSearchListeners();

        // Don't add to taskbar - use permanent dock icon instead
    }

    setupHuggingFaceSearchListeners() {
        const window = this.desktop.windows.get(this.windowId);
        if (!window) return;

        const searchInput = window.querySelector('#hf-search-input');
        const sortBySelect = window.querySelector('#hf-sort-by');
        const limitSelect = window.querySelector('#hf-limit');

        // Toggle dropdown when clicking the input
        searchInput.addEventListener('click', (e) => {
            e.stopPropagation();
            const dropdown = window.querySelector('#hf-search-history-dropdown');
            if (dropdown) {
                this.updateHfSearchHistoryDropdown(searchInput.value);
                if (dropdown.classList.contains('show')) {
                    dropdown.classList.remove('show');
                } else if (typeof searchHistory !== 'undefined' && searchHistory.hasHistory()) {
                    // Set dropdown width to match input width
                    const inputWidth = searchInput.offsetWidth;
                    dropdown.style.width = `${inputWidth}px`;
                    dropdown.classList.add('show');
                }
            }
        });

        // Hide dropdown when typing
        searchInput.addEventListener('input', (e) => {
            const dropdown = window.querySelector('#hf-search-history-dropdown');
            if (dropdown) {
                // Update dropdown with filtered results but keep it shown if it was already shown
                this.updateHfSearchHistoryDropdown(e.target.value);
                
                // Only show dropdown if there are matching results and input is not empty
                if (e.target.value.trim() !== '' && (typeof searchHistory !== 'undefined' && searchHistory.hasHistory())) {
                    dropdown.classList.add('show');
                } else {
                    dropdown.classList.remove('show');
                }
            }
        });

        // Hide dropdown when clicking outside
        searchInput.addEventListener('blur', () => {
            // Delay to allow click on history items
            setTimeout(() => {
                const dropdown = window.querySelector('#hf-search-history-dropdown');
                if (dropdown) {
                    dropdown.classList.remove('show');
                }
            }, 200);
        });

        // Enter key in search input
        searchInput.addEventListener('keypress', (e) => {
            if (e.key === 'Enter') {
                this.performHuggingFaceSearch();
            }
        });

        // Clear search button
        const searchClearBtn = window.querySelector('#hf-search-clear');
        if (searchClearBtn) {
            searchClearBtn.addEventListener('click', () => {
                searchInput.value = '';
                searchInput.focus();
                // Reset to initial page with suggestions
                const resultsContainer = window.querySelector('#hf-search-results');
                if (resultsContainer) {
                    resultsContainer.innerHTML = this.generatePlaceholderHTML();
                }
            });
        }

        // Search history dropdown
        const searchHistoryList = window.querySelector('#hf-search-history-list');
        if (searchHistoryList) {
            searchHistoryList.addEventListener('click', (e) => {
                const item = e.target.closest('.search-history-item');
                if (item) {
                    const deleteBtn = e.target.closest('.search-history-delete');
                    if (deleteBtn) {
                        // Delete individual history item
                        const term = deleteBtn.dataset.term;
                        if (typeof searchHistory !== 'undefined') {
                            searchHistory.removeSearch(term);
                            this.updateHfSearchHistoryDropdown();
                        }
                    } else {
                        // Click on history item - populate search
                        const term = item.dataset.term;
                        searchInput.value = term;
                        this.performHuggingFaceSearch();
                        const dropdown = window.querySelector('#hf-search-history-dropdown');
                        if (dropdown) {
                            dropdown.classList.remove('show');
                        }
                    }
                }
            });
        }

        // Clear all history button
        const clearAllBtn = window.querySelector('#hf-search-history-clear-all');
        if (clearAllBtn) {
            clearAllBtn.addEventListener('click', () => {
                if (typeof searchHistory !== 'undefined') {
                    searchHistory.clearHistory();
                    this.updateHfSearchHistoryDropdown();
                }
            });
        }

        // Sort by change
        sortBySelect.addEventListener('change', () => {
            this.performHuggingFaceSearch();
        });

        // Limit change
        limitSelect.addEventListener('change', () => {
            this.performHuggingFaceSearch();
        });

        // Focus search input
        setTimeout(() => searchInput.focus(), 100);

        // Setup paste link tab listeners
        this.setupPasteLinkListeners();
        this.setupTabSwitching();
    }

    setupTabSwitching() {
        const window = this.desktop.windows.get(this.windowId);
        if (!window) return;

        const tabBtns = window.querySelectorAll('.tab-btn');
        tabBtns.forEach(btn => {
            btn.addEventListener('click', (e) => {
                const tabName = e.currentTarget.dataset.tab;
                this.switchTab(tabName);
            });
        });
    }

    switchTab(tabName) {
        const window = this.desktop.windows.get(this.windowId);
        if (!window) return;

        // Update button states
        window.querySelectorAll('.tab-btn').forEach(btn => {
            btn.classList.toggle('active', btn.dataset.tab === tabName);
        });

        // Update panel visibility
        window.querySelectorAll('.tab-panel').forEach(panel => {
            panel.classList.toggle('active', panel.id === `tab-${tabName}`);
        });
    }

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
                this.showNotification('Failed to read clipboard', 'error');
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

        // Browse button for destination
        const browseBtn = window.querySelector('#hf-browse-btn');
        browseBtn?.addEventListener('click', () => {
            this.browseDestinationFolder();
        });

        // Download button
        const downloadBtn = window.querySelector('#hf-download-btn');
        downloadBtn?.addEventListener('click', () => {
            this.startPasteLinkDownload();
        });

        // Cancel button
        const cancelBtn = window.querySelector('#hf-cancel-btn');
        cancelBtn?.addEventListener('click', () => {
            this.resetPasteLinkTab();
        });
    }

    validateUrl(url) {
        const window = this.desktop.windows.get(this.windowId);
        if (!window) return false;

        const input = window.querySelector('#hf-url-input');
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

    async handleUrlValidation() {
        const window = this.desktop.windows.get(this.windowId);
        if (!window) return;

        const urlInput = window.querySelector('#hf-url-input');
        const url = urlInput?.value?.trim();

        if (!url) {
            this.showNotification('Please enter a HuggingFace model URL', 'error');
            return;
        }

        if (!this.validateUrl(url)) {
            this.showNotification('Invalid URL format. Use: https://huggingface.co/author/model', 'error');
            return;
        }

        // Show loading state
        const validateBtn = window.querySelector('#hf-validate-btn');
        if (validateBtn) {
            validateBtn.disabled = true;
            validateBtn.innerHTML = '<span class="material-icons spinning">refresh</span> Loading...';
        }

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
            await this.setDefaultDestination(modelId);

            // Show file selection and destination sections
            window.querySelector('#hf-files-section').style.display = 'block';
            window.querySelector('#hf-destination-section').style.display = 'block';
            window.querySelector('#hf-actions').style.display = 'flex';

        } catch (error) {
            console.error('Failed to fetch model:', error);
            this.showNotification(`Failed to fetch model: ${error}`, 'error');
        } finally {
            // Reset button
            if (validateBtn) {
                validateBtn.disabled = false;
                validateBtn.innerHTML = '<span class="material-icons">search</span> Validate & Fetch';
            }
        }
    }

    displayModelInfo(modelInfo) {
        const window = this.desktop.windows.get(this.windowId);
        if (!window) return;

        const container = window.querySelector('#hf-model-info');
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
                        <span class="model-license">
                            <span class="material-icons">description</span>
                            License: ${this.escapeHtml(modelInfo.license)}
                        </span>
                    ` : ''}
                    ${modelInfo.tags?.length ? `
                        <span class="model-tags">
                            <span class="material-icons">label</span>
                            ${modelInfo.tags.slice(0, 5).map(t => this.escapeHtml(t)).join(', ')}
                        </span>
                    ` : ''}
                </div>
                <div class="model-stats">
                    ${modelInfo.downloads ? `
                        <span class="stat-item">
                            <span class="material-icons">download</span>
                            ${this.formatNumber(modelInfo.downloads)} downloads
                        </span>
                    ` : ''}
                    ${modelInfo.likes ? `
                        <span class="stat-item">
                            <span class="material-icons">favorite</span>
                            ${this.formatNumber(modelInfo.likes)} likes
                        </span>
                    ` : ''}
                </div>
            </div>
        `;

        container.style.display = 'block';
    }

    displayFilesList(files) {
        const window = this.desktop.windows.get(this.windowId);
        if (!window) return;

        const container = window.querySelector('#hf-files-list');
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
            <div class="file-item" data-filename="${this.escapeHtml(file.filename)}" data-size="${file.size}">
                <input type="checkbox" class="file-checkbox" id="file-${index}" 
                       data-filename="${this.escapeHtml(file.filename)}"
                       data-size="${file.size}">
                <label for="file-${index}" class="file-label">
                    <span class="file-name">${this.escapeHtml(file.filename)}</span>
                    <span class="file-size">${file.size_formatted}</span>
                    ${file.quantization ? `<span class="file-badge">${this.escapeHtml(file.quantization)}</span>` : ''}
                </label>
            </div>
        `).join('');

        container.innerHTML = filesHtml;

        // Setup file selection listeners
        this.setupFileSelectionListeners();
    }

    setupFileSelectionListeners() {
        const window = this.desktop.windows.get(this.windowId);
        if (!window) return;

        const container = window.querySelector('#hf-files-list');
        if (!container) return;

        // Individual file checkboxes
        container.querySelectorAll('.file-checkbox').forEach(checkbox => {
            checkbox.addEventListener('change', () => {
                this.updateTotalSize();
            });
        });

        // Select all
        window.querySelector('#hf-select-all')?.addEventListener('click', () => {
            container.querySelectorAll('.file-checkbox').forEach(cb => {
                cb.checked = true;
            });
            this.updateTotalSize();
        });

        // Deselect all
        window.querySelector('#hf-select-none')?.addEventListener('click', () => {
            container.querySelectorAll('.file-checkbox').forEach(cb => {
                cb.checked = false;
            });
            this.updateTotalSize();
        });
    }

    updateTotalSize() {
        const window = this.desktop.windows.get(this.windowId);
        if (!window) return;

        const checkboxes = window.querySelectorAll('.file-checkbox:checked');
        let totalSize = 0;
        let count = 0;

        checkboxes.forEach(cb => {
            totalSize += parseInt(cb.dataset.size) || 0;
            count++;
        });

        const display = window.querySelector('#hf-total-size');
        if (display) {
            display.textContent = `Selected: ${count} file${count !== 1 ? 's' : ''} (${this.formatBytes(totalSize)})`;
        }
    }

    async setDefaultDestination(modelId) {
        const invoke = this.getInvoke();
        if (!invoke) return;

        const window = this.desktop.windows.get(this.windowId);
        if (!window) return;

        try {
            const defaultPath = await invoke('get_default_download_path', { modelId });
            const pathInput = window.querySelector('#hf-destination-path');
            if (pathInput) {
                pathInput.value = defaultPath;
            }
        } catch (error) {
            console.error('Failed to get default path:', error);
        }
    }

    async browseDestinationFolder() {
        const invoke = this.getInvoke();
        if (!invoke) return;

        const window = this.desktop.windows.get(this.windowId);
        if (!window) return;

        try {
            const selected = await invoke('browse_folder', { inputId: 'hf-destination-path' });
            if (selected) {
                const pathInput = window.querySelector('#hf-destination-path');
                if (pathInput) {
                    pathInput.value = selected;
                }
            }
        } catch (error) {
            console.error('Failed to browse folder:', error);
        }
    }

    async startPasteLinkDownload() {
        const window = this.desktop.windows.get(this.windowId);
        if (!window) return;

        const checkboxes = window.querySelectorAll('.file-checkbox:checked');
        const selectedFiles = Array.from(checkboxes).map(cb => cb.dataset.filename);

        if (selectedFiles.length === 0) {
            this.showNotification('Please select at least one file to download', 'error');
            return;
        }

        const destination = window.querySelector('#hf-destination-path')?.value;
        if (!destination) {
            this.showNotification('Please select a destination folder', 'error');
            return;
        }

        const urlInput = window.querySelector('#hf-url-input');
        const url = urlInput?.value;

        const invoke = this.getInvoke();
        if (!invoke) {
            this.showNotification('Tauri API not available', 'error');
            return;
        }

        try {
            const modelId = await invoke('parse_hf_url', { url });

            // Show download progress UI
            window.querySelector('#hf-files-section').style.display = 'none';
            window.querySelector('#hf-destination-section').style.display = 'none';
            window.querySelector('#hf-actions').style.display = 'none';
            window.querySelector('#hf-model-info').style.display = 'none';
            window.querySelector('#hf-download-progress').style.display = 'block';

            // Download files one at a time
            for (let i = 0; i < selectedFiles.length; i++) {
                const filename = selectedFiles[i];
                this.updateDownloadProgress(filename, i + 1, selectedFiles.length);

                try {
                    await invoke('download_hf_file', {
                        modelId,
                        filename,
                        destination: `${destination}/${filename}`
                    });

                    this.markFileComplete(filename);
                } catch (error) {
                    console.error(`Failed to download ${filename}:`, error);
                    this.showNotification(`Failed to download ${filename}: ${error}`, 'error');
                }
            }

            this.showDownloadComplete();

        } catch (error) {
            console.error('Download error:', error);
            this.showNotification(`Download failed: ${error}`, 'error');
        }
    }

    updateDownloadProgress(filename, current, total) {
        const window = this.desktop.windows.get(this.windowId);
        if (!window) return;

        const currentFileEl = window.querySelector('#hf-current-file');
        if (currentFileEl) {
            currentFileEl.textContent = `File ${current} of ${total}: ${filename}`;
        }
    }

    markFileComplete(filename) {
        const window = this.desktop.windows.get(this.windowId);
        if (!window) return;

        const queueList = window.querySelector('#hf-queue-list');
        if (queueList) {
            const item = document.createElement('div');
            item.className = 'queue-item completed';
            item.innerHTML = `
                <span class="material-icons">check_circle</span>
                <span class="queue-filename">${this.escapeHtml(filename)}</span>
                <span class="queue-status">Completed</span>
            `;
            queueList.appendChild(item);
        }
    }

    showDownloadComplete() {
        const window = this.desktop.windows.get(this.windowId);
        if (!window) return;

        const progressSection = window.querySelector('#hf-download-progress');
        if (progressSection) {
            progressSection.innerHTML = `
                <div class="success-message">
                    <span class="material-icons">check_circle</span>
                    <h3>Download Complete!</h3>
                    <p>All files have been downloaded successfully.</p>
                    <button class="action-btn" id="hf-download-more-btn">Download More</button>
                </div>
            `;

            window.querySelector('#hf-download-more-btn')?.addEventListener('click', () => {
                this.resetPasteLinkTab();
            });
        }

        this.showNotification('Download complete!', 'success');
    }

    resetPasteLinkTab() {
        const window = this.desktop.windows.get(this.windowId);
        if (!window) return;

        // Reset all fields
        window.querySelector('#hf-url-input').value = '';
        window.querySelector('#hf-url-input').classList.remove('valid', 'error');
        window.querySelector('#hf-model-info').style.display = 'none';
        window.querySelector('#hf-model-info').innerHTML = '';
        window.querySelector('#hf-files-section').style.display = 'none';
        window.querySelector('#hf-files-list').innerHTML = '';
        window.querySelector('#hf-destination-section').style.display = 'none';
        window.querySelector('#hf-destination-path').value = '';
        window.querySelector('#hf-actions').style.display = 'none';
        window.querySelector('#hf-download-progress').style.display = 'none';
        window.querySelector('#hf-total-size').textContent = 'Selected: 0 files (0 GB)';

        // Reset download progress section
        window.querySelector('#hf-download-progress').innerHTML = `
            <h3>Downloading...</h3>
            <div class="current-file" id="hf-current-file"></div>
            <div class="progress-bar-container">
                <div class="progress-bar" id="hf-progress-bar">
                    <div class="progress-fill" id="hf-progress-fill"></div>
                </div>
                <div class="progress-text" id="hf-progress-text">0%</div>
            </div>
            <div class="download-stats" id="hf-download-stats"></div>
            <div class="download-actions">
                <button class="pause-btn" id="hf-pause-btn">Pause</button>
                <button class="cancel-btn" id="hf-cancel-download-btn">Cancel</button>
            </div>
            <div class="queue-list" id="hf-queue-list"></div>
        `;
    }

    showNotification(message, type = 'info') {
        if (this.desktop && this.desktop.showNotification) {
            this.desktop.showNotification(message, type);
        } else {
            console.log(`[${type}] ${message}`);
        }
    }

    formatBytes(bytes) {
        if (bytes === 0) return '0 B';
        const k = 1024;
        const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
        const i = Math.floor(Math.log(bytes) / Math.log(k));
        return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + ' ' + sizes[i];
    }

    updateHfSearchHistoryDropdown(filterTerm = null) {
        const window = this.desktop.windows.get(this.windowId);
        if (!window) return;

        const historyList = window.querySelector('#hf-search-history-list');
        if (!historyList) return;

        if (typeof searchHistory === 'undefined') {
            historyList.innerHTML = '<li class="search-history-empty">No recent searches</li>';
            return;
        }

        let history = searchHistory.getHistory(30); // Show up to 30 items
        
        // Filter history based on the current search term if provided
        if (filterTerm && filterTerm.trim() !== '') {
            const lowerFilterTerm = filterTerm.toLowerCase().trim();
            history = history.filter(term =>
                term.toLowerCase().includes(lowerFilterTerm)
            );
        } else {
            // If no filter, limit to 10 for display purposes
            history = history.slice(0, 10);
        }

        if (history.length === 0) {
            historyList.innerHTML = '<li class="search-history-empty">No matching searches</li>';
            return;
        }

        historyList.innerHTML = history.map(term => `
            <li class="search-history-item" data-term="${this.escapeHtml(term)}">
                <span class="search-history-text">${this.escapeHtml(term)}</span>
                <button class="search-history-delete" data-term="${this.escapeHtml(term)}" title="Remove">
                    <span class="material-icons">close</span>
                </button>
            </li>
        `).join('');
    }

    escapeHtml(text) {
        const div = document.createElement('div');
        div.textContent = text;
        return div.innerHTML;
    }

    // Helper function to format time ago
    formatTimeAgo(dateString) {
        if (!dateString) {
            console.log('formatTimeAgo: dateString is null/undefined');
            return 'Unknown';
        }

        try {
            const date = new Date(dateString);

            // Check if date is valid
            if (isNaN(date.getTime())) {
                console.log('formatTimeAgo: Invalid date:', dateString);
                return 'Unknown';
            }

            const now = new Date();
            const diffInMs = now - date;
            const diffInDays = Math.floor(diffInMs / (1000 * 60 * 60 * 24));

            if (diffInDays < 0) {
                console.log('formatTimeAgo: Future date detected:', dateString);
                return 'Recently';
            }

            if (diffInDays === 0) return 'Today';
            if (diffInDays === 1) return '1 day ago';
            if (diffInDays < 7) return `${diffInDays} days ago`;
            if (diffInDays < 30) return `${Math.floor(diffInDays / 7)} weeks ago`;
            if (diffInDays < 365) return `${Math.floor(diffInDays / 30)} months ago`;
            return `${Math.floor(diffInDays / 365)} years ago`;
        } catch (error) {
            console.error('formatTimeAgo error:', error, 'dateString:', dateString);
            return 'Unknown';
        }
    }

    formatFileSize(bytes) {
        return this.desktop.formatFileSize(bytes);
    }

    formatNumber(num) {
        return this.desktop.formatNumber(num);
    }

    async performHuggingFaceSearch() {
        const window = this.desktop.windows.get(this.windowId);
        if (!window) return;

        const searchInput = window.querySelector('#hf-search-input');
        const resultsContainer = window.querySelector('#hf-search-results');
        const sortBySelect = window.querySelector('#hf-sort-by');
        const limitSelect = window.querySelector('#hf-limit');

        const query = searchInput.value.trim();
        if (!query) {
            this.desktop.showNotification('Please enter a search term', 'error');
            return;
        }

        // Save to search history
        if (typeof searchHistory !== 'undefined') {
            searchHistory.addSearch(query);
        }

        // Show loading state
        searchInput.disabled = true;
        resultsContainer.innerHTML = `
            <div class="search-loading">
                <div class="loading-spinner"></div>
                <p>Searching Hugging Face for "${query}"...</p>
            </div>
        `;

        try {
            const invoke = this.getInvoke();
            if (!invoke) {
                throw new Error('Tauri API not available - invoke is null');
            }

            console.log('Searching with query:', query);
            console.log('Limit:', limitSelect.value);
            console.log('Sort by:', sortBySelect.value);

            const result = await invoke('search_huggingface', {
                query: query,
                limit: parseInt(limitSelect.value),
                sortBy: sortBySelect.value
            });

            console.log('Raw result:', result);
            console.log('Result type:', typeof result);
            console.log('Has models property:', 'models' in result);
            console.log('Models is array:', Array.isArray(result.models));
            console.log('Models count:', result.models?.length);
            console.log('Models data:', result.models);

            if (!result.models || !Array.isArray(result.models)) {
                throw new Error('Invalid response format from backend');
            }
            
            if (result.models.length === 0) {
                console.warn('Empty models array returned');
            } else {
                console.log('First model ID:', result.models[0]?.id);
                console.log('All model IDs:', result.models.map(m => m.id));
            }

            this.displayHuggingFaceResults(result.models, query);

        } catch (error) {
            console.error('Search error:', error);
            resultsContainer.innerHTML = `
                <div class="search-error">
                    <div class="error-icon">Error</div>
                    <h4>Search Failed</h4>
                    <p>${error.message}</p>
                    <button onclick="huggingFaceApp.performHuggingFaceSearch()" class="retry-btn">Try Again</button>
                </div>
            `;
            this.desktop.showNotification('Search failed: ' + error.message, 'error');
        } finally {
            searchInput.disabled = false;
        }
    }

    displayHuggingFaceResults(models, query) {
        const window = this.desktop.windows.get(this.windowId);
        if (!window) return;

        const resultsContainer = window.querySelector('#hf-search-results');

        // DEBUG: Show raw data
        const debugInfo = `
            <div class="debug-info" style="background: #ffeb3b; color: #000; padding: 10px; margin: 10px; border-radius: 4px; font-family: monospace; font-size: 12px;">
                <strong>DEBUG INFO:</strong><br>
                Query: "${query}"<br>
                Models received: ${models?.length || 0}<br>
                Models is array: ${Array.isArray(models)}<br>
                ${models && models.length > 0 ? `First model ID: ${models[0].id}<br>All IDs: ${models.map(m => m.id).join(', ')}` : 'No models data'}
            </div>
        `;

        if (!models || models.length === 0) {
            resultsContainer.innerHTML = debugInfo + `
                <div class="search-no-results">
                    <div class="no-results-icon">No Results</div>
                    <h4>No Models Found</h4>
                    <p>No llama.cpp compatible text generation models found for "${query}"</p>
                    <p>Try different search terms like 'llama', 'mistral', 'qwen', or 'codellama'</p>
                </div>
            `;
            return;
        }

        // Create the two-panel layout with simplified list
        const resultsHTML = debugInfo + `
            <div class="search-results-header">
                <h4>Found ${models.length} llama.cpp compatible text generation models for "${query}"</h4>
            </div>
            <div class="search-results-content">
                <div class="models-sidebar">
                    <div class="models-list">
                        ${models.map((model, index) => {
            const updatedText = model.lastModified ? this.formatTimeAgo(model.lastModified) : 'Unknown';

            return `
                                <div class="model-list-item ${index === 0 ? 'selected' : ''}" data-model-index="${index}" onclick="huggingFaceApp.selectModel(${index})">
                                    <div class="model-list-name">${model.name}</div>
                                    <div class="model-list-author">by ${model.author}</div>
                                    <div class="model-list-stats">
                                        <span class="stat-downloads" title="${this.formatNumber(model.downloads)} downloads">⬇ ${this.formatNumber(model.downloads)}</span>
                                        <span class="stat-likes" title="${this.formatNumber(model.likes)} likes">❤ ${this.formatNumber(model.likes)}</span>
                                        <span class="stat-updated" title="Last updated: ${model.lastModified || 'Unknown'}">${updatedText}</span>
                                    </div>
                                </div>
                            `;
        }).join('')}
                    </div>
                </div>
                <div class="model-details-panel">
                    <div id="model-details-content">
                        <div class="model-detail-placeholder">
                            <h3>Select a model to view details</h3>
                            <p>Click on a model from the list to see detailed information, available GGUF files, and download options.</p>
                        </div>
                    </div>
                </div>
            </div>
        `;

        resultsContainer.innerHTML = resultsHTML;

        // Store models data for selection
        window._modelsData = models;

        // Auto-select first model if available
        if (models.length > 0) {
            setTimeout(() => this.selectModel(0), 100);
        }
    }

    selectModel(index) {
        const window = this.desktop.windows.get(this.windowId);
        if (!window || !window._modelsData) return;

        // Update selected item in list
        window.querySelectorAll('.model-list-item').forEach((item, i) => {
            item.classList.toggle('selected', i === index);
        });

        // Get the basic model info
        const basicModel = window._modelsData[index];
        const detailsContent = window.querySelector('#model-details-content');

        // Show basic info immediately with loading state
        detailsContent.innerHTML = `
            <div class="model-detail-header">
                <div class="model-header-top">
                    <h3 class="model-detail-name">${basicModel.name}</h3>
                    <button class="model-page-btn" onclick="desktop.openUrl('https://huggingface.co/${basicModel.id}')" title="Open model page on Hugging Face">
                        View on HF
                    </button>
                </div>
                <div class="model-detail-meta">
                    <span class="model-detail-author">by ${basicModel.author}</span>
                </div>
            </div>
            
            <div class="model-stats-compact">
                <div class="stat-row">
                    <div class="stat-compact">
                        <span class="stat-number">${this.formatNumber(basicModel.downloads)}</span>
                        <span class="stat-label">Downloads</span>
                    </div>
                    <div class="stat-compact">
                        <span class="stat-number">${this.formatNumber(basicModel.likes)}</span>
                        <span class="stat-label">Likes</span>
                    </div>
                    <div class="stat-compact">
                        <span class="stat-number">Loading...</span>
                        <span class="stat-label">Files</span>
                    </div>
                    <div class="stat-compact">
                        <span class="stat-number">Loading...</span>
                        <span class="stat-label">GGUF Files</span>
                    </div>
                </div>
            </div>
            
            <div class="model-detail-download">
                <h4>Available GGUF Files</h4>
                <div class="quantizations-list">
                    <div class="quant-item">
                        <div class="quant-info">
                            <span class="quant-name">Loading...</span>
                            <span class="quant-size"></span>
                        </div>
                        <button class="quant-download-btn" disabled data-status="unknown">
                            Loading
                        </button>
                    </div>
                </div>
            </div>
        `;

        // Fetch detailed model information in the background
        this.fetchModelDetails(basicModel.id, index)
            .then(detailedModel => {
                // Update details panel with full information only if still selected
                const currentSelected = window.querySelector('.model-list-item.selected');
                if (currentSelected && parseInt(currentSelected.dataset.modelIndex) === index) {
                    detailsContent.innerHTML = this.generateModelDetails(detailedModel, index);

                    // Store model data for download access
                    detailsContent.modelData = detailedModel;

                    // Check download status for each GGUF file
                    this.updateFileDownloadStatus(detailedModel);
                }
            })
            .catch(error => {
                console.error('Error fetching model details:', error);
                // Only show error if this model is still selected
                const currentSelected = window.querySelector('.model-list-item.selected');
                if (currentSelected && parseInt(currentSelected.dataset.modelIndex) === index) {
                    detailsContent.innerHTML = `
                        <div class="model-detail-error">
                            <h3>${basicModel.name}</h3>
                            <p>Error loading model details: ${error.message}</p>
                            <p>This model may not have GGUF files available.</p>
                            <button onclick="huggingFaceApp.selectModel(${index})" class="retry-btn">Retry</button>
                        </div>
                    `;
                }
            });
    }

    async fetchModelDetails(modelId) {
        // Check cache first
        const window = this.desktop.windows.get(this.windowId);
        if (!window._modelDetailsCache) {
            window._modelDetailsCache = new Map();
        }

        // Return cached data if available
        if (window._modelDetailsCache.has(modelId)) {
            return window._modelDetailsCache.get(modelId);
        }

        try {
            const invoke = this.getInvoke();
            if (!invoke) {
                throw new Error('Tauri API not available');
            }

            const result = await invoke('get_model_details', {
                modelId: modelId
            });

            // Cache the result
            window._modelDetailsCache.set(modelId, result);

            return result;

        } catch (error) {
            console.error('Error fetching model details:', error);
            throw error;
        }
    }

    generateModelDetails(model, index) {
        const fileTree = this.buildFileTree(model.gguf_files);
        const fileItems = this.generateFileTreeHTML(fileTree, model, index);

        return `
            <div class="model-detail-header">
                <div class="model-header-top">
                    <h3 class="model-detail-name">${model.name}</h3>
                    <button class="model-page-btn" onclick="desktop.openUrl('https://huggingface.co/${model.id}')" title="Open model page on Hugging Face">
                        View on HF
                    </button>
                </div>
                <div class="model-detail-meta">
                    <span class="model-detail-author">by ${model.author}</span>
                </div>
            </div>
            
            <div class="model-stats-compact">
                <div class="stat-row">
                    <div class="stat-compact">
                        <span class="stat-number">${this.formatNumber(model.downloads)}</span>
                        <span class="stat-label">Downloads</span>
                    </div>
                    <div class="stat-compact">
                        <span class="stat-number">${this.formatNumber(model.likes)}</span>
                        <span class="stat-label">Likes</span>
                    </div>
                    <div class="stat-compact">
                        <span class="stat-number">${model.total_files}</span>
                        <span class="stat-label">Files</span>
                    </div>
                    <div class="stat-compact">
                        <span class="stat-number">${Object.keys(model.gguf_files).length}</span>
                        <span class="stat-label">GGUF Files</span>
                    </div>
                </div>
            </div>
            
            ${model.description ? `
            <div class="model-detail-description">
                <p>${model.description}</p>
            </div>
            ` : ''}
            
            <div class="model-detail-download">
                <h4>Available GGUF Files</h4>
                <div class="quantizations-list">
                    ${fileItems}
                </div>
            </div>
        `;
    }

    buildFileTree(gguf_files) {
        const tree = { files: [], children: {} };

        for (const fileData of Object.values(gguf_files)) {
            const pathParts = fileData.path.split('/');
            let currentNode = tree;

            for (let i = 0; i < pathParts.length - 1; i++) {
                const part = pathParts[i];
                if (!currentNode.children[part]) {
                    currentNode.children[part] = { files: [], children: {} };
                }
                currentNode = currentNode.children[part];
            }

            currentNode.files.push(fileData);
        }

        return tree;
    }

    generateFileTreeHTML(node, model, index, level = 0) {
        let html = '';

        // Render child directories first
        for (const dirName of Object.keys(node.children).sort()) {
            const childNode = node.children[dirName];
            html += `
                <div class="quant-folder" style="padding-left: ${(level * 20) + 12}px;" onclick="this.classList.toggle('open')">
                    <span class="folder-icon"></span>
                    <span class="folder-name">${dirName}</span>
                </div>
                <div class="folder-content">
                    ${this.generateFileTreeHTML(childNode, model, index, level + 1)}
                </div>
            `;
        }

        // Then render files in the current directory
        if (node.files.length > 0) {
            if (level === 0) html += '<div class="folder-content" style="display: block;">';

            for (const fileData of node.files.sort((a, b) => a.size - b.size)) {
                const size = this.formatFileSize(fileData.size);
                const sizeText = size !== 'Unknown size' ? ` (${size})` : '';
                const displayName = fileData.filename.replace(/\.gguf$/i, '');

                html += `
                    <div class="quant-item" data-filename="${fileData.filename}" data-model-id="${model.id}" style="padding-left: ${(level * 20) + 12}px;">
                        <div class="quant-info">
                            <span class="quant-name">${displayName}</span>
                            <span class="quant-size">${sizeText}</span>
                        </div>
                        <button class="quant-download-btn" onclick="huggingFaceApp.downloadFile('${model.id}', '${fileData.filename}', ${index})" data-status="unknown">
                            Download
                        </button>
                    </div>
                `;
            }

            if (level === 0) html += '</div>';
        }

        return html;
    }

    async downloadFile(modelId, filename, index) {
        const window = this.desktop.windows.get(this.windowId);
        if (!window) return;

        // Find and check the download button state
        const fileItem = window.querySelector(`[data-filename="${filename}"][data-model-id="${modelId}"]`);
        const downloadBtn = fileItem?.querySelector('.quant-download-btn');

        // Don't allow download if file is already downloaded
        if (downloadBtn && downloadBtn.dataset.status === 'downloaded') {
            this.desktop.showNotification('File already downloaded', 'info');
            return;
        }

        // Get model data from the details content
        const detailsContent = window.querySelector('#model-details-content');
        const modelData = detailsContent.modelData;

        if (!modelData || !modelData.gguf_files[filename]) {
            this.desktop.showNotification('Error: file data not found', 'error');
            return;
        }

        // Double-check if file exists before starting download
        const invoke = this.getInvoke();
        if (!invoke) {
            this.desktop.showNotification('Error: Tauri API not available', 'error');
            return;
        }

        try {
            const fileExists = await invoke('check_file_exists', {
                modelId: modelId,
                filename: filename
            });

            if (fileExists) {
                this.desktop.showNotification('File already exists', 'info');
                // Update button state
                if (downloadBtn) {
                    downloadBtn.innerHTML = 'Downloaded';
                    downloadBtn.dataset.status = 'downloaded';
                    downloadBtn.disabled = true;
                    downloadBtn.classList.add('downloaded');
                }
                return;
            }
        } catch (error) {
            console.error('Error checking file existence:', error);
        }

        const fileData = modelData.gguf_files[filename];
        const files = [filename]; // Just the single file

        if (downloadBtn) {
            // Disable the button and show downloading state
            downloadBtn.disabled = true;
            downloadBtn.innerHTML = 'Downloading...';
            downloadBtn.dataset.status = 'downloading';
        }

        const filePath = modelData.gguf_files[filename].path;

        invoke('download_model', {
            modelId: modelId,
            filename: filePath,
            files: [filePath]
        }).then(result => {
            console.log('Download command successful:', result);
            // this.desktop.showNotification(`Download started: ${result.download_id}`, 'success');

            // Store the download ID for tracking
            if (downloadBtn) {
                downloadBtn.dataset.downloadId = result.download_id;
            }

            // Show the download manager
            if (typeof downloadManager !== 'undefined' && downloadManager) {
                downloadManager.showDownloadManager();
            }

            // Start monitoring this download
            this.monitorDownload(result.download_id, modelId, filename);
        }).catch(error => {
            console.error('Download error:', error);
            this.desktop.showNotification('Download failed: ' + error, 'error');

            // Reset button on error
            if (downloadBtn) {
                downloadBtn.disabled = false;
                downloadBtn.innerHTML = 'Download';
                downloadBtn.dataset.status = 'available';
                downloadBtn.classList.remove('downloaded');
            }
        });
    }

    groupFilesBySimilarity(filenames) {
        if (filenames.length <= 1) {
            return [filenames];
        }

        const groups = [];
        const processed = new Set();

        for (const filename of filenames) {
            if (processed.has(filename)) continue;

            const currentGroup = [filename];
            processed.add(filename);

            // Find files that share a significant common prefix with this file
            const cleanName = filename.replace(/\.gguf$/i, '');

            for (const otherFilename of filenames) {
                if (processed.has(otherFilename)) continue;

                const otherCleanName = otherFilename.replace(/\.gguf$/i, '');
                const commonPrefix = this.findCommonPrefix(cleanName, otherCleanName);

                // Group files if they share a meaningful common prefix (at least 5 chars)
                // and the prefix ends with punctuation or the files are very similar
                if (commonPrefix.length >= 5) {
                    const similarity = commonPrefix.length / Math.max(cleanName.length, otherCleanName.length);
                    if (similarity > 0.6 || /[-_.]$/.test(commonPrefix)) {
                        currentGroup.push(otherFilename);
                        processed.add(otherFilename);
                    }
                }
            }

            groups.push(currentGroup);
        }

        return groups;
    }

    findCommonPrefix(str1, str2) {
        let commonPrefix = '';
        const minLength = Math.min(str1.length, str2.length);

        for (let i = 0; i < minLength; i++) {
            if (str1[i] === str2[i]) {
                commonPrefix += str1[i];
            } else {
                break;
            }
        }

        return commonPrefix;
    }

    styleFilenameWithinGroup(displayName, groupFilenames) {
        if (groupFilenames.length <= 1) {
            return displayName;
        }

        // Remove .gguf extension from all filenames in the group for comparison
        const cleanFilenames = groupFilenames.map(f => f.replace(/\.gguf$/i, ''));

        // Find the longest common prefix within this group
        let rawCommonPrefix = cleanFilenames[0];
        for (const filename of cleanFilenames.slice(1)) {
            rawCommonPrefix = this.findCommonPrefix(rawCommonPrefix, filename);
        }

        // Find the best split point by looking for punctuation boundaries
        let commonPrefix = rawCommonPrefix;

        // Look for the last meaningful punctuation (dash, underscore, dot) in the common prefix
        const punctuationMatches = [...rawCommonPrefix.matchAll(/[-_.]/g)];
        if (punctuationMatches.length > 0) {
            // Get the position after the last punctuation mark
            const lastPunctuationPos = punctuationMatches[punctuationMatches.length - 1].index + 1;

            // Only use this split if it leaves a meaningful prefix (at least 3 chars)
            if (lastPunctuationPos >= 3) {
                commonPrefix = rawCommonPrefix.substring(0, lastPunctuationPos);
            }
        }

        // Only apply styling if common prefix is meaningful (more than 2 characters)
        if (commonPrefix.length <= 2) {
            return displayName;
        }

        // Extract the unique part (everything after the common prefix)
        const uniquePart = displayName.substring(commonPrefix.length);

        // Style the parts
        let styledName = '';
        styledName += `<span class="filename-common">${commonPrefix}</span>`;
        styledName += `<span class="filename-unique">${uniquePart}</span>`;

        return styledName;
    }

    async updateFileDownloadStatus(model) {
        const window = this.desktop.windows.get(this.windowId);
        if (!window) return;

        const invoke = this.getInvoke();
        if (!invoke) return;

        // Check download status for each GGUF file
        const promises = Object.keys(model.gguf_files).map(async (filename) => {
            const fileItem = window.querySelector(`[data-filename="${filename}"][data-model-id="${model.id}"]`);
            const downloadBtn = fileItem?.querySelector('.quant-download-btn');

            if (downloadBtn) {
                // Don't update if currently downloading, paused, or in progress
                const currentStatus = downloadBtn.dataset.status;
                if (currentStatus === 'downloading' || currentStatus === 'paused') {
                    return; // Skip this file, let the monitor handle it
                }

                try {
                    // Check if file already exists
                    const fileExists = await invoke('check_file_exists', {
                        modelId: model.id,
                        filename: filename
                    });

                    if (fileExists) {
                        downloadBtn.innerHTML = 'Downloaded';
                        downloadBtn.dataset.status = 'downloaded';
                        downloadBtn.disabled = true;
                        downloadBtn.classList.add('downloaded');
                    } else {
                        downloadBtn.innerHTML = 'Download';
                        downloadBtn.dataset.status = 'available';
                        downloadBtn.disabled = false;
                        downloadBtn.classList.remove('downloaded');
                    }
                } catch (error) {
                    console.error('Error checking file existence:', error);
                    // If check fails, assume available for download
                    downloadBtn.innerHTML = 'Download';
                    downloadBtn.dataset.status = 'available';
                    downloadBtn.disabled = false;
                    downloadBtn.classList.remove('downloaded');
                }
            }
        });

        // Wait for all checks to complete
        await Promise.all(promises);
    }

    // Method to refresh download status for all visible models
    async refreshAllDownloadStatus() {
        const window = this.desktop.windows.get(this.windowId);
        if (!window) return;

        const detailsContent = window.querySelector('#model-details-content');
        if (detailsContent && detailsContent.modelData) {
            await this.updateFileDownloadStatus(detailsContent.modelData);
        }
    }

    // Monitor a specific download and update button status
    async monitorDownload(downloadId, modelId, filename) {
        const invoke = this.getInvoke();
        if (!invoke) return;

        const checkDownloadStatus = async () => {
            try {
                const status = await invoke('get_download_status', { downloadId: downloadId });

                const window = this.desktop.windows.get(this.windowId);
                if (!window) return;

                const fileItem = window.querySelector(`[data-filename="${filename}"][data-model-id="${modelId}"]`);
                const downloadBtn = fileItem?.querySelector('.quant-download-btn');

                if (!downloadBtn) return;

                switch (status.status) {
                    case 'Downloading':
                    case 'Starting':
                        downloadBtn.innerHTML = `Downloading... ${status.progress}%`;
                        downloadBtn.dataset.status = 'downloading';
                        downloadBtn.disabled = true;
                        // Continue monitoring
                        setTimeout(checkDownloadStatus, 1000);
                        break;

                    case 'Completed':
                        downloadBtn.innerHTML = 'Downloaded';
                        downloadBtn.dataset.status = 'downloaded';
                        downloadBtn.disabled = true;
                        downloadBtn.classList.add('downloaded');
                        break;

                    case 'Failed':
                    case 'Cancelled':
                        downloadBtn.innerHTML = 'Download';
                        downloadBtn.dataset.status = 'available';
                        downloadBtn.disabled = false;
                        downloadBtn.classList.remove('downloaded');
                        if (status.status === 'Failed') {
                            // this.desktop.showNotification(`Download failed: ${filename}`, 'error');
                        }
                        break;

                    case 'Paused':
                        downloadBtn.innerHTML = 'Paused';
                        downloadBtn.dataset.status = 'paused';
                        downloadBtn.disabled = true;
                        // Continue monitoring
                        setTimeout(checkDownloadStatus, 2000);
                        break;

                    default:
                        // Continue monitoring for unknown states
                        setTimeout(checkDownloadStatus, 1000);
                        break;
                }
            } catch (error) {
                console.error('Error checking download status:', error);
                // Stop monitoring on error and reset button
                const window = this.desktop.windows.get(this.windowId);
                if (window) {
                    const fileItem = window.querySelector(`[data-filename="${filename}"][data-model-id="${modelId}"]`);
                    const downloadBtn = fileItem?.querySelector('.quant-download-btn');
                    if (downloadBtn) {
                        downloadBtn.innerHTML = 'Download';
                        downloadBtn.dataset.status = 'available';
                        downloadBtn.disabled = false;
                        downloadBtn.classList.remove('downloaded');
                    }
                }
            }
        };

        // Start monitoring
        setTimeout(checkDownloadStatus, 1000);
    }

    quickSearch(query) {
        const window = this.desktop.windows.get(this.windowId);
        if (!window) return;

        const searchInput = window.querySelector('#hf-search-input');
        if (searchInput) {
            searchInput.value = query;
            this.performHuggingFaceSearch();
        }
    }
}