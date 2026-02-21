use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use regex::Regex;

/// Information about a single GGUF file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HfFileInfo {
    pub filename: String,
    pub path: String,
    pub size: u64,
    pub size_formatted: String,
    pub quantization: Option<String>,
    pub commit_date: Option<String>,
}

/// Model information from HuggingFace API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelCardInfo {
    pub id: String,
    pub name: Option<String>,
    pub description: Option<String>,
    pub license: String,
    pub tags: Vec<String>,
    pub downloads: Option<u64>,
    pub likes: Option<u64>,
}

/// Parse various URL formats to extract model ID (author/model)
pub fn parse_model_id(input: &str) -> Result<String, String> {
    let input = input.trim();
    
    // Pattern 1: Full URL with huggingface.co domain
    // https://huggingface.co/author/model
    let full_url_pattern = Regex::new(r"https?://huggingface\.co/([^/]+/[^/]+)").unwrap();
    if let Some(caps) = full_url_pattern.captures(input) {
        return Ok(caps.get(1).unwrap().as_str().to_string());
    }
    
    // Pattern 2: URL without protocol
    // huggingface.co/author/model
    let no_protocol_pattern = Regex::new(r"huggingface\.co/([^/]+/[^/]+)").unwrap();
    if let Some(caps) = no_protocol_pattern.captures(input) {
        return Ok(caps.get(1).unwrap().as_str().to_string());
    }
    
    // Pattern 3: Direct file URL (blob path)
    // https://huggingface.co/author/model/blob/main/file.gguf
    let blob_url_pattern = Regex::new(r"https?://huggingface\.co/([^/]+/[^/]+)/blob/").unwrap();
    if let Some(caps) = blob_url_pattern.captures(input) {
        return Ok(caps.get(1).unwrap().as_str().to_string());
    }
    
    // Pattern 4: Resolve URL (direct download)
    // https://huggingface.co/author/model/resolve/main/file.gguf
    let resolve_url_pattern = Regex::new(r"https?://huggingface\.co/([^/]+/[^/]+)/resolve/").unwrap();
    if let Some(caps) = resolve_url_pattern.captures(input) {
        return Ok(caps.get(1).unwrap().as_str().to_string());
    }
    
    // Pattern 5: Just the ID (author/model)
    // Must match exactly: author/model (alphanumeric, hyphens, underscores)
    let id_pattern = Regex::new(r"^[a-zA-Z0-9_-]+/[a-zA-Z0-9_-]+$").unwrap();
    if id_pattern.is_match(input) {
        return Ok(input.to_string());
    }
    
    Err("Invalid URL format. Expected: https://huggingface.co/author/model or author/model".to_string())
}

/// Fetch model information from HuggingFace API
pub async fn fetch_model_info(model_id: &str) -> Result<ModelCardInfo, String> {
    let url = format!("https://huggingface.co/api/models/{}", model_id);
    
    let client = reqwest::Client::new();
    let response = client
        .get(&url)
        .header("User-Agent", "Arandu-Tauri/1.0")
        .send()
        .await
        .map_err(|e| format!("Failed to fetch model info: {}", e))?;
    
    if !response.status().is_success() {
        let status = response.status();
        if status == reqwest::StatusCode::NOT_FOUND {
            return Err("Model not found".to_string());
        } else if status == reqwest::StatusCode::FORBIDDEN {
            return Err("Access denied. This model may require authentication.".to_string());
        } else if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
            return Err("Rate limited. Please wait a moment and try again.".to_string());
        }
        return Err(format!("API request failed (HTTP {})", status));
    }
    
    let data: serde_json::Value = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))?;
    
    // Extract description from cardData if available
    let description = data.get("cardData")
        .and_then(|v| v.as_object())
        .and_then(|obj| obj.get("description"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    
    // Alternative: use pipeline_tag or tags to infer model type
    let tags: Vec<String> = data.get("tags")
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter()
            .filter_map(|v| v.as_str())
            .map(|s| s.to_string())
            .collect()
        )
        .unwrap_or_default();
    
    Ok(ModelCardInfo {
        id: data.get("id").and_then(|v| v.as_str()).unwrap_or(model_id).to_string(),
        name: data.get("modelId").and_then(|v| v.as_str()).map(|s| s.to_string()),
        description,
        license: data.get("license").and_then(|v| v.as_str()).unwrap_or("Unknown").to_string(),
        tags,
        downloads: data.get("downloads").and_then(|v| v.as_u64()),
        likes: data.get("likes").and_then(|v| v.as_u64()),
    })
}

/// Fetch list of GGUF files from model repository
pub async fn fetch_model_files(model_id: &str) -> Result<Vec<HfFileInfo>, String> {
    let url = format!("https://huggingface.co/api/models/{}/tree/main", model_id);
    
    let client = reqwest::Client::new();
    let response = client
        .get(&url)
        .header("User-Agent", "Arandu-Tauri/1.0")
        .send()
        .await
        .map_err(|e| format!("Failed to fetch file list: {}", e))?;
    
    if !response.status().is_success() {
        let status = response.status();
        if status == reqwest::StatusCode::NOT_FOUND {
            return Err("Model repository not found".to_string());
        }
        return Err(format!("Failed to fetch files (HTTP {})", status));
    }
    
    let files: Vec<serde_json::Value> = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse file list: {}", e))?;
    
    let mut gguf_files: Vec<HfFileInfo> = files
        .into_iter()
        .filter(|file| {
            file.get("path")
                .and_then(|p| p.as_str())
                .map(|path| {
                    let path_lower = path.to_lowercase();
                    path_lower.ends_with(".gguf") || path_lower.ends_with(".gguf.download")
                })
                .unwrap_or(false)
        })
        .filter(|file| {
            // Only include files (not directories)
            file.get("type")
                .and_then(|t| t.as_str())
                .map(|t| t == "file")
                .unwrap_or(true) // Default to true if type not specified
        })
        .map(|file| {
            let path = file.get("path").and_then(|p| p.as_str()).unwrap_or("").to_string();
            let filename = path.split('/').last().unwrap_or(&path).to_string();
            let size = file.get("size").and_then(|s| s.as_u64()).unwrap_or(0);
            
            // Extract quantization from filename
            let quantization = extract_quantization(&filename);
            
            HfFileInfo {
                path: path.clone(),
                filename: filename.clone(),
                size,
                size_formatted: format_bytes(size),
                quantization,
                commit_date: file.get("lastCommit")
                    .and_then(|c| c.get("date"))
                    .and_then(|d| d.as_str())
                    .map(|s| s.to_string()),
            }
        })
        .collect();
    
    // Sort by size (smallest first)
    gguf_files.sort_by_key(|f| f.size);
    
    Ok(gguf_files)
}

/// Extract quantization from filename
/// e.g., "model-Q4_K_M.gguf" -> "Q4_K_M"
fn extract_quantization(filename: &str) -> Option<String> {
    // Remove .gguf extension
    let base = filename.strip_suffix(".gguf")?;
    
    // Common patterns for quantization extraction
    let patterns = [
        // IQ formats with underscores
        Regex::new(r"[-_](IQ[0-9]+_[A-Z]+)$").unwrap(),
        // Q formats with K variants (Q4_K_M, etc.)
        Regex::new(r"[-_](Q[0-9]+_[A-Z]+_[A-Z]+)$").unwrap(),
        // Q formats with single suffix (Q4_K, etc.)
        Regex::new(r"[-_](Q[0-9]+_[A-Z]+)$").unwrap(),
        // Standard Q formats (Q4_0, Q5_1, etc.)
        Regex::new(r"[-_](Q[0-9]+_[0-9])$").unwrap(),
        // Float formats
        Regex::new(r"[-_](F16|FP16|F32|FP32|BF16)$").unwrap(),
        // Legacy Q formats (Q4, Q5, etc.)
        Regex::new(r"[-_](Q[0-9]+)$").unwrap(),
    ];
    
    for pattern in &patterns {
        if let Some(caps) = pattern.captures(base) {
            return caps.get(1).map(|m| m.as_str().to_uppercase());
        }
    }
    
    None
}

/// Build default destination path
pub fn build_destination_path(base_dir: &str, model_id: &str) -> PathBuf {
    let path = PathBuf::from(base_dir);
    let parts: Vec<&str> = model_id.split('/').collect();
    
    if parts.len() >= 2 {
        // ~/.Arandu/models/author/model/
        path.join("models").join(parts[0]).join(parts[1])
    } else {
        path.join("models").join(model_id)
    }
}

/// Format bytes to human readable
fn format_bytes(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit_index = 0;
    
    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }
    
    if unit_index == 0 {
        format!("{} {}", bytes, UNITS[unit_index])
    } else {
        format!("{:.1} {}", size, UNITS[unit_index])
    }
}

/// Extract filename from a direct file URL
/// e.g., "https://huggingface.co/author/model/blob/main/file.gguf" -> "file.gguf"
pub fn extract_filename_from_url(url: &str) -> Option<String> {
    // Pattern for blob/resolve URLs
    let blob_pattern = Regex::new(r"/blob/[^/]+/(.+)$").unwrap();
    let resolve_pattern = Regex::new(r"/resolve/[^/]+/(.+)$").unwrap();
    
    if let Some(caps) = blob_pattern.captures(url) {
        return caps.get(1).map(|m| m.as_str().to_string());
    }
    
    if let Some(caps) = resolve_pattern.captures(url) {
        return caps.get(1).map(|m| m.as_str().to_string());
    }
    
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_model_id_full_url() {
        let url = "https://huggingface.co/THUDM/glm-4-9b-chat";
        assert_eq!(parse_model_id(url).unwrap(), "THUDM/glm-4-9b-chat");
    }

    #[test]
    fn test_parse_model_id_no_protocol() {
        let url = "huggingface.co/THUDM/glm-4-9b-chat";
        assert_eq!(parse_model_id(url).unwrap(), "THUDM/glm-4-9b-chat");
    }

    #[test]
    fn test_parse_model_id_direct_id() {
        let id = "THUDM/glm-4-9b-chat";
        assert_eq!(parse_model_id(id).unwrap(), "THUDM/glm-4-9b-chat");
    }

    #[test]
    fn test_parse_model_id_blob_url() {
        let url = "https://huggingface.co/THUDM/glm-4-9b-chat/blob/main/model.gguf";
        assert_eq!(parse_model_id(url).unwrap(), "THUDM/glm-4-9b-chat");
    }

    #[test]
    fn test_parse_model_id_resolve_url() {
        let url = "https://huggingface.co/THUDM/glm-4-9b-chat/resolve/main/model.gguf";
        assert_eq!(parse_model_id(url).unwrap(), "THUDM/glm-4-9b-chat");
    }

    #[test]
    fn test_parse_model_id_with_subfolder() {
        let url = "https://huggingface.co/black-forest-labs/FLUX.1-schnell-GGUF";
        assert_eq!(parse_model_id(url).unwrap(), "black-forest-labs/FLUX.1-schnell-GGUF");
    }

    #[test]
    fn test_parse_model_id_invalid() {
        assert!(parse_model_id("not-a-url").is_err());
        assert!(parse_model_id("https://huggingface.co/invalid").is_err());
        assert!(parse_model_id("").is_err());
    }

    #[test]
    fn test_extract_quantization() {
        assert_eq!(
            extract_quantization("model-Q4_K_M.gguf"),
            Some("Q4_K_M".to_string())
        );
        assert_eq!(
            extract_quantization("model-Q5_0.gguf"),
            Some("Q5_0".to_string())
        );
        assert_eq!(
            extract_quantization("model-F16.gguf"),
            Some("F16".to_string())
        );
        assert_eq!(
            extract_quantization("flux1-schnell-Q4_K_M.gguf"),
            Some("Q4_K_M".to_string())
        );
    }

    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(0), "0 B");
        assert_eq!(format_bytes(1024), "1.0 KB");
        assert_eq!(format_bytes(1024 * 1024), "1.0 MB");
        assert_eq!(format_bytes(1024 * 1024 * 1024), "1.0 GB");
    }

    #[test]
    fn test_build_destination_path() {
        let base = "C:\\Users\\Test";
        let model_id = "author/model";
        let path = build_destination_path(base, model_id);
        assert!(path.to_string_lossy().contains("models"));
        assert!(path.to_string_lossy().contains("author"));
        assert!(path.to_string_lossy().contains("model"));
    }

    #[test]
    fn test_extract_filename_from_url() {
        let url = "https://huggingface.co/author/model/blob/main/model-Q4_K_M.gguf";
        assert_eq!(
            extract_filename_from_url(url),
            Some("model-Q4_K_M.gguf".to_string())
        );
    }
}
