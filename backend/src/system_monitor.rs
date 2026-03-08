use serde::{Deserialize, Serialize};
use std::time::SystemTime;
use sysinfo::{System};
use std::path::Path;
use std::fs;
#[cfg(target_os = "windows")]
use std::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemStats {
    pub cpu_usage: f32,
    pub memory_total_gb: f32,
    pub memory_used_gb: f32,
    pub gpu_name: String,
    pub gpu_usage: f32,
    pub gpu_memory_total_gb: f32,
    pub gpu_memory_used_gb: f32,
    pub timestamp: u64,
    pub models_folder_size_gb: f32,
    pub models_count: u32,
}

#[tauri::command]
pub async fn get_system_stats(state: tauri::State<'_, crate::AppState>) -> Result<SystemStats, String> {
    let mut sys = System::new_all();
    sys.refresh_all();
    
    // CPU usage (average of all cores)
    let cpu_usage = sys.global_cpu_usage();
    
    // Memory information in GB
    let memory_total_gb = sys.total_memory() as f32 / (1024.0 * 1024.0 * 1024.0);
    let memory_used_gb = sys.used_memory() as f32 / (1024.0 * 1024.0 * 1024.0);
    
    // GPU information
    let (gpu_name, gpu_usage, gpu_memory_total_gb, gpu_memory_used_gb) = get_gpu_info();
    
    // Models folder statistics
    let (models_folder_size_gb, models_count) = get_models_stats(&state).await;
    
    let timestamp = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    
    Ok(SystemStats {
        cpu_usage,
        memory_total_gb,
        memory_used_gb,
        gpu_name,
        gpu_usage,
        gpu_memory_total_gb,
        gpu_memory_used_gb,
        timestamp,
        models_folder_size_gb,
        models_count,
    })
}

fn get_gpu_info() -> (String, f32, f32, f32) {
    if let Some(nvidia_info) = get_nvidia_gpu_info() {
        return nvidia_info;
    }

    #[cfg(target_os = "windows")]
    if let Some(windows_info) = get_windows_gpu_fallback_info() {
        return windows_info;
    }

    ("No GPU detected".to_string(), 0.0, 0.0, 0.0)
}

fn get_nvidia_gpu_info() -> Option<(String, f32, f32, f32)> {
    let nvml = nvml_wrapper::Nvml::init().ok()?;
    let count = nvml.device_count().ok()?;
    if count == 0 {
        return None;
    }

    let device = nvml.device_by_index(0).ok()?;
    let name = device.name().unwrap_or_else(|_| "NVIDIA GPU".to_string());

    let gpu_usage = device
        .utilization_rates()
        .map(|util| util.gpu as f32)
        .unwrap_or(0.0);

    let (gpu_memory_total_gb, gpu_memory_used_gb) = device
        .memory_info()
        .map(|mem_info| {
            let total = mem_info.total as f32 / (1024.0 * 1024.0 * 1024.0);
            let used = mem_info.used as f32 / (1024.0 * 1024.0 * 1024.0);
            (total, used)
        })
        .unwrap_or((0.0, 0.0));

    Some((name, gpu_usage, gpu_memory_total_gb, gpu_memory_used_gb))
}

#[cfg(target_os = "windows")]
fn get_windows_gpu_fallback_info() -> Option<(String, f32, f32, f32)> {
    let output = Command::new("powershell")
        .args([
            "-NoProfile",
            "-Command",
            "Get-CimInstance Win32_VideoController | Select-Object Name,AdapterRAM | ConvertTo-Json -Compress",
        ])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8(output.stdout).ok()?;
    let parsed: serde_json::Value = serde_json::from_str(stdout.trim()).ok()?;

    let mut best_name = String::new();
    let mut best_vram_bytes: u64 = 0;

    let entries: Vec<&serde_json::Value> = match &parsed {
        serde_json::Value::Array(items) => items.iter().collect(),
        _ => vec![&parsed],
    };

    for entry in entries {
        let name = entry
            .get("Name")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .trim();

        if name.is_empty() {
            continue;
        }

        let lowered = name.to_lowercase();
        if lowered.contains("microsoft basic") || lowered.contains("remote display") {
            continue;
        }

        let adapter_ram_bytes = entry
            .get("AdapterRAM")
            .and_then(|v| v.as_u64())
            .or_else(|| {
                entry
                    .get("AdapterRAM")
                    .and_then(|v| v.as_i64())
                    .and_then(|n| u64::try_from(n).ok())
            })
            .unwrap_or(0);

        if best_name.is_empty() || adapter_ram_bytes > best_vram_bytes {
            best_name = name.to_string();
            best_vram_bytes = adapter_ram_bytes;
        }
    }

    if best_name.is_empty() {
        return None;
    }

    let total_gb = best_vram_bytes as f32 / (1024.0 * 1024.0 * 1024.0);
    Some((best_name, 0.0, total_gb.max(0.0), 0.0))
}

async fn get_models_stats(state: &crate::AppState) -> (f32, u32) {
    // Get models directory from config
    let models_dir = {
        let config = state.config.lock().await;
        config.models_directory.clone()
    };
    
    // Check if directory exists
    if models_dir.is_empty() || !Path::new(&models_dir).is_dir() {
        return (0.0, 0);
    }
    
    // Calculate total size and count .gguf files
    let (total_size, model_count) = calculate_directory_stats(&models_dir);
    
    // Convert bytes to GB
    let size_gb = total_size as f32 / (1024.0 * 1024.0 * 1024.0);
    
    (size_gb, model_count)
}

fn calculate_directory_stats(dir: &str) -> (u64, u32) {
    let mut total_size = 0u64;
    let mut model_count = 0u32;
    
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            
            if path.is_file() {
                // Count .gguf files
                if let Some(ext) = path.extension() {
                    if ext == "gguf" {
                        model_count += 1;
                    }
                }
                
                // Add file size
                if let Ok(metadata) = entry.metadata() {
                    total_size += metadata.len();
                }
            } else if path.is_dir() {
                // Recursively process subdirectories
                let (sub_size, sub_count) = calculate_directory_stats(path.to_str().unwrap_or(""));
                total_size += sub_size;
                model_count += sub_count;
            }
        }
    }
    
    (total_size, model_count)
}
