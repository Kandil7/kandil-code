//! Auto-configuration engine for Kandil Code
//!
//! Automatically selects optimal settings based on hardware capabilities.

use crate::config::layered::{
    Config, FallbackConfig, ModelConfig, PerformanceConfig, Quantization,
};
use crate::core::hardware::HardwareProfile;
use crate::models::catalog::MODEL_CATALOG;

pub struct AutoConfig;

impl AutoConfig {
    pub fn from_hardware(profile: &HardwareProfile) -> Config {
        let model_spec = Self::select_model(profile);
        let performance = Self::tune_performance(profile, model_spec);
        let fallback = Self::configure_fallback(profile);

        Config {
            model: ModelConfig {
                name: model_spec.name.to_string(),
                path: None,
                context_size: Self::select_context_size(profile, model_spec),
                quantization: Quantization::Q4_K_M, // Default quantization
                threads: Some(performance.threads), // Use the performance threads value
            },
            performance,
            fallback,
            strategy: crate::config::layered::StrategyConfig::default(),
        }
    }

    fn select_model(profile: &HardwareProfile) -> &'static crate::models::catalog::ModelSpec {
        // Hard constraints first - filter by available RAM
        let compatible: Vec<_> = MODEL_CATALOG
            .iter()
            .filter(|m| m.ram_required_gb <= profile.total_ram_gb)
            .collect();

        // If GPU is available, prefer GPU-capable models
        if let Some(gpu) = &profile.gpu {
            if let Some(best) = compatible
                .iter()
                .filter(|m| m.gpu_vram_min.is_none() || m.gpu_vram_min.unwrap() <= gpu.memory_gb)
                .max_by_key(|m| m.quality_rating.as_i32())
            {
                return best;
            }
        }

        // CPU-only fallback - pick the highest quality model that fits in RAM
        if let Some(best) = compatible
            .iter()
            .filter(|m| m.gpu_vram_min.is_none()) // CPU-only models
            .max_by_key(|m| m.quality_rating.as_i32())
        {
            return best;
        }

        // Last resort: smallest model that fits
        MODEL_CATALOG
            .iter()
            .filter(|m| m.ram_required_gb <= profile.total_ram_gb)
            .min_by_key(|m| m.size_gb as u64)
            .unwrap_or_else(|| MODEL_CATALOG.first().unwrap()) // Fallback to first model
    }

    fn select_context_size(
        profile: &HardwareProfile,
        model: &crate::models::catalog::ModelSpec,
    ) -> usize {
        // Start with the largest context size the model supports
        let max_supported = *model.context_sizes.iter().max().unwrap_or(&4096);

        // Estimate based on available RAM (leaving some headroom)
        let ram_based = if profile.total_ram_gb > 0 {
            // Rough calculation: larger models need more memory per token
            let memory_per_token_mb = if model.size_gb > 10.0 { 0.5 } else { 0.25 }; // Rough estimate
            let available_for_context = (profile.available_ram_gb.saturating_sub(2)) as f64; // Reserve 2GB
            ((available_for_context * 1024.0) / memory_per_token_mb) as usize
        } else {
            4096 // Default if we can't calculate
        };

        // Use the smaller of model capability and RAM-based calculation
        let max_usable = max_supported.min(ram_based);

        // Find the largest context size that fits within our limit
        *model
            .context_sizes
            .iter()
            .filter(|&&size| size <= max_usable)
            .max()
            .unwrap_or(&4096) // Safe default
    }

    fn tune_performance(
        profile: &HardwareProfile,
        _model_spec: &crate::models::catalog::ModelSpec,
    ) -> PerformanceConfig {
        let threads = if profile.total_ram_gb < 8 {
            // On low RAM systems, use fewer threads to reduce memory pressure
            profile.cpu_physical_cores.min(4).max(1) // At least 1 thread, max 4
        } else {
            // Use more threads on systems with adequate RAM
            profile.cpu_physical_cores.min(8).max(1) // Cap at 8 for most consumer systems
        };

        let use_mmap = profile.total_ram_gb >= 16; // MMAP is beneficial on systems with sufficient RAM

        PerformanceConfig {
            threads,
            use_mmap,
            batch_size: 512,         // Standard batch size
            target_latency_ms: 2000, // 2 second target for response time
            reserve_ram_gb: profile.total_ram_gb.saturating_div(4).max(2).min(8), // Reserve 25% or between 2-8GB
        }
    }

    fn configure_fallback(profile: &HardwareProfile) -> FallbackConfig {
        FallbackConfig {
            enabled: profile.total_ram_gb < 16, // Enable fallback for systems with less than 16GB
            cloud_provider: None,               // User must configure API key
            timeout_ms: 30000,                  // 30 second timeout for fallback requests
        }
    }
}
