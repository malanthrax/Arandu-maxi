use crate::chat_models::*;
use crate::openai_types::ChatMessage as OpenAIChatMessage;
use chrono::Utc;
use std::path::PathBuf;
use tokio::fs;
use uuid::Uuid;

pub struct ChatManager {
    chats_dir: PathBuf,
    index_path: PathBuf,
}

impl ChatManager {
    pub fn new() -> Result<Self, String> {
        let home_dir = dirs::home_dir().ok_or_else(|| "Failed to get home directory".to_string())?;
        let arandu_dir = home_dir.join(".Arandu");
        let chats_dir = arandu_dir.join("chats");
        let index_path = chats_dir.join("index.json");

        Ok(Self {
            chats_dir,
            index_path,
        })
    }

    async fn ensure_chats_dir_exists(&self) -> Result<(), String> {
        fs::create_dir_all(&self.chats_dir)
            .await
            .map_err(|e| format!("Failed to create chats directory: {}", e))
    }

    async fn load_index(&self) -> Result<ChatIndex, String> {
        if !self.index_path.exists() {
            return Ok(ChatIndex::default());
        }

        let content = fs::read_to_string(&self.index_path)
            .await
            .map_err(|e| format!("Failed to read index file: {}", e))?;

        serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse index file: {}", e))
    }

    async fn save_index(&self, index: &ChatIndex) -> Result<(), String> {
        let content = serde_json::to_string_pretty(index)
            .map_err(|e| format!("Failed to serialize index: {}", e))?;

        fs::write(&self.index_path, content)
            .await
            .map_err(|e| format!("Failed to write index file: {}", e))
    }

    fn get_chat_file_path(&self, chat_id: &str) -> PathBuf {
        self.chats_dir.join(format!("{}.json", chat_id))
    }

    pub async fn create_chat(
        &self,
        model_path: String,
        model_name: String,
    ) -> Result<ChatSession, String> {
        self.ensure_chats_dir_exists().await?;

        let chat_id = Uuid::new_v4().to_string();
        let now = Utc::now();

        let session = ChatSession {
            id: chat_id.clone(),
            title: format!("Chat with {}", model_name),
            model_path: model_path.clone(),
            model_name: model_name.clone(),
            created_at: now,
            updated_at: now,
            messages: Vec::new(),
            parameters: ChatParameters::default(),
            message_count: 0,
        };

        self.save_chat_internal(&session).await?;

        let mut index = self.load_index().await?;
        index.chats.insert(chat_id.clone(), ChatIndexEntry::from(&session));
        index.version = 1;
        self.save_index(&index).await?;

        Ok(session)
    }

    pub async fn load_chat(&self, chat_id: String) -> Result<ChatSession, String> {
        let chat_file_path = self.get_chat_file_path(&chat_id);

        if !chat_file_path.exists() {
            return Err(format!("Chat session '{}' not found", chat_id));
        }

        let content = fs::read_to_string(&chat_file_path)
            .await
            .map_err(|e| format!("Failed to read chat file '{}': {}", chat_id, e))?;

        let session: ChatSession = serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse chat file '{}': {}", chat_id, e))?;

        Ok(session)
    }

    pub async fn save_chat(&self, session: ChatSession) -> Result<(), String> {
        self.save_chat_internal(&session).await?;

        let mut index = self.load_index().await?;
        index.chats.insert(session.id.clone(), ChatIndexEntry::from(&session));
        index.version = 1;
        self.save_index(&index).await?;

        Ok(())
    }

    async fn save_chat_internal(&self, session: &ChatSession) -> Result<(), String> {
        self.ensure_chats_dir_exists().await?;

        let chat_file_path = self.get_chat_file_path(&session.id);

        let content = serde_json::to_string_pretty(session)
            .map_err(|e| format!("Failed to serialize chat session '{}': {}", session.id, e))?;

        fs::write(&chat_file_path, content)
            .await
            .map_err(|e| format!("Failed to write chat file '{}': {}", session.id, e))
    }

    pub async fn list_chats(&self, model_path: Option<String>) -> Result<Vec<ChatSummary>, String> {
        let index = self.load_index().await?;

        let mut summaries: Vec<ChatSummary> = index
            .chats
            .values()
            .filter(|entry| {
                // If model_path is provided, only include chats for that model
                match &model_path {
                    Some(path) => entry.model_path == *path,
                    None => true,
                }
            })
            .map(|entry| ChatSummary {
                id: entry.id.clone(),
                title: entry.title.clone(),
                model_name: entry.model_name.clone(),
                model_path: entry.model_path.clone(),
                created_at: entry.created_at,
                updated_at: entry.updated_at,
                message_count: entry.message_count,
            })
            .collect();

        summaries.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));

        Ok(summaries)
    }

    pub async fn delete_chat(&self, chat_id: String) -> Result<(), String> {
        let chat_file_path = self.get_chat_file_path(&chat_id);

        if chat_file_path.exists() {
            fs::remove_file(&chat_file_path)
                .await
                .map_err(|e| format!("Failed to delete chat file '{}': {}", chat_id, e))?;
        }

        let mut index = self.load_index().await?;
        index.chats.remove(&chat_id);
        index.version = 1;
        self.save_index(&index).await?;

        Ok(())
    }

    pub async fn update_chat_title(
        &self,
        chat_id: String,
        title: String,
    ) -> Result<(), String> {
        let mut session = self.load_chat(chat_id.clone()).await?;
        session.title = title;
        session.updated_at = Utc::now();

        self.save_chat_internal(&session).await?;

        let mut index = self.load_index().await?;
        if let Some(entry) = index.chats.get_mut(&chat_id) {
            entry.title = session.title.clone();
            entry.updated_at = session.updated_at;
            index.version = 1;
            self.save_index(&index).await?;
        }

        Ok(())
    }

    pub async fn add_message(
        &self,
        chat_id: String,
        role: MessageRole,
        content: String,
    ) -> Result<ChatMessage, String> {
        let mut session = self.load_chat(chat_id.clone()).await?;

        let message = ChatMessage {
            id: Uuid::new_v4().to_string(),
            role,
            content,
            created_at: Utc::now(),
            metadata: None,
        };

        session.messages.push(message.clone());
        session.message_count = session.messages.len() as u32;
        session.updated_at = Utc::now();

        self.save_chat_internal(&session).await?;

        let mut index = self.load_index().await?;
        if let Some(entry) = index.chats.get_mut(&chat_id) {
            entry.message_count = session.message_count;
            entry.updated_at = session.updated_at;
            index.version = 1;
            self.save_index(&index).await?;
        }

        Ok(message)
    }

    pub async fn update_chat_parameters(
        &self,
        chat_id: String,
        parameters: ChatParameters,
    ) -> Result<(), String> {
        let mut session = self.load_chat(chat_id.clone()).await?;
        session.parameters = parameters;
        session.updated_at = Utc::now();

        self.save_chat(session).await
    }

    pub async fn generate_chat_title(
        &self,
        chat_id: String,
    ) -> Result<String, String> {
        let session = self.load_chat(chat_id.clone()).await?;

        let title = if session.messages.is_empty() {
            format!("Chat with {}", session.model_name)
        } else {
            let first_user_message = session
                .messages
                .iter()
                .find(|m| m.role == MessageRole::User)
                .map(|m| {
                    let content = m.content.trim();
                    if content.len() > 40 {
                        format!("{}...", &content[..40])
                    } else {
                        content.to_string()
                    }
                });

            match first_user_message {
                Some(msg) => msg,
                None => format!("Chat with {}", session.model_name),
            }
        };

        self.update_chat_title(chat_id, title.clone()).await?;

        Ok(title)
    }

    pub async fn send_message_to_llm(
        &self,
        chat_id: String,
        user_content: String,
        server_url: String,
    ) -> Result<String, String> {
        use crate::llama_client::LlamaClient;
        use crate::openai_types::ChatCompletionRequest;

        // 1. Load chat
        let mut session = self.load_chat(chat_id.clone()).await?;

        // 2. Add user message to chat
        let user_message = ChatMessage {
            id: Uuid::new_v4().to_string(),
            role: MessageRole::User,
            content: user_content.clone(),
            created_at: Utc::now(),
            metadata: None,
        };
        session.messages.push(user_message);
        session.message_count = session.messages.len() as u32;
        session.updated_at = Utc::now();

        // 3. Build messages array for API
        let mut messages: Vec<OpenAIChatMessage> = Vec::new();
        
        // Add system message if present
        if let Some(ref system_prompt) = session.parameters.system_prompt {
            if !system_prompt.is_empty() {
                messages.push(OpenAIChatMessage {
                    role: "system".to_string(),
                    content: system_prompt.clone(),
                });
            }
        }
        
        // Add conversation history
        for msg in &session.messages {
            let role = match msg.role {
                MessageRole::User => "user",
                MessageRole::Assistant => "assistant",
                MessageRole::System => "system",
            };
            messages.push(OpenAIChatMessage {
                role: role.to_string(),
                content: msg.content.clone(),
            });
        }

        // 4. Create LlamaClient with server_url
        let client = LlamaClient::new(server_url);

        // 5. Create request and call chat_completion
        let request = ChatCompletionRequest {
            model: session.model_name.clone(),
            messages,
            temperature: Some(session.parameters.temperature),
            max_tokens: Some(session.parameters.max_tokens as i32),
            stream: Some(false),
            top_p: Some(session.parameters.top_p),
            stop: None,
        };

        // 6. Call chat_completion
        let response = client.chat_completion(&request).await
            .map_err(|e| format!("LLM request failed: {}", e))?;

        // 7. Extract assistant's response
        let assistant_content = response
            .get("choices")
            .and_then(|choices| choices.as_array())
            .and_then(|choices| choices.first())
            .and_then(|choice| choice.get("message"))
            .and_then(|message| message.get("content"))
            .and_then(|content| content.as_str())
            .ok_or_else(|| "Failed to parse assistant response".to_string())?;

        let assistant_response = assistant_content.to_string();

        // 8. Add assistant response to chat
        let assistant_message = ChatMessage {
            id: Uuid::new_v4().to_string(),
            role: MessageRole::Assistant,
            content: assistant_response.clone(),
            created_at: Utc::now(),
            metadata: Some(MessageMetadata {
                tokens_used: None,
                model_response_time_ms: None,
                stop_reason: None,
            }),
        };
        session.messages.push(assistant_message);
        session.message_count = session.messages.len() as u32;
        session.updated_at = Utc::now();

        // 9. Save chat
        self.save_chat_internal(&session).await?;

        // 10. Update index
        let mut index = self.load_index().await?;
        if let Some(entry) = index.chats.get_mut(&chat_id) {
            entry.message_count = session.message_count;
            entry.updated_at = session.updated_at;
            index.version = 1;
            self.save_index(&index).await?;
        }

        // 11. Return assistant content
        Ok(assistant_response)
    }
}

impl Default for ChatManager {
    fn default() -> Self {
        Self::new().expect("Failed to create ChatManager")
    }
}
