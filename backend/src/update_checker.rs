use crate::models::{UpdateCheckResult, UpdateStatus, HfMetadata};
use std::path::Path;

/// Try to extract HF model ID from folder path
/// Path format: models/author/model-name/file.gguf
pub fn extract_hf_model_id_from_path(path: &str) -> Option<String> {
    let path = Path::new(path);
    let file_name = path.file_name()?.to_str()?;
    let parent = path.parent()?;
    let model_name = parent.file_name()?.to_str()?;
    let grandparent = parent.parent()?;
    let author = grandparent.file_name()?.to_str()?;
    
    // Validate it looks like an HF path (author/model-name/file.gguf)
    if author.is_empty() || model_name.is_empty() || file_name.is_empty() {
        return None;
    }
    
    // Check if filename contains model name or common model patterns
    if file_name.to_lowercase().contains(&model_name.to_lowercase()) ||
       model_name.to_lowercase().contains("glm") ||
       model_name.to_lowercase().contains("llama") ||
       model_name.to_lowercase().contains("mistral") ||
       model_name.to_lowercase().contains("qwen") ||
       model_name.to_lowercase().contains("phi") {
        Some(format!("{}/{}", author, model_name))
    } else {
        None
    }
}

/// Check for updates on HuggingFace
pub async fn check_huggingface_updates(
    model_path: &str,
    hf_metadata: Option<&HfMetadata>,
    local_modification_date: i64,
) -> UpdateCheckResult {
    // Tier 1: Use explicit HF metadata
    let (model_id, filename) = if let Some(metadata) = hf_metadata {
        (metadata.model_id.clone(), metadata.filename.clone())
    } else {
        // Tier 2: Try to extract from path
        match extract_hf_model_id_from_path(model_path) {
            Some(id) => {
                let filename = Path::new(model_path)
                    .file_name()
                    .and_then(|f| f.to_str())
                    .unwrap_or("unknown")
                    .to_string();
                (id, filename)
            }
            None => {
                return UpdateCheckResult {
                    status: UpdateStatus::NotLinked,
                    local_date: None,
                    remote_date: None,
                    message: "Model not linked to HuggingFace. Click to link.".to_string(),
                };
            }
        }
    };
    
    // Fetch file info from HF API
    let api_url = format!(
        "https://huggingface.co/api/models/{}/tree/main",
        model_id
    );
    
    let client = reqwest::Client::new();
    let response = match client.get(&api_url).send().await {
        Ok(resp) => resp,
        Err(e) => {
            return UpdateCheckResult {
                status: UpdateStatus::Error(format!("Failed to fetch from HF: {}", e)),
                local_date: None,
                remote_date: None,
                message: format!("API error: {}", e),
            };
        }
    };
    
    if !response.status().is_success() {
        return UpdateCheckResult {
            status: UpdateStatus::Error(format!("HF API returned: {}", response.status())),
            local_date: None,
            remote_date: None,
            message: format!("API error: {}", response.status()),
        };
    }
    
    let files: serde_json::Value = match response.json().await {
        Ok(data) => data,
        Err(e) => {
            return UpdateCheckResult {
                status: UpdateStatus::Error(format!("Failed to parse HF response: {}", e)),
                local_date: None,
                remote_date: None,
                message: format!("Parse error: {}", e),
            };
        }
    };
    
    // Find matching file
    let local_datetime = chrono::DateTime::from_timestamp(local_modification_date, 0)
        .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string());
    
    if let Some(files_array) = files.as_array() {
        for file in files_array {
            if let Some(path_val) = file.get("path").and_then(|p| p.as_str()) {
                if path_val == filename || path_val.ends_with(&filename) {
                    // Get last commit date
                    let remote_date = file.get("lastCommit")
                        .and_then(|c| c.get("date"))
                        .and_then(|d| d.as_str())
                        .map(|s| s.to_string());
                    
                    if let Some(ref remote_str) = remote_date {
                        // Parse remote date
                        if let Ok(remote_dt) = chrono::DateTime::parse_from_rfc3339(remote_str) {
                            let remote_timestamp = remote_dt.timestamp();
                            
                            // Compare dates (allow 1 hour buffer for timezone differences)
                            if local_modification_date >= remote_timestamp - 3600 {
                                return UpdateCheckResult {
                                    status: UpdateStatus::UpToDate,
                                    local_date: local_datetime,
                                    remote_date: Some(remote_dt.format("%Y-%m-%d %H:%M:%S").to_string()),
                                    message: "Model is up to date".to_string(),
                                };
                            } else {
                                return UpdateCheckResult {
                                    status: UpdateStatus::UpdateAvailable,
                                    local_date: local_datetime,
                                    remote_date: Some(remote_dt.format("%Y-%m-%d %H:%M:%S").to_string()),
                                    message: "Update available on HuggingFace".to_string(),
                                };
                            }
                        }
                    }
                    
                    // If we found file but couldn't parse date, check file size as fallback
                    let remote_size = file.get("size").and_then(|s| s.as_u64());
                    let local_size = std::fs::metadata(model_path)
                        .ok()
                        .map(|m| m.len());
                    
                    if remote_size == local_size {
                        return UpdateCheckResult {
                            status: UpdateStatus::UpToDate,
                            local_date: local_datetime,
                            remote_date,
                            message: "Model appears up to date (size match)".to_string(),
                        };
                    } else {
                        return UpdateCheckResult {
                            status: UpdateStatus::UpdateAvailable,
                            local_date: local_datetime,
                            remote_date,
                            message: "Update available (size differs)".to_string(),
                        };
                    }
                }
            }
        }
    }
    
    // File not found on HF
    UpdateCheckResult {
        status: UpdateStatus::Unknown,
        local_date: local_datetime,
        remote_date: None,
        message: "File not found in HuggingFace repository".to_string(),
    }
}

/// Link a local model to HuggingFace
pub fn link_model_to_hf(
    _model_path: &str,
    model_id: &str,
    filename: &str,
) -> Result<HfMetadata, String> {
    let metadata = HfMetadata {
        model_id: model_id.to_string(),
        filename: filename.to_string(),
        commit_date: None,
        linked_at: chrono::Utc::now().to_rfc3339(),
    };
    
    Ok(metadata)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_hf_model_id_from_path() {
        // Test GLM model path
        let path = "C:/Users/Gordprime/.Arandu/models/THUDM/glm-4-9b-chat/glm-4-9b-chat-Q4_K_M.gguf";
        let result = extract_hf_model_id_from_path(path);
        assert_eq!(result, Some("THUDM/glm-4-9b-chat".to_string()));
        
        // Test non-HF path (no author/model structure)
        let path2 = "C:/Users/Gordprime/.Arandu/models/model.gguf";
        let result2 = extract_hf_model_id_from_path(path2);
        assert_eq!(result2, None);
    }

    #[test]
    fn test_link_model_to_hf() {
        let result = link_model_to_hf(
            "/path/to/model.gguf",
            "THUDM/glm-4-9b-chat",
            "model-Q4_K_M.gguf"
        );
        
        assert!(result.is_ok());
        let metadata = result.unwrap();
        assert_eq!(metadata.model_id, "THUDM/glm-4-9b-chat");
        assert_eq!(metadata.filename, "model-Q4_K_M.gguf");
        assert!(metadata.linked_at.len() > 0);
    }
}
