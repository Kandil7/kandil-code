//! AI adapter implementations
//!
//! Contains unified interface for different AI providers (Ollama, Claude, Qwen, OpenAI)
//! This will be expanded in Phase 1: Core CLI & AI Adapter

use crate::adapters::windows;
use crate::core::context_manager::ContextManager;
use crate::utils::config::SecureKey;
use anyhow::{Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use crate::monitoring::circuit_breaker::CircuitBreaker;

pub mod factory;
pub mod tracked;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AIProvider {
    Ollama,
    Claude,
    Qwen,
    OpenAI,
    LmStudio,
    Gpt4All,
    FoundryLocal,
}

#[derive(Serialize, Deserialize)]
struct OpenAIMessage {
    role: String,
    content: String,
}

#[derive(Serialize)]
struct OpenAIChatRequest {
    model: String,
    messages: Vec<OpenAIMessage>,
    temperature: f32,
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

#[derive(Deserialize)]
struct OpenAIChatResponse {
    choices: Vec<OpenAIChoice>,
    usage: Option<OpenAIUsage>,
}

#[async_trait::async_trait]
pub trait AIProviderTrait: Send + Sync {
    async fn chat(&self, message: &str) -> Result<String>;
    async fn chat_with_context(
        &self,
        message: &str,
        workspace_path: Option<&str>,
    ) -> Result<String>;
}

use std::{env, sync::Arc};

#[derive(Clone)]
pub struct KandilAI {
    provider: AIProvider,
    model: String,
    client: Arc<Client>,
    base_url: String,
    /// Flag indicating if we should use hybrid (local fallback) mode
    use_hybrid_mode: bool,
    breaker: Arc<CircuitBreaker>,
}

impl KandilAI {
    pub fn new(provider: String, model: String) -> Result<Self> {
        let provider_enum = match provider.as_str() {
            "ollama" => AIProvider::Ollama,
            "claude" => AIProvider::Claude,
            "qwen" => AIProvider::Qwen,
            "openai" => AIProvider::OpenAI,
            "lmstudio" => AIProvider::LmStudio,
            "gpt4all" => AIProvider::Gpt4All,
            "foundry" | "foundry_local" => AIProvider::FoundryLocal,
            _ => return Err(anyhow::anyhow!("Unsupported AI provider: {}", provider)),
        };

        let base_url = match &provider_enum {
            AIProvider::Ollama => windows::preferred_ollama_endpoint(),
            AIProvider::Claude => "https://api.anthropic.com".to_string(),
            AIProvider::Qwen => "https://dashscope.aliyuncs.com".to_string(),
            AIProvider::OpenAI => "https://api.openai.com".to_string(),
            AIProvider::LmStudio => "http://localhost:1234".to_string(),
            AIProvider::Gpt4All => "http://localhost:4891".to_string(),
            AIProvider::FoundryLocal => env::var("FOUNDRY_LOCAL_ENDPOINT")
                .unwrap_or_else(|_| "http://localhost:5001".to_string()),
        };

        let threshold = std::env::var("KANDIL_CIRCUIT_THRESHOLD")
            .ok()
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(3);
        let timeout_ms = std::env::var("KANDIL_CIRCUIT_TIMEOUT_MS")
            .ok()
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(2000);
        let breaker = Arc::new(CircuitBreaker::new(threshold, std::time::Duration::from_millis(timeout_ms)));

        Ok(Self {
            provider: provider_enum,
            model,
            client: Arc::new(Client::new()),
            base_url,
            use_hybrid_mode: true, // Default to hybrid mode
            breaker,
        })
    }

    // Initialize the client after deserialization
    fn init_client(&mut self) {
        self.client = Arc::new(Client::new());
    }

    pub fn provider_name(&self) -> &'static str {
        match self.provider {
            AIProvider::Ollama => "ollama",
            AIProvider::Claude => "claude",
            AIProvider::Qwen => "qwen",
            AIProvider::OpenAI => "openai",
            AIProvider::LmStudio => "lmstudio",
            AIProvider::Gpt4All => "gpt4all",
            AIProvider::FoundryLocal => "foundry",
        }
    }

    pub fn model_name(&self) -> &str {
        &self.model
    }

    pub async fn chat(&self, message: &str) -> Result<String> {
        // For short/simple queries, try local model first
        if self.use_hybrid_mode
            && message.len() < 5000
            && matches!(
                self.provider,
                AIProvider::Claude | AIProvider::OpenAI | AIProvider::Qwen
            )
        {
            // Try to use local model as fallback
            if let Ok(local_result) = self.ollama_chat(message).await {
                // Add a note about the local model being used
                return Ok(format!("(Local Model Response) {}", local_result));
            }
        }

        if self.breaker.is_open() {
            return Err(anyhow::anyhow!("Circuit breaker open for provider {}", self.provider_name()));
        }

        // Use the configured provider and update breaker
        let result = match &self.provider {
            AIProvider::Ollama => self.ollama_chat(message).await,
            AIProvider::Claude => self.claude_chat(message).await,
            AIProvider::Qwen => self.qwen_chat(message).await,
            AIProvider::OpenAI => self.openai_chat(message).await,
            AIProvider::LmStudio => self.lmstudio_chat(message).await,
            AIProvider::Gpt4All => self.gpt4all_chat(message).await,
            AIProvider::FoundryLocal => self.foundry_local_chat(message).await,
        };

        match &result {
            Ok(_) => self.breaker.record_success(),
            Err(_) => self.breaker.record_failure(),
        }

        result
    }

    /// Enhanced chat with context management
    pub async fn chat_with_context(
        &self,
        message: &str,
        workspace_path: Option<&str>,
    ) -> Result<String> {
        let enhanced_message = if let Some(path) = workspace_path {
            // Prepare context using the context manager
            if let Ok(context_manager) = ContextManager::new() {
                if let Ok(context) = context_manager.prepare_context(message, path) {
                    // Build enhanced prompt with relevant context
                    let mut enhanced_prompt = format!("Context from your project:\n");

                    for file in context.files.iter().take(5) {
                        // Take top 5 most relevant files
                        enhanced_prompt.push_str(&format!(
                            "\nFile: {}\nContent: {}\n",
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
            parameters: QwenParameters { temperature: 0.7 },
        };

        let response = self
            .client
            .post(&format!(
                "{}/api/v1/services/aigc/text-generation/generation",
                self.base_url
            ))
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
        self.openai_style_chat(
            message,
            "/v1/chat/completions",
            Some(format!("Bearer {}", api_key)),
        )
        .await
    }

    async fn lmstudio_chat(&self, message: &str) -> Result<String> {
        let api_key = SecureKey::load("lmstudio")
            .context(
                "Missing LM Studio API key. Set one via `kandil config set-key lmstudio <key>`.",
            )?
            .expose()
            .to_string();
        self.openai_style_chat(
            message,
            "/v1/chat/completions",
            Some(format!("Bearer {}", api_key)),
        )
        .await
    }

    async fn gpt4all_chat(&self, message: &str) -> Result<String> {
        self.openai_style_chat(message, "/v1/chat/completions", None)
            .await
    }

    async fn foundry_local_chat(&self, message: &str) -> Result<String> {
        let auth_header = SecureKey::load("foundry")
            .ok()
            .map(|key| format!("Bearer {}", key.expose()));
        self.openai_style_chat(message, "/v1/chat/completions", auth_header)
            .await
    }

    async fn openai_style_chat(
        &self,
        message: &str,
        relative_path: &str,
        auth_header: Option<String>,
    ) -> Result<String> {
        let request = OpenAIChatRequest {
            model: self.model.clone(),
            messages: vec![OpenAIMessage {
                role: "user".to_string(),
                content: message.to_string(),
            }],
            temperature: 0.7,
        };

        let mut req = self
            .client
            .post(format!("{}{}", self.base_url, relative_path))
            .header("Content-Type", "application/json")
            .json(&request);

        if let Some(header) = auth_header {
            req = req.header("Authorization", header);
        }

        let response = req.send().await?;

        if response.status().is_success() {
            let result: OpenAIChatResponse = response.json().await?;

            if let Some(usage) = result.usage {
                let _ = usage.total_tokens; // placeholder for future tracking
            }

            if let Some(choice) = result.choices.first() {
                Ok(choice.message.content.trim().to_string())
            } else {
                Err(anyhow::anyhow!(
                    "No choices returned from {}",
                    self.base_url
                ))
            }
        } else {
            let status = response.status();
            let error_text = response.text().await?;
            Err(anyhow::anyhow!(
                "Request to {}{} failed: {} - {}",
                self.base_url,
                relative_path,
                status,
                error_text
            ))
        }
    }
}

impl std::fmt::Debug for KandilAI {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("KandilAI")
            .field("provider", &self.provider_name())
            .field("model", &self.model)
            .finish()
    }
}

#[async_trait::async_trait]
impl AIProviderTrait for KandilAI {
    async fn chat(&self, message: &str) -> Result<String> {
        // Call the existing chat method
        self.chat(message).await
    }

    async fn chat_with_context(
        &self,
        message: &str,
        workspace_path: Option<&str>,
    ) -> Result<String> {
        // Call the existing chat_with_context method
        self.chat_with_context(message, workspace_path).await
    }
}
