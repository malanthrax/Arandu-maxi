use reqwest::Client;
use serde_json::{json, Value};
use crate::openai_types::ChatCompletionRequest;

#[derive(Clone)]
pub struct LlamaClient {
    client: Client,
    base_url: String,
}

impl LlamaClient {
    pub fn new(base_url: String) -> Self {
        Self {
            client: Client::new(),
            base_url,
        }
    }

    /// Convert OpenAI format request to llama.cpp format
    pub fn convert_request(&self, openai_req: &ChatCompletionRequest) -> Value {
        // llama.cpp uses OpenAI-compatible /v1/chat/completions endpoint
        json!({
            "model": &openai_req.model,
            "messages": openai_req.messages,
            "temperature": openai_req.temperature.unwrap_or(0.7),
            "max_tokens": openai_req.max_tokens,
            "stream": openai_req.stream.unwrap_or(false),
            "top_p": openai_req.top_p,
            "stop": openai_req.stop,
        })
    }

    /// Send non-streaming chat completion request
    pub async fn chat_completion(&self, request: &ChatCompletionRequest) -> Result<Value, String> {
        let url = format!("{}/v1/chat/completions", self.base_url);
        let body = self.convert_request(request);

        let response = self.client
            .post(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("Failed to connect to llama.cpp: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(format!("llama.cpp returned error {}: {}", status, text));
        }

        response.json::<Value>().await
            .map_err(|e| format!("Failed to parse llama.cpp response: {}", e))
    }

    /// Send streaming chat completion request
    pub async fn chat_completion_stream(
        &self, 
        request: &ChatCompletionRequest
    ) -> Result<reqwest::Response, String> {
        let url = format!("{}/v1/chat/completions", self.base_url);
        let body = self.convert_request(request);

        let response = self.client
            .post(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("Failed to connect to llama.cpp: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(format!("llama.cpp returned error {}: {}", status, text));
        }

        Ok(response)
    }
}
