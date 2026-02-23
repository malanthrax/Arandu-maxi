//! Parameter Controller for Arandu's modern chat interface
//! 
//! Manages runtime parameter adjustments for llama.cpp models.
//! Distinguishes between:
//! - Per-request parameters (can change per message): temperature, top_p, top_k, max_tokens, repeat_penalty
//! - Model-load parameters (require restart): context_length, n_gpu_layers

use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use crate::chat_models::ChatParameters;
use crate::openai_types::ChatMessage;

/// Parameters that can be changed per request to llama.cpp
/// These are sent with each /v1/chat/completions request
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GenerationParams {
    pub temperature: f32,
    pub top_p: f32,
    pub top_k: i32,
    pub max_tokens: i32,
    pub repeat_penalty: f32,
}

impl Default for GenerationParams {
    fn default() -> Self {
        Self {
            temperature: 0.7,
            top_p: 0.9,
            top_k: 40,
            max_tokens: 2048,
            repeat_penalty: 1.1,
        }
    }
}

impl GenerationParams {
    /// Validate parameters are within acceptable ranges
    pub fn validate(&self) -> Result<(), String> {
        if self.temperature < 0.0 || self.temperature > 2.0 {
            return Err(format!("Temperature must be between 0.0 and 2.0, got {}", self.temperature));
        }
        if self.top_p < 0.0 || self.top_p > 1.0 {
            return Err(format!("Top-p must be between 0.0 and 1.0, got {}", self.top_p));
        }
        if self.top_k < 1 || self.top_k > 100 {
            return Err(format!("Top-k must be between 1 and 100, got {}", self.top_k));
        }
        if self.max_tokens < 1 || self.max_tokens > 32768 {
            return Err(format!("Max tokens must be between 1 and 32768, got {}", self.max_tokens));
        }
        if self.repeat_penalty < 0.0 || self.repeat_penalty > 2.0 {
            return Err(format!("Repeat penalty must be between 0.0 and 2.0, got {}", self.repeat_penalty));
        }
        Ok(())
    }
}

/// Parameters that require model restart to change
/// These are set via command-line arguments when loading the model
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ModelLoadParams {
    pub context_length: i32,
    pub n_gpu_layers: i32,
}

impl Default for ModelLoadParams {
    fn default() -> Self {
        Self {
            context_length: 4096,
            n_gpu_layers: -1, // -1 means all layers on GPU
        }
    }
}

/// Result of attempting to change a parameter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterChangeResult {
    pub success: bool,
    pub requires_restart: bool,
    pub message: String,
    pub saved_for_restart: bool,
}

/// Current runtime status of parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterStatus {
    pub generation: GenerationParams,
    pub model_load: ModelLoadParams,
    pub server_url: String,
    pub model_loaded: bool,
}

/// Message context builder that respects context length
pub struct MessageContext {
    pub messages: Vec<ChatMessage>,
    pub estimated_tokens: usize,
    pub truncated: bool,
}

/// Main controller for managing llama.cpp parameters
pub struct ParameterController {
    llama_server_url: String,
    http_client: reqwest::Client,
    current_generation_params: GenerationParams,
    current_model_load_params: ModelLoadParams,
    config_storage: ConfigStorage,
}

/// Storage for parameters that need to persist across sessions
#[derive(Debug, Clone)]
struct ConfigStorage {
    pending_context_length: Option<i32>,
    pending_n_gpu_layers: Option<i32>,
}

impl Default for ConfigStorage {
    fn default() -> Self {
        Self {
            pending_context_length: None,
            pending_n_gpu_layers: None,
        }
    }
}

impl ParameterController {
    /// Create a new ParameterController with the given server URL
    pub fn new(llama_server_url: String) -> Self {
        let http_client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .connect_timeout(std::time::Duration::from_secs(10))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            llama_server_url,
            http_client,
            current_generation_params: GenerationParams::default(),
            current_model_load_params: ModelLoadParams::default(),
            config_storage: ConfigStorage::default(),
        }
    }

    /// Create a new ParameterController with custom initial parameters
    pub fn with_params(
        llama_server_url: String,
        generation_params: GenerationParams,
        model_load_params: ModelLoadParams,
    ) -> Self {
        let mut controller = Self::new(llama_server_url);
        controller.current_generation_params = generation_params;
        controller.current_model_load_params = model_load_params;
        controller
    }

    // ============================================================================
    // GENERATION PARAMETERS (Per-Request)
    // ============================================================================

    /// Set generation parameters that will be used for subsequent requests
    /// These parameters can be changed at any time without restarting the model
    /// 
    /// # Arguments
    /// * `params` - The new generation parameters to use
    /// 
    /// # Returns
    /// * `Ok(())` if parameters were set successfully
    /// * `Err(String)` if validation fails
    pub fn set_generation_params(&mut self, params: GenerationParams) -> Result<(), String> {
        // Validate parameters
        params.validate()?;
        
        // Update current parameters
        self.current_generation_params = params.clone();
        
        tracing::info!(
            "Generation parameters updated: temperature={}, top_p={}, top_k={}, max_tokens={}, repeat_penalty={}",
            params.temperature,
            params.top_p,
            params.top_k,
            params.max_tokens,
            params.repeat_penalty
        );
        
        Ok(())
    }

    /// Get the current generation parameters
    pub fn get_generation_params(&self) -> &GenerationParams {
        &self.current_generation_params
    }

    /// Apply current generation parameters to a chat completion request body
    pub fn apply_generation_params(&self, body: &mut serde_json::Value) {
        if let Some(obj) = body.as_object_mut() {
            obj.insert("temperature".to_string(), json!(self.current_generation_params.temperature));
            obj.insert("top_p".to_string(), json!(self.current_generation_params.top_p));
            obj.insert("top_k".to_string(), json!(self.current_generation_params.top_k));
            obj.insert("max_tokens".to_string(), json!(self.current_generation_params.max_tokens));
            obj.insert("repeat_penalty".to_string(), json!(self.current_generation_params.repeat_penalty));
        }
    }

    // ============================================================================
    // MODEL LOAD PARAMETERS (Require Restart)
    // ============================================================================

    /// Attempt to set the context length
    /// 
    /// # Arguments
    /// * `length` - The desired context length in tokens
    /// 
    /// # Returns
    /// * `Ok(true)` if the change was applied immediately (model not loaded)
    /// * `Ok(false)` if the change requires a restart (model is loaded)
    /// * `Err(String)` if the length is invalid
    pub async fn set_context_length(&mut self, length: i32) -> Result<bool, String> {
        // Validate context length
        if length < 512 {
            return Err(format!("Context length must be at least 512, got {}", length));
        }
        if length > 131072 {
            return Err(format!("Context length cannot exceed 131072, got {}", length));
        }

        // Check if model is currently loaded
        let model_loaded = self.is_model_loaded().await?;
        
        if model_loaded {
            // Cannot change context length at runtime - save for next load
            self.config_storage.pending_context_length = Some(length);
            
            tracing::info!(
                "Context length change to {} saved for next model load (requires restart)",
                length
            );
            
            Ok(false)
        } else {
            // No model loaded - can apply immediately
            self.current_model_load_params.context_length = length;
            self.config_storage.pending_context_length = None;
            
            tracing::info!("Context length set to {} (no model currently loaded)", length);
            
            Ok(true)
        }
    }

    /// Attempt to set the number of GPU layers
    /// 
    /// # Arguments
    /// * `n_layers` - Number of layers to offload to GPU (-1 for all)
    /// 
    /// # Returns
    /// * `Ok(true)` if the change was applied immediately (model not loaded)
    /// * `Ok(false)` if the change requires a restart (model is loaded)
    /// * `Err(String)` if the value is invalid
    pub async fn set_n_gpu_layers(&mut self, n_layers: i32) -> Result<bool, String> {
        // Validate n_gpu_layers
        if n_layers < -1 {
            return Err(format!("n_gpu_layers must be -1 or greater, got {}", n_layers));
        }

        // Check if model is currently loaded
        let model_loaded = self.is_model_loaded().await?;
        
        if model_loaded {
            // Cannot change n_gpu_layers at runtime - save for next load
            self.config_storage.pending_n_gpu_layers = Some(n_layers);
            
            tracing::info!(
                "n_gpu_layers change to {} saved for next model load (requires restart)",
                n_layers
            );
            
            Ok(false)
        } else {
            // No model loaded - can apply immediately
            self.current_model_load_params.n_gpu_layers = n_layers;
            self.config_storage.pending_n_gpu_layers = None;
            
            tracing::info!("n_gpu_layers set to {} (no model currently loaded)", n_layers);
            
            Ok(true)
        }
    }

    /// Get the current model load parameters
    pub fn get_model_load_params(&self) -> &ModelLoadParams {
        &self.current_model_load_params
    }

    /// Get any pending changes that require a model restart
    pub fn get_pending_changes(&self) -> HashMap<String, i32> {
        let mut pending = HashMap::new();
        
        if let Some(length) = self.config_storage.pending_context_length {
            pending.insert("context_length".to_string(), length);
        }
        
        if let Some(layers) = self.config_storage.pending_n_gpu_layers {
            pending.insert("n_gpu_layers".to_string(), layers);
        }
        
        pending
    }

    /// Clear pending changes (call after model restart)
    pub fn clear_pending_changes(&mut self) {
        self.config_storage.pending_context_length = None;
        self.config_storage.pending_n_gpu_layers = None;
    }

    /// Apply pending changes to current parameters (call after model restart)
    pub fn apply_pending_changes(&mut self) {
        if let Some(length) = self.config_storage.pending_context_length {
            self.current_model_load_params.context_length = length;
        }
        
        if let Some(layers) = self.config_storage.pending_n_gpu_layers {
            self.current_model_load_params.n_gpu_layers = layers;
        }
        
        self.clear_pending_changes();
    }

    // ============================================================================
    // CURRENT PARAMETERS RETRIEVAL
    // ============================================================================

    /// Retrieve current parameters from the running llama.cpp server
    /// 
    /// # Returns
    /// * `Ok(ChatParameters)` containing the current parameters
    /// * `Err(String)` if the server is unreachable
    pub async fn get_current_params(&self) -> Result<ChatParameters, String> {
        // Try to get health/props from llama.cpp server
        let url = format!("{}/health", self.llama_server_url);
        
        match self.http_client.get(&url).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    // llama.cpp server is running
                    // Note: llama.cpp may not expose all parameters via HTTP API
                    // We'll return what we have stored, updated with any info from server
                    let params = ChatParameters {
                        temperature: self.current_generation_params.temperature,
                        top_p: self.current_generation_params.top_p,
                        max_tokens: self.current_generation_params.max_tokens as u32,
                        system_prompt: None,
                        context_length: Some(self.current_model_load_params.context_length as u32),
                    };
                    
                    Ok(params)
                } else {
                    Err(format!("llama.cpp server returned status: {}", response.status()))
                }
            }
            Err(e) => {
                Err(format!("Failed to connect to llama.cpp server: {}", e))
            }
        }
    }

    /// Get the full parameter status including runtime and pending changes
    pub fn get_parameter_status(&self) -> ParameterStatus {
        ParameterStatus {
            generation: self.current_generation_params.clone(),
            model_load: self.current_model_load_params.clone(),
            server_url: self.llama_server_url.clone(),
            model_loaded: false, // Will be updated by async check if needed
        }
    }

    // ============================================================================
    // CONTEXT HANDLING
    // ============================================================================

    /// Build message context respecting context_length
    /// Handles token counting and message truncation
    /// 
    /// # Arguments
    /// * `messages` - All messages in the conversation
    /// * `system_prompt` - Optional system prompt to include
    /// 
    /// # Returns
    /// * `MessageContext` containing the prepared messages and metadata
    pub fn build_message_context(
        &self,
        messages: &[ChatMessage],
        system_prompt: Option<&str>,
    ) -> MessageContext {
        let context_length = self.current_model_load_params.context_length as usize;
        let max_tokens = self.current_generation_params.max_tokens as usize;
        
        // Reserve tokens for generation
        let available_context = context_length.saturating_sub(max_tokens);
        
        // Estimate tokens (rough approximation: 4 chars ≈ 1 token)
        let estimate_tokens = |text: &str| -> usize {
            text.len() / 4 + 1
        };
        
        let mut prepared_messages = Vec::new();
        let mut estimated_tokens = 0;
        let mut truncated = false;
        
        // Add system prompt if provided
        if let Some(system) = system_prompt {
            let system_tokens = estimate_tokens(system);
            if system_tokens < available_context {
                prepared_messages.push(ChatMessage {
                    role: "system".to_string(),
                    content: system.to_string(),
                });
                estimated_tokens += system_tokens;
            }
        }
        
        // Process messages from newest to oldest, keeping as many as fit
        let mut included_messages = Vec::new();
        
        for message in messages.iter().rev() {
            let msg_tokens = estimate_tokens(&message.content);
            
            if estimated_tokens + msg_tokens > available_context {
                truncated = true;
                break;
            }
            
            included_messages.push(message.clone());
            estimated_tokens += msg_tokens;
        }
        
        // Reverse to maintain chronological order
        included_messages.reverse();
        prepared_messages.extend(included_messages);
        
        MessageContext {
            messages: prepared_messages,
            estimated_tokens,
            truncated,
        }
    }

    /// Get command-line arguments for model loading based on current parameters
    /// This should be used when launching llama-server
    pub fn get_model_load_args(&self) -> Vec<String> {
        let mut args = Vec::new();
        
        // Context length
        args.push("-c".to_string());
        args.push(self.current_model_load_params.context_length.to_string());
        
        // GPU layers
        if self.current_model_load_params.n_gpu_layers >= 0 {
            args.push("-ngl".to_string());
            args.push(self.current_model_load_params.n_gpu_layers.to_string());
        }
        
        args
    }

    // ============================================================================
    // HELPER METHODS
    // ============================================================================

    /// Check if a model is currently loaded by querying the server
    async fn is_model_loaded(&self) -> Result<bool, String> {
        let url = format!("{}/health", self.llama_server_url);
        
        match self.http_client.get(&url).send().await {
            Ok(response) => Ok(response.status().is_success()),
            Err(_) => Ok(false), // Server not reachable = no model loaded
        }
    }

    /// Update the server URL (e.g., when model changes)
    pub fn set_server_url(&mut self, url: String) {
        self.llama_server_url = url;
    }

    /// Get the current server URL
    pub fn get_server_url(&self) -> &str {
        &self.llama_server_url
    }
}

/// Convert ChatParameters from chat_models.rs to GenerationParams
impl From<ChatParameters> for GenerationParams {
    fn from(params: ChatParameters) -> Self {
        Self {
            temperature: params.temperature,
            top_p: params.top_p,
            top_k: 40, // Default value not in ChatParameters
            max_tokens: params.max_tokens as i32,
            repeat_penalty: 1.1, // Default value not in ChatParameters
        }
    }
}

/// Convert GenerationParams to ChatParameters
impl From<GenerationParams> for ChatParameters {
    fn from(params: GenerationParams) -> Self {
        Self {
            temperature: params.temperature,
            top_p: params.top_p,
            max_tokens: params.max_tokens as u32,
            system_prompt: None,
            context_length: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generation_params_default() {
        let params = GenerationParams::default();
        assert_eq!(params.temperature, 0.7);
        assert_eq!(params.top_p, 0.9);
        assert_eq!(params.top_k, 40);
        assert_eq!(params.max_tokens, 2048);
        assert_eq!(params.repeat_penalty, 1.1);
    }

    #[test]
    fn test_generation_params_validation() {
        // Valid params
        let valid = GenerationParams::default();
        assert!(valid.validate().is_ok());

        // Invalid temperature
        let mut invalid = GenerationParams::default();
        invalid.temperature = 3.0;
        assert!(invalid.validate().is_err());

        // Invalid top_p
        let mut invalid = GenerationParams::default();
        invalid.top_p = 1.5;
        assert!(invalid.validate().is_err());

        // Invalid top_k
        let mut invalid = GenerationParams::default();
        invalid.top_k = 0;
        assert!(invalid.validate().is_err());
    }

    #[test]
    fn test_model_load_params_default() {
        let params = ModelLoadParams::default();
        assert_eq!(params.context_length, 4096);
        assert_eq!(params.n_gpu_layers, -1);
    }

    #[test]
    fn test_message_context_building() {
        let controller = ParameterController::new("http://localhost:8080".to_string());
        
        let messages = vec![
            ChatMessage {
                role: "user".to_string(),
                content: "Hello".to_string(),
            },
            ChatMessage {
                role: "assistant".to_string(),
                content: "Hi there!".to_string(),
            },
        ];
        
        let context = controller.build_message_context(&messages, Some("You are a helpful assistant."));
        
        assert!(!context.messages.is_empty());
        assert!(context.estimated_tokens > 0);
    }

    #[test]
    fn test_get_model_load_args() {
        let controller = ParameterController::new("http://localhost:8080".to_string());
        let args = controller.get_model_load_args();
        
        assert!(args.contains(&"-c".to_string()));
        assert!(args.contains(&"4096".to_string()));
    }

    #[test]
    fn test_pending_changes() {
        let mut controller = ParameterController::new("http://localhost:8080".to_string());
        
        // Initially no pending changes
        let pending = controller.get_pending_changes();
        assert!(pending.is_empty());
        
        // Simulate pending changes
        controller.config_storage.pending_context_length = Some(8192);
        controller.config_storage.pending_n_gpu_layers = Some(35);
        
        let pending = controller.get_pending_changes();
        assert_eq!(pending.get("context_length"), Some(&8192));
        assert_eq!(pending.get("n_gpu_layers"), Some(&35));
        
        // Clear pending changes
        controller.clear_pending_changes();
        let pending = controller.get_pending_changes();
        assert!(pending.is_empty());
    }
}
