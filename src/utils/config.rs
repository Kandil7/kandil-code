//! Configuration management for Kandil Code
//! 
//! Handles secure storage and retrieval of API keys and other configuration settings.

use anyhow::Result;
use secrecy::{Secret, ExposeSecret};
use keyring::Entry;
use serde::{Deserialize, Serialize};

pub struct SecureKey {
    inner: Secret<String>,
    provider: String,
}

impl SecureKey {
    pub fn load(provider: &str) -> Result<Self> {
        let entry = Entry::new("kandil", provider)?;
        let key = entry.get_password()
            .map_err(|_| anyhow::anyhow!("No key for {}. Run: kandil config set-key {}", provider, provider))?;

        Ok(Self {
            inner: Secret::new(key),
            provider: provider.to_string(),
        })
    }

    pub fn expose(&self) -> &str {
        self.inner.expose_secret()
    }

    pub fn save(provider: &str, key: &str) -> Result<()> {
        let entry = Entry::new("kandil", provider)?;
        entry.set_password(key)?;
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub ai_provider: String,
    pub ai_model: String,
}

impl Config {
    pub fn load() -> Result<Self> {
        // In a real implementation, this would load from a config file
        // For now, return default configuration
        Ok(Config {
            ai_provider: std::env::var("KANDIL_AI_PROVIDER")
                .unwrap_or_else(|_| "ollama".to_string()),
            ai_model: std::env::var("KANDIL_AI_MODEL")
                .unwrap_or_else(|_| "llama3:70b".to_string()),
        })
    }
    
    pub fn save(&self) -> Result<()> {
        // In a real implementation, this would save to a config file
        Ok(())
    }
}