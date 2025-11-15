# Complete Local Model Integration Plan for Kandil Code

This is a **comprehensive, phase-by-phase implementation blueprint** for local model integration that supports **every hardware configuration** from 4GB RAM laptops to 128GB workstations, with **zero-configuration defaults** and **maximum user control**.

---

## **Phase 0: Foundation & Hardware Detection (Week 1)**

### **Goals**
- Detect hardware capabilities (RAM, CPU, GPU)
- Establish configuration layer
- Create model catalog
- Set up error handling and logging

### **Implementation**

#### **0.1 Hardware Profiler**
```rust
// src/core/hardware.rs
use sysinfo::{System, SystemExt};
use nvml_wrapper::Nvml;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareProfile {
    pub total_ram_gb: u64,
    pub available_ram_gb: u64,
    pub cpu_physical_cores: usize,
    pub cpu_logical_cores: usize,
    pub cpu_brand: String,
    pub gpu: Option<GpuInfo>,
    pub os: String,
    pub arch: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuInfo {
    pub brand: String,
    pub model: String,
    pub memory_gb: u64,
    pub compute_capability: Option<(u32, u32)>,
    pub driver_version: String,
}

pub fn detect_hardware() -> HardwareProfile {
    let mut sys = System::new_all();
    sys.refresh_all();
    
    let total_ram_gb = sys.total_memory() / (1024 * 1024 * 1024);
    let available_ram_gb = sys.available_memory() / (1024 * 1024 * 1024);
    
    let cpu_physical_cores = sys.physical_core_count().unwrap_or(1);
    let cpu_logical_cores = sys.cpus().len();
    let cpu_brand = sys.cpus().first()
        .map(|c| c.brand().to_string())
        .unwrap_or_else(|| "Unknown".to_string());
    
    let gpu = detect_gpu().ok();
    
    HardwareProfile {
        total_ram_gb,
        available_ram_gb,
        cpu_physical_cores,
        cpu_logical_cores,
        cpu_brand,
        gpu,
        os: std::env::consts::OS.to_string(),
        arch: std::env::consts::ARCH.to_string(),
    }
}

fn detect_gpu() -> Result<GpuInfo, GpuDetectionError> {
    // Try NVIDIA first
    if let Ok(nvml) = Nvml::init() {
        let device = nvml.device_by_index(0)?;
        return Ok(GpuInfo {
            brand: "NVIDIA".to_string(),
            model: device.name()?,
            memory_gb: device.memory_info()?.total / (1024 * 1024 * 1024),
            compute_capability: device.cuda_compute_capability().ok(),
            driver_version: device.nvml()?.sys_driver_version()?,
        });
    }
    
    // Try AMD ROCm on Linux
    #[cfg(target_os = "linux")]
    if let Ok(rocm) = detect_rocm_gpu() {
        return Ok(rocm);
    }
    
    // Try Apple Silicon
    #[cfg(target_os = "macos")]
    if is_apple_silicon() {
        return Ok(GpuInfo {
            brand: "Apple".to_string(),
            model: "Apple Silicon".to_string(),
            memory_gb: detect_apple_gpu_memory()?,
            compute_capability: None,
            driver_version: "N/A".to_string(),
        });
    }
    
    Err(GpuDetectionError::NoGpuFound)
}
```

#### **0.2 Configuration Layer**
```rust
// src/config/layered.rs
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub model: ModelConfig,
    pub performance: PerformanceConfig,
    pub fallback: FallbackConfig,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ModelConfig {
    pub name: ModelName,
    pub path: Option<PathBuf>,
    pub context_size: usize,
    pub quantization: Quantization,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PerformanceConfig {
    pub threads: usize,
    pub use_mmap: bool,
    pub batch_size: usize,
    pub target_latency_ms: u64,
    pub reserve_ram_gb: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FallbackConfig {
    pub enabled: bool,
    pub cloud_provider: Option<CloudProvider>,
    pub timeout_ms: u64,
}

impl Config {
    pub fn load() -> Result<Self> {
        let cli = CliConfig::parse();
        let env = EnvConfig::load();
        let local = LocalConfig::load().unwrap_or_default();
        let user = UserConfig::load().unwrap_or_default();
        
        // Merge with priority: CLI > ENV > Local > User > Auto-detect
        let hardware = detect_hardware();
        let auto = AutoConfig::from_hardware(&hardware);
        
        Ok(merge_configs(cli, env, local, user, auto))
    }
}
```

#### **0.3 Model Catalog**
```rust
// src/models/catalog.rs
use lazy_static::lazy_static;

pub struct ModelSpec {
    pub name: &'static str,
    pub huggingface_repo: &'static str,
    pub filename: &'static str,
    pub size_gb: f64,
    pub ram_required_gb: u64,
    pub gpu_vram_min: Option<u64>,
    pub speed_rating: Speed,
    pub quality_rating: Quality,
    pub description: &'static str,
    pub context_sizes: &'static [usize],
}

lazy_static! {
    pub static ref MODEL_CATALOG: Vec<ModelSpec> = vec![
        // Ultra-lightweight (2-4GB RAM systems)
        ModelSpec {
            name: "qwen2.5-coder-1.5b-q4",
            huggingface_repo: "Qwen/Qwen2.5-Coder-1.5B-Instruct-GGUF",
            filename: "qwen2.5-coder-1.5b-instruct-q4_k_m.gguf",
            size_gb: 0.9,
            ram_required_gb: 3,
            gpu_vram_min: None,
            speed_rating: Speed::UltraFast(350),
            quality_rating: Quality::Basic,
            description: "Ultra-lightweight for basic completion on minimal hardware",
            context_sizes: &[2048, 4096],
        },
        
        // Lightweight (4-8GB RAM systems)
        ModelSpec {
            name: "qwen2.5-coder-3b-q4",
            huggingface_repo: "Qwen/Qwen2.5-Coder-3B-Instruct-GGUF",
            filename: "qwen2.5-coder-3b-instruct-q4_k_m.gguf",
            size_gb: 1.8,
            ram_required_gb: 6,
            gpu_vram_min: Some(2),
            speed_rating: Speed::VeryFast(200),
            quality_rating: Quality::Good,
            description: "Balanced speed/quality for entry-level machines",
            context_sizes: &[2048, 4096, 8192],
        },
        
        // Standard (8-16GB RAM systems)
        ModelSpec {
            name: "qwen2.5-coder-7b-q4",
            huggingface_repo: "Qwen/Qwen2.5-Coder-7B-Instruct-GGUF",
            filename: "qwen2.5-coder-7b-instruct-q4_k_m.gguf",
            size_gb: 4.5,
            ram_required_gb: 12,
            gpu_vram_min: Some(4),
            speed_rating: Speed::Fast(120),
            quality_rating: Quality::VeryGood,
            description: "Recommended for most development tasks",
            context_sizes: &[4096, 8192, 16384, 32768],
        },
        
        // Professional (16-32GB RAM systems)
        ModelSpec {
            name: "qwen2.5-coder-14b-q4",
            huggingface_repo: "Qwen/Qwen2.5-Coder-14B-Instruct-GGUF",
            filename: "qwen2.5-coder-14b-instruct-q4_k_m.gguf",
            size_gb: 8.5,
            ram_required_gb: 20,
            gpu_vram_min: Some(8),
            speed_rating: Speed::Medium(70),
            quality_rating: Quality::Excellent,
            description: "High-quality for complex refactoring",
            context_sizes: &[4096, 8192, 16384],
        },
        
        // Premium (32-64GB RAM systems)
        ModelSpec {
            name: "llama3.1-70b-q4",
            huggingface_repo: "bartowski/Meta-Llama-3.1-70B-Instruct-GGUF",
            filename: "Meta-Llama-3.1-70B-Instruct-Q4_K_M.gguf",
            size_gb: 42.0,
            ram_required_gb: 60,
            gpu_vram_min: Some(24),
            speed_rating: Speed::Slow(25),
            quality_rating: Quality::Superior,
            description: "Best-in-class for architectural decisions",
            context_sizes: &[4096, 8192],
        },
    ];
}

#[derive(Debug)]
pub enum Speed {
    UltraFast(tps: usize),
    VeryFast(tps: usize),
    Fast(tps: usize),
    Medium(tps: usize),
    Slow(tps: usize),
}

#[derive(Debug)]
pub enum Quality {
    Basic,
    Good,
    VeryGood,
    Excellent,
    Superior,
}
```

---

## **Phase 1: Basic Local Model Adapter (Week 2)**

### **Goals**
- Integrate `llm` crate for safe model loading
- Implement basic text completion
- Add model download CLI
- Create health check system

### **1.1 Core Adapter Implementation**
```rust
// src/adapters/ai/local_llm.rs
use llm::{Llama, Model, ModelParameters, TokenizerSource};
use async_trait::async_trait;

pub struct LocalLLMAdapter {
    model: Llama,
    context_size: usize,
    threads: usize,
}

impl LocalLLMAdapter {
    pub async fn load(spec: &ModelSpec, config: &ModelConfig) -> Result<Self> {
        let model_path = config.path.as_ref()
            .map(|p| p.join(&spec.filename))
            .unwrap_or_else(|| default_model_dir().join(&spec.filename));
        
        if !model_path.exists() {
            bail!("Model not found at {:?}. Run `kandil model install {}`", model_path, spec.name);
        }
        
        let params = ModelParameters {
            prefer_mmap: config.use_mmap,
            context_size: config.context_size,
            n_gpu_layers: config.gpu_layers.unwrap_or(0),
            use_gpu: config.gpu_layers.is_some(),
            n_batch: 512,
            main_gpu: 0,
            tensor_split: None,
            rope_freq_scale: 1.0,
            rope_freq_base: 10000.0,
        };
        
        let model = tokio::task::spawn_blocking(move || {
            Llama::load(&model_path, params, TokenizerSource::Embedded)
        }).await??;
        
        Ok(Self {
            model,
            context_size: config.context_size,
            threads: config.threads,
        })
    }
}

#[async_trait]
impl AIProvider for LocalLLMAdapter {
    async fn complete(&self, prompt: &str) -> Result<String> {
        let mut session = self.model.start_session(Default::default());
        let mut output = String::new();
        
        // Use channel for streaming tokens
        let (tx, mut rx) = tokio::sync::mpsc::channel(100);
        
        let model = &self.model;
        let prompt = prompt.to_string();
        let threads = self.threads;
        
        tokio::task::spawn_blocking(move || {
            let result = session.infer(
                model,
                &llm::InferenceRequest {
                    prompt: (&prompt).into(),
                    parameters: &llm::InferenceParameters {
                        sampler: Arc::new(llm::samplers::TopPTopK {
                            top_k: 40,
                            top_p: 0.9,
                            temperature: 0.2,
                        }),
                    },
                    play_back_previous_tokens: false,
                    maximum_token_count: Some(2048),
                },
                &mut Default::default(),
                llm::OutputRequest {
                    all_callback: Some(Box::new(|token| {
                        tx.blocking_send(token.to_string()).ok();
                        Ok::<_, anyhow::Error>(())
                    })),
                    ..Default::default()
                },
            );
            
            if let Err(e) = result {
                tx.blocking_send(format!("__ERROR__:{:?}", e)).ok();
            }
        });
        
        // Stream tokens as they arrive
        while let Some(token) = rx.recv().await {
            if token.starts_with("__ERROR__:") {
                bail!("Inference error: {}", token);
            }
            output.push_str(&token);
        }
        
        Ok(output)
    }
    
    async fn is_available(&self) -> bool {
        true // Local model is always available if loaded
    }
}
```

### **1.2 Model Management CLI**
```rust
// src/cli/model.rs
use clap::Subcommand;
use indicatif::{ProgressBar, ProgressStyle};

#[derive(Subcommand)]
pub enum ModelCommand {
    /// List all available models
    List {
        /// Show only models compatible with your hardware
        #[arg(long)]
        compatible: bool,
    },
    
    /// Install a model
    Install {
        /// Model name (e.g., qwen2.5-coder-7b-q4)
        name: String,
        /// Force install even if hardware is insufficient
        #[arg(long)]
        force: bool,
    },
    
    /// Remove a model
    Remove {
        name: String,
    },
    
    /// Verify model integrity
    Verify {
        name: String,
    },
    
    /// Benchmark installed model
    Benchmark {
        name: Option<String>,
        /// Output format
        #[arg(long, default_value = "table")]
        format: BenchmarkFormat,
    },
    
    /// Set default model
    Use {
        name: String,
    },
}

pub async fn handle_model_command(cmd: ModelCommand) -> Result<()> {
    match cmd {
        ModelCommand::List { compatible } => {
            let hardware = detect_hardware();
            let catalog = MODEL_CATALOG.iter();
            
            if compatible {
                let catalog = catalog.filter(|m| m.ram_required_gb <= hardware.total_ram_gb);
            }
            
            println!("Available Models:");
            for model in catalog {
                println!("  {}", model.name);
                println!("    Size: {}GB, RAM: {}GB, GPU: {:?}GB", 
                    model.size_gb, model.ram_required_gb, model.gpu_vram_min);
                println!("    Speed: {:?}, Quality: {:?}", model.speed_rating, model.quality_rating);
                println!("    {}", model.description);
            }
        }
        
        ModelCommand::Install { name, force } => {
            let model = MODEL_CATALOG.iter()
                .find(|m| m.name == name)
                .ok_or_else(|| anyhow!("Unknown model: {}", name))?;
            
            let hardware = detect_hardware();
            if !force && model.ram_required_gb > hardware.total_ram_gb {
                bail!("Insufficient RAM. Model requires {}GB, you have {}GB. Use --force to override.", 
                    model.ram_required_gb, hardware.total_ram_gb);
            }
            
            let path = default_model_dir().join(&model.filename);
            if path.exists() {
                println!("Model already installed at {:?}", path);
                return Ok(());
            }
            
            download_model(&model.url, &path).await?;
            verify_model(&path, &model.sha256).await?;
            
            println!("✅ Model {} installed successfully", name);
        }
        
        ModelCommand::Benchmark { name, format } => {
            let model_name = name.unwrap_or_else(|| Config::load().model.name);
            benchmark_model(&model_name, format).await?;
        }
        
        _ => todo!(),
    }
}

async fn download_model(url: &str, path: &Path) -> Result<()> {
    let client = reqwest::Client::new();
    let response = client.get(url).send().await?;
    let total_size = response.content_length().unwrap_or(0);
    
    let pb = ProgressBar::new(total_size);
    pb.set_style(ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})")
        .unwrap());
    
    let mut file = tokio::io::BufWriter::new(tokio::fs::File::create(path).await?);
    let mut stream = response.bytes_stream();
    
    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        pb.inc(chunk.len() as u64);
        tokio::io::copy(&mut &chunk[..], &mut file).await?;
    }
    
    pb.finish_with_message("Download complete");
    file.flush().await?;
    Ok(())
}

async fn benchmark_model(name: &str, format: BenchmarkFormat) -> Result<()> {
    println!("🔍 Benchmarking model: {}", name);
    
    let tests = vec![
        ("Simple Completion", 100, "fn fibonacci(n: u32) -> u32 {"),
        ("Refactoring", 500, "Refactor this function to use iterators: ..."),
        ("Architecture", 1000, "Design a system for real-time chat with WebSockets"),
    ];
    
    for (test_name, token_budget, prompt) in tests {
        print!("  {}: ", test_name);
        io::stdout().flush()?;
        
        let start = Instant::now();
        let result = model.complete(prompt).await?;
        let elapsed = start.elapsed();
        
        let tokens = tokenize(&result).len();
        let tps = tokens as f64 / elapsed.as_secs_f64();
        
        println!("{} tokens in {:.2}s ({:.1} t/s)", tokens, elapsed.as_secs_f64(), tps);
        
        if tps < 50.0 {
            println!("    ⚠️  Slow - consider smaller model or GPU acceleration");
        }
    }
}
```

---

## **Phase 2: Auto-Configuration & Strategies (Week 3)**

### **Goals**
- Automatically select optimal model based on hardware
- Implement execution strategies
- Add hybrid cloud/local mode
- Create per-project overrides

### **2.1 Auto-Configuration Engine**
```rust
// src/core/auto_config.rs
pub struct AutoConfig;

impl AutoConfig {
    pub fn from_hardware(profile: &HardwareProfile) -> Config {
        let model_spec = Self::select_model(profile);
        let performance = Self::tune_performance(profile, &model_spec);
        let fallback = Self::configure_fallback(profile);
        
        Config {
            model: ModelConfig {
                name: model_spec.name.into(),
                path: None,
                context_size: Self::select_context_size(profile, &model_spec),
                quantization: Quantization::Q4_K_M,
            },
            performance,
            fallback,
        }
    }
    
    fn select_model(profile: &HardwareProfile) -> &'static ModelSpec {
        // Hard constraints first
        let compatible = MODEL_CATALOG.iter()
            .filter(|m| m.ram_required_gb <= profile.total_ram_gb);
        
        // Prefer GPU-capable models if GPU available
        if let Some(gpu) = &profile.gpu {
            if let Some(best) = compatible
                .filter(|m| m.gpu_vram_min.is_none() || m.gpu_vram_min.unwrap() <= gpu.memory_gb)
                .max_by_key(|m| m.quality_rating as i32)
            {
                return best;
            }
        }
        
        // CPU-only fallback
        compatible
            .filter(|m| m.gpu_vram_min.is_none())
            .max_by_key(|m| m.quality_rating as i32)
            .unwrap_or_else(|| {
                // Last resort: smallest model
                MODEL_CATALOG.first().unwrap()
            })
    }
    
    fn select_context_size(profile: &HardwareProfile, model: &ModelSpec) -> usize {
        let max_supported = model.context_sizes.last().unwrap();
        let ram_based = ((profile.available_ram_gb as f64 * 0.7) / (model.size_gb / *max_supported as f64)) as usize;
        
        model.context_sizes
            .iter()
            .filter(|&&size| size <= ram_based)
            .last()
            .copied()
            .unwrap_or(4096) // Safe default
    }
    
    fn tune_performance(profile: &HardwareProfile, model: &ModelSpec) -> PerformanceConfig {
        let threads = if profile.total_ram_gb < 8 {
            profile.cpu_physical_cores.min(4) // Leave room for OS
        } else {
            profile.cpu_physical_cores.min(6)
        };
        
        let use_mmap = profile.total_ram_gb >= 16; // MMAP benefits from extra RAM
        
        PerformanceConfig {
            threads,
            use_mmap,
            batch_size: 512,
            target_latency_ms: 2000,
            reserve_ram_gb: 4.min(profile.total_ram_gb / 4), // Reserve 25% or 4GB min
        }
    }
    
    fn configure_fallback(profile: &HardwareProfile) -> FallbackConfig {
        FallbackConfig {
            enabled: profile.total_ram_gb < 32, // Enable fallback for low-RAM systems
            cloud_provider: None, // User must configure API key
            timeout_ms: 30000,
        }
    }
}
```

### **2.2 Execution Strategies**
```rust
// src/core/strategy.rs
pub enum ExecutionStrategy {
    LocalOnly {
        model: Arc<LocalLLMAdapter>,
    },
    Hybrid {
        local: Arc<LocalLLMAdapter>,
        cloud: Arc<dyn AIProvider>,
        threshold: Duration,
    },
    Dynamic {
        fast_model: Arc<LocalLLMAdapter>,
        quality_model: Arc<LocalLLMAdapter>,
        cloud: Option<Arc<dyn AIProvider>>,
    },
}

impl ExecutionStrategy {
    pub async fn create(config: &Config) -> Result<Self> {
        match &config.strategy.mode {
            ExecutionMode::Local => {
                let model = LocalLLMAdapter::load(
                    &config.model.spec()?,
                    &config.model
                ).await?;
                Ok(Self::LocalOnly { model: Arc::new(model) })
            }
            
            ExecutionMode::Hybrid => {
                let local = LocalLLMAdapter::load(
                    &config.model.spec()?,
                    &config.model
                ).await?;
                
                let cloud = CloudAdapter::from_env()
                    .ok_or_else(|| anyhow!("Cloud provider not configured"))?;
                
                Ok(Self::Hybrid {
                    model: Arc::new(local),
                    cloud: Arc::new(cloud),
                    threshold: Duration::from_millis(config.strategy.timeout_ms),
                })
            }
            
            ExecutionMode::Dynamic => {
                // Load two models: fast (3B) and quality (7B)
                let fast_spec = get_smaller_model(&config.model.spec()?);
                let quality_spec = config.model.spec()?;
                
                let fast_model = LocalLLMAdapter::load(&fast_spec, &config.model).await?;
                let quality_model = LocalLLMAdapter::load(&quality_spec, &config.model).await?;
                
                Ok(Self::Dynamic {
                    fast_model: Arc::new(fast_model),
                    quality_model: Arc::new(quality_model),
                    cloud: CloudAdapter::from_env().map(|c| Arc::new(c) as Arc<dyn AIProvider>),
                })
            }
            
            ExecutionMode::CloudOnly => {
                let cloud = CloudAdapter::from_env()
                    .ok_or_else(|| anyhow!("No cloud provider configured"))?;
                Ok(Self::LocalOnly { model: Arc::new(cloud) })
            }
        }
    }
    
    pub async fn complete(&self, prompt: &str, complexity: TaskComplexity) -> Result<String> {
        match self {
            ExecutionStrategy::LocalOnly { model } => {
                model.complete(prompt).await
            }
            
            ExecutionStrategy::Hybrid { local, cloud, threshold } => {
                match timeout(*threshold, local.complete(prompt)).await {
                    Ok(Ok(response)) => Ok(response),
                    Ok(Err(e)) => {
                        tracing::warn!("Local model failed: {}, falling back to cloud", e);
                        cloud.complete(prompt).await
                    }
                    Err(_) => {
                        tracing::warn!("Local model timed out, falling back to cloud");
                        cloud.complete(prompt).await
                    }
                }
            }
            
            ExecutionStrategy::Dynamic { fast_model, quality_model, cloud } => {
                match complexity {
                    TaskComplexity::Simple => fast_model.complete(prompt).await,
                    TaskComplexity::Medium => quality_model.complete(prompt).await,
                    TaskComplexity::Complex => {
                        // Try quality model first, then cloud if available
                        match quality_model.complete(prompt).await {
                            Ok(resp) => Ok(resp),
                            Err(e) if cloud.is_some() => {
                                tracing::warn!("Quality model failed: {}, using cloud", e);
                                cloud.as_ref().unwrap().complete(prompt).await
                            }
                            Err(e) => Err(e),
                        }
                    }
                }
            }
        }
    }
}

#[derive(Debug)]
pub enum TaskComplexity {
    Simple,   // <200 tokens, autocomplete, syntax fix
    Medium,   // 200-1000 tokens, function generation
    Complex,  // >1000 tokens, architecture, debugging
}

impl TaskComplexity {
    pub fn from_prompt(prompt: &str) -> Self {
        let token_count = tiktoken_rs::num_tokens_from_messages("gpt-4", &[ChatCompletionRequestMessage {
            role: "user".to_string(),
            content: ChatCompletionRequestMessageContent::Text(prompt.to_string()),
            name: None,
            function_call: None,
            tool_calls: None,
        }]);
        
        match token_count {
            0..=200 => Self::Simple,
            201..=1000 => Self::Medium,
            _ => Self::Complex,
        }
    }
}
```

### **2.3 Per-Project Configuration**
```toml
# .kandil.toml (project root)
[model]
# Override global model for this project
name = "qwen2.5-coder-14b-q4"
context_size = 16384
force_cpu = false

[performance]
# Aggressive caching for large monorepo
cache_size = 10000
# Reserve less RAM for this project
reserve_ram_gb = 2

[strategy]
# Force hybrid mode for this project
mode = "hybrid"
# Use cloud for complex tasks only
complexity_threshold = "complex"

[project]
# Auto-detect language
language = "rust"
# Include these files in context
include_patterns = ["src/**/*.rs", "Cargo.toml"]
exclude_patterns = ["target/**", "*.log"]
```

---

## **Phase 3: Caching & Performance (Week 4)**

### **Goals**
- Implement semantic cache for repeated queries
- Add response caching with TTL
- Create prefetching system
- Add performance monitoring

### **3.1 Semantic Cache**
```rust
// src/cache/semantic.rs
use hnsw_rs::prelude::*;
use tiktoken_rs::num_tokens_from_messages;

pub struct SemanticCache {
    index: Hnsw<f32, usize>,
    store: Arc<DashMap<usize, CacheEntry>>,
    embeddings: Arc<EmbeddingModel>,
    max_size: usize,
    similarity_threshold: f32, // 0.95 = 95% similarity
}

struct CacheEntry {
    response: String,
    tokens: usize,
    hit_count: AtomicU64,
    last_access: AtomicU64,
}

impl SemanticCache {
    pub fn new(max_size: usize) -> Result<Self> {
        let embeddings = EmbeddingModel::load("all-MiniLM-L6-v2")?; // 50MB model
        
        Ok(Self {
            index: Hnsw::new(
                384, // Embedding dimension
                100, // Max connections
                16,  // M (for HNSW)
                200, // EF construction
                Norm::Dot, // Cosine similarity
            ),
            store: Arc::new(DashMap::new()),
            embeddings: Arc::new(embeddings),
            max_size,
            similarity_threshold: 0.95,
        })
    }
    
    pub async fn get(&self, prompt: &str) -> Option<String> {
        // Fast path: exact match first
        if let Some(entry) = self.get_exact(prompt).await {
            return Some(entry);
        }
        
        // Semantic search
        let embedding = self.embeddings.encode(prompt).await.ok()?;
        let neighbors = self.index.search(&embedding, 1, self.similarity_threshold);
        
        if let Some((idx, distance)) = neighbors.first() {
            if *distance < (1.0 - self.similarity_threshold) {
                let entry = self.store.get(idx)?;
                entry.hit_count.fetch_add(1, Ordering::Relaxed);
                entry.last_access.fetch_add(current_time(), Ordering::Relaxed);
                return Some(entry.response.clone());
            }
        }
        
        None
    }
    
    async fn get_exact(&self, prompt: &str) -> Option<String> {
        // Hash-based lookup for exact matches
        let hash = blake3::hash(prompt.as_bytes()).as_bytes().to_vec();
        let idx = u64::from_le_bytes(hash[0..8].try_into().unwrap()) as usize;
        
        self.store.get(&idx).map(|e| {
            e.hit_count.fetch_add(1, Ordering::Relaxed);
            e.response.clone()
        })
    }
    
    pub async fn put(&self, prompt: String, response: String) {
        // Prune if over capacity
        if self.store.len() >= self.max_size {
            self.evict_lru().await;
        }
        
        let tokens = num_tokens_from_messages("gpt-4", &[/* ... */]);
        let embedding = self.embeddings.encode(&prompt).await.unwrap();
        let idx = self.store.len();
        
        self.index.insert(&embedding, idx);
        self.store.insert(idx, CacheEntry {
            response,
            tokens,
            hit_count: AtomicU64::new(1),
            last_access: AtomicU64::new(current_time()),
        });
    }
    
    async fn evict_lru(&self) {
        // Remove least recently used entry
        let lru_idx = self.store.iter()
            .min_by_key(|e| e.last_access.load(Ordering::Relaxed))
            .map(|e| *e.key());
        
        if let Some(idx) = lru_idx {
            self.store.remove(&idx);
            // Note: HNSW doesn't support deletion, rebuild periodically
        }
    }
}
```

### **3.2 Response Cache with TTL**
```rust
// src/cache/response.rs
pub struct ResponseCache {
    cache: Arc<DashMap<String, CachedResponse>>,
    ttl: Duration,
}

struct CachedResponse {
    response: String,
    created_at: Instant,
    prompt_hash: u64,
}

impl ResponseCache {
    pub fn get(&self, prompt: &str) -> Option<String> {
        let hash = calculate_hash(prompt);
        let entry = self.cache.get(&hash)?;
        
        if entry.created_at.elapsed() > self.ttl {
            // TTL expired
            self.cache.remove(&hash);
            return None;
        }
        
        Some(entry.response.clone())
    }
    
    pub fn insert(&self, prompt: &str, response: String) {
        let hash = calculate_hash(prompt);
        self.cache.insert(hash, CachedResponse {
            response,
            created_at: Instant::now(),
            prompt_hash: hash,
        });
    }
}

fn calculate_hash(s: &str) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    
    let mut hasher = DefaultHasher::new();
    s.hash(&mut hasher);
    hasher.finish()
}
```

### **3.3 Prefetching System**
```rust
// src/cache/prefetch.rs
pub struct Prefetcher {
    model: Arc<dyn AIProvider>,
    pattern_cache: Arc<DashMap<String, Vec<String>>>,
}

impl Prefetcher {
    pub async fn prefetch_for_task(&self, task: &str) -> Result<()> {
        // Analyze task for likely follow-up queries
        let patterns = self.extract_patterns(task);
        
        for pattern in patterns {
            // Warm the cache
            let model = self.model.clone();
            tokio::spawn(async move {
                if let Err(e) = model.complete(&pattern).await {
                    tracing::debug!("Prefetch failed: {}", e);
                }
            });
        }
        
        Ok(())
    }
    
    fn extract_patterns(&self, task: &str) -> Vec<String> {
        // Simple pattern extraction
        let mut patterns = vec![];
        
        if task.contains("function") {
            patterns.push("fn test_".to_string());
        }
        
        if task.contains("refactor") {
            patterns.push("cargo check".to_string());
            patterns.push("cargo clippy".to_string());
        }
        
        patterns
    }
}
```

---

## **Phase 4: Production Hardening (Week 5)**

### **Goals**
- Add health checks and monitoring
- Implement circuit breakers
- Add model telemetry
- Create graceful shutdown

### **4.1 Health Check System**
```rust
// src/monitoring/health.rs
pub struct HealthChecker {
    model: Arc<dyn AIProvider>,
    profile: HardwareProfile,
}

impl HealthChecker {
    pub async fn run_health_check(&self) -> HealthReport {
        let mut results = vec![];
        
        // Test 1: Basic inference
        let test_prompts = vec![
            ("Simple", "What is 2+2? Answer with number only."),
            ("Code", "Write a Rust function to reverse a string."),
        ];
        
        for (name, prompt) in test_prompts {
            let start = Instant::now();
            let result = self.model.complete(prompt).await;
            let latency = start.elapsed();
            
            results.push(TestResult {
                name: name.to_string(),
                success: result.is_ok(),
                latency_ms: latency.as_millis() as u32,
                error: result.err().map(|e| e.to_string()),
            });
        }
        
        // Test 2: Memory usage
        let memory_usage = self.measure_memory_usage().await;
        
        // Test 3: GPU utilization (if available)
        let gpu_info = self.profile.gpu.as_ref().map(|g| {
            // Check actual GPU memory usage
            self.measure_gpu_usage()
        });
        
        HealthReport {
            timestamp: Utc::now(),
            model_name: self.model.name(),
            results,
            memory_usage,
            gpu_info,
            overall_status: self.calculate_status(&results),
        }
    }
    
    fn calculate_status(&self, results: &[TestResult]) -> HealthStatus {
        let success_rate = results.iter().filter(|r| r.success).count() as f32 / results.len() as f32;
        
        match success_rate {
            1.0 => HealthStatus::Healthy,
            0.7..=0.99 => HealthStatus::Degraded,
            _ => HealthStatus::Failing,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct HealthReport {
    pub timestamp: DateTime<Utc>,
    pub model_name: String,
    pub results: Vec<TestResult>,
    pub memory_usage: MemoryUsage,
    pub gpu_info: Option<GpuUsage>,
    pub overall_status: HealthStatus,
}

#[derive(Debug, Serialize)]
pub struct TestResult {
    pub name: String,
    pub success: bool,
    pub latency_ms: u32,
    pub error: Option<String>,
}

#[derive(Debug, Serialize)]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Failing,
}
```

### **4.2 Circuit Breaker**
```rust
// src/monitoring/circuit_breaker.rs
pub struct CircuitBreaker {
    failures: AtomicU64,
    successes: AtomicU64,
    threshold: u64,
    timeout: Duration,
    last_failure: AtomicU64, // Unix timestamp
}

impl CircuitBreaker {
    pub fn new(threshold: u64, timeout: Duration) -> Self {
        Self {
            failures: AtomicU64::new(0),
            successes: AtomicU64::new(0),
            threshold,
            timeout,
            last_failure: AtomicU64::new(0),
        }
    }
    
    pub fn is_open(&self) -> bool {
        let failures = self.failures.load(Ordering::Relaxed);
        if failures < self.threshold {
            return false;
        }
        
        let last_failure = self.last_failure.load(Ordering::Relaxed);
        let elapsed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
            - last_failure;
        
        elapsed < self.timeout.as_secs()
    }
    
    pub fn record_success(&self) {
        self.failures.store(0, Ordering::Relaxed);
        self.successes.fetch_add(1, Ordering::Relaxed);
    }
    
    pub fn record_failure(&self) {
        self.failures.fetch_add(1, Ordering::Relaxed);
        self.last_failure.store(
            SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            Ordering::Relaxed,
        );
    }
}

// Usage in adapter
pub struct MonitoredAdapter {
    inner: Arc<dyn AIProvider>,
    circuit_breaker: Arc<CircuitBreaker>,
}

#[async_trait]
impl AIProvider for MonitoredAdapter {
    async fn complete(&self, prompt: &str) -> Result<String> {
        if self.circuit_breaker.is_open() {
            bail!("Circuit breaker is open - too many failures");
        }
        
        match self.inner.complete(prompt).await {
            Ok(response) => {
                self.circuit_breaker.record_success();
                Ok(response)
            }
            Err(e) => {
                self.circuit_breaker.record_failure();
                Err(e)
            }
        }
    }
}
```

### **4.3 Telemetry & Metrics**
```rust
// src/monitoring/telemetry.rs
pub struct InferenceTracer {
    metrics: Arc<RwLock<Metrics>>,
}

#[derive(Default)]
struct Metrics {
    total_requests: u64,
    total_tokens: u64,
    total_latency: Duration,
    cache_hits: u64,
    cache_misses: u64,
    fallback_triggers: u64,
}

impl InferenceTracer {
    pub async fn record(&self, prompt: &str, result: &Result<String>) {
        let mut metrics = self.metrics.write().await;
        metrics.total_requests += 1;
        
        if let Ok(response) = result {
            let tokens = tiktoken_rs::num_tokens_from_messages("gpt-4", &[/* ... */]);
            metrics.total_tokens += tokens;
        }
        
        if metrics.total_requests % 100 == 0 {
            self.print_summary().await;
        }
    }
    
    pub async fn print_summary(&self) {
        let metrics = self.metrics.read().await;
        let avg_latency = metrics.total_latency / metrics.total_requests as u32;
        let cache_hit_rate = metrics.cache_hits as f64 / (metrics.cache_hits + metrics.cache_misses) as f64;
        
        println!("📊 Inference Summary:");
        println!("  Requests: {}", metrics.total_requests);
        println!("  Tokens: {}", metrics.total_tokens);
        println!("  Avg Latency: {:?}", avg_latency);
        println!("  Cache Hit Rate: {:.1}%", cache_hit_rate * 100.0);
        println!("  Fallbacks: {}", metrics.fallback_triggers);
    }
}
```

---

## **Phase 5: Advanced Features (Week 6-8)**

### **Goals**
- Add LoRA adapter support
- Implement vision model integration
- Create model ensemble capabilities
- Add fine-tuning pipeline

### **5.1 LoRA Adapter Support**
```rust
// src/adapters/ai/lora.rs
pub struct LoraAdapter {
    base_model: Arc<LocalLLMAdapter>,
    lora_path: PathBuf,
}

impl LoraAdapter {
    pub async fn load(base_model: Arc<LocalLLMAdapter>, lora_path: PathBuf) -> Result<Self> {
        // Verify LoRA compatibility with base model
        let adapter_config: LoRAConfig = serde_json::from_reader(
            File::open(lora_path.join("adapter_config.json"))?
        )?;
        
        if adapter_config.base_model_name != base_model.name() {
            bail!("LoRA adapter incompatible with base model");
        }
        
        Ok(Self { base_model, lora_path })
    }
    
    pub async fn activate(&self) -> Result<()> {
        // This requires native llama.cpp FFI
        // For now, we'll use llm-cpp-2 crate
        #[cfg(feature = "native-ffi")]
        unsafe {
            let c_path = CString::new(self.lora_path.to_str().unwrap())?;
            llama_apply_lora_from_file(
                self.base_model.model.as_ptr(),
                c_path.as_ptr(),
                1.0, // Scale
                std::ptr::null(),
                std::ptr::null(),
            );
        }
        
        #[cfg(not(feature = "native-ffi"))]
        {
            bail!("LoRA support requires native-ffi feature");
        }
        
        Ok(())
    }
}

#[derive(Deserialize)]
struct LoRAConfig {
    base_model_name: String,
    lora_alpha: f32,
    lora_rank: usize,
}
```

### **5.2 Vision Model Integration**
```rust
// src/adapters/ai/vision.rs
pub struct VisionAdapter {
    llava: Arc<LocalLLMAdapter>,
}

impl VisionAdapter {
    pub async fn interpret_image(&self, image_path: &Path, question: &str) -> Result<String> {
        // Convert image to base64
        let image = image::open(image_path)?;
        let resized = image.resize(336, 336, image::imageops::FilterType::Lanczos3);
        let mut buffer = Vec::new();
        resized.write_to(&mut Cursor::new(&mut buffer), ImageFormat::Png)?;
        let base64 = general_purpose::STANDARD.encode(&buffer);
        
        // Format prompt for LLaVA
        let prompt = format!(
            "<|im_start|>user\n<image>\n{}<|im_end|>\n<|im_start|>assistant\n",
            question
        );
        
        // Replace <image> token with actual image embedding
        let prompt = prompt.replace("<image>", &format!("data:image/png;base64,{}", base64));
        
        self.llava.complete(&prompt).await
    }
}
```

### **5.3 Model Ensemble**
```rust
// src/agents/ensemble.rs
pub struct ModelEnsemble {
    models: Vec<Arc<dyn AIProvider>>,
    voting_strategy: VotingStrategy,
}

impl ModelEnsemble {
    pub async fn complete(&self, prompt: &str) -> Result<String> {
        let responses = join_all(
            self.models.iter().map(|m| m.complete(prompt))
        ).await;
        
        let successful: Vec<String> = responses.into_iter()
            .filter_map(|r| r.ok())
            .collect();
        
        if successful.is_empty() {
            bail!("All models failed");
        }
        
        match self.voting_strategy {
            VotingStrategy::Majority => self.majority_vote(successful),
            VotingStrategy::BestOfN => Ok(successful.into_iter().next().unwrap()),
        }
    }
    
    fn majority_vote(&self, responses: Vec<String>) -> Result<String> {
        // Use edit distance to cluster similar responses
        let clusters = self.cluster_responses(responses);
        
        // Return the most common cluster's representative
        clusters.into_iter()
            .max_by_key(|c| c.members.len())
            .map(|c| c.representative)
            .ok_or_else(|| anyhow!("No consensus"))
    }
}
```

---

## **Configuration Management**

### **Complete Config Schema**
```toml
# ~/.config/kandil/config.toml

# Hardware Overriding (rarely needed)
[force]
# Uncomment to override auto-detection
#ram_gb = 32
#gpu_memory_gb = 12
#cpu_threads = 8

[model]
# "auto" uses hardware detection
name = "auto"
# Or specify exact model
# name = "qwen2.5-coder-7b-q4"

# Context size: "auto" or specific number
context_size = "auto"

# Quantization level
quantization = "Q4_K_M" # Options: Q3_K_M, Q4_K_M, Q5_K_M, Q6_K

[performance]
# Threading: "auto" or specific number
threads = "auto"

# Memory mapping reduces RAM usage but increases latency
use_mmap = "auto" # "auto", true, false

# Target latency for adaptive strategies
target_latency_ms = 2000

# RAM to reserve for other apps
reserve_ram_gb = 4

[cache]
# Semantic cache size (number of entries)
semantic_cache_size = 5000

# Response cache TTL in minutes
response_cache_ttl = 30

# Exact match cache size
exact_cache_size = 10000

[strategy]
# Execution mode: "auto", "local", "hybrid", "dynamic", "cloud"
mode = "auto"

# Hybrid mode settings
[hybrid]
# When to fallback to cloud (milliseconds)
timeout_ms = 30000
# Cloud provider: "claude", "openai", "ollama"
provider = "claude"

[dynamic]
# Model for simple tasks
fast_model = "qwen2.5-coder-3b-q4"
# Model for complex tasks
quality_model = "qwen2.5-coder-7b-q4"

[fallback]
# Enable cloud fallback when local fails
enabled = true
# API key (or set via KANDIL_API_KEY)
api_key = ""
# Max retries
max_retries = 3

[telemetry]
# Enable anonymous usage metrics
enable_metrics = true
# Log level: "error", "warn", "info", "debug", "trace"
log_level = "info"
# Log file location
log_file = "~/.local/share/kandil/kandil.log"

[advanced]
# LoRA adapter path
lora_path = ""
# Vision model
enable_vision = false
# Ensemble mode
ensemble_models = []
```

---

## **Testing Strategy**

### **Hardware Matrix Testing**
```bash
# .github/workflows/hardware-matrix.yml
name: Hardware Matrix Test

jobs:
  test:
    strategy:
      matrix:
        include:
          - runner: ubuntu-4gb  # Simulated low-end
            model: qwen2.5-coder-1.5b-q4
            expected_tps: 200
          
          - runner: macos-latest  # M1/M2
            model: qwen2.5-coder-7b-q4
            expected_tps: 150
          
          - runner: ubuntu-gpu  # Self-hosted with RTX 4090
            model: llama3-70b-q4
            expected_tps: 80
    
    steps:
      - uses: actions/checkout@v3
      - name: Install Kandil
        run: cargo install --path .
      
      - name: Install Model
        run: kandil model install ${{ matrix.model }}
      
      - name: Benchmark
        run: |
          kandil model benchmark ${{ matrix.model }} --format=json > results.json
          tps=$(jq '.tokens_per_second' results.json)
          if [ "$tps" -lt "${{ matrix.expected_tps }}" ]; then
            echo "Performance below threshold: $tps < ${{ matrix.expected_tps }}"
            exit 1
          fi
      
      - name: Integration Test
        run: |
          cargo test --test integration -- --model ${{ matrix.model }}
```

### **Unit Test Examples**
```rust
// src/adapters/ai/local_llm_test.rs
#[tokio::test]
async fn test_local_model_loading() {
    let profile = HardwareProfile::mock_16gb();
    let spec = profile.select_model();
    let config = AutoConfig::from_hardware(&profile);
    
    let adapter = LocalLLMAdapter::load(spec, &config.model).await.unwrap();
    
    assert!(adapter.is_available().await);
}

#[tokio::test]
async fn test_fallback_chain() {
    let chain = FallbackChain::new(vec![
        Box::new(FailingAdapter::new()),
        Box::new(MockAdapter::new("fallback response")),
    ]);
    
    let result = chain.complete("test").await.unwrap();
    assert_eq!(result, "fallback response");
}

#[tokio::test]
async fn test_semantic_cache() {
    let cache = SemanticCache::new(100);
    
    // First call - cache miss
    let result1 = cache.get("fn fibonacci").await;
    assert!(result1.is_none());
    
    cache.put("fn fibonacci".to_string(), "-> u32".to_string()).await;
    
    // Similar call - cache hit
    let result2 = cache.get("function fibonacci").await;
    assert_eq!(result2, Some("-> u32".to_string()));
}
```

---

## **Deployment & Distribution**

### **Installation Script**
```bash
#!/bin/bash
# install.sh - Universal installer

set -e

# Detect OS
OS=$(uname -s | tr '[:upper:]' '[:lower:]')
ARCH=$(uname -m)

# Map architecture
case $ARCH in
    x86_64) ARCH="x86_64" ;;
    aarch64|arm64) ARCH="aarch64" ;;
    *) echo "Unsupported architecture: $ARCH"; exit 1 ;;
esac

# Download binary
URL="https://github.com/kandil/kandil/releases/latest/download/kandil-$OS-$ARCH"
echo "Downloading Kandil from $URL..."
curl -L -o /tmp/kandil "$URL"
chmod +x /tmp/kandil

# Install
if command -v sudo &> /dev/null; then
    sudo mv /tmp/kandil /usr/local/bin/kandil
else
    mv /tmp/kandil /usr/local/bin/kandil
fi

# Post-install setup
echo "🔍 Detecting hardware..."
kandil init --non-interactive

echo "✅ Kandil installed successfully!"
echo ""
echo "Next steps:"
echo "  $ kandil model list --compatible"
echo "  $ kandil model install qwen2.5-coder-7b-q4"
echo "  $ kandil chat"
```

### **Docker Support**
```dockerfile
# Dockerfile
FROM rust:1.75 as builder
WORKDIR /app
COPY . .
RUN cargo build --release --features local-model

FROM ubuntu:22.04
RUN apt-get update && apt-get install -y \
    libgomp1 \
    && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/kandil /usr/local/bin/
COPY --from=builder /app/models /usr/share/kandil/models
ENTRYPOINT ["kandil"]
```

### **Homebrew Formula**
```ruby
# Formula/kandil.rb
class Kandil < Formula
  desc "AI-powered CLI coding assistant with local models"
  homepage "https://kandil.dev"
  version "1.0.0"
  
  if OS.mac? && Hardware::CPU.arm?
    url "https://github.com/kandil/kandil/releases/download/v1.0.0/kandil-macos-aarch64"
    sha256 "..."
  elsif OS.mac? && Hardware::CPU.intel?
    url "https://github.com/kandil/kandil/releases/download/v1.0.0/kandil-macos-x86_64"
    sha256 "..."
  elsif OS.linux? && Hardware::CPU.arm?
    url "https://github.com/kandil/kandil/releases/download/v1.0.0/kandil-linux-aarch64"
    sha256 "..."
  elsif OS.linux?
    url "https://github.com/kandil/kandil/releases/download/v1.0.0/kandil-linux-x86_64"
    sha256 "..."
  end
  
  def install
    bin.install "kandil"
    
    # Install default models
    system "#{bin}/kandil", "model", "install", "qwen2.5-coder-3b-q4"
  end
  
  test do
    system "#{bin}/kandil", "--version"
  end
end
```

---

## **Quick Reference Charts**

### **Hardware → Model Mapping**
| RAM (GB) | GPU (GB) | Recommended Model | Expected TPS | Use Case |
|----------|----------|-------------------|--------------|----------|
| 4-6 | None | Qwen2.5-1.5B-Q4 | 350 | Basic autocomplete |
| 8-12 | None | Qwen2.5-3B-Q4 | 200 | Small projects |
| 8-12 | 2-4 | Qwen2.5-7B-Q4 | 120 (CPU) / 300 (GPU) | Standard dev |
| 16-32 | 4-8 | Qwen2.5-14B-Q4 | 70 (CPU) / 150 (GPU) | Complex refactoring |
| 32-64 | 8-16 | Llama3-70B-Q4 | 40 (CPU) / 80 (GPU) | Architecture |
| 64+ | 24+ | Llama3-70B-Q6 | 25 (GPU) | Maximum quality |

### **Configuration Override Priority**
```
Priority 1: CLI arguments (--model qwen2.5-7b-q4)
Priority 2: Environment variables (KANDIL_MODEL=qwen2.5-7b-q4)
Priority 3: Project .kandil.toml ([model] name = "...")
Priority 4: User ~/.config/kandil/config.toml
Priority 5: Auto-detection based on hardware
```

### **Performance Tuning Cheat Sheet**
```bash
# Low RAM system (8GB)
kandil init --profile=minimalist
# Or manually:
export KANDIL_MODEL=qwen2.5-coder-3b-q4
export KANDIL_THREADS=4
export KANDIL_USE_MMAP=true

# High-performance system (64GB + RTX 4090)
kandil init --profile=perfectionist
# Or manually:
export KANDIL_MODEL=llama3-70b-q4
export KANDIL_GPU_LAYERS=99
export KANDIL_BATCH_SIZE=2048

# CI/CD environment (deterministic)
export KANDIL_TEMPERATURE=0.0
export KANDIL_DETERMINISTIC=true
export KANDIL_SEED=42
```

---

## **Implementation Timeline Summary**

| Phase | Duration | Key Deliverables | Status |
|-------|----------|------------------|--------|
| **0** | 5 days | Hardware detection, config layer, model catalog | ✅ |
| **1** | 5 days | LocalLLMAdapter, model CLI, basic inference | |
| **2** | 4 days | Auto-configuration, strategies, hybrid mode | |
| **3** | 5 days | Semantic cache, prefetching, performance | |
| **4** | 5 days | Health checks, circuit breaker, telemetry | |
| **5** | 10 days | LoRA, vision, ensemble (optional advanced) | |

**Total MVP Time**: ~3 weeks for fully functional local model integration
**Total with Advanced Features**: ~6 weeks

This plan provides a **rock-solid foundation** that works on **every hardware configuration** while remaining **extensible for power users**. 