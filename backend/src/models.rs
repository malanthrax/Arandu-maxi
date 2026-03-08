use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::path::PathBuf;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalConfig {
    pub models_directory: String,
    #[serde(default)]
    pub additional_models_directories: Vec<String>,
    pub executable_folder: String,
    #[serde(default)]
    pub active_executable_folder: Option<String>,
    #[serde(default)]
    pub active_executable_version: Option<String>,
    pub theme_color: String,
    #[serde(default = "default_background_color")]
    pub background_color: String,
    #[serde(default = "default_theme_is_synced")]
    pub theme_is_synced: bool,
    #[serde(default)]
    pub openai_proxy_enabled: bool,
    #[serde(default)]
    pub openai_proxy_port: u16,
    #[serde(default = "default_network_server_host")]
    pub network_server_host: String,
    #[serde(default = "default_network_server_port")]
    pub network_server_port: u16,
    #[serde(default)]
    pub mcp_servers: Vec<McpServerConfig>,
    // === NETWORK DISCOVERY CONFIGURATION ===
    #[serde(default)]
    pub discovery_enabled: bool,
    #[serde(default = "default_discovery_port")]
    pub discovery_port: u16,
    #[serde(default = "default_discovery_broadcast_interval")]
    pub discovery_broadcast_interval: u64,
    #[serde(default = "default_discovery_instance_name")]
    pub discovery_instance_name: String,
    #[serde(default = "default_discovery_instance_id")]
    pub discovery_instance_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum McpTransport {
    Stdio,
    Sse,
    Http,
    Json,
    #[serde(rename = "streamable_http")]
    StreamableHttp,
}

impl Default for McpTransport {
    fn default() -> Self {
        McpTransport::Sse
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct McpServerConfig {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub transport: McpTransport,
    #[serde(default)]
    pub url: String,
    #[serde(default)]
    pub command: String,
    #[serde(default)]
    pub json_payload: String,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default)]
    pub env_vars: HashMap<String, String>,
    #[serde(default)]
    pub headers: HashMap<String, String>,
    #[serde(default)]
    pub timeout_seconds: u64,
    #[serde(default)]
    pub last_test_at: Option<String>,
    #[serde(default)]
    pub last_test_status: Option<String>,
    #[serde(default)]
    pub last_test_message: Option<String>,
    #[serde(default)]
    pub tools: Vec<McpToolInfo>,
    #[serde(default)]
    pub tools_last_refresh_at: Option<String>,
    #[serde(default)]
    pub tools_last_status: Option<String>,
    #[serde(default)]
    pub tools_last_message: Option<String>,
    #[serde(default)]
    pub tools_last_error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct McpToolInfo {
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default, rename = "inputSchema")]
    pub input_schema: Option<Value>,
    #[serde(default, rename = "outputSchema")]
    pub output_schema: Option<Value>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn global_config_deserializes_without_mcp_servers() {
        let legacy_payload = r#"{
            "models_directory": "C:/tmp/models",
            "additional_models_directories": [],
            "executable_folder": "C:/tmp/llama",
            "theme_color": "dark-gray",
            "theme_is_synced": true,
            "openai_proxy_enabled": false,
            "openai_proxy_port": 8081,
            "network_server_host": "127.0.0.1",
            "network_server_port": 8080
        }"#;

        let config: GlobalConfig =
            serde_json::from_str(legacy_payload).expect("legacy config should deserialize");

        assert_eq!(config.mcp_servers.len(), 0);
        // Network discovery fields should have defaults
        assert!(!config.discovery_enabled);
        assert_eq!(config.discovery_port, 5352);
        assert_eq!(config.discovery_broadcast_interval, 5);
        assert!(!config.discovery_instance_name.is_empty());
        assert!(!config.discovery_instance_id.is_empty());
    }

    #[test]
    fn global_config_deserializes_with_discovery_fields() {
        let payload = r#"{
            "models_directory": "C:/tmp/models",
            "additional_models_directories": [],
            "executable_folder": "C:/tmp/llama",
            "theme_color": "dark-gray",
            "theme_is_synced": true,
            "discovery_enabled": true,
            "discovery_port": 8080,
            "discovery_broadcast_interval": 10,
            "discovery_instance_name": "My-Arandu-Instance",
            "discovery_instance_id": "550e8400-e29b-41d4-a716-446655440000"
        }"#;

        let config: GlobalConfig =
            serde_json::from_str(payload).expect("config with discovery fields should deserialize");

        assert!(config.discovery_enabled);
        assert_eq!(config.discovery_port, 8080);
        assert_eq!(config.discovery_broadcast_interval, 10);
        assert_eq!(config.discovery_instance_name, "My-Arandu-Instance");
        assert_eq!(
            config.discovery_instance_id,
            "550e8400-e29b-41d4-a716-446655440000"
        );
    }

    #[test]
    fn streamable_http_transport_alias_deserializes() {
        let payload = r#"{
            "id": "mcp-1",
            "name": "Test",
            "transport": "streamable_http"
        }"#;

        let connection: McpServerConfig =
            serde_json::from_str(payload).expect("streamable_http transport should deserialize");

        assert_eq!(connection.transport, McpTransport::StreamableHttp);
    }

    #[test]
    fn json_transport_deserializes() {
        let payload = r#"{
            "id": "mcp-1",
            "name": "Test",
            "transport": "json"
        }"#;

        let connection: McpServerConfig =
            serde_json::from_str(payload).expect("json transport should deserialize");

        assert_eq!(connection.transport, McpTransport::Json);
    }

    #[test]
    fn mcp_tools_fields_deserialize_defaults() {
        let payload = r#"{
            "id": "mcp-1",
            "name": "Test"
        }"#;

        let connection: McpServerConfig =
            serde_json::from_str(payload).expect("tool fields should default when missing");

        assert_eq!(connection.tools.len(), 0);
        assert!(connection.tools_last_status.is_none());
        assert!(connection.tools_last_message.is_none());
        assert!(connection.tools_last_error.is_none());
        assert!(connection.tools_last_refresh_at.is_none());
    }

    #[test]
    fn mcp_tool_info_deserializes_optional_fields() {
        let payload = r#"{
            "name": "list_models",
            "description": "List models",
            "inputSchema": {"type":"object"},
            "outputSchema": {"type":"object"}
        }"#;

        let tool: McpToolInfo =
            serde_json::from_str(payload).expect("tool info should deserialize");

        assert_eq!(tool.name, "list_models");
        assert_eq!(tool.description.as_deref(), Some("List models"));
        assert!(tool.input_schema.is_some());
        assert!(tool.output_schema.is_some());
    }
}

impl McpServerConfig {
    pub fn new_blank(name: String) -> Self {
        Self {
            id: String::new(),
            name,
            enabled: true,
            transport: McpTransport::Sse,
            url: String::new(),
            command: String::new(),
            json_payload: String::new(),
            args: Vec::new(),
            env_vars: HashMap::new(),
            headers: HashMap::new(),
            timeout_seconds: 10,
            last_test_at: None,
            last_test_status: None,
            last_test_message: None,
            tools: Vec::new(),
            tools_last_refresh_at: None,
            tools_last_status: None,
            tools_last_message: None,
            tools_last_error: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpTestResult {
    pub success: bool,
    pub latency_ms: i64,
    pub message: String,
    #[serde(default)]
    pub status_code: Option<u16>,
    #[serde(default)]
    pub exit_code: Option<i32>,
    #[serde(default)]
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpToolsResult {
    pub success: bool,
    pub latency_ms: i64,
    pub message: String,
    #[serde(default)]
    pub tool_count: usize,
    #[serde(default)]
    pub tools: Vec<McpToolInfo>,
    #[serde(default)]
    pub status_code: Option<u16>,
    #[serde(default)]
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpToolCallRequest {
    pub connection_id: String,
    pub tool_name: String,
    #[serde(default = "default_mcp_tool_call_arguments")]
    pub arguments: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpToolCallResult {
    pub success: bool,
    pub latency_ms: i64,
    pub message: String,
    #[serde(default)]
    pub content: String,
    #[serde(default)]
    pub is_error: bool,
    #[serde(default)]
    pub raw_result: Option<Value>,
    #[serde(default)]
    pub error: Option<String>,
    #[serde(default)]
    pub status_code: Option<u16>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupermemoryNativeCallRequest {
    pub api_key: String,
    pub tool_name: String,
    #[serde(default = "default_mcp_tool_call_arguments")]
    pub arguments: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupermemoryNativeCallResult {
    pub success: bool,
    pub latency_ms: i64,
    pub message: String,
    #[serde(default)]
    pub content: String,
    #[serde(default)]
    pub is_error: bool,
    #[serde(default)]
    pub raw_result: Option<Value>,
    #[serde(default)]
    pub error: Option<String>,
    #[serde(default)]
    pub status_code: Option<u16>,
}

fn default_mcp_tool_call_arguments() -> Value {
    Value::Object(serde_json::Map::new())
}

fn default_background_color() -> String {
    "dark-gray".to_string()
}

fn default_theme_is_synced() -> bool {
    true
}

fn default_network_server_host() -> String {
    "127.0.0.1".to_string()
}

fn default_network_server_port() -> u16 {
    8080
}

// === NETWORK DISCOVERY DEFAULT FUNCTIONS ===
fn default_discovery_port() -> u16 {
    5352
}

fn default_discovery_broadcast_interval() -> u64 {
    5
}

fn default_discovery_instance_name() -> String {
    hostname::get()
        .ok()
        .and_then(|h| h.into_string().ok())
        .unwrap_or_else(|| "Arandu-PC".to_string())
}

fn default_discovery_instance_id() -> String {
    Uuid::new_v4().to_string()
}

pub(crate) fn preferred_arandu_base_dir() -> PathBuf {
    let preferred = PathBuf::from(r"H:\Ardanu Fix\Arandu-maxi\.Arandu");
    if preferred.parent().is_some_and(|parent| parent.exists()) {
        return preferred;
    }

    dirs::home_dir().unwrap_or_default().join(".Arandu")
}

impl Default for GlobalConfig {
    fn default() -> Self {
        let base_dir = preferred_arandu_base_dir();
        Self {
            models_directory: base_dir
                .join("models")
                .to_str()
                .unwrap_or_default()
                .to_string(),
            additional_models_directories: Vec::new(),
            executable_folder: base_dir
                .join("llama.cpp")
                .to_str()
                .unwrap_or_default()
                .to_string(),
            active_executable_folder: None,
            active_executable_version: None,
            theme_color: "dark-gray".to_string(),
            background_color: "dark-gray".to_string(),
            theme_is_synced: true,
            openai_proxy_enabled: false,
            openai_proxy_port: 8081,
            network_server_host: "127.0.0.1".to_string(),
            network_server_port: 8080,
            mcp_servers: Vec::new(),
            // === NETWORK DISCOVERY DEFAULTS ===
            discovery_enabled: false,
            discovery_port: default_discovery_port(),
            discovery_broadcast_interval: default_discovery_broadcast_interval(),
            discovery_instance_name: default_discovery_instance_name(),
            discovery_instance_id: default_discovery_instance_id(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelPreset {
    pub id: String,
    pub name: String,
    pub custom_args: String,
    pub is_default: bool,
    #[serde(default)]
    pub env_vars: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfig {
    pub custom_args: String,
    pub server_host: String,
    pub server_port: u16,
    pub model_path: String,
    #[serde(default)]
    pub presets: Vec<ModelPreset>,
    #[serde(default)]
    pub default_preset_id: Option<String>,
    #[serde(default)]
    pub env_vars: HashMap<String, String>,

    // HF Update tracking fields
    #[serde(default)]
    pub hf_model_id: Option<String>, // "author/model" format (legacy field)
    #[serde(default)]
    pub hf_link_source: Option<String>, // "download", "guess", "manual" (legacy field)
    #[serde(default)]
    pub local_file_modified: Option<i64>, // Unix timestamp (legacy field)
    #[serde(default)]
    pub file_size_bytes: Option<i64>, // For additional comparison (legacy field)
    #[serde(default)]
    pub last_hf_check: Option<i64>, // When we last queried HF (legacy field)
    #[serde(default)]
    pub hf_file_modified: Option<i64>, // HF file timestamp (legacy field)
    #[serde(default)]
    pub hf_file_size: Option<i64>, // HF file size (legacy field)
    #[serde(default)]
    pub update_available: bool, // Computed flag (legacy field)
    #[serde(default)]
    pub hf_metadata: Option<HfMetadata>, // New HF metadata from update_checker
}

impl ModelConfig {
    pub fn new(model_path: String) -> Self {
        Self {
            custom_args: String::new(),
            server_host: "127.0.0.1".to_string(),
            server_port: 8080,
            model_path,
            presets: Vec::new(),
            default_preset_id: None,
            env_vars: HashMap::new(),
            hf_model_id: None,
            hf_link_source: None,
            local_file_modified: None,
            file_size_bytes: None,
            last_hf_check: None,
            hf_file_modified: None,
            hf_file_size: None,
            update_available: false,
            hf_metadata: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub path: String,
    pub name: String,
    pub size_gb: f64,
    pub architecture: String,
    pub model_name: String,
    pub quantization: String,
    pub date: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GgufMetadata {
    pub architecture: String,
    pub name: String,
    pub quantization: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HfMetadata {
    pub model_id: String,            // "author/model-name"
    pub filename: String,            // "model-Q4_K_M.gguf"
    pub commit_date: Option<String>, // ISO 8601 from HF API
    pub linked_at: String,           // ISO 8601 when link was created
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateCheckResult {
    pub status: UpdateStatus,
    pub local_date: Option<String>,
    pub remote_date: Option<String>,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum UpdateStatus {
    UpToDate,
    UpdateAvailable,
    Unknown,
    NotLinked,
    Error(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessInfo {
    pub id: String,
    pub model_path: String,
    pub model_name: String,
    pub host: String,
    pub port: u16,
    pub command: Vec<String>,
    pub status: ProcessStatus,
    pub output: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub last_sent_line: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProcessStatus {
    Starting,
    Running,
    Stopped,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LaunchResult {
    pub success: bool,
    pub process_id: String,
    pub server_host: String,
    pub server_port: u16,
    pub model_name: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessOutput {
    pub output: Vec<String>,
    pub is_running: bool,
    pub return_code: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ModelStatus {
    Starting,
    Ready,
    Failed(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveModel {
    pub process_id: String,
    pub model_path: String,
    pub model_name: String,
    pub host: String,
    pub port: u16,
    pub server_host: String,
    pub server_port: u16,
    pub status: ModelStatus,
    pub launched_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteLaunchRequest {
    pub model_path: String,
    pub server_host: Option<String>,
    pub server_port: Option<u16>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteLaunchResponse {
    pub success: bool,
    pub message: String,
    pub process_id: Option<String>,
    pub server_host: Option<String>,
    pub server_port: Option<u16>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteStopRequest {
    pub process_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteStopResponse {
    pub success: bool,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteActiveModelsResponse {
    pub success: bool,
    pub models: Vec<ActiveModel>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionState {
    pub windows: HashMap<String, WindowState>,
    pub terminals: HashMap<String, TerminalState>,
    pub desktop_state: DesktopState,
}

impl Default for SessionState {
    fn default() -> Self {
        Self {
            windows: HashMap::new(),
            terminals: HashMap::new(),
            desktop_state: DesktopState::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowState {
    pub window_type: String,
    pub title: String,
    pub content: String,
    pub position: Position,
    pub size: Size,
    pub visible: bool,
    pub z_index: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Size {
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalState {
    pub process_id: String,
    pub model_name: String,
    pub model_path: String,
    pub host: String,
    pub port: u16,
    pub status: String,
    pub output: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DesktopState {
    pub icon_positions: HashMap<String, Position>,
    pub sort_type: Option<String>,
    pub sort_direction: String,
    pub theme: String,
    #[serde(default = "default_background_color")]
    pub background: String,
    #[serde(default = "default_theme_is_synced")]
    pub theme_synced: bool,
}

impl Default for DesktopState {
    fn default() -> Self {
        Self {
            icon_positions: HashMap::new(),
            sort_type: None,
            sort_direction: "asc".to_string(),
            theme: "dark-gray".to_string(),
            background: "dark-gray".to_string(),
            theme_synced: true,
        }
    }
}

// Hugging Face related structures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub success: bool,
    pub models: Vec<ModelBasic>,
    pub total: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelBasic {
    pub id: String,
    pub name: String,
    pub author: String,
    pub downloads: u64,
    pub likes: u64,
    #[serde(rename = "lastModified")]
    pub last_modified: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelDetails {
    pub id: String,
    pub name: String,
    pub author: String,
    pub description: Option<String>,
    pub downloads: u64,
    pub likes: u64,
    pub total_files: u32,
    pub gguf_files: HashMap<String, GgufFileInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadStartResult {
    pub download_id: String,
    pub message: String,
}

// Update checker structures - UpdateCheckResult is defined above (line 132)

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InitialScanResult {
    pub success: bool,
    pub models_processed: usize,
    pub models_linked: usize,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HFLinkResult {
    pub success: bool,
    pub message: String,
    pub files: Vec<HFFileInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HFFileInfo {
    pub filename: String,
    pub size: i64,
    pub last_modified: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GgufFileInfo {
    pub filename: String,
    pub path: String,
    pub size: u64,
    pub quantization_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackerModel {
    pub id: String,
    pub name: String,
    pub author: String,
    pub description: String,
    pub source: String,
    pub category: String,
    pub is_chinese: bool,
    pub is_gguf: bool,
    pub quantizations: Vec<String>,
    pub backends: Vec<String>,
    pub estimated_size_gb: f64,
    pub vram_requirement_gb: Option<f64>,
    pub context_length: Option<u32>,
    pub downloads: u64,
    pub likes: u64,
    pub last_updated: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackerConfig {
    pub vram_limit_gb: f64,
    pub scrape_interval_hours: u32,
    pub last_scrape: Option<String>,
    pub enabled_sources: Vec<String>,
    pub include_chinese: bool,
}

impl Default for TrackerConfig {
    fn default() -> Self {
        Self {
            vram_limit_gb: 24.0,
            scrape_interval_hours: 6,
            last_scrape: None,
            enabled_sources: vec!["huggingface".to_string()],
            include_chinese: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackerStats {
    pub total_models: u32,
    pub chinese_models: u32,
    pub gguf_models: u32,
    pub categories: HashMap<String, u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeeklyReport {
    pub id: String,
    pub generated_at: String,
    pub period_start: String,
    pub period_end: String,
    pub total_models: u32,
    pub new_models: u32,
    pub chinese_models: u32,
    pub gguf_models: u32,
    pub categories: HashMap<String, u32>,
    pub top_downloads: Vec<TrackerModel>,
}

// Note: DiscoveredPeer, RemoteModel, and DiscoveryStatus are defined in discovery.rs
// Re-export them here for use in other modules
pub use crate::discovery::{DiscoveredPeer, DiscoveryStatus};
