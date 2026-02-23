use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum MessageRole {
    User,
    Assistant,
    System,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub id: String,
    pub role: MessageRole,
    pub content: String,
    pub created_at: DateTime<Utc>,
    pub metadata: Option<MessageMetadata>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MessageMetadata {
    pub tokens_used: Option<u32>,
    pub model_response_time_ms: Option<u64>,
    pub stop_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatSession {
    pub id: String,
    pub title: String,
    pub model_path: String,
    pub model_name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub messages: Vec<ChatMessage>,
    pub parameters: ChatParameters,
    pub message_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatParameters {
    pub temperature: f32,
    pub top_p: f32,
    pub max_tokens: u32,
    pub system_prompt: Option<String>,
    pub context_length: Option<u32>,
}

impl Default for ChatParameters {
    fn default() -> Self {
        Self {
            temperature: 0.7,
            top_p: 0.9,
            max_tokens: 4096,
            system_prompt: None,
            context_length: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatSummary {
    pub id: String,
    pub title: String,
    pub model_name: String,
    pub model_path: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub message_count: u32,
}

impl From<&ChatSession> for ChatSummary {
    fn from(session: &ChatSession) -> Self {
        Self {
            id: session.id.clone(),
            title: session.title.clone(),
            model_name: session.model_name.clone(),
            model_path: session.model_path.clone(),
            created_at: session.created_at,
            updated_at: session.updated_at,
            message_count: session.message_count,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ChatIndex {
    pub chats: HashMap<String, ChatIndexEntry>,
    pub version: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatIndexEntry {
    pub id: String,
    pub title: String,
    pub model_name: String,
    pub model_path: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub message_count: u32,
    pub file_path: String,
}

impl From<&ChatSession> for ChatIndexEntry {
    fn from(session: &ChatSession) -> Self {
        Self {
            id: session.id.clone(),
            title: session.title.clone(),
            model_name: session.model_name.clone(),
            model_path: session.model_path.clone(),
            created_at: session.created_at,
            updated_at: session.updated_at,
            message_count: session.message_count,
            file_path: format!("{}.json", session.id),
        }
    }
}
