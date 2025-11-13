# 📄 PHASE_4_REFACTOR_TESTS_MODELS.md

```markdown
# Phase 4: Refactor, Tests, & Multi-Model Integration

## Objectives
Implement cloud AI adapters (Claude, Qwen, OpenAI), code refactoring with preview/apply workflow, automated test generation, and model switching with cost tracking. Enable `kandil refactor`, `kandil test generate`, and `kandil switch-model`.

## Prerequisites
- Phase 3 complete (TUI with code analysis)
- API keys for: Anthropic (Claude), OpenAI, Alibaba Cloud (Qwen)
- Ollama running as fallback
- Budget tracking system ready

## Detailed Sub-Tasks

### Day 1-3: Cloud AI Adapters

1. **Add Dependencies**
```bash
cargo add anthropic-rs # For Claude
cargo add async-openai # For OpenAI
cargo add reqwest --features json # For Qwen (manual API)
cargo add secrecy # Secure API key handling
cargo add ring # For request signing (Qwen)
cargo add dashmap # Thread-safe usage tracking
```

2. **Secure API Key Storage**
```rust
// src/utils/keys.rs (enhanced)
use secrecy::{Secret, ExposeSecret};
use keyring::Entry;
use anyhow::Result;

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

// Cost tracking per provider
pub struct CostTracker {
    anthropic: dashmap::DashMap<String, f64>,
    openai: dashmap::DashMap<String, f64>,
    qwen: dashmap::DashMap<String, f64>,
}

impl CostTracker {
    pub fn new() -> Self {
        Self {
            anthropic: dashmap::DashMap::new(),
            openai: dashmap::DashMap::new(),
            qwen: dashmap::DashMap::new(),
        }
    }
    
    pub fn add_cost(&self, provider: &str, model: &str, tokens: u64) {
        let cost_per_1k = match (provider, model) {
            ("anthropic", "claude-3-opus") => 0.015,
            ("anthropic", "claude-3-sonnet") => 0.003,
            ("openai", "gpt-4") => 0.03,
            ("openai", "gpt-3.5-turbo") => 0.0015,
            ("qwen", "qwen-max") => 0.02,
            _ => 0.0,
        };
        
        let total = (tokens as f64 / 1000.0) * cost_per_1k;
        
        match provider {
            "anthropic" => self.anthropic.insert(model.to_string(), total),
            "openai" => self.openai.insert(model.to_string(), total),
            "qwen" => self.qwen.insert(model.to_string(), total),
            _ => None,
        };
    }
    
    pub fn get_total_cost(&self) -> f64 {
        let mut total = 0.0;
        for cost in self.anthropic.iter() {
            total += *cost.value();
        }
        for cost in self.openai.iter() {
            total += *cost.value();
        }
        for cost in self.qwen.iter() {
            total += *cost.value();
        }
        total
    }
}
```

3. **Claude Adapter (Anthropic)**
```rust
// src/adapters/ai/claude.rs
use super::{AIProvider, AIConfig};
use async_trait::async_trait;
use anyhow::Result;
use anthropic_rs::Client;
use anthropic_rs::types::Message;
use secrecy::ExposeSecret;
use crate::utils::keys::SecureKey;

pub struct ClaudeAdapter {
    client: Client,
    config: AIConfig,
    cost_tracker: dashmap::DashMap<String, f64>,
}

impl ClaudeAdapter {
    pub fn new(config: AIConfig, key: SecureKey, cost_tracker: dashmap::DashMap<String, f64>) -> Result<Self> {
        let client = Client::new()
            .with_api_key(key.expose())
            .with_model(&config.model);
        
        Ok(Self {
            client,
            config,
            cost_tracker,
        })
    }
}

#[async_trait]
impl AIProvider for ClaudeAdapter {
    async fn chat(&self, message: &str, context: Option<&str>) -> Result<String> {
        let full_message = if let Some(ctx) = context {
            format!("Context: {}\n\nUser: {}", ctx, message)
        } else {
            message.to_string()
        };
        
        let messages = vec![
            Message::user(&full_message)
        ];
        
        let response = self.client.messages()
            .create(messages)
            .max_tokens(self.config.max_tokens.unwrap_or(4000))
            .temperature(0.7)
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("Claude API error: {}", e))?;
        
        // Track cost
        let tokens_used = response.usage.input_tokens + response.usage.output_tokens;
        self.cost_tracker.add_cost("anthropic", &self.config.model, tokens_used);
        
        Ok(response.content[0].text.clone())
    }
    
    async fn chat_stream(&self, _message: &str) -> Result<String> {
        // Claude streaming API available but complex
        // For MVP, use non-streaming
        Ok("Streaming not implemented for Claude".to_string())
    }
}
```

4. **OpenAI Adapter**
```rust
// src/adapters/ai/openai.rs
use super::{AIProvider, AIConfig};
use async_trait::async_trait;
use anyhow::Result;
use async_openai::{Client, config::OpenAIConfig};
use async_openai::types::{
    CreateChatCompletionRequestArgs,
    ChatCompletionRequestMessage,
    Role,
};
use secrecy::ExposeSecret;
use crate::utils::keys::SecureKey;

pub struct OpenAIAdapter {
    client: Client<OpenAIConfig>,
    config: AIConfig,
    cost_tracker: dashmap::DashMap<String, f64>,
}

impl OpenAIAdapter {
    pub fn new(config: AIConfig, key: SecureKey, cost_tracker: dashmap::DashMap<String, f64>) -> Result<Self> {
        let config = OpenAIConfig::new()
            .with_api_key(key.expose().to_string());
        let client = Client::with_config(config);
        
        Ok(Self {
            client,
            config,
            cost_tracker,
        })
    }
}

#[async_trait]
impl AIProvider for OpenAIAdapter {
    async fn chat(&self, message: &str, context: Option<&str>) -> Result<String> {
        let mut messages = Vec::new();
        
        if let Some(ctx) = context {
            messages.push(ChatCompletionRequestMessage {
                role: Role::System,
                content: Some(ctx.to_string()),
                name: None,
                function_call: None,
            });
        }
        
        messages.push(ChatCompletionRequestMessage {
            role: Role::User,
            content: Some(message.to_string()),
            name: None,
            function_call: None,
        });
        
        let request = CreateChatCompletionRequestArgs::default()
            .model(&self.config.model)
            .messages(messages)
            .max_tokens(self.config.max_tokens.unwrap_or(2000))
            .temperature(0.7)
            .build()?;
        
        let response = self.client.chat()
            .create(request)
            .await
            .map_err(|e| anyhow::anyhow!("OpenAI error: {}", e))?;
        
        // Track cost
        if let Some(usage) = response.usage {
            self.cost_tracker.add_cost(
                "openai",
                &self.config.model,
                usage.total_tokens as u64
            );
        }
        
        Ok(response.choices[0].message.content.clone()
            .unwrap_or_else(|| "No response".to_string()))
    }
    
    async fn chat_stream(&self, message: &str) -> Result<String> {
        // OpenAI streaming is well-supported
        // For MVP, keep non-streaming
        Ok("Use chat() for OpenAI".to_string())
    }
}
```

5. **Qwen Adapter (Alibaba Cloud)**
```rust
// src/adapters/ai/qwen.rs
use super::{AIProvider, AIConfig};
use async_trait::async_trait;
use anyhow::Result;
use reqwest::{Client, header};
use serde::{Deserialize, Serialize};
use secrecy::ExposeSecret;
use crate::utils::keys::SecureKey;

#[derive(Serialize)]
struct QwenRequest {
    model: String,
    input: QwenInput,
    parameters: QwenParameters,
}

#[derive(Serialize)]
struct QwenInput {
    messages: Vec<QwenMessage>,
}

#[derive(Serialize)]
struct QwenMessage {
    role: String,
    content: String,
}

#[derive(Serialize)]
struct QwenParameters {
    result_format: String,
}

#[derive(Deserialize)]
struct QwenResponse {
    output: QwenOutput,
    usage: QwenUsage,
}

#[derive(Deserialize)]
struct QwenOutput {
    text: String,
}

#[derive(Deserialize)]
struct QwenUsage {
    total_tokens: u64,
}

pub struct QwenAdapter {
    client: Client,
    config: AIConfig,
    api_key: SecureKey,
    cost_tracker: dashmap::DashMap<String, f64>,
}

impl QwenAdapter {
    pub fn new(config: AIConfig, key: SecureKey, cost_tracker: dashmap::DashMap<String, f64>) -> Self {
        let mut headers = header::HeaderMap::new();
        headers.insert(
            "Authorization",
            format!("Bearer {}", key.expose()).parse().unwrap(),
        );
        
        let client = Client::builder()
            .default_headers(headers)
            .timeout(std::time::Duration::from_secs(60))
            .build()
            .unwrap();
        
        Self {
            client,
            config,
            api_key: key,
            cost_tracker,
        }
    }
}

#[async_trait]
impl AIProvider for QwenAdapter {
    async fn chat(&self, message: &str, context: Option<&str>) -> Result<String> {
        let content = if let Some(ctx) = context {
            format!("Context: {}\n\nUser: {}", ctx, message)
        } else {
            message.to_string()
        };
        
        let request = QwenRequest {
            model: self.config.model.clone(),
            input: QwenInput {
                messages: vec![QwenMessage {
                    role: "user".to_string(),
                    content,
                }],
            },
            parameters: QwenParameters {
                result_format: "text".to_string(),
            },
        };
        
        let response = self.client
            .post("https://dashscope.aliyuncs.com/api/v1/services/aigc/text-generation/generation")
            .json(&request)
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("Qwen API error: {}", e))?;
        
        if !response.status().is_success() {
            return Err(anyhow::anyhow!("Qwen API error: {}", response.text().await?));
        }
        
        let qwen_resp: QwenResponse = response.json().await?;
        
        // Track cost
        self.cost_tracker.add_cost(
            "qwen",
            &self.config.model,
            qwen_resp.usage.total_tokens
        );
        
        Ok(qwen_resp.output.text)
    }
    
    async fn chat_stream(&self, _message: &str) -> Result<String> {
        Ok("Streaming not implemented for Qwen".to_string())
    }
}
```

### Day 4-6: Refactor Command

1. **Refactor Engine**
```rust
// src/code/refactor.rs
use crate::adapters::ai::factory::AIProviderFactory;
use crate::utils::config::Config;
use anyhow::Result;
use std::fs;
use std::path::Path;
use console::Term;
use dialoguer::{theme::ColorfulTheme, Confirm};

pub struct RefactorEngine {
    ai_factory: AIProviderFactory,
}

impl RefactorEngine {
    pub fn new() -> Result<Self> {
        let config = Config::load()?;
        let ai_factory = AIProviderFactory::new(config.ai);
        Ok(Self { ai_factory })
    }
    
    pub async fn refactor_file(
        &self,
        file_path: &Path,
        goal: &str,
        preview_only: bool,
    ) -> Result<Option<String>> {
        let original_code = fs::read_to_string(file_path)?;
        
        let prompt = format!(
            r#"Refactor this code to achieve: {}
            
            Original code:
            ```
            {}
            ```
            
            Return ONLY the refactored code. Do not include explanations."#,
            goal, original_code
        );
        
        let ai = self.ai_factory.create().await?;
        let refactored = ai.chat(&prompt, None).await?;
        
        if preview_only {
            return Ok(Some(refactored));
        }
        
        // Interactive confirmation
        self.show_diff(&original_code, &refactored)?;
        
        let term = Term::stdout();
        let confirm = Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt("Apply these changes?")
            .default(false)
            .interact_on(&term)?;
        
        if confirm {
            fs::write(file_path, refactored)?;
            println!("✅ Refactor applied to {}", file_path.display());
            Ok(None)
        } else {
            println!("❌ Refactor cancelled");
            Ok(None)
        }
    }
    
    fn show_diff(&self, original: &str, refactored: &str) -> Result<()> {
        use similar::{ChangeTag, TextDiff};
        
        let diff = TextDiff::from_lines(original, refactored);
        
        println!("📊 Changes:");
        for change in diff.iter_all_changes() {
            match change.tag() {
                ChangeTag::Delete => {
                    println!("- {}", change.to_string().trim_end());
                }
                ChangeTag::Insert => {
                    println!("+ {}", change.to_string().trim_end());
                }
                ChangeTag::Equal => {
                    // Skip unchanged lines for brevity
                }
            }
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    
    #[tokio::test]
    async fn test_refactor_preview() {
        let temp = NamedTempFile::new().unwrap();
        fs::write(temp.path(), "fn add(a: i32, b: i32) -> i32 { a + b }").unwrap();
        
        let engine = RefactorEngine::new().unwrap();
        let result = engine.refactor_file(temp.path(), "Add error handling", true).await;
        
        assert!(result.unwrap().is_some());
    }
}
```

2. **CLI Integration**
```rust
// src/cli/refactor.rs
use std::path::PathBuf;
use crate::code::refactor::RefactorEngine;
use anyhow::Result;

pub async fn refactor_command(
    file: PathBuf,
    goal: String,
    preview: bool,
) -> Result<()> {
    if !file.exists() {
        return Err(anyhow::anyhow!("File not found: {}", file.display()));
    }
    
    let engine = RefactorEngine::new()?;
    engine.refactor_file(&file, &goal, preview).await?;
    
    Ok(())
}
```

### Day 7-8: Test Generation Engine

1. **Test Generation Logic**
```rust
// src/code/test_gen.rs
use crate::adapters::ai::factory::AIProviderFactory;
use crate::utils::config::Config;
use anyhow::Result;
use std::fs;
use std::path::Path;
use std::process::Command;

pub struct TestGenerator {
    ai_factory: AIProviderFactory,
    target_coverage: u8,
}

impl TestGenerator {
    pub fn new(target_coverage: u8) -> Result<Self> {
        let config = Config::load()?;
        let ai_factory = AIProviderFactory::new(config.ai);
        Ok(Self {
            ai_factory,
            target_coverage,
        })
    }
    
    pub async fn generate_tests(&self, file_path: &Path) -> Result<String> {
        let code = fs::read_to_string(file_path)?;
        let ext = file_path.extension()
            .and_then(|s| s.to_str())
            .unwrap_or("");
        
        // Get context about existing tests
        let test_context = self.analyze_existing_tests(file_path)?;
        
        let prompt = format!(
            r#"Generate comprehensive unit tests for this {} code.
            
            Target coverage: {}%
            Existing tests: {}
            
            Code to test:
            ```
            {}
            ```
            
            Requirements:
            1. Use best practices for {} testing
            2. Mock external dependencies
            3. Cover edge cases and error paths
            4. Include arrange-act-assert pattern
            5. Tests should be deterministic
            
            Return ONLY the test code."#,
            ext.to_uppercase(),
            self.target_coverage,
            test_context.as_deref().unwrap_or("None"),
            code,
            ext.to_uppercase()
        );
        
        let ai = self.ai_factory.create().await?;
        let test_code = ai.chat(&prompt, None).await?;
        
        // Write to test file
        let test_file = self.get_test_file_name(file_path, ext)?;
        fs::write(&test_file, &test_code)?;
        
        // Verify tests compile/run
        self.verify_tests(&test_file, ext).await?;
        
        Ok(test_file.to_string_lossy().to_string())
    }
    
    fn analyze_existing_tests(&self, file_path: &Path) -> Result<Option<String>> {
        let test_file = self.get_test_file_name(file_path, "test")?;
        if test_file.exists() {
            Ok(Some(fs::read_to_string(test_file)?))
        } else {
            Ok(None)
        }
    }
    
    fn get_test_file_name(&self, original: &Path, ext: &str) -> Result<PathBuf> {
        let parent = original.parent().unwrap();
        let stem = original.file_stem().unwrap();
        
        let test_name = match ext {
            "dart" => format!("{}_test.dart", stem.to_string_lossy()),
            "py" => format!("test_{}.py", stem.to_string_lossy()),
            "rs" => format!("{}_test.rs", stem.to_string_lossy()),
            _ => format!("{}_test.{}", stem.to_string_lossy(), ext),
        };
        
        Ok(parent.join(test_name))
    }
    
    async fn verify_tests(&self, test_file: &Path, ext: &str) -> Result<()> {
        match ext {
            "dart" => {
                Command::new("flutter")
                    .args(&["test", test_file.to_str().unwrap()])
                    .status()?;
            }
            "py" => {
                Command::new("pytest")
                    .arg(test_file)
                    .status()?;
            }
            "rs" => {
                Command::new("cargo")
                    .args(&["test", "--test", test_file.file_stem().unwrap().to_str().unwrap()])
                    .status()?;
            }
            _ => {}
        }
        Ok(())
    }
}
```

2. **Coverage Analysis Integration**
```rust
// src/code/coverage.rs
use std::path::Path;
use std::process::Command;
use anyhow::Result;

pub struct CoverageAnalyzer;

impl CoverageAnalyzer {
    pub fn analyze(project_path: &Path, lang: &str) -> Result<CoverageReport> {
        match lang {
            "flutter" => Self::analyze_flutter(project_path),
            "python" => Self::analyze_python(project_path),
            "rust" => Self::analyze_rust(project_path),
            _ => Ok(CoverageReport::default()),
        }
    }
    
    fn analyze_flutter(path: &Path) -> Result<CoverageReport> {
        Command::new("flutter")
            .args(&["test", "--coverage"])
            .current_dir(path)
            .status()?;
        
        // Parse lcov.info
        let lcov = std::fs::read_to_string(path.join("coverage/lcov.info"))?;
        Self::parse_lcov(&lcov)
    }
    
    fn analyze_python(path: &Path) -> Result<CoverageReport> {
        Command::new("pytest")
            .args(&["--cov=.", "--cov-report=term-missing"])
            .current_dir(path)
            .status()?;
        
        Ok(CoverageReport {
            line_coverage: 0.0, // Parse from output if needed
            branch_coverage: 0.0,
            function_coverage: 0.0,
        })
    }
    
    fn analyze_rust(path: &Path) -> Result<CoverageReport> {
        Command::new("cargo")
            .args(&["tarpaulin", "--out", "Json"])
            .current_dir(path)
            .status()?;
        
        // Parse tarpaulin output
        let json = std::fs::read_to_string(path.join("tarpaulin-report.json"))?;
        let report: serde_json::Value = serde_json::from_str(&json)?;
        
        Ok(CoverageReport {
            line_coverage: report["coverage"]["percentage"].as_f64().unwrap_or(0.0),
            branch_coverage: 0.0,
            function_coverage: 0.0,
        })
    }
    
    fn parse_lcov(lcov: &str) -> Result<CoverageReport> {
        // Simple LCOV parser
        let mut lines_hit = 0;
        let mut lines_found = 0;
        
        for line in lcov.lines() {
            if line.starts_with("LH:") {
                lines_hit += line[3..].parse::<u32>().unwrap_or(0);
            } else if line.starts_with("LF:") {
                lines_found += line[3..].parse::<u32>().unwrap_or(0);
            }
        }
        
        Ok(CoverageReport {
            line_coverage: if lines_found > 0 {
                (lines_hit as f64 / lines_found as f64) * 100.0
            } else {
                0.0
            },
            branch_coverage: 0.0,
            function_coverage: 0.0,
        })
    }
}

#[derive(Debug, Default)]
pub struct CoverageReport {
    pub line_coverage: f64,
    pub branch_coverage: f64,
    pub function_coverage: f64,
}
```

### Day 9-11: Multi-Model Switching & Fallback

1. **Model Switcher**
```rust
// src/cli/switch.rs
use crate::utils::config::Config;
use anyhow::Result;

pub async fn switch_model(provider: &str, model: Option<&str>) -> Result<()> {
    let mut config = Config::load()?;
    
    // Validate provider
    match provider {
        "ollama" => {
            config.ai.provider = "ollama".to_string();
            config.ai.model = model.unwrap_or("llama3:70b").to_string();
        }
        "anthropic" => {
            // Check API key exists
            crate::utils::keys::SecureKey::load("anthropic")?;
            config.ai.provider = "anthropic".to_string();
            config.ai.model = model.unwrap_or("claude-3-opus-20240229").to_string();
        }
        "openai" => {
            crate::utils::keys::SecureKey::load("openai")?;
            config.ai.provider = "openai".to_string();
            config.ai.model = model.unwrap_or("gpt-4").to_string();
        }
        "qwen" => {
            crate::utils::keys::SecureKey::load("qwen")?;
            config.ai.provider = "qwen".to_string();
            config.ai.model = model.unwrap_or("qwen-max").to_string();
        }
        _ => return Err(anyhow::anyhow!("Unknown provider: {}", provider)),
    }
    
    config.save()?;
    
    println!("✅ Switched to {} ({})", config.ai.provider, config.ai.model);
    println!("Run 'kandil config show' to verify");
    
    Ok(())
}
```

2. **Fallback Logic in Factory**
```rust
// src/adapters/ai/factory.rs
impl AIProviderFactory {
    pub async fn create_with_fallback(&self) -> Result<Box<dyn AIProvider>> {
        // Try primary provider
        match self.create().await {
            Ok(ai) => Ok(ai),
            Err(e) => {
                tracing::warn!("Primary AI failed: {}, falling back to Ollama", e);
                
                // Create Ollama fallback
                let fallback_config = AIConfig {
                    provider: "ollama".to_string(),
                    model: "llama3:70b".to_string(),
                    max_tokens: self.config.max_tokens,
                };
                
                let fallback_factory = AIProviderFactory::new(fallback_config);
                fallback_factory.create().await
            }
        }
    }
}
```

3. **Cost Display Command**
```rust
// src/cli/cost.rs
use crate::adapters::ai::factory::AIProviderFactory;
use crate::utils::config::Config;
use anyhow::Result;

pub async fn show_costs() -> Result<()> {
    let config = Config::load()?;
    let factory = AIProviderFactory::new(config.ai);
    
    println!("💰 AI Usage Costs:");
    println!("==================");
    
    let total = factory.get_total_cost().await;
    println!("Total spent: ${:.4}", total);
    
    if total > 10.0 {
        println!("⚠️  Warning: High spending detected!");
        println!("Consider switching to local models with: kandil switch-model ollama");
    }
    
    Ok(())
}
```

### Day 12-14: Documentation Generation

1. **DocGen Engine**
```rust
// src/code/docgen.rs
use crate::adapters::ai::factory::AIProviderFactory;
use crate::utils::config::Config;
use anyhow::Result;
use walkdir::WalkDir;
use std::path::Path;

pub struct DocGenerator {
    ai_factory: AIProviderFactory,
}

impl DocGenerator {
    pub fn new() -> Result<Self> {
        let config = Config::load()?;
        let ai_factory = AIProviderFactory::new(config.ai);
        Ok(Self { ai_factory })
    }
    
    pub async fn generate_readme(&self, project_path: &Path) -> Result<String> {
        // Analyze project structure
        let structure = self.analyze_structure(project_path)?;
        
        // Extract key files
        let main_code = self.extract_main_files(project_path)?;
        
        let prompt = format!(
            r#"Generate a comprehensive README.md for this project.
            
            Project structure:
            ```
            {}
            ```
            
            Key code files:
            {}
            
            Include:
            1. Project description and purpose
            2. Installation instructions
            3. Usage examples
            4. Architecture overview
            5. Contributing guidelines
            6. License section
            
            Format as professional Markdown."#,
            structure,
            main_code
        );
        
        let ai = self.ai_factory.create().await?;
        let readme = ai.chat(&prompt, None).await?;
        
        let readme_path = project_path.join("README.md");
        std::fs::write(&readme_path, &readme)?;
        
        Ok(readme_path.to_string_lossy().to_string())
    }
    
    pub async fn generate_api_docs(&self, code_path: &Path) -> Result<String> {
        let code = std::fs::read_to_string(code_path)?;
        let ext = code_path.extension().and_then(|s| s.to_str()).unwrap_or("");
        
        let prompt = format!(
            r#"Generate API documentation for this {} code.
            
            Code:
            ```
            {}
            ```
            
            Include:
            1. Function signatures with parameters
            2. Return types and descriptions
            3. Examples for each function
            4. Error conditions
            
            Format as Markdown."#,
            ext.to_uppercase(),
            code
        );
        
        let ai = self.ai_factory.create().await?;
        let docs = ai.chat(&prompt, None).await?;
        
        let docs_path = code_path.with_extension("md");
        std::fs::write(&docs_path, &docs)?;
        
        Ok(docs_path.to_string_lossy().to_string())
    }
    
    fn analyze_structure(&self, path: &Path) -> Result<String> {
        let mut structure = String::new();
        
        for entry in WalkDir::new(path).max_depth(2) {
            let entry = entry?;
            let relative = entry.path().strip_prefix(path)?;
            structure.push_str(&format!("{}\n", relative.display()));
        }
        
        Ok(structure)
    }
    
    fn extract_main_files(&self, path: &Path) -> Result<String> {
        let mut files = String::new();
        
        // Look for common main files
        for main_file in &["lib/main.dart", "src/main.rs", "app.py", "index.js"] {
            let full_path = path.join(main_file);
            if full_path.exists() {
                let content = std::fs::read_to_string(full_path)?;
                files.push_str(&format!("// {}\n{}\n\n", main_file, content));
            }
        }
        
        Ok(files)
    }
}
```

## Tools & Dependencies
| Crate | Version | Purpose |
|-------|---------|---------|
| anthropic-rs | 0.2 | Claude API |
| async-openai | 0.16 | OpenAI API |
| secrecy | 0.8 | Secure key handling |
| ring | 0.17 | Request signing |
| dashmap | 5.5 | Thread-safe tracking |
| dialoguer | 0.11 | Interactive prompts |
| similar | 2.4 | Diff generation |
| walkdir | 2.4 | Project scanning |

## Testing Strategy
- **Unit**: Mock cloud APIs with wiremock (90% coverage)
- **Integration**: Test full refactor flow with temp files
- **Manual**: Run on real projects, verify cost tracking accuracy
- **Cost Limits**: Set $0.01 budget in tests, verify fallback triggers

## Deliverables
- ✅ Claude, OpenAI, Qwen adapters with cost tracking
- ✅ `kandil refactor <file> --goal="..."` with preview
- ✅ `kandil test generate <file> --coverage=80`
- ✅ `kandil switch-model <provider> <model>`
- ✅ `kandil show-costs` displays spending
- ✅ `kandil docs generate` creates README/API docs
- ✅ Automatic fallback to Ollama on cloud failures
- ✅ Budget warnings at $10, $25, $50 thresholds
- ✅ 90% test coverage on refactor/test modules

## Timeline Breakdown
- **Days 1-3**: Cloud adapters + cost tracking
- **Days 4-5**: Refactor engine + diff UI
- **Days 6-7**: Test generator + coverage analyzer
- **Days 8-9**: Model switching + fallback logic
- **Days 10-11**: Doc generator
- **Days 12-14**: Integration testing + cost validation

## Success Criteria
- Cloud AI responses within 3s
- Refactor preview shows accurate diffs
- Generated tests compile and run
- Cost tracking accurate to ±$0.01
- Fallback to Ollama <1s on failure
- No API keys in logs or errors
- `cargo tarpaulin` ≥90% coverage
- Manual test: Refactor real project, costs tracked correctly

## Potential Risks & Mitigations

| Risk | Mitigation |
|------|------------|
| API rate limiting | Implement exponential backoff (3 retries) |
| Cloud credentials leaked | Use secrecy crate + OS keychain only |
| High unexpected costs | Set $10 soft limit, $25 hard limit |
| Refactor breaks code | Require compilation check before apply |
| Generated tests don't compile | Run syntax check in verify_tests() |
| Fallback loops forever | Limit to 1 fallback per request |
| Cost double-counting | Use atomic operations in tracker |

---

**Next**: Proceed to PHASE_5_PROJECTS_CLOUD.md after Phase 4 cost validation with real API calls (use sandbox keys).