// Arandu Backend - Main Library
// AI AGENTS: Check nowledge-mem memory for file locations and patterns before modifying
// Search: "Arandu Complete File Location Reference" | "Arandu Common Development Patterns"
use std::collections::HashMap;
use std::path::PathBuf;
use std::fs;
use chrono::Utc;
use tauri::{Manager, Listener, tray::TrayIconBuilder, menu::{Menu, MenuItemBuilder}};

const APP_VERSION: &str = env!("CARGO_PKG_VERSION");
use tokio::sync::Mutex;
use std::sync::Arc;
use tokio::process::Command as TokioCommand;
use tokio::time::{timeout, Duration, Instant};

mod models;
mod config;
mod process;
mod scanner;
mod huggingface;
mod downloader;
mod llamacpp_manager;
mod system_monitor;
mod gguf_parser;
mod update_checker;
mod huggingface_downloader;
mod tracker_scraper;
mod tracker_manager;
mod openai_types;
mod openai_proxy;
mod llama_client;

use config::*;
use process::*;
use process::launch_model_external as launch_model_external_impl;
use scanner::*;
use huggingface::*;
use huggingface_downloader::*;
use models::{GlobalConfig, ModelConfig, ModelPreset, ProcessInfo, SessionState, WindowState, ProcessOutput, SearchResult, ModelDetails, DownloadStartResult, UpdateCheckResult, UpdateStatus, InitialScanResult, HFLinkResult, HFFileInfo, HfMetadata, GgufMetadata, TrackerModel, TrackerConfig, TrackerStats, WeeklyReport, McpServerConfig, McpToolsResult, McpToolInfo, McpTestResult, McpTransport};
use downloader::{DownloadManager, DownloadStatus};
use llamacpp_manager::{LlamaCppReleaseFrontend as LlamaCppRelease, LlamaCppAssetFrontend as LlamaCppAsset};
use system_monitor::*;
use tracker_scraper::TrackerScraper;
use tracker_manager::TrackerManager;

// Import ProcessHandle from process module
use process::ProcessHandle;

fn arandu_base_dir() -> Result<PathBuf, String> {
    let home = dirs::home_dir().ok_or_else(|| "Unable to resolve home directory".to_string())?;
    Ok(home.join(".Arandu"))
}

fn chats_dir() -> Result<PathBuf, String> {
    let dir = arandu_base_dir()?.join("chats");
    if !dir.exists() {
        fs::create_dir_all(&dir).map_err(|e| format!("Failed to create chats directory: {}", e))?;
    }
    Ok(dir)
}

fn chats_index_path() -> Result<PathBuf, String> {
    Ok(chats_dir()?.join("index.json"))
}

fn read_chats_index() -> Result<Vec<serde_json::Value>, String> {
    let index_path = chats_index_path()?;
    if !index_path.exists() {
        return Ok(Vec::new());
    }

    let content = fs::read_to_string(&index_path)
        .map_err(|e| format!("Failed to read chats index: {}", e))?;

    serde_json::from_str::<Vec<serde_json::Value>>(&content)
        .map_err(|e| format!("Failed to parse chats index: {}", e))
}

fn write_chats_index(index: &[serde_json::Value]) -> Result<(), String> {
    let index_path = chats_index_path()?;
    let content = serde_json::to_string_pretty(index)
        .map_err(|e| format!("Failed to serialize chats index: {}", e))?;
    fs::write(&index_path, content)
        .map_err(|e| format!("Failed to write chats index: {}", e))
}

fn chat_markdown_path(chat_id: &str) -> Result<PathBuf, String> {
    Ok(chats_dir()?.join(format!("{}.md", chat_id)))
}

fn sanitize_chat_title(raw: &str) -> String {
    let cleaned = raw
        .replace(['\n', '\r'], " ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    if cleaned.is_empty() {
        "Untitled Chat".to_string()
    } else if cleaned.len() > 80 {
        cleaned.chars().take(80).collect::<String>()
    } else {
        cleaned
    }
}

#[tauri::command]
async fn list_chat_logs() -> Result<Vec<serde_json::Value>, String> {
    let mut index = read_chats_index()?;
    index.sort_by(|a, b| {
        let a_ts = a.get("last_used_at").and_then(|v| v.as_str()).unwrap_or("");
        let b_ts = b.get("last_used_at").and_then(|v| v.as_str()).unwrap_or("");
        b_ts.cmp(a_ts)
    });
    Ok(index)
}

#[tauri::command]
async fn create_chat_log(model: String) -> Result<serde_json::Value, String> {
    let now = Utc::now().to_rfc3339();
    let chat_id = format!("chat-{}", Utc::now().timestamp_millis());
    let title = format!("Chat {}", Utc::now().format("%Y-%m-%d %H:%M"));
    let model_label = model.trim();

    let mut index = read_chats_index()?;
    let entry = serde_json::json!({
        "chat_id": chat_id,
        "title": title,
        "created_at": now,
        "last_used_at": now,
        "last_model": model_label,
        "models_used": if model_label.is_empty() { Vec::<String>::new() } else { vec![model_label.to_string()] },
        "message_count": 0
    });

    let chat_path = chat_markdown_path(entry.get("chat_id").and_then(|v| v.as_str()).unwrap_or(""))?;
    let md = format!(
        "---\nchat_id: {}\ntitle: {}\ncreated_at: {}\nlast_used_at: {}\nmodels_used: {}\n---\n\n",
        entry["chat_id"].as_str().unwrap_or(""),
        entry["title"].as_str().unwrap_or("Untitled Chat"),
        entry["created_at"].as_str().unwrap_or(""),
        entry["last_used_at"].as_str().unwrap_or(""),
        entry["models_used"].as_array()
            .map(|arr| arr.iter().filter_map(|v| v.as_str()).collect::<Vec<_>>().join(", "))
            .unwrap_or_default()
    );
    fs::write(&chat_path, md).map_err(|e| format!("Failed to create chat file: {}", e))?;

    index.push(entry.clone());
    write_chats_index(&index)?;
    Ok(entry)
}

#[tauri::command]
async fn append_chat_log_message(chat_id: String, role: String, content: String, model: String) -> Result<serde_json::Value, String> {
    let role_norm = role.trim().to_lowercase();
    if role_norm != "user" && role_norm != "assistant" && role_norm != "system" {
        return Err("Invalid chat role".to_string());
    }

    let now = Utc::now().to_rfc3339();
    let mut index = read_chats_index()?;
    let idx = index
        .iter()
        .position(|item| item.get("chat_id").and_then(|v| v.as_str()) == Some(chat_id.as_str()))
        .ok_or_else(|| "Chat not found".to_string())?;

    let path = chat_markdown_path(&chat_id)?;
    if !path.exists() {
        return Err("Chat markdown file not found".to_string());
    }

    let model_label = model.trim();
    let section = format!(
        "## {} | {} | {}\n\n{}\n\n",
        role_norm.to_uppercase(),
        now,
        if model_label.is_empty() { "unknown" } else { model_label },
        content
    );

    let mut existing = fs::read_to_string(&path).map_err(|e| format!("Failed to read chat file: {}", e))?;
    existing.push_str(&section);
    fs::write(&path, existing).map_err(|e| format!("Failed to append chat file: {}", e))?;

    let message_count = index[idx].get("message_count").and_then(|v| v.as_i64()).unwrap_or(0) + 1;
    index[idx]["message_count"] = serde_json::json!(message_count);
    index[idx]["last_used_at"] = serde_json::json!(now);
    if !model_label.is_empty() {
        index[idx]["last_model"] = serde_json::json!(model_label);
        let mut models = index[idx]
            .get("models_used")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();
        if !models.iter().any(|v| v.as_str() == Some(model_label)) {
            models.push(serde_json::json!(model_label));
            index[idx]["models_used"] = serde_json::Value::Array(models);
        }
    }

    write_chats_index(&index)?;
    Ok(index[idx].clone())
}

#[tauri::command]
async fn rename_chat_log(chat_id: String, title: String) -> Result<serde_json::Value, String> {
    let mut index = read_chats_index()?;
    let idx = index
        .iter()
        .position(|item| item.get("chat_id").and_then(|v| v.as_str()) == Some(chat_id.as_str()))
        .ok_or_else(|| "Chat not found".to_string())?;

    let cleaned = sanitize_chat_title(&title);
    index[idx]["title"] = serde_json::json!(cleaned);
    index[idx]["last_used_at"] = serde_json::json!(Utc::now().to_rfc3339());
    write_chats_index(&index)?;
    Ok(index[idx].clone())
}

#[tauri::command]
async fn get_chat_log(chat_id: String) -> Result<serde_json::Value, String> {
    let index = read_chats_index()?;
    let entry = index
        .iter()
        .find(|item| item.get("chat_id").and_then(|v| v.as_str()) == Some(chat_id.as_str()))
        .cloned()
        .ok_or_else(|| "Chat not found".to_string())?;

    let path = chat_markdown_path(&chat_id)?;
    let markdown = fs::read_to_string(&path).map_err(|e| format!("Failed to read chat file: {}", e))?;

    Ok(serde_json::json!({
        "entry": entry,
        "markdown": markdown
    }))
}

#[tauri::command]
async fn search_chat_logs(term: String) -> Result<Vec<serde_json::Value>, String> {
    let needle = term.trim().to_lowercase();
    if needle.is_empty() {
        return list_chat_logs().await;
    }

    let index = read_chats_index()?;
    let mut matches = Vec::new();

    for item in index {
        let chat_id = item.get("chat_id").and_then(|v| v.as_str()).unwrap_or("");
        let title = item.get("title").and_then(|v| v.as_str()).unwrap_or("");
        let mut is_match = title.to_lowercase().contains(&needle);
        if !is_match && !chat_id.is_empty() {
            if let Ok(path) = chat_markdown_path(chat_id) {
                if let Ok(md) = fs::read_to_string(path) {
                    if md.to_lowercase().contains(&needle) {
                        is_match = true;
                    }
                }
            }
        }

        if is_match {
            matches.push(item);
        }
    }

    matches.sort_by(|a, b| {
        let a_ts = a.get("last_used_at").and_then(|v| v.as_str()).unwrap_or("");
        let b_ts = b.get("last_used_at").and_then(|v| v.as_str()).unwrap_or("");
        b_ts.cmp(a_ts)
    });
    Ok(matches)
}

/// Detect backend type from asset name
fn detect_backend_type(asset_name: &str) -> String {
    let name_lower = asset_name.to_lowercase();
    
    if name_lower.contains("cuda") || name_lower.contains("cudart") {
        "cuda".to_string()
    } else if name_lower.contains("rocm") || name_lower.contains("hip") {
        "rocm".to_string()
    } else if name_lower.contains("vulkan") {
        "vulkan".to_string()
    } else if name_lower.contains("opencl") {
        "opencl".to_string()
    } else if name_lower.contains("metal") {
        "metal".to_string()
    } else if name_lower.contains("cpu") || (!name_lower.contains("cuda") && !name_lower.contains("vulkan") && !name_lower.contains("opencl") && !name_lower.contains("metal") && !name_lower.contains("rocm") && !name_lower.contains("hip")) {
        "cpu".to_string()
    } else {
        "unknown".to_string()
    }
}


// Global application state
#[derive(Debug)]
pub struct AppState {
    pub config: Arc<Mutex<GlobalConfig>>,
    pub model_configs: Arc<Mutex<HashMap<String, ModelConfig>>>,
    pub running_processes: Arc<Mutex<HashMap<String, ProcessInfo>>>,
    pub child_processes: Arc<Mutex<HashMap<String, Arc<Mutex<ProcessHandle>>>>>, // Simplified process tracking
    pub session_state: Arc<Mutex<SessionState>>,
    pub download_manager: Arc<Mutex<DownloadManager>>,
    pub tracker_manager: Arc<Mutex<Option<TrackerManager>>>,
    pub openai_proxy: Arc<Mutex<Option<openai_proxy::ProxyServer>>>,
}

// Implement Clone manually to avoid derive issues with Child
impl Clone for AppState {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            model_configs: self.model_configs.clone(),
            running_processes: self.running_processes.clone(),
            child_processes: self.child_processes.clone(),
            session_state: self.session_state.clone(),
            download_manager: self.download_manager.clone(),
            tracker_manager: self.tracker_manager.clone(),
            openai_proxy: self.openai_proxy.clone(),
        }
    }
}

impl AppState {
    pub fn new() -> Self {
        Self {
            config: Arc::new(Mutex::new(GlobalConfig::default())),
            model_configs: Arc::new(Mutex::new(HashMap::new())),
            running_processes: Arc::new(Mutex::new(HashMap::new())),
            child_processes: Arc::new(Mutex::new(HashMap::new())),
            session_state: Arc::new(Mutex::new(SessionState::default())),
            download_manager: Arc::new(Mutex::new(DownloadManager::new())),
            tracker_manager: Arc::new(Mutex::new(None)),
            openai_proxy: Arc::new(Mutex::new(None)),
        }
    }
    
    // Method to cleanup all child processes when app exits
    pub async fn cleanup_all_processes(&self) {
        println!("Starting cleanup of all child processes...");
        
        let process_count = {
            let child_processes = self.child_processes.lock().await;
            child_processes.len()
        };
        
        println!("Found {} processes to clean up", process_count);
        
        if process_count == 0 {
            println!("No processes to clean up");
            return;
        }
        
        let mut child_processes = self.child_processes.lock().await;
        let mut running_processes = self.running_processes.lock().await;
        
        for (process_id, handle_arc) in child_processes.drain() {
            println!("Terminating process: {}", process_id);
            let mut handle_guard = handle_arc.lock().await;
            if let Some(mut child) = handle_guard.take_child() {
                match child.kill().await {
                    Ok(_) => println!("Successfully killed process: {}", process_id),
                    Err(e) => {
                        eprintln!("Failed to kill process {}: {}", process_id, e);
                        // Try to force kill on Windows
                        #[cfg(windows)]
                        {
                            if let Some(id) = child.id() {
                                println!("Attempting force kill of PID: {}", id);
                                let _ = std::process::Command::new("taskkill")
                                    .args(["/PID", &id.to_string(), "/F"])
                                    .output();
                            }
                        }
                    }
                }
            } else {
                println!("Process {} already terminated", process_id);
            }
        }
        
        // Clear the running processes list
        running_processes.clear();
        println!("Process cleanup completed");
    }
    
    // Force cleanup that drops all child processes immediately
    // This relies on kill_on_drop(true) to terminate the processes
    pub fn force_cleanup_all_processes(&self) {
        println!("Force cleaning up all child processes (synchronous)...");
        
        // Use try_lock to avoid blocking if already locked
        if let Ok(mut child_processes) = self.child_processes.try_lock() {
            let count = child_processes.len();
            if count == 0 {
                println!("No processes to clean up");
                return;
            }
            
            println!("Force cleaning {} processes", count);
            
            // On Windows, use taskkill for immediate termination
            #[cfg(windows)]
            {
                // Collect all PIDs first
                let mut pids = Vec::new();
                for (_process_id, handle_arc) in child_processes.iter() {
                    if let Ok(handle_guard) = handle_arc.try_lock() {
                        if let Some(pid) = handle_guard.get_child_id() {
                            pids.push(pid);
                        }
                    }
                }
                
                // Kill all processes at once if we have PIDs
                if !pids.is_empty() {
                    println!("Force killing {} PIDs", pids.len());
                    for pid in pids {
                        let _ = std::process::Command::new("taskkill")
                            .args(["/PID", &pid.to_string(), "/F", "/T"]) // /T kills child processes too
                            .status();
                    }
                }
            }
            
            #[cfg(not(windows))]
            {
                // On Unix systems, use kill -9
                for (process_id, handle_arc) in child_processes.iter() {
                    if let Ok(handle_guard) = handle_arc.try_lock() {
                        if let Some(pid) = handle_guard.get_child_id() {
                            let _ = std::process::Command::new("kill")
                                .args(["-9", &pid.to_string()])
                                .status();
                        }
                    }
                }
            }
            
            child_processes.clear(); // This will drop all ProcessHandle instances
            println!("Force dropped {} process handles", count);
        } else {
            println!("Could not acquire lock for force cleanup, relying on kill_on_drop");
        }
        
        if let Ok(mut running_processes) = self.running_processes.try_lock() {
            running_processes.clear();
        }
        
        println!("Force cleanup completed");
    }
}

// Implement Drop trait for emergency cleanup
// Note: This will only be called when the entire application is shutting down
impl Drop for AppState {
    fn drop(&mut self) {
        // For now, let's be conservative and NOT do global cleanup in Drop
        // The window event handlers should handle cleanup when needed
        println!("AppState dropping, skipping emergency process cleanup in Drop implementation");
        
        // If you want to be extra safe, you could do this:
        /*
        let has_processes = {
            if let Ok(child_processes) = self.child_processes.try_lock() {
                !child_processes.is_empty()
            } else {
                false
            }
        };
        
        if has_processes {
            println!("AppState dropping with running processes, but skipping cleanup in Drop");
        }
        */
    }
}

// Tauri commands
#[tauri::command]
async fn get_config(state: tauri::State<'_, AppState>) -> Result<GlobalConfig, String> {
    let config = state.config.lock().await;
    Ok(config.clone())
}

#[tauri::command]
async fn save_config(
    models_directory: String,
    additional_models_directories: Vec<String>,
    executable_folder: String,
    theme_color: String,
    background_color: String,
    theme_is_synced: bool,
    state: tauri::State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    println!("Saving config: models_dir={}, additional_dirs={:?}, exec_folder={}, theme={}, background={}, synced={}", 
        models_directory, additional_models_directories, executable_folder, theme_color, background_color, theme_is_synced);
    
    // Preserve existing active executable folder and proxy/network settings
    let (existing_active_path, existing_active_version, existing_proxy_enabled, existing_proxy_port, existing_network_host, existing_network_port, existing_mcp_servers) = {
        let cfg = state.config.lock().await;
        (
            cfg.active_executable_folder.clone(),
            cfg.active_executable_version.clone(),
            cfg.openai_proxy_enabled,
            cfg.openai_proxy_port,
            cfg.network_server_host.clone(),
            cfg.network_server_port,
            cfg.mcp_servers.clone(),
        )
    };
    
    // Filter out empty strings from additional directories
    let additional_dirs: Vec<String> = additional_models_directories
        .into_iter()
        .filter(|s| !s.is_empty())
        .collect();
    
    let config = GlobalConfig {
        models_directory: models_directory.clone(),
        additional_models_directories: additional_dirs.clone(),
        executable_folder,
        active_executable_folder: existing_active_path,
        active_executable_version: existing_active_version,
        theme_color,
        background_color,
        theme_is_synced,
        openai_proxy_enabled: existing_proxy_enabled,
        openai_proxy_port: existing_proxy_port,
        network_server_host: existing_network_host,
        network_server_port: existing_network_port,
        mcp_servers: existing_mcp_servers,
    };
    
    // Update global config
    {
        let mut global_config = state.config.lock().await;
        *global_config = config.clone();
    }
    
    // Save to file
    if let Err(e) = save_settings(&state).await {
        println!("Failed to save settings: {}", e);
        return Ok(serde_json::json!({
            "success": false,
            "error": format!("Failed to save settings: {}", e)
        }));
    }
    
    // Build list of all directories to scan and cleanup
    let mut all_directories = vec![models_directory.clone()];
    all_directories.extend(additional_dirs);
    
    // Cleanup leftover download files in all models directories
    for dir in &all_directories {
        if let Err(e) = huggingface::cleanup_leftover_downloads(dir).await {
            eprintln!("Warning: Failed to cleanup leftover downloads in {}: {}", dir, e);
        }
    }
    
    // Scan models from all directories
    match scan_models(&all_directories).await {
        Ok(models) => {
            println!("Successfully scanned {} models from {} directories", models.len(), all_directories.len());
            Ok(serde_json::json!({
                "success": true,
                "models": models
            }))
        },
        Err(e) => {
            println!("Failed to scan models: {}", e);
            Ok(serde_json::json!({
                "success": false,
                "error": format!("Failed to scan models: {}", e)
            }))
        }
    }
}

#[tauri::command]
async fn scan_models_command(
    state: tauri::State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    let config = state.config.lock().await;
    
    // Build list of all directories to scan
    let mut all_directories = vec![config.models_directory.clone()];
    all_directories.extend(config.additional_models_directories.clone());
    
    let models = scan_models(&all_directories).await
        .map_err(|e| format!("Failed to scan models: {}", e))?;
    
    Ok(serde_json::json!({
        "success": true,
        "models": models
    }))
}

#[tauri::command]
async fn scan_mmproj_files_command(
    state: tauri::State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    let config = state.config.lock().await;
    
    // Build list of all directories to scan
    let mut all_directories = vec![config.models_directory.clone()];
    all_directories.extend(config.additional_models_directories.clone());
    
    let files = scan_mmproj_files(&all_directories).await
        .map_err(|e| format!("Failed to scan mmproj files: {}", e))?;
    
    Ok(serde_json::json!({
        "success": true,
        "files": files
    }))
}

#[tauri::command]
async fn get_model_settings(
    model_path: String,
    state: tauri::State<'_, AppState>,
) -> Result<ModelConfig, String> {
    let model_configs = state.model_configs.lock().await;
    Ok(model_configs.get(&model_path)
        .cloned()
        .unwrap_or_else(|| ModelConfig::new(model_path)))
}

#[tauri::command]
async fn update_model_settings(
    model_path: String,
    config: ModelConfig,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    {
        let mut model_configs = state.model_configs.lock().await;
        model_configs.insert(model_path, config);
    }
    
    save_settings(&state).await
        .map_err(|e| format!("Failed to save settings: {}", e))
}

#[tauri::command]
async fn get_model_presets(
    model_path: String,
    state: tauri::State<'_, AppState>,
) -> Result<Vec<ModelPreset>, String> {
    let model_configs = state.model_configs.lock().await;
    let config = model_configs.get(&model_path)
        .cloned()
        .unwrap_or_else(|| ModelConfig::new(model_path.clone()));
    Ok(config.presets)
}

#[tauri::command]
async fn update_model_presets(
    model_path: String,
    presets: Vec<ModelPreset>,
    state: tauri::State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    // Update the model config
    {
        let mut model_configs = state.model_configs.lock().await;
        let mut config = model_configs.get(&model_path)
            .cloned()
            .unwrap_or_else(|| ModelConfig::new(model_path.clone()));
        
        // Replace all presets with the new list
        config.presets = presets;
        
        // Update the config
        model_configs.insert(model_path, config);
    } // Release the lock here
    
    // Save to disk (this will acquire its own locks)
    if let Err(e) = save_settings(&state).await {
        return Err(format!("Failed to save settings: {}", e));
    }
    
    Ok(serde_json::json!({
        "success": true,
        "message": "Presets updated successfully"
    }))
}

#[tauri::command]
async fn save_model_preset(
    model_path: String,
    preset: ModelPreset,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    println!("Saving preset: {:?} for model: {}", preset, model_path);
    {
        let mut model_configs = state.model_configs.lock().await;
        let mut config = model_configs.get(&model_path)
            .cloned()
            .unwrap_or_else(|| ModelConfig::new(model_path.clone()));
        
        println!("Current presets count: {}", config.presets.len());
        
        // Check if preset with this ID already exists
        if let Some(existing) = config.presets.iter_mut().find(|p| p.id == preset.id) {
            println!("Updating existing preset: {}", preset.id);
            *existing = preset.clone();
        } else {
            println!("Adding new preset: {}", preset.id);
            config.presets.push(preset.clone());
        }
        
        println!("New presets count: {}", config.presets.len());
        
        // If this preset is marked as default, update default_preset_id
        if preset.is_default {
            // Remove default flag from other presets
            for p in config.presets.iter_mut() {
                if p.id != preset.id {
                    p.is_default = false;
                }
            }
            config.default_preset_id = Some(preset.id.clone());
        }
        
        model_configs.insert(model_path.clone(), config);
        println!("Model config updated in state");
    }
    
    println!("Calling save_settings...");
    let result = save_settings(&state).await
        .map_err(|e| format!("Failed to save settings: {}", e));
    
    if result.is_ok() {
        println!("Settings saved successfully");
    } else {
        println!("Failed to save settings: {:?}", result);
    }
    
    result
}

#[tauri::command]
async fn delete_model_preset(
    model_path: String,
    preset_id: String,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    {
        let mut model_configs = state.model_configs.lock().await;
        let mut config = model_configs.get(&model_path)
            .cloned()
            .unwrap_or_else(|| ModelConfig::new(model_path.clone()));
        
        config.presets.retain(|p| p.id != preset_id);
        
        // If the deleted preset was the default, clear default_preset_id
        if config.default_preset_id.as_ref() == Some(&preset_id) {
            config.default_preset_id = None;
        }
        
        model_configs.insert(model_path, config);
    }
    
    save_settings(&state).await
        .map_err(|e| format!("Failed to save settings: {}", e))
}

#[tauri::command]
async fn set_default_preset(
    model_path: String,
    preset_id: String,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    {
        let mut model_configs = state.model_configs.lock().await;
        let mut config = model_configs.get(&model_path)
            .cloned()
            .unwrap_or_else(|| ModelConfig::new(model_path.clone()));
        
        // Update default flag on all presets
        for preset in config.presets.iter_mut() {
            preset.is_default = preset.id == preset_id;
        }
        
        config.default_preset_id = Some(preset_id);
        model_configs.insert(model_path, config);
    }
    
    save_settings(&state).await
        .map_err(|e| format!("Failed to save settings: {}", e))
}

#[tauri::command]
async fn launch_model_with_preset(
    model_path: String,
    preset_id: Option<String>,
    state: tauri::State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    // Get the preset arguments and env vars
    let (custom_args, env_vars) = {
        let model_configs = state.model_configs.lock().await;
        let config = model_configs.get(&model_path)
            .cloned()
            .unwrap_or_else(|| ModelConfig::new(model_path.clone()));
        
        if let Some(pid) = preset_id {
            // Find the preset
            config.presets.iter()
                .find(|p| p.id == pid)
                .map(|p| {
                    let mut envs = config.env_vars.clone();
                    envs.extend(p.env_vars.clone());
                    (p.custom_args.clone(), envs)
                })
                .unwrap_or_else(|| (config.custom_args.clone(), config.env_vars.clone()))
        } else if let Some(default_id) = config.default_preset_id {
            // Use default preset
            config.presets.iter()
                .find(|p| p.id == default_id)
                .map(|p| {
                    let mut envs = config.env_vars.clone();
                    envs.extend(p.env_vars.clone());
                    (p.custom_args.clone(), envs)
                })
                .unwrap_or_else(|| (config.custom_args.clone(), config.env_vars.clone()))
        } else {
            // Use current custom_args
            (config.custom_args.clone(), config.env_vars.clone())
        }
    };
    
    // Store original args for restoration
    let (original_args, original_env_vars) = {
        let model_configs = state.model_configs.lock().await;
        let config = model_configs.get(&model_path)
            .cloned()
            .unwrap_or_else(|| ModelConfig::new(model_path.clone()));
        (config.custom_args.clone(), config.env_vars.clone())
    };
    
    // Temporarily update the model config with preset args
    {
        let mut model_configs = state.model_configs.lock().await;
        let mut config = model_configs.get(&model_path)
            .cloned()
            .unwrap_or_else(|| ModelConfig::new(model_path.clone()));
        
        config.custom_args = custom_args;
        config.env_vars = env_vars;
        model_configs.insert(model_path.clone(), config);
    } // Release the lock here
    
    // Launch the model (this may acquire locks internally)
    let result = launch_model_server(model_path.clone(), &state).await
        .map_err(|e| format!("Failed to launch model: {}", e))?;
    
    // Restore original args
    {
        let mut model_configs = state.model_configs.lock().await;
        let mut config = model_configs.get(&model_path)
            .cloned()
            .unwrap_or_else(|| ModelConfig::new(model_path.clone()));
        config.custom_args = original_args;
        config.env_vars = original_env_vars;
        model_configs.insert(model_path, config);
    }
    
    Ok(serde_json::json!({
        "success": true,
        "process_id": result.process_id,
        "model_name": result.model_name,
        "server_host": result.server_host,
        "server_port": result.server_port
    }))
}

fn append_half_context_arg(custom_args: &str) -> String {
    let trimmed_args = custom_args.trim();
    if trimmed_args.is_empty() {
        "--context-shift".to_string()
    } else if trimmed_args
        .split_whitespace()
        .any(|token| token == "--context-shift")
    {
        trimmed_args.to_string()
    } else {
        format!("{} {}", trimmed_args, "--context-shift")
    }
}

#[tauri::command]
async fn launch_model_with_half_context(
    model_path: String,
    state: tauri::State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    let custom_args_with_half_context = {
        let model_configs = state.model_configs.lock().await;
        let config = model_configs.get(&model_path)
            .cloned()
            .unwrap_or_else(|| ModelConfig::new(model_path.clone()));

        append_half_context_arg(&config.custom_args)
    };

    let (original_args, original_env_vars) = {
        let model_configs = state.model_configs.lock().await;
        let config = model_configs.get(&model_path)
            .cloned()
            .unwrap_or_else(|| ModelConfig::new(model_path.clone()));
        (config.custom_args.clone(), config.env_vars.clone())
    };

    {
        let mut model_configs = state.model_configs.lock().await;
        let mut config = model_configs.get(&model_path)
            .cloned()
            .unwrap_or_else(|| ModelConfig::new(model_path.clone()));

        config.custom_args = custom_args_with_half_context;
        model_configs.insert(model_path.clone(), config);
    }

    let result = launch_model_server(model_path.clone(), &state)
        .await
        .map_err(|e| format!("Failed to launch model with half context: {}", e))?;

    {
        let mut model_configs = state.model_configs.lock().await;
        let mut config = model_configs.get(&model_path)
            .cloned()
            .unwrap_or_else(|| ModelConfig::new(model_path.clone()));
        config.custom_args = original_args;
        config.env_vars = original_env_vars;
        model_configs.insert(model_path, config);
    }

    Ok(serde_json::json!({
        "success": true,
        "process_id": result.process_id,
        "model_name": result.model_name,
        "server_host": result.server_host,
        "server_port": result.server_port
    }))
}

#[tauri::command]
async fn launch_model(
    model_path: String,
    state: tauri::State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    let result = launch_model_server(model_path, &state).await
        .map_err(|e| format!("Failed to launch model: {}", e))?;
    
    Ok(serde_json::json!({
        "success": true,
        "process_id": result.process_id,
        "model_name": result.model_name,
        "server_host": result.server_host,
        "server_port": result.server_port
    }))
}

#[tauri::command]
async fn launch_model_external(
    model_path: String,
    state: tauri::State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    let result = launch_model_external_impl(model_path, &state).await
        .map_err(|e| format!("Failed to launch model externally: {}", e))?;
    
    Ok(serde_json::json!({
        "success": true,
        "message": result.message
    }))
}

#[tauri::command]
async fn launch_model_with_preset_external(
    model_path: String,
    preset_id: Option<String>,
    state: tauri::State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    // Get the preset arguments and env vars
    let (custom_args, env_vars) = {
        let model_configs = state.model_configs.lock().await;
        let config = model_configs.get(&model_path)
            .cloned()
            .unwrap_or_else(|| ModelConfig::new(model_path.clone()));
        
        if let Some(pid) = preset_id {
            // Find the preset
            config.presets.iter()
                .find(|p| p.id == pid)
                .map(|p| {
                    let mut envs = config.env_vars.clone();
                    envs.extend(p.env_vars.clone());
                    (p.custom_args.clone(), envs)
                })
                .unwrap_or_else(|| (config.custom_args.clone(), config.env_vars.clone()))
        } else if let Some(default_id) = config.default_preset_id {
            // Use default preset
            config.presets.iter()
                .find(|p| p.id == default_id)
                .map(|p| {
                    let mut envs = config.env_vars.clone();
                    envs.extend(p.env_vars.clone());
                    (p.custom_args.clone(), envs)
                })
                .unwrap_or_else(|| (config.custom_args.clone(), config.env_vars.clone()))
        } else {
            // Use current custom_args
            (config.custom_args.clone(), config.env_vars.clone())
        }
    };
    
    // Store original args for restoration
    let (original_args, original_env_vars) = {
        let model_configs = state.model_configs.lock().await;
        let config = model_configs.get(&model_path)
            .cloned()
            .unwrap_or_else(|| ModelConfig::new(model_path.clone()));
        (config.custom_args.clone(), config.env_vars.clone())
    };
    
    // Temporarily update the model config with preset args
    {
        let mut model_configs = state.model_configs.lock().await;
        let mut config = model_configs.get(&model_path)
            .cloned()
            .unwrap_or_else(|| ModelConfig::new(model_path.clone()));
        
        config.custom_args = custom_args;
        config.env_vars = env_vars;
        model_configs.insert(model_path.clone(), config);
    } // Release the lock here
    
    // Launch the model externally (this may acquire locks internally)
    let result = launch_model_external_impl(model_path.clone(), &state).await
        .map_err(|e| format!("Failed to launch model externally: {}", e))?;
    
    // Restore original args
    {
        let mut model_configs = state.model_configs.lock().await;
        let mut config = model_configs.get(&model_path)
            .cloned()
            .unwrap_or_else(|| ModelConfig::new(model_path.clone()));
        config.custom_args = original_args;
        config.env_vars = original_env_vars;
        model_configs.insert(model_path, config);
    }
    
    Ok(serde_json::json!({
        "success": true,
        "message": result.message
    }))
}

#[tauri::command]
async fn delete_model_file(
    model_path: String,
    state: tauri::State<'_, AppState>,
    app_handle: tauri::AppHandle,
) -> Result<serde_json::Value, String> {
    use std::fs;
    
    // Security checks - scope the config lock
    let (allowed_dirs, model_file) = {
        let config = state.config.lock().await;
        let mut all_dirs = vec![config.models_directory.clone()];
        all_dirs.extend(config.additional_models_directories.clone());
        let allowed_dirs: Vec<PathBuf> = all_dirs.into_iter().map(PathBuf::from).collect();
        let model_file = PathBuf::from(&model_path);
        (allowed_dirs, model_file)
    }; // Config lock is dropped here
    
    // Check if file exists before deletion
    if !model_file.exists() {
        return Ok(serde_json::json!({
            "success": false,
            "error": "File does not exist"
        }));
    }
    
    // Ensure the file is within one of the allowed models directories
    let is_in_allowed_dir = allowed_dirs.iter().any(|dir| model_file.starts_with(dir));
    if !is_in_allowed_dir {
        return Ok(serde_json::json!({
            "success": false,
            "error": "Cannot delete files outside of models directories"
        }));
    }
    
    // Ensure it's a .gguf file
    if !model_path.to_lowercase().ends_with(".gguf") {
        return Ok(serde_json::json!({
            "success": false,
            "error": "Only .gguf files can be deleted"
        }));
    }
    
    // Delete the file
    match fs::remove_file(&model_path) {
        Ok(_) => {
            // Remove from model configs - scope the lock
            {
                let mut model_configs = state.model_configs.lock().await;
                model_configs.remove(&model_path);
            } // Model configs lock is dropped here
            
            // Save settings
            if let Err(e) = save_settings(&state).await {
                return Ok(serde_json::json!({
                    "success": false,
                    "error": format!("Failed to save settings: {}", e)
                }));
            }
            
            // Add a small delay to ensure file system has processed the deletion
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            
            // Emit file deletion event to frontend
            use tauri::Emitter;
            let _ = app_handle.emit("file-deleted", ());
            
            Ok(serde_json::json!({
                "success": true
            }))
        },
        Err(e) => {
            Ok(serde_json::json!({
                "success": false,
                "error": format!("Failed to delete file: {}", e)
            }))
        }
    }
}

#[tauri::command]
async fn kill_process(
    process_id: String,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    terminate_process(process_id, &state).await
        .map_err(|e| format!("Failed to kill process: {}", e))
}

#[tauri::command]
async fn get_process_output(
    process_id: String,
    state: tauri::State<'_, AppState>,
) -> Result<ProcessOutput, String> {
    get_process_logs(process_id, &state).await
        .map_err(|e| format!("Failed to get process output: {}", e))
}

#[tauri::command]
async fn browse_folder(
    initial_dir: Option<String>,
    app: tauri::AppHandle,
) -> Result<Option<String>, String> {
    use tauri_plugin_dialog::DialogExt;
    
    let dialog = app.dialog();
    let mut file_dialog = dialog.file();
    
    if let Some(initial) = initial_dir {
        file_dialog = file_dialog.set_directory(initial);
    }
    
    // Use a channel to convert callback to async
    let (tx, rx) = tokio::sync::oneshot::channel();
    
    file_dialog.pick_folder(move |path| {
        let result = path.map(|p| p.to_string());
        let _ = tx.send(result);
    });
    
    match rx.await {
        Ok(result) => Ok(result),
        Err(_) => Err("Dialog was cancelled or failed".to_string()),
    }
}

#[tauri::command]
async fn open_url(url: String, app: tauri::AppHandle) -> Result<(), String> {
    use tauri_plugin_opener::OpenerExt;
    
    // Open URL in default browser using opener plugin
    app.opener()
        .open_url(url, None::<String>)
        .map_err(|e| format!("Failed to open URL: {}", e))
}

#[tauri::command]
async fn open_model_folder(model_path: String, app: tauri::AppHandle) -> Result<(), String> {
    use tauri_plugin_opener::OpenerExt;
    use std::path::Path;
    
    // Get the directory containing the model file
    let model_file_path = Path::new(&model_path);
    let folder_path = model_file_path.parent()
        .ok_or_else(|| "Could not determine parent directory".to_string())?;
    
    // Convert path to string for the opener plugin
    let folder_path_str = folder_path.to_string_lossy().to_string();
    
    // Open folder in default file manager using opener plugin
    app.opener()
        .open_path(folder_path_str, None::<String>)
        .map_err(|e| format!("Failed to open folder: {}", e))
}

#[tauri::command]
async fn search_huggingface(
    query: String,
    limit: Option<usize>,
    sort_by: Option<String>,
) -> Result<SearchResult, String> {
    search_models(query, limit.unwrap_or(100), sort_by.unwrap_or_else(|| "relevance".to_string()))
        .await
        .map_err(|e| format!("Search failed: {}", e))
}

#[tauri::command]
async fn get_model_details(
    model_id: String,
) -> Result<ModelDetails, String> {
    get_huggingface_model_details(model_id)
        .await
        .map_err(|e| format!("Failed to get model details: {}", e))
}

#[tauri::command]
async fn download_model(
    model_id: String,
    _filename: String,
    files: Vec<String>,
    state: tauri::State<'_, AppState>,
   app_handle: tauri::AppHandle,
) -> Result<DownloadStartResult, String> {
    use crate::downloader::{DownloadConfig, start_download};
    
    // Get models directory from config
    let models_directory = {
        let config = state.config.lock().await;
        config.models_directory.clone()
    };
    
    // Create destination folder structure: models_directory/author/model_name/
    let author = model_id.split('/').next().unwrap_or("unknown");
    let model_name = model_id.split('/').nth(1).unwrap_or(&model_id);
    let destination_folder = format!("{}/{}/{}", models_directory, author, model_name);
    
    // Create download configuration
    let config = DownloadConfig {
        base_url: format!("https://huggingface.co/{}/resolve/main", model_id),
        destination_folder,
        auto_extract: false, // GGUF files don't need extraction
        create_subfolder: None, // We already created the subfolder structure
        files: files.clone(),
        custom_headers: Some({
            let mut headers = std::collections::HashMap::new();
            headers.insert("User-Agent".to_string(), "Arandu-Tauri/1.0".to_string());
            headers
        }),
    };
    
    start_download(config, &state, app_handle)
        .await
        .map_err(|e| format!("Failed to start download: {}", e))
}

#[tauri::command]
async fn get_download_status(
    download_id: String,
    state: tauri::State<'_, AppState>,
) -> Result<DownloadStatus, String> {
    let download_manager = state.download_manager.lock().await;
    download_manager.get_status(&download_id)
        .map(|status| status.clone())
        .ok_or_else(|| "Download not found".to_string())
}

#[tauri::command]
async fn get_all_downloads(
    state: tauri::State<'_, AppState>,
) -> Result<Vec<DownloadStatus>, String> {
    let download_manager = state.download_manager.lock().await;
    Ok(download_manager.downloads.values().cloned().collect())
}

#[tauri::command]
async fn cancel_download(
    download_id: String,
    state: tauri::State<'_, AppState>,
) -> Result<Vec<DownloadStatus>, String> {
    let mut download_manager = state.download_manager.lock().await;
    download_manager.cancel_download(&download_id).map_err(|e| format!("Failed to cancel download: {}", e))?;
    Ok(download_manager.downloads.values().cloned().collect())
}

#[tauri::command]
async fn pause_download(
    download_id: String,
    state: tauri::State<'_, AppState>,
) -> Result<Vec<DownloadStatus>, String> {
    let mut download_manager = state.download_manager.lock().await;
    download_manager.pause_download(&download_id).map_err(|e| format!("Failed to pause download: {}", e))?;
    Ok(download_manager.downloads.values().cloned().collect())
}

#[tauri::command]
async fn resume_download(
    download_id: String,
    state: tauri::State<'_, AppState>,
) -> Result<Vec<DownloadStatus>, String> {
    let mut download_manager = state.download_manager.lock().await;
    download_manager.resume_download(&download_id).map_err(|e| format!("Failed to resume download: {}", e))?;
    Ok(download_manager.downloads.values().cloned().collect())
}

#[tauri::command]
async fn get_all_downloads_and_history(
    state: tauri::State<'_, AppState>,
) -> Result<Vec<DownloadStatus>, String> {
    let download_manager = state.download_manager.lock().await;
    let mut all_downloads = download_manager.downloads.values().cloned().collect::<Vec<_>>();
    all_downloads.extend(download_manager.download_history.clone());
    Ok(all_downloads)
}

#[tauri::command]
async fn clear_download_history(
    state: tauri::State<'_, AppState>,
) -> Result<Vec<DownloadStatus>, String> {
    let mut download_manager = state.download_manager.lock().await;
    download_manager.clear_download_history();
    let mut all_downloads = download_manager.downloads.values().cloned().collect::<Vec<_>>();
    all_downloads.extend(download_manager.download_history.clone());
    Ok(all_downloads)
}

#[tauri::command]
async fn delete_model(
    model_path: String,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    use std::fs;
    
    // Security checks
    let config = state.config.lock().await;
    let mut all_dirs = vec![config.models_directory.clone()];
    all_dirs.extend(config.additional_models_directories.clone());
    let allowed_dirs: Vec<PathBuf> = all_dirs.into_iter().map(PathBuf::from).collect();
    let model_file = PathBuf::from(&model_path);
    
    // Ensure the file is within one of the allowed models directories
    let is_in_allowed_dir = allowed_dirs.iter().any(|dir| model_file.starts_with(dir));
    if !is_in_allowed_dir {
        return Err("Cannot delete files outside of models directories".to_string());
    }
    
    // Ensure it's a .gguf file
    if !model_path.to_lowercase().ends_with(".gguf") {
        return Err("Only .gguf files can be deleted".to_string());
    }
    
    // Delete the file
    fs::remove_file(&model_path).map_err(|e| format!("Failed to delete file: {}", e))?;
    
    // Remove from model configs
    let mut model_configs = state.model_configs.lock().await;
    model_configs.remove(&model_path);
    
    // Save settings
    save_settings(&state).await.map_err(|e| format!("Failed to save settings: {}", e))?;
    
    Ok(())
}

// Update Checker Commands

#[tauri::command]
async fn initial_scan_models(
    state: tauri::State<'_, AppState>,
) -> Result<InitialScanResult, String> {
    use std::time::SystemTime;
    use std::fs;
    use std::path::Path;
    
    let mut result = InitialScanResult {
        success: true,
        models_processed: 0,
        models_linked: 0,
        errors: Vec::new(),
    };
    
    // Get all model directories
    let all_directories = {
        let config = state.config.lock().await;
        let mut dirs = vec![config.models_directory.clone()];
        dirs.extend(config.additional_models_directories.clone());
        dirs
    };
    
    // Process each directory
    for directory in all_directories {
        if directory.is_empty() || !Path::new(&directory).is_dir() {
            continue;
        }
        
        // Scan for GGUF files
        let pattern = format!("{}/**/*.gguf", directory);
        let entries = match glob::glob(&pattern) {
            Ok(entries) => entries,
            Err(e) => {
                result.errors.push(format!("Failed to scan directory {}: {}", directory, e));
                continue;
            }
        };
        
        for entry in entries {
            let path = match entry {
                Ok(path) => path,
                Err(e) => {
                    result.errors.push(format!("Path error: {}", e));
                    continue;
                }
            };
            
            let path_str = path.to_string_lossy().to_string();
            
            // Get file metadata
            let metadata = match fs::metadata(&path) {
                Ok(meta) => meta,
                Err(e) => {
                    result.errors.push(format!("Metadata error for {}: {}", path_str, e));
                    continue;
                }
            };
            
            let modified_time = metadata.modified()
                .ok()
                .and_then(|t| t.duration_since(SystemTime::UNIX_EPOCH).ok())
                .map(|d| d.as_secs() as i64);
            
            let file_size = metadata.len() as i64;
            
            // Try to extract HF model ID from path (Arandu download structure)
            let hf_model_id = extract_hf_model_id_from_path(&path_str, &directory);
            
            // Update model config
            {
                let mut model_configs = state.model_configs.lock().await;
                let config = model_configs.entry(path_str.clone())
                    .or_insert_with(|| ModelConfig::new(path_str.clone()));
                
                config.local_file_modified = modified_time;
                config.file_size_bytes = Some(file_size);
                
                if let Some(hf_id) = hf_model_id {
                    config.hf_model_id = Some(hf_id);
                    config.hf_link_source = Some("download".to_string());
                    result.models_linked += 1;
                }
            }
            
            result.models_processed += 1;
        }
    }
    
    // Save settings
    if let Err(e) = save_settings(&state).await {
        result.errors.push(format!("Failed to save settings: {}", e));
    }
    
    Ok(result)
}

// Helper function to extract HF model ID from Arandu download path structure
fn extract_hf_model_id_from_path(path: &str, base_dir: &str) -> Option<String> {
    use std::path::Path;
    
    let path_obj = Path::new(path);
    let base_obj = Path::new(base_dir);
    
    // Get relative path from base directory
    let rel_path = path_obj.strip_prefix(base_obj).ok()?;
    let components: Vec<_> = rel_path.components().collect();
    
    // Arandu structure: base_dir/author/model_name/filename.gguf
    if components.len() >= 3 {
        let author = components[0].as_os_str().to_str()?;
        let model_name = components[1].as_os_str().to_str()?;
        Some(format!("{}/{}", author, model_name))
    } else {
        None
    }
}

// Helper function to parse HF date format
fn parse_hf_date(date_str: &str) -> Option<i64> {
    // HF dates are typically in ISO 8601 format: "2024-01-15T10:30:00Z"
    chrono::DateTime::parse_from_rfc3339(date_str)
        .ok()
        .map(|dt| dt.timestamp())
}

#[tauri::command]
async fn get_hf_model_files(
    hf_model_id: String,
) -> Result<HFLinkResult, String> {
    use reqwest;
    
    let url = format!("https://huggingface.co/api/models/{}/tree/main", hf_model_id);
    
    let client = reqwest::Client::new();
    let response = match client.get(&url).send().await {
        Ok(resp) => resp,
        Err(e) => {
            return Ok(HFLinkResult {
                success: false,
                message: format!("Failed to query HF API: {}", e),
                files: Vec::new(),
            });
        }
    };
    
    if !response.status().is_success() {
        return Ok(HFLinkResult {
            success: false,
            message: format!("HF API returned error: {}", response.status()),
            files: Vec::new(),
        });
    }
    
    let data: serde_json::Value = match response.json().await {
        Ok(data) => data,
        Err(e) => {
            return Ok(HFLinkResult {
                success: false,
                message: format!("Failed to parse HF response: {}", e),
                files: Vec::new(),
            });
        }
    };
    
    let files: Vec<HFFileInfo> = data.as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|f| {
                    let path = f.get("path")?.as_str()?;
                    if !path.ends_with(".gguf") {
                        return None;
                    }
                    
                    let size = f.get("size")?.as_i64()?;
                    let last_commit = f.get("lastCommit")?.get("date")?.as_str()?;
                    let last_modified = parse_hf_date(last_commit)?;
                    
                    Some(HFFileInfo {
                        filename: path.to_string(),
                        size,
                        last_modified,
                    })
                })
                .collect()
        })
        .unwrap_or_default();
    
    Ok(HFLinkResult {
        success: !files.is_empty(),
        message: if files.is_empty() {
            "No GGUF files found in repository".to_string()
        } else {
            format!("Found {} GGUF files", files.len())
        },
        files,
    })
}

#[tauri::command]
async fn get_session_state(
    state: tauri::State<'_, AppState>,
) -> Result<SessionState, String> {
    let session = state.session_state.lock().await;
    Ok(session.clone())
}

#[tauri::command]
async fn save_window_state(
    window_id: String,
    window_state: WindowState,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let mut session = state.session_state.lock().await;
    session.windows.insert(window_id, window_state);
    Ok(())
}

#[tauri::command]
async fn restart_application(
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    println!("Application restart requested via command");
    
    // Perform cleanup but don't exit
    state.cleanup_all_processes().await;
    
    // Give time for cleanup to complete
    tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
    
    println!("Application restart cleanup completed - frontend will reload");
    
    // Don't exit - let the frontend handle the reload
    Ok(())
}

#[tauri::command]
async fn graceful_exit(
    state: tauri::State<'_, AppState>,
    app: tauri::AppHandle,
) -> Result<(), String> {
    println!("Graceful exit requested via command");
    
    // Perform cleanup
    state.cleanup_all_processes().await;
    
    // Give time for cleanup to complete
    tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
    
    println!("Graceful exit cleanup completed");
    
    // Exit the application
    app.exit(0);
    
    Ok(())
}

#[tauri::command]
async fn get_app_version() -> Result<String, String> {
    Ok(env!("CARGO_PKG_VERSION").to_string())
}

#[tauri::command]
async fn check_file_exists(
    model_id: String,
    filename: String,
    state: tauri::State<'_, AppState>,
) -> Result<bool, String> {
    use std::path::Path;
    
    let config = state.config.lock().await;
    
    // Build list of all directories to check
    let mut all_directories = vec![config.models_directory.clone()];
    all_directories.extend(config.additional_models_directories.clone());
    
    // Remove empty directories
    all_directories.retain(|dir| !dir.is_empty());
    
    if all_directories.is_empty() {
        return Ok(false);
    }
    
    // Create the expected file path structure (author/model/filename)
    let author = model_id.split('/').next().unwrap_or("unknown");
    let model_name = model_id.split('/').nth(1).unwrap_or(&model_id);
    
    // Check if file exists in any of the directories
    for dir in &all_directories {
        let file_path = Path::new(dir)
            .join(author)
            .join(model_name)
            .join(&filename);
        
        if file_path.exists() {
            return Ok(true);
        }
    }
    
    Ok(false)
}

#[tauri::command]
async fn hide_window(app: tauri::AppHandle) -> Result<(), String> {
    if let Some(main_window) = app.get_webview_window("main") {
        main_window.hide().map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
async fn show_window(app: tauri::AppHandle) -> Result<(), String> {
    if let Some(main_window) = app.get_webview_window("main") {
        main_window.show().map_err(|e| e.to_string())?;
        main_window.set_focus().ok();
    }
    Ok(())
}

#[tauri::command]
async fn get_model_metadata(
    model_path: String,
) -> Result<GgufMetadata, String> {
    gguf_parser::parse_gguf_metadata(&model_path)
}

#[tauri::command]
async fn check_model_update(
    model_path: String,
    state: tauri::State<'_, AppState>,
) -> Result<UpdateCheckResult, String> {
    use std::time::SystemTime;
    
    // Get model config to check for HF metadata and migrate if needed
    let hf_metadata = {
        let mut configs = state.model_configs.lock().await;
        let config = configs.get_mut(&model_path);
        
        if let Some(config) = config {
            if let Some(ref metadata) = config.hf_metadata {
                metadata.clone()
            } else if let Some(ref hf_id) = config.hf_model_id {
                // Migrate legacy HF link to new metadata format
                let filename = std::path::Path::new(&model_path)
                    .file_name()
                    .and_then(|f| f.to_str())
                    .unwrap_or("unknown.gguf")
                    .to_string();
                
                let metadata = HfMetadata {
                    model_id: hf_id.clone(),
                    filename,
                    commit_date: None,
                    linked_at: format!(
                        "{}",
                        SystemTime::now()
                            .duration_since(SystemTime::UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_secs()
                    ),
                };
                
                config.hf_metadata = Some(metadata.clone());
                metadata
            } else {
                // No HF link at all
                return Ok(UpdateCheckResult {
                    status: UpdateStatus::NotLinked,
                    local_date: None,
                    remote_date: None,
                    message: "Model not linked to HuggingFace. Click to link.".to_string(),
                });
            }
        } else {
            // Config doesn't exist
            return Ok(UpdateCheckResult {
                status: UpdateStatus::NotLinked,
                local_date: None,
                remote_date: None,
                message: "Model not linked to HuggingFace. Click to link.".to_string(),
            });
        }
    };
    
    // Save settings to persist any migration
    let _ = save_settings(&state).await;
    
    // Get file modification date
    let modification_date = gguf_parser::get_file_modification_date(&model_path)
        .map_err(|e| format!("Failed to get file date: {}", e))?;
    
    // Check HF for updates
    let result = update_checker::check_huggingface_updates(
        &model_path,
        Some(&hf_metadata),
        modification_date,
    ).await;
    
    Ok(result)
}

#[tauri::command]
async fn link_model_to_hf(
    model_path: String,
    hf_model_id: String,
    hf_filename: String,
    state: tauri::State<'_, AppState>,
) -> Result<HfMetadata, String> {
    // Create HF metadata
    let metadata = update_checker::link_model_to_hf(
        &model_path,
        &hf_model_id,
        &hf_filename,
    )?;
    
    // Store in model config
    {
        let mut configs = state.model_configs.lock().await;
        let config = configs.entry(model_path.clone()).or_insert_with(|| {
            ModelConfig::new(model_path.clone())
        });
        config.hf_metadata = Some(metadata.clone());
    }
    
    // Save settings to persist
    if let Err(e) = save_settings(&state).await {
        eprintln!("Warning: Failed to save settings after linking: {}", e);
    }
    
    Ok(metadata)
}

#[tauri::command]
async fn remove_window_state(
    window_id: String,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let mut session = state.session_state.lock().await;
    session.windows.remove(&window_id);
    Ok(())
}

#[tauri::command]
async fn download_from_url(
    url: String,
    destination_folder: String,
    extract: bool,
    state: tauri::State<'_, AppState>,
    app_handle: tauri::AppHandle,
) -> Result<DownloadStartResult, String> {
    use crate::downloader::{DownloadConfig, start_download};
    
    // Create download configuration
    let config = DownloadConfig {
        base_url: url,
        destination_folder,
        auto_extract: extract,
        create_subfolder: None,
        files: Vec::new(), // Single file download
        custom_headers: None,
    };
    
    start_download(config, &state, app_handle)
        .await
        .map_err(|e| format!("Failed to start download: {}", e))
}

#[tauri::command]
async fn get_llamacpp_releases() -> Result<Vec<LlamaCppRelease>, String> {
    llamacpp_manager::fetch_llamacpp_releases()
        .await
        .map_err(|e| format!("Failed to fetch llama.cpp releases: {}", e))
}

#[tauri::command]
async fn get_llamacpp_commit_info(tag_name: String) -> Result<llamacpp_manager::CommitInfo, String> {
    llamacpp_manager::fetch_commit_info(&tag_name)
        .await
        .map_err(|e| format!("Failed to fetch commit info: {}", e))
}

#[tauri::command]
async fn download_llamacpp_asset(
    asset: LlamaCppAsset,
    state: tauri::State<'_, AppState>,
    app_handle: tauri::AppHandle,
) -> Result<DownloadStartResult, String> {
    use crate::downloader::{DownloadConfig, start_download};
    
    // Get executable folder from config
    let executable_folder = {
        let config = state.config.lock().await;
        config.executable_folder.clone()
    };
    
    // Create download configuration
    let config = DownloadConfig {
        base_url: asset.download_url,
        destination_folder: executable_folder,
        auto_extract: true, // Llama.cpp assets are usually zips
        create_subfolder: None,
        files: Vec::new(), // Single file download
        custom_headers: Some({
            let mut headers = std::collections::HashMap::new();
            headers.insert("User-Agent".to_string(), "Arandu-Tauri/1.0".to_string());
            headers
        }),
    };
    
    start_download(config, &state, app_handle)
        .await
        .map_err(|e| format!("Failed to download llama.cpp asset: {}", e))
}

#[tauri::command]
async fn download_llamacpp_asset_to_version(
    asset: LlamaCppAsset,
    version_folder: String,
    state: tauri::State<'_, AppState>,
    app_handle: tauri::AppHandle,
) -> Result<DownloadStartResult, String> {
    use crate::downloader::{DownloadConfig, start_download};

    // Base executable folder from config
    let base_exec = {
        let config = state.config.lock().await;
        config.executable_folder.clone()
    };

    // Destination: <exec>/versions/<version_folder>
    let destination_folder = std::path::Path::new(&base_exec)
        .join("versions")
        .join(&version_folder)
        .to_string_lossy()
        .to_string();

    // Create download configuration
    let config = DownloadConfig {
        base_url: asset.download_url,
        destination_folder,
        auto_extract: true,
        create_subfolder: None,
        files: Vec::new(),
        custom_headers: Some({
            let mut headers = std::collections::HashMap::new();
            headers.insert("User-Agent".to_string(), "Arandu-Tauri/1.0".to_string());
            headers
        }),
    };

    start_download(config, &state, app_handle)
        .await
        .map_err(|e| format!("Failed to download llama.cpp asset: {}", e))
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
struct LlamaCppInstalledVersion {
    name: String,
    path: String,
    has_server: bool,
    created: Option<i64>,
    is_active: bool,
    backend_type: Option<String>,
}

#[tauri::command]
async fn list_llamacpp_versions(state: tauri::State<'_, AppState>) -> Result<Vec<LlamaCppInstalledVersion>, String> {
    use std::fs;
    use std::time::SystemTime;

    let (base_exec, active_path, active_version) = {
        let cfg = state.config.lock().await;
        (
            cfg.executable_folder.clone(),
            cfg.active_executable_folder.clone(),
            cfg.active_executable_version.clone(),
        )
    };
    let versions_dir = std::path::Path::new(&base_exec).join("versions");
    let mut out = Vec::new();
    
    if versions_dir.exists() {
        if let Ok(read_dir) = fs::read_dir(&versions_dir) {
            for entry in read_dir.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    let version_name = path.file_name().and_then(|s| s.to_str()).unwrap_or("").to_string();
                    
                    // Check if this is a nested structure (version/backend) or old flat structure
                    if let Ok(backend_dir) = fs::read_dir(&path) {
                        for backend_entry in backend_dir.flatten() {
                            let backend_path = backend_entry.path();
                            if backend_path.is_dir() {
                                let backend_name = backend_path.file_name().and_then(|s| s.to_str()).unwrap_or("").to_string();
                                let server_name = if cfg!(windows) { "llama-server.exe" } else { "llama-server" };
                                let has_server = backend_path.join(server_name).exists();
                                let created = backend_entry.metadata().ok()
                                    .and_then(|m| m.created().ok())
                                    .and_then(|t| t.duration_since(SystemTime::UNIX_EPOCH).ok())
                                    .map(|d| d.as_secs() as i64);
                                let path_string = backend_path.to_string_lossy().to_string();
                                let backend_type = backend_name.clone();
                                
                                // Determine active by comparing full path
                                let is_active = if let Some(active_path) = &active_path {
                                    #[cfg(windows)]
                                    {
                                        let a = active_path.replace('\\', "/").trim_end_matches('/').to_lowercase();
                                        let b = path_string.replace('\\', "/").trim_end_matches('/').to_lowercase();
                                        a == b
                                    }
                                    #[cfg(not(windows))]
                                    {
                                        let a = active_path.trim_end_matches('/');
                                        let b = path_string.trim_end_matches('/');
                                        a == b
                                    }
                                } else { false };
                                
                                out.push(LlamaCppInstalledVersion { 
                                    name: format!("{}-{}", version_name, backend_name), 
                                    path: path_string, 
                                    has_server, 
                                    created, 
                                    is_active,
                                    backend_type: Some(backend_type),
                                });
                            }
                        }
                    }
                    
                    // Also check for old flat structure (backward compatibility)
                    let server_name = if cfg!(windows) { "llama-server.exe" } else { "llama-server" };
                    if path.join(server_name).exists() {
                        let created = entry.metadata().ok()
                            .and_then(|m| m.created().ok())
                            .and_then(|t| t.duration_since(SystemTime::UNIX_EPOCH).ok())
                            .map(|d| d.as_secs() as i64);
                        let path_string = path.to_string_lossy().to_string();
                        let backend_type = detect_backend_type(&version_name);
                        
                        let is_active = if let Some(active_version) = &active_version {
                            active_version == &version_name
                        } else if let Some(active_path) = &active_path {
                            #[cfg(windows)]
                            {
                                let a = active_path.replace('\\', "/").trim_end_matches('/').to_lowercase();
                                let b = path_string.replace('\\', "/").trim_end_matches('/').to_lowercase();
                                a == b
                            }
                            #[cfg(not(windows))]
                            {
                                let a = active_path.trim_end_matches('/');
                                let b = path_string.trim_end_matches('/');
                                a == b
                            }
                        } else { false };
                        
                        out.push(LlamaCppInstalledVersion { 
                            name: version_name, 
                            path: path_string, 
                            has_server: true, 
                            created, 
                            is_active,
                            backend_type: Some(backend_type),
                        });
                    }
                }
            }
        }
    }
    
    // If there is exactly one installed version and none is active, set it active automatically
    let has_active = out.iter().any(|v| v.is_active);
    if out.len() == 1 && !has_active {
        if let Some(only) = out.get(0) {
            // Update config with this single version as active
            {
                let mut cfg = state.config.lock().await;
                cfg.active_executable_folder = Some(only.path.clone());
                cfg.active_executable_version = Some(only.name.clone());
            }
            // Best-effort save; if it fails, we still return the list
            if let Err(e) = save_settings(&state).await {
                eprintln!("Failed to save settings after auto-activating version: {}", e);
            }
            // Reflect activation in the returned list
            if let Some(first) = out.get_mut(0) {
                first.is_active = true;
            }
        }
    }
    Ok(out)
}

#[tauri::command]
async fn set_active_llamacpp_version(path: String, state: tauri::State<'_, AppState>) -> Result<(), String> {
    {
        let mut cfg = state.config.lock().await;
        // Save both path and derived version name
        let version_name = std::path::Path::new(&path)
            .file_name()
            .and_then(|s| s.to_str())
            .map(|s| s.to_string());
        cfg.active_executable_folder = Some(path);
        cfg.active_executable_version = version_name;
    }
    save_settings(&state).await.map_err(|e| format!("Failed to save settings: {}", e))
}

#[tauri::command]
async fn delete_llamacpp_version(path: String, state: tauri::State<'_, AppState>) -> Result<(), String> {
    use std::fs;
    use std::path::Path;

    let base_exec = {
        let cfg = state.config.lock().await;
        cfg.executable_folder.clone()
    };

    let versions_root = Path::new(&base_exec).join("versions");
    let path_buf = Path::new(&path).to_path_buf();
    
    // Ensure deletion target is under versions root
    if !path_buf.starts_with(&versions_root) {
        return Err("Cannot delete outside versions directory".into());
    }
    
    if path_buf.exists() {
        fs::remove_dir_all(&path_buf).map_err(|e| format!("Failed to delete version: {}", e))?;
        
        // If we deleted a backend folder, check if the parent version folder is now empty
        if let Some(parent) = path_buf.parent() {
            if parent != versions_root && parent.exists() {
                if let Ok(entries) = fs::read_dir(parent) {
                    if entries.count() == 0 {
                        // Parent version folder is empty, remove it too
                        let _ = fs::remove_dir(parent);
                    }
                }
            }
        }
    }
    
    // Clear active if it pointed here
    {
        let mut cfg = state.config.lock().await;
        if cfg.active_executable_folder.as_deref() == Some(&path) {
            cfg.active_executable_folder = None;
            cfg.active_executable_version = None;
        }
    }
    save_settings(&state).await.map_err(|e| format!("Failed to save settings: {}", e))
}

// ==================== HuggingFace Direct Link Download Commands ====================

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
    app_handle: tauri::AppHandle,
) -> Result<DownloadStartResult, String> {
    use downloader::{DownloadConfig, start_download};
    use std::path::Path;
    
    // Construct download URL
    let download_url = format!(
        "https://huggingface.co/{}/resolve/main/{}",
        model_id, filename
    );
    
    // Ensure destination directory exists
    let dest_path = Path::new(&destination);
    if let Some(parent) = dest_path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create directory: {}", e))?;
    }
    
    // Build download configuration
    let config = DownloadConfig {
        base_url: download_url.clone(),
        destination_folder: destination.clone(),
        auto_extract: false,
        create_subfolder: None,
        files: vec![filename.clone()],
        custom_headers: None,
    };
    
    // Use existing download infrastructure
    match start_download(config, &state, app_handle).await {
        Ok(result) => Ok(result),
        Err(e) => Err(format!("Failed to start download: {}", e))
    }
}

// Initialize and load settings
async fn initialize_app_state(app_data_dir: std::path::PathBuf) -> Result<AppState, Box<dyn std::error::Error>> {
    let state = AppState::new();
    
    // Initialize tracker manager
    {
        let tracker_dir = app_data_dir.join("tracker");
        match TrackerManager::new(tracker_dir) {
            Ok(manager) => {
                let mut tracker = state.tracker_manager.lock().await;
                *tracker = Some(manager);
                println!("Tracker manager initialized successfully");
            }
            Err(e) => {
                eprintln!("Failed to initialize tracker manager: {}", e);
            }
        }
    }
    
    load_settings(&state).await?;

    // Create models and executable directories if they don't exist
    {
        let config = state.config.lock().await;
        let models_dir = &config.models_directory;
        let exec_dir = &config.executable_folder;

        // Create primary models directory
        if !models_dir.is_empty() {
            if let Err(e) = std::fs::create_dir_all(models_dir) {
                eprintln!("Failed to create models directory: {}", e);
            }
        }

        // Create additional models directories
        for additional_dir in &config.additional_models_directories {
            if !additional_dir.is_empty() {
                if let Err(e) = std::fs::create_dir_all(additional_dir) {
                    eprintln!("Failed to create additional models directory '{}': {}", additional_dir, e);
                }
            }
        }

        if !exec_dir.is_empty() {
            if let Err(e) = std::fs::create_dir_all(exec_dir) {
                eprintln!("Failed to create executable directory: {}", e);
            }
            // also create versions directory
            let versions_dir = std::path::Path::new(exec_dir).join("versions");
            if let Err(e) = std::fs::create_dir_all(&versions_dir) {
                eprintln!("Failed to create versions directory: {}", e);
            }
        }
    }
    
    // Cleanup leftover download files from previous sessions
    {
        let config = state.config.lock().await;
        let mut all_directories = vec![config.models_directory.clone()];
        all_directories.extend(config.additional_models_directories.clone());
        
        for dir in &all_directories {
            if !dir.is_empty() {
                if let Err(e) = huggingface::cleanup_leftover_downloads(dir).await {
                    eprintln!("Warning: Failed to cleanup leftover downloads in '{}': {}", dir, e);
                }
            }
        }
    }
    
    Ok(state)
}

#[tauri::command]
async fn get_tracker_models(
    vram_limit: Option<f64>,
    categories: Option<Vec<String>>,
    chinese_only: bool,
    gguf_only: bool,
    file_types: Option<Vec<String>>,
    quantizations: Option<Vec<String>>,
    search: Option<String>,
    sort_by: Option<String>,
    sort_desc: Option<bool>,
    state: tauri::State<'_, AppState>,
) -> Result<Vec<TrackerModel>, String> {
    let tracker = state.tracker_manager.lock().await;
    let manager = tracker.as_ref().ok_or("Tracker not initialized")?;
    
    manager.get_models(
        vram_limit,
        categories,
        chinese_only,
        gguf_only,
        file_types,
        quantizations,
        search,
        &sort_by.unwrap_or_else(|| "downloads".to_string()),
        sort_desc.unwrap_or(true),
    )
}

#[tauri::command]
async fn refresh_tracker_data(
    state: tauri::State<'_, AppState>,
    _app_handle: tauri::AppHandle,
) -> Result<TrackerStats, String> {
    let scraper = TrackerScraper::new();
    
    let models = scraper.fetch_trending_models(100).await?;
    
    let tracker = state.tracker_manager.lock().await;
    let manager = tracker.as_ref().ok_or("Tracker not initialized")?;
    
    // Clear existing models before saving new ones to ensure counts are accurate
    manager.clear_models()?;
    manager.save_models(&models)?;
    
    manager.get_stats()
}

#[tauri::command]
async fn export_tracker_json(
    state: tauri::State<'_, AppState>,
) -> Result<String, String> {
    let tracker = state.tracker_manager.lock().await;
    let manager = tracker.as_ref().ok_or("Tracker not initialized")?;
    
    manager.export_json()
}

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

#[tauri::command]
async fn get_tracker_stats(
    state: tauri::State<'_, AppState>,
) -> Result<TrackerStats, String> {
    let tracker = state.tracker_manager.lock().await;
    let manager = tracker.as_ref().ok_or("Tracker not initialized")?;
    
    manager.get_stats()
}

#[tauri::command]
async fn get_tracker_config(
    state: tauri::State<'_, AppState>,
) -> Result<TrackerConfig, String> {
    let tracker = state.tracker_manager.lock().await;
    let manager = tracker.as_ref().ok_or("Tracker not initialized")?;
    
    manager.get_config()
}

#[tauri::command]
async fn update_tracker_config(
    config: TrackerConfig,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let tracker = state.tracker_manager.lock().await;
    let manager = tracker.as_ref().ok_or("Tracker not initialized")?;
    
    manager.save_config(&config)
}

#[tauri::command]
async fn get_weekly_reports(
    state: tauri::State<'_, AppState>,
) -> Result<Vec<WeeklyReport>, String> {
    let tracker = state.tracker_manager.lock().await;
    let manager = tracker.as_ref().ok_or("Tracker not initialized")?;
    
    manager.get_weekly_reports(4)
}

#[tauri::command]
async fn generate_weekly_report(
    state: tauri::State<'_, AppState>,
) -> Result<WeeklyReport, String> {
    let tracker = state.tracker_manager.lock().await;
    let manager = tracker.as_ref().ok_or("Tracker not initialized")?;
    
    manager.generate_weekly_report()
}

#[tauri::command]
async fn save_network_config(
    address: String,
    port: u16,
    proxy_port: u16,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let mut config = state.config.lock().await;
    config.network_server_host = address.clone();
    config.network_server_port = port;
    config.openai_proxy_port = proxy_port;
    
    drop(config);
    
    if let Err(e) = save_settings(&state).await {
        return Err(format!("Failed to save config: {}", e));
    }
    
    Ok(())
}

#[tauri::command]
async fn get_network_config(
    state: tauri::State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    let config = state.config.lock().await;
    
    Ok(serde_json::json!({
        "address": config.network_server_host,
        "port": config.network_server_port,
        "proxy_port": config.openai_proxy_port,
        "enabled": config.openai_proxy_enabled,
    }))
}

#[tauri::command]
async fn get_network_interfaces() -> Result<serde_json::Value, String> {
    use std::net::Ipv4Addr;
    
    let mut interfaces = vec![
        serde_json::json!({
            "address": "127.0.0.1",
            "name": "Localhost",
            "type": "loopback"
        }),
        serde_json::json!({
            "address": "0.0.0.0",
            "name": "All Interfaces",
            "type": "all"
        })
    ];
    
    // Try to get local IP addresses using socket connection trick
    match tokio::task::spawn_blocking(|| {
        let socket = std::net::UdpSocket::bind("0.0.0.0:0");
        if let Ok(socket) = socket {
            // Connect to a public DNS server to determine our outbound interface
            if socket.connect("8.8.8.8:80").is_ok() {
                if let Ok(local_addr) = socket.local_addr() {
                    if let Some(ip) = local_addr.ip().to_string().parse::<Ipv4Addr>().ok() {
                        // Don't add loopback again
                        if !ip.is_loopback() {
                            return Some(ip.to_string());
                        }
                    }
                }
            }
        }
        None
    }).await {
        Ok(Some(ip)) => {
            interfaces.push(serde_json::json!({
                "address": ip,
                "name": "Primary Network Interface",
                "type": "primary"
            }));
        }
        _ => {}
    }
    
    // The primary interface detection above should catch most cases
    // For more comprehensive interface listing, we'd need the if-addrs crate
    
    Ok(serde_json::json!({
        "interfaces": interfaces
    }))
}

#[tauri::command]
async fn activate_network_server(
    address: String,
    port: u16,
    state: tauri::State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    let proxy_port = {
        let config = state.config.lock().await;
        config.openai_proxy_port
    };
    
    let mut proxy = state.openai_proxy.lock().await;
    if proxy.is_some() {
        return Err("Network server already active".to_string());
    }
    
    let mut new_proxy = openai_proxy::ProxyServer::new(
        address.clone(),
        port,
        proxy_port,
    );
    
    match new_proxy.start().await {
        Ok(_) => {
            *proxy = Some(new_proxy);
            
            let mut config = state.config.lock().await;
            config.openai_proxy_enabled = true;
            config.network_server_host = address.clone();
            config.network_server_port = port;
            drop(config);
            let _ = save_settings(&state).await;
            
            Ok(serde_json::json!({
                "success": true,
                "address": address,
                "port": port,
                "proxy_port": proxy_port,
                "message": format!("OpenAI proxy server activated on port {}", proxy_port)
            }))
        }
        Err(e) => Err(format!("Failed to start proxy: {}", e)),
    }
}

#[tauri::command]
async fn deactivate_network_server(
    state: tauri::State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    let mut proxy = state.openai_proxy.lock().await;
    
    if let Some(ref mut p) = *proxy {
        p.stop().await;
        *proxy = None;
        
        let mut config = state.config.lock().await;
        config.openai_proxy_enabled = false;
        drop(config);
        let _ = save_settings(&state).await;
        
        Ok(serde_json::json!({
            "success": true,
            "message": "Network server deactivated"
        }))
    } else {
        Ok(serde_json::json!({
            "success": true,
            "message": "Network server was not active"
        }))
    }
}

#[tauri::command]
async fn get_network_server_status(
    state: tauri::State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    let proxy = state.openai_proxy.lock().await;
    let config = state.config.lock().await;
    
    Ok(serde_json::json!({
        "active": proxy.is_some(),
        "config": {
            "address": config.network_server_host,
            "port": config.network_server_port,
            "proxy_port": config.openai_proxy_port,
        }
    }))
}

#[tauri::command]
async fn get_mcp_connections(
    state: tauri::State<'_, AppState>,
) -> Result<Vec<McpServerConfig>, String> {
    let config = state.config.lock().await;
    Ok(config.mcp_servers.clone())
}

#[tauri::command]
async fn save_mcp_connection(
    mut connection: McpServerConfig,
    state: tauri::State<'_, AppState>,
) -> Result<McpServerConfig, String> {
    validate_mcp_connection_payload(&connection)?;

    if connection.timeout_seconds == 0 {
        connection.timeout_seconds = 10;
    }

    if connection.id.trim().is_empty() {
        connection.id = format!("mcp-{}", Utc::now().timestamp_micros());
    }

    if connection.last_test_status.is_none() {
        connection.last_test_status = Some("never_tested".to_string());
    }

    let mut config = state.config.lock().await;

    let position = config.mcp_servers.iter().position(|item| item.id == connection.id);

    match position {
        Some(index) => {
            config.mcp_servers[index] = connection.clone();
        }
        None => {
            config.mcp_servers.push(connection.clone());
        }
    }

    drop(config);

    if let Err(e) = save_settings(&state).await {
        return Err(format!("Failed to save MCP connection: {}", e));
    }

    Ok(connection)
}

fn validate_mcp_connection_payload(connection: &McpServerConfig) -> Result<(), String> {
    if connection.name.trim().is_empty() {
        return Err("MCP connection name is required".to_string());
    }

    if connection.timeout_seconds == 0 {
        return Err("Timeout must be a positive number".to_string());
    }

    match connection.transport {
        McpTransport::Stdio => {
            if connection.command.trim().is_empty() {
                return Err("Command is required for stdio transport".to_string());
            }
        }
        McpTransport::Json => {}
        _ => {
            let trimmed = connection.url.trim();
            if trimmed.is_empty() {
                return Err("URL is required for this transport".to_string());
            }

            let parsed = reqwest::Url::parse(trimmed)
                .map_err(|_| "URL must be a valid absolute URL".to_string())?;

            if parsed.scheme() != "http" && parsed.scheme() != "https" {
                return Err("URL must use http or https scheme".to_string());
            }
        }
    }

    Ok(())
}

fn default_mcp_initialize_payload() -> serde_json::Value {
    serde_json::json!({
        "jsonrpc": "2.0",
        "id": "arandu-test",
        "method": "initialize",
        "params": {
            "protocolVersion": "2024-11-05",
            "capabilities": {
                "roots": { "listChanged": true }
            },
            "clientInfo": {
                "name": "arandu",
                "version": APP_VERSION
            }
        }
    })
}

fn mcp_test_payload(connection: &McpServerConfig) -> Result<serde_json::Value, String> {
    if connection.transport != McpTransport::Json {
        return Ok(default_mcp_initialize_payload());
    }

    if connection.json_payload.trim().is_empty() {
        return Ok(default_mcp_initialize_payload());
    }

    serde_json::from_str(connection.json_payload.trim())
        .map_err(|_| "JSON payload must be valid JSON".to_string())
}

fn resolve_mcp_url(connection: &McpServerConfig) -> Option<String> {
    let direct = connection.url.trim();
    if !direct.is_empty() {
        return Some(direct.to_string());
    }

    if connection.transport != McpTransport::Json {
        return None;
    }

    let payload = serde_json::from_str::<serde_json::Value>(connection.json_payload.trim()).ok()?;

    for key in ["url", "endpoint", "serverUrl", "server_url"] {
        if let Some(value) = payload.get(key).and_then(|v| v.as_str()) {
            let trimmed = value.trim();
            if !trimmed.is_empty() {
                return Some(trimmed.to_string());
            }
        }
    }

    if let Some(server_obj) = payload.get("server") {
        for key in ["url", "endpoint"] {
            if let Some(value) = server_obj.get(key).and_then(|v| v.as_str()) {
                let trimmed = value.trim();
                if !trimmed.is_empty() {
                    return Some(trimmed.to_string());
                }
            }
        }
    }

    if let Some(mcp_servers) = payload.get("mcpServers").and_then(|v| v.as_object()) {
        for server in mcp_servers.values() {
            for key in ["url", "endpoint"] {
                if let Some(value) = server.get(key).and_then(|v| v.as_str()) {
                    let trimmed = value.trim();
                    if !trimmed.is_empty() {
                        return Some(trimmed.to_string());
                    }
                }
            }
        }
    }

    None
}

fn parse_mcp_tools_from_response(response: &serde_json::Value) -> Result<Vec<McpToolInfo>, String> {
    if let Some(error) = response.get("error") {
        let details = if let Some(message) = error.get("message").and_then(|v| v.as_str()) {
            message.to_string()
        } else {
            error.to_string()
        };
        return Err(format!("MCP tool discovery failed: {}", details));
    }

    let tools = response
        .get("result")
        .and_then(|result| result.get("tools"))
        .and_then(|tools| tools.as_array())
        .ok_or_else(|| "No tools list found in MCP response".to_string())?;

    let parsed = tools
        .iter()
        .filter_map(|tool| {
            let name = tool.get("name")?.as_str()?.trim();
            if name.is_empty() {
                return None;
            }

            Some(McpToolInfo {
                name: name.to_string(),
                description: tool
                    .get("description")
                    .and_then(|value| value.as_str())
                    .map(std::string::ToString::to_string),
                input_schema: tool
                    .get("inputSchema")
                    .or_else(|| tool.get("input_schema"))
                    .cloned(),
                output_schema: tool
                    .get("outputSchema")
                    .or_else(|| tool.get("output_schema"))
                    .cloned(),
            })
        })
        .collect::<Vec<_>>();

    Ok(parsed)
}

fn mcp_accept_header(transport: &McpTransport) -> &'static str {
    match transport {
        McpTransport::Json => "application/json",
        _ => "application/json, text/event-stream",
    }
}

async fn post_mcp_request(
    client: &reqwest::Client,
    transport: &McpTransport,
    url: &str,
    payload: serde_json::Value,
    timeout_duration: Duration,
) -> Result<reqwest::Response, String> {
    let request = client
        .post(url)
        .header("accept", mcp_accept_header(transport))
        .json(&payload);

    timeout(timeout_duration, request.send())
        .await
        .map_err(|_| "MCP request timed out".to_string())?
        .map_err(|err| err.to_string())
}

async fn run_mcp_tool_discovery(
    transport: McpTransport,
    url: String,
    timeout_duration: Duration,
) -> McpToolsResult {
    let start_time = Instant::now();
    let client = reqwest::Client::new();

    let initialize_payload = default_mcp_initialize_payload();
    let tools_payload = serde_json::json!({
        "jsonrpc": "2.0",
        "id": "arandu-tools-list",
        "method": "tools/list",
        "params": {}
    });

    let initialize_response = match post_mcp_request(&client, &transport, &url, initialize_payload, timeout_duration).await {
        Ok(resp) => resp,
        Err(error) => {
            return McpToolsResult {
                success: false,
                latency_ms: start_time.elapsed().as_millis() as i64,
                message: "Initialize request failed".to_string(),
                tool_count: 0,
                tools: Vec::new(),
                status_code: None,
                error: Some(error),
            };
        }
    };

    match initialize_response.status().is_success() {
        false => {
            return McpToolsResult {
                success: false,
                latency_ms: start_time.elapsed().as_millis() as i64,
                message: format!(
                    "Initialize request returned HTTP {}",
                    initialize_response.status()
                ),
                tool_count: 0,
                tools: Vec::new(),
                status_code: Some(initialize_response.status().as_u16()),
                error: Some("initialize_failed".to_string()),
            };
        }
        true => {}
    }

    let tools_response = match post_mcp_request(&client, &transport, &url, tools_payload, timeout_duration).await {
        Ok(resp) => resp,
        Err(error) => {
            return McpToolsResult {
                success: false,
                latency_ms: start_time.elapsed().as_millis() as i64,
                message: "Tools list request failed".to_string(),
                tool_count: 0,
                tools: Vec::new(),
                status_code: None,
                error: Some(error),
            };
        }
    };

    if !tools_response.status().is_success() {
        return McpToolsResult {
            success: false,
            latency_ms: start_time.elapsed().as_millis() as i64,
            message: format!("Tools list request returned HTTP {}", tools_response.status()),
            tool_count: 0,
            tools: Vec::new(),
            status_code: Some(tools_response.status().as_u16()),
            error: Some("tools_list_failed".to_string()),
        };
    }

    let tools_response_body = match tools_response
        .json::<serde_json::Value>()
        .await
    {
        Ok(body) => body,
        Err(error) => {
            return McpToolsResult {
                success: false,
                latency_ms: start_time.elapsed().as_millis() as i64,
                message: "Failed to parse tools response JSON".to_string(),
                tool_count: 0,
                tools: Vec::new(),
                status_code: None,
                error: Some(error.to_string()),
            };
        }
    };

    match parse_mcp_tools_from_response(&tools_response_body) {
        Ok(tools) => {
            let tool_count = tools.len();
            let message = if tool_count == 0 {
                "No tools returned by server".to_string()
            } else {
                format!("Found {} tool(s)", tool_count)
            };

            McpToolsResult {
                success: true,
                latency_ms: start_time.elapsed().as_millis() as i64,
                message,
                tool_count,
                tools,
                status_code: None,
                error: None,
            }
        }
        Err(error) => McpToolsResult {
            success: false,
            latency_ms: start_time.elapsed().as_millis() as i64,
            message: "Tools list parse failed".to_string(),
            tool_count: 0,
            tools: Vec::new(),
            status_code: None,
            error: Some(error),
        },
    }
}

#[tauri::command]
async fn delete_mcp_connection(
    id: String,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let mut config = state.config.lock().await;
    let original_len = config.mcp_servers.len();
    config.mcp_servers.retain(|item| item.id != id);

    if config.mcp_servers.len() == original_len {
        return Err("MCP connection not found".to_string());
    }

    drop(config);

    if let Err(e) = save_settings(&state).await {
        return Err(format!("Failed to save MCP connections: {}", e));
    }

    Ok(())
}

#[tauri::command]
async fn list_mcp_tools(
    id: String,
    state: tauri::State<'_, AppState>,
) -> Result<McpToolsResult, String> {
    let start_time = Instant::now();

    let connection = {
        let config = state.config.lock().await;
        config
            .mcp_servers
            .iter()
            .find(|item| item.id == id)
            .cloned()
            .ok_or_else(|| "MCP connection not found".to_string())?
    };

    if !connection.enabled {
        return Ok(McpToolsResult {
            success: false,
            latency_ms: start_time.elapsed().as_millis() as i64,
            message: "Connection is disabled. Enable it before testing.".to_string(),
            tool_count: 0,
            tools: Vec::new(),
            status_code: None,
            error: Some("disabled".to_string()),
        });
    }

    if matches!(connection.transport, McpTransport::Stdio) {
        return Ok(McpToolsResult {
            success: false,
            latency_ms: start_time.elapsed().as_millis() as i64,
            message: "Tool discovery is not yet available for stdio transport in this phase".to_string(),
            tool_count: 0,
            tools: Vec::new(),
            status_code: None,
            error: Some("unsupported_transport".to_string()),
        });
    }

    if matches!(connection.transport, McpTransport::Sse) {
        return Ok(McpToolsResult {
            success: false,
            latency_ms: start_time.elapsed().as_millis() as i64,
            message: "Tool discovery is not yet available for SSE transport in this phase".to_string(),
            tool_count: 0,
            tools: Vec::new(),
            status_code: None,
            error: Some("unsupported_transport".to_string()),
        });
    }

    let resolved_url = resolve_mcp_url(&connection);
    if resolved_url.is_none() {
        return Ok(McpToolsResult {
            success: false,
            latency_ms: start_time.elapsed().as_millis() as i64,
            message: "URL is required for tool discovery. For JSON transport, include a URL in the URL field or inside JSON payload (url/endpoint).".to_string(),
            tool_count: 0,
            tools: Vec::new(),
            status_code: None,
            error: Some("missing_url".to_string()),
        });
    }

    let url = resolved_url.unwrap_or_default();
    let timeout_duration = Duration::from_secs(connection.timeout_seconds.max(1));

    let result = run_mcp_tool_discovery(connection.transport.clone(), url, timeout_duration).await;

    let mut config = state.config.lock().await;
    if let Some(conn) = config.mcp_servers.iter_mut().find(|item| item.id == id) {
        conn.tools_last_refresh_at = Some(Utc::now().to_rfc3339());
        conn.tools_last_status = Some(if result.success { "ok".to_string() } else { "error".to_string() });
        conn.tools_last_message = Some(result.message.clone());
        conn.tools_last_error = result.error.clone();
        conn.tools = result.tools.clone();
    }

    drop(config);
    let _ = save_settings(&state).await;

    Ok(result)
}

#[tauri::command]
async fn toggle_mcp_connection(
    id: String,
    enabled: bool,
    state: tauri::State<'_, AppState>,
) -> Result<McpServerConfig, String> {
    let mut config = state.config.lock().await;

    let connection = config
        .mcp_servers
        .iter_mut()
        .find(|item| item.id == id)
        .ok_or_else(|| "MCP connection not found".to_string())?;

    connection.enabled = enabled;
    connection.last_test_status = Some(if enabled {
        "enabled".to_string()
    } else {
        "disabled".to_string()
    });
    connection.last_test_message = Some(if enabled {
        "Connection enabled".to_string()
    } else {
        "Connection disabled".to_string()
    });
    connection.last_test_at = Some(Utc::now().to_rfc3339());

    let connection = connection.clone();

    drop(config);

    if let Err(e) = save_settings(&state).await {
        return Err(format!("Failed to save MCP connection: {}", e));
    }

    Ok(connection)
}

#[tauri::command]
async fn test_mcp_connection(
    id: String,
    state: tauri::State<'_, AppState>,
) -> Result<McpTestResult, String> {
    let start_time = Instant::now();

    let connection = {
        let config = state.config.lock().await;
        config
            .mcp_servers
            .iter()
            .find(|item| item.id == id)
            .cloned()
            .ok_or_else(|| "MCP connection not found".to_string())?
    };

    if !connection.enabled {
        return Ok(McpTestResult {
            success: false,
            latency_ms: 0,
            message: "Connection is disabled. Enable it before testing.".to_string(),
            status_code: None,
            exit_code: None,
            error: Some("disabled".to_string()),
        });
    }

    let timeout_duration = Duration::from_secs(connection.timeout_seconds.max(1));

    let mut result = match connection.transport {
        McpTransport::Stdio => {
            match TokioCommand::new(&connection.command).args(&connection.args).spawn() {
                Ok(mut child) => match timeout(timeout_duration, child.wait()).await {
                    Ok(Ok(status)) => {
                        let code = status.code();
                        McpTestResult {
                            success: false,
                            latency_ms: start_time.elapsed().as_millis() as i64,
                            message: format!(
                                "Process exited before validation was complete: {:?}",
                                code
                            ),
                            status_code: None,
                            exit_code: code,
                            error: Some(format!("Process exited: {:?}", status)),
                        }
                    }
                    Ok(Err(err)) => McpTestResult {
                        success: false,
                        latency_ms: start_time.elapsed().as_millis() as i64,
                        message: "Failed to query process status".to_string(),
                        status_code: None,
                        exit_code: None,
                        error: Some(err.to_string()),
                    },
                    Err(_) => {
                        let _ = child.kill().await;
                        let _ = child.wait().await;
                        McpTestResult {
                            success: true,
                            latency_ms: start_time.elapsed().as_millis() as i64,
                            message: "Stdio transport started successfully".to_string(),
                        status_code: None,
                        exit_code: None,
                        error: None,
                    }
                    }
                },
                Err(err) => McpTestResult {
                    success: false,
                    latency_ms: start_time.elapsed().as_millis() as i64,
                    message: "Failed to start stdio process".to_string(),
                    status_code: None,
                    exit_code: None,
                    error: Some(err.to_string()),
                },
            }
        }
        _ => {
        let resolved_url = resolve_mcp_url(&connection);
        if resolved_url.is_none() {
            return Ok(McpTestResult {
                success: false,
                latency_ms: start_time.elapsed().as_millis() as i64,
                message: "URL is required for transport validation. For JSON transport, include a URL in the URL field or inside JSON payload (url/endpoint).".to_string(),
                status_code: None,
                exit_code: None,
                error: Some("missing_url".to_string()),
            });
        }

        let url = resolved_url.unwrap_or_default();
        let client = reqwest::Client::new();
        let transport = connection.transport.clone();
            let init_payload = match mcp_test_payload(&connection) {
                Ok(payload) => payload,
                Err(error) => {
                    return Ok(McpTestResult {
                        success: false,
                        latency_ms: start_time.elapsed().as_millis() as i64,
                        message: error.clone(),
                        status_code: None,
                        exit_code: None,
                        error: Some(error),
                    });
                }
            };

            let request = match transport {
                McpTransport::Http | McpTransport::Json | McpTransport::StreamableHttp => {
                    let mut request = client
                        .post(url)
                        .json(&init_payload);

                    if transport == McpTransport::Json {
                        request = request.header("accept", "application/json");
                    } else {
                        request = request.header("accept", "application/json, text/event-stream");
                    }

                    Some(request)
                }
                McpTransport::Sse => Some(client.get(url).header("accept", "text/event-stream")),
                _ => None,
            };

            match request {
                Some(req) => match timeout(timeout_duration, req.send()).await {
                Ok(Ok(resp)) => {
                    let code = resp.status().as_u16();
                    let success = resp.status().is_success();
                    McpTestResult {
                        success,
                        latency_ms: start_time.elapsed().as_millis() as i64,
                        message: if success {
                            "HTTP transport endpoint is reachable".to_string()
                        } else {
                            format!("HTTP transport returned status {}", code)
                        },
                    status_code: Some(code),
                    exit_code: None,
                    error: None,
                }
                }
                Ok(Err(err)) => McpTestResult {
                    success: false,
                    latency_ms: start_time.elapsed().as_millis() as i64,
                    message: "Request failed".to_string(),
                    status_code: None,
                    exit_code: None,
                    error: Some(err.to_string()),
                },
                Err(_) => McpTestResult {
                    success: false,
                    latency_ms: start_time.elapsed().as_millis() as i64,
                    message: "HTTP test timed out".to_string(),
                    status_code: None,
                    exit_code: None,
                    error: Some("timeout".to_string()),
                },
                },
                None => McpTestResult {
                    success: false,
                    latency_ms: start_time.elapsed().as_millis() as i64,
                    message: "Unsupported MCP transport for HTTP validation".to_string(),
                    status_code: None,
                    exit_code: None,
                    error: Some("unsupported_transport".to_string()),
                },
            }
        }
    };

    let mut config = state.config.lock().await;
    if let Some(conn) = config.mcp_servers.iter_mut().find(|item| item.id == id) {
        conn.last_test_at = Some(Utc::now().to_rfc3339());
        conn.last_test_status = Some(if result.success {
            "ok".to_string()
        } else {
            "error".to_string()
        });
        conn.last_test_message = Some(result.message.clone());
    }

    drop(config);
    let _ = save_settings(&state).await;

    if !result.success {
        if result.message.is_empty() {
            result.message = result.error.clone().unwrap_or_else(|| "Test failed".to_string());
        }
    }

    Ok(result)
}

#[tauri::command]
async fn correct_mcp_json_with_active_model(
    json_input: String,
    state: tauri::State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    let trimmed = json_input.trim();
    if trimmed.is_empty() {
        return Err("JSON input is required".to_string());
    }

    let running = state.running_processes.lock().await;
    let active = running
        .values()
        .find(|proc| matches!(proc.status, models::ProcessStatus::Running))
        .or_else(|| running.values().next())
        .cloned()
        .ok_or_else(|| "No running model found. Start a model and try again.".to_string())?;
    drop(running);

    let server_url = format!("http://{}:{}", active.host, active.port);
    let client = reqwest::Client::new();

    let prompt = format!(
        "You are a strict JSON syntax fixer. Correct syntax only and return valid minified JSON with no explanation or markdown. Input:\n{}",
        trimmed
    );

    let response = client
        .post(format!("{}/v1/chat/completions", server_url))
        .json(&serde_json::json!({
            "messages": [
                {"role": "system", "content": "Return only valid JSON. No prose."},
                {"role": "user", "content": prompt}
            ],
            "stream": false
        }))
        .send()
        .await
        .map_err(|e| format!("Failed to contact model: {}", e))?;

    if !response.status().is_success() {
        return Err(format!(
            "Model request failed with status {}",
            response.status().as_u16()
        ));
    }

    let body: serde_json::Value = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse model response: {}", e))?;

    let corrected = body
        .get("choices")
        .and_then(|c| c.get(0))
        .and_then(|c| c.get("message"))
        .and_then(|m| m.get("content"))
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .ok_or_else(|| "Model returned empty content".to_string())?;

    let normalized: serde_json::Value = serde_json::from_str(corrected)
        .map_err(|e| format!("Model output is not valid JSON: {}", e))?;

    let corrected_json = serde_json::to_string_pretty(&normalized)
        .map_err(|e| format!("Failed to format corrected JSON: {}", e))?;

    Ok(serde_json::json!({
        "corrected_json": corrected_json,
        "model": active.model_name,
        "server": server_url
    }))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Initialize logging
    tracing_subscriber::fmt::init();
    
tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            // When a second instance is launched, show/focus the existing window
            let _ = app.get_webview_window("main")
                .expect("no main window")
                .set_focus();
        }))
        .setup(|app| {
            // Initialize app state
            let rt = tokio::runtime::Runtime::new().unwrap();
            let app_data_dir = app.path().app_data_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
            let state = rt.block_on(initialize_app_state(app_data_dir))
                .map_err(|e| format!("Failed to initialize app state: {}", e))?;
            
            println!("Application started, process tracking enabled with kill_on_drop");
            
            // Build the tray icon with menu
            let restore = MenuItemBuilder::with_id("restore", "Restore").build(app)?;
            let quit = MenuItemBuilder::with_id("quit", "Quit").build(app)?;
            
            let tray_menu = Menu::with_items(app, &[&restore, &quit])?;
            
            let _tray = TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .tooltip("Arandu - Click to restore")
                .menu(&tray_menu)
                .show_menu_on_left_click(false)
                .on_menu_event(move |app, event| {
                    match event.id.as_ref() {
                        "restore" => {
                            if let Some(win) = app.get_webview_window("main") {
                                win.show().ok();
                                win.set_focus().ok();
                            }
                        }
                        "quit" => {
                            // Perform cleanup and exit
                            let state = app.state::<AppState>();
                            let rt = tokio::runtime::Runtime::new().unwrap();
                            rt.block_on(async {
                                state.cleanup_all_processes().await;
                            });
                            app.exit(0);
                        }
                        _ => {}
                    }
                })
                .on_tray_icon_event(|tray, event| {
                    use tauri::tray::{TrayIconEvent, MouseButton, MouseButtonState};
                    
                    match event {
                        TrayIconEvent::Click { button, button_state, .. } => {
                            if button == MouseButton::Left && button_state == MouseButtonState::Up {
                                if let Some(window) = tray.app_handle().get_webview_window("main") {
                                    let is_visible = window.is_visible().unwrap_or(false);
                                    let is_minimized = window.is_minimized().unwrap_or(false);

                                    if is_visible && !is_minimized {
                                        let _ = window.hide();
                                    } else {
                                        if is_minimized {
                                            let _ = window.unminimize();
                                        }
                                        let _ = window.show();
                                        let _ = window.set_focus();
                                    }
                                }
                            }
                        }
                        TrayIconEvent::DoubleClick { button, .. } => {
                            if button == MouseButton::Left {
                                if let Some(window) = tray.app_handle().get_webview_window("main") {
                                    if window.is_minimized().unwrap_or(false) {
                                        let _ = window.unminimize();
                                    }
                                    let _ = window.show();
                                    let _ = window.set_focus();
                                }
                            }
                        }
                        _ => {}
                    }
                })
                .build(app)?;
            
            // Handle main window close event - hide to tray instead
            let app_handle = app.handle().clone();
            if let Some(main_window) = app.get_webview_window("main") {
                let version = env!("CARGO_PKG_VERSION");
                let title = format!("Arandu v{}", version);
                main_window.set_title(&title).ok();
                
                main_window.on_window_event(move |event| {
                    match event {
                        tauri::WindowEvent::CloseRequested { api, .. } => {
                            println!("Main window close button clicked, hiding to tray...");
                            
                            // Prevent the window from closing and hide it instead
                            api.prevent_close();
                            
                            // Hide the window to tray
                            if let Some(win) = app_handle.get_webview_window("main") {
                                win.hide().ok();
                            }
                        }
                        _ => {}
                    }
                });
            }
            
            // Fallback cleanup on app before exit
            let state_for_exit = state.clone();
            app.listen("tauri://before-exit", move |_| {
                println!("Before exit event received");
                let state_clone = state_for_exit.clone();
                tokio::spawn(async move {
                    println!("Application before exit, emergency cleanup...");
                    state_clone.cleanup_all_processes().await;
                });
            });
            
            app.manage(state);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_config,
            save_config,
            scan_models_command,
            get_model_settings,
            update_model_settings,
            get_model_presets,
            save_model_preset,
            update_model_presets,
            delete_model_preset,
            set_default_preset,
            launch_model_with_preset,
            launch_model_with_half_context,
            launch_model,
            launch_model_external,
            launch_model_with_preset_external,
            delete_model_file,
            delete_model,
            kill_process,
            get_process_output,
            browse_folder,
            open_url,
            open_model_folder,
            search_huggingface,
            get_model_details,
            download_model,
            get_download_status,
            get_all_downloads,
            get_all_downloads_and_history,
            cancel_download,
            pause_download,
            resume_download,
            clear_download_history,
            download_from_url,
            get_llamacpp_releases,
            get_llamacpp_commit_info,
            download_llamacpp_asset,
            download_llamacpp_asset_to_version,
            list_llamacpp_versions,
            set_active_llamacpp_version,
            delete_llamacpp_version,
            get_session_state,
            save_window_state,
            remove_window_state,
            restart_application,
            graceful_exit,
            get_app_version,
            check_file_exists,
            get_system_stats,
            scan_mmproj_files_command,
            hide_window,
            show_window,
initial_scan_models,
            check_model_update,
            get_hf_model_files,
            link_model_to_hf,
            get_model_metadata,
            parse_hf_url,
            fetch_hf_model_info,
            fetch_hf_model_files,
            get_default_download_path,
            download_hf_file,
            get_tracker_models,
            refresh_tracker_data,
            export_tracker_json,
            get_tracker_live_results,
            get_tracker_stats,
            get_tracker_config,
            update_tracker_config,
get_weekly_reports,
            generate_weekly_report,
            save_network_config,
            get_network_config,
            get_network_interfaces,
            activate_network_server,
            deactivate_network_server,
            get_network_server_status,
            get_mcp_connections,
            save_mcp_connection,
            delete_mcp_connection,
            toggle_mcp_connection,
            test_mcp_connection,
            list_mcp_tools,
            correct_mcp_json_with_active_model,
            list_chat_logs,
            create_chat_log,
            append_chat_log_message,
            rename_chat_log,
            get_chat_log,
            search_chat_logs,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
    }

#[cfg(test)]
mod tests {
    use super::*;

    fn base_connection() -> McpServerConfig {
        McpServerConfig {
            id: "mcp-1".to_string(),
            name: "Test MCP".to_string(),
            enabled: true,
            transport: McpTransport::Stdio,
            url: String::new(),
            command: "python".to_string(),
            json_payload: String::new(),
            args: vec!["-m".to_string(), "server".to_string()],
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

    #[test]
    fn mcp_validation_rejects_empty_name() {
        let mut connection = base_connection();
        connection.name = "".to_string();

        let result = validate_mcp_connection_payload(&connection);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "MCP connection name is required");
    }

    #[test]
    fn mcp_validation_rejects_zero_timeout() {
        let mut connection = base_connection();
        connection.timeout_seconds = 0;

        let result = validate_mcp_connection_payload(&connection);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Timeout must be a positive number");
    }

    #[test]
    fn mcp_validation_rejects_http_without_url() {
        let mut connection = base_connection();
        connection.transport = McpTransport::Http;
        connection.url = "".to_string();

        let result = validate_mcp_connection_payload(&connection);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "URL is required for this transport");
    }

    #[test]
    fn mcp_validation_accepts_http_with_url() {
        let mut connection = base_connection();
        connection.transport = McpTransport::Http;
        connection.url = "http://127.0.0.1:8080/mcp".to_string();

        let result = validate_mcp_connection_payload(&connection);
        assert!(result.is_ok());
    }

    #[test]
    fn mcp_validation_accepts_json_without_url() {
        let mut connection = base_connection();
        connection.transport = McpTransport::Json;
        connection.url = "".to_string();

        let result = validate_mcp_connection_payload(&connection);
        assert!(result.is_ok());
    }

    #[test]
    fn mcp_validation_accepts_json_with_url() {
        let mut connection = base_connection();
        connection.transport = McpTransport::Json;
        connection.url = "http://127.0.0.1:8080/mcp".to_string();

        let result = validate_mcp_connection_payload(&connection);
        assert!(result.is_ok());
    }

    #[test]
    fn mcp_validation_rejects_non_http_scheme() {
        let mut connection = base_connection();
        connection.transport = McpTransport::StreamableHttp;
        connection.url = "ftp://127.0.0.1/mcp".to_string();

        let result = validate_mcp_connection_payload(&connection);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "URL must use http or https scheme");
    }

    #[test]
    fn mcp_validation_rejects_malformed_url() {
        let mut connection = base_connection();
        connection.transport = McpTransport::Http;
        connection.url = "http://[::1".to_string();

        let result = validate_mcp_connection_payload(&connection);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "URL must be a valid absolute URL");
    }

    #[test]
    fn mcp_validation_rejects_empty_command_for_stdio() {
        let mut connection = base_connection();
        connection.transport = McpTransport::Stdio;
        connection.command = "".to_string();

        let result = validate_mcp_connection_payload(&connection);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Command is required for stdio transport");
    }

    #[test]
    fn mcp_validation_accepts_streamable_http() {
        let mut connection = base_connection();
        connection.transport = McpTransport::StreamableHttp;
        connection.url = "https://127.0.0.1:8080/mcp".to_string();

        let result = validate_mcp_connection_payload(&connection);
        assert!(result.is_ok());
    }

    #[test]
    fn mcp_test_payload_returns_default_for_empty_json_payload() {
        let mut connection = base_connection();
        connection.transport = McpTransport::Json;
        connection.url = "https://127.0.0.1:8080/mcp".to_string();
        connection.json_payload = "".to_string();

        let payload = mcp_test_payload(&connection).expect("empty payload should fallback to default");
        assert_eq!(payload["method"], "initialize");
        assert_eq!(payload["jsonrpc"], "2.0");
    }

    #[test]
    fn mcp_test_payload_parses_custom_json_payload() {
        let mut connection = base_connection();
        connection.transport = McpTransport::Json;
        connection.url = "https://127.0.0.1:8080/mcp".to_string();
        connection.json_payload = r#"{"jsonrpc":"2.0","id":"custom","method":"ping"}"#.to_string();

        let payload = mcp_test_payload(&connection).expect("custom payload should parse");
        assert_eq!(payload["method"], "ping");
        assert_eq!(payload["id"], "custom");
    }

    #[test]
    fn mcp_test_payload_rejects_invalid_json_payload() {
        let mut connection = base_connection();
        connection.transport = McpTransport::Json;
        connection.url = "https://127.0.0.1:8080/mcp".to_string();
        connection.json_payload = "{ invalid_json".to_string();

        let result = mcp_test_payload(&connection);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "JSON payload must be valid JSON");
    }

    #[test]
    fn parse_mcp_tools_from_response_extracts_tools() {
        let response = serde_json::json!({
            "jsonrpc": "2.0",
            "id": "arandu-tools-list",
            "result": {
                "tools": [
                    {
                        "name": "list_models",
                        "description": "List available models",
                        "inputSchema": { "type": "object" }
                    },
                    {
                        "name": "load_model",
                        "outputSchema": { "type": "object" }
                    }
                ]
            }
        });

        let tools = parse_mcp_tools_from_response(&response).expect("tools should parse");

        assert_eq!(tools.len(), 2);
        assert_eq!(tools[0].name, "list_models");
        assert_eq!(tools[0].description.as_deref(), Some("List available models"));
        assert!(tools[0].input_schema.is_some());
        assert!(tools[1].output_schema.is_some());
    }

    #[test]
    fn parse_mcp_tools_from_response_rejects_mcp_error() {
        let response = serde_json::json!({
            "jsonrpc": "2.0",
            "id": "arandu-tools-list",
            "error": {
                "code": -32600,
                "message": "Invalid request"
            }
        });

        let result = parse_mcp_tools_from_response(&response);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid request"));
    }
}
