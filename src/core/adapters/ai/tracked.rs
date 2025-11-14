//! AI adapter with cost tracking
//!
//! Wrapper around KandilAI that adds cost tracking functionality

use crate::core::adapters::ai::KandilAI;
use crate::utils::cost_tracking::CostTracker;
use anyhow::Result;
use std::sync::Arc;

pub struct TrackedAI {
    pub ai: Arc<KandilAI>,
    cost_tracker: Arc<CostTracker>,
}

impl TrackedAI {
    pub fn new(ai: Arc<KandilAI>, cost_tracker: Arc<CostTracker>) -> Self {
        Self { ai, cost_tracker }
    }

    pub async fn chat(&self, message: &str) -> Result<String> {
        // For now, just call the underlying AI and return the result
        // In a real implementation, we would track token usage and costs
        let response = self.ai.chat(message).await?;

        // Extract provider string for cost tracking
        let provider_str = match self.ai.provider {
            crate::core::adapters::ai::AIProvider::Ollama => "ollama",
            crate::core::adapters::ai::AIProvider::Claude => "claude",
            crate::core::adapters::ai::AIProvider::Qwen => "qwen",
            crate::core::adapters::ai::AIProvider::OpenAI => "openai",
        };

        // In a full implementation, we would estimate token counts from the message/response
        // and call self.cost_tracker.record_usage() with real values
        // For now, we'll just do a placeholder call
        self.cost_tracker.record_usage(
            provider_str,
            &self.ai.model,
            message.len() as u32,  // Placeholder - real token count needed
            response.len() as u32, // Placeholder - real token count needed
        );

        Ok(response)
    }

    pub async fn chat_with_context(&self, message: &str, workspace_path: Option<&str>) -> Result<String> {
        // Call the enhanced chat with context functionality
        let response = self.ai.chat_with_context(message, workspace_path).await?;

        // Extract provider string for cost tracking
        let provider_str = match self.ai.provider {
            crate::core::adapters::ai::AIProvider::Ollama => "ollama",
            crate::core::adapters::ai::AIProvider::Claude => "claude",
            crate::core::adapters::ai::AIProvider::Qwen => "qwen",
            crate::core::adapters::ai::AIProvider::OpenAI => "openai",
        };

        // In a full implementation, we would estimate token counts from the message/response
        // and call self.cost_tracker.record_usage() with real values
        // For now, we'll just do a placeholder call
        let enhanced_message = if let Some(path) = workspace_path {
            // Simulate the enhanced message for cost tracking
            format!("Context from your project:\nFile: example.rs\nContent: example content\n\nUser Query: {}", message)
        } else {
            message.to_string()
        };

        self.cost_tracker.record_usage(
            provider_str,
            &self.ai.model,
            enhanced_message.len() as u32,  // Placeholder - real token count needed
            response.len() as u32, // Placeholder - real token count needed
        );

        Ok(response)
    }

    pub fn get_provider(&self) -> String {
        match self.ai.provider {
            crate::core::adapters::ai::AIProvider::Ollama => "ollama".to_string(),
            crate::core::adapters::ai::AIProvider::Claude => "claude".to_string(),
            crate::core::adapters::ai::AIProvider::Qwen => "qwen".to_string(),
            crate::core::adapters::ai::AIProvider::OpenAI => "openai".to_string(),
        }
    }

    pub fn get_model(&self) -> &str {
        &self.ai.model
    }
}
