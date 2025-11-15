use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use anyhow::Result;

use crate::core::hardware::detect_hardware;
use crate::core::auto_config::AutoConfig;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StrategyConfig {
    pub mode: crate::core::strategy::ExecutionMode,
    pub timeout_ms: u64,
    pub fast_model: Option<String>,
    pub quality_model: Option<String>,
}

impl Default for StrategyConfig {
    fn default() -> Self {
        Self {
            mode: crate::core::strategy::ExecutionMode::Local,
            timeout_ms: 30000, // 30 seconds default
            fast_model: None,
            quality_model: None,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub model: ModelConfig,
    pub performance: PerformanceConfig,
    pub fallback: FallbackConfig,
    pub strategy: StrategyConfig,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ModelConfig {
    pub name: String,
    pub path: Option<PathBuf>,
    pub context_size: usize,
    pub quantization: Quantization,
    pub threads: Option<usize>,  // Added for model loading
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PerformanceConfig {
    pub threads: usize,
    pub use_mmap: bool,
    pub batch_size: usize,
    pub target_latency_ms: u64,
    pub reserve_ram_gb: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FallbackConfig {
    pub enabled: bool,
    pub cloud_provider: Option<CloudProvider>,
    pub timeout_ms: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Quantization {
    Q3_K_M,
    Q4_K_M,
    Q5_K_M,
    Q6_K,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum CloudProvider {
    Claude,
    OpenAI,
    Ollama,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            model: ModelConfig {
                name: "auto".to_string(),
                path: None,
                context_size: 4096,
                quantization: Quantization::Q4_K_M,
                threads: Some(4),
            },
            performance: PerformanceConfig {
                threads: 4,
                use_mmap: true,
                batch_size: 512,
                target_latency_ms: 2000,
                reserve_ram_gb: 4,
            },
            fallback: FallbackConfig {
                enabled: true,
                cloud_provider: None,
                timeout_ms: 30000,
            },
            strategy: StrategyConfig::default(),
        }
    }
}

impl Config {
    pub fn load() -> Result<Self> {
        // Load from different sources with priority: CLI > ENV > Local > User > Auto-detect

        // Start with default config
        let mut config = Config::default();

        // Apply auto-detected config based on hardware
        let hardware = detect_hardware();
        let auto_config = AutoConfig::from_hardware(&hardware);
        config.merge(auto_config);

        // Apply user config if available
        if let Some(user_config) = Self::load_user_config()? {
            config.merge(user_config);
        }

        // Apply local/project config if available
        if let Some(local_config) = Self::load_local_config()? {
            config.merge(local_config);
        }

        // Apply environment variables
        config.apply_env_vars();

        // Apply CLI args - this would be done in the CLI module, not here
        // For now, we'll add a placeholder for where this would happen

        Ok(config)
    }

    fn load_user_config() -> Result<Option<Self>> {
        // Look for config in user's home directory
        let config_dir = dirs::config_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not find config directory"))?
            .join("kandil");

        let config_path = config_dir.join("config.toml");

        if config_path.exists() {
            let content = std::fs::read_to_string(config_path)?;
            let config: Config = toml::from_str(&content)?;
            Ok(Some(config))
        } else {
            Ok(None)
        }
    }

    fn load_local_config() -> Result<Option<Self>> {
        // Look for config in current directory
        let config_paths = [
            std::path::PathBuf::from(".kandil.toml"),
            std::path::PathBuf::from("kandil.toml"),
        ];

        for config_path in &config_paths {
            if config_path.exists() {
                let content = std::fs::read_to_string(config_path)?;
                let config: Config = toml::from_str(&content)?;
                return Ok(Some(config));
            }
        }

        Ok(None)
    }

    fn apply_env_vars(&mut self) {
        // Apply environment variables if they exist
        if let Ok(model_name) = std::env::var("KANDIL_MODEL") {
            self.model.name = model_name;
        }

        if let Ok(threads_str) = std::env::var("KANDIL_THREADS") {
            if let Ok(threads) = threads_str.parse::<usize>() {
                self.performance.threads = threads;
            }
        }

        if let Ok(use_mmap_str) = std::env::var("KANDIL_USE_MMAP") {
            if let Ok(use_mmap) = use_mmap_str.parse::<bool>() {
                self.performance.use_mmap = use_mmap;
            }
        }

        if let Ok(context_size_str) = std::env::var("KANDIL_CONTEXT_SIZE") {
            if let Ok(context_size) = context_size_str.parse::<usize>() {
                self.model.context_size = context_size;
            }
        }

        if let Ok(timeout_str) = std::env::var("KANDIL_TIMEOUT_MS") {
            if let Ok(timeout) = timeout_str.parse::<u64>() {
                self.fallback.timeout_ms = timeout;
            }
        }
    }

    fn merge(&mut self, other: Self) {
        // Override current config with values from 'other'
        // Only override if the other value is meaningful

        if !other.model.name.is_empty() && other.model.name != "auto" {
            self.model.name = other.model.name;
        }

        if other.model.path.is_some() {
            self.model.path = other.model.path;
        }

        if other.model.context_size != 0 {
            self.model.context_size = other.model.context_size;
        }

        if other.model.threads.is_some() {
            self.model.threads = other.model.threads;
        }

        // Performance config
        if other.performance.threads != 0 {
            self.performance.threads = other.performance.threads;
        }

        self.performance.use_mmap = other.performance.use_mmap;

        if other.performance.batch_size != 0 {
            self.performance.batch_size = other.performance.batch_size;
        }

        if other.performance.target_latency_ms != 0 {
            self.performance.target_latency_ms = other.performance.target_latency_ms;
        }

        if other.performance.reserve_ram_gb != 0 {
            self.performance.reserve_ram_gb = other.performance.reserve_ram_gb;
        }

        // Fallback config
        self.fallback.enabled = other.fallback.enabled;
        if other.fallback.cloud_provider.is_some() {
            self.fallback.cloud_provider = other.fallback.cloud_provider;
        }
        if other.fallback.timeout_ms != 0 {
            self.fallback.timeout_ms = other.fallback.timeout_ms;
        }

        // Strategy config
        self.strategy.mode = other.strategy.mode;
        if other.strategy.timeout_ms != 0 {
            self.strategy.timeout_ms = other.strategy.timeout_ms;
        }
        if other.strategy.fast_model.is_some() {
            self.strategy.fast_model = other.strategy.fast_model;
        }
        if other.strategy.quality_model.is_some() {
            self.strategy.quality_model = other.strategy.quality_model;
        }
    }
}

// Define merge_configs function that was referenced in the original plan
fn merge_configs(
    _cli: CliConfig,
    _env: EnvConfig,
    _local: LocalConfig,
    _user: UserConfig,
    auto: Config,
) -> Config {
    // For now, return the auto-config as a placeholder
    // In a full implementation, we would properly merge all configs with priority
    auto
}

// Placeholder structs for the different config sources
#[derive(Debug, Default)]
pub struct CliConfig;

#[derive(Debug, Default)]
pub struct EnvConfig;

#[derive(Debug, Default)]
pub struct LocalConfig;

#[derive(Debug, Default)]
pub struct UserConfig;