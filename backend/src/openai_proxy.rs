use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::{get, post},
    Router,
};
use axum::response::sse::{Event, Sse};
use std::convert::Infallible;
use futures::stream::Stream;
use futures_util::StreamExt;
use serde_json::json;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tower_http::cors::{Any, CorsLayer};
use crate::openai_types::{
    ChatCompletionRequest, AudioTranscriptionRequest, AudioTranscriptionResponse,
    AudioSpeechRequest, ImageGenerationRequest,
    ModelInfo, ModelsResponse, OpenAIError, OpenAIErrorResponse
};
use crate::llama_client::LlamaClient;

/// OpenAI-compatible API proxy server
#[derive(Debug)]
pub struct ProxyServer {
    llama_server_url: String,
    proxy_port: u16,
    shutdown_tx: Option<tokio::sync::mpsc::Sender<()>>,
}

impl ProxyServer {
    pub fn new(llama_server_host: String, llama_server_port: u16, proxy_port: u16) -> Self {
        Self {
            llama_server_url: format!("http://{}:{}", llama_server_host, llama_server_port),
            proxy_port,
            shutdown_tx: None,
        }
    }

    pub async fn start(&mut self) -> Result<(), String> {
        // Configure CORS to allow all origins (needed for cross-LAN access)
        let cors = CorsLayer::new()
            .allow_origin(Any)
            .allow_methods(Any)
            .allow_headers(Any);

        let app = Router::new()
            .route("/v1/models", get(list_models))
            .route("/v1/chat/completions", post(chat_completions))
            .route("/v1/audio/transcriptions", post(audio_transcriptions))
            .route("/v1/audio/speech", post(audio_speech))
            .route("/v1/images/generations", post(image_generations))
            .route("/health", get(health_check))
            .layer(cors)
            .with_state(Arc::new(RwLock::new(ProxyState {
                llama_server_url: self.llama_server_url.clone(),
                llama_client: LlamaClient::new(self.llama_server_url.clone()),
            })));

        let addr = SocketAddr::from(([0, 0, 0, 0], self.proxy_port));
        
        let (shutdown_tx, mut shutdown_rx) = tokio::sync::mpsc::channel::<()>(1);
        self.shutdown_tx = Some(shutdown_tx);

        let listener = tokio::net::TcpListener::bind(addr)
            .await
            .map_err(|e| format!("Failed to bind proxy server: {}", e))?;

        println!("OpenAI proxy server starting on {}", addr);

        tokio::spawn(async move {
            axum::serve(listener, app)
                .with_graceful_shutdown(async move {
                    shutdown_rx.recv().await;
                })
                .await
                .unwrap_or_else(|e| eprintln!("Proxy server error: {}", e));
        });

        Ok(())
    }

    pub async fn stop(&mut self) {
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(()).await;
        }
    }
}

/// Shared state for proxy handlers
pub struct ProxyState {
    pub llama_server_url: String,
    pub llama_client: LlamaClient,
}

// ============== HANDLER FUNCTIONS ==============

async fn health_check() -> impl IntoResponse {
    Json(json!({"status": "healthy"}))
}

async fn list_models(
    State(state): State<Arc<RwLock<ProxyState>>>,
) -> impl IntoResponse {
    let state_guard = state.read().await;
    // llama.cpp uses /props endpoint to get model info, not /v1/models
    let url = format!("{}/props", state_guard.llama_server_url);
    drop(state_guard);

    let client = reqwest::Client::new();

    match client.get(&url).timeout(std::time::Duration::from_secs(5)).send().await {
        Ok(response) if response.status().is_success() => {
            // Parse llama.cpp props response to get model name
            match response.json::<serde_json::Value>().await {
                Ok(props) => {
                    // llama.cpp returns model info in different possible fields
                    let model_name = props.get("model")
                        .and_then(|m| m.as_str())
                        .or_else(|| props.get("default_generation_settings")
                            .and_then(|s| s.get("model"))
                            .and_then(|m| m.as_str()))
                        .or_else(|| props.get("generation_settings")
                            .and_then(|s| s.get("model"))
                            .and_then(|m| m.as_str()))
                        .unwrap_or("unknown-model");
                    
                    let models = vec![ModelInfo {
                        id: model_name.to_string(),
                        object: "model".to_string(),
                        created: chrono::Utc::now().timestamp(),
                        owned_by: "llama.cpp".to_string(),
                    }];
                    let response = ModelsResponse {
                        object: "list".to_string(),
                        data: models,
                    };
                    (StatusCode::OK, Json(response)).into_response()
                }
                Err(_) => {
                    // Fallback to generic response
                    let models = vec![ModelInfo {
                        id: "llama-model".to_string(),
                        object: "model".to_string(),
                        created: chrono::Utc::now().timestamp(),
                        owned_by: "llama.cpp".to_string(),
                    }];
                    let response = ModelsResponse {
                        object: "list".to_string(),
                        data: models,
                    };
                    (StatusCode::OK, Json(response)).into_response()
                }
            }
        }
        _ => {
            // llama.cpp not running or no model loaded
            let error = OpenAIErrorResponse {
                error: OpenAIError {
                    message: "No model currently loaded. Please start a model in Arandu first.".to_string(),
                    error_type: "no_model_loaded".to_string(),
                    code: Some("503".to_string()),
                },
            };
            (StatusCode::SERVICE_UNAVAILABLE, Json(error)).into_response()
        }
    }
}

async fn chat_completions(
    State(state): State<Arc<RwLock<ProxyState>>>,
    Json(request): Json<ChatCompletionRequest>,
) -> impl IntoResponse {
    // Check if llama.cpp server is reachable
    let health_url = format!("{}/health", state.read().await.llama_server_url);
    let health_client = reqwest::Client::new();
    
    match health_client.get(&health_url).timeout(Duration::from_secs(2)).send().await {
        Ok(resp) if resp.status().is_success() => {
            // Server is healthy, proceed
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
            ).into_response();
        }
    }

    // Check if streaming is requested
    let stream = request.stream.unwrap_or(false);
    
    if stream {
        return handle_streaming_completion(state, request).await.into_response();
    }
    
    // Handle non-streaming completion
    let state_guard = state.read().await;
    let client = &state_guard.llama_client;
    
    match client.chat_completion(&request).await {
        Ok(response) => {
            // llama.cpp returns OpenAI-compatible format, just pass it through
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(e) => {
            let error = json!({
                "error": {
                    "message": e,
                    "type": "api_error",
                    "code": "500"
                }
            });
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error)).into_response()
        }
    }
}

async fn audio_transcriptions(
    State(_state): State<Arc<RwLock<ProxyState>>>,
    Json(_request): Json<AudioTranscriptionRequest>,
) -> impl IntoResponse {
    // Placeholder - will be implemented in Task 7
    let response = AudioTranscriptionResponse {
        text: "Audio transcription not yet implemented.".to_string(),
    };
    
    Json(response)
}

async fn audio_speech(
    State(_state): State<Arc<RwLock<ProxyState>>>,
    Json(_request): Json<AudioSpeechRequest>,
) -> impl IntoResponse {
    // Placeholder - will be implemented in Task 7
    let error = OpenAIErrorResponse {
        error: OpenAIError {
            message: "Text-to-speech not yet implemented.".to_string(),
            error_type: "not_implemented".to_string(),
            code: Some("501".to_string()),
        },
    };
    
    (StatusCode::NOT_IMPLEMENTED, Json(error))
}

async fn image_generations(
    State(_state): State<Arc<RwLock<ProxyState>>>,
    Json(_request): Json<ImageGenerationRequest>,
) -> impl IntoResponse {
    // Placeholder - will be implemented in Task 8
    let error = OpenAIErrorResponse {
        error: OpenAIError {
            message: "Image generation not yet implemented.".to_string(),
            error_type: "not_implemented".to_string(),
            code: Some("501".to_string()),
        },
    };
    
    (StatusCode::NOT_IMPLEMENTED, Json(error))
}

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
