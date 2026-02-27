# Task 6: Chat Completion with Streaming - Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Implement OpenAI-compatible chat completions endpoint that forwards requests to llama.cpp server with support for both streaming (SSE) and non-streaming responses.

**Architecture:** 
- Extend `openai_proxy.rs` to forward POST requests to llama.cpp's `/completion` endpoint
- Add streaming support using Server-Sent Events (SSE) for `stream=true` requests
- Maintain OpenAI API response format for compatibility
- Handle errors gracefully with proper HTTP status codes

**Tech Stack:** 
- Rust, Axum web framework
- reqwest for HTTP client
- tokio for async streaming
- SSE (text/event-stream) for streaming responses

---

## Current State

The OpenAI proxy server has a placeholder `chat_completions` handler at:
- **File:** `backend/src/openai_proxy.rs:106-120`
- **Current behavior:** Returns HTTP 501 "not implemented" error
- **Target behavior:** Forward to llama.cpp and return actual completions

**llama.cpp API Endpoint:**
- Non-streaming: `POST http://<host>:<port>/completion`
- Streaming: `POST http://<host>:<port>/completion` with `stream: true`
- Request format: Similar to OpenAI (messages, temperature, max_tokens, etc.)
- Response format: OpenAI-compatible

---

## Task 1: Add reqwest HTTP Client Dependency

**Files:**
- Modify: `backend/Cargo.toml`

**Step 1: Add reqwest dependency**

Add to `[dependencies]` section:
```toml
reqwest = { version = "0.12", features = ["json", "stream"] }
```

**Step 2: Verify dependency**

Run: `cargo check`
Expected: Compiles without errors

**Step 3: Commit**

```bash
git add backend/Cargo.toml
git commit -m "deps: add reqwest for HTTP client in OpenAI proxy"
```

---

## Task 2: Create HTTP Client Module for llama.cpp Communication

**Files:**
- Create: `backend/src/llama_client.rs`
- Modify: `backend/src/lib.rs` (add module declaration)

**Step 1: Create llama_client.rs**

```rust
use reqwest::Client;
use serde_json::{json, Value};
use crate::openai_types::{ChatCompletionRequest, ChatMessage};

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
        // llama.cpp uses "prompt" instead of "messages" for the /completion endpoint
        // but newer versions support OpenAI-compatible /v1/chat/completions
        // We'll use /v1/chat/completions directly since llama.cpp supports it
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
```

**Step 2: Add module to lib.rs**

Add to `backend/src/lib.rs` after other module declarations:
```rust
mod llama_client;
```

**Step 3: Run cargo check**

Run: `cargo check`
Expected: Compiles without errors

**Step 4: Commit**

```bash
git add backend/src/llama_client.rs backend/src/lib.rs
git commit -m "feat: add llama_client module for llama.cpp HTTP communication"
```

---

## Task 3: Update ProxyState to Include LlamaClient

**Files:**
- Modify: `backend/src/openai_proxy.rs:78-80`

**Step 1: Add LlamaClient to ProxyState**

```rust
use crate::llama_client::LlamaClient;

/// Shared state for proxy handlers
pub struct ProxyState {
    pub llama_server_url: String,
    pub llama_client: LlamaClient,
}
```

**Step 2: Update ProxyServer::start() to initialize LlamaClient**

In `start()` method, update the state creation:
```rust
.with_state(Arc::new(RwLock::new(ProxyState {
    llama_server_url: self.llama_server_url.clone(),
    llama_client: LlamaClient::new(self.llama_server_url.clone()),
})));
```

**Step 3: Run cargo check**

Run: `cargo check`
Expected: Compiles without errors

**Step 4: Commit**

```bash
git add backend/src/openai_proxy.rs
git commit -m "feat: integrate LlamaClient into proxy state"
```

---

## Task 4: Implement Non-Streaming Chat Completions

**Files:**
- Modify: `backend/src/openai_proxy.rs:106-120`

**Step 1: Rewrite chat_completions handler for non-streaming**

```rust
async fn chat_completions(
    State(state): State<Arc<RwLock<ProxyState>>>,
    Json(request): Json<ChatCompletionRequest>,
) -> impl IntoResponse {
    // Check if streaming is requested
    let stream = request.stream.unwrap_or(false);
    
    if stream {
        // Handle streaming in Task 5
        return handle_streaming_completion(state, request).await;
    }
    
    // Handle non-streaming completion
    let state_guard = state.read().await;
    let client = &state_guard.llama_client;
    
    match client.chat_completion(&request).await {
        Ok(response) => {
            // llama.cpp returns OpenAI-compatible format, just pass it through
            (StatusCode::OK, Json(response))
        }
        Err(e) => {
            let error = OpenAIErrorResponse {
                error: OpenAIError {
                    message: e,
                    error_type: "api_error".to_string(),
                    code: Some("500".to_string()),
                },
            };
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error))
        }
    }
}
```

**Step 2: Run cargo check**

Run: `cargo check`
Expected: Compiles (will warn about unused handle_streaming_completion)

**Step 3: Commit**

```bash
git add backend/src/openai_proxy.rs
git commit -m "feat: implement non-streaming chat completions"
```

---

## Task 5: Implement Streaming Chat Completions (SSE)

**Files:**
- Create: `backend/src/openai_types.rs` (add streaming types)
- Modify: `backend/src/openai_proxy.rs` (add streaming handler)

**Step 1: Add streaming types to openai_types.rs**

Add after existing types:

```rust
// ============== STREAMING ==============

#[derive(Debug, Serialize)]
pub struct ChatCompletionStreamResponse {
    pub id: String,
    pub object: String,
    pub created: i64,
    pub model: String,
    pub choices: Vec<ChatCompletionStreamChoice>,
}

#[derive(Debug, Serialize)]
pub struct ChatCompletionStreamChoice {
    pub index: i32,
    pub delta: ChatMessageDelta,
    pub finish_reason: Option<String>,
}

#[derive(Debug, Serialize, Default)]
pub struct ChatMessageDelta {
    pub role: Option<String>,
    pub content: Option<String>,
}
```

**Step 2: Add streaming handler to openai_proxy.rs**

Add imports at top:
```rust
use axum::response::sse::{Event, Sse};
use std::convert::Infallible;
use futures::stream::{self, Stream};
```

Add streaming handler function:

```rust
async fn handle_streaming_completion(
    state: Arc<RwLock<ProxyState>>,
    request: ChatCompletionRequest,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let state_guard = state.read().await;
    let client = state_guard.llama_client.clone();
    drop(state_guard);
    
    let stream = async_stream::stream! {
        match client.chat_completion_stream(&request).await {
            Ok(response) => {
                let mut stream = response.bytes_stream();
                
                while let Some(chunk) = stream.next().await {
                    match chunk {
                        Ok(bytes) => {
                            // Parse SSE data from llama.cpp
                            let text = String::from_utf8_lossy(&bytes);
                            for line in text.lines() {
                                if line.starts_with("data: ") {
                                    let data = &line[6..];
                                    if data == "[DONE]" {
                                        yield Ok(Event::default().data("[DONE]"));
                                    } else {
                                        yield Ok(Event::default().data(data));
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!("Stream error: {}", e);
                            break;
                        }
                    }
                }
            }
            Err(e) => {
                let error = json!({
                    "error": {
                        "message": e,
                        "type": "api_error"
                    }
                });
                yield Ok(Event::default().data(error.to_string()));
            }
        }
    };
    
    Sse::new(stream).keep_alive(axum::response::sse::KeepAlive::default())
}
```

**Step 3: Add async-stream dependency**

Add to `backend/Cargo.toml`:
```toml
async-stream = "0.3"
futures = "0.3"
```

**Step 4: Clone LlamaClient**

Add to `llama_client.rs`:
```rust
#[derive(Clone)]
pub struct LlamaClient {
    client: Client,
    base_url: String,
}
```

**Step 5: Run cargo check**

Run: `cargo check`
Expected: Compiles without errors

**Step 6: Commit**

```bash
git add backend/Cargo.toml backend/src/openai_types.rs backend/src/openai_proxy.rs backend/src/llama_client.rs
git commit -m "feat: implement streaming chat completions with SSE"
```

---

## Task 6: Add Error Handling and Timeout Configuration

**Files:**
- Modify: `backend/src/llama_client.rs`
- Modify: `backend/src/openai_proxy.rs`

**Step 1: Add timeout to LlamaClient**

```rust
use std::time::Duration;

impl LlamaClient {
    pub fn new(base_url: String) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(300)) // 5 minute timeout
            .connect_timeout(Duration::from_secs(10))
            .build()
            .expect("Failed to create HTTP client");
            
        Self { client, base_url }
    }
    // ... rest unchanged
}
```

**Step 2: Handle connection errors better in openai_proxy.rs**

Update error handling to check if llama.cpp is running:

```rust
async fn chat_completions(
    State(state): State<Arc<RwLock<ProxyState>>>,
    Json(request): Json<ChatCompletionRequest>,
) -> impl IntoResponse {
    let state_guard = state.read().await;
    let llama_url = state_guard.llama_server_url.clone();
    drop(state_guard);
    
    // Check if llama.cpp server is reachable
    let health_url = format!("{}/health", llama_url);
    let client = reqwest::Client::new();
    
    match client.get(&health_url).timeout(Duration::from_secs(2)).send().await {
        Ok(resp) if resp.status().is_success() => {
            // Server is healthy, proceed with completion
        }
        _ => {
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(OpenAIErrorResponse {
                    error: OpenAIError {
                        message: "No llama.cpp server is currently running. Please start a model first.".to_string(),
                        error_type: "server_not_running".to_string(),
                        code: Some("503".to_string()),
                    },
                })
            );
        }
    }
    
    // ... rest of handler
}
```

**Step 3: Run cargo check**

Run: `cargo check`
Expected: Compiles without errors

**Step 4: Commit**

```bash
git add backend/src/llama_client.rs backend/src/openai_proxy.rs
git commit -m "feat: add timeout and connection error handling"
```

---

## Task 7: Build and Test

**Files:**
- All modified files

**Step 1: Full build**

Run: `cargo tauri build`
Expected: Successful release build

**Step 2: Manual test checklist**

1. Start Arandu application
2. Launch a model (e.g., click on a GGUF file)
3. Activate the OpenAI proxy server (Network Serve widget)
4. Test with curl:

```bash
# Non-streaming test
curl -X POST http://127.0.0.1:8081/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d '{
    "model": "local-llama",
    "messages": [{"role": "user", "content": "Hello!"}],
    "max_tokens": 50
  }'

# Streaming test
curl -X POST http://127.0.0.1:8081/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d '{
    "model": "local-llama",
    "messages": [{"role": "user", "content": "Hello!"}],
    "stream": true,
    "max_tokens": 50
  }'
```

**Step 3: Test error case**

Stop the model and try the curl command - should return 503 error.

**Step 4: Commit**

```bash
git commit -m "build: successful chat completions implementation"
```

---

## Summary

This implementation provides:
1. ✅ Non-streaming chat completions via llama.cpp
2. ✅ Streaming chat completions with Server-Sent Events
3. ✅ OpenAI API compatibility (request/response format)
4. ✅ Error handling with proper HTTP status codes
5. ✅ Connection validation (checks if llama.cpp is running)
6. ✅ Timeout configuration (5 min request, 10 sec connect)

**Next Steps:**
- Task 7: Audio endpoints (whisper.cpp integration)
- Task 8: Image generation endpoints (Stable Diffusion)

**Key Design Decisions:**
- Use llama.cpp's native `/v1/chat/completions` endpoint (already OpenAI-compatible)
- Forward requests as-is with minimal transformation
- Use SSE for streaming to match OpenAI spec
- Check llama.cpp health before processing requests
