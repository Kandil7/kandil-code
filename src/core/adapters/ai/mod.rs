//! AI adapter implementations
//! 
//! Contains unified interface for different AI providers (Ollama, Claude, Qwen, OpenAI)
//! This will be expanded in Phase 1: Core CLI & AI Adapter

use anyhow::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use crate::utils::config::SecureKey;

pub mod factory;
pub mod tracked;

#[derive(Debug, Clone)]
pub enum AIProvider {
    Ollama,
    Claude,
    Qwen,
    OpenAI,
}

#[derive(Debug, Clone)]
pub struct KandilAI {
    provider: AIProvider,
    model: String,
    client: Client,
    base_url: String,
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
            client: Client::new(),
            base_url,
        })
    }

    pub async fn chat(&self, message: &str) -> Result<String> {
        match &self.provider {
            AIProvider::Ollama => self.ollama_chat(message).await,
            AIProvider::Claude => self.claude_chat(message).await,
            AIProvider::Qwen => self.qwen_chat(message).await,
            AIProvider::OpenAI => self.openai_chat(message).await,
        }
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
            let error_text = response.text().await?;
            Err(anyhow::anyhow!(
                "Claude request failed: {} - {}",
                response.status(),
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
            let error_text = response.text().await?;
            Err(anyhow::anyhow!(
                "Qwen request failed: {} - {}",
                response.status(),
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

        #[derive(Serialize)]
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
            let error_text = response.text().await?;
            Err(anyhow::anyhow!(
                "OpenAI request failed: {} - {}",
                response.status(),
                error_text
            ))
        }
    }
}
