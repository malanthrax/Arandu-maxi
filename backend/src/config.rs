use std::path::{Path, PathBuf};
use std::collections::HashMap;
use serde_json;
use tokio::fs;
use crate::models::*;
use crate::AppState;

const SETTINGS_FILE: &str = "settings.json";

pub async fn get_settings_path() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let mut path = dirs::home_dir()
        .ok_or("Could not find home directory")?;
    path.push(".Arandu");
    
    // Create directory if it doesn't exist
    fs::create_dir_all(&path).await?;
    path.push(SETTINGS_FILE);
    
    Ok(path)
}

/// Convert absolute model path to relative path based on models directory
fn make_path_relative(absolute_path: &str, models_dir: &str) -> String {
    if models_dir.is_empty() {
        return absolute_path.to_string();
    }
    
    let abs_path = Path::new(absolute_path);
    let base_path = Path::new(models_dir);
    
    if let Ok(rel_path) = abs_path.strip_prefix(base_path) {
        rel_path.to_string_lossy().to_string()
    } else {
        absolute_path.to_string()
    }
}

/// Convert relative model path to absolute path based on models directory
fn make_path_absolute(relative_path: &str, models_dir: &str) -> String {
    if models_dir.is_empty() || Path::new(relative_path).is_absolute() {
        return relative_path.to_string();
    }
    
    let base_path = Path::new(models_dir);
    let full_path = base_path.join(relative_path);
    full_path.to_string_lossy().to_string()
}

fn remap_arandu_path(path: &str, old_base: &Path, new_base: &Path) -> Option<String> {
    let path_obj = Path::new(path);
    if path_obj.starts_with(old_base) {
        if let Ok(relative) = path_obj.strip_prefix(old_base) {
            let remapped = new_base.join(relative).to_string_lossy().to_string();
            if remapped != path {
                return Some(remapped);
            }
        }
    }

    let lower = path.to_ascii_lowercase();
    let marker = r"\.arandu\";
    if lower.starts_with("c:\\") {
        if let Some(index) = lower.find(marker) {
            let suffix_start = index + marker.len();
            let suffix = &path[suffix_start..];
            let remapped = if suffix.is_empty() {
                new_base.to_path_buf()
            } else {
                new_base.join(suffix)
            }
            .to_string_lossy()
            .to_string();

            if remapped != path {
                return Some(remapped);
            }
        }
    }

    None
}

fn migrate_global_config_paths(config: &mut GlobalConfig) -> bool {
    let old_base = dirs::home_dir().unwrap_or_default().join(".Arandu");
    let new_base = preferred_arandu_base_dir();
    let mut changed = false;

    if let Some(remapped) = remap_arandu_path(&config.models_directory, &old_base, &new_base) {
        config.models_directory = remapped;
        changed = true;
    }

    if let Some(remapped) = remap_arandu_path(&config.executable_folder, &old_base, &new_base) {
        config.executable_folder = remapped;
        changed = true;
    }

    for directory in &mut config.additional_models_directories {
        if let Some(remapped) = remap_arandu_path(directory, &old_base, &new_base) {
            *directory = remapped;
            changed = true;
        }
    }

    if let Some(active_folder) = config.active_executable_folder.clone() {
        if let Some(remapped) = remap_arandu_path(&active_folder, &old_base, &new_base) {
            config.active_executable_folder = Some(remapped);
            changed = true;
        }
    }

    changed
}

#[derive(serde::Serialize, serde::Deserialize)]
struct SettingsFile {
    global_config: GlobalConfig,
    model_configs: HashMap<String, ModelConfig>,
}

pub async fn load_settings(state: &AppState) -> Result<(), Box<dyn std::error::Error>> {
    let settings_path = get_settings_path().await?;
    
    if !settings_path.exists() {
        tracing::info!("Settings file does not exist, using defaults");
        return Ok(());
    }
    
    let contents = fs::read_to_string(&settings_path).await?;
    let mut settings: SettingsFile = serde_json::from_str(&contents)?;

    let migrated = migrate_global_config_paths(&mut settings.global_config);
    if migrated {
        let updated_contents = serde_json::to_string_pretty(&settings)?;
        fs::write(&settings_path, updated_contents).await?;
        tracing::info!(
            "Migrated legacy .Arandu paths to preferred base in {:?}",
            settings_path
        );
    }

    let SettingsFile {
        global_config,
        model_configs: stored_model_configs,
    } = settings;

    // Get models directory for path conversion
    let models_dir = global_config.models_directory.clone();
    
    // Update global config
    {
        let mut config = state.config.lock().await;
        *config = global_config;
    }
    
    // Update model configs, converting relative paths to absolute
    {
        let mut model_configs = state.model_configs.lock().await;
        let mut absolute_configs = HashMap::new();
        
        for (relative_path, mut config) in stored_model_configs {
            let absolute_path = make_path_absolute(&relative_path, &models_dir);
            config.model_path = absolute_path.clone();
            absolute_configs.insert(absolute_path, config);
        }
        
        *model_configs = absolute_configs;
    }
    
    tracing::info!("Settings loaded successfully from {:?}", settings_path);
    Ok(())
}

pub async fn save_settings(state: &AppState) -> Result<(), Box<dyn std::error::Error>> {
    let settings_path = get_settings_path().await?;
    
    let global_config = {
        let config = state.config.lock().await;
        config.clone()
    };
    
    let models_dir = global_config.models_directory.clone();
    
    let model_configs = {
        let configs = state.model_configs.lock().await;
        let mut relative_configs = HashMap::new();
        
        // Convert absolute paths to relative paths for storage
        for (absolute_path, config) in configs.iter() {
            let relative_path = make_path_relative(absolute_path, &models_dir);
            let mut config_clone = config.clone();
            config_clone.model_path = relative_path.clone();
            relative_configs.insert(relative_path, config_clone);
        }
        
        relative_configs
    };
    
    let settings = SettingsFile {
        global_config,
        model_configs,
    };
    
    let contents = serde_json::to_string_pretty(&settings)?;
    fs::write(&settings_path, contents).await?;
    
    tracing::info!("Settings saved successfully to {:?}", settings_path);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_make_path_relative() {
        // Test with Windows-style paths
        let absolute = "C:\\Users\\test\\models\\llama-3.gguf";
        let base = "C:\\Users\\test\\models";
        let result = make_path_relative(absolute, base);
        assert_eq!(result, "llama-3.gguf");

        // Test with nested paths
        let absolute = "C:\\Users\\test\\models\\subfolder\\model.gguf";
        let base = "C:\\Users\\test\\models";
        let result = make_path_relative(absolute, base);
        assert_eq!(result, "subfolder\\model.gguf");

        // Test with path outside base directory
        let absolute = "C:\\Users\\other\\model.gguf";
        let base = "C:\\Users\\test\\models";
        let result = make_path_relative(absolute, base);
        assert_eq!(result, absolute); // Should return original path

        // Test with empty base directory
        let absolute = "C:\\Users\\test\\models\\model.gguf";
        let base = "";
        let result = make_path_relative(absolute, base);
        assert_eq!(result, absolute); // Should return original path
    }

    #[test]
    fn test_make_path_absolute() {
        // Test with relative path
        let relative = "llama-3.gguf";
        let base = "C:\\Users\\test\\models";
        let result = make_path_absolute(relative, base);
        assert_eq!(result, "C:\\Users\\test\\models\\llama-3.gguf");

        // Test with nested relative path
        let relative = "subfolder\\model.gguf";
        let base = "C:\\Users\\test\\models";
        let result = make_path_absolute(relative, base);
        assert_eq!(result, "C:\\Users\\test\\models\\subfolder\\model.gguf");

        // Test with already absolute path
        let absolute = "C:\\Users\\test\\models\\model.gguf";
        let base = "C:\\Users\\test\\models";
        let result = make_path_absolute(absolute, base);
        assert_eq!(result, absolute); // Should return original path

        // Test with empty base directory
        let relative = "model.gguf";
        let base = "";
        let result = make_path_absolute(relative, base);
        assert_eq!(result, relative); // Should return original path
    }

    #[test]
    fn test_remap_arandu_path_old_home_base() {
        let old_base = PathBuf::from(r"C:\Users\tester\.Arandu");
        let new_base = PathBuf::from(r"H:\Ardanu Fix\Arandu-maxi\.Arandu");
        let old_models = r"C:\Users\tester\.Arandu\models";

        let remapped = remap_arandu_path(old_models, &old_base, &new_base);
        assert_eq!(
            remapped,
            Some(r"H:\Ardanu Fix\Arandu-maxi\.Arandu\models".to_string())
        );
    }

    #[test]
    fn test_remap_arandu_path_c_drive_marker_style() {
        let old_base = PathBuf::from(r"D:\Different\.Arandu");
        let new_base = PathBuf::from(r"H:\Ardanu Fix\Arandu-maxi\.Arandu");
        let legacy = r"C:\Users\legacy\.Arandu\llama.cpp\versions\v1";

        let remapped = remap_arandu_path(legacy, &old_base, &new_base);
        assert_eq!(
            remapped,
            Some(r"H:\Ardanu Fix\Arandu-maxi\.Arandu\llama.cpp\versions\v1".to_string())
        );
    }

    #[test]
    fn test_remap_arandu_path_keeps_custom_path() {
        let old_base = PathBuf::from(r"C:\Users\tester\.Arandu");
        let new_base = PathBuf::from(r"H:\Ardanu Fix\Arandu-maxi\.Arandu");
        let custom = r"D:\AI\models";

        let remapped = remap_arandu_path(custom, &old_base, &new_base);
        assert!(remapped.is_none());
    }
}
