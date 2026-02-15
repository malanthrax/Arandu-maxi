use crate::models::GgufMetadata;
use std::path::Path;

/// Parse GGUF file metadata
/// For now, uses the existing scanner implementation for reliability
pub fn parse_gguf_metadata(path: &str) -> Result<GgufMetadata, String> {
    let path = Path::new(path);
    
    if !path.exists() {
        return Err("File not found".to_string());
    }
    
    if !path.extension().map_or(false, |ext| ext == "gguf") {
        return Err("Not a GGUF file".to_string());
    }
    
    // Use the existing scanner's extract_gguf_metadata function
    match crate::scanner::extract_gguf_metadata(path) {
        Ok(metadata) => Ok(metadata),
        Err(e) => {
            // Fallback: extract from filename
            eprintln!("Failed to parse GGUF file {}: {:?}", path.display(), e);
            
            let filename = path.file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown");
            
            // Try to extract architecture from filename
            let architecture = if filename.to_lowercase().contains("llama") {
                "llama"
            } else if filename.to_lowercase().contains("glm") {
                "glm"
            } else if filename.to_lowercase().contains("mistral") {
                "mistral"
            } else {
                "unknown"
            }.to_string();
            
            Ok(GgufMetadata {
                architecture,
                name: filename.to_string(),
                quantization: None,
            })
        }
    }
}

/// Get file modification timestamp (Unix epoch seconds)
pub fn get_file_modification_date(path: &str) -> Result<i64, String> {
    let metadata = std::fs::metadata(path)
        .map_err(|e| format!("Failed to get metadata: {}", e))?;
    
    let modified = metadata.modified()
        .map_err(|e| format!("Failed to get modification time: {}", e))?;
    
    let duration = modified.duration_since(std::time::UNIX_EPOCH)
        .map_err(|e| format!("Invalid modification time: {}", e))?;
    
    Ok(duration.as_secs() as i64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_file_modification_date() {
        // Create a temporary file for testing
        let temp_file = std::env::temp_dir().join("test_gguf_parser.tmp");
        std::fs::write(&temp_file, "test content").unwrap();
        
        let result = get_file_modification_date(temp_file.to_str().unwrap());
        assert!(result.is_ok());
        assert!(result.unwrap() > 0);
        
        // Cleanup
        std::fs::remove_file(&temp_file).ok();
    }

    #[test]
    fn test_parse_nonexistent_file() {
        let result = parse_gguf_metadata("/nonexistent/file.gguf");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not found"));
    }

    #[test]
    fn test_parse_non_gguf_file() {
        let temp_file = std::env::temp_dir().join("test.txt");
        std::fs::write(&temp_file, "not a gguf").unwrap();
        
        let result = parse_gguf_metadata(temp_file.to_str().unwrap());
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Not a GGUF"));
        
        // Cleanup
        std::fs::remove_file(&temp_file).ok();
    }
}
