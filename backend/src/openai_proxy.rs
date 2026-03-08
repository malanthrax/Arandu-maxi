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
use tracing::info;
use crate::openai_types::{
    ChatCompletionRequest, AudioTranscriptionRequest, AudioTranscriptionResponse,
    AudioSpeechRequest, ImageGenerationRequest,
    ModelInfo, ModelsResponse, OpenAIError, OpenAIErrorResponse
};
use crate::llama_client::LlamaClient;
use crate::AppState;
use crate::models::{ActiveModel, ModelStatus, ProcessStatus};

fn normalize_model_path(path: &str) -> String {
    path.replace('\\', "/").to_lowercase()
}

/// OpenAI-compatible API proxy server
#[derive(Debug)]
pub struct ProxyServer {
    llama_server_url: String,
    proxy_port: u16,
    shutdown_tx: Option<tokio::sync::mpsc::Sender<()>>,
    models_directories: Vec<String>,
}

impl ProxyServer {
    pub fn new(llama_server_host: String, llama_server_port: u16, proxy_port: u16, models_directories: Vec<String>) -> Self {
        Self {
            llama_server_url: format!("http://{}:{}", llama_server_host, llama_server_port),
            proxy_port,
            shutdown_tx: None,
            models_directories,
        }
    }

    pub async fn start(&mut self, app_state: Arc<AppState>) -> Result<(), String> {
        // Configure CORS to allow all origins (needed for cross-LAN access)
        let cors = CorsLayer::new()
            .allow_origin(Any)
            .allow_methods(Any)
            .allow_headers(Any);

        let models_dirs = self.models_directories.clone();

        let app = Router::new()
            .route("/v1/models", get(list_models))
            .route("/v1/models/arandu", get(list_models_arandu))
            .route("/v1/chat/completions", post(chat_completions))
            .route("/v1/audio/transcriptions", post(audio_transcriptions))
            .route("/v1/audio/speech", post(audio_speech))
            .route("/v1/images/generations", post(image_generations))
            .route("/health", get(health_check))

            .route("/api/models/launch", post(launch_model))
            .route("/api/models/stop", post(stop_model))
            .route("/api/models/active", get(list_active_models))

            .layer(cors)
            .with_state(Arc::new(RwLock::new(ProxyState {
                llama_server_url: self.llama_server_url.clone(),
                llama_client: LlamaClient::new(self.llama_server_url.clone()),
                models_directories: models_dirs,
                app_state,
            })));

        let addr = SocketAddr::from(([0, 0, 0, 0], self.proxy_port));
        
        let (shutdown_tx, mut shutdown_rx) = tokio::sync::mpsc::channel::<()>(1);
        self.shutdown_tx = Some(shutdown_tx);

        let listener = tokio::net::TcpListener::bind(addr)
            .await
            .map_err(|e| format!("Failed to bind proxy server: {}", e))?;

        println!("OpenAI proxy server starting on {}", addr);
        
        // Log successful startup
        info!("OpenAI proxy server bound to {} and ready to accept connections", addr);

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
    pub models_directories: Vec<String>,
    pub app_state: Arc<AppState>,
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
                        size_gb: None,
                        quantization: None,
                        architecture: None,
                        date: None,
                        path: None,
                        has_custom_launch_config: None,
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
                        size_gb: None,
                        quantization: None,
                        architecture: None,
                        date: None,
                        path: None,
                        has_custom_launch_config: None,
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

// New endpoint for Arandu network discovery - returns all available models with full metadata
async fn list_models_arandu(
    State(state): State<Arc<RwLock<ProxyState>>>,
) -> impl IntoResponse {
    let state_guard = state.read().await;
    let model_directories = state_guard.models_directories.clone();
    let app_state = state_guard.app_state.clone();
    drop(state_guard);

    let fake_enabled = {
        let flag = app_state.fake_discovery_model_enabled.lock().await;
        *flag
    };

    // Scan all model directories
    match crate::scanner::scan_models(&model_directories).await {
        Ok(scanned_models) => {
            // Convert scanned ModelInfo to OpenAI ModelInfo format with Arandu extensions
            let mut models: Vec<ModelInfo> = scanned_models
                .into_iter()
                .map(|model| ModelInfo {
                    id: model.name.clone(),
                    object: "model".to_string(),
                    created: model.date,
                    owned_by: "arandu".to_string(),
                    size_gb: Some(model.size_gb),
                    quantization: Some(model.quantization),
                    architecture: Some(model.architecture),
                    date: Some(model.date),
                    path: Some(model.path.clone()),
                    has_custom_launch_config: None,
                })
                .collect();

            if fake_enabled {
                models.push(ModelInfo {
                    id: "arandu-test-fake-model.gguf".to_string(),
                    object: "model".to_string(),
                    created: chrono::Utc::now().timestamp(),
                    owned_by: "arandu-test".to_string(),
                    size_gb: Some(0.01),
                    quantization: Some("Q4_TEST".to_string()),
                    architecture: Some("test".to_string()),
                    date: Some(chrono::Utc::now().timestamp()),
                    path: Some("__ARANDU_FAKE_MODEL__".to_string()),
                    has_custom_launch_config: Some(false),
                });
            }

            let response = ModelsResponse {
                object: "list".to_string(),
                data: models,
            };
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(e) => {
            eprintln!("Error scanning models for /v1/models/arandu: {}", e);
            // Return empty list on error
            let response = ModelsResponse {
                object: "list".to_string(),
                data: vec![],
            };
            (StatusCode::OK, Json(response)).into_response()
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

// ============== REMOTE MODEL LAUNCH ENDPOINTS ==============

async fn launch_model(
    State(state): State<Arc<RwLock<ProxyState>>>,
    Json(request): Json<crate::models::RemoteLaunchRequest>,
) -> Json<crate::models::RemoteLaunchResponse> {
    use crate::models::RemoteLaunchResponse;

    let state_guard = state.read().await;
    let app_state = state_guard.app_state.clone();
    let model_directories = state_guard.models_directories.clone();
    drop(state_guard);

    let requested_path = request.model_path;
    let requested_norm = normalize_model_path(&requested_path);

    let scanned_models = match crate::scanner::scan_models(&model_directories).await {
        Ok(models) => models,
        Err(e) => {
            return Json(RemoteLaunchResponse {
                success: false,
                message: format!("Failed to enumerate server model library: {}", e),
                process_id: None,
                server_host: None,
                server_port: None,
            });
        }
    };

    let canonical_model_path = match scanned_models
        .iter()
        .find(|model| normalize_model_path(&model.path) == requested_norm)
    {
        Some(model) => model.path.clone(),
        None => {
            return Json(RemoteLaunchResponse {
                success: false,
                message: "Requested model is not in this server's configured model library".to_string(),
                process_id: None,
                server_host: None,
                server_port: None,
            });
        }
    };

    if let Err(e) = tokio::fs::File::open(&canonical_model_path).await {
        return Json(RemoteLaunchResponse {
            success: false,
            message: format!("Server cannot read requested model file: {}", e),
            process_id: None,
            server_host: None,
            server_port: None,
        });
    }

    let existing_active = {
        let active_models = app_state.active_models.lock().await;
        active_models
            .values()
            .find(|active| normalize_model_path(&active.model_path) == requested_norm)
            .cloned()
    };

    if let Some(existing) = existing_active {
        let is_running = {
            let running = app_state.running_processes.lock().await;
            running
                .get(&existing.process_id)
                .map(|proc| matches!(proc.status, ProcessStatus::Starting | ProcessStatus::Running))
                .unwrap_or(false)
        };

        if is_running {
            return Json(RemoteLaunchResponse {
                success: true,
                message: "Model already loaded on server".to_string(),
                process_id: Some(existing.process_id),
                server_host: Some(existing.server_host),
                server_port: Some(existing.server_port),
            });
        }

        let mut active_models = app_state.active_models.lock().await;
        active_models.remove(&existing.process_id);
    }

    // Bind to all interfaces so the requesting remote client can connect
    match crate::process::launch_model_server(canonical_model_path.clone(), &app_state, Some("0.0.0.0".to_string())).await {
        Ok(launch_result) => {
            let model_name = std::path::Path::new(&canonical_model_path)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or(&canonical_model_path)
                .to_string();

            let active_model = ActiveModel {
                process_id: launch_result.process_id.clone(),
                model_path: canonical_model_path,
                model_name,
                host: launch_result.server_host.clone(),
                port: launch_result.server_port,
                server_host: launch_result.server_host.clone(),
                server_port: launch_result.server_port,
                status: ModelStatus::Ready,
                launched_at: chrono::Utc::now(),
            };

            let app_state_for_insert = app_state.clone();
            tokio::spawn(async move {
                let mut active_models = app_state_for_insert.active_models.lock().await;
                active_models.insert(active_model.process_id.clone(), active_model);
            });

            Json(RemoteLaunchResponse {
                success: true,
                message: "Model launched successfully".to_string(),
                process_id: Some(launch_result.process_id),
                server_host: Some(launch_result.server_host),
                server_port: Some(launch_result.server_port),
            })
        }
        Err(e) => {
            Json(RemoteLaunchResponse {
                success: false,
                message: format!("Failed to launch model: {}", e),
                process_id: None,
                server_host: None,
                server_port: None,
            })
        }
    }
}

async fn stop_model(
    State(state): State<Arc<RwLock<ProxyState>>>,
    Json(request): Json<crate::models::RemoteStopRequest>,
) -> impl IntoResponse {
    use crate::models::RemoteStopResponse;

    let state_guard = state.read().await;
    let app_state = state_guard.app_state.clone();
    let process_id_for_remove = request.process_id.clone();
    drop(state_guard);

    let _ = crate::process::terminate_process(request.process_id.clone(), &app_state).await;

    let mut active_models = app_state.active_models.lock().await;
    active_models.remove(&process_id_for_remove);

    Json(RemoteStopResponse {
        success: true,
        message: "Model stopped successfully".to_string(),
    })
}

async fn list_active_models(
    State(state): State<Arc<RwLock<ProxyState>>>,
) -> impl IntoResponse {
    use crate::models::RemoteActiveModelsResponse;

    let state_guard = state.read().await;
    let app_state = state_guard.app_state.clone();
    drop(state_guard);

    let running_snapshot = {
        let running = app_state.running_processes.lock().await;
        running.clone()
    };

    let mut active_models = app_state.active_models.lock().await;
    active_models.retain(|process_id, model| {
        if let Some(process) = running_snapshot.get(process_id) {
            if model.server_host.is_empty() {
                model.server_host = process.host.clone();
            }
            if model.server_port == 0 {
                model.server_port = process.port;
            }

            match process.status {
                ProcessStatus::Starting => {
                    model.status = ModelStatus::Starting;
                    true
                }
                ProcessStatus::Running => {
                    model.status = ModelStatus::Ready;
                    true
                }
                ProcessStatus::Stopped | ProcessStatus::Failed => false,
            }
        } else {
            false
        }
    });

    let models: Vec<ActiveModel> = active_models.values().cloned().collect();

    Json(RemoteActiveModelsResponse {
        success: true,
        models,
    })
}
