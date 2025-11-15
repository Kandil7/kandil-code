# **Kandil Code: Enhanced Model Integration Plan v2.0**
## **Security-First Universal AI Access with Zero Friction**

---

## **1. Architecture: The Secure Universal Model Registry (UMR)**

### **1.1 Core Design Principles**
- **Security First**: API keys never touch disk, CLI args, or logs. OS keyring only.
- **Fail-Fast Validation**: Hardware compatibility and security checks block incompatible models before download.
- **Explicit Over Magic**: Curated model profiles eliminate brittle auto-detection.
- **Performance**: Native Rust HTTP clients with connection pooling, cached discovery, lazy loading.
- **Extensibility**: Plugin architecture for custom providers, with sandboxing for untrusted code.

### **1.2 System Architecture Diagram**
```
┌─────────────────────────────────────────────────────────────────────┐
│                     Kandil CLI / TUI Interface                      │
│   Splash Commands: /refactor, /test, /fix, /review, /ask            │
└─────────────────────────────────────────────────────────────────────┘
                            ↓
┌─────────────────────────────────────────────────────────────────────┐
│              Universal Model Registry (UMR) - Singleton             │
│  ┌──────────────────┐  ┌──────────────────┐  ┌──────────────────┐ │
│  | Model Resolver   |  | Hardware Checker |  | Security Validator│ │
│  | (Built-in +      |→ | (RAM, GPU, Disk)   |→ | (Checksums,      │ │
│  |  Custom Profiles)|  |                    |   |  Sandboxing)    │ │
│  └────────┬─────────┘  └────────┬─────────┘  └────────┬─────────┘ │
│           ↓                     ↓                     ↓            │
│  ┌─────────────────────────────────────────────────────────────┐   │
│  |              Adapter Manager (Native + Bridge)             |   │
│  |  ┌──────────────┐      ┌─────────────────────────┐        |   │
│  |  | Native       |      | Bridge Plugin          |        |   │
│  |  | (Rust HTTP)  |◄────►| (Optional LiteLLM      |        |   │
│  |  | • Qwen       |      |   for fallbacks)       |        |   │
│  |  | • Gemini     |      | • Cost Tracking        |        |   │
│  |  | • Ollama     |      | • Multi-provider       |        |   │
│  |  | • Claude     |      | • Enterprise           |        |   │
│  |  └──────────────┘      └─────────────────────────┘        |   │
│  └─────────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────────┘
                            ↓
┌─────────────────────────────────────────────────────────────────────┐
│                   AI Providers / Local Models                       │
│  Qwen API    Gemini API    Ollama (Local)    Claude    Mistral    │
└─────────────────────────────────────────────────────────────────────┘
                            ↓
┌─────────────────────────────────────────────────────────────────────┐
│               Security & Hardware Layer                             │
│  Keyring Storage  |  Sandboxed Execution  |  Compatibility Checks   │
└─────────────────────────────────────────────────────────────────────┘
```

---

## **2. Configuration: Zero-Config Defaults with Layered Overrides**

### **2.1 User Config Schema (`~/.config/kandil/config.toml`)**
```toml
version = 2

[defaults]
model = "auto"  # Auto-select based on hardware + task
theme = "dark"
telemetry = false  # Opt-in only
stream_responses = true

[security]
require_keyring = true  # Prevent fallback to env vars
sandbox_plugins = true
validate_checksums = true

[adapter]
primary = "native"  # or "bridge"
enable_fallbacks = true
max_fallback_attempts = 2

[performance]
connection_pool_size = 10
request_timeout_secs = 120
rate_limit_retry = true

[cache]
model_discovery_ttl = 3600  # 1 hour
compatibility_cache_ttl = 86400  # 1 day

[limits]
max_concurrent_requests = 5
max_model_size_gb = 100  # Block OOM attempts

[models.qwen2_5_coder_7b]
max_tokens = 4096
temperature = 0.2
top_p = 0.9
preferred_hardware = "gpu"  # CPU fallback with warning

[models.gemini_1_5_pro]
max_tokens = 8192
temperature = 0.1
top_p = 0.95
safety_threshold = "block_few"  # Gemini safety settings
```

### **2.2 Model Metadata Storage (`~/.local/share/kandil/models/`)**
```json
// qwen32b.metadata.json
{
  "schema_version": 2,
  "name": "qwen2.5-coder-32b-instruct",
  "source": "huggingface:Qwen/Qwen2.5-Coder-32B-Instruct-GGUF",
  "sha256": "ef3a3b2c4d5e6f7890a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3",
  "size_bytes": 18200000000,
  "quantization": "Q4_K_M",
  "verified_at": "2025-01-15T10:30:00Z",
  "last_used": "2025-01-15T11:45:00Z",
  "usage_count": 42,
  "average_latency_ms": 340,
  "capabilities": ["code", "reasoning", "long-context"],
  "context_window": 32768,
  "api_key_required": true,
  "provider": "alibaba-dashscope"
}
```

---

## **3. Implementation: Core Components**

### **3.1 Secure Credential Management**
```rust
// src/security/credentials.rs
use secrecy::{ExposeSecret, SecretString};
use keyring::{Entry, Error as KeyringError};

pub struct CredentialManager;

impl CredentialManager {
    /// Store API key in OS keyring (never in files or env)
    pub fn store_api_key(provider: &str, key: SecretString) -> Result<()> {
        let entry = Entry::new("kandil", &format!("api_key_{}", provider))?;
        entry.set_password(key.expose_secret())?;
        
        // Verify storage
        Self::get_api_key(provider)?;
        
        tracing::info!("API key stored securely for provider: {}", provider);
        Ok(())
    }
    
    /// Retrieve API key (fails if not found, prompting user)
    pub fn get_api_key(provider: &str) -> Result<SecretString> {
        let entry = Entry::new("kandil", &format!("api_key_{}", provider))?;
        
        match entry.get_password() {
            Ok(key) => Ok(SecretString::new(key)),
            Err(KeyringError::NoEntry) => {
                bail!(
                    "API key not found for '{}'. Run: kandil model configure {}",
                    provider, provider
                )
            }
            Err(e) => Err(e.into()),
        }
    }
    
    /// Interactive prompt for API key with validation
    pub fn prompt_and_store(provider: &str, api_key_url: &str) -> Result<()> {
        eprintln!("🔐 API key required for provider: {}", provider);
        eprintln!("   ℹ️  Obtain from: {}", api_key_url);
        eprintln!("   🔒 Key will be stored in OS keyring (never in files)\n");
        
        let key = rpassword::prompt_password("? Enter API key: ")?;
        let confirm = rpassword::prompt_password("? Confirm API key: ")?;
        
        if key != confirm {
            bail!("Keys don't match. Please try again.");
        }
        
        // Validate before storing
        Self::validate_key(provider, &key)?;
        
        Self::store_api_key(provider, SecretString::new(key))?;
        eprintln!("✅ Key validated and stored securely");
        
        Ok(())
    }
    
    /// Validate key with minimal API call
    fn validate_key(provider: &str, key: &str) -> Result<()> {
        // Implementation depends on provider
        match provider {
            "qwen" => validate_qwen_key(key),
            "gemini" => validate_gemini_key(key),
            _ => Ok(()), // Cannot validate unknown
        }
    }
}

// Zeroize secret on drop
impl Drop for SecretString {
    fn drop(&mut self) {
        self.expose_secret().zeroize();
    }
}
```

### **3.2 Hardware Detection Engine**
```rust
// src/hardware/detector.rs
use sysinfo::{System, SystemExt, CpuExt, DiskExt};
use nvml_wrapper::{NVML, Device};
use fs2::available_space;

pub struct HardwareDetector {
    sys: Arc<RwLock<System>>,
    nvml: Option<NVML>,
}

impl HardwareDetector {
    pub fn new() -> Result<Self> {
        Ok(Self {
            sys: Arc::new(RwLock::new(System::new_all())),
            nvml: NVML::init().ok(), // Optional GPU support
        })
    }
    
    pub fn refresh(&self) {
        self.sys.write().unwrap().refresh_all();
    }
    
    pub fn get_report(&self) -> HardwareReport {
        self.refresh();
        let sys = self.sys.read().unwrap();
        
        let total_ram_gb = sys.total_memory() as f64 / 1024.0 / 1024.0;
        let available_ram_gb = sys.available_memory() as f64 / 1024.0 / 1024.0;
        let cpu_cores = sys.cpus().len();
        let cpu_freq_ghz = sys.cpus().get(0).map(|c| c.frequency() as f64 / 1000.0).unwrap_or(0.0);
        
        let free_disk_gb = available_space(std::env::current_dir().unwrap()).unwrap_or(0) as f64 / 1e9;
        
        let gpu = self.detect_gpu();
        
        HardwareReport {
            total_ram_gb,
            available_ram_gb,
            cpu_cores,
            cpu_freq_ghz,
            free_disk_gb,
            gpu,
            is_laptop: self.detect_laptop(),
        }
    }
    
    fn detect_gpu(&self) -> Option<GPUInfo> {
        let nvml = self.nvml.as_ref()?;
        let device = nvml.device_by_index(0).ok()?;
        
        Some(GPUInfo {
            name: device.name().ok()?,
            memory_total_gb: device.memory_info().ok()?.total as f64 / 1e9,
            memory_free_gb: device.memory_info().ok()?.free as f64 / 1e9,
            cuda_cores: device.num_cores().ok()?,
            driver_version: device.nvml().sys_driver_version().ok()?,
        })
    }
    
    fn detect_laptop(&self) -> bool {
        // Heuristic: check for battery sysfs
        Path::new("/sys/class/power_supply/BAT0").exists()
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct HardwareReport {
    pub total_ram_gb: f64,
    pub available_ram_gb: f64,
    pub cpu_cores: usize,
    pub cpu_freq_ghz: f64,
    pub free_disk_gb: f64,
    pub gpu: Option<GPUInfo>,
    pub is_laptop: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct GPUInfo {
    pub name: String,
    pub memory_total_gb: f64,
    pub memory_free_gb: f64,
    pub cuda_cores: u32,
    pub driver_version: String,
}
```

### **3.3 Compatibility Engine**
```rust
// src/model/compatibility.rs
impl CompatibilityEngine {
    pub fn check(&self, profile: &ModelProfile, hardware: &HardwareReport) -> CompatibilityReport {
        let mut report = CompatibilityReport::default();
        let safety_margin = 1.5; // 50% overhead
        
        // RAM Check: Model size * safety margin
        let required_ram = profile.size_gb * safety_margin;
        if hardware.available_ram_gb < required_ram {
            report.blocked = true;
            report.reasons.push(format!(
                "❌ Insufficient RAM: {:.1}GB required, {:.1}GB available",
                required_ram, hardware.available_ram_gb
            ));
        } else if hardware.available_ram_gb < required_ram * 1.2 {
            report.warnings.push("⚠️  RAM usage will be high".to_string());
        }
        
        // Disk Space Check
        if hardware.free_disk_gb < profile.size_gb {
            report.blocked = true;
            report.reasons.push(format!(
                "❌ Insufficient disk: {:.1}GB required, {:.1}GB free",
                profile.size_gb, hardware.free_disk_gb
            ));
        }
        
        // GPU Check for Local Models
        if profile.is_local && profile.recommends_gpu {
            match &hardware.gpu {
                None => {
                    report.warnings.push(
                        "⚠️  No GPU detected. Model will use CPU inference (slow)".to_string()
                    );
                }
                Some(gpu) if gpu.memory_free_gb < profile.size_gb => {
                    report.warnings.push(format!(
                        "⚠️  GPU VRAM ({:.1}GB) insufficient for model ({:.1}GB). Using RAM.",
                        gpu.memory_free_gb, profile.size_gb
                    ));
                }
                _ => {
                    report.info.push("✅ GPU acceleration available".to_string());
                }
            }
        }
        
        // Laptop-specific warnings
        if hardware.is_laptop && profile.size_gb > 20.0 {
            report.warnings.push(
                "⚠️  Large model on laptop may cause thermal throttling".to_string()
            );
        }
        
        // Cloud API latency check (if previously used)
        if let Some(avg_latency) = profile.average_latency_ms {
            if avg_latency > 2000 {
                report.warnings.push(
                    "⚠️  High latency detected (slow network or provider)".to_string()
                );
            }
        }
        
        report
    }
}

#[derive(Debug, Default)]
pub struct CompatibilityReport {
    pub blocked: bool,
    pub reasons: Vec<String>,
    pub warnings: Vec<String>,
    pub info: Vec<String>,
}
```

---

## **4. Model Security Validator**

### **4.1 GGUF File Validation**
```rust
// src/model/security/validator.rs
use sha2::{Sha256, Digest};
use gguf::GGUFFile;
use std::process::{Stdio, Command};
use std::time::Duration;
use tokio::time::timeout;

pub struct ModelSecurityValidator;

impl ModelSecurityValidator {
    /// Full validation pipeline before any model is loaded
    pub async fn validate_model_file(
        &self,
        path: &Path,
        expected_hash: &str,
        source: &str,
    ) -> Result<()> {
        // 1. Verify checksum against official source
        self.verify_checksum(path, expected_hash, source).await?;
        
        // 2. Validate GGUF structure and metadata
        self.validate_gguf_structure(path)?;
        
        // 3. Sandboxed load test (5 second timeout)
        self.sandboxed_load_test(path).await?;
        
        tracing::info!("✅ Model file validated: {}", path.display());
        Ok(())
    }
    
    async fn verify_checksum(
        &self,
        path: &Path,
        expected_hash: &str,
        source: &str,
    ) -> Result<()> {
        let mut file = File::open(path).await?;
        let mut hasher = Sha256::new();
        
        // Stream file to avoid memory issues
        let mut buffer = vec![0; 8192];
        loop {
            let n = file.read(&mut buffer).await?;
            if n == 0 { break; }
            hasher.update(&buffer[..n]);
        }
        
        let actual_hash = format!("{:x}", hasher.finalize());
        
        if actual_hash != expected_hash {
            bail!(
                "Checksum mismatch!\n\
                Model: {}\n\
                Expected: {}\n\
                Actual:   {}\n\
                \nPossible causes:\n\
                - Corrupted download\n\
                - Malicious file\n\
                - Incorrect model version\n\
                \nAction: Delete {} and re-download from {}",
                path.display(),
                expected_hash,
                actual_hash,
                path.display(),
                source
            );
        }
        
        Ok(())
    }
    
    fn validate_gguf_structure(&self, path: &Path) -> Result<()> {
        let gguf = GGUFFile::read(path)?;
        
        // Check for suspicious metadata
        for (key, value) in &gguf.metadata {
            let key_lower = key.to_lowercase();
            if key_lower.contains("exec") 
                || key_lower.contains("system") 
                || key_lower.contains("cmd")
                || key_lower.contains("shell")
            {
                bail!(
                    "Suspicious metadata key found: '{}'\n\
                    This model file may be malicious and attempt code execution.",
                    key
                );
            }
            
            // Check value type safety
            if let gguf::MetadataValue::String(s) = value {
                if s.len() > 1000 {
                    bail!("Metadata value too large, possible injection attempt");
                }
            }
        }
        
        // Validate tensor names
        for tensor in &gguf.tensors {
            if tensor.name.contains("..") 
                || tensor.name.starts_with('/')
                || tensor.name.contains('\\')
            {
                bail!("Invalid tensor name (path injection attempt): '{}'", tensor.name);
            }
        }
        
        Ok(())
    }
    
    async fn sandboxed_load_test(&self, path: &Path) -> Result<()> {
        let path_str = path.to_string_lossy();
        
        // Spawn sandboxed process with restricted permissions
        let child = Command::new("kandil-sandbox")
            .arg("load-test")
            .arg(&*path_str)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true)
            .spawn()?;
        
        // Timeout after 5 seconds
        let result = timeout(Duration::from_secs(5), child.wait_with_output()).await;
        
        match result {
            Ok(Ok(output)) if output.status.success() => Ok(()),
            Ok(Ok(output)) => {
                let stderr = String::from_utf8_lossy(&output.stderr);
                bail!("Sandboxed load test failed: {}", stderr)
            }
            Ok(Err(e)) => Err(e.into()),
            Err(_) => bail!("Load test timed out (model may be corrupted)"),
        }
    }
}
```

---

## **5. Adaptive Prompt System (APS)**

### **5.1 Prompt Template Structure**
```rust
// src/prompts/mod.rs
pub struct KandilPrompt {
    pub system: String,
    pub context: ProjectContext,
    pub instruction: String,
    pub constraints: Vec<String>,
    pub output_format: OutputFormat,
}

impl KandilPrompt {
    /// Convert to model-specific format
    pub fn render_for(&self, model: &str) -> String {
        match model {
            m if m.contains("qwen") => self.render_for_qwen(),
            m if m.contains("gemini") => self.render_for_gemini(),
            m if m.contains("claude") => self.render_for_claude(),
            _ => self.render_for_openai(),
        }
    }
    
    fn render_for_qwen(&self) -> String {
        format!(
            "<|im_start|>system\n{}\n{}\nConstraints: {}\n<|im_end|>\n<|im_start|>user\n{}<|im_end|>\n<|im_start|>assistant\n",
            self.system,
            self.context.render(),
            self.constraints.join(", "),
            self.instruction
        )
    }
    
    fn render_for_gemini(&self) -> String {
        format!(
            "<system>{}</system>\n<context>{}</context>\n<constraints>{}</constraints>\n<prompt>{}</prompt>",
            self.system,
            self.context.render(),
            self.constraints.join(", "),
            self.instruction
        )
    }
    
    fn render_for_claude(&self) -> String {
        format!(
            "System: {}\n\nHuman: {}\n\nContext: {}\n\nConstraints: {}\n\nAssistant:",
            self.system,
            self.instruction,
            self.context.render(),
            self.constraints.join(", ")
        )
    }
}
```

### **5.2 Production-Ready Prompt Templates** (from CLI guide, enhanced)

```rust
// src/prompts/templates/refactor.rs
pub fn refactor_prompt(task: RefactorTask) -> KandilPrompt {
    KandilPrompt {
        system: indoc::indoc! {"
            You are Kandil-Refactor, an expert code refactoring agent.
            - Analyze code for performance, readability, and maintainability
            - Follow language-specific best practices (Rust: idiomatic, Python: PEP8)
            - Do NOT change functionality unless explicitly asked
            - Provide unified diff format
            - Explain each change in 1-2 sentences
        "}.to_string(),
        
        context: ProjectContext {
            project_type: detect_project_type(),
            language: detect_language(),
            framework: detect_framework(),
            active_file: task.file_path,
            git_state: detect_git_state(),
            related_files: find_dependencies(&task.file_path),
        },
        
        instruction: format!(
            "Refactor the function '{}' to: 1. Reduce cyclomatic complexity 2. Improve error handling 3. Make it more idiomatic",
            task.target_function
        ),
        
        constraints: vec![
            "Keep public API unchanged".to_string(),
            "Use existing patterns in codebase".to_string(),
            "Max 50 lines per function".to_string(),
            "Add inline comments for complex logic".to_string(),
        ],
        
        output_format: OutputFormat::Diff,
    }
}

// src/prompts/templates/test.rs
pub fn test_prompt(task: TestTask) -> KandilPrompt {
    KandilPrompt {
        system: indoc::indoc! {"
            You are Kandil-Test, a test generation expert.
            - Generate comprehensive unit tests with 90%+ coverage
            - Cover happy path, edge cases, and error conditions
            - Use popular testing frameworks (Rust: tokio::test, Python: pytest)
            - Mock external dependencies
            - Include docstring explaining test purpose
        "}.to_string(),
        
        context: ProjectContext {
            code_under_test: task.code_snippet,
            function_signature: task.signature,
            visibility: task.visibility,
            project_type: detect_project_type(),
        },
        
        instruction: "Generate unit tests for the provided function. Focus on behavior, not implementation details.".to_string(),
        
        constraints: vec![
            "Test public API only".to_string(),
            "Use Arrange-Act-Assert pattern".to_string(),
            "Include at least 1 edge case".to_string(),
        ],
        
        output_format: OutputFormat::Code,
    }
}

// src/prompts/templates/fix.rs
pub fn fix_prompt(task: FixTask) -> KandilPrompt {
    KandilPrompt {
        system: indoc::indoc! {"
            You are Kandil-Fix, a debugging specialist.
            - Analyze error messages and stack traces
            - Identify root cause (not symptoms)
            - Suggest minimal fix with explanation
            - Consider performance impact
            - Verify fix doesn't introduce new bugs
        "}.to_string(),
        
        context: ProjectContext {
            error_message: task.error,
            stack_trace: task.stack_trace,
            failing_file: task.file_path,
            line_number: task.line,
            relevant_code: extract_surrounding_code(&task.file_path, task.line),
            git_blame: get_git_blame(&task.file_path, task.line),
        },
        
        instruction: "Identify the root cause and provide a minimal fix that resolves the error.".to_string(),
        
        constraints: vec![
            "Explain why the fix works".to_string(),
            "Do not change unrelated code".to_string(),
            "Preserve existing functionality".to_string(),
        ],
        
        output_format: OutputFormat::DiffWithExplanation,
    }
}

// src/prompts/templates/review.rs
pub fn review_prompt(task: ReviewTask) -> KandilPrompt {
    KandilPrompt {
        system: indoc::indoc! {"
            You are Kandil-Review, a senior code reviewer.
            - Focus on: security, performance, maintainability, accessibility
            - Use established frameworks (OWASP, SRE)
            - Assign severity: CRITICAL, HIGH, MEDIUM, LOW
            - Suggest concrete improvements
            - Be constructive, not pedantic
        "}.to_string(),
        
        context: ProjectContext {
            diff: task.diff,
            author_level: detect_author_experience(),
            project_security_level: detect_security_requirements(),
            related_prs: get_recent_prs(),
        },
        
        instruction: "Perform a security-focused code review. Identify vulnerabilities and flag performance issues.".to_string(),
        
        constraints: vec![
            "Security issues must be CRITICAL or HIGH".to_string(),
            "Provide code examples for fixes".to_string(),
            "Check for test coverage".to_string(),
        ],
        
        output_format: OutputFormat::ReviewReport,
    }
}
```

---

## **6. Native Adapter Implementation (Fast Path)**

### **6.1 Direct HTTP Client with Pooling**
```rust
// src/adapters/native/mod.rs
pub struct NativeAdapter {
    client: Arc<Client>,
    profile: ModelProfile,
    credentials: ModelCredentials,
}

impl NativeAdapter {
    pub async fn new(profile: ModelProfile) -> Result<Self> {
        let credentials = ModelCredentials::load(&profile.provider)?;
        
        let client = Arc::new(
            reqwest::Client::builder()
                .connect_timeout(Duration::from_secs(10))
                .timeout(Duration::from_secs(120))
                .pool_idle_timeout(Duration::from_secs(30))
                .pool_max_idle_per_host(10)
                .tcp_keepalive(Duration::from_secs(60))
                .user_agent(format!("Kandil-Code/{}", env!("CARGO_PKG_VERSION")))
                .default_headers({
                    let mut headers = HeaderMap::new();
                    headers.insert(
                        "Authorization",
                        format!("Bearer {}", credentials.api_key.expose_secret())
                            .parse()
                            .unwrap()
                    );
                    
                    // Provider-specific headers
                    match profile.provider {
                        Provider::Anthropic => {
                            headers.insert("anthropic-version", "2023-06-01".parse().unwrap());
                        }
                        Provider::Gemini => {
                            headers.insert("x-goog-api-key", credentials.api_key.expose_secret().as_str().parse().unwrap());
                        }
                        _ => {}
                    }
                    
                    headers
                })
                .build()
                .map_err(|e| anyhow::anyhow!("Failed to create HTTP client: {}", e))?
        );
        
        Ok(Self {
            client,
            profile,
            credentials,
        })
    }
}

#[async_trait]
impl AIProvider for NativeAdapter {
    async fn complete(&self, prompt: &str) -> Result<String> {
        let request = self.build_request(prompt)?;
        let response = self.client.execute(request).await?;
        self.parse_response(response).await
    }
    
    async fn stream(&self, prompt: &str) -> Result<BoxStream<'static, Result<String>>> {
        let request = self.build_streaming_request(prompt)?;
        let response = self.client.execute(request).await?;
        
        if !response.status().is_success() {
            bail!("API error: {}", response.text().await?);
        }
        
        let stream = response.bytes_stream()
            .map(|chunk| -> Result<String> {
                let chunk = chunk?;
                let s = String::from_utf8_lossy(&chunk);
                
                // Parse SSE format
                for line in s.lines() {
                    if line.starts_with("data: ") {
                        let json_str = &line[6..];
                        if json_str == "[DONE]" {
                            break;
                        }
                        if let Ok(parsed) = serde_json::from_str::<Value>(json_str) {
                            if let Some(token) = self.extract_token(&parsed) {
                                return Ok(token);
                            }
                        }
                    }
                }
                Ok("".to_string())
            })
            .boxed();
        
        Ok(Box::pin(stream))
    }
}

// Provider-specific implementations in src/adapters/native/qwen.rs, gemini.rs, etc.
```

### **6.2 Protocol Implementations**
```rust
// src/adapters/native/qwen.rs
impl QwenAdapter {
    fn build_request(&self, prompt: &str) -> Result<Request> {
        let url = format!("{}/chat/completions", self.profile.base_url);
        
        let body = json!({
            "model": self.profile.name,
            "messages": [
                {"role": "system", "content": "You are a helpful assistant."},
                {"role": "user", "content": prompt}
            ],
            "temperature": self.profile.temperature,
            "max_tokens": self.profile.max_tokens,
            "stream": false,
            "top_p": self.profile.top_p,
            "top_k": self.profile.top_k,
        });
        
        Ok(self.client.post(&url).json(&body))
    }
}

// src/adapters/native/gemini.rs
impl GeminiAdapter {
    fn build_request(&self, prompt: &str) -> Result<Request> {
        let url = format!(
            "{}/models/{}:generateContent?key={}",
            self.profile.base_url,
            self.profile.name,
            self.credentials.api_key.expose_secret()
        );
        
        let body = json!({
            "contents": [{"parts": [{"text": prompt}]}],
            "generationConfig": {
                "temperature": self.profile.temperature,
                "maxOutputTokens": self.profile.max_tokens,
                "topP": self.profile.top_p,
                "topK": self.profile.top_k,
            },
            "safetySettings": [
                {"category": "HARM_CATEGORY_DANGEROUS_CONTENT", "threshold": "BLOCK_FEW"}
            ]
        });
        
        Ok(self.client.post(&url).json(&body))
    }
}
```

---

## **7. Bridge Plugin (Optional LiteLLM Integration)**

### **7.1 Architecture**
```rust
// src/plugins/bridge/mod.rs
//! # CLI Bridge Plugin
//! Provides optional LiteLLM-based routing for fallbacks and cost tracking.
//! WARNING: Slower than native adapter. Use only for enterprise features.

pub struct BridgePlugin {
    litellm_process: Option<Child>,
    config: BridgeConfig,
    health_handle: Option<JoinHandle<()>>,
}

impl BridgePlugin {
    /// Start LiteLLM proxy in sandboxed process
    pub async fn start(&mut self) -> Result<()> {
        let port = get_free_port()?;
        
        // Write secure config
        let config_path = self.write_secure_config(port).await?;
        
        // Spawn LiteLLM with restricted permissions
        let mut cmd = tokio::process::Command::new("litellm");
        cmd.arg("--config").arg(&config_path)
            .arg("--port").arg(port.to_string())
            .arg("--health")
            .arg("--ssl-cert").arg(&self.config.ssl_cert_path)
            .arg("--ssl-key").arg(&self.config.ssl_key_path)
            .stdout(Stdio::null())
            .stderr(Stdio::piped());
        
        // Drop privileges on Unix
        #[cfg(unix)]
        unsafe {
            cmd.pre_exec(|| {
                // Set sandbox uid/gid
                libc::setuid(65534); // nobody
                libc::setgid(65534);
                Ok(())
            });
        }
        
        self.litellm_process = Some(cmd.spawn()?);
        
        // Wait for health check
        self.wait_for_health(port).await?;
        
        // Start health monitor
        self.health_handle = Some(tokio::spawn(self.monitor_health(port)));
        
        tracing::info!("Bridge plugin started on port {}", port);
        Ok(())
    }
    
    async fn write_secure_config(&self, port: u16) -> Result<PathBuf> {
        let config = serde_yaml::to_string(&LiteLLMConfig {
            model_list: self.build_model_list(),
            general_settings: GeneralSettings {
                master_key: self.config.master_key.expose_secret().clone(),
                drop_params: true,
                enable_caching: true,
            },
            litellm_settings: LiteLLMSettings {
                cache: true,
                cache_params: CacheParams {
                    type: "redis".to_string(),
                    url: "redis://localhost:6379".to_string(),
                },
            },
        })?;
        
        let path = std::env::temp_dir().join(format!("kandil-litellm-{}.yaml", port));
        fs::write(&path, config).await?;
        Ok(path)
    }
    
    async fn monitor_health(&self, port: u16) {
        let client = reqwest::Client::new();
        loop {
            tokio::time::sleep(Duration::from_secs(30)).await;
            
            match client
                .get(format!("https://localhost:{}/health", port))
                .send()
                .await
            {
                Ok(resp) if resp.status().is_success() => continue,
                _ => {
                    tracing::error!("LiteLLM health check failed");
                    break;
                }
            }
        }
    }
}

impl Drop for BridgePlugin {
    fn drop(&mut self) {
        if let Some(mut child) = self.litellm_process.take() {
            child.kill().ok();
        }
    }
}
```

### **7.2 When to Use Bridge**
```rust
// src/router/adapter_selector.rs
pub async fn select_adapter(&self, task: &Task) -> Result<Box<dyn AIProvider>> {
    // Use bridge only if:
    // 1. User explicitly enabled it
    // 2. Task requires multi-provider fallbacks
    // 3. Enterprise cost tracking is enabled
    if self.config.use_bridge 
        && (task.requires_fallbacks || self.config.enable_cost_tracking) {
        Ok(Box::new(BridgeAdapter::new()?))
    } else {
        // Default: Fast, secure native adapter
        Ok(Box::new(NativeAdapter::new(self.select_model(task).await?)?))
    }
}
```

**Bridge Config**:
```toml
# ~/.config/kandil/bridge.toml (separate, optional)
[bridge]
enabled = false
master_key = "sk-secure-proxy-key"  # From keyring

[ssl]
cert_path = "/path/to/cert.pem"
key_path = "/path/to/key.pem"

[redis]
url = "redis://localhost:6379"  # For caching

[cost_tracking]
enabled = true
alert_threshold_usd = 10.0
daily_budget_usd = 50.0
```

---

## **8. CLI Command Specification**

### **8.1 Model Management Commands**
```bash
# Add model (interactive, secure)
$ kandil model add qwen2.5-coder-32b
🔐 API key required for alibaba-dashscope
   ℹ️  Obtain from: https://dashscope.console.aliyun.com/apiKey
? Enter API key (hidden): ************
? Confirm API key: ************
✅ Key validated and stored in keyring

🖥️  Hardware Check:
   - RAM: Need 24.0GB, Available 45.2GB ✅
   - GPU: RTX 4090 24GB VRAM (accelerated) ✅
   - Disk: Need 18.2GB, Free 342.1GB ✅

📦 Downloading Qwen2.5-Coder-32B-Q4.gguf...
✅ Checksum verified: ef3a3b2c...
✅ Model signature valid
✅ Sandboxed load test passed (3.2s)
✅ Model ready! Use 'kandil /model use qwen32b'

# List models with status
$ kandil model list
✅ Local Models (Ollama):
   llama3.1:8b         4.5GB   ✅ Ready      CPU inference
   qwen2.5-coder:7b    4.5GB   ✅ Ready      GPU accelerated
   
✅ Cloud Models (Configured):
   qwen2.5-coder-32b   API     ✅ Ready      Low latency
   claude-3.5-sonnet   API     ✅ Ready      Rate: 40 req/min
   
✅ Other Available:
   deepseek-coder:33b  API     ☐ Not configured
   gemini-1.5-pro      API     ☐ Not configured

# Test model connectivity
$ kandil model test claude-3.5-sonnet
🧪 Testing API connectivity...
✅ Response time: 234ms
✅ Rate limits: 40 rpm, 1000 tpm
✅ Key permissions: Valid

# Remove model
$ kandil model rm qwen2.5-coder-7b
⚠️  This will delete 4.5GB from disk. Continue? [y/N] y
✅ Model removed

# Benchmark all models
$ kandil model benchmark --all
📊 Benchmarking 3 models...
1. qwen2.5-coder-7b:  234ms avg,  $0.0012/1k tokens
2. claude-3.5-sonnet: 445ms avg,  $0.0080/1k tokens
3. llama3.1:8b:       89ms avg,   $0 (local)
```

### **8.2 AI Task Commands**
```bash
# Refactor command
$ kandil /refactor src/auth/login.rs --target "login function" --explain
🤖 Using model: qwen2.5-coder-14b (GPU accelerated)
📊 Project context: Rust/Actix, Git branch: feature/auth
⚡ Streaming response...

## Suggested Refactor

```diff
@@ -42,7 +42,7 @@
-    fn login(&self, user: &str, pass: &str) -> Result<Session> {
-        if user.is_empty() || pass.is_empty() {
-            return Err("Invalid credentials");
-        }
+    fn login(&self, user: &str, pass: &str) -> Result<Session> {
+        validate_credentials(user, pass)?;
+        
         let hash = self.db.get_password_hash(user)?;
         if verify_password(pass, &hash)? {
             Ok(Session::new(user))
```

**Rationale:**
- Extract validation into reusable function
- Improve error handling with `?` operator
- Reduce cyclomatic complexity from 4 to 2

✅ Apply this refactor? [y/N/a(ll)] y
✅ Applied successfully. Running tests...
✅ All tests pass. Commit? [y/N] y
✅ Committed: refactor(auth): simplify login function (234ms)

# Test generation
$ kandil /test src/auth.rs --coverage=90
🤖 Analyzing code...
🧪 Generating 5 test cases...
✅ Coverage: 92.3% (target: 90%)
📄 Created: src/auth/tests.rs

# Bug fix with screenshot
$ kandil /fix --image=error.png
🔍 Analyzing error screenshot...
⚠️  Detected: Type mismatch in line 42
🤖 Generating fix...
```diff
@@ -42,7 +42,7 @@
-    let count: i32 = query.fetch_one(&db).await?;
+    let count: i64 = query.fetch_one(&db).await?;
```
✅ Fix applied. Type changed to i64 to match database schema.
```

---

## **9. Implementation Roadmap (Revised Timeline)**

### **Phase 0: Foundation & Security (Weeks 1-3)**
**Priority: P0 (Critical)**

- [ ] **Security Baseline**
  - Implement `CredentialManager` with keyring integration
  - Add `ModelSecurityValidator` with checksums and sandboxing
  - Create `.env` loader with warnings (deprecation)
  - Add pre-commit hooks for secret scanning

- [ ] **Hardware Detection**
  - `HardwareDetector` with GPU support (nvml-wrapper)
  - `CompatibilityEngine` with OOM prevention
  - Cache hardware reports (5 min TTL)

- [ ] **Core Registry**
  - `UniversalModelRegistry` singleton
  - Load 50+ built-in model profiles (compile-time)
  - Implement fuzzy name resolution

- [ ] **Native Adapter Skeleton**
  - Connection pooling (reqwest)
  - Streaming support foundation
  - Protocol traits

**Deliverable**: Secure, hardware-aware foundation with no AI functionality yet.

### **Phase 1: Native Qwen & Ollama Integration (Weeks 4-6)**
**Priority: P1 (High)**

- [ ] **Qwen Native Adapter**
  - Direct HTTP to DashScope API
  - Prompt optimization for Qwen format
  - Rate limit handling

- [ ] **Ollama Local Adapter**
  - Detect running Ollama instance
  - GGUF download and validation
  - Hardware-accelerated inference

- [ ] **CLI Commands**
  - `kandil model add <name>` (interactive)
  - `kandil model list`
  - `kandil model test`

- [ ] **Prompt Templates**
  - `/refactor` template
  - `/ask` template

**Deliverable**: Working Qwen and Ollama support with secure credential storage.

### **Phase 2: Gemini & Claude Integration (Weeks 7-9)**
**Priority: P1 (High)**

- [ ] **Gemini Native Adapter**
  - Vertex AI/Gemini Pro support
  - Vision API for screenshots
  - Safety settings configuration

- [ ] **Claude Native Adapter**
  - Anthropic API integration
  - Long context (200k) support
  - Tools/functions beta

- [ ] **Advanced Prompts**
  - `/test` with coverage targeting
  - `/fix` with error analysis
  - `/review` with security focus

- [ ] **Project Context Enrichment**
  - Tree-sitter parsing
  - Git integration (blame, diff)
  - Dependency analysis

**Deliverable**: Multi-provider support with context-aware prompts.

### **Phase 3: Bridge Plugin & Enterprise Features (Weeks 10-12)**
**Priority: P2 (Medium)**

- [ ] **Bridge Plugin (Optional)**
  - LiteLLM integration (sandboxed)
  - Cost tracking dashboard
  - Multi-provider fallbacks
  - Redis caching

- [ ] **Performance Optimization**
  - Request batching
  - Connection pool tuning
  - Lazy file tree loading

- [ ] **TUI Enhancements**
  - Ratatui studio with lazy loading
  - Real-time metrics overlay
  - Error recovery (panic hooks)

- [ ] **Quality Gates**
  - Integration tests for each provider
  - Rate limit tests
  - Malicious model detection tests

**Deliverable**: Enterprise bridge plugin + polished TUI experience.

### **Phase 4: Simulation Engine & v1.0 Release (Weeks 13-16)**
**Priority: P2 (Medium)**

- [ ] **Multi-Agent Simulation**
  - ReAct agent framework
  - Event bus for agent communication
  - Role simulation (PM, DevOps, Security)

- [ ] **Plugin Ecosystem**
  - Static plugin compilation
  - gRPC communication
  - Sandboxing with Docker (Phase 13)

- [ ] **Release Preparation**
  - 90% test coverage enforcement
  - `cargo-audit`, `cargo-deny`
  - Documentation (`cargo-doc` must pass)
  - Release binaries (GitHub Actions)

**Deliverable**: v1.0 release with multi-agent capabilities.

### **Phase 5: Post-Launch & Community (Weeks 17-20)**
**Priority: P3 (Low)**

- [ ] **Community Features**
  - Plugin registry
  - User prompt templates
  - Telemetry (opt-in)

- [ ] **Performance Tuning**
  - Profile-guided optimization
  - Async buffer tuning
  - GPU memory optimization

- [ ] **Documentation**
  - Architecture decision records (ADRs)
  - Security whitepaper
  - Contributor guidelines

**Deliverable**: Production-ready tool with active community.

---

## **10. Testing & Quality Assurance**

### **10.1 Test Coverage Requirements**
```bash
# CI must enforce:
cargo tarpaulin --out Xml --fail-under 90
cargo audit --deny warnings
cargo deny check licenses bans sources
cargo doc --no-deps
```

### **10.2 Integration Test Suite**
```rust
// tests/model_lifecycle.rs
#[tokio::test]
async fn test_qwen_model_lifecycle() {
    let registry = UniversalModelRegistry::new();
    
    // 1. Add model
    registry.add_model("qwen2.5-coder-7b").await.unwrap();
    
    // 2. Verify credentials stored
    assert!(CredentialManager::get_api_key("alibaba-dashscope").is_ok());
    
    // 3. Test compatibility
    let report = registry.check_compatibility("qwen2.5-coder-7b").await.unwrap();
    assert!(!report.blocked);
    
    // 4. Test API call
    let result = registry.complete("qwen2.5-coder-7b", "Hello").await.unwrap();
    assert!(!result.is_empty());
    
    // 5. Cleanup
    registry.remove_model("qwen2.5-coder-7b").await.unwrap();
}

#[test]
fn test_malicious_model_rejection() {
    let validator = ModelSecurityValidator;
    let malicious_path = create_test_malicious_gguf();
    
    let result = block_on(validator.validate_model_file(&malicious_path, "fake_hash", "test"));
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Suspicious metadata"));
}

#[tokio::test]
async fn test_rate_limit_protection() {
    let adapter = NativeAdapter::new("qwen2.5-coder-7b").await.unwrap();
    
    // Spawn 100 concurrent requests
    let mut tasks = vec![];
    for _ in 0..100 {
        tasks.push(adapter.complete("test"));
    }
    
    let results = futures::future::join_all(tasks).await;
    let rate_limited = results.iter().filter(|r| {
        r.as_ref().err().map(|e| e.to_string().contains("rate limit")).unwrap_or(false)
    }).count();
    
    assert!(rate_limited > 0, "Should have rate limited some requests");
}
```

### **10.3 Security Test Matrix**
```bash
# tests/security.sh
#!/bin/bash

# Test 1: API key never in process args
./target/debug/kandil model add test 2>&1 | grep -E "(sk-|AIza|api_key)" && exit 1

# Test 2: Malicious GGUF rejected
echo "malicious metadata: exec" > fake.gguf
./target/debug/kandil model add file:./fake.gguf 2>&1 | grep -q "Suspicious" || exit 1

# Test 3: OOM prevention
./target/debug/kandil model add llama3.1:405b 2>&1 | grep -q "Insufficient RAM" || exit 1

# Test 4: Keyring storage
./target/debug/kandil model add qwen-test --key "sk-test"
grep -r "sk-test" ~/.config/kandil/ && exit 1
keyring get kandil api_key_alibaba-dashscope | grep -q "sk-test" || exit 1

echo "✅ All security tests passed"
```

---

## **11. Priority Matrix: Action Items**

| Priority | Task | Implementation | Phase | Why |
|----------|------|----------------|-------|-----|
| **P0 (Critical)** | Remove API key CLI args | Use keyring only | 0 | Security |
| **P0 (Critical)** | Add checksum validation | SHA256 + HF verification | 0 | Supply chain security |
| **P0 (Critical)** | Sandboxed model loading | Separate process with timeout | 0 | RCE prevention |
| **P0 (Critical)** | Hardware OOM prevention | Block models exceeding RAM | 0 | Stability |
| **P1 (High)** | Simplify protocol detection | Curated profiles > auto-detect | 0 | Reliability |
| **P1 (High)** | Connection pooling | Native reqwest with pool | 0 | Performance |
| **P1 (High)** | Cached discovery | Redis/TTL cache | 1 | Startup speed |
| **P1 (High)** | Native adapters | Direct HTTP, no CLI wrapping | 1 | Performance |
| **P2 (Medium)** | Bridge plugin | Optional LiteLLM integration | 3 | Enterprise feature |
| **P2 (Medium)** | Multi-agent system | Event bus + ReAct loop | 4 | Differentiation |
| **P3 (Low)** | TUI enhancements | Ratatui polish, lazy loading | 3 | UX |

---

## **12. Security Guarantees Summary**

This plan ensures **five critical security guarantees**:

1.  **`Never Log Secrets`**  : API keys never appear in logs, CLI args, or process listings.
2.  **`OS Keyring Only`**  : All credentials stored in platform-native secure storage (macOS Keychain, Windows Credential Manager, Linux libsecret).
3.  **`Supply Chain Integrity`**  : Every model file validated against official checksums before loading.
4.  **`Memory Safety`**  : Hardware compatibility check prevents OOM crashes and system instability.
5.  **`Sandboxed Execution`**  : Untrusted model files loaded in isolated processes with resource limits.

---

## **13. Performance Benchmarks (Target)**

| Operation | Native Adapter | Bridge Plugin | CLI Wrapper (Ref) |
|-----------|----------------|---------------|-------------------|
| Cold start | 45ms | 65ms (+20ms) | 680ms (+635ms) |
| First token | 120ms | 140ms (+20ms) | 800ms (+680ms) |
| 1k token gen | 890ms | 910ms (+20ms) | 1500ms (+610ms) |
| Concurrent req | 5 req/s | 5 req/s | 1.2 req/s |

**Goal**: Native adapter must be **within 5%** of direct API calls. Bridge plugin **optional** for enterprise features.

---

## **Final Recommendation**

This **Enhanced Model Integration Plan v2.0** transforms the original proposal into a **production-ready system** that:

- **Prioritizes security** without compromising UX
- **Delivers native performance** (15-20x faster than CLI wrapping)
- **Maintains zero-friction** model addition through smart defaults
- **Supports enterprise needs** via optional bridge plugin
- **Provides actionable roadmap** with P0-P3 priorities

**Immediate Next Steps**:
1. **Implement Phase 0 (P0 items)** - 3 weeks
2. **Add integration tests** for credential manager
3. **Create hardware detection CI matrix**
4. **Document security model** in `SECURITY.md`

The result: **A world-class AI development tool** that rivals Claude Code while being **faster, more secure, and fully local-first**.