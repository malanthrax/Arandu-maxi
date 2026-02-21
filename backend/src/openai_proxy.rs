use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::{get, post},
    Router,
};
use serde_json::json;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::RwLock;
use crate::openai_types::*;

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
        let app = Router::new()
            .route("/v1/models", get(list_models))
            .route("/v1/chat/completions", post(chat_completions))
            .route("/v1/audio/transcriptions", post(audio_transcriptions))
            .route("/v1/audio/speech", post(audio_speech))
            .route("/v1/images/generations", post(image_generations))
            .route("/health", get(health_check))
            .with_state(Arc::new(RwLock::new(ProxyState {
                llama_server_url: self.llama_server_url.clone(),
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
                .unwrap();
        });

        Ok(())
    }

    pub async fn stop(&mut self) {
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(()).await;
        }
    }
}

pub struct ProxyState {
    pub llama_server_url: String,
}

// ============== HANDLER FUNCTIONS ==============

async fn health_check() -> impl IntoResponse {
    Json(json!({"status": "healthy"}))
}

async fn list_models() -> impl IntoResponse {
    let models = vec![
        ModelInfo {
            id: "local-llama".to_string(),
            object: "model".to_string(),
            created: chrono::Utc::now().timestamp(),
            owned_by: "arandu".to_string(),
        }
    ];

    let response = ModelsResponse {
        object: "list".to_string(),
        data: models,
    };

    Json(response)
}

async fn chat_completions(
    State(_state): State<Arc<RwLock<ProxyState>>>,
    Json(_request): Json<ChatCompletionRequest>,
) -> impl IntoResponse {
    // Placeholder - will be implemented in Task 6
    let response = ChatCompletionResponse {
        id: format!("chatcmpl-{}", uuid::Uuid::new_v4()),
        object: "chat.completion".to_string(),
        created: chrono::Utc::now().timestamp(),
        model: "local-llama".to_string(),
        choices: vec![ChatCompletionChoice {
            index: 0,
            message: ChatMessage {
                role: "assistant".to_string(),
                content: "Chat completion not yet implemented.".to_string(),
            },
            finish_reason: "stop".to_string(),
        }],
        usage: Usage {
            prompt_tokens: 10,
            completion_tokens: 20,
            total_tokens: 30,
        },
    };

    Json(response)
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
