// Chat Application Module
// Modern ChatGPT/Claude-style chat interface integrated into terminal windows
// Uses Tauri commands for backend communication

const TEXTAREA_MIN_HEIGHT = 128;
const TEXTAREA_MAX_HEIGHT = 300;

class ChatApp {
    constructor(terminalManager, processId, modelPath, modelName) {
        this.terminalManager = terminalManager;
        this.processId = processId;
        this.modelPath = modelPath;
        this.modelName = modelName;
        this.currentChatId = null;
        this.messages = [];
        this.chats = [];
        this.streamingListeners = new Map();
        this.isStreaming = false;
        this.titleGenerated = false;
        this.parameters = {
            temperature: 0.7,
            top_p: 0.9,
            top_k: 40,
            max_tokens: 2048,
            context_length: 4096
        };
        
        // Initialize Tauri API access
        this.invoke = null;
        this.listen = null;
        this.initTauriAPI();
    }
    
    initTauriAPI() {
        try {
            if (window.__TAURI__ && window.__TAURI__.core) {
                this.invoke = window.__TAURI__.core.invoke;
                console.log('Tauri API initialized in ChatApp');
            } else {
                console.warn('Tauri API not available yet');
            }
            
            if (window.__TAURI__ && window.__TAURI__.event) {
                this.listen = window.__TAURI__.event.listen;
            }
        } catch (error) {
            console.error('Failed to initialize Tauri API in ChatApp:', error);
        }
    }
    
    getInvoke() {
        if (!this.invoke) {
            this.initTauriAPI();
        }
        return this.invoke;
    }
    
    getListen() {
        if (!this.listen) {
            this.initTauriAPI();
        }
        return this.listen;
    }
    
    async init() {
        console.log('Initializing ChatApp for model:', this.modelName);
        
        try {
            // Load existing chats
            await this.refreshChatList();
            
            // If no chats exist, create a new one
            if (this.chats.length === 0) {
                await this.createNewChat();
            } else {
                // Load the most recent chat
                await this.loadChat(this.chats[0].id);
            }
            
            // Initialize event listeners after a short delay to ensure DOM is ready
            setTimeout(() => {
                this.attachEventListeners();
                this.setupTextareaAutoResize();
                // Ensure input is enabled after init
                this.setInputEnabled(true);
                console.log('Event listeners attached, input enabled');
            }, 100);
            
            console.log('ChatApp initialized successfully');
        } catch (error) {
            console.error('Error initializing ChatApp:', error);
            this.showNotification('Failed to initialize chat', 'error');
        }
    }
    
    render() {
        return `
            <div class="chat-app-container" id="chat-app-${this.processId}">
                <!-- Left Sidebar: Chat History -->
                <div class="chat-sidebar">
                    <div class="chat-sidebar-header">
                        <button class="new-chat-btn" id="new-chat-btn-${this.processId}">
                            <span class="material-icons">add</span>
                            New Chat
                        </button>
                    </div>
                    <div class="chat-list" id="chat-list-${this.processId}">
                        <!-- Chat items will be rendered here -->
                    </div>
                </div>
                
                <!-- Main Chat Area -->
                <div class="chat-main">
                    <div class="chat-header">
                        <div class="model-info">
                            <span class="model-name">${this.modelName}</span>
                            <span class="model-status online">● Online</span>
                        </div>
                        <button class="settings-btn" id="settings-btn-${this.processId}" title="Chat Settings">
                            <span class="material-icons">settings</span>
                        </button>
                    </div>
                    
                    <div class="chat-messages" id="chat-messages-${this.processId}">
                        <div class="chat-messages-spacer"></div>
                        <div class="welcome-message" id="welcome-message-${this.processId}">
                            <h2>How can I help you today?</h2>
                            <p>Start a conversation with ${this.modelName}</p>
                        </div>
                    </div>
                    
                    <div class="chat-input-area" id="chat-input-area-${this.processId}">
                        <div class="input-container">
                            <textarea
                                class="chat-input"
                                id="chat-input-${this.processId}"
                                placeholder="Type your message..."
                                rows="4"
                        ></textarea>
                            <button class="send-btn" id="send-btn-${this.processId}">
                                <span class="material-icons">send</span>
                            </button>
                        </div>
                        <div class="input-footer">
                            <span class="model-indicator">Using: ${this.modelName}</span>
                            <span class="streaming-indicator hidden" id="streaming-indicator-${this.processId}">
                                <span class="material-icons spinning">refresh</span>
                                Thinking...
                            </span>
                        </div>
                    </div>
                </div>
                
                <!-- Settings Panel (Slide-out from right) -->
                <div class="chat-settings-panel hidden" id="settings-panel-${this.processId}">
                    <div class="settings-header">
                        <h3>Chat Settings</h3>
                        <button class="close-settings" id="close-settings-${this.processId}">×</button>
                    </div>
                    <div class="settings-content">
                        ${this.renderParameterSliders()}
                    </div>
                </div>
            </div>
        `;
    }
    
    renderParameterSliders() {
        return `
            <div class="parameter-group">
                <label for="param-temperature-${this.processId}">
                    Temperature: <span id="temp-value-${this.processId}">${this.parameters.temperature}</span>
                </label>
                <input type="range" 
                    id="param-temperature-${this.processId}" 
                    min="0" max="2" step="0.1" 
                    value="${this.parameters.temperature}"
                    class="param-slider"
                >
                <div class="param-hint">Controls randomness: 0 = deterministic, 2 = very creative</div>
            </div>
            
            <div class="parameter-group">
                <label for="param-top-p-${this.processId}">
                    Top P: <span id="top-p-value-${this.processId}">${this.parameters.top_p}</span>
                </label>
                <input type="range" 
                    id="param-top-p-${this.processId}" 
                    min="0" max="1" step="0.05" 
                    value="${this.parameters.top_p}"
                    class="param-slider"
                >
                <div class="param-hint">Nucleus sampling: lower = more focused</div>
            </div>
            
            <div class="parameter-group">
                <label for="param-top-k-${this.processId}">
                    Top K: <span id="top-k-value-${this.processId}">${this.parameters.top_k}</span>
                </label>
                <input type="range" 
                    id="param-top-k-${this.processId}" 
                    min="1" max="100" step="1" 
                    value="${this.parameters.top_k}"
                    class="param-slider"
                >
                <div class="param-hint">Limits vocabulary: lower = more focused</div>
            </div>
            
            <div class="parameter-group">
                <label for="param-max-tokens-${this.processId}">
                    Max Tokens: <span id="max-tokens-value-${this.processId}">${this.parameters.max_tokens}</span>
                </label>
                <input type="range" 
                    id="param-max-tokens-${this.processId}" 
                    min="256" max="8192" step="256" 
                    value="${this.parameters.max_tokens}"
                    class="param-slider"
                >
                <div class="param-hint">Maximum response length</div>
            </div>
            
            <div class="parameter-group">
                <label for="param-context-${this.processId}">
                    Context Length: <span id="context-value-${this.processId}">${this.parameters.context_length}</span>
                    <span class="restart-badge">Restart Required</span>
                </label>
                <input type="range" 
                    id="param-context-${this.processId}" 
                    min="512" max="32768" step="512" 
                    value="${this.parameters.context_length}"
                    class="param-slider"
                >
                <div class="param-hint">Conversation memory size</div>
            </div>
            
            <button class="save-params-btn" id="save-params-btn-${this.processId}">
                <span class="material-icons">save</span>
                Save Settings
            </button>
        `;
    }
    
    renderChatList() {
        const chatListEl = document.getElementById(`chat-list-${this.processId}`);
        if (!chatListEl) return;
        
        // Group chats by date
        const groups = this.groupChatsByDate();
        
        let html = '';
        for (const [groupName, chats] of Object.entries(groups)) {
            if (chats.length === 0) continue;
            
            html += `
                <div class="chat-group">
                    <div class="chat-group-header">${groupName}</div>
            `;
            
            chats.forEach(chat => {
                const isActive = chat.id === this.currentChatId;
                const date = new Date(chat.updated_at || chat.created_at);
                const timeStr = date.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
                
                html += `
                    <div class="chat-item ${isActive ? 'active' : ''}" 
                         data-chat-id="${chat.id}"
                         id="chat-item-${chat.id}">
                        <div class="chat-item-title">${this.escapeHtml(chat.title || 'Untitled Chat')}</div>
                        <div class="chat-item-meta">
                            <span class="chat-item-time">${timeStr}</span>
                            <button class="chat-delete-btn" data-chat-id="${chat.id}" title="Delete chat">
                                <span class="material-icons">delete</span>
                            </button>
                        </div>
                    </div>
                `;
            });
            
            html += '</div>';
        }
        
        chatListEl.innerHTML = html;
    }
    
    groupChatsByDate() {
        const groups = {
            'Today': [],
            'Yesterday': [],
            'Last 7 Days': [],
            'Older': []
        };
        
        const now = new Date();
        const today = new Date(now.getFullYear(), now.getMonth(), now.getDate());
        const yesterday = new Date(today);
        yesterday.setDate(yesterday.getDate() - 1);
        const lastWeek = new Date(today);
        lastWeek.setDate(lastWeek.getDate() - 7);
        
        this.chats.forEach(chat => {
            const chatDate = new Date(chat.updated_at || chat.created_at);
            const chatDay = new Date(chatDate.getFullYear(), chatDate.getMonth(), chatDate.getDate());
            
            if (chatDay.getTime() === today.getTime()) {
                groups['Today'].push(chat);
            } else if (chatDay.getTime() === yesterday.getTime()) {
                groups['Yesterday'].push(chat);
            } else if (chatDay > lastWeek) {
                groups['Last 7 Days'].push(chat);
            } else {
                groups['Older'].push(chat);
            }
        });
        
        return groups;
    }
    
    renderMessages() {
        const messagesContainer = document.getElementById(`chat-messages-${this.processId}`);
        const welcomeMessage = document.getElementById(`welcome-message-${this.processId}`);
        if (!messagesContainer) return;
        
        // Show/hide welcome message
        if (welcomeMessage) {
            welcomeMessage.style.display = this.messages.length === 0 ? 'flex' : 'none';
        }
        
        // Clear existing messages (except welcome)
        const existingMessages = messagesContainer.querySelectorAll('.message');
        existingMessages.forEach(el => el.remove());
        
        // Render messages
        this.messages.forEach(msg => {
            this.addMessageToUIRaw(msg.role, msg.content, msg.id);
        });
        
        // Scroll to bottom
        this.scrollToBottom();
    }
    
    attachEventListeners() {
        const processId = this.processId;
        
        // New chat button
        const newChatBtn = document.getElementById(`new-chat-btn-${processId}`);
        if (newChatBtn) {
            newChatBtn.addEventListener('click', () => this.createNewChat());
        }
        
        // Send button
        const sendBtn = document.getElementById(`send-btn-${processId}`);
        if (sendBtn) {
            sendBtn.addEventListener('click', () => this.handleSendMessage());
        }
        
        // Chat input
        const chatInput = document.getElementById(`chat-input-${processId}`);
        if (chatInput) {
            chatInput.addEventListener('keypress', (e) => {
                if (e.key === 'Enter' && !e.shiftKey) {
                    e.preventDefault();
                    this.handleSendMessage();
                }
            });
            
            chatInput.addEventListener('input', () => {
                this.autoResizeTextarea(chatInput);
                this.updateSendButtonState();
                this.updateComposerPadding();
                // Auto-save draft after a short delay
                clearTimeout(this.draftSaveTimeout);
                this.draftSaveTimeout = setTimeout(() => this.saveDraft(), 500);
            });
        }
        
        // Settings button
        const settingsBtn = document.getElementById(`settings-btn-${processId}`);
        if (settingsBtn) {
            settingsBtn.addEventListener('click', () => this.openSettingsPanel());
        }
        
        // Close settings button
        const closeSettingsBtn = document.getElementById(`close-settings-${processId}`);
        if (closeSettingsBtn) {
            closeSettingsBtn.addEventListener('click', () => this.closeSettingsPanel());
        }
        
        // Chat list - click to switch
        const chatList = document.getElementById(`chat-list-${processId}`);
        if (chatList) {
            chatList.addEventListener('click', (e) => {
                const chatItem = e.target.closest('.chat-item');
                const deleteBtn = e.target.closest('.chat-delete-btn');
                
                if (deleteBtn) {
                    e.stopPropagation();
                    const chatId = deleteBtn.dataset.chatId;
                    this.deleteChat(chatId);
                } else if (chatItem) {
                    const chatId = chatItem.dataset.chatId;
                    this.switchChat(chatId);
                }
            });
        }
        
        // Parameter sliders
        this.attachParameterListeners();

        // Keep composer pinned on resize
        window.addEventListener('resize', () => this.updateComposerPadding());
    }
    
    attachParameterListeners() {
        const processId = this.processId;
        
        // Temperature
        const tempSlider = document.getElementById(`param-temperature-${processId}`);
        if (tempSlider) {
            tempSlider.addEventListener('input', (e) => {
                this.parameters.temperature = parseFloat(e.target.value);
                document.getElementById(`temp-value-${processId}`).textContent = this.parameters.temperature;
            });
        }
        
        // Top P
        const topPSlider = document.getElementById(`param-top-p-${processId}`);
        if (topPSlider) {
            topPSlider.addEventListener('input', (e) => {
                this.parameters.top_p = parseFloat(e.target.value);
                document.getElementById(`top-p-value-${processId}`).textContent = this.parameters.top_p;
            });
        }
        
        // Top K
        const topKSlider = document.getElementById(`param-top-k-${processId}`);
        if (topKSlider) {
            topKSlider.addEventListener('input', (e) => {
                this.parameters.top_k = parseInt(e.target.value);
                document.getElementById(`top-k-value-${processId}`).textContent = this.parameters.top_k;
            });
        }
        
        // Max Tokens
        const maxTokensSlider = document.getElementById(`param-max-tokens-${processId}`);
        if (maxTokensSlider) {
            maxTokensSlider.addEventListener('input', (e) => {
                this.parameters.max_tokens = parseInt(e.target.value);
                document.getElementById(`max-tokens-value-${processId}`).textContent = this.parameters.max_tokens;
            });
        }
        
        // Context Length
        const contextSlider = document.getElementById(`param-context-${processId}`);
        if (contextSlider) {
            contextSlider.addEventListener('input', (e) => {
                this.parameters.context_length = parseInt(e.target.value);
                document.getElementById(`context-value-${processId}`).textContent = this.parameters.context_length;
            });
        }
        
        // Save parameters button
        const saveParamsBtn = document.getElementById(`save-params-btn-${processId}`);
        if (saveParamsBtn) {
            saveParamsBtn.addEventListener('click', () => this.saveParameters());
        }
    }
    
    async createNewChat() {
        try {
            const invoke = this.getInvoke();
            if (!invoke) {
                throw new Error('Tauri API not available');
            }
            
            // create_chat returns ChatSession directly, not {success, chat_id} wrapper
            const session = await invoke('create_chat', {
                modelPath: this.modelPath,
                modelName: this.modelName
            });
            
            if (session && session.id) {
                this.currentChatId = session.id;
                this.messages = [];
                this.titleGenerated = false;
                
                // Enable input
                this.setInputEnabled(true);
                
                // Refresh chat list
                await this.refreshChatList();
                
                // Clear messages UI
                this.renderMessages();
                
                console.log('Created new chat:', session.id);
            } else {
                throw new Error('Failed to create chat - invalid session returned');
            }
        } catch (error) {
            console.error('Error creating new chat:', error);
            this.showNotification('Failed to create new chat', 'error');
        }
    }
    
    async loadChat(chatId) {
        try {
            const invoke = this.getInvoke();
            if (!invoke) {
                throw new Error('Tauri API not available');
            }
            
            // load_chat returns ChatSession directly, not {success} wrapper
            const session = await invoke('load_chat', { chatId });
            
            if (session && session.id) {
                this.currentChatId = chatId;
                this.messages = session.messages || [];
                this.parameters = session.parameters || this.parameters;
                this.titleGenerated = session.title_generated || false;
                
                // Enable input
                this.setInputEnabled(true);
                
                // Render messages
                this.renderMessages();
                
                // Restore any draft message
                this.restoreDraft();
                
                // Update active state in chat list
                this.updateActiveChatInList();
                
                console.log('Loaded chat:', chatId);
            } else {
                throw new Error('Failed to load chat - invalid session returned');
            }
        } catch (error) {
            console.error('Error loading chat:', error);
            this.showNotification('Failed to load chat', 'error');
        }
    }
    
    async sendMessage(content) {
        if (!content.trim() || !this.currentChatId || this.isStreaming) {
            return;
        }
        
        try {
            const invoke = this.getInvoke();
            if (!invoke) {
                throw new Error('Tauri API not available');
            }
            
            // Disable input while sending
            this.setInputEnabled(false);
            this.isStreaming = true;
            
            // Add user message to UI immediately
            const userMessageId = `msg-${Date.now()}-user`;
            this.addMessageToUI('user', content, userMessageId);
            
            // Clear input and draft
            const chatInput = document.getElementById(`chat-input-${this.processId}`);
            if (chatInput) {
                chatInput.value = '';
                this.autoResizeTextarea(chatInput);
            }
            this.clearDraft();
            
            // Show streaming indicator
            this.showStreamingIndicator(true);
            
            // Create placeholder for assistant response
            const assistantMessageId = `msg-${Date.now()}-assistant`;
            this.addMessageToUI('assistant', '', assistantMessageId);
            
            // Set up streaming listener
            await this.handleStreamingResponse(assistantMessageId);
            
            // Send message to backend
            await invoke('send_message', {
                chatId: this.currentChatId,
                content: content,
                stream: true
            });
            
        } catch (error) {
            console.error('Error sending message:', error);
            this.showNotification('Failed to send message', 'error');
            this.setInputEnabled(true);
            this.isStreaming = false;
            this.showStreamingIndicator(false);
        }
    }
    
    async handleStreamingResponse(messageId) {
        const listen = this.getListen();
        if (!listen) {
            console.error('Tauri event listener not available');
            return;
        }
        
        try {
            // Listen for streaming events
            const unlisten = await listen(`chat-stream-${this.currentChatId}`, (event) => {
                if (event.payload.done) {
                    // Streaming complete
                    this.finalizeStreamingMessage(messageId, event.payload);
                    unlisten();
                    this.streamingListeners.delete(messageId);
                } else if (event.payload.error) {
                    // Error occurred
                    console.error('Streaming error:', event.payload.error);
                    this.updateStreamingMessage(messageId, `\n\nError: ${event.payload.error}`);
                    this.finalizeStreamingMessage(messageId, event.payload);
                    unlisten();
                    this.streamingListeners.delete(messageId);
                } else {
                    // Append chunk
                    this.appendStreamingChunk(messageId, event.payload.chunk);
                }
            });
            
            // Store unlisten function for cleanup
            this.streamingListeners.set(messageId, unlisten);
            
        } catch (error) {
            console.error('Error setting up streaming listener:', error);
            this.showNotification('Failed to set up message streaming', 'error');
        }
    }
    
    addMessageToUI(role, content, messageId) {
        const messagesContainer = document.getElementById(`chat-messages-${this.processId}`);
        if (!messagesContainer) return;
        
        // Hide welcome message
        const welcomeMessage = document.getElementById(`welcome-message-${this.processId}`);
        if (welcomeMessage) {
            welcomeMessage.style.display = 'none';
        }
        
        const messageEl = document.createElement('div');
        messageEl.className = `message ${role}`;
        messageEl.id = messageId;
        
        const avatarIcon = role === 'user' ? 'person' : 'smart_toy';
        const avatarClass = role === 'user' ? 'user-avatar' : 'assistant-avatar';
        
        messageEl.innerHTML = `
            <div class="message-avatar ${avatarClass}">
                <span class="material-icons">${avatarIcon}</span>
            </div>
            <div class="message-content">
                <div class="message-text">${this.formatMessageContent(content)}</div>
            </div>
        `;
        
        messagesContainer.appendChild(messageEl);
        this.scrollToBottom();
    }
    
    addMessageToUIRaw(role, content, messageId) {
        const messagesContainer = document.getElementById(`chat-messages-${this.processId}`);
        if (!messagesContainer) return;
        
        const messageEl = document.createElement('div');
        messageEl.className = `message ${role}`;
        messageEl.id = messageId;
        
        const avatarIcon = role === 'user' ? 'person' : 'smart_toy';
        const avatarClass = role === 'user' ? 'user-avatar' : 'assistant-avatar';
        
        messageEl.innerHTML = `
            <div class="message-avatar ${avatarClass}">
                <span class="material-icons">${avatarIcon}</span>
            </div>
            <div class="message-content">
                <div class="message-text">${this.formatMessageContent(content)}</div>
            </div>
        `;
        
        messagesContainer.appendChild(messageEl);
    }
    
    appendStreamingChunk(messageId, chunk) {
        const messageEl = document.getElementById(messageId);
        if (!messageEl) return;
        
        const textEl = messageEl.querySelector('.message-text');
        if (textEl) {
            // Append text content
            const currentText = textEl.textContent || '';
            textEl.innerHTML = this.formatMessageContent(currentText + chunk);
            
            // Scroll to keep up with streaming
            this.scrollToBottom();
        }
    }
    
    updateStreamingMessage(messageId, content) {
        const messageEl = document.getElementById(messageId);
        if (!messageEl) return;
        
        const textEl = messageEl.querySelector('.message-text');
        if (textEl) {
            textEl.innerHTML = this.formatMessageContent(content);
        }
    }
    
    finalizeStreamingMessage(messageId, payload) {
        // Hide streaming indicator
        this.showStreamingIndicator(false);
        
        // Re-enable input
        this.setInputEnabled(true);
        this.isStreaming = false;
        
        // Update message content if provided
        if (payload.full_content) {
            this.updateStreamingMessage(messageId, payload.full_content);
        }
        
        // Refresh chat list (to update timestamp)
        this.refreshChatList();
        
        // Check if we should generate a title
        this.checkAndGenerateTitle();
        
        console.log('Message streaming completed');
    }
    
    async checkAndGenerateTitle() {
        const pairs = Math.floor(this.messages.length / 2);
        if (pairs >= 5 && !this.titleGenerated && this.currentChatId) {
            try {
                const invoke = this.getInvoke();
                if (!invoke) return;
                
                // generate_chat_title returns Result<String, String> - title string on success
                const title = await invoke('generate_chat_title', { 
                    chatId: this.currentChatId 
                });
                
                if (title && typeof title === 'string') {
                    this.updateChatTitle(title);
                    this.titleGenerated = true;
                    
                    // Refresh chat list to show new title
                    await this.refreshChatList();
                }
            } catch (error) {
                console.error('Error generating chat title:', error);
            }
        }
    }
    
    updateChatTitle(title) {
        // Find the current chat in the list and update its title
        const chatItem = document.querySelector(`#chat-item-${this.currentChatId} .chat-item-title`);
        if (chatItem) {
            chatItem.textContent = this.escapeHtml(title);
        }
        
        // Update in local chats array
        const chat = this.chats.find(c => c.id === this.currentChatId);
        if (chat) {
            chat.title = title;
        }
    }
    
    async refreshChatList() {
        try {
            const invoke = this.getInvoke();
            if (!invoke) {
                throw new Error('Tauri API not available');
            }
            
            // list_chats returns Vec<ChatSummary> directly, not {success} wrapper
            const chats = await invoke('list_chats', {
                modelPath: this.modelPath
            });
            
            if (Array.isArray(chats)) {
                this.chats = chats;
                this.renderChatList();
            }
        } catch (error) {
            console.error('Error refreshing chat list:', error);
        }
    }
    
    async deleteChat(chatId) {
        // Confirm deletion
        if (!confirm('Are you sure you want to delete this chat? This action cannot be undone.')) {
            return;
        }
        
        try {
            const invoke = this.getInvoke();
            if (!invoke) {
                throw new Error('Tauri API not available');
            }
            
            // delete_chat returns Result<(), String> - success if no error thrown
            await invoke('delete_chat', { chatId });
            
            // Remove from local array
            this.chats = this.chats.filter(c => c.id !== chatId);
            
            // If we deleted the current chat, create a new one
            if (chatId === this.currentChatId) {
                await this.createNewChat();
            } else {
                // Just refresh the list
                this.renderChatList();
            }
            
            console.log('Deleted chat:', chatId);
        } catch (error) {
            console.error('Error deleting chat:', error);
            this.showNotification('Failed to delete chat', 'error');
        }
    }
    
    async switchChat(chatId) {
        if (chatId === this.currentChatId) return;
        
        // Clean up any existing streaming
        this.cleanupStreaming();
        
        // Load the new chat
        await this.loadChat(chatId);
    }
    
    async saveParameters() {
        try {
            const invoke = this.getInvoke();
            if (!invoke) {
                throw new Error('Tauri API not available');
            }
            
            const result = await invoke('update_chat_parameters', {
                chatId: this.currentChatId,
                params: this.parameters
            });
            
            if (result.success) {
                this.showNotification('Settings saved successfully', 'success');
            } else {
                throw new Error(result.error || 'Failed to save settings');
            }
        } catch (error) {
            console.error('Error saving parameters:', error);
            this.showNotification('Failed to save settings', 'error');
        }
    }
    
    openSettingsPanel() {
        const panel = document.getElementById(`settings-panel-${this.processId}`);
        if (panel) {
            panel.classList.remove('hidden');
        }
    }
    
    closeSettingsPanel() {
        const panel = document.getElementById(`settings-panel-${this.processId}`);
        if (panel) {
            panel.classList.add('hidden');
        }
    }
    
    // Helper methods
    
    handleSendMessage() {
        const chatInput = document.getElementById(`chat-input-${this.processId}`);
        if (chatInput) {
            const content = chatInput.value.trim();
            if (content) {
                this.sendMessage(content);
            }
        }
    }
    
    setInputEnabled(enabled) {
        const chatInput = document.getElementById(`chat-input-${this.processId}`);
        const sendBtn = document.getElementById(`send-btn-${this.processId}`);
        
        if (chatInput) {
            chatInput.disabled = !enabled;
        }
        if (sendBtn) {
            sendBtn.disabled = !enabled;
        }
    }
    
    updateSendButtonState() {
        const chatInput = document.getElementById(`chat-input-${this.processId}`);
        const sendBtn = document.getElementById(`send-btn-${this.processId}`);
        
        if (chatInput && sendBtn) {
            const hasContent = chatInput.value.trim().length > 0;
            sendBtn.disabled = !hasContent || this.isStreaming;
        }
    }
    
    showStreamingIndicator(show) {
        const indicator = document.getElementById(`streaming-indicator-${this.processId}`);
        if (indicator) {
            indicator.classList.toggle('hidden', !show);
        }
    }
    
    updateActiveChatInList() {
        // Remove active class from all items
        const allItems = document.querySelectorAll(`#chat-list-${this.processId} .chat-item`);
        allItems.forEach(item => item.classList.remove('active'));
        
        // Add active class to current chat
        const currentItem = document.getElementById(`chat-item-${this.currentChatId}`);
        if (currentItem) {
            currentItem.classList.add('active');
        }
    }
    
    setupTextareaAutoResize() {
        const chatInput = document.getElementById(`chat-input-${this.processId}`);
        if (chatInput) {
            this.autoResizeTextarea(chatInput);
            this.updateComposerPadding();
        }
    }
    
    autoResizeTextarea(textarea) {
        if (!textarea) return;
        const { minHeight, maxHeight } = this.getTextareaHeightBounds(textarea);

        textarea.style.height = 'auto';
        const clampedHeight = Math.min(
            Math.max(textarea.scrollHeight, minHeight),
            maxHeight
        );
        textarea.style.height = clampedHeight + 'px';
        this.updateComposerPadding();
    }

    updateComposerPadding() {
        const inputArea = document.getElementById(`chat-input-area-${this.processId}`);
        const messagesContainer = document.getElementById(`chat-messages-${this.processId}`);
        if (!inputArea || !messagesContainer) return;

        const padding = inputArea.offsetHeight + 16;
        messagesContainer.style.setProperty('--composer-padding', `${padding}px`);
    }

    getTextareaHeightBounds(textarea) {
        const styles = window.getComputedStyle(textarea);
        const minHeight = parseFloat(styles.minHeight);
        const maxHeight = parseFloat(styles.maxHeight);

        return {
            minHeight: Number.isFinite(minHeight) ? minHeight : TEXTAREA_MIN_HEIGHT,
            maxHeight: Number.isFinite(maxHeight) && maxHeight > 0 ? maxHeight : TEXTAREA_MAX_HEIGHT
        };
    }
    
    scrollToBottom() {
        const messagesContainer = document.getElementById(`chat-messages-${this.processId}`);
        if (messagesContainer) {
            messagesContainer.scrollTop = messagesContainer.scrollHeight;
        }
    }
    
    cleanupStreaming() {
        // Clean up all streaming listeners
        for (const [messageId, unlisten] of this.streamingListeners.entries()) {
            try {
                unlisten();
            } catch (error) {
                console.error('Error cleaning up streaming listener:', error);
            }
        }
        this.streamingListeners.clear();
        
        this.isStreaming = false;
        this.showStreamingIndicator(false);
    }
    
    formatMessageContent(content) {
        if (!content) return '';
        
        // Escape HTML
        let formatted = this.escapeHtml(content);
        
        // Convert line breaks to <br>
        formatted = formatted.replace(/\n/g, '<br>');
        
        // Convert code blocks
        formatted = formatted.replace(/```(\w+)?\n([\s\S]*?)```/g, (match, lang, code) => {
            return `<pre><code class="language-${lang || 'text'}">${this.escapeHtml(code)}</code></pre>`;
        });
        
        // Convert inline code
        formatted = formatted.replace(/`([^`]+)`/g, '<code>$1</code>');
        
        return formatted;
    }
    
    escapeHtml(text) {
        const div = document.createElement('div');
        div.textContent = text;
        return div.innerHTML;
    }
    
    showNotification(message, type = 'info') {
        // Use the desktop notification system if available
        if (this.terminalManager && this.terminalManager.desktop) {
            this.terminalManager.desktop.showNotification(message, type);
        } else {
            console.log(`[${type.toUpperCase()}] ${message}`);
        }
    }
    
    // Draft message auto-save functionality
    getDraftStorageKey() {
        return `chat-draft-${this.currentChatId || 'new'}`;
    }
    
    saveDraft() {
        if (!this.currentChatId) return;
        
        const chatInput = document.getElementById(`chat-input-${this.processId}`);
        if (chatInput && chatInput.value.trim()) {
            try {
                localStorage.setItem(this.getDraftStorageKey(), chatInput.value);
                console.log('Draft saved for chat:', this.currentChatId);
            } catch (error) {
                console.error('Failed to save draft:', error);
            }
        }
    }
    
    restoreDraft() {
        if (!this.currentChatId) return;
        
        try {
            const draft = localStorage.getItem(this.getDraftStorageKey());
            if (draft) {
                const chatInput = document.getElementById(`chat-input-${this.processId}`);
                if (chatInput) {
                    chatInput.value = draft;
                    this.autoResizeTextarea(chatInput);
                    this.updateSendButtonState();
                    console.log('Draft restored for chat:', this.currentChatId);
                }
            }
        } catch (error) {
            console.error('Failed to restore draft:', error);
        }
    }
    
    clearDraft() {
        if (!this.currentChatId) return;
        
        try {
            localStorage.removeItem(this.getDraftStorageKey());
            console.log('Draft cleared for chat:', this.currentChatId);
        } catch (error) {
            console.error('Failed to clear draft:', error);
        }
    }
    
    // Cleanup method for when the chat tab is closed
    destroy() {
        // Save draft before destroying
        this.saveDraft();
        this.cleanupStreaming();
        
        // Remove event listeners
        const processId = this.processId;
        const newChatBtn = document.getElementById(`new-chat-btn-${processId}`);
        const sendBtn = document.getElementById(`send-btn-${processId}`);
        const chatInput = document.getElementById(`chat-input-${processId}`);
        const settingsBtn = document.getElementById(`settings-btn-${processId}`);
        
        // Clone elements to remove all event listeners
        if (newChatBtn) {
            const newBtn = newChatBtn.cloneNode(true);
            newChatBtn.parentNode.replaceChild(newBtn, newChatBtn);
        }
        if (sendBtn) {
            const newSend = sendBtn.cloneNode(true);
            sendBtn.parentNode.replaceChild(newSend, sendBtn);
        }
        if (chatInput) {
            const newInput = chatInput.cloneNode(true);
            chatInput.parentNode.replaceChild(newInput, chatInput);
        }
        if (settingsBtn) {
            const newSettings = settingsBtn.cloneNode(true);
            settingsBtn.parentNode.replaceChild(newSettings, settingsBtn);
        }
        
        console.log('ChatApp destroyed for process:', processId);
    }
}

// Export for use in other modules
if (typeof module !== 'undefined' && module.exports) {
    module.exports = { ChatApp };
}
