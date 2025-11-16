//! Universal model registry implementation.
//!
//! Provides a simple in-memory registry that combines curated model profiles
//! with user supplied entries. The registry acts as the single source of
//! truth for routing decisions referenced throughout the enhance_model plan.

use crate::models::catalog::{ModelSpec, MODEL_CATALOG};
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::RwLock;

lazy_static! {
    static ref GLOBAL_REGISTRY: UniversalModelRegistry = UniversalModelRegistry::new();
}

/// Enumerates the major model providers supported by Kandil Code.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ProviderKind {
    Ollama,
    QwenCloud,
    Anthropic,
    Gemini,
    LocalBridge,
    Custom(String),
}

impl ProviderKind {
    fn alias_key(&self) -> String {
        match self {
            ProviderKind::Ollama => "ollama".to_string(),
            ProviderKind::QwenCloud => "qwen".to_string(),
            ProviderKind::Anthropic => "anthropic".to_string(),
            ProviderKind::Gemini => "gemini".to_string(),
            ProviderKind::LocalBridge => "local".to_string(),
            ProviderKind::Custom(value) => value.to_lowercase(),
        }
    }
}

/// Resource expectations for a model profile.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelResources {
    pub size_gb: f64,
    pub min_system_ram_gb: u64,
    pub min_vram_gb: Option<u64>,
}

/// Canonical metadata describing a model entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelProfile {
    pub name: String,
    pub provider: ProviderKind,
    pub description: String,
    pub is_local: bool,
    pub api_key_required: bool,
    pub context_window: usize,
    pub default_endpoint: Option<String>,
    pub resources: ModelResources,
    pub tags: Vec<String>,
}

impl From<&ModelSpec> for ModelProfile {
    fn from(spec: &ModelSpec) -> Self {
        ModelProfile {
            name: spec.name.to_string(),
            provider: ProviderKind::Ollama,
            description: spec.description.to_string(),
            is_local: true,
            api_key_required: false,
            context_window: spec.context_sizes.iter().max().copied().unwrap_or(4096),
            default_endpoint: Some("http://localhost:11434".to_string()),
            resources: ModelResources {
                size_gb: spec.size_gb,
                min_system_ram_gb: spec.ram_required_gb,
                min_vram_gb: spec.gpu_vram_min,
            },
            tags: vec!["gguf".to_string(), "local".to_string()],
        }
    }
}

/// Thread-safe registry that stores model metadata and allows runtime lookups.
pub struct UniversalModelRegistry {
    builtins: HashMap<String, ModelProfile>,
    aliases: RwLock<HashMap<String, String>>,
    custom: RwLock<HashMap<String, ModelProfile>>,
}

impl UniversalModelRegistry {
    fn new() -> Self {
        let mut builtins = HashMap::new();
        let mut aliases = HashMap::new();

        for spec in MODEL_CATALOG.iter() {
            let profile: ModelProfile = spec.into();
            aliases.insert(profile.name.to_lowercase(), profile.name.clone());
            for tag in &profile.tags {
                aliases.insert(
                    format!("{}:{}", profile.name.to_lowercase(), tag),
                    profile.name.clone(),
                );
            }
            builtins.insert(profile.name.clone(), profile);
        }

        Self {
            builtins,
            aliases: RwLock::new(aliases),
            custom: RwLock::new(HashMap::new()),
        }
    }

    /// Returns the global registry instance.
    pub fn global() -> &'static Self {
        &GLOBAL_REGISTRY
    }

    /// Lists all available model profiles (built-ins + custom).
    pub fn list_profiles(&self) -> Vec<ModelProfile> {
        let mut profiles: Vec<ModelProfile> = self.builtins.values().cloned().collect();
        if let Ok(custom) = self.custom.read() {
            profiles.extend(custom.values().cloned());
        }
        profiles.sort_by(|a, b| a.name.cmp(&b.name));
        profiles
    }

    /// Retrieves a profile by name or alias.
    pub fn get_profile(&self, name: &str) -> Option<ModelProfile> {
        let key = name.trim().to_lowercase();
        if let Some(actual) = self
            .aliases
            .read()
            .ok()
            .and_then(|aliases| aliases.get(&key).cloned())
        {
            if let Some(profile) = self.builtins.get(&actual) {
                return Some(profile.clone());
            }
        }

        if let Some(profile) = self.builtins.get(name) {
            return Some(profile.clone());
        }

        if let Ok(custom) = self.custom.read() {
            if let Some(profile) = custom.get(name) {
                return Some(profile.clone());
            }
            if let Some(actual) = self
                .aliases
                .read()
                .ok()
                .and_then(|aliases| aliases.get(&key).cloned())
            {
                if let Some(profile) = custom.get(&actual) {
                    return Some(profile.clone());
                }
            }
        }

        None
    }

    /// Registers or updates a custom profile.
    pub fn register_custom(&self, profile: ModelProfile) -> ModelProfile {
        {
            let mut custom = self.custom.write().expect("custom registry poisoned");
            custom.insert(profile.name.clone(), profile.clone());
        }
        if let Ok(mut aliases) = self.aliases.write() {
            aliases.insert(profile.name.to_lowercase(), profile.name.clone());
        }
        profile
    }

    /// Returns `true` if a profile with the given name is already registered.
    pub fn has_profile(&self, name: &str) -> bool {
        self.builtins.contains_key(name)
            || self
                .custom
                .read()
                .map(|c| c.contains_key(name))
                .unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builtins_available() {
        let registry = UniversalModelRegistry::global();
        assert!(registry.has_profile("qwen2.5-coder-7b-q4"));
        let list = registry.list_profiles();
        assert!(!list.is_empty());
    }

    #[test]
    fn custom_registration_round_trip() {
        let registry = UniversalModelRegistry::global();
        let profile = ModelProfile {
            name: "custom-model".to_string(),
            provider: ProviderKind::Custom("acme".to_string()),
            description: "User supplied profile".to_string(),
            is_local: false,
            api_key_required: true,
            context_window: 8192,
            default_endpoint: Some("https://api.example.dev/v1".to_string()),
            resources: ModelResources {
                size_gb: 12.0,
                min_system_ram_gb: 32,
                min_vram_gb: Some(16),
            },
            tags: vec!["custom".to_string()],
        };

        registry.register_custom(profile.clone());
        let fetched = registry
            .get_profile("custom-model")
            .expect("profile should exist");
        assert_eq!(fetched.name, profile.name);
        assert_eq!(fetched.context_window, 8192);
    }
}
