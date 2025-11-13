//! AI Provider Factory
//! 
//! Creates and manages different AI providers based on configuration

use anyhow::Result;
use crate::utils::config::{Config, SecureKey};
use crate::utils::cost_tracking::CostTracker;
use super::KandilAI;
use std::sync::Arc;

pub struct AIProviderFactory {
    cost_tracker: Arc<CostTracker>,
}

impl AIProviderFactory {
    pub fn new(_config: Config) -> Self {
        Self {
            cost_tracker: Arc::new(CostTracker::new()),
        }
    }

    pub fn create_ai(&self, provider: &str, model: &str) -> Result<KandilAI> {
        // In a more complete implementation, we would retrieve the API key securely
        // from the OS keyring and pass it to the KandilAI constructor if needed
        KandilAI::new(provider.to_string(), model.to_string())
    }
    
    pub fn create_ai_with_auth(&self, provider: &str, model: &str) -> Result<KandilAI> {
        // This would be used when we need to create an AI instance with authentication
        // For cloud providers, we would load the API key securely here
        if provider != "ollama" {
            // Verify that API key exists in keyring
            let _api_key = SecureKey::load(provider)?;
        }
        
        KandilAI::new(provider.to_string(), model.to_string())
    }
    
    pub fn get_cost_tracker(&self) -> Arc<CostTracker> {
        self.cost_tracker.clone()
    }
}