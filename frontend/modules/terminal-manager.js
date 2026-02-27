// Terminal Management Module
// Tauri API will be accessed when needed to prevent loading issues
class TerminalManager {
    constructor(desktop) {
        this.desktop = desktop;
        this.terminals = new Map(); // Store terminal instances
        this.terminalCounter = 0;

        // Initialize Tauri API access - will be set up when initTauriAPI is called
        this.invoke = null;

        // Load auto-switch setting from localStorage (default: enabled)
        try {
            this.autoSwitchEnabled = localStorage.getItem('terminalAutoSwitch') !== 'false';
        } catch (e) {
            console.warn('[TerminalManager] localStorage not available, using default');
            this.autoSwitchEnabled = true;
        }
        
        // Initialize after construction (method is defined below constructor)
        setTimeout(() => {
            try {
                this.initTauriAPI();
            } catch (e) {
                console.error('[TerminalManager] Error in initTauriAPI:', e);
            }
        }, 0);
    }

    initTauriAPI() {
        // Add message listener for iframe communication (always, even if Tauri not ready yet)
        window.addEventListener('message', async (event) => {
            if (event.data && event.data.type === 'request-restart') {
                console.log('[TerminalManager] Restart requested from chat UI:', event.data);
                await this.handleRestartRequest(event.data, event.source);
            } else if (event.data && event.data.type === 'request-compatible-models') {
                console.log('[TerminalManager] Compatible models requested from chat UI');
                await this.handleCompatibleModelsRequest(event.source);
            } else if (event.data && event.data.type === 'request-current-config') {
                console.log('[TerminalManager] Current config requested from chat UI');
                await this.handleCurrentConfigRequest(event.source);
            }
        });

        try {
            if (window.__TAURI__ && window.__TAURI__.core) {
                this.invoke = window.__TAURI__.core.invoke;
                console.log('Tauri API initialized in TerminalManager');
            } else {
                console.warn('Tauri API not available yet, will retry when needed');
            }
        } catch (error) {
            console.error('Failed to initialize Tauri API:', error);
        }
    }

    parseLaunchArgSignature(launchArgs = '') {
        const tokens = String(launchArgs || '').match(/"[^"]*"|\S+/g) || [];
        const normalized = [];
        const pairs = {};

        const canonicalKey = {
            '-c': 'context',
            '-np': 'slots',
            '--no-cont-batching': 'cont_batching',
            '--context-shift': 'context_shift',
            '-ngl': 'gpu_layers',
            '-ncmoe': 'cpu_moe_layers',
            '-sm': 'split_mode',
            '-mg': 'main_gpu',
            '-fa': 'flash_attention',
            '--no-mmap': 'mmap',
            '-ctk': 'cache_type_k',
            '-ctv': 'cache_type_v',
            '--rope-scaling': 'rope_scaling',
            '--rope-scale': 'rope_scale',
            '--rope-freq-base': 'rope_freq_base',
            '--model-draft': 'model_draft',
            '-md': 'model_draft',
            '--draft-p-min': 'draft_p_min',
            '--draft-max': 'draft_max',
            '-ngld': 'draft_ngld',
            '--numa': 'numa',
            '--no-pinned-memory': 'pinned_memory',
            '--jinja': 'jinja',
            '--no-cont-batching=1': 'cont_batching'
        };

        for (let i = 0; i < tokens.length; i++) {
            let token = tokens[i];
            if (!token) {
                continue;
            }

            if (token.startsWith('--') && token.includes('=')) {
                const parts = token.split('=');
                token = parts[0];
                if (parts.length > 1) {
                    tokens[i + 1] = parts.slice(1).join('=');
                }
            }

            if (token.startsWith('-') && token.endsWith('"') && token.startsWith('"')) {
                token = token.slice(1, -1);
            }

            const key = canonicalKey[token];
            if (!key) {
                continue;
            }

            if (
                key === 'cont_batching' ||
                key === 'context_shift' ||
                key === 'mmap' ||
                key === 'pinned_memory' ||
                key === 'jinja'
            ) {
                pairs[key] = true;
                normalized.push(token);
                continue;
            }

            const hasValue = (i + 1 < tokens.length);
            const rawValue = hasValue ? tokens[i + 1] : '';
            const value = rawValue.replace(/^"|"$/g, '').toLowerCase();
            if (hasValue) {
                if (key === 'model_draft') {
                    pairs[key] = value.replace(/\\/g, '/');
                    i += 1;
                    normalized.push(`${key}=${value}`);
                    continue;
                }

                pairs[key] = value;
                i += 1;
                normalized.push(`${key}=${value}`);
            }
        }

        // Normalize boolean flags for deterministic equality checks
        for (const key of ['cont_batching', 'context_shift', 'mmap', 'pinned_memory', 'jinja']) {
            if (pairs[key] === undefined) {
                pairs[key] = false;
            }
        }

        return { pairs, normalized };
    }

    envVarsToComparableString(rawEnvVars = '') {
        let envMap = {};

        if (typeof rawEnvVars === 'string') {
            rawEnvVars.split('\n').forEach((line) => {
                const trimmedLine = String(line).trim();
                if (!trimmedLine) {
                    return;
                }

                const equalsIndex = trimmedLine.indexOf('=');
                if (equalsIndex <= 0) {
                    return;
                }

                const key = trimmedLine.slice(0, equalsIndex).trim();
                const value = trimmedLine.slice(equalsIndex + 1).trim();
                if (key) {
                    envMap[key] = value;
                }
            });
        } else if (rawEnvVars && typeof rawEnvVars === 'object') {
            try {
                Object.entries(rawEnvVars).forEach(([rawKey, rawValue]) => {
                    const key = String(rawKey || '').trim();
                    if (key) {
                        envMap[key] = rawValue == null ? '' : String(rawValue);
                    }
                });
            } catch (error) {
                console.warn('[TerminalManager] Failed to normalize env vars object, using fallback:', error);
            }
        }

        return Object.keys(envMap)
            .sort((a, b) => String(a).localeCompare(String(b)))
            .map((key) => `${key}=${envMap[key]}`)
            .join('\n');
    }

    parseEnvVarsToObject(rawEnvVars = '') {
        const envVars = {};
        const normalizedEnvString = this.envVarsToComparableString(rawEnvVars);

        normalizedEnvString.split('\n').forEach((entry) => {
            if (!entry) {
                return;
            }

            const index = entry.indexOf('=');
            if (index < 0) {
                return;
            }

            const key = entry.slice(0, index).trim();
            const value = entry.slice(index + 1);
            if (key) {
                envVars[key] = value;
            }
        });

        return envVars;
    }

    hasRestartImpactArgChange(previousArgs, nextArgs, previousEnvVars, nextEnvVars) {
        const prev = this.parseLaunchArgSignature(previousArgs).pairs;
        const next = this.parseLaunchArgSignature(nextArgs).pairs;

        const keys = new Set([...Object.keys(prev), ...Object.keys(next)]);
        for (const key of keys) {
            if (prev[key] !== next[key]) {
                return true;
            }
        }

        const prevEnvSignature = this.envVarsToComparableString(previousEnvVars);
        const nextEnvSignature = this.envVarsToComparableString(nextEnvVars);
        if (prevEnvSignature !== nextEnvSignature) {
            return true;
        }

        return false;
    }

    async handleCompatibleModelsRequest(sourceWindow) {
        console.log('[TerminalManager] Finding compatible models for source window...');
        
        // Find which terminal this request came from by matching the iframe window
        let sourceTerminal = null;
        let sourceWindowId = null;

        for (const [windowId, info] of this.terminals.entries()) {
            const chatPanel = document.getElementById(`panel-chat-${windowId}`);
            if (chatPanel) {
                const iframe = chatPanel.querySelector('iframe');
                if (iframe && iframe.contentWindow === sourceWindow) {
                    sourceTerminal = info;
                    sourceWindowId = windowId;
                    console.log(`[TerminalManager] Identified source window: ${windowId} (${info.modelName})`);
                    break;
                }
            }
        }

        // Fallback: If we couldn't match the window (rare), use the active window
        if (!sourceTerminal) {
            const activeWindow = document.querySelector('.window.active[id^="server_"]');
            if (activeWindow) {
                sourceWindowId = activeWindow.id;
                sourceTerminal = this.terminals.get(sourceWindowId);
                console.log(`[TerminalManager] Fallback used: active window ${sourceWindowId}`);
            }
        }

        if (!sourceTerminal) {
            console.error('[TerminalManager] Could not identify source terminal for compatible models request');
            return;
        }

        try {
            const invoke = this.getInvoke();
            if (!invoke) throw new Error('Invoke not available');
            
            // 1. Get architecture of the current model
            console.log(`[TerminalManager] Getting metadata for ${sourceTerminal.modelPath}`);
            const currentMetadata = await invoke('get_model_metadata', { modelPath: sourceTerminal.modelPath });
            const currentArch = currentMetadata.architecture;
            console.log(`[TerminalManager] Current architecture: ${currentArch}`);
            
            // 2. Scan all models
            const scanResult = await invoke('scan_models_command');
            if (!scanResult.success) {
                console.error('[TerminalManager] Model scan failed');
                return;
            }
            
            // 3. Filter by architecture (ignore current model)
            const compatibleModels = scanResult.models.filter(m => 
                m.architecture === currentArch && m.path !== sourceTerminal.modelPath
            );
            console.log(`[TerminalManager] Found ${compatibleModels.length} compatible models`);
            
            // 4. Send back to the iframe
            sourceWindow.postMessage({
                type: 'compatible-models-list',
                models: compatibleModels,
                currentArch: currentArch
            }, '*');
            
        } catch (error) {
            console.error('[TerminalManager] Error finding compatible models:', error);
            // Send empty list on error to stop the spinner
            sourceWindow.postMessage({
                type: 'compatible-models-list',
                models: [],
                currentArch: 'unknown'
            }, '*');
        }
    }

    async handleCurrentConfigRequest(sourceWindow) {
        console.log('[TerminalManager] Getting current config for source window...');
        
        // Find which terminal this request came from by matching the iframe window
        let sourceTerminal = null;
        let sourceWindowId = null;

        for (const [windowId, info] of this.terminals.entries()) {
            const chatPanel = document.getElementById(`panel-chat-${windowId}`);
            if (chatPanel) {
                const iframe = chatPanel.querySelector('iframe');
                if (iframe && iframe.contentWindow === sourceWindow) {
                    sourceTerminal = info;
                    sourceWindowId = windowId;
                    console.log(`[TerminalManager] Identified source window for config: ${windowId}`);
                    break;
                }
            }
        }

        // Fallback: Use active window
        if (!sourceTerminal) {
            const activeWindow = document.querySelector('.window.active[id^="server_"]');
            if (activeWindow) {
                sourceWindowId = activeWindow.id;
                sourceTerminal = this.terminals.get(sourceWindowId);
                console.log(`[TerminalManager] Fallback used for config: active window ${sourceWindowId}`);
            }
        }

        if (!sourceTerminal) {
            console.error('[TerminalManager] Could not identify source terminal for config request');
            sourceWindow.postMessage({
                type: 'current-config',
                launchArgs: '',
                draftModelPath: '',
                env_vars: ''
            }, '*');
            return;
        }

        // Extract draft model path from launchArgs
        let draftModelPath = '';
        if (sourceTerminal.launchArgs) {
            const mdMatch = sourceTerminal.launchArgs.match(/-md\s+"([^"]+)"/);
            const mdMatch2 = sourceTerminal.launchArgs.match(/-md\s+(\S+)/);
            if (mdMatch) {
                draftModelPath = mdMatch[1];
            } else if (mdMatch2) {
                draftModelPath = mdMatch2[1];
            }
        }

        console.log(`[TerminalManager] Current draft model: ${draftModelPath}`);

        let envVars = '';
        try {
            const invoke = this.getInvoke();
            if (invoke) {
                const currentConfig = await invoke('get_model_settings', { modelPath: sourceTerminal.modelPath });
                if (currentConfig && currentConfig.env_vars) {
                    if (typeof currentConfig.env_vars === 'string') {
                        envVars = currentConfig.env_vars;
                    } else {
                        envVars = Object.entries(currentConfig.env_vars)
                            .sort(([keyA], [keyB]) => String(keyA).localeCompare(String(keyB)))
                            .map(([key, value]) => `${String(key)}=${String(value ?? '')}`)
                            .join('\n');
                    }
                }
            }
        } catch (error) {
            console.error('[TerminalManager] Failed to fetch env vars for current config:', error);
        }
        
        // Send config back to iframe
        sourceWindow.postMessage({
            type: 'current-config',
            launchArgs: sourceTerminal.launchArgs || '',
            draftModelPath: draftModelPath,
            env_vars: envVars
        }, '*');
    }

    async handleRestartRequest(data, sourceWindow) {
        console.log('[TerminalManager] Handling restart request...');
        
        // Identify the terminal from the source window
        let sourceTerminal = null;
        let sourceWindowId = null;

        for (const [windowId, info] of this.terminals.entries()) {
            const chatPanel = document.getElementById(`panel-chat-${windowId}`);
            if (chatPanel) {
                const iframe = chatPanel.querySelector('iframe');
                if (iframe && iframe.contentWindow === sourceWindow) {
                    sourceTerminal = info;
                    sourceWindowId = windowId;
                    console.log(`[TerminalManager] Identified source window for restart: ${windowId}`);
                    break;
                }
            }
        }

        // Fallback: Use active window
        if (!sourceTerminal) {
            const activeWindow = document.querySelector('.window.active[id^="server_"]');
            if (activeWindow) {
                sourceWindowId = activeWindow.id;
                sourceTerminal = this.terminals.get(sourceWindowId);
                console.log(`[TerminalManager] Fallback used for restart: active window ${sourceWindowId}`);
            }
        }

        if (!sourceTerminal) {
            console.error('[TerminalManager] Could not identify source terminal for restart request');
            return;
        }

        const requestedArgs = typeof data?.args === 'string' ? data.args : '';
        const requestedEnvVars = data && data.env_vars !== undefined ? data.env_vars : '';
        const requestSaysRestart = data && data.requiresRestart === true;
        const requestSaysNoRestart = data && data.requiresRestart === false;

        const invoke = this.getInvoke();
        if (!invoke) {
            const message = 'Error saving settings: Invoke not available';
            console.error('[TerminalManager] Error handling restart request:', message);
            sourceWindow.postMessage({ type: 'settings-saved', message }, '*');
            return;
        }

        let currentConfig = null;
        try {
            currentConfig = await invoke('get_model_settings', { modelPath: sourceTerminal.modelPath });
        } catch (error) {
            console.error('[TerminalManager] Error fetching model settings:', error);
            sourceWindow.postMessage({
                type: 'settings-saved',
                message: 'Error saving settings: ' + error.message
            }, '*');
            return;
        }

        const baseConfig = currentConfig && typeof currentConfig === 'object' ? currentConfig : {};
        const previousArgs = typeof baseConfig.custom_args === 'string' ? baseConfig.custom_args : sourceTerminal.launchArgs || '';
        const previousEnvVars = baseConfig.env_vars;

        const hasRestartImpact = this.hasRestartImpactArgChange(previousArgs, requestedArgs, previousEnvVars, requestedEnvVars);

        // Keep safe defaults for unknown/legacy callers: restart unless the caller
        // explicitly sends requiresRestart=false and there is no launch-impacting
        // change in args or env vars.
        const shouldRestart = requestSaysRestart || (requestSaysNoRestart ? hasRestartImpact : true);
        const shouldNotRestart = !requestSaysRestart && requestSaysNoRestart;

        if (requestSaysNoRestart && shouldRestart) {
            console.warn('[TerminalManager] Restart override: child requested no restart but launch-impacting fields changed.', {
                previous: previousArgs,
                next: requestedArgs,
                previousEnv: this.envVarsToComparableString(previousEnvVars),
                nextEnv: this.envVarsToComparableString(requestedEnvVars)
            });
        }

        const env_vars = this.parseEnvVarsToObject(requestedEnvVars);
        const normalizedArgs = requestedArgs.trim();

        if (shouldNotRestart && !shouldRestart) {
            console.log('[TerminalManager] No restart required, just saving settings...');
            try {
                console.log(`[TerminalManager] Saving settings without restart for ${sourceTerminal.modelPath}`);
                await invoke('update_model_settings', {
                    modelPath: sourceTerminal.modelPath,
                    config: {
                        ...baseConfig,
                        custom_args: normalizedArgs,
                        env_vars: env_vars
                    }
                });

                sourceTerminal.launchArgs = normalizedArgs;
                this.terminals.set(sourceWindowId, sourceTerminal);

                sourceWindow.postMessage({
                    type: 'settings-saved',
                    message: 'Settings saved. They will take effect on next server start.'
                }, '*');

                console.log('[TerminalManager] Settings saved successfully');
            } catch (error) {
                console.error('[TerminalManager] Error saving settings:', error);
                sourceWindow.postMessage({
                    type: 'settings-saved',
                    message: 'Error saving settings: ' + error.message
                }, '*');
            }
            return;
        }

        // Full restart is required
        console.log('[TerminalManager] Full restart required');
        try {
            // Update model settings in backend
            console.log(`[TerminalManager] Updating model settings for ${sourceTerminal.modelPath}`);
            await invoke('update_model_settings', {
                modelPath: sourceTerminal.modelPath,
                config: {
                    ...baseConfig,
                    custom_args: normalizedArgs,
                    env_vars: env_vars
                }
            });

            // Update local terminalInfo launchArgs
            sourceTerminal.launchArgs = normalizedArgs;
            this.terminals.set(sourceWindowId, sourceTerminal);

            sourceWindow.postMessage({
                type: 'settings-saved',
                message: 'Settings saved. Restarting server to apply launch changes.',
                restartTriggered: true
            }, '*');

            // Trigger restart
            console.log(`[TerminalManager] Triggering server restart for ${sourceWindowId}`);
            await this.restartServer(sourceWindowId, sourceTerminal.modelPath, sourceTerminal.modelName);
        } catch (error) {
            console.error('[TerminalManager] Error handling restart request:', error);
            sourceWindow.postMessage({
                type: 'settings-saved',
                message: 'Error saving settings: ' + error.message
            }, '*');
        }
    }

    getInvoke() {
        if (!this.invoke) {
            this.initTauriAPI();
        }
        return this.invoke;
    }

    async openServerTerminal(processId, modelName, host, port, modelPath, activeVersion, launchArgs = null) {
        let resolvedLaunchArgs = launchArgs;

        if (resolvedLaunchArgs === null || resolvedLaunchArgs === undefined) {
            try {
                const invoke = this.getInvoke();
                if (!invoke) {
                    console.warn('[TerminalManager] Tauri invoke unavailable while resolving launch args for terminal creation.');
                    resolvedLaunchArgs = '';
                } else {
                    const currentConfig = await invoke('get_model_settings', { modelPath: modelPath });
                    resolvedLaunchArgs = currentConfig && currentConfig.custom_args ? currentConfig.custom_args : '';
                }
            } catch (error) {
                console.error('[TerminalManager] Failed to resolve launch args from model settings:', error);
                resolvedLaunchArgs = '';
            }
        }

        if (typeof resolvedLaunchArgs !== 'string') {
            resolvedLaunchArgs = '';
        }

        resolvedLaunchArgs = resolvedLaunchArgs.trim();
        console.log('OpenServerTerminal called with:', {
            processId,
            modelName,
            host,
            port,
            modelPath,
            activeVersion,
            launchArgs: resolvedLaunchArgs
        });

        const windowId = `server_${processId}`;
        console.log('Creating terminal window with ID:', windowId);

const content = `
            <div class="server-terminal-container">
                <div class="server-main-content">
                    <div class="server-tab-panel active" id="panel-terminal-${windowId}">
                        <div class="server-info">
                            <span class="server-status starting"><span class="material-icons" style="color: #ffc107; font-size: 14px;">circle</span> Starting</span>
                            <span class="server-details">${modelName} - <span class="clickable" style="cursor: pointer; text-decoration: underline;" onclick="terminalManager.openUrl('http://${host}:${port}')">${host}:${port}</span><button class="copy-link-btn" style="background: none; border: none; cursor: pointer; margin-left: 5px; padding: 0; font-size: 14px; vertical-align: middle;" onclick="terminalManager.copyToClipboard('http://${host}:${port}', this)" title="Copy link"><span class="material-icons" style="font-size: 14px; color: var(--theme-text-muted);">content_copy</span></button></span>
                            <div class="server-controls">
                                <button class="server-btn auto-switch-btn ${this.autoSwitchEnabled ? 'active' : ''}" id="auto-switch-btn-${windowId}" onclick="terminalManager.toggleAutoSwitch('${windowId}')" title="${this.autoSwitchEnabled ? 'Auto-switch to chat: ON' : 'Auto-switch to chat: OFF'}"><span class="material-icons">${this.autoSwitchEnabled ? 'toggle_on' : 'toggle_off'}</span></button>
                                <button class="server-btn stop-btn" id="stop-btn-${windowId}"><span class="material-icons">stop</span> Stop</button>
                            </div>
                        </div>
                        <div class="server-output" id="server-output-${windowId}"><div class="server-line server-system">Starting ${modelName}...</div><div class="server-line server-system">Process ID: ${processId}</div><div class="server-line server-system">Server will be available at: ${host}:${port}</span></div><div class="server-line server-system">Waiting for server output...</div></div>
                    </div>
                    <div class="server-tab-panel" id="panel-chat-${windowId}" style="background: white;">
                        <iframe src="http://${host}:${port}" frameBorder="0" style="width: 100%; height: 100%; border: none;"></iframe>
                    </div>
                </div>
            </div>
        `;

        console.log('Calling desktop.createWindow...');
        const window = this.desktop.createWindow(windowId, `Server - ${modelName} (Build: ${activeVersion})`, 'server-terminal-window', content);
        console.log('Desktop.createWindow returned:', window);

        // Ensure the window is visible and not hidden
        if (window) {
            // Inject tabs into header
            const header = window.querySelector('.window-header');
            if (header) {
const tabsHtml = `
                    <div class="server-tabs header-tabs">
                        <div class="server-tab active" id="tab-terminal-${windowId}" onclick="terminalManager.switchTab('${windowId}', 'terminal')" title="Terminal Output">
                            <span class="material-icons">terminal</span>
                        </div>
                        <div class="server-tab" id="tab-chat-${windowId}" onclick="terminalManager.switchTab('${windowId}', 'chat')" title="Native Chat" style="opacity: 0.5; pointer-events: none;">
                            <span class="material-icons">chat</span>
                        </div>
                    </div>
                `;
                const titleElement = header.querySelector('.window-title');
                if (titleElement) {
                    titleElement.insertAdjacentHTML('afterend', tabsHtml);
                }
            }

            console.log('Making window visible...');
            window.classList.remove('hidden');
            window.style.display = 'block';
            // Bring to front
            window.style.zIndex = this.desktop.windowZIndex + 1;
            this.desktop.windowZIndex += 1;

            console.log('Window after visibility setup:', {
                id: window.id,
                display: window.style.display,
                visibility: window.style.visibility,
                zIndex: window.style.zIndex,
                classList: Array.from(window.classList)
            });

            // Handle focus/click to bring to front and handle global click interactions
            // Since iframe consumes clicks, we monitor window blur which happens when clicking into iframe
            setTimeout(() => {
                const chatPanel = window.querySelector(`#panel-chat-${windowId}`);
                if (chatPanel) {
                    const iframe = chatPanel.querySelector('iframe');
                    if (iframe) {
                        const blurHandler = () => {
                            if (document.activeElement === iframe) {
                                // Bring this window to front
                                window.style.zIndex = ++this.desktop.windowZIndex;
                                
                                // Update visual active state
                                document.querySelectorAll('.window').forEach(w => w.classList.remove('active'));
                                window.classList.add('active');
                                
                                document.querySelectorAll('.taskbar-item').forEach(t => t.classList.remove('active'));
                                const taskbarItem = document.getElementById(`taskbar-${windowId}`);
                                if (taskbarItem) taskbarItem.classList.add('active');

                                // Trigger global click interactions (hide menus, collapse hardware monitor)
                                if (this.desktop.handleGlobalClickInteraction) {
                                    this.desktop.handleGlobalClickInteraction();
                                }
                            }
                        };
                        window.addEventListener('blur', blurHandler);
                        
                        // Store handler for cleanup
                        const terminalInfo = this.terminals.get(windowId);
                        if (terminalInfo) {
                            terminalInfo.blurHandler = blurHandler;
                            this.terminals.set(windowId, terminalInfo);
                        }
                    }
                }
            }, 100);
        } else {
            console.error('Failed to create terminal window!');
            return null;
        }

        // Store model info for this terminal
        this.terminals.set(windowId, {
            processId,
            modelName,
            modelPath,
            host,
            port,
            status: 'starting',
            output: [], // Store terminal output lines
            activeVersion: activeVersion,
            launchArgs: resolvedLaunchArgs // Store the actual arguments used for launch
        });

        console.log('Adding taskbar item...');
        // Add to taskbar
        this.desktop.addTaskbarItem(`Server - ${modelName}`, windowId, '<span class="material-icons">computer</span>');

        // Set up event listeners for buttons
        setTimeout(() => {


            const stopBtn = document.getElementById(`stop-btn-${windowId}`);
            if (stopBtn) {
                stopBtn.addEventListener('click', () => {
                    this.updateServerStatus(windowId, 'terminating');
                    this.stopServer(processId, windowId, modelPath, modelName);
                });
            }
        }, 0);

        console.log('Starting output polling...');
        // Start polling for output
        this.startServerOutputPolling(processId, windowId);

        // Also add a status check after a few seconds to ensure we show something
        setTimeout(() => {
            this.checkServerHealth(windowId, host, port, modelName);
        }, 2000);

        console.log('Terminal window creation completed successfully');
        
        // Maximize the window on creation - need to clear transform first
        setTimeout(() => {
            // Remove the centering transform before maximizing
            window.style.transform = 'none';
            window.style.left = '100px';
            window.style.top = '100px';
            
            // Now maximize
            this.desktop.maximizeWindow(windowId);
}, 50);
        
        return window;
    }

    startServerOutputPolling(processId, windowId) {
        const terminalInfo = this.terminals.get(windowId);
        if (!terminalInfo) return;

        // Track last scroll position to determine if user is scrolled up
        let isScrolledToBottom = true;

        // Batch updates to reduce DOM operations
        let outputBuffer = [];
        let updateTimer = null;
        let lastOutputTime = 0;
        const minUpdateInterval = 50; // ms

        const flushOutputBuffer = (outputDiv) => {
            if (outputBuffer.length > 0) {
                // Check if user has scrolled up
                const wasScrolledToBottom = isScrolledToBottom;

                // Create a document fragment to batch DOM operations
                const fragment = document.createDocumentFragment();
                outputBuffer.forEach(line => {
                    if (line !== null && line !== undefined) {
                        const lineDiv = document.createElement('div');
                        lineDiv.className = 'server-line';
                        // Handle special characters and escape sequences
                        lineDiv.textContent = line.toString();
                        fragment.appendChild(lineDiv);
                    }
                });

                outputDiv.appendChild(fragment);

                // Only scroll to bottom if user hasn't scrolled up
                if (wasScrolledToBottom) {
                    outputDiv.scrollTop = outputDiv.scrollHeight;
                }

                outputBuffer = [];
            }
        };

        const pollOutput = async () => {
            try {
                console.log(`Polling output for process ${processId}...`);
                // Use Tauri command instead of fetch
                const invoke = this.getInvoke();
                if (!invoke) {
                    console.error('Tauri invoke not available for output polling');
                    return;
                }
                const data = await invoke('get_process_output', { processId: processId });
                console.log(`Output data received:`, data);

                const outputDiv = document.getElementById(`server-output-${windowId}`);

                if (!outputDiv) {
                    console.warn(`Output div not found for ${windowId}`);
                    return;
                }

                // Update scroll position tracking ONLY if visible
                if (outputDiv.offsetParent !== null) {
                    const scrollTop = outputDiv.scrollTop;
                    const scrollHeight = outputDiv.scrollHeight;
                    const clientHeight = outputDiv.clientHeight;
                    isScrolledToBottom = (scrollTop + clientHeight >= scrollHeight - 5);
                }

                // Add new output lines to buffer if they exist
                if (data.output && Array.isArray(data.output) && data.output.length > 0) {
                    console.log(`Adding ${data.output.length} output lines to buffer`);
                    outputBuffer.push(...data.output);

                    // Check for speculative decoding error
                    const specErrorPatterns = [
                        'speculative decoding not supported',
                        'speculative stuff won\'t work',
                        'target context does not support partial sequence removal'
                    ];
                    const hasSpecError = data.output.some(line =>
                        line && specErrorPatterns.some(pattern => line.toString().includes(pattern))
                    );
                    
                    if (hasSpecError && terminalInfo.status === 'starting') {
                        console.warn('[TerminalManager] Speculative decoding error detected!');
                        this.desktop.showNotification('Speculative decoding is not supported by this model architecture. The draft model will not be used.', 'warning');
                    }

                    // Check for server ready message and update status
                    const serverReadyMessage = "main: server is listening on http://";
                    const hasServerReadyMessage = data.output.some(line =>
                        line && line.toString().includes(serverReadyMessage)
                    );

                    if (hasServerReadyMessage && terminalInfo.status === 'starting') {
                        console.log('Server ready message detected, updating status to running');
                        this.updateServerStatus(windowId, 'running');
                        
                        // Force chat iframe reload to ensure it connects
                        const chatPanel = document.getElementById(`panel-chat-${windowId}`);
                        if (chatPanel) {
                            const iframe = chatPanel.querySelector('iframe');
                            if (iframe) {
                                console.log(`[TerminalManager] Server ready - reloading chat iframe for ${windowId}`);
                                // Force reload by re-assigning src
                                const currentSrc = iframe.src;
                                iframe.src = 'about:blank';
                                setTimeout(() => {
                                    iframe.src = currentSrc;
                                }, 50);
                            }
                        }

                        // Auto-switch to chat tab when server is running (if enabled)
                        if (this.autoSwitchEnabled) {
                            setTimeout(() => {
                                this.switchTab(windowId, 'chat');
                            }, 4000);
                        }
                    }

                    // Save output to terminal data (keep last 1000 lines)
                    const terminalData = this.terminals.get(windowId);
                    if (terminalData) {
                        if (!terminalData.output) terminalData.output = [];
                        terminalData.output.push(...data.output);
                        // Keep only last 1000 lines to prevent memory issues
                        if (terminalData.output.length > 1000) {
                            terminalData.output = terminalData.output.slice(-1000);
                        }
                        this.terminals.set(windowId, terminalData);
                    }

                    // Throttle updates to prevent UI freezing but be more responsive
                    const now = Date.now();
                    if (now - lastOutputTime > minUpdateInterval || outputBuffer.length > 50) {
                        console.log('Flushing output buffer immediately');
                        flushOutputBuffer(outputDiv);
                        lastOutputTime = now;
                    } else if (!updateTimer) {
                        // Schedule buffer flush
                        updateTimer = setTimeout(() => {
                            console.log('Flushing output buffer (scheduled)');
                            flushOutputBuffer(outputDiv);
                            updateTimer = null;
                            lastOutputTime = Date.now();
                        }, minUpdateInterval);
                    }
                } else {
                    console.log('No new output data received');
                }

                // Check if process is still running
                if (data.is_running !== false && (terminalInfo.status === 'running' || terminalInfo.status === 'starting')) {
                    console.log('Process still running, continuing polling in 100ms');
                    // Continue polling if process is still running - faster polling for better responsiveness
                    setTimeout(pollOutput, 100);
                } else if (data.is_running === false) {
                    console.log('Process has stopped, finalizing output');
                    // Process has stopped, flush any remaining output
                    if (updateTimer) {
                        clearTimeout(updateTimer);
                    }
                    flushOutputBuffer(outputDiv);
                    this.updateServerStatus(windowId, 'stopped', data.return_code || 0);
                }
            } catch (error) {
                console.error('Error polling server output:', error);
                const terminalInfo = this.terminals.get(windowId);

                // If the server is gone or another error, stop polling.
                this.updateServerStatus(windowId, 'stopped', -1);
                const outputDiv = document.getElementById(`server-output-${windowId}`);
                if (outputDiv) {
                    const errorDiv = document.createElement('div');
                    errorDiv.className = 'server-line server-error';
                    errorDiv.textContent = `Connection to server lost. Polling stopped. Error: ${error.message}`;
                    outputDiv.appendChild(errorDiv);
                    outputDiv.scrollTop = outputDiv.scrollHeight;
                }
                return; // Stop polling
            }
        };

        // Start polling immediately
        pollOutput();
    }

    async checkServerHealth(windowId, host, port, modelName, retryCount = 0) {
        const outputDiv = document.getElementById(`server-output-${windowId}`);
        if (!outputDiv) return;

        const maxRetries = 15; // Increased retries for dual-model loading
        const retryDelay = 2000; // 2 seconds between checks

        try {
            console.log(`[TerminalManager] Health check attempt ${retryCount + 1}/${maxRetries} for ${modelName}...`);
            const response = await fetch(`http://${host}:${port}/health`, {
                method: 'GET',
                signal: AbortSignal.timeout(3000)
            });

            if (response.ok) {
                console.log(`[TerminalManager] Server ${modelName} is healthy and responding.`);
                const lineDiv = document.createElement('div');
                lineDiv.className = 'server-line server-success';
                lineDiv.textContent = `Server is responding! Ready for chat.`;
                outputDiv.appendChild(lineDiv);
                outputDiv.scrollTop = outputDiv.scrollHeight;

                // Update server status to running
                const terminalInfo = this.terminals.get(windowId);
                if (terminalInfo) {
                    terminalInfo.status = 'running';
                    this.terminals.set(windowId, terminalInfo);
                    this.updateServerStatus(windowId, 'running');
                }

                // IMPORTANT: Trigger the Chat Iframe reload now that we KNOW it's ready
                this.refreshChatIframe(windowId);
                
                // If auto-switch is enabled, switch to chat tab
                if (this.autoSwitchEnabled) {
                    setTimeout(() => this.switchTab(windowId, 'chat'), 500);
                }
            } else {
                throw new Error('Not ready');
            }
        } catch (error) {
            if (retryCount < maxRetries) {
                setTimeout(() => this.checkServerHealth(windowId, host, port, modelName, retryCount + 1), retryDelay);
            } else {
                console.error(`[TerminalManager] Health check failed for ${modelName} after ${maxRetries} attempts.`);
                const lineDiv = document.createElement('div');
                lineDiv.className = 'server-line server-error';
                lineDiv.textContent = `Health check failed. The server might have failed to start or is taking too long to load.`;
                outputDiv.appendChild(lineDiv);
                
                // Feature: Auto-Recovery for failing draft models
                const terminalInfo = this.terminals.get(windowId);
                if (terminalInfo && (terminalInfo.launchArgs || "").includes('-md')) {
                    console.log(`[TerminalManager] Speculative drafting failure detected for ${modelName}. Attempting auto-recovery...`);
                    await this.recoverFromDraftFailure(windowId, terminalInfo);
                } else {
                    lineDiv.textContent += ` Check terminal output for errors.`;
                }
                
                outputDiv.scrollTop = outputDiv.scrollHeight;
            }
        }
    }

    refreshChatIframe(windowId) {
        const chatPanel = document.getElementById(`panel-chat-${windowId}`);
        if (!chatPanel) return;

        const iframe = chatPanel.querySelector('iframe');
        if (iframe) {
            console.log(`[TerminalManager] Refreshing chat iframe for ${windowId}`);
            const currentSrc = iframe.src;
            iframe.src = 'about:blank';
            setTimeout(() => {
                iframe.src = currentSrc;
            }, 100);
        }
    }

    async recoverFromDraftFailure(windowId, terminalInfo) {
        const outputDiv = document.getElementById(`server-output-${windowId}`);
        if (!outputDiv) return;

        try {
            const invoke = this.getInvoke();
            if (!invoke) return;

            console.log(`[TerminalManager] Reverting draft model config for ${terminalInfo.modelName}...`);
            
            // 1. Get current settings
            const currentConfig = await invoke('get_model_settings', { modelPath: terminalInfo.modelPath });
            
            // 2. Strip draft-specific arguments but keep everything else
            const stableArgs = this.stripDraftArguments(currentConfig.custom_args);
            
            // 3. Save the "Clean" configuration
            await invoke('update_model_settings', {
                modelPath: terminalInfo.modelPath,
                config: {
                    ...currentConfig,
                    custom_args: stableArgs
                }
            });

            // 4. Update local state
            terminalInfo.launchArgs = stableArgs;
            this.terminals.set(windowId, terminalInfo);

            // 5. Inform user
            const recoveryDiv = document.createElement('div');
            recoveryDiv.className = 'server-line server-warning';
            recoveryDiv.style.color = '#ff9800';
            recoveryDiv.style.fontWeight = 'bold';
            recoveryDiv.style.marginTop = '10px';
            recoveryDiv.textContent = `⚠️ AUTO-RECOVERY: The draft model failed to load. Speculative drafting has been disabled in your settings to restore stability. You can now start the model normally.`;
            outputDiv.appendChild(recoveryDiv);
            outputDiv.scrollTop = outputDiv.scrollHeight;

        } catch (error) {
            console.error('[TerminalManager] Failed to execute auto-recovery:', error);
        }
    }

    stripDraftArguments(args) {
        if (!args) return "";
        // Removes draft-specific arguments AND forces -fa off and -np 1
        // for stable recovery after draft model failure
        let cleaned = args
            .replace(/--model-draft\s+"[^"]*"/g, '')
            .replace(/--model-draft\s+\S+/g, '')
            .replace(/-md\s+"[^"]*"/g, '')
            .replace(/-md\s+\S+/g, '')
            .replace(/--ngld\s+\d+/g, '')
            .replace(/-ngld\s+\d+/g, '')
            .replace(/--draft-p-min\s+\d+(\.\d+)?/g, '')
            .replace(/--draft-max\s+\d+/g, '')
            .replace(/-fa\s+(on|off|auto)/g, '-fa off')
            .replace(/-np\s+\d+/g, '-np 1')
            .replace(/\s+/g, ' ')
            .trim();
        
        // Ensure -fa off and -np 1 are present
        if (!cleaned.includes('-fa off')) {
            cleaned += ' -fa off';
        }
        if (!cleaned.includes('-np 1')) {
            cleaned += ' -np 1';
        }
        
        return cleaned;
    }

    updateServerStatus(windowId, status, returnCode = null) {
        const window = this.desktop.windows.get(windowId);
        const terminalInfo = this.terminals.get(windowId);

        if (window && terminalInfo) {
            // Feature: Immediate Recovery for Crashes (Code 1)
            // Only trigger if:
            // 1. Process has an actual crash code (not -1 which is polling error)
            // 2. Process was in 'starting' status (not already running)
            // 3. Has draft model in args
            const isPollingError = (returnCode === -1 || returnCode === null);
            const wasStarting = (terminalInfo.status === 'starting');
            
            if (status === 'stopped' && !isPollingError && wasStarting && (terminalInfo.launchArgs || "").includes('-md')) {
                console.log(`[TerminalManager] Actual crash (code ${returnCode}) detected during startup for draft model. Triggering recovery...`);
                this.recoverFromDraftFailure(windowId, terminalInfo);
            } else if (status === 'stopped' && isPollingError && wasStarting && (terminalInfo.launchArgs || "").includes('-md')) {
                console.log(`[TerminalManager] Polling error during startup for draft model - NOT triggering auto-recovery (was starting, polling may have failed)`);
            } else if (status === 'stopped' && (terminalInfo.launchArgs || "").includes('-md')) {
                console.log(`[TerminalManager] Server stopped but was already running - NOT triggering auto-recovery (may be normal shutdown or polling issue)`);
            }

            const statusElement = window.querySelector('.server-status');
            const stopBtn = window.querySelector('.stop-btn');

            if (statusElement) {
                if (status === 'starting') {
                    statusElement.innerHTML = '<span class="material-icons" style="color: #ffc107; font-size: 14px;">circle</span> Starting';
                    statusElement.className = 'server-status starting';
                    terminalInfo.status = 'starting';
                } else if (status === 'running') {
                    statusElement.innerHTML = '<span class="material-icons" style="color: #4caf50; font-size: 14px;">circle</span> Running';
                    statusElement.className = 'server-status running';
                    terminalInfo.status = 'running';
                } else if (status === 'terminating') {
                    statusElement.innerHTML = '<span class="material-icons" style="color: #ffc107; font-size: 14px;">circle</span> Terminating';
                    statusElement.className = 'server-status starting';
                    terminalInfo.status = 'terminating';
                } else if (status === 'stopped') {
                    statusElement.textContent = 'Stopped';
                    statusElement.className = 'server-status stopped';

                    // Update terminal info
                    terminalInfo.status = 'stopped';



                    // Change stop button to start button
                    if (stopBtn) {
                        // Clone button to remove old event listeners
                        const newStartBtn = stopBtn.cloneNode(true);
                        stopBtn.parentNode.replaceChild(newStartBtn, stopBtn);

                        newStartBtn.textContent = 'Start';
                        newStartBtn.className = 'server-btn start-btn';
                        newStartBtn.id = `start-btn-${windowId}`;

                        // Add restart listener
                        newStartBtn.addEventListener('click', () => {
                            this.restartServer(windowId, terminalInfo.modelPath, terminalInfo.modelName);
                        });
                    }
                }
            }
        }

// Update chat icon state based on server status
        const chatTab = document.getElementById(`tab-chat-${windowId}`);
        if (chatTab) {
            const icon = chatTab.querySelector('.material-icons');
            if (status === 'running') {
                chatTab.style.opacity = '1';
                chatTab.style.pointerEvents = 'auto';
                chatTab.classList.add('pulse-animation');
                if (icon) icon.style.color = '#4caf50';
            } else {
                chatTab.style.opacity = '0.5';
                chatTab.style.pointerEvents = 'none';
                chatTab.classList.remove('pulse-animation');
                if (icon) icon.style.color = '';

                // If we are currently on the chat tab and server stops, switch to terminal
                if (chatTab.classList.contains('active')) {
                    this.switchTab(windowId, 'terminal');
                }
            }
        }
    }

    async stopServer(processId, windowId, modelPath, modelName) {
        try {
            // Use Tauri command instead of fetch
            const invoke = this.getInvoke();
            if (!invoke) {
                console.error('Tauri invoke not available for process termination');
                return;
            }
            await invoke('kill_process', { processId: processId });

            this.updateServerStatus(windowId, 'stopped', 0);
            // this.desktop.showNotification(`${modelName} stopped`, 'info');


        } catch (error) {
            console.error('Error stopping server:', error);
            // this.desktop.showNotification(`Error stopping server: ${error.message}`, 'error');
        }
    }

    async startServer(windowId, modelPath, modelName) {
        const terminalInfo = this.terminals.get(windowId);
        if (!terminalInfo) return;

        try {
            // Use Tauri command instead of fetch
            const invoke = this.getInvoke();
            if (!invoke) {
                console.error('Tauri invoke not available for model restart');
                return;
            }
            
            // Use the same arguments that were used for the original launch
            let result;
            if (terminalInfo.launchArgs) {
                console.log(`Restarting with stored arguments: ${terminalInfo.launchArgs}`);
                // Temporarily update the model config with the stored arguments
                const currentConfig = await invoke('get_model_settings', { modelPath: modelPath });
                const originalArgs = currentConfig.custom_args;
                
                // Update config with launch args
                await invoke('update_model_settings', { 
                    modelPath: modelPath, 
                    config: { ...currentConfig, custom_args: terminalInfo.launchArgs }
                });
                
                // Launch the model
                result = await invoke('launch_model', { modelPath: modelPath });
                
                // Restore original args
                await invoke('update_model_settings', { 
                    modelPath: modelPath, 
                    config: { ...currentConfig, custom_args: originalArgs }
                });
            } else {
                console.log('Restarting with default configuration');
                result = await invoke('launch_model', { modelPath: modelPath });
            }

            if (result.success) {
                // Update terminal info with new process ID
                terminalInfo.processId = result.process_id;
                terminalInfo.host = result.server_host;
                terminalInfo.port = result.server_port;
                terminalInfo.status = 'starting';

                // Update chat iframe URL with new host:port
                const chatPanel = document.getElementById(`panel-chat-${windowId}`);
                if (chatPanel) {
                    const iframe = chatPanel.querySelector('iframe');
                    if (iframe) {
                        const newUrl = `http://${result.server_host}:${result.server_port}`;
                        console.log(`Updating chat iframe URL to: ${newUrl}`);
                        iframe.src = newUrl;
                    }
                }

                // Update UI
                const window = this.desktop.windows.get(windowId);
                if (window) {
                    const statusElement = window.querySelector('.server-status');
                    const startBtn = window.querySelector('.start-btn');
                    const serverDetails = window.querySelector('.server-details');

                    if (statusElement) {
                        statusElement.innerHTML = '<span class="material-icons" style="color: #ffc107; font-size: 14px;">circle</span> Starting';
                        statusElement.className = 'server-status starting';
                    }

                    if (serverDetails) {
                        serverDetails.innerHTML = `${modelName} - <span class="clickable" style="cursor: pointer; text-decoration: underline;" onclick="terminalManager.openUrl('http://${result.server_host}:${result.server_port}')">${result.server_host}:${result.server_port}</span><button class="copy-link-btn" style="background: none; border: none; cursor: pointer; margin-left: 5px; padding: 0; font-size: 14px; vertical-align: middle;" onclick="terminalManager.copyToClipboard('http://${result.server_host}:${result.server_port}', this)" title="Copy link"><span class="material-icons" style="font-size: 14px; color: var(--theme-text-muted);">content_copy</span></button>`;
                    }

                    // Change start button back to stop button
                    if (startBtn) {
                        startBtn.textContent = 'Stop';
                        startBtn.className = 'server-btn stop-btn';
                        startBtn.id = `stop-btn-${windowId}`;
                        // Remove any existing event listeners
                        const newStartBtn = startBtn.cloneNode(true);
                        startBtn.parentNode.replaceChild(newStartBtn, startBtn);
                        // Add event listener
                        newStartBtn.addEventListener('click', () => {
                            this.stopServer(result.process_id, windowId, modelPath, modelName);
                        });
                    }

                    // Add restart message to output
                    const outputDiv = window.querySelector(`[id^="server-output-"]`);
                    if (outputDiv) {
                        const separator = document.createElement('div');
                        separator.className = 'server-line server-separator';
                        separator.textContent = '--- Restarting Server ---';
                        outputDiv.appendChild(separator);

                        const restartDiv = document.createElement('div');
                        restartDiv.className = 'server-line server-system';
                        restartDiv.textContent = `Restarting ${modelName}...`;
                        outputDiv.appendChild(restartDiv);

                        const processDiv = document.createElement('div');
                        processDiv.className = 'server-line server-system';
                        processDiv.textContent = `New Process ID: ${result.process_id}`;
                        outputDiv.appendChild(processDiv);

                        const serverDiv = document.createElement('div');
                        serverDiv.className = 'server-line server-system';
                        serverDiv.textContent = `Server: ${result.server_host}:${result.server_port}`;
                        outputDiv.appendChild(serverDiv);

                        outputDiv.scrollTop = outputDiv.scrollHeight;
                    }
                }

                // Start polling for new output
                this.startServerOutputPolling(result.process_id, windowId);
                
                // Set up health check after restart (integrated retries handle slow loading)
                setTimeout(() => {
                    this.checkServerHealth(windowId, result.server_host, result.server_port, modelName);
                }, 2000);
                
                // this.desktop.showNotification(`${modelName} restarted`, 'success');
            } else {
                throw new Error(result.error || 'Failed to launch model');
            }
        } catch (error) {
            console.error('Error restarting server:', error);
            // this.desktop.showNotification(`Failed to restart ${modelName}: ${error.message}`, 'error');
        }
    }

    // Proper restart functionality that stops then starts
    async restartServer(windowId, modelPath, modelName) {
        console.log(`🔄 [INDIVIDUAL SERVER RESTART] Starting restart for ${modelName} (window: ${windowId})`);

        const terminalInfo = this.terminals.get(windowId);
        if (!terminalInfo) {
            console.warn(`No terminal info found for window ${windowId}`);
            console.log(`🆕 [NEW SERVER START] No existing terminal, starting fresh server for ${modelName}`);
            return this.startServer(windowId, modelPath, modelName);
        }

        try {
            console.log(`🔄 [RESTART SEQUENCE] Restarting server for ${modelName}...`);

            // First, stop the existing process if it's running or starting
            if (terminalInfo.processId && (terminalInfo.status === 'running' || terminalInfo.status === 'starting')) {
                console.log(`🛑 [STOP PHASE] Stopping existing process ${terminalInfo.processId}`);
                await this.stopServer(terminalInfo.processId, windowId, modelPath, modelName);

                // Wait a moment for the process to fully stop
                console.log(`⏱️ [WAIT PHASE] Waiting 500ms for process cleanup...`);
                await new Promise(resolve => setTimeout(resolve, 500));
                console.log(`✅ [WAIT COMPLETE] Ready to start new process`);
            } else {
                console.log(`ℹ️ [SKIP STOP] Process not running, proceeding to start`);
            }

            // Then start a new instance
            console.log(`▶️ [START PHASE] Starting new instance of ${modelName}`);
            const result = await this.startServer(windowId, modelPath, modelName);
            console.log(`🎉 [RESTART COMPLETE] Successfully restarted ${modelName}`);
            return result;

        } catch (error) {
            console.error('❌ [RESTART ERROR] Error in restart sequence:', error);
            // this.desktop.showNotification(`Failed to restart ${modelName}: ${error.message}`, 'error');
        }
    }

switchTab(windowId, tabName) {
        const terminalTab = document.getElementById(`tab-terminal-${windowId}`);
        const chatTab = document.getElementById(`tab-chat-${windowId}`);
        const terminalPanel = document.getElementById(`panel-terminal-${windowId}`);
        const chatPanel = document.getElementById(`panel-chat-${windowId}`);

        // Remove active class from all tabs and panels
        terminalTab?.classList.remove('active');
        chatTab?.classList.remove('active');
        terminalPanel?.classList.remove('active');
        chatPanel?.classList.remove('active');

        if (tabName === 'terminal') {
            terminalTab?.classList.add('active');
            terminalPanel?.classList.add('active');

            // Fix: Scroll to bottom when switching back to terminal
            const outputDiv = document.getElementById(`server-output-${windowId}`);
            if (outputDiv) {
                // Use a small timeout to ensure the display: flex has taken effect
                setTimeout(() => {
                    outputDiv.scrollTop = outputDiv.scrollHeight;
                }, 50);
            }
        } else if (tabName === 'chat') {
            chatTab?.classList.add('active');
            chatPanel?.classList.add('active');
        }
    }

    toggleAutoSwitch(windowId) {
        // Toggle the global auto-switch setting
        this.autoSwitchEnabled = !this.autoSwitchEnabled;
        
        // Save to localStorage
        localStorage.setItem('terminalAutoSwitch', this.autoSwitchEnabled.toString());
        
        // Update all auto-switch buttons across all terminals
        const allAutoSwitchButtons = document.querySelectorAll('.auto-switch-btn');
        allAutoSwitchButtons.forEach(btn => {
            const icon = btn.querySelector('.material-icons');
            if (this.autoSwitchEnabled) {
                btn.classList.add('active');
                btn.title = 'Auto-switch to chat: ON';
                if (icon) icon.textContent = 'toggle_on';
            } else {
                btn.classList.remove('active');
                btn.title = 'Auto-switch to chat: OFF';
                if (icon) icon.textContent = 'toggle_off';
            }
        });
        
        console.log(`Auto-switch ${this.autoSwitchEnabled ? 'enabled' : 'disabled'}`);
    }

    openNativeChatForServer(modelName, host, port) {
        const url = `http://${host}:${port}`;
        const windowId = `native_chat_${Date.now()}`; // Unique ID for each window

        // Setup iframe content
        // We use an iframe that takes up the full window content and ensure it has white background
        const content = `
            <div style="width: 100%; height: 100%; display: flex; flex-direction: column; background: white;">
                <iframe src="${url}" frameBorder="0" style="flex: 1; border: none; width: 100%; height: 100%;"></iframe>
            </div>
        `;

        // Create the window with host and port in title
        this.desktop.createWindow(windowId, `Native Chat - ${modelName} (${host}:${port})`, 'browser-window', content);

        // Apply some specific styles if needed (desktop.js handles basic window creation)
        const windowElement = this.desktop.windows.get(windowId);
        if (windowElement) {
            // Make it a decent size by default
            windowElement.style.width = '1000px';
            windowElement.style.height = '800px';

            // Center it roughly
            const left = (window.innerWidth - 1000) / 2;
            const top = (window.innerHeight - 800) / 2;
            windowElement.style.left = `${Math.max(50, left)}px`;
            windowElement.style.top = `${Math.max(50, top)}px`;

            // Bring to front
            windowElement.style.zIndex = this.desktop.windowZIndex + 1;
            this.desktop.windowZIndex += 1;

            // Add taskbar item for this window with host and port in title
            this.desktop.addTaskbarItem(`Native Chat - ${modelName} (${host}:${port})`, windowId, '<span class="material-icons">open_in_browser</span>');

            // Handle focus/click to bring to front
            // Since iframe consumes clicks, we monitor window blur which happens when clicking into iframe
            const iframe = windowElement.querySelector('iframe');
            if (iframe) {
                const blurHandler = () => {
                    if (document.activeElement === iframe) {
                        // Bring this window to front
                        windowElement.style.zIndex = ++this.desktop.windowZIndex;

                        // Update visual active state if possible (DesktopManager doesn't expose a clean method for this but we can try)
                        // This mimics what happens in desktop.js mousedown handler
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
    }



    // Session management methods
    async saveTerminalState(windowId, terminalData) {
        try {
            const response = await fetch('/api/session/terminal', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({
                    windowId: windowId,
                    processId: terminalData.processId,
                    modelName: terminalData.modelName,
                    modelPath: terminalData.modelPath,
                    host: terminalData.host,
                    port: terminalData.port,
                    status: terminalData.status,
                    output: terminalData.output || [],
                    activeVersion: terminalData.activeVersion || ''
                })
            });

            if (!response.ok) {
                throw new Error('Failed to save terminal state');
            }
        } catch (error) {
            console.error('Error saving terminal state:', error);
        }
    }

    // Check for existing terminal for a model
    getExistingTerminal(modelPath) {
        return Array.from(this.terminals.entries()).find(([windowId, terminalInfo]) =>
            terminalInfo.modelPath === modelPath
        );
    }

    // Get terminal data for session restoration
    getTerminalData(windowId) {
        return this.terminals.get(windowId);
    }

    // Remove terminal from tracking
    removeTerminal(windowId) {
        this.terminals.delete(windowId);
    }

    // Get all terminals for session management
    getAllTerminals() {
        return this.terminals;
    }

    // Get active terminal IDs for cleanup
    getActiveTerminals() {
        const activeTerminals = [];
        for (const [windowId, terminalInfo] of this.terminals.entries()) {
            if ((terminalInfo.status === 'running' || terminalInfo.status === 'starting') && terminalInfo.processId) {
                activeTerminals.push(windowId);
            }
        }
        return activeTerminals;
    }

    // Close a specific terminal and its process
    async closeTerminal(windowId) {
        const terminalInfo = this.terminals.get(windowId);

        // Remove blur event listener if it exists
        if (terminalInfo && terminalInfo.blurHandler) {
            window.removeEventListener('blur', terminalInfo.blurHandler);
            terminalInfo.blurHandler = null;
        }

        if (terminalInfo && terminalInfo.processId && (terminalInfo.status === 'running' || terminalInfo.status === 'starting')) {
            console.log(`📺 Closing terminal ${windowId} with process ${terminalInfo.processId}`);
            try {
                await this.stopServer(terminalInfo.processId, windowId, terminalInfo.modelPath, terminalInfo.modelName);
                console.log(`✅ Successfully closed terminal ${windowId}`);

                // Disconnect any chat sessions connected to this server

            } catch (error) {
                console.error(`❌ Failed to close terminal ${windowId}:`, error);
            }
        }
this.terminals.delete(windowId);
    }

    // Method to open URL in default browser
    async openUrl(url) {
        try {
            // Use desktop manager's openUrl method
            await this.desktop.openUrl(url);
        } catch (error) {
            console.error('Error opening URL:', error);
        }
    }

    // Method to copy text to clipboard
    async copyToClipboard(text, buttonElement) {
        try {
            await navigator.clipboard.writeText(text);
            // Show visual feedback
            const originalIcon = buttonElement.innerHTML;
            buttonElement.innerHTML = '<span class="material-icons" style="font-size: 14px; color: #4caf50;">check</span>';
            setTimeout(() => {
                buttonElement.innerHTML = originalIcon;
            }, 2000);
        } catch (error) {
            console.error('Error copying to clipboard:', error);
            // Show error feedback
            const originalIcon = buttonElement.innerHTML;
            buttonElement.innerHTML = '<span class="material-icons" style="font-size: 14px; color: #f44336;">error</span>';
            setTimeout(() => {
                buttonElement.innerHTML = originalIcon;
            }, 2000);
        }
    }

    // Terminal restoration and session management methods
    async restoreTerminalsAndWindows() {
        if (!this.desktop.sessionData) {
            console.log('No session data available for restoration');
            return;
        }

        if (this.desktop.restorationInProgress) {
            console.log('Restoration already in progress, skipping duplicate call');
            return;
        }

        this.desktop.restorationInProgress = true;
        console.log('Starting terminal and window restoration...');
        console.log('Session data terminals:', this.desktop.sessionData.terminals);
        console.log('Session data windows:', this.desktop.sessionData.windows);

        // First restore terminals data
        for (const [windowId, terminalData] of Object.entries(this.desktop.sessionData.terminals || {})) {
            console.log('Loading terminal data for', windowId, ':', terminalData);
            console.log('Terminal output length:', terminalData.output ? terminalData.output.length : 'no output');

            this.terminals.set(windowId, terminalData);
            // Check if process is still running - this will update the status
            await this.checkTerminalProcess(windowId, terminalData);
        }

        // Then restore windows - restore terminals regardless of visibility, other windows only if visible
        for (const [windowId, windowData] of Object.entries(this.desktop.sessionData.windows || {})) {
            console.log('Processing window restoration for:', windowId, windowData);

            if (windowData.type === 'terminal') {
                // Always restore terminal windows (whether they were visible or minimized)
                const terminalData = this.getTerminalData(windowId);
                console.log('Terminal data for window restoration:', terminalData);
                if (terminalData && (terminalData.status === 'running' || terminalData.status === 'starting')) {
                    console.log('Restoring terminal window for running/starting process:', windowId, windowData);
                    this.desktop.restoreWindow(windowId, windowData);
                } else {
                    console.log('Skipping terminal window restoration - process not running:', windowId,
                        terminalData ? `status: ${terminalData.status}` : 'no terminal data');
                    // Remove the non-running terminal from session
                    await this.desktop.removeWindowFromSession(windowId);
                }
            }
        }

        console.log('Terminal and window restoration complete');
        this.desktop.restorationInProgress = false;
    }

    async checkTerminalProcess(windowId, terminalData) {
        if (!terminalData.processId) {
            terminalData.status = 'stopped';
            this.terminals.set(windowId, terminalData);
            console.log(`Terminal ${windowId} has no processId, marked as stopped`);
            return;
        }

        try {
            console.log(`Checking process status for ${windowId} with processId: ${terminalData.processId}`);
            // Use Tauri command instead of fetch API
            const invoke = this.getInvoke();
            const result = await invoke('get_process_output', { processId: terminalData.processId });
            const newStatus = result.is_running ? (terminalData.status === 'starting' ? 'starting' : 'running') : 'stopped';
            console.log(`Process ${terminalData.processId} status: ${newStatus}`);
            terminalData.status = newStatus;
            this.terminals.set(windowId, terminalData);
        } catch (error) {
            console.warn(`Error checking terminal process ${terminalData.processId}:`, error);
            // If the process is not found or another error, mark as stopped
            terminalData.status = 'stopped';
            this.terminals.set(windowId, terminalData);
        }
    }

    restoreTerminalWindow(windowId, terminalData, windowData) {
        console.log('Restoring terminal window with data:', terminalData);
        console.log('Terminal output length:', terminalData.output ? terminalData.output.length : 'no output');

        const content = `
            <div class="server-terminal-container">
                <div class="server-main-content">
                    <div class="server-tab-panel active" id="panel-terminal-${windowId}">
                        <div class="server-info">
                            <span class="server-status ${terminalData.status}">
                                <span class="material-icons" style="color: ${terminalData.status === 'running' ? '#4caf50' : terminalData.status === 'starting' ? '#ffc107' : '#f44336'}; font-size: 14px;">circle</span>
                                ${terminalData.status}
                            </span>
                            <span class="server-details">${terminalData.modelName} - <span class="clickable" style="cursor: pointer; text-decoration: underline;" onclick="terminalManager.openUrl('http://${terminalData.host}:${terminalData.port}')">${terminalData.host}:${terminalData.port}</span><button class="copy-link-btn" style="background: none; border: none; cursor: pointer; margin-left: 5px; padding: 0; font-size: 14px; vertical-align: middle;" onclick="terminalManager.copyToClipboard('http://${terminalData.host}:${terminalData.port}', this)" title="Copy link"><span class="material-icons" style="font-size: 14px; color: var(--theme-text-muted);">content_copy</span></button></span>
                            <div class="server-controls">
                                <button class="server-btn auto-switch-btn ${this.autoSwitchEnabled ? 'active' : ''}" id="auto-switch-btn-${windowId}" onclick="terminalManager.toggleAutoSwitch('${windowId}')" title="${this.autoSwitchEnabled ? 'Auto-switch to chat: ON' : 'Auto-switch to chat: OFF'}"><span class="material-icons">${this.autoSwitchEnabled ? 'toggle_on' : 'toggle_off'}</span></button>
                                ${terminalData.status === 'running' || terminalData.status === 'starting' ?
                `<button class="server-btn stop-btn" onclick="terminalManager.stopServer('${terminalData.processId}', '${windowId}', '${terminalData.modelPath}', '${terminalData.modelName}')"><span class="material-icons">stop</span> Stop</button>` :
                `<button class="server-btn start-btn" onclick="terminalManager.restartServer('${windowId}', '${terminalData.modelPath}', '${terminalData.modelName}')"><span class="material-icons">play_arrow</span> Start</button>`
            }
                            </div>
                        </div>
                        <div class="server-output" id="server-output-${windowId}">
                            <div class="server-line">Restored ${terminalData.modelName} session</div>
                            <div class="server-line">Process ID: ${terminalData.processId}</div>
                            <div class="server-line">Server: <span class="clickable" style="cursor: pointer; text-decoration: underline;" onclick="terminalManager.openUrl('http://${terminalData.host}:${terminalData.port}')">${terminalData.host}:${terminalData.port}</span><button class="copy-link-btn" style="background: none; border: none; cursor: pointer; margin-left: 5px; padding: 0; font-size: 14px; vertical-align: middle;" onclick="terminalManager.copyToClipboard('http://${terminalData.host}:${terminalData.port}', this)" title="Copy link"><span class="material-icons" style="font-size: 14px; color: var(--theme-text-muted);">content_copy</span></button></div>
                            <div class="server-line">Output lines: ${terminalData.output ? terminalData.output.length : 0}</div>
                            ${terminalData.output && terminalData.output.length > 0 ? terminalData.output.map(line =>
                `<div class="server-line">${line.toString().replace(/ /g, '&nbsp;')}</div>`
            ).join('') : '<div class="server-line">No saved output found</div>'}
                        </div>
                    </div>
                    <div class="server-tab-panel" id="panel-chat-${windowId}" style="background: white;">
                        <iframe src="http://${terminalData.host}:${terminalData.port}" frameBorder="0" style="width: 100%; height: 100%; border: none;"></iframe>
                    </div>
                </div>
            </div>
        `;

        const window = this.desktop.createWindow(windowId, `Server - ${terminalData.modelName}`, 'server-terminal-window', content);

        // Inject tabs into header
        const header = window.querySelector('.window-header');
        if (header) {
            const tabsHtml = `
                <div class="server-tabs header-tabs">
                    <div class="server-tab active" id="tab-terminal-${windowId}" onclick="terminalManager.switchTab('${windowId}', 'terminal')" title="Terminal Output">
                        <span class="material-icons">terminal</span>
                    </div>
                    <div class="server-tab" id="tab-chat-${windowId}" onclick="terminalManager.switchTab('${windowId}', 'chat')" title="Native Chat" style="${terminalData.status === 'running' ? 'opacity: 1; pointer-events: auto;' : 'opacity: 0.5; pointer-events: none;'}" class="${terminalData.status === 'running' ? 'server-tab pulse-animation' : 'server-tab'}">
                        <span class="material-icons" style="${terminalData.status === 'running' ? 'color: #4caf50;' : ''}">chat</span>
                    </div>
                </div>
            `;
            const titleElement = header.querySelector('.window-title');
            if (titleElement) {
                titleElement.insertAdjacentHTML('afterend', tabsHtml);
            }
        }

        this.desktop.addTaskbarItem(`Server - ${terminalData.modelName}`, windowId, '<span class="material-icons">computer</span>');

        // Restore window position and size
        if (windowData.position) {
            window.style.left = windowData.position.x + 'px';
            window.style.top = windowData.position.y + 'px';
        }
        if (windowData.size) {
            window.style.width = windowData.size.width + 'px';
            window.style.height = windowData.size.height + 'px';
            // Store the dimensions for proper size saving when minimized
            window.dataset.savedWidth = windowData.size.width.toString();
            window.dataset.savedHeight = windowData.size.height.toString();
        }
        if (windowData.zIndex) {
            window.style.zIndex = windowData.zIndex;
            this.desktop.windowZIndex = Math.max(this.desktop.windowZIndex, windowData.zIndex);
        }

        // Always start minimized during restoration
        this.desktop.minimizeWindow(windowId);

        // Scroll terminal to bottom after restoration
        setTimeout(() => {
            const outputDiv = document.getElementById(`server-output-${windowId}`);
            if (outputDiv) {
                outputDiv.scrollTop = outputDiv.scrollHeight;
            }
        }, 100);

        // Resume output polling if process is still running or starting
        if ((terminalData.status === 'running' || terminalData.status === 'starting') && terminalData.processId) {
            this.startServerOutputPolling(terminalData.processId, windowId);
        }
    }

    async closeAllTerminalSessions() {
        try {
            // Get all terminal windows from session state
            const terminalWindows = Object.values(this.desktop.windows).filter(window =>
                window && (window.type === 'terminal' || window.id.includes('terminal'))
            );

            // Get all active terminals from terminal manager
            const activeTerminals = this.getActiveTerminals();

            console.log(`Found ${terminalWindows.length} terminal windows and ${activeTerminals.length} active terminals`);

            // Close all terminal processes
            const closePromises = [];

            // Close terminals via terminal manager if available
            if (activeTerminals.length > 0) {
                for (const terminalId of activeTerminals) {
                    try {
                        const promise = this.closeTerminal(terminalId);
                        if (promise && typeof promise.then === 'function') {
                            closePromises.push(promise);
                        }
                    } catch (error) {
                        console.error(`Error closing terminal ${terminalId}:`, error);
                    }
                }
            }

            // Also close terminal windows via window manager
            for (const window of terminalWindows) {
                try {
                    if (window.processId) {
                        // Kill the process via API
                        const killPromise = fetch(`/api/process/${window.processId}/kill`, {
                            method: 'POST',
                            headers: {
                                'Content-Type': 'application/json'
                            }
                        }).catch(error => console.error(`Error killing process ${window.processId}:`, error));

                        closePromises.push(killPromise);
                    }

                    // Close the window
                    this.desktop.closeWindow(window.id);
                } catch (error) {
                    console.error(`Error closing terminal window ${window.id}:`, error);
                }
            }

            // Wait for all close operations to complete (with timeout)
            if (closePromises.length > 0) {
                await Promise.allSettled(closePromises);
                // Give processes a moment to fully terminate
                await new Promise(resolve => setTimeout(resolve, 1000));
            }

            console.log('All terminal sessions closed successfully');

        } catch (error) {
            console.error('Error closing terminal sessions:', error);
            // Don't throw - we want restart to continue even if terminal cleanup fails
        }
    }
}

// Debug: Confirm TerminalManager class is loaded
console.log('TerminalManager class loaded successfully');
window.TerminalManager = TerminalManager;
