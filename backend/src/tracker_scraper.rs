use crate::models::{TrackerConfig, TrackerModel, TrackerStats};
use chrono::Utc;
use reqwest::Client;
use serde::Deserialize;
use std::collections::HashMap;

pub struct TrackerScraper {
    client: Client,
}

#[derive(Debug, Deserialize)]
struct HFSearchResponse {
    id: String,
    author: String,
    #[serde(default, rename = "modelId")]
    model_id: String,
    #[serde(default)]
    sha: String,
    #[serde(default, rename = "lastModified")]
    last_modified: String,
    #[serde(default)]
    private: bool,
    downloads: u64,
    likes: u64,
    #[serde(default)]
    tags: Vec<String>,
    #[serde(default, rename = "pipeline_tag")]
    pipeline_tag: Option<String>,
}

#[derive(Debug, Deserialize)]
struct HFModelDetails {
    id: String,
    author: String,
    #[serde(default)]
    sha: String,
    #[serde(default, rename = "lastModified")]
    last_modified: String,
    #[serde(default)]
    private: bool,
    downloads: u64,
    likes: u64,
    #[serde(default)]
    tags: Vec<String>,
    #[serde(default, rename = "pipeline_tag")]
    pipeline_tag: Option<String>,
    #[serde(default)]
    siblings: Vec<HFFile>,
}

#[derive(Debug, Deserialize)]
struct HFFile {
    #[serde(default)]
    rfilename: String,
    #[serde(default)]
    size: i64,
}

#[derive(Debug, Deserialize)]
struct HFTreeItem {
    #[serde(default)]
    path: String,
    #[serde(default)]
    name: String,
    #[serde(default)]
    size: i64,
    #[serde(default)]
    #[serde(rename = "type")]
    file_type: String,
}

impl TrackerScraper {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }

    pub async fn fetch_trending_models(&self, limit: u32) -> Result<Vec<TrackerModel>, String> {
        let url = format!(
            "https://huggingface.co/api/models?sort=downloads&direction=-1&limit={}&full=true",
            limit
        );

        let response = self.client
            .get(&url)
            .header("Accept", "application/json")
            .send()
            .await
            .map_err(|e| format!("Failed to fetch models: {}", e))?;

        let models: Vec<HFSearchResponse> = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse models: {}", e))?;

        let mut tracker_models = Vec::new();

        for model in models {
            let model_id = if model.model_id.is_empty() {
                format!("{}/{}", model.author, model.id)
            } else {
                model.model_id.clone()
            };

            let details = self.fetch_model_details(&model_id).await;
            
            let (quantizations, backends, is_gguf, size_gb) = if let Ok(d) = &details {
                let files = self.fetch_model_files(&model_id).await.unwrap_or_default();
                let quants = Self::detect_quantizations(&files);
                let backs = Self::detect_backends(&d.tags);
                let gguf = !quants.is_empty();
                let size = files.iter().map(|f| f.size as f64).sum::<f64>() / 1_000_000_000.0;
                (quants, backs, gguf, size)
            } else {
                (Vec::new(), Vec::new(), false, 0.0)
            };

            let category = Self::categorize_model(
                &details.as_ref().ok().map(|d| d.tags.clone()).unwrap_or_default(),
                "",
            );

            let is_chinese = Self::is_chinese_model(&model_id, &details.as_ref().ok().map(|d| d.tags.clone()).unwrap_or_default());

            let description = details.as_ref()
                .ok()
                .and_then(|d| d.pipeline_tag.clone())
                .unwrap_or_else(|| "No description available".to_string());

            tracker_models.push(TrackerModel {
                id: model_id,
                name: model.id.clone(),
                author: model.author,
                description: description[..description.len().min(200)].to_string(),
                source: "huggingface".to_string(),
                category,
                is_chinese,
                is_gguf,
                quantizations,
                backends,
                estimated_size_gb: size_gb,
                vram_requirement_gb: None,
                context_length: None,
                downloads: model.downloads,
                likes: model.likes,
                last_updated: Some(model.last_modified),
                created_at: Utc::now().to_rfc3339(),
            });
        }

        Ok(tracker_models)
    }

    pub async fn fetch_live_results(
        &self,
        query: Option<String>,
        categories: Option<Vec<String>>,
        chinese_only: bool,
        gguf_only: bool,
        limit: u32,
    ) -> Result<Vec<TrackerModel>, String> {
        let mut url = format!(
            "https://huggingface.co/api/models?sort=downloads&direction=-1&limit={}&full=true",
            limit.min(200) // Cap at 200 for speed
        );

        // Add search query if provided
        if let Some(q) = query {
            if !q.is_empty() {
                url.push_str(&format!("&search={}", urlencoding::encode(&q)));
            }
        }

        // Add filter tags based on categories
        if let Some(cats) = categories {
            for cat in cats {
                let tag = match cat.as_str() {
                    "text" => "text-generation",
                    "image" => "image-generation",
                    "audio" => "audio-processing",
                    "video" => "video-processing",
                    "coding" => "code-generation",
                    "multimodal" => "multimodal",
                    _ => continue,
                };
                url.push_str(&format!("&filter={}", tag));
            }
        }

        let response = self.client
            .get(&url)
            .header("Accept", "application/json")
            .send()
            .await
            .map_err(|e| format!("Failed to fetch live results: {}", e))?;

        let models: Vec<HFSearchResponse> = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse live results: {}", e))?;

        let mut tracker_models = Vec::new();

        for model in models {
            let model_id = if model.model_id.is_empty() {
                format!("{}/{}", model.author, model.id)
            } else {
                model.model_id.clone()
            };

            // Apply filters
            if chinese_only && !Self::is_chinese_model(&model_id, &model.tags) {
                continue;
            }

            // Get detailed info for GGUF detection
            let details = self.fetch_model_details(&model_id).await;
            let files = self.fetch_model_files(&model_id).await.unwrap_or_default();
            let quants = Self::detect_quantizations(&files);
            let is_gguf = !quants.is_empty();

            if gguf_only && !is_gguf {
                continue;
            }

            let category = Self::categorize_model(
                &details.as_ref().ok().map(|d| d.tags.clone()).unwrap_or_default(),
                "",
            );

            let size_gb = files.iter().map(|f| f.size as f64).sum::<f64>() / 1_000_000_000.0;

            tracker_models.push(TrackerModel {
                id: model_id.clone(),
                name: model.id.clone(),
                author: model.author.clone(),
                description: details.as_ref()
                    .ok()
                    .and_then(|d| d.pipeline_tag.clone())
                    .unwrap_or_else(|| "No description available".to_string())
                    .chars().take(200).collect(),
                source: "huggingface".to_string(),
                category,
                is_chinese: Self::is_chinese_model(&model_id, &model.tags),
                is_gguf,
                quantizations: quants,
                backends: details.as_ref()
                    .map(|d| Self::detect_backends(&d.tags))
                    .unwrap_or_default(),
                estimated_size_gb: size_gb,
                vram_requirement_gb: None,
                context_length: None,
                downloads: model.downloads,
                likes: model.likes,
                last_updated: Some(model.last_modified),
                created_at: chrono::Utc::now().to_rfc3339(),
            });
        }

        Ok(tracker_models)
    }

    pub async fn fetch_model_details(&self, model_id: &str) -> Result<HFModelDetails, String> {
        let url = format!("https://huggingface.co/api/models/{}", model_id);

        let response = self.client
            .get(&url)
            .header("Accept", "application/json")
            .send()
            .await
            .map_err(|e| format!("Failed to fetch model details: {}", e))?;

        let details: HFModelDetails = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse model details: {}", e))?;

        Ok(details)
    }

    pub async fn fetch_model_files(&self, model_id: &str) -> Result<Vec<HFFile>, String> {
        let url = format!("https://huggingface.co/api/models/{}/tree/main", model_id);

        let response = self.client
            .get(&url)
            .header("Accept", "application/json")
            .send()
            .await
            .map_err(|e| format!("Failed to fetch model files: {}", e))?;

        let tree: Vec<HFTreeItem> = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse model files: {}", e))?;

        let files: Vec<HFFile> = tree
            .into_iter()
            .filter(|item| item.file_type == "file" && item.path.ends_with(".gguf"))
            .map(|item| HFFile {
                rfilename: item.path,
                size: item.size,
            })
            .collect();

        Ok(files)
    }

    pub fn detect_quantizations(files: &[HFFile]) -> Vec<String> {
        let mut quants = Vec::new();
        
        for file in files {
            let name = file.rfilename.to_lowercase();
            
            let quant = if name.contains("iq1_") || name.contains("iq1-") {
                "IQ1"
            } else if name.contains("iq2_") || name.contains("iq2-") {
                "IQ2"
            } else if name.contains("iq3_") || name.contains("iq3-") {
                "IQ3"
            } else if name.contains("iq4_") || name.contains("iq4-") {
                "IQ4"
            } else if name.contains("iq5_") || name.contains("iq5-") {
                "IQ5"
            } else if name.contains("_q2.") || name.contains("-q2.") {
                "Q2"
            } else if name.contains("_q3.") || name.contains("-q3.") {
                "Q3"
            } else if name.contains("_q4_") || name.contains("-q4_") || name.contains("q4_k") {
                "Q4"
            } else if name.contains("_q5.") || name.contains("-q5.") {
                "Q5"
            } else if name.contains("_q6.") || name.contains("-q6.") {
                "Q6"
            } else if name.contains("_q8.") || name.contains("-q8.") || name.contains("q8_0") {
                "Q8"
            } else if name.contains("f16") || name.contains("fp16") {
                "F16"
            } else if name.contains("f32") || name.contains("fp32") {
                "F32"
            } else {
                continue;
            };

            if !quants.contains(&quant.to_string()) {
                quants.push(quant.to_string());
            }
        }

        quants
    }

    pub fn detect_backends(tags: &[String]) -> Vec<String> {
        let mut backends = Vec::new();
        let tag_names: Vec<String> = tags.iter()
            .map(|t| t.to_lowercase())
            .collect();

        if tag_names.iter().any(|t| t.contains("cuda")) {
            backends.push("cuda".to_string());
        }
        if tag_names.iter().any(|t| t.contains("vulkan")) {
            backends.push("vulkan".to_string());
        }
        if tag_names.iter().any(|t| t.contains("rocm") || t.contains("hip")) {
            backends.push("rocm".to_string());
        }
        if tag_names.iter().any(|t| t.contains("cpu") && !t.contains("cuda")) {
            backends.push("cpu".to_string());
        }
        if tag_names.iter().any(|t| t.contains("intel") || t.contains("ipex")) {
            backends.push("intel".to_string());
        }

        backends
    }

    pub fn categorize_model(tags: &[String], _description: &str) -> String {
        let tag_names: Vec<String> = tags.iter()
            .map(|t| t.to_lowercase())
            .collect();

        if tag_names.iter().any(|t| t.contains("text-generation") || t.contains("llm") || t.contains("language-model")) {
            "text".to_string()
        } else if tag_names.iter().any(|t| t.contains("image-classification") || t.contains("stable-diffusion") || t.contains("image-generation") || t.contains("sd3") || t.contains("flux")) {
            "image".to_string()
        } else if tag_names.iter().any(|t| t.contains("video") || t.contains("video-generation")) {
            "video".to_string()
        } else if tag_names.iter().any(|t| t.contains("audio") || t.contains("speech") || t.contains("tts") || t.contains("whisper")) {
            "audio".to_string()
        } else if tag_names.iter().any(|t| t.contains("code") || t.contains("codellama") || t.contains("code-generation")) {
            "coding".to_string()
        } else if tag_names.iter().any(|t| t.contains("multimodal") || t.contains("vision") || t.contains("vlm")) {
            "multimodal".to_string()
        } else {
            "text".to_string()
        }
    }

    pub fn is_chinese_model(model_id: &str, tags: &[String]) -> bool {
        let model_lower = model_id.to_lowercase();
        
        let chinese_prefixes = ["qwen", "deepseek", "yi", "chatglm", "baichuan", "internlm", "skywork", "zhipu", "minimax"];
        
        for prefix in chinese_prefixes {
            if model_lower.starts_with(prefix) || model_lower.contains(prefix) {
                return true;
            }
        }

        let chinese_tags = ["qwen", "deepseek", "chinese", "yi", "chatglm"];
        for tag in tags {
            let tag_name = tag.to_lowercase();
            if chinese_tags.iter().any(|ct| tag_name.contains(ct)) && tag_name.len() < 20 {
                return true;
            }
        }

        false
    }
}
