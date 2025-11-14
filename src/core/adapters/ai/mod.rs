//! AI adapter implementations
//!
//! Contains unified interface for different AI providers (Ollama, Claude, Qwen, OpenAI)
//! This will be expanded in Phase 1: Core CLI & AI Adapter

use anyhow::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use crate::utils::config::SecureKey;
use crate::core::context_manager::ContextManager;

pub mod factory;
pub mod tracked;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AIProvider {
    Ollama,
    Claude,
    Qwen,
    OpenAI,
}

#[async_trait::async_trait]
pub trait AIProviderTrait: Send + Sync {
    async fn chat(&self, message: &str) -> Result<String>;
    async fn chat_with_context(&self, message: &str, workspace_path: Option<&str>) -> Result<String>;
}

use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KandilAI {
    provider: AIProvider,
    model: String,
    #[serde(skip)]
    client: Arc<Client>,
    base_url: String,
    /// Flag indicating if we should use hybrid (local fallback) mode
    use_hybrid_mode: bool,
}

impl KandilAI {
    pub fn new(provider: String, model: String) -> Result<Self> {
        let provider_enum = match provider.as_str() {
            "ollama" => AIProvider::Ollama,
            "claude" => AIProvider::Claude,
            "qwen" => AIProvider::Qwen,
            "openai" => AIProvider::OpenAI,
            _ => return Err(anyhow::anyhow!("Unsupported AI provider: {}", provider)),
        };

        let base_url = match &provider_enum {
            AIProvider::Ollama => "http://localhost:11434".to_string(),
            AIProvider::Claude => "https://api.anthropic.com".to_string(),
            AIProvider::Qwen => "https://dashscope.aliyuncs.com".to_string(),
            AIProvider::OpenAI => "https://api.openai.com".to_string(),
        };

        Ok(Self {
            provider: provider_enum,
            model,
            client: Arc::new(Client::new()),
            base_url,
            use_hybrid_mode: true, // Default to hybrid mode
        })
    }

    // Initialize the client after deserialization
    fn init_client(&mut self) {
        self.client = Arc::new(Client::new());
    }

    pub async fn chat(&self, message: &str) -> Result<String> {
        // For short/simple queries, try local model first
        if self.use_hybrid_mode && message.len() < 5000 && matches!(self.provider, AIProvider::Claude | AIProvider::OpenAI | AIProvider::Qwen) {
            // Try to use local model as fallback
            if let Ok(local_result) = self.ollama_chat(message).await {
                // Add a note about the local model being used
                return Ok(format!("(Local Model Response) {}", local_result));
            }
        }

        // Use the configured provider
        match &self.provider {
            AIProvider::Ollama => self.ollama_chat(message).await,
            AIProvider::Claude => self.claude_chat(message).await,
            AIProvider::Qwen => self.qwen_chat(message).await,
            AIProvider::OpenAI => self.openai_chat(message).await,
        }
    }

    /// Enhanced chat with context management
    pub async fn chat_with_context(&self, message: &str, workspace_path: Option<&str>) -> Result<String> {
        let enhanced_message = if let Some(path) = workspace_path {
            // Prepare context using the context manager
            if let Ok(context_manager) = ContextManager::new() {
                if let Ok(context) = context_manager.prepare_context(message, path) {
                    // Build enhanced prompt with relevant context
                    let mut enhanced_prompt = format!("Context from your project:\n");

                    for file in context.files.iter().take(5) { // Take top 5 most relevant files
                        enhanced_prompt.push_str(&format!("\nFile: {}\nContent: {}\n",
                            file.path,
                            file.content.chars().take(1000).collect::<String>() // Limit file content
                        ));
                    }

                    enhanced_prompt.push_str(&format!("\nUser Query: {}", message));
                    enhanced_prompt
                } else {
                    // If context preparation failed, use original message
                    message.to_string()
                }
            } else {
                // If context manager creation failed, use original message
                message.to_string()
            }
        } else {
            message.to_string()
        };

        self.chat(&enhanced_message).await
    }

    async fn ollama_chat(&self, message: &str) -> Result<String> {
        #[derive(Serialize)]
        struct OllamaRequest {
            model: String,
            prompt: String,
            stream: bool,
        }

        #[derive(Deserialize)]
        struct OllamaResponse {
            response: String,
            // In a real implementation, Ollama might provide token counts
        }

        let request = OllamaRequest {
            model: self.model.clone(),
            prompt: message.to_string(),
            stream: false,
        };

        let response = self
            .client
            .post(format!("{}/api/generate", self.base_url))
            .json(&request)
            .send()
            .await?;

        if response.status().is_success() {
            let result: OllamaResponse = response.json().await?;
            Ok(result.response)
        } else {
            Err(anyhow::anyhow!(
                "Ollama request failed with status: {}",
                response.status()
            ))
        }
    }

    async fn claude_chat(&self, message: &str) -> Result<String> {
        let api_key = SecureKey::load("claude")?.expose().to_string();
        crate::utils::rate_limit::check_limit(&api_key)?;

        #[derive(Serialize)]
        struct ClaudeRequest {
            model: String,
            prompt: String,
            max_tokens_to_sample: u32,
        }

        #[derive(Deserialize)]
        struct ClaudeResponse {
            completion: String,
        }

        let request = ClaudeRequest {
            model: self.model.clone(),
            prompt: format!("Human: {}\n\nAssistant:", message),
            max_tokens_to_sample: 1000,
        };

        let response = self
            .client
            .post(&format!("{}/v1/complete", self.base_url))
            .header("Content-Type", "application/json")
            .header("X-API-Key", api_key)
            .json(&request)
            .send()
            .await?;

        if response.status().is_success() {
            let result: ClaudeResponse = response.json().await?;
            Ok(result.completion.trim().to_string())
        } else {
            let status = response.status();
            let error_text = response.text().await?;
            Err(anyhow::anyhow!(
                "Claude request failed: {} - {}",
                status,
                error_text
            ))
        }
    }

    async fn qwen_chat(&self, message: &str) -> Result<String> {
        let api_key = SecureKey::load("qwen")?.expose().to_string();
        crate::utils::rate_limit::check_limit(&api_key)?;

        #[derive(Serialize)]
        struct QwenRequest {
            model: String,
            input: QwenInput,
            parameters: QwenParameters,
        }

        #[derive(Serialize)]
        struct QwenInput {
            prompt: String,
        }

        #[derive(Serialize)]
        struct QwenParameters {
            temperature: f32,
        }

        #[derive(Deserialize)]
        struct QwenResponse {
            output: QwenOutput,
        }

        #[derive(Deserialize)]
        struct QwenOutput {
            text: String,
        }

        let request = QwenRequest {
            model: self.model.clone(),
            input: QwenInput {
                prompt: message.to_string(),
            },
            parameters: QwenParameters {
                temperature: 0.7,
            },
        };

        let response = self
            .client
            .post(&format!("{}/api/v1/services/aigc/text-generation/generation", self.base_url))
            .header("Content-Type", "application/json")
            .header("Authorization", format!("Bearer {}", api_key))
            .json(&request)
            .send()
            .await?;

        if response.status().is_success() {
            let result: QwenResponse = response.json().await?;
            Ok(result.output.text.trim().to_string())
        } else {
            let status = response.status();
            let error_text = response.text().await?;
            Err(anyhow::anyhow!(
                "Qwen request failed: {} - {}",
                status,
                error_text
            ))
        }
    }

    async fn openai_chat(&self, message: &str) -> Result<String> {
        let api_key = SecureKey::load("openai")?.expose().to_string();
        crate::utils::rate_limit::check_limit(&api_key)?;

        #[derive(Serialize)]
        struct OpenAIRequest {
            model: String,
            messages: Vec<OpenAIMessage>,
            temperature: f32,
        }

        #[derive(Serialize, Deserialize)]
        struct OpenAIMessage {
            role: String,
            content: String,
        }

        #[derive(Deserialize)]
        struct OpenAIResponse {
            choices: Vec<OpenAIChoice>,
            usage: Option<OpenAIUsage>,
        }

        #[derive(Deserialize)]
        struct OpenAIChoice {
            message: OpenAIMessage,
        }

        #[derive(Deserialize)]
        struct OpenAIUsage {
            prompt_tokens: u32,
            completion_tokens: u32,
            total_tokens: u32,
        }

        let request = OpenAIRequest {
            model: self.model.clone(),
            messages: vec![
                OpenAIMessage {
                    role: "user".to_string(),
                    content: message.to_string(),
                }
            ],
            temperature: 0.7,
        };

        let response = self
            .client
            .post(&format!("{}/v1/chat/completions", self.base_url))
            .header("Content-Type", "application/json")
            .header("Authorization", format!("Bearer {}", api_key))
            .json(&request)
            .send()
            .await?;

        if response.status().is_success() {
            let result: OpenAIResponse = response.json().await?;

            // If we got usage information, we could track the cost here
            if let Some(usage) = result.usage {
                // In a real implementation, we would track the actual token usage
            }

            if let Some(choice) = result.choices.first() {
                Ok(choice.message.content.trim().to_string())
            } else {
                Err(anyhow::anyhow!("No choices returned from OpenAI"))
            }
        } else {
            let status = response.status();
            let error_text = response.text().await?;
            Err(anyhow::anyhow!(
                "OpenAI request failed: {} - {}",
                status,
                error_text
            ))
        }
    }
}

#[async_trait::async_trait]
impl AIProviderTrait for KandilAI {
    async fn chat(&self, message: &str) -> Result<String> {
        // Call the existing chat method
        self.chat(message).await
    }

    async fn chat_with_context(&self, message: &str, workspace_path: Option<&str>) -> Result<String> {
        // Call the existing chat_with_context method
        self.chat_with_context(message, workspace_path).await
    }
}
