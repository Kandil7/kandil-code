use serde::{Deserialize, Serialize};
use sysinfo::{System, SystemExt};
use thiserror::Error;

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

#[derive(Error, Debug)]
pub enum GpuDetectionError {
    #[error("No GPU found")]
    NoGpuFound,
    #[cfg(feature = "nvidia")]
    #[error("NVML error: {0}")]
    NvmlError(#[from] nvml_wrapper::error::NvmlError),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
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
    #[cfg(feature = "nvidia")]
    if let Ok(nvml) = nvml_wrapper::Nvml::init() {
        if let Ok(device) = nvml.device_by_index(0) {
            return Ok(GpuInfo {
                brand: "NVIDIA".to_string(),
                model: device.name()?,
                memory_gb: device.memory_info()?.total / (1024 * 1024 * 1024),
                compute_capability: device.cuda_compute_capability().ok(),
                driver_version: "N/A".to_string(), // Simplifying for now to avoid nested Result
            });
        }
    }

    // Try AMD ROCm on Linux
    #[cfg(all(target_os = "linux", feature = "rocm"))]
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

#[cfg(target_os = "macos")]
fn is_apple_silicon() -> bool {
    // Simple heuristic: check if we're on macOS with ARM64
    cfg!(target_arch = "aarch64") && cfg!(target_os = "macos")
}

#[cfg(target_os = "macos")]
fn detect_apple_gpu_memory() -> Result<u64, GpuDetectionError> {
    // For Apple Silicon, we can estimate based on available RAM
    // In a real implementation, we would use Metal API to get exact GPU memory
    let sys = sysinfo::System::new();
    Ok(sys.total_memory() / (1024 * 1024 * 1024))
}

#[cfg(all(target_os = "linux", feature = "rocm"))]
fn detect_rocm_gpu() -> Result<GpuInfo, GpuDetectionError> {
    // This is a placeholder implementation
    // In a real implementation, we would query ROCm
    // This might involve parsing /opt/rocm/ files or using ROCm API
    // For now, we return an error to fall back to other methods
    Err(GpuDetectionError::NoGpuFound)
}