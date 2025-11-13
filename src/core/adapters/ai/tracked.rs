//! AI adapter with cost tracking
//! 
//! Wrapper around KandilAI that adds cost tracking functionality

use anyhow::Result;
use std::sync::Arc;
use crate::core::adapters::ai::KandilAI;
use crate::utils::cost_tracking::CostTracker;

pub struct TrackedAI {
    ai: KandilAI,
    cost_tracker: Arc<CostTracker>,
}

impl TrackedAI {
    pub fn new(ai: KandilAI, cost_tracker: Arc<CostTracker>) -> Self {
        Self {
            ai,
            cost_tracker,
        }
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
            response.len() as u32  // Placeholder - real token count needed
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