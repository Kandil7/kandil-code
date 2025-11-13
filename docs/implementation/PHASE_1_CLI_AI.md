## 📄 PHASE_1_CLI_AI.md

```markdown
# Phase 1: Core CLI & AI Adapter

## Objectives
Build secure CLI with argument parsing and unified AI interface supporting local (Ollama) and cloud models. Enable `kandil init` and `kandil chat` commands.

## Prerequisites
- Phase 0 complete (all security tools installed)
- Ollama running on localhost:11434
- API keys stored in OS keychain (if using cloud)

## Detailed Sub-Tasks

### Day 1-2: Enhanced CLI Structure

1. **Add Dependencies**
```bash
cargo add clap --features derive
cargo add tokio --features full
cargo add reqwest --features json
cargo add serde --features derive
cargo add serde_json
cargo add async-trait
```

2. **Implement CLI Entry Point**
```rust
// src/cli/mod.rs
use clap::{Parser, Subcommand};
use anyhow::Result;

#[derive(Parser)]
#[command(name = "kandil")]
#[command(about = "Intelligent development platform", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
    
    #[arg(short, long, global = true, help = "Verbose output")]
    pub verbose: bool,
}

#[derive(Subcommand)]
pub enum Commands {
    Init,
    Chat { message: Option<String> },
    Config { sub: ConfigSub },
}

#[derive(Subcommand)]
pub enum ConfigSub {
    SetKey { provider: String },
    ListKeys,
}

pub async fn run(cli: Cli) -> Result<()> {
    match cli.command {
        Some(Commands::Init) => init_project().await?,
        Some(Commands::Chat { message }) => chat(message.unwrap_or_default()).await?,
        Some(Commands::Config { sub }) => handle_config(sub).await?,
        None => println!("Use --help for commands"),
    }
    Ok(())
}
```

3. **Add Configuration Commands**
```rust
// src/cli/config.rs
use keyring::Entry;
use anyhow::Result;

pub async fn set_key(provider: &str) -> Result<()> {
    println!("Enter API key for {}:", provider);
    let mut key = String::new();
    std::io::stdin().read_line(&mut key)?;
    
    let entry = Entry::new("kandil", provider)?;
    entry.set_password(key.trim())?;
    println!("Key stored securely in OS keychain!");
    Ok(())
}

pub async fn list_keys() -> Result<()> {
    println!("Configured providers:");
    for provider in ["ollama", "anthropic", "openai"] {
        let entry = Entry::new("kandil", provider);
        match entry {
            Ok(e) => match e.get_password() {
                Ok(_) => println!("  ✅ {}", provider),
                Err(_) => println!("  ❌ {} (no key)", provider),
            },
            Err(_) => println!("  ❌ {} (not configured)", provider),
        }
    }
    Ok(())
}
```

### Day 3-4: Workspace Detection

1. **Implement Core Module**
```rust
// src/core/workspace.rs
use std::path::Path;
use anyhow::Result;

#[derive(Debug, Clone)]
pub struct Workspace {
    pub root: String,
    pub project_type: String, // flutter, python, js, rust, unknown
    pub config_path: String,
}

impl Workspace {
    pub fn detect() -> Result<Self> {
        let root = std::env::current_dir()?
            .to_string_lossy()
            .to_string();
        
        let project_type = Self::detect_type(&root)?;
        let config_path = format!("{}/kandil.toml", root);
        
        Ok(Workspace {
            root,
            project_type,
            config_path,
        })
    }
    
    fn detect_type(root: &str) -> Result<String> {
        let path = Path::new(root);
        
        if path.join("pubspec.yaml").exists() {
            return Ok("flutter".to_string());
        }
        if path.join("package.json").exists() {
            return Ok("js".to_string());
        }
        if path.join("requirements.txt").exists() || path.join("pyproject.toml").exists() {
            return Ok("python".to_string());
        }
        if path.join("Cargo.toml").exists() {
            return Ok("rust".to_string());
        }
        Ok("unknown".to_string())
    }
    
    pub fn is_initialized(&self) -> bool {
        Path::new(&self.config_path).exists()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;
    
    #[test]
    fn test_detect_flutter() {
        let temp = TempDir::new().unwrap();
        fs::write(temp.path().join("pubspec.yaml"), "").unwrap();
        std::env::set_current_dir(temp.path()).unwrap();
        
        let ws = Workspace::detect().unwrap();
        assert_eq!(ws.project_type, "flutter");
    }
}
```

### Day 5-7: AI Adapter Base

1. **Define Port (Trait)**
```rust
// src/adapters/ai/mod.rs
use async_trait::async_trait;
use anyhow::Result;
use serde::{Deserialize, Serialize};

#[async_trait]
pub trait AIProvider: Send + Sync {
    async fn chat(&self, message: &str, context: Option<&str>) -> Result<String>;
    async fn chat_stream(&self, message: &str) -> Result<String>; // For future
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIConfig {
    pub provider: String,
    pub model: String,
    pub max_tokens: Option<u32>,
}

pub mod ollama;
pub mod claude;
pub mod openai;
pub mod factory;
```

2. **Ollama Adapter (Local AI)**
```rust
// src/adapters/ai/ollama.rs
use super::{AIProvider, AIConfig};
use async_trait::async_trait;
use anyhow::Result;
use reqwest::Client;
use serde_json::json;

pub struct OllamaAdapter {
    client: Client,
    config: AIConfig,
}

impl OllamaAdapter {
    pub fn new(config: AIConfig) -> Self {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(120))
            .build()
            .unwrap();
        
        Self { client, config }
    }
}

#[async_trait]
impl AIProvider for OllamaAdapter {
    async fn chat(&self, message: &str, _context: Option<&str>) -> Result<String> {
        let payload = json!({
            "model": self.config.model,
            "prompt": message,
            "stream": false,
            "options": {
                "temperature": 0.7
            }
        });
        
        let url = format!("{}/api/generate", 
            std::env::var("KANDIL_OLLAMA_URL")
                .unwrap_or_else(|_| "http://localhost:11434".to_string())
        );
        
        let resp = self.client
            .post(&url)
            .json(&payload)
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("Ollama connection failed: {}", e))?;
        
        if !resp.status().is_success() {
            return Err(anyhow::anyhow!("Ollama error: {}", resp.text().await?));
        }
        
        let body: serde_json::Value = resp.json().await?;
        Ok(body["response"].as_str().unwrap_or("No response").to_string())
    }
    
    async fn chat_stream(&self, _message: &str) -> Result<String> {
        // TODO: Implement streaming for TUI
        Ok("Streaming not yet implemented".to_string())
    }
}
```

3. **Adapter Factory with Cost Tracking**
```rust
// src/adapters/ai/factory.rs
use super::{AIProvider, AIConfig};
use crate::adapters::ai::{ollama::OllamaAdapter, claude::ClaudeAdapter};
use anyhow::Result;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct AIProviderFactory {
    config: AIConfig,
    usage_tracker: Arc<Mutex<UsageTracker>>,
}

#[derive(Default)]
pub struct UsageTracker {
    pub requests: u64,
    pub tokens_sent: u64,
    pub tokens_received: u64,
    pub estimated_cost_usd: f64,
}

impl AIProviderFactory {
    pub fn new(config: AIConfig) -> Self {
        Self {
            config,
            usage_tracker: Arc::new(Mutex::new(UsageTracker::default())),
        }
    }
    
    pub async fn create(&self) -> Result<Box<dyn AIProvider>> {
        let tracker = self.usage_tracker.clone();
        let provider: Box<dyn AIProvider> = match self.config.provider.as_str() {
            "ollama" => Box::new(OllamaAdapter::new(self.config.clone())),
            "anthropic" => {
                let api_key = crate::utils::keys::get_key("anthropic")?;
                Box::new(ClaudeAdapter::new(self.config.clone(), api_key, tracker))
            },
            "openai" => {
                let api_key = crate::utils::keys::get_key("openai")?;
                Box::new(OpenAIAdapter::new(self.config.clone(), api_key, tracker))
            },
            _ => return Err(anyhow::anyhow!("Unknown provider: {}", self.config.provider)),
        };
        
        Ok(provider)
    }
    
    pub async fn get_usage(&self) -> UsageTracker {
        self.usage_tracker.lock().await.clone()
    }
}
```

### Day 8-10: Configuration & Utils

1. **Secure Key Management**
```rust
// src/utils/keys.rs
use keyring::Entry;
use anyhow::Result;

pub fn get_key(provider: &str) -> Result<String> {
    let entry = Entry::new("kandil", provider)?;
    entry.get_password()
        .map_err(|_| anyhow::anyhow!("No API key for {}. Run: kandil config set-key {}", provider, provider))
}

pub fn set_key(provider: &str, key: &str) -> Result<()> {
    let entry = Entry::new("kandil", provider)?;
    entry.set_password(key)?;
    Ok(())
}

pub fn delete_key(provider: &str) -> Result<()> {
    let entry = Entry::new("kandil", provider)?;
    entry.delete_password()?;
    Ok(())
}
```

2. **Configuration Loader**
```rust
// src/utils/config.rs
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub ai: AIConfig,
    pub projects: ProjectsConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIConfig {
    pub provider: String,
    pub model: String,
    pub max_tokens: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectsConfig {
    pub memory_enabled: bool,
}

impl Config {
    pub fn load() -> Result<Self> {
        let path = Path::new("kandil.toml");
        if !path.exists() {
            return Ok(Self::default());
        }
        
        let content = fs::read_to_string(path)?;
        toml::from_str(&content)
            .map_err(|e| anyhow::anyhow!("Invalid config: {}", e))
    }
    
    pub fn save(&self) -> Result<()> {
        let content = toml::to_string_pretty(self)?;
        fs::write("kandil.toml", content)?;
        Ok(())
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            ai: AIConfig {
                provider: "ollama".to_string(),
                model: "llama3:70b".to_string(),
                max_tokens: 2000,
            },
            projects: ProjectsConfig {
                memory_enabled: true,
            },
        }
    }
}
```

### Day 11-14: Command Implementation & Testing

1. **Init Command**
```rust
// src/cli/init.rs
use crate::utils::config::Config;
use anyhow::Result;

pub async fn init_project() -> Result<()> {
    if Config::load()?.ai.provider != "unknown" {
        println!("Project already initialized");
        return Ok(());
    }
    
    let config = Config::default();
    config.save()?;
    
    // Create .gitignore if not exists
    if !std::path::Path::new(".gitignore").exists() {
        std::fs::write(".gitignore", include_str!("../../templates/gitignore.txt"))?;
    }
    
    println!("✅ Kandil project initialized");
    println!("Config saved to kandil.toml");
    Ok(())
}
```

2. **Chat Command with Context**
```rust
// src/cli/chat.rs
use crate::adapters::ai::factory::AIProviderFactory;
use crate::utils::config::Config;
use anyhow::Result;

pub async fn chat(message: String) -> Result<()> {
    if message.trim().is_empty() {
        return Err(anyhow::anyhow!("Message cannot be empty"));
    }
    
    let config = Config::load()?;
    let factory = AIProviderFactory::new(config.ai);
    let ai = factory.create().await?;
    
    // Add workspace context
    let ws = crate::core::workspace::Workspace::detect()?;
    let context = if ws.is_initialized() {
        Some(format!("Project type: {}", ws.project_type))
    } else {
        None
    };
    
    let response = ai.chat(&message, context.as_deref()).await?;
    println!("🤖 AI: {}", response);
    
    // Log usage
    let usage = factory.get_usage().await;
    tracing::info!("AI request completed: {} tokens", usage.tokens_sent);
    
    Ok(())
}
```

3. **Unit Tests**
```rust
// tests/unit/ai_adapter_test.rs
use kandil_code::adapters::ai::{AIConfig, AIProvider, ollama::OllamaAdapter};
use async_trait::async_trait;

struct MockProvider;

#[async_trait]
impl AIProvider for MockProvider {
    async fn chat(&self, message: &str, _context: Option<&str>) -> anyhow::Result<String> {
        Ok(format!("Mock: {}", message))
    }
    async fn chat_stream(&self, _message: &str) -> anyhow::Result<String> {
        Ok("Mock stream".to_string())
    }
}

#[tokio::test]
async fn test_ai_provider_trait() {
    let mock = MockProvider;
    let result = mock.chat("Hello", None).await.unwrap();
    assert_eq!(result, "Mock: Hello");
}
```

## Tools & Dependencies
| Crate | Version | Purpose |
|-------|---------|---------|
| clap | 4.4 | CLI parsing |
| tokio | 1.35 | Async runtime |
| reqwest | 0.11 | HTTP client |
| serde | 1.0 | Serialization |
| async-trait | 0.1 | Trait objects |
| keyring | 2.0 | Secure storage |
| tempfile | 3.8 | Test fixtures |

## Testing Strategy
- **Unit**: Mock AI providers (80% coverage)
- **Integration**: Test CLI commands with temp dirs
- **Manual**: Verify Ollama responses

## Deliverables
- ✅ CLI with init, chat, config commands
- ✅ Workspace detection for 4 languages
- ✅ Secure AI adapter (Ollama + cloud-ready)
- ✅ OS keychain integration
- ✅ Usage tracking
- ✅ 80% test coverage

## Timeline Breakdown
- **Days 1-3**: CLI structure + workspace
- **Days 4-7**: AI adapter + security
- **Days 8-10**: Commands + testing
- **Days 11-14**: Polish & docs

## Success Criteria
- `kandil init` creates valid config
- `kandil chat "test"` returns Ollama response
- `kandil config set-key anthropic` stores key securely
- `cargo tarpaulin` shows ≥80% coverage
- No clippy warnings
- Pre-commit hooks pass

## Potential Risks & Mitigations

| Risk | Mitigation |
|------|------------|
| Ollama timeout | Add 120s timeout + retry logic |
| Keyring fails on Linux | Install `libsecret-dev`; fallback to file encryption |
| Async complexity | Use `tokio::select!` for cancellation |
| Provider API changes | Version trait, add adapter tests |

---

**Next**: Proceed to PHASE_2_TEMPLATES_PLUGINS.md after Phase 1 CI passes.
```

---

*(Continues with remaining phases...)*