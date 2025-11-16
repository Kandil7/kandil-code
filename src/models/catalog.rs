use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
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

// Serializable/Deserializable version for configuration (without static references)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializableModelSpec {
    pub name: String,
    pub size_gb: f64,
    pub ram_required_gb: u64,
    pub gpu_vram_min: Option<u64>,
    pub speed_rating: Speed,
    pub quality_rating: Quality,
    pub description: String,
    pub context_sizes: Vec<usize>, // Use owned Vec for serialization
}

impl From<&ModelSpec> for SerializableModelSpec {
    fn from(spec: &ModelSpec) -> Self {
        SerializableModelSpec {
            name: spec.name.to_string(),
            size_gb: spec.size_gb,
            ram_required_gb: spec.ram_required_gb,
            gpu_vram_min: spec.gpu_vram_min,
            speed_rating: spec.speed_rating.clone(),
            quality_rating: spec.quality_rating.clone(),
            description: spec.description.to_string(),
            context_sizes: spec.context_sizes.to_vec(), // Convert slice to vector
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Speed {
    UltraFast(usize), // tokens per second
    VeryFast(usize),
    Fast(usize),
    Medium(usize),
    Slow(usize),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Quality {
    Basic,
    Good,
    VeryGood,
    Excellent,
    Superior,
}

impl Quality {
    pub fn as_i32(&self) -> i32 {
        match self {
            Quality::Basic => 1,
            Quality::Good => 2,
            Quality::VeryGood => 3,
            Quality::Excellent => 4,
            Quality::Superior => 5,
        }
    }
}

impl Speed {
    pub fn tps(&self) -> usize {
        match self {
            Speed::UltraFast(tps) => *tps,
            Speed::VeryFast(tps) => *tps,
            Speed::Fast(tps) => *tps,
            Speed::Medium(tps) => *tps,
            Speed::Slow(tps) => *tps,
        }
    }
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

impl ModelSpec {
    pub fn find_by_name(name: &str) -> Option<&'static ModelSpec> {
        MODEL_CATALOG.iter().find(|model| model.name == name)
    }

    pub fn get_compatible_models(ram_gb: u64) -> Vec<&'static ModelSpec> {
        MODEL_CATALOG
            .iter()
            .filter(|model| model.ram_required_gb <= ram_gb)
            .collect()
    }

    pub fn supports_context_size(&self, size: usize) -> bool {
        self.context_sizes.contains(&size)
    }
}

// Add the module to the models mod.rs file
