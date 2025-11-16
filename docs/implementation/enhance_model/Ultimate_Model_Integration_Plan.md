# **Enhanced Model Integration Plan for Kandil Code**

**Version 2.0 - Security-First, Production-Ready Specification**

---

## **1. Architecture: The Secure Universal Model Registry (UMR)**

### **1.1 Core Design Principles**
- **Security First**: No API keys in CLI args, environment variables, or files
- **Fail-Fast Validation**: Block incompatible models before download
- **Explicit Over Magic**: Curated profiles > brittle auto-detection
- **Performance**: Cached discovery, connection pooling, lazy loading

### **1.2 Registry Structure**
```rust
// src/model/registry/mod.rs
use std::sync::Arc;
use tokio::sync::RwLock;
use lazy_static::lazy_static;

/// Central registry managing all model interactions
pub struct UniversalModelRegistry {
    /// Known model profiles (curated, verified)
    profiles: Arc<RwLock<HashMap<String, ModelProfile>>>,
    /// User-added custom models
    custom_models: Arc<RwLock<HashMap<String, ModelConfig>>>,
    /// Hardware compatibility cache
    compatibility_cache: Arc<RwLock<HashMap<String, CompatibilityReport>>>,
    /// Connection pool for API clients
    connection_pool: Arc<ConnectionPool>,
}

lazy_static! {
    pub static ref REGISTRY: UniversalModelRegistry = UniversalModelRegistry::new();
}

impl UniversalModelRegistry {
    fn new() -> Self {
        Self {
            profiles: Arc::new(RwLock::new(Self::load_builtin_profiles())),
            custom_models: Arc::new(RwLock::new(Self::load_custom_models())),
            compatibility_cache: Arc::new(RwLock::new(HashMap::new())),
            connection_pool: Arc::new(ConnectionPool::new()),
        }
    }
    
    /// Load 50+ verified model profiles at compile time
    fn load_builtin_profiles() -> HashMap<String, ModelProfile> {
        include!("builtin_profiles.rs") // Compile-time inclusion
    }
}
```

### **1.3 Secure Model Configuration**
```rust
// src/model/config.rs
use serde::{Deserialize, Serialize};
use keyring::Entry;

/// Public configuration (safe to serialize)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfig {
    pub name: String,
    pub provider: Provider,
    pub base_url: String,
    pub protocol: Protocol,
    pub context_window: usize,
    pub supports_vision: bool,
    pub supports_tools: bool,
    pub recommended_for: Vec<TaskType>,
    pub api_key_required: bool,
}

/// Sensitive credentials (never serialized)
pub struct ModelCredentials {
    pub api_key: SecretString,
    pub organization: Option<String>,
}

impl ModelCredentials {
    /// Load from OS keyring
    pub fn load(provider: &str) -> Result<Self> {
        let entry = Entry::new("kandil", &format!("api_key_{}", provider))?;
        let api_key = entry.get_password()?;
        
        let org_entry = Entry::new("kandil", &format!("org_{}", provider)).ok();
        let organization = org_entry.and_then(|e| e.get_password().ok());
        
        Ok(Self {
            api_key: SecretString::new(api_key),
            organization: organization.map(SecretString::new),
        })
    }
    
    /// Store securely
    pub fn store(provider: &str, api_key: String, organization: Option<String>) -> Result<()> {
        let entry = Entry::new("kandil", &format!("api_key_{}", provider))?;
        entry.set_password(&api_key)?;
        
        if let Some(org) = organization {
            let org_entry = Entry::new("kandil", &format!("org_{}", provider))?;
            org_entry.set_password(&org)?;
        }
        
        Ok(())
    }
}

// src/types.rs
use secrecy::{Secret, SecretString};

/// Secure wrapper for sensitive data
pub type SecretString = Secret<String>;
```

---

## **2. Secure Model Addition Workflow**

### **2.1 CLI Command (Safe by Design)**
```bash
# ✅ SECURE: No API key in command
$ kandil model add qwen2.5-coder-32b

# Interactive prompt for API key
🔐 API key required for qwen2.5-coder-32b
   Provider: Alibaba DashScope
   Security: Key will be stored in OS keyring
   ℹ️  Get key from: https://dashscope.console.aliyun.com/apiKey
   
? Enter API key (hidden): ************
? Confirm API key: ************
✅ Key validated and stored securely

# Hardware check
🖥️  Hardware Check:
   - RAM: 32GB available (Need 24GB) ✅
   - GPU: RTX 4090 24GB ✅
   - Disk: 45GB free (Need 18.2GB) ✅
✅ Compatible

# Download verification
📦 Downloading Qwen2.5-Coder-32B-Q4.gguf...
✅ Checksum verified: ef3a3b2c...
✅ Model signature valid (Alibaba official)
✅ No malicious metadata detected
✅ Sandboxed load test passed

✅ Model ready: Use 'kandil /model use qwen32b'
```

### **2.2 Implementation**
```rust
// src/cli/commands/model_add.rs
use rpassword::prompt_password;

pub async fn execute(args: ModelAddArgs) -> Result<()> {
    // 1. Resolve model name to profile
    let profile = REGISTRY.resolve_profile(&args.name).await?;
    
    // 2. Hardware compatibility check (with caching)
    let report = REGISTRY.check_compatibility(&profile.name).await?;
    if report.blocked {
        return Err(anyhow::anyhow!(
            "Model incompatible:\n{}",
            report.reasons.join("\n")
        ));
    }
    
    // 3. Secure API key input if required
    if profile.api_key_required {
        let entry = keyring::Entry::new("kandil", &format!("api_key_{}", profile.provider))?;
        if matches!(entry.get_password(), Err(keyring::Error::NoEntry)) {
            eprintln!("🔐 API key required for {}", profile.name);
            eprintln!("   ℹ️  Get key from: {}", profile.api_key_url);
            
            let key = prompt_password("? Enter API key: ")?;
            let confirm = prompt_password("? Confirm API key: ")?;
            
            if key != confirm {
                bail!("Keys don't match");
            }
            
            // 4. Validate key before storing
            validate_api_key(&profile, &key).await?;
            
            // 5. Store securely
            ModelCredentials::store(&profile.provider, key, None)?;
            eprintln!("✅ Key validated and stored in keyring");
        }
    }
    
    // 6. Download and verify local models
    if profile.is_local {
        download_and_verify(&profile, &report.warnings).await?;
    }
    
    // 7. Test actual API call
    test_model_connection(&profile).await?;
    
    // 8. Save to user config
    ConfigManager::add_model(&profile).await?;
    
    Ok(())
}

/// Validate API key with a minimal test request
async fn validate_api_key(profile: &ModelProfile, key: &str) -> Result<()> {
    let client = reqwest::Client::new();
    let res = client
        .post(&profile.base_url)
        .header("Authorization", format!("Bearer {}", key))
        .json(&json!({
            "model": profile.name,
            "messages": [{"role": "user", "content": "hi"}],
            "max_tokens": 1
        }))
        .send()
        .await?;
    
    if res.status() == 401 {
        bail!("Invalid API key");
    } else if res.status() == 429 {
        bail!("Rate limit exceeded - key may be valid but overused");
    } else if !res.status().is_success() {
        bail!("Key validation failed: {}", res.text().await?);
    }
    
    Ok(())
}
```

---

## **3. Hardware Compatibility Engine**

### **3.1 Real-Time Detection**
```rust
// src/hardware/detector.rs
use sysinfo::{System, SystemExt, DiskExt};
use nvml_wrapper::{NVML, Device};

pub struct HardwareDetector {
    sys: Arc<RwLock<System>>,
    nvml: Option<NVML>,
}

impl HardwareDetector {
    pub fn new() -> Result<Self> {
        let nvml = NVML::init().ok(); // Optional GPU support
        
        Ok(Self {
            sys: Arc::new(RwLock::new(System::new_all())),
            nvml,
        })
    }
    
    pub fn get_report(&self) -> HardwareReport {
        let sys = self.sys.read().unwrap();
        
        let total_ram = sys.total_memory(); // in KB
        let free_disk = self.get_free_disk();
        
        let gpu_info = self.detect_gpu();
        
        HardwareReport {
            total_ram_gb: total_ram as f64 / 1024.0 / 1024.0,
            free_disk_gb: free_disk,
            cpu_cores: sys.cpus().len(),
            gpu: gpu_info,
        }
    }
    
    fn detect_gpu(&self) -> Option<GPUInfo> {
        let nvml = self.nvml.as_ref()?;
        let device = nvml.device_by_index(0).ok()?;
        
        Some(GPUInfo {
            name: device.name().ok()?,
            memory_gb: device.memory_info().ok()?.total as f64 / 1024.0 / 1024.0 / 1024.0,
            cuda_cores: device.num_cores().ok()?,
        })
    }
}

#[derive(Debug, Clone)]
pub struct HardwareReport {
    pub total_ram_gb: f64,
    pub free_disk_gb: f64,
    pub cpu_cores: usize,
    pub gpu: Option<GPUInfo>,
}

#[derive(Debug, Clone)]
pub struct GPUInfo {
    pub name: String,
    pub memory_gb: f64,
    pub cuda_cores: u32,
}
```

### **3.2 Compatibility Scoring**
```rust
// src/model/compatibility.rs
pub struct CompatibilityEngine;

impl CompatibilityEngine {
    pub fn check(&self, profile: &ModelProfile, hardware: &HardwareReport) -> CompatibilityReport {
        let mut report = CompatibilityReport::default();
        
        // RAM requirement: Model size * 1.5 for overhead
        let required_ram = profile.size_gb * 1.5;
        if hardware.total_ram_gb < required_ram {
            report.blocked = true;
            report.reasons.push(format!(
                "Insufficient RAM: {:.1}GB required, {:.1}GB available",
                required_ram, hardware.total_ram_gb
            ));
        } else if hardware.total_ram_gb < required_ram * 1.2 {
            report.warnings.push("RAM usage will be high".to_string());
        }
        
        // Disk space
        if hardware.free_disk_gb < profile.size_gb {
            report.blocked = true;
            report.reasons.push(format!(
                "Insufficient disk: {:.1}GB required, {:.1}GB free",
                profile.size_gb, hardware.free_disk_gb
            ));
        }
        
        // GPU recommendation
        if profile.recommends_gpu && hardware.gpu.is_none() {
            report.warnings.push(
                "Model performs best with GPU acceleration. CPU-only will be slow.".to_string()
            );
        }
        
        // GPU memory for local models
        if profile.is_local {
            if let Some(ref gpu) = hardware.gpu {
                if gpu.memory_gb < profile.size_gb {
                    report.warnings.push(format!(
                        "GPU VRAM ({:.1}GB) < Model size ({:.1}GB). Will use CPU inference.",
                        gpu.memory_gb, profile.size_gb
                    ));
                }
            }
        }
        
        report
    }
}

pub struct CompatibilityReport {
    pub blocked: bool,
    pub reasons: Vec<String>,
    pub warnings: Vec<String>,
}
```

---

## **4. Smart Discovery & Caching**

### **4.1 Cached Discovery Engine**
```rust
// src/model/discovery/cache.rs
use tokio::sync::RwLock;
use std::time::{Instant, Duration};

pub struct CachedDiscoveryEngine {
    inner: DiscoveryEngine,
    cache: Arc<RwLock<HashMap<String, CacheEntry>>>,
    ttl: Duration,
}

struct CacheEntry {
    results: Vec<ModelCandidate>,
    timestamp: Instant,
}

impl CachedDiscoveryEngine {
    pub async fn discover(&self, query: &str) -> Result<Vec<ModelCandidate>> {
        // Check cache
        {
            let cache = self.cache.read().await;
            if let Some(entry) = cache.get(query) {
                if entry.timestamp.elapsed() < self.ttl {
                    return Ok(entry.results.clone());
                }
            }
        }
        
        // Parallel prioritized discovery
        let results = self.discover_prioritized(query).await?;
        
        // Store in cache
        {
            let mut cache = self.cache.write().await;
            cache.insert(query.to_string(), CacheEntry {
                results: results.clone(),
                timestamp: Instant::now(),
            });
        }
        
        Ok(results)
    }
    
    async fn discover_prioritized(&self, query: &str) -> Result<Vec<ModelCandidate>> {
        // Priority 1: Local Ollama (fastest)
        if let Ok(local) = self.discover_ollama(query).await {
            if !local.is_empty() {
                return Ok(local);
            }
        }
        
        // Priority 2: Known cloud models (no discovery needed)
        if let Some(profile) = REGISTRY.get_builtin_profile(query) {
            return Ok(vec![ModelCandidate::from_profile(profile)]);
        }
        
        // Priority 3: Hugging Face (async, cached)
        if query.contains('/') || query.len() > 10 {
            if let Ok(hf) = self.discover_huggingface(query).await {
                if !hf.is_empty() {
                    return Ok(hf);
                }
            }
        }
        
        bail!("No models found for '{}'", query)
    }
}
```

---

## **5. Curated Model Profiles (Built-in)**

### **5.1 Compile-Time Profiles**
```rust
// src/model/builtin_profiles.rs
pub const BUILTIN_PROFILES: &[(&str, ModelProfile)] = &[
    // Anthropic
    ("claude-3.5-sonnet", ModelProfile {
        provider: Provider::Anthropic,
        base_url: "https://api.anthropic.com/v1/messages",
        protocol: Protocol::Anthropic,
        context_window: 200_000,
        max_tokens: 4096,
        supports_vision: true,
        supports_tools: true,
        api_key_required: true,
        api_key_url: "https://console.anthropic.com/settings/keys",
        size_gb: 0.0, // Cloud model
        is_local: false,
        recommends_gpu: false,
        recommended_for: vec![TaskType::Coding, TaskType::Reasoning],
    }),
    
    // Alibaba Qwen
    ("qwen2.5-coder-32b", ModelProfile {
        provider: Provider::Dashscope,
        base_url: "https://dashscope-intl.aliyuncs.com/compatible-mode/v1",
        protocol: Protocol::OpenAI,
        context_window: 32_768,
        max_tokens: 8192,
        supports_vision: false,
        supports_tools: true,
        api_key_required: true,
        api_key_url: "https://dashscope.console.aliyun.com/apiKey",
        size_gb: 18.2,
        is_local: false,
        recommends_gpu: true,
        recommended_for: vec![TaskType::Coding, TaskType::Speed],
    }),
    
    // Ollama local
    ("llama3.1:70b", ModelProfile {
        provider: Provider::Ollama,
        base_url: "http://localhost:11434",
        protocol: Protocol::OpenAI,
        context_window: 128_000,
        max_tokens: 4096,
        supports_vision: false,
        supports_tools: false,
        api_key_required: false,
        api_key_url: String::new(),
        size_gb: 40.5,
        is_local: true,
        recommends_gpu: true,
        recommended_for: vec![TaskType::Coding, TaskType::Privacy],
    }),
    
    // 50+ more models...
];
```

### **5.2 Fuzzy Name Resolution**
```rust
// src/model/resolver.rs
pub struct ModelResolver;

impl ModelResolver {
    /// Resolve user input to exact model name
    pub fn resolve(&self, input: &str) -> Result<String> {
        let input = input.to_lowercase();
        
        // 1. Exact match
        if BUILTIN_PROFILES.iter().any(|(name, _)| name == &input) {
            return Ok(input);
        }
        
        // 2. Alias match
        match input.as_str() {
            "claude" => return Ok("claude-3.5-sonnet".to_string()),
            "gpt4" => return Ok("gpt-4-turbo".to_string()),
            "qwen32b" => return Ok("qwen2.5-coder-32b".to_string()),
            _ => {}
        }
        
        // 3. Fuzzy match
        let matches = self.fuzzy_search(&input);
        if matches.len() == 1 {
            return Ok(matches[0].clone());
        } else if matches.len() > 1 {
            bail!("Ambiguous name. Did you mean: {:?}?", matches);
        }
        
        // 4. Unknown model - require explicit manual add
        bail!(
            "Unknown model '{}'. Run: kandil model add {} --manual\n\
            Or see available models: kandil model list",
            input, input
        )
    }
    
    fn fuzzy_search(&self, input: &str) -> Vec<String> {
        BUILTIN_PROFILES
            .iter()
            .filter(|(name, _)| name.contains(input) || input.contains(name))
            .map(|(name, _)| name.to_string())
            .collect()
    }
}
```

---

## **6. Connection Pooling & Performance**

### **6.1 Per-Model Connection Pool**
```rust
// src/client/pool.rs
use reqwest::{Client, ClientBuilder};
use std::time::Duration;

pub struct ConnectionPool {
    clients: Arc<RwLock<HashMap<String, Arc<Client>>>>,
}

impl ConnectionPool {
    pub fn new() -> Self {
        Self {
            clients: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    pub async fn get(&self, model: &str) -> Arc<Client> {
        let mut clients = self.clients.write().await;
        
        if let Some(client) = clients.get(model) {
            return client.clone();
        }
        
        // Create optimized client for this model
        let profile = REGISTRY.get_profile(model).await.unwrap();
        let client = Arc::new(
            ClientBuilder::new()
                .connect_timeout(Duration::from_secs(10))
                .timeout(Duration::from_secs(120)) // For long completions
                .pool_idle_timeout(Duration::from_secs(30))
                .pool_max_idle_per_host(10)
                .tcp_keepalive(Duration::from_secs(60))
                .user_agent(format!("Kandil-Code/{}", env!("CARGO_PKG_VERSION")))
                .default_headers({
                    let mut headers = HeaderMap::new();
                    if let Ok(creds) = ModelCredentials::load(&profile.provider) {
                        headers.insert(
                            "Authorization",
                            format!("Bearer {}", creds.api_key.expose_secret()).parse().unwrap()
                        );
                    }
                    headers
                })
                .build()
                .unwrap()
        );
        
        clients.insert(model.to_string(), client.clone());
        client
    }
}
```

### **6.2 Request Batching**
```rust
// src/client/batch.rs
pub struct BatchClient {
    pool: Arc<ConnectionPool>,
    queue: Arc<RwLock<Vec<BatchRequest>>>,
}

struct BatchRequest {
    model: String,
    prompt: String,
    response_tx: oneshot::Sender<Result<String>>,
}

impl BatchClient {
    pub async fn batch_complete(&self, requests: Vec<(String, String)>) -> Result<Vec<String>> {
        // Group by model
        let mut by_model: HashMap<String, Vec<_>> = HashMap::new();
        for (model, prompt) in requests {
            by_model.entry(model).or_default().push(prompt);
        }
        
        // Execute in parallel
        let mut results = vec![];
        let futures = by_model.into_iter().map(|(model, prompts)| {
            let pool = self.pool.clone();
            async move {
                let client = pool.get(&model).await;
                let mut outputs = vec![];
                
                for prompt in prompts {
                    let res = client
                        .post(format!("{}/chat/completions", model))
                        .json(&json!({
                            "model": model,
                            "messages": [{"role": "user", "content": prompt}],
                            "stream": false,
                        }))
                        .send()
                        .await?
                        .json::<serde_json::Value>()
                        .await?;
                    
                    outputs.push(res["choices"][0]["message"]["content"]
                        .as_str()
                        .unwrap_or("")
                        .to_string());
                }
                
                Ok::<_, anyhow::Error>(outputs)
            }
        });
        
        let batch_results = futures::future::join_all(futures).await;
        for res in batch_results {
            results.extend(res?);
        }
        
        Ok(results)
    }
}
```

---

## **7. Safety & Validation Infrastructure**

### **7.1 Model File Security**
```rust
// src/model/security/validator.rs
use sha2::{Sha256, Digest};
use gguf::GGUFFile;

pub struct ModelSecurityValidator;

impl ModelSecurityValidator {
    /// Comprehensive validation before loading any model file
    pub async fn validate_model_file(path: &Path, expected_hash: &str) -> Result<()> {
        // 1. Verify checksum
        let mut file = File::open(path)?;
        let mut hasher = Sha256::new();
        std::io::copy(&mut file, &mut hasher)?;
        let hash = format!("{:x}", hasher.finalize());
        
        if hash != expected_hash {
            bail!(
                "Checksum mismatch!\n\
                Expected: {}\n\
                Actual:   {}\n\
                Possible causes: corrupted download or tampering",
                expected_hash, hash
            );
        }
        
        // 2. Parse and validate GGUF structure
        let gguf = GGUFFile::read(path)?;
        self.validate_gguf_metadata(&gguf)?;
        
        // 3. Sandboxed load test (in separate process)
        self.sandboxed_load_test(path).await?;
        
        Ok(())
    }
    
    fn validate_gguf_metadata(&self, gguf: &GGUFFile) -> Result<()> {
        // Check for suspicious metadata keys
        for key in gguf.metadata.keys() {
            if key.starts_with("system.") || key.contains("exec") || key.contains("cmd") {
                bail!("Suspicious metadata key found: '{}'", key);
            }
        }
        
        // Validate tensor names
        for tensor in &gguf.tensors {
            if tensor.name.contains("..") || tensor.name.starts_with('/') {
                bail!("Invalid tensor name: '{}'", tensor.name);
            }
        }
        
        Ok(())
    }
    
    async fn sandboxed_load_test(&self, path: &Path) -> Result<()> {
        let path_str = path.to_string_lossy();
        
        // Spawn separate process with restricted permissions
        let mut child = tokio::process::Command::new("kandil-sandbox")
            .arg("load-test")
            .arg(&*path_str)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true)
            .spawn()?;

        let result = tokio::time::timeout(Duration::from_secs(30), child.wait_with_output()).await;

        match result {
            Ok(Ok(output)) if output.status.success() => Ok(()),
            Ok(Ok(output)) => {
                bail!("Sandboxed load test failed: {}", String::from_utf8_lossy(&output.stderr))
            }
            Ok(Err(e)) => Err(e.into()),
            Err(_) => {
                child.kill().ok();
                bail!("Sandboxed load test timed out (possible corrupted model)")
            }
        }
    }
}
```

### **7.2 API Safety Checker**
```rust
// src/model/security/api_checker.rs
pub struct APISafetyChecker;

impl APISafetyChecker {
    pub async fn validate_endpoint(profile: &ModelProfile) -> Result<ValidationReport> {
        let mut report = ValidationReport::default();
        
        // 1. DNS and connectivity check
        let url = Url::parse(&profile.base_url)?;
        if let Err(e) = tokio::net::lookup_host(url.host_str().unwrap()).await {
            bail!("DNS resolution failed: {}", e);
        }
        
        // 2. TLS certificate validation
        let client = reqwest::Client::builder()
            .danger_accept_invalid_certs(false) // Enforce valid certs
            .build()?;
        
        // 3. Test endpoint with minimal request
        let start = Instant::now();
        let test_res = client
            .post(&profile.base_url)
            .json(&json!({
                "model": profile.name,
                "messages": [{"role": "user", "content": "test"}],
                "max_tokens": 1
            }))
            .send()
            .await;
        
        report.latency_ms = start.elapsed().as_millis();
        
        match test_res {
            Ok(res) => {
                report.status_code = res.status().as_u16();
                
                // Check rate limit headers
                if let Some(limit) = res.headers().get("x-ratelimit-limit") {
                    report.rate_limit = limit.to_str().ok().map(|s| s.to_string());
                }
                
                // Check cost headers
                if let Some(cost) = res.headers().get("x-cost-estimate") {
                    report.estimated_cost = cost.to_str().ok().and_then(|s| s.parse().ok());
                }
                
                // Validate response format
                let body: serde_json::Value = res.json().await?;
                Self::validate_response_format(&body, &profile.protocol)?;
            }
            Err(e) => {
                bail!("API endpoint unreachable: {}", e);
            }
        }
        
        Ok(report)
    }
}
```

---

## **8. User Experience: Interactive Setup**

### **8.1 Guided Configuration**
```bash
$ kandil setup

✨ Kandil Code Setup Wizard

🖥️  Hardware Detected:
   CPU: AMD Ryzen 9 7950X (16 cores)
   RAM: 64GB DDR5
   GPU: NVIDIA RTX 4090 (24GB VRAM)
   Disk: 1TB NVMe (342GB free)

❓ Choose your primary model provider:
   1) Local Models (Ollama) - Best for privacy
   2) Alibaba Qwen - Best cost/performance
   3) Anthropic Claude - Best quality
   4) Google Gemini - Best for multi-modal
   > 2

🔐 Please provide your DashScope API key:
   ℹ️  Get key from: https://dashscope.console.aliyun.com/apiKey
   ? Enter key (hidden): ************
   ✅ Key validated

❓ Which models to install?
   ✅ qwen2.5-coder-7b (4.5GB, recommended)
   ✅ qwen2.5-coder-14b (9.2GB, recommended)
   ☐ qwen2.5-coder-32b (18.2GB)
   ☐ qwen-max (API only)
   > Enter

📦 Downloading qwen2.5-coder-7b...
✅ Checksum verified
✅ Load test passed
✅ Model ready

📦 Downloading qwen2.5-coder-14b...
✅ Checksum verified
✅ Load test passed
✅ Model ready

✅ Setup complete! Try: kandil /ask "Hello world in Rust"
```

### **8.2 Model Listing with Status**
```bash
$ kandil model list
✅ Local Models (Ollama):
   llama3.1:8b         4.5GB   ✅ Ready      CPU inference
   qwen2.5-coder:7b    4.5GB   ✅ Ready      GPU accelerated
   qwen2.5-coder:14b   9.2GB   ✅ Ready      GPU accelerated
   mistral-large:123b  68GB    ❌ Incompatible (Need 102GB RAM)

✅ Cloud Models (Configured):
   qwen2.5-coder-32b   API     ✅ Ready      Low latency
   claude-3.5-sonnet   API     ✅ Ready      Rate: 40 req/min
   gemini-1.5-pro      API     ⚠️  Slow      Latency: 850ms

✅ Other Available:
   deepseek-coder:33b  API     ☐ Not configured
   gpt-4-turbo         API     ☐ Not configured

💡 Run 'kandil model add <name>' to configure
```

---

## **9. Testing & Quality Assurance**

### **9.1 Integration Test Suite**
```rust
// tests/model_integration.rs
#[tokio::test]
async fn test_full_model_lifecycle() {
    // Setup test registry
    let registry = UniversalModelRegistry::new();
    
    // Test 1: Add model
    registry.add_model("qwen-test").await.unwrap();
    
    // Test 2: Verify credentials stored
    let creds = ModelCredentials::load("qwen").unwrap();
    assert_eq!(creds.api_key.expose_secret(), "test-key");
    
    // Test 3: Hardware check
    let report = registry.check_compatibility("qwen-test").await.unwrap();
    assert!(!report.blocked);
    
    // Test 4: API validation
    let profile = registry.get_profile("qwen-test").await.unwrap();
    let validation = APISafetyChecker::validate_endpoint(&profile).await.unwrap();
    assert_eq!(validation.status_code, 200);
    
    // Test 5: Actual completion
    let result = registry.complete("qwen-test", "Hello").await.unwrap();
    assert!(!result.is_empty());
    
    // Cleanup
    registry.remove_model("qwen-test").await.unwrap();
}

#[test]
fn test_malicious_model_detection() {
    let validator = ModelSecurityValidator;
    
    // Create fake malicious GGUF
    let malicious_file = create_malicious_gguf();
    
    let result = block_on(validator.validate_model_file(&malicious_file, "fake_hash"));
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Suspicious metadata"));
}
```

### **9.2 Rate Limit Testing**
```rust
// tests/rate_limit.rs
#[tokio::test]
async fn test_rate_limit_protection() {
    let registry = UniversalModelRegistry::new();
    
    // Spawn many concurrent requests
    let mut tasks = vec![];
    for _ in 0..100 {
        tasks.push(registry.complete("qwen-test", "test"));
    }
    
    let results = futures::future::join_all(tasks).await;
    
    // Should not all succeed due to rate limiting
    let successes = results.iter().filter(|r| r.is_ok()).count();
    let rate_limited = results.iter().filter(|r| {
        r.as_ref().err().map(|e| e.to_string().contains("rate limit")).unwrap_or(false)
    }).count();
    
    assert!(rate_limited > 0, "Should have rate limited some requests");
    assert!(successes <= 50, "Should not exceed rate limit");
}
```

---

## **10. Configuration Schema**

### **10.1 User Config File**
```toml
# ~/.config/kandil/config.toml
# Auto-generated, versioned config

version = 2

[defaults]
model = "qwen2.5-coder-7b"  # Auto-detected if not set
theme = "dark"
telemetry = false

[api]  # API endpoints (safe to commit, no keys)
qwen = { base_url = "https://dashscope-intl.aliyuncs.com/compatible-mode/v1", protocol = "openai" }
anthropic = { base_url = "https://api.anthropic.com/v1/messages", protocol = "anthropic" }

[models]  # Per-model overrides
[qwen2.5-coder-7b]
max_tokens = 4096
temperature = 0.2
top_p = 0.9

[cache]
ttl_seconds = 3600
max_size_mb = 100

[limits]
max_concurrent_requests = 5
rate_limit_retry = true
```

### **10.2 Model Metadata Storage**
```json
// ~/.local/share/kandil/models/qwen32b.metadata.json
{
  "schema_version": 2,
  "name": "qwen2.5-coder-32b",
  "source": "huggingface:Qwen/Qwen2.5-Coder-32B-Instruct-GGUF",
  "sha256": "ef3a3b2c4d5e6f7890a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3",
  "size_bytes": 18200000000,
  "verified_at": "2025-01-15T10:30:00Z",
  "capabilities": ["code", "reasoning", "long-context"],
  "context_window": 32768,
  "last_used": "2025-01-15T11:45:00Z",
  "usage_count": 42,
  "average_latency_ms": 340
}
```

---

## **11. Implementation Roadmap**

### **Phase 0: Foundation (Weeks 1-2)**
- [ ] Create `UniversalModelRegistry` with built-in profiles
- [ ] Implement hardware detection (`sysinfo`, `nvml-wrapper`)
- [ ] Set up secure credential storage (`keyring`, `secrecy`)
- [ ] Add connection pooling (`reqwest` with custom config)

### **Phase 1: Core Commands (Weeks 3-4)**
- [ ] Implement `kandil model add <name>` with interactive prompts
- [ ] Hardware compatibility checker
- [ ] API key validation
- [ ] Basic model listing

### **Phase 2: Security Hardening (Weeks 5-6)**
- [ ] GGUF file validator with checksums
- [ ] Sandboxed load testing
- [ ] API safety checker
- [ ] Rate limit implementation

### **Phase 3: Discovery & Caching (Weeks 7-8)**
- [ ] Cached discovery engine
- [ ] Ollama registry integration
- [ ] Hugging Face registry
- [ ] Fuzzy name resolution

### **Phase 4: UX Polish (Weeks 9-10)**
- [ ] Interactive setup wizard
- [ ] Enhanced CLI output with emojis
- [ ] Model performance metrics tracking
- [ ] Configuration versioning

### **Phase 5: Testing & Release (Weeks 11-12)**
- [ ] Integration test suite
- [ ] Rate limit testing
- [ ] Security test suite (malicious files)
- [ ] Documentation and examples

---

## **12. Command Reference (Final)**

```bash
# Model Management
kandil model add <name>              # Add model (interactive, secure)
kandil model add <name> --manual     # Manual configuration for custom models
kandil model list                    # List all models with status
kandil model rm <name>               # Remove model
kandil model test <name>             # Test model connectivity
kandil model benchmark <name>        # Benchmark latency/throughput

# Configuration
kandil setup                         # Interactive initial setup
kandil config show                   # Show current configuration
kandil config set <key> <value>      # Set config value
kandil config reload                 # Reload from disk

# Usage (AI commands)
kandil /ask "question"               # Ask question (auto-select model)
kandil /ask "question" --model <name> # Use specific model
kandil /model use <name>             # Set default for session
kandil /model default <name>         # Set default for project
kandil /refactor <file>              # Refactor code
kandil /test <path>                  # Generate tests
kandil /review <path>                # Code review

# Advanced
kandil model discover <query>        # Search for models
kandil model aliases                 # Show available aliases
kandil model update <name>           # Update to latest version
```

---

## **13. Key Security Guarantees**

This enhanced plan ensures:

1. **No API Key Exposure**: Keys never appear in CLI, logs, or files
2. **Malicious Model Prevention**: Checksum validation + sandboxed loading
3. **Hardware Safety**: Block models that could cause OOM crashes
4. **Network Safety**: TLS validation, rate limiting, circuit breakers
5. **Supply Chain Security**: Curated profiles for known models
6. **User Consent**: Explicit confirmation for large downloads/incompatible models

---

**This enhanced plan transforms the original proposal into a production-ready system that prioritizes security, reliability, and user experience while maintaining the ambitious goal of universal model access.**