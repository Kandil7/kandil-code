# **Kandil Code: Universal Local Model Integration Plan (Continued)**

---

## **2. Cross-Platform Hardware Detection (Continued)**

### **2.2 Platform-Specific Implementations**

```rust
// src/hardware/windows.rs
impl CrossPlatformHardwareDetector {
    async fn detect_windows() -> Result<HardwareReport> {
        use wmi::{COMLibrary, WMIConnection, Variant};
        use winapi::um::sysinfoapi::{GetPhysicallyInstalledSystemMemory, MEMORYSTATUSEX};
        
        let com_con = COMLibrary::new()?;
        let wmi_con = WMIConnection::new(com_con.into())?;
        
        // CPU Info
        let cpu_results: Vec<HashMap<String, Variant>> = wmi_con.raw_query(
            "SELECT Name, NumberOfCores, MaxClockSpeed FROM Win32_Processor"
        )?;
        
        let cpu = &cpu_results[0];
        let cpu_cores = cpu["NumberOfCores"].as_u32().unwrap_or(0) as usize;
        let cpu_freq_ghz = cpu["MaxClockSpeed"].as_u32().unwrap_or(0) as f64 / 1000.0;
        
        // Memory (more accurate than sysinfo on Windows)
        let mut mem_status: MEMORYSTATUSEX = unsafe { std::mem::zeroed() };
        mem_status.dwLength = std::mem::size_of::<MEMORYSTATUSEX>() as u32;
        unsafe { GetPhysicallyInstalledSystemMemory(&mut mem_status) };
        let total_ram_gb = (mem_status.ullTotalPhys as f64) / 1024.0 / 1024.0 / 1024.0;
        let available_ram_gb = (mem_status.ullAvailPhys as f64) / 1024.0 / 1024.0 / 1024.0;
        
        // Disk space
        let free_disk_gb = Self::get_windows_disk_space()?;
        
        // GPU (WMI + NVML fallback)
        let gpu = Self::detect_windows_gpu(&wmi_con)?;
        
        // Power source (laptop vs desktop)
        let is_laptop = Self::detect_windows_form_factor(&wmi_con)?;
        
        Ok(HardwareReport {
            platform: Platform::Windows,
            total_ram_gb,
            available_ram_gb,
            cpu_cores,
            cpu_freq_ghz,
            free_disk_gb,
            gpu,
            is_laptop,
            is_wsl2: false,
            battery_level: if is_laptop { Self::get_windows_battery() } else { None },
            thermal_throttle: if is_laptop { Some(Self::check_windows_thermal()) } else { None },
        })
    }
    
    fn detect_windows_gpu(wmi_con: &WMIConnection) -> Result<Option<GPUInfo>> {
        let gpu_results: Vec<HashMap<String, Variant>> = wmi_con.raw_query(
            "SELECT Name, AdapterRAM, DriverVersion FROM Win32_VideoController"
        )?;
        
        // Find NVIDIA GPU (AMD/Intel fallback)
        for gpu in gpu_results {
            if let (Some(Variant::String(name)), Some(Variant::UI4(vram))) = 
                (gpu.get("Name"), gpu.get("AdapterRAM")) {
                
                if name.contains("NVIDIA") || name.contains("AMD") {
                    return Ok(Some(GPUInfo {
                        name: name.clone(),
                        memory_total_gb: *vram as f64 / 1024.0 / 1024.0 / 1024.0,
                        memory_free_gb: 0.0, // Windows doesn't expose free VRAM easily
                        cuda_cores: Self::get_cuda_cores(name)?, // Parse from name
                        driver_version: gpu.get("DriverVersion")
                            .and_then(|v| v.as_str().map(|s| s.to_string()))
                            .unwrap_or_default(),
                    }));
                }
            }
        }
        Ok(None)
    }
    
    fn detect_windows_form_factor(wmi_con: &WMIConnection) -> Result<bool> {
        let results: Vec<HashMap<String, Variant>> = wmi_con.raw_query(
            "SELECT PCSystemType FROM Win32_ComputerSystem"
        )?;
        
        Ok(results.first()
            .and_then(|r| r.get("PCSystemType"))
            .and_then(|v| v.as_u16())
            .map(|t| t == 2) // 2 = Mobile
            .unwrap_or(false))
    }
    
    fn get_windows_battery() -> Option<f32> {
        use winapi::um::winbase::GetSystemPowerStatus;
        let mut status = unsafe { std::mem::zeroed() };
        unsafe { GetSystemPowerStatus(&mut status) };
        
        if status.BatteryLifePercent == 255 { // Unknown
            None
        } else {
            Some(status.BatteryLifePercent as f32 / 100.0)
        }
    }
}

// src/hardware/macos.rs
impl CrossPlatformHardwareDetector {
    async fn detect_macos() -> Result<HardwareReport> {
        // Use sysctl for accurate data
        let total_ram_gb = Self::sysctl_f64("hw.memsize")? / 1024.0 / 1024.0 / 1024.0;
        let cpu_cores = Self::sysctl_i32("hw.physicalcpu")? as usize;
        let cpu_freq_ghz = Self::sysctl_f64("hw.cpufrequency")? / 1e9;
        
        // Apple Silicon GPU
        let gpu = Self::detect_apple_silicon()?;
        
        // Disk space
        let free_disk_gb = Self::get_macos_disk_space()?;
        
        // macOS doesn't expose free GPU memory easily
        // Use Metal framework to estimate
        let gpu_free_gb = Self::get_metal_free_memory()?;
        
        Ok(HardwareReport {
            platform: Platform::macOS,
            total_ram_gb,
            available_ram_gb: Self::get_macos_available_ram()?,
            cpu_cores,
            cpu_freq_ghz,
            free_disk_gb,
            gpu: Some(GPUInfo {
                name: "Apple Silicon GPU".to_string(),
                memory_total_gb: gpu.memory_total_gb,
                memory_free_gb: gpu_free_gb,
                cuda_cores: 0, // Not applicable
                driver_version: Self::sysctl_string("kern.osversion")?,
            }),
            is_laptop: Self::is_macos_laptop(),
            is_wsl2: false,
            battery_level: Self::get_macos_battery(),
            thermal_throttle: Self::check_macos_thermal(),
        })
    }
    
    fn sysctl_f64(key: &str) -> Result<f64> {
        let output = std::process::Command::new("sysctl")
            .arg("-n")
            .arg(key)
            .output()?;
        Ok(String::from_utf8_lossy(&output.stdout).trim().parse()?)
    }
    
    fn detect_apple_silicon() -> Result<GPUInfo> {
        // Apple Silicon: GPU shares memory with CPU
        let total_memory = Self::sysctl_f64("hw.memsize")? / 1e9;
        
        // Estimate GPU pool (typically 50-70% of total on M1/M2/M3)
        let gpu_pool_gb = total_memory * 0.6;
        
        Ok(GPUInfo {
            name: format!("Apple {} GPU", Self::sysctl_string("hw.machine")?),
            memory_total_gb: gpu_pool_gb,
            memory_free_gb: 0.0, // Hard to detect
            cuda_cores: 0,
            driver_version: Self::sysctl_string("kern.osproductversion")?,
        })
    }
    
    fn is_macos_laptop() -> bool {
        // Check for battery presence
        std::process::Command::new("pmset")
            .arg("-g")
            .arg("batt")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }
}

// src/hardware/linux.rs
impl CrossPlatformHardwareDetector {
    async fn detect_linux() -> Result<HardwareReport> {
        let sys = sysinfo::System::new_with_specifics(
            sysinfo::RefreshKind::new()
                .with_memory()
                .with_cpu()
                .with_disks()
        );
        
        let total_ram_gb = sys.total_memory() as f64 / 1024.0 / 1024.0;
        let available_ram_gb = sys.available_memory() as f64 / 1024.0 / 1024.0;
        let cpu_cores = sys.cpus().len();
        let cpu_freq_ghz = sys.cpus().get(0).map(|c| c.frequency() as f64 / 1000.0).unwrap_or(0.0);
        
        // GPU detection (NVML or ROCm)
        let gpu = Self::detect_linux_gpu()?;
        
        // Disk space
        let free_disk_gb = sys.disks().iter()
            .find(|d| d.mount_point() == "/")
            .map(|d| d.available_space() as f64 / 1e9)
            .unwrap_or(0.0);
        
        Ok(HardwareReport {
            platform: if Self::is_wsl2() { Platform::WindowsWsl2 } else { Platform::Linux },
            total_ram_gb,
            available_ram_gb,
            cpu_cores,
            cpu_freq_ghz,
            free_disk_gb,
            gpu,
            is_laptop: Self::detect_linux_form_factor(),
            is_wsl2: Self::is_wsl2(),
            battery_level: Self::get_linux_battery(),
            thermal_throttle: Self::check_linux_thermal(),
        })
    }
    
    fn is_wsl2() -> bool {
        std::env::var("WSL_INTEROP").is_ok() || std::env::var("WSL_DISTRO_NAME").is_ok()
    }
    
    fn detect_linux_gpu() -> Result<Option<GPUInfo>> {
        // Try NVML first
        if let Ok(nvml) = nvml_wrapper::NVML::init() {
            if let Ok(device) = nvml.device_by_index(0) {
                return Ok(Some(GPUInfo {
                    name: device.name()?,
                    memory_total_gb: device.memory_info()?.total as f64 / 1e9,
                    memory_free_gb: device.memory_info()?.free as f64 / 1e9,
                    cuda_cores: device.num_cores()?,
                    driver_version: device.nvml().sys_driver_version()?,
                }));
            }
        }
        
        // Fallback to ROCm (AMD)
        Self::detect_rocm_gpu()
    }
    
    fn detect_rocm_gpu() -> Result<Option<GPUInfo>> {
        // Check for AMD GPU via rocm-smi
        let output = std::process::Command::new("rocm-smi")
            .arg("--showmeminfo")
            .output();
        
        // Parse ROCm output...
        // Simplified implementation
        Ok(None)
    }
}

// src/hardware/mobile.rs
impl CrossPlatformHardwareDetector {
    async fn detect_ios() -> Result<HardwareReport> {
        // iOS: Use limited APIs (no direct hardware access)
        // Use sysctl and NSProcessInfo via FFI
        
        Ok(HardwareReport {
            platform: Platform::iOS,
            total_ram_gb: Self::ios_total_memory()?,
            available_ram_gb: Self::ios_available_memory()?,
            cpu_cores: std::thread::available_parallelism()?.get(),
            cpu_freq_ghz: 0.0, // iOS doesn't expose
            free_disk_gb: Self::ios_free_disk()?,
            gpu: Some(GPUInfo {
                name: "Apple Neural Engine".to_string(),
                memory_total_gb: 0.0, // Shared
                memory_free_gb: 0.0,
                cuda_cores: 0,
                driver_version: "N/A".to_string(),
            }),
            is_laptop: false,
            is_wsl2: false,
            battery_level: Self::ios_battery_level(),
            thermal_throttle: Self::ios_thermal_state(),
        })
    }
    
    async fn detect_android() -> Result<HardwareReport> {
        // Android: Use ActivityManager and native APIs via JNI
        // Requires Android permission: ACTIVITY_RECOGNITION
        
        Ok(HardwareReport {
            platform: Platform::Android,
            total_ram_gb: Self::android_total_memory()?,
            available_ram_gb: Self::android_available_memory()?,
            cpu_cores: num_cpus::get(),
            cpu_freq_ghz: Self::android_cpu_freq()?,
            free_disk_gb: Self::android_free_disk()?,
            gpu: Self::android_gpu_info()?,
            is_laptop: false,
            is_wsl2: false,
            battery_level: Self::android_battery_level(),
            thermal_throttle: Self::android_thermal_state(),
        })
    }
}
```

### **2.3 Hardware Compatibility Matrix**

```rust
// src/model/harware_matrix.rs
/// Hardcoded compatibility rules per platform
pub struct HardwareCompatibilityMatrix;

impl HardwareCompatibilityMatrix {
    /// Returns max recommended model size per platform
    pub fn max_model_size_gb(platform: &Platform, gpu: Option<&GPUInfo>) -> f64 {
        match (platform, gpu) {
            // Mobile devices: Tiny models
            (Platform::iOS, _) => 2.0, // 2GB max on iPhone
            (Platform::Android, _) => 3.0, // 3GB on high-end Android
            
            // Edge devices: Small models
            (Platform::EdgeDevice, Some(g)) if g.memory_total_gb > 8.0 => 7.0, // Jetson Nano
            (Platform::EdgeDevice, _) => 4.0,
            
            // Desktops/Laptops: Based on RAM
            (Platform::Windows, Some(g)) if g.memory_total_gb > 20.0 => 100.0, // 100GB with 24GB VRAM
            (Platform::Windows, Some(g)) if g.memory_total_gb > 12.0 => 70.0,
            (Platform::Windows, _) => 40.0, // CPU fallback
            
            (Platform::WindowsWsl2, Some(g)) => 70.0, // WSL2 can use GPU
            (Platform::WindowsWsl2, _) => 30.0,
            
            (Platform::macOS, Some(g)) => 50.0, // Apple Silicon unified memory
            (Platform::macOS, _) => 8.0, // Older Intel Macs
            
            (Platform::Linux, Some(g)) if g.memory_total_gb > 24.0 => 100.0,
            (Platform::Linux, Some(g)) if g.memory_total_gb > 12.0 => 70.0,
            (Platform::Linux, _) => 50.0,
            
            (Platform::LinuxArm64, Some(g)) => 8.0, // ARM servers (AWS Graviton)
            (Platform::LinuxArm64, _) => 4.0,
        }
    }
    
    /// Recommended quantization level per platform
    pub fn recommended_quantization(platform: &Platform, ram_gb: f64) -> &'static str {
        match platform {
            Platform::iOS | Platform::Android => "Q4_0", // Mobile: ultra-low size
            Platform::EdgeDevice => "Q5_K_M", // Edge: balanced
            Platform::macOS if ram_gb > 16.0 => "Q6_K", // M1/M2/M3: high quality
            Platform::macOS => "Q5_K_M",
            Platform::Windows | Platform::Linux if ram_gb > 32.0 => "Q8_0", // High-end desktops
            Platform::Windows | Platform::Linux if ram_gb > 16.0 => "Q6_K",
            _ => "Q4_K_M", // Default
        }
    }
}
```

---

## **3. Platform-Native AI Runtime Adapters**

### **3.1 Windows Native Adapters**

```rust
// src/adapters/windows/ollama_wsl2.rs
//! Ollama WSL2 Adapter (Recommended for Windows)
//! Bridges to Ollama running in WSL2 for GPU passthrough

pub struct OllamaWsl2Adapter {
    client: Arc<Client>,
    wsl_ip: String,
    auth_token: Option<SecretString>,
}

impl OllamaWsl2Adapter {
    pub async fn new(profile: ModelProfile) -> Result<Self> {
        let wsl_ip = Self::get_wsl2_ip().await?;
        
        // Verify Ollama is accessible in WSL2
        Self::verify_ollama_in_wsl(&wsl_ip).await?;
        
        // Check for GPU passthrough
        let has_gpu = Self::check_wsl_gpu().await?;
        if !has_gpu {
            tracing::warn!("WSL2 GPU passthrough not detected. Performance will be reduced.");
        }
        
        Ok(Self {
            client: Arc::new(Client::new()),
            wsl_ip,
            auth_token: Self::get_auth_token()?,
        })
    }
    
    async fn get_wsl2_ip() -> Result<String> {
        // Method 1: wsl hostname -I
        let output = tokio::process::Command::new("wsl")
            .arg("hostname")
            .arg("-I")
            .output()
            .await?;
        
        let ip = String::from_utf8_lossy(&output.stdout)
            .trim()
            .split_whitespace()
            .next()
            .ok_or_else(|| anyhow::anyhow!("No WSL2 IP found"))?;
        
        Ok(ip.to_string())
    }
    
    async fn verify_ollama_in_wsl(ip: &str) -> Result<()> {
        let client = Client::new();
        let url = format!("http://{}:11434/api/tags", ip);
        
        match tokio::time::timeout(Duration::from_secs(5), client.get(&url).send()).await {
            Ok(Ok(res)) if res.status().is_success() => Ok(()),
            Ok(Ok(res)) => bail!("Ollama in WSL2 returned error: {}", res.status()),
            Ok(Err(e)) => bail!("Cannot connect to Ollama in WSL2: {}", e),
            Err(_) => bail!("Ollama in WSL2 connection timeout"),
        }
    }
    
    async fn check_wsl_gpu() -> Result<bool> {
        // Check if nvidia-smi works in WSL2
        let output = tokio::process::Command::new("wsl")
            .arg("nvidia-smi")
            .arg("--query-gpu=count")
            .arg("--format=csv,noheader")
            .output()
            .await?;
        
        Ok(!output.stdout.is_empty() && String::from_utf8_lossy(&output.stdout).trim() != "0")
    }
}

#[async_trait]
impl AIProvider for OllamaWsl2Adapter {
    // Identical to regular OllamaAdapter but uses WSL2 IP
    // Requests go to http://{wsl_ip}/api/...
}
```

**CLI Commands**:
```bash
# Windows WSL2 Quick Setup
$ kandil windows setup-wsl2
✅ Checking WSL2 installation...
✅ Installing Ollama in WSL2...
✅ Configuring GPU passthrough...
✅ Testing Ollama connection...
✅ Setup complete! Run: kandil model add ollama:qwen2.5-coder:14b

# Verify GPU
$ kandil windows check-gpu-wsl
✅ GPU Passthrough: NVIDIA RTX 4070 detected
✅ CUDA Version: 12.2
✅ Driver Version: 535.98
✅ Ready for GPU acceleration
```

---

### **3.2 macOS Native Adapters**

```rust
// src/adapters/macos/coreml.rs
//! macOS Core ML Adapter (Apple Silicon)
//! Leverages Apple Neural Engine for maximum performance

use core_ml_bindings::{MLModel, MLModelConfiguration, MLComputeUnits};

pub struct CoreMLAdapter {
    model: MLModel,
    profile: ModelProfile,
}

impl CoreMLAdapter {
    pub fn new(profile: ModelProfile) -> Result<Self> {
        // Check for Core ML compatible model
        let model_path = Self::get_coreml_model_path(&profile.name)?;
        
        // Verify model exists and is optimized
        if !model_path.exists() {
            bail!(
                "Core ML model not found: {}\n\
                Convert from GGUF using: kandil model convert --format coreml {}",
                model_path.display(),
                profile.name
            );
        }
        
        let config = MLModelConfiguration::new();
        
        // Force use of Neural Engine on Apple Silicon
        if cfg!(target_arch = "aarch64") {
            config.set_compute_units(MLComputeUnits::ALL); // CPU+GPU+ANE
        } else {
            config.set_compute_units(MLComputeUnits::CPU_ONLY);
        }
        
        let model = MLModel::new_with_configuration(&model_path.to_string_lossy(), &config)
            .map_err(|e| anyhow::anyhow!("Core ML model loading failed: {:?}", e))?;
        
        Ok(Self { model, profile })
    }
    
    fn get_coreml_model_path(name: &str) -> Result<PathBuf> {
        let home = dirs::home_dir().ok_or_else(|| anyhow::anyhow!("No home dir"))?;
        Ok(home.join(".kandil").join("models").join("coreml").join(format!("{}.mlmodelc", name)))
    }
}

#[async_trait]
impl AIProvider for CoreMLAdapter {
    async fn complete(&self, prompt: &str) -> Result<String> {
        // Core ML is synchronous, run in blocking thread
        let model = self.model.clone();
        let prompt = prompt.to_string();
        
        let output = tokio::task::spawn_blocking(move || {
            let input = core_ml_bindings::NSDictionary::new();
            input.set("prompt", &prompt);
            input.set("max_tokens", &(self.profile.max_tokens as i64));
            input.set("temperature", &self.profile.temperature);
            
            let prediction = model.predict(&input)
                .map_err(|e| anyhow::anyhow!("Core ML prediction failed: {:?}", e))?;
            
            prediction.get_str("response")
                .ok_or_else(|| anyhow::anyhow!("No response in prediction"))
        }).await??;
        
        Ok(output)
    }
}

// Model conversion helper
// src/adapters/macos/convert.rs
pub struct CoreMLConverter;

impl CoreMLConverter {
    /// Convert GGUF to Core ML format
    pub async fn convert_gguf_to_coreml(
        gguf_path: &Path,
        output_name: &str,
    ) -> Result<PathBuf> {
        tracing::info!("Converting {} to Core ML format...", gguf_path.display());
        
        // Use Apple's coremltools via Python
        let output = tokio::process::Command::new("python3")
            .arg("-m")
            .arg("coremltools")
            .arg("convert")
            .arg("--source")
            .arg("gguf")
            .arg("--input")
            .arg(gguf_path)
            .arg("--output")
            .arg(format!("{}.mlmodel", output_name))
            .arg("--compute-units")
            .arg("all") // Use CPU+GPU+ANE
            .output()
            .await?;
        
        if !output.status.success() {
            bail!("Core ML conversion failed: {}", String::from_utf8_lossy(&output.stderr));
        }
        
        // Compile model for faster loading
        let compiled_path = Self::compile_model(&format!("{}.mlmodel", output_name))?;
        
        Ok(compiled_path)
    }
    
    fn compile_model(model_path: &str) -> Result<PathBuf> {
        let output = std::process::Command::new("xcrun")
            .arg("coremlcompiler")
            .arg("compile")
            .arg(model_path)
            .arg(format!("{}.mlmodelc", model_path))
            .output()?;
        
        if !output.status.success() {
            bail!("Core ML compilation failed");
        }
        
        Ok(PathBuf::from(format!("{}.mlmodelc", model_path)))
    }
}
```

**CLI Commands**:
```bash
# Convert model to Core ML (one-time)
$ kandil model convert qwen2.5-coder-7b --format coreml
📦 Converting Qwen2.5-Coder-7B to Core ML...
⚠️  This will take 5-10 minutes
✅ Conversion complete: ~/.kandil/models/coreml/qwen2.5-coder-7b.mlmodelc
✅ Optimized for Apple Neural Engine

# Use Core ML model
$ kandil /model use qwen2.5-coder-7b --runtime coreml
✅ Using Core ML runtime (ANE accelerated)

# Benchmark
$ kandil model benchmark qwen2.5-coder-7b --runtime coreml
📊 Core ML vs GGUF:
   - Core ML: 45ms / 1k tokens (ANE)
   - GGUF: 120ms / 1k tokens (GPU)
   Speedup: 2.7x
```

---

### **3.3 Linux Native Adapters**

```rust
// src/adapters/linux/ollama.rs
//! Linux Ollama Adapter (with SELinux/AppArmor support)

pub struct OllamaLinuxAdapter {
    client: Arc<Client>,
    socket_path: PathBuf,
}

impl OllamaLinuxAdapter {
    pub async fn new(profile: ModelProfile) -> Result<Self> {
        // Check if Ollama is running systemd service
        let socket_path = PathBuf::from("/var/run/ollama/ollama.sock");
        
        if !socket_path.exists() {
            // Fallback to TCP
            Self::verify_tcp_connection().await?;
        }
        
        // Check SELinux context
        #[cfg(target_os = "linux")]
        Self::check_selinux_context(&socket_path)?;
        
        Ok(Self {
            client: Arc::new(Client::new()),
            socket_path,
        })
    }
    
    #[cfg(target_os = "linux")]
    fn check_selinux_context(socket_path: &Path) -> Result<()> {
        use std::os::unix::fs::PermissionsExt;
        
        let metadata = socket_path.metadata()?;
        let permissions = metadata.permissions();
        
        if permissions.mode() & 0o777 != 0o660 {
            bail!(
                "SELinux/AppArmor: Ollama socket has insecure permissions: {:o}\n\
                Fix: sudo chmod 660 {}", 
                permissions.mode() & 0o777,
                socket_path.display()
            );
        }
        
        Ok(())
    }
    
    async fn verify_tcp_connection() -> Result<()> {
        let client = Client::new();
        client.get("http://localhost:11434/api/tags")
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("Ollama not accessible via TCP: {}", e))?;
        Ok(())
    }
}

#[async_trait]
impl AIProvider for OllamaLinuxAdapter {
    async fn complete(&self, prompt: &str) -> Result<String> {
        if self.socket_path.exists() {
            // Use Unix socket for better security
            self.complete_via_socket(prompt).await
        } else {
            // Fallback to TCP
            self.complete_via_tcp(prompt).await
        }
    }
}
```

---

### **3.4 Edge Device & Mobile Adapters**

```rust
// src/adapters/edge/onnx.rs
//! ONNX Runtime Adapter for Edge Devices (Raspberry Pi, Jetson, etc.)
//! Quantizes models to INT8/FP16 for maximum efficiency

pub struct OnnxEdgeAdapter {
    session: ort::Session,
    profile: ModelProfile,
}

impl OnnxEdgeAdapter {
    pub fn new(profile: ModelProfile) -> Result<Self> {
        // Verify ONNX Runtime is built for edge
        Self::verify_onnx_build()?;
        
        // Load quantized model
        let model_path = Self::get_quantized_model(&profile.name)?;
        
        let session = ort::Session::builder()?
            .with_optimization_level(ort::GraphOptimizationLevel::Level3)?
            .with_intra_threads(num_cpus::get())?
            .with_execution_mode(ort::ExecutionMode::ORT_SEQUENTIAL)?
            .commit_from_file(model_path)?;
        
        Ok(Self { session, profile })
    }
    
    fn verify_onnx_build() -> Result<()> {
        // Check for NEON on ARM, AVX2 on x86
        #[cfg(target_arch = "aarch64")]
        {
            if !std::arch::is_aarch64_feature_detected!("neon") {
                bail!("NEON not detected. ONNX Runtime will be slow on ARM.");
            }
        }
        
        Ok(())
    }
    
    fn get_quantized_model(name: &str) -> Result<PathBuf> {
        // Models must be pre-quantized for edge
        let path = dirs::home_dir()
            .ok_or_else(|| anyhow::anyhow!("No home dir"))?
            .join(".kandil")
            .join("models")
            .join("onnx")
            .join(format!("{}_int8.onnx", name));
        
        if !path.exists() {
            bail!(
                "Quantized ONNX model not found: {}\n\
                Convert using: kandil model quantize --format onnx_int8 {}",
                path.display(),
                name
            );
        }
        
        Ok(path)
    }
}

#[async_trait]
impl AIProvider for OnnxEdgeAdapter {
    async fn complete(&self, prompt: &str) -> Result<String> {
        // ONNX Runtime is sync, run in thread pool
        let session = self.session.clone();
        let prompt = prompt.to_string();
        
        tokio::task::spawn_blocking(move || {
            let inputs = ort::inputs! {
                "prompt" => prompt,
                "max_length" => self.profile.max_tokens as i64,
                "temperature" => self.profile.temperature,
            }?;
            
            let outputs = session.run(inputs)?;
            Ok(outputs["response"].extract_string()?)
        }).await?
    }
}

// src/mobile/ios/coreml_mobile.rs
//! iOS Core ML Mobile Adapter
//! Uses Core ML with model protection (encrypted models)

pub struct IOSCoreMLAdapter {
    model: MLModel,
    profile: ModelProfile,
}

impl IOSCoreMLAdapter {
    pub fn new(profile: ModelProfile) -> Result<Self> {
        // iOS requires models in app bundle or encrypted download
        let model_url = Self::get_protected_model_url(&profile.name)?;
        
        let config = MLModelConfiguration::new();
        config.set_compute_units(MLComputeUnits::CPU_AND_NE); // Battery efficient
        
        // For encrypted models
        config.set_model_encryption_key(Self::get_encryption_key()?);
        
        let model = MLModel::new_with_configuration(&model_url, &config)
            .map_err(|e| anyhow::anyhow!("iOS model loading failed: {:?}", e))?;
        
        Ok(Self { model, profile })
    }
    
    fn get_encryption_key() -> Result<Vec<u8>> {
        // Use iOS Keychain to store model decryption key
        let entry = SecurityFramework::password::PasswordEntry::new("kandil.model.key")?;
        entry.get_password().map(|s| s.into_bytes())
    }
}

// src/mobile/android/ai_core.rs
//! Android AI Core / TensorFlow Lite Adapter
//! Uses Google AI Core service for on-device inference

pub struct AndroidAICoreAdapter {
    model: tflite::FlatBufferModel,
    interpreter: tflite::Interpreter<'static, 'static>,
}

impl AndroidAICoreAdapter {
    pub fn new(profile: ModelProfile) -> Result<Self> {
        // Check for AI Core availability
        if !Self::is_ai_core_available()? {
            bail!("Android AI Core not available on this device. Requires Android 12+");
        }
        
        // Download model via Google Play Services
        let model_handle = Self::download_model(&profile.name).await?;
        
        // Load with TensorFlow Lite
        let model = tflite::FlatBufferModel::build_from_file(&model_handle.path)?;
        
        let resolver = tflite::ops::builtin::BuiltinOpResolver::new();
        let builder = tflite::InterpreterBuilder::new(&model, &resolver)?;
        let interpreter = builder.build()?;
        
        Ok(Self { model, interpreter })
    }
}
```

---

## **4. Cross-Platform CLI Commands**

### **4.1 Universal Commands (All Platforms)**
```bash
# Add model (auto-detects best runtime)
$ kandil model add qwen2.5-coder-7b
🔍 Platform detected: macOS (Apple Silicon)
🔍 Hardware: 36GB RAM, 12-core CPU, 30-core GPU
💡 Recommendation: Use Core ML runtime for 3x speedup
❓ Use Core ML? [Y/n] y
📦 Converting model to Core ML...
✅ Model ready: qwen2.5-coder-7b (Core ML, ANE accelerated)

# Benchmark all runtimes
$ kandil model benchmark qwen2.5-coder-7b --all-runtimes
📊 Benchmark Results (macOS, M2 Max):
1. Core ML (ANE):   42ms / 1k tokens ⚡ Fastest
2. GGUF (GPU):     125ms / 1k tokens
3. GGUF (CPU):     890ms / 1k tokens
4. Qwen API:       180ms / 1k tokens
   (Network overhead)

# Doctor command (cross-platform diagnostics)
$ kandil doctor --platform
✅ Platform: Linux (Ubuntu 22.04)
✅ Architecture: x86_64
✅ CPU: Intel i9-13900K (12P+16E cores)
✅ RAM: 64GB DDR5
✅ GPU: NVIDIA RTX 4090 (24GB) ✅ CUDA 12.2 detected
✅ Disk: 2TB NVMe (1.2TB free)
✅ Security: SELinux enforcing, keyring active
⚠️  Ollama: Running but not using GPU passthrough
   Fix: sudo apt install nvidia-container-toolkit && sudo systemctl restart ollama
💡 Optimal model: qwen2.5-coder-32b (fits in GPU memory)
```

### **4.2 Platform-Specific Commands**

```bash
# Windows
$ kandil windows setup
✅ Checking Visual Studio Build Tools...
✅ Checking WSL2...
✅ Installing Ollama in WSL2...
✅ Configuring GPU passthrough for NVIDIA...
✅ Firewall: Blocking Ollama external access for security
✅ Testing connection...
✅ Windows setup complete!

# macOS
$ kandil macos setup
✅ Checking Xcode Command Line Tools...
✅ Installing Core ML optimization tools...
✅ Downloading quantized models for ANE...
✅ Setting up LaunchAgent for auto-start...
✅ macOS setup complete!

# Linux (Ubuntu/Debian)
$ kandil linux setup
✅ Updating package lists...
✅ Installing Ollama and dependencies...
✅ Configuring SELinux/AppArmor policies...
✅ Setting systemd limits (ulimit -n 65536)...
✅ Linux setup complete!

# Edge Device (Raspberry Pi)
$ kandil edge setup
⚠️  Limited hardware detected: 8GB RAM, no GPU
✅ Installing ONNX Runtime (ARM-optimized)...
✅ Downloading INT8 quantized models...
✅ Disabling unnecessary services for memory...
✅ Edge setup complete!
```

### **4.3 Mobile Commands (iOS/Android)**

```bash
# iOS (via Termux or Shortcuts integration)
$ kandil mobile ios sync
📱 Syncing models from desktop...
✅ Encrypted model bundle created
✅ Uploaded to iCloud Drive
✅ Open Kandil iOS app to import

# Android (via Termux)
$ kandil mobile android sync
📱 Syncing models from desktop...
✅ Downloaded to /sdcard/kandil/models
✅ Run in Termux: kandil --mobile /ask "question"

# Check mobile compatibility
$ kandil mobile check
📱 iPhone 15 Pro (iOS 17)
💾 Available storage: 45GB
🔋 Battery: 78%
⚠️  Warning: Large models will drain battery
💡 Recommended: qwen2.5-coder-3b (Core ML quantized)
```

---

## **5. Security Model Per Platform**

### **5.1 Platform-Specific Security Hardening**

```rust
// src/security/platform/windows.rs
pub struct WindowsSecurityHardening;

impl WindowsSecurityHardening {
    /// Apply Windows-specific security best practices
    pub fn apply() -> Result<()> {
        // 1. Firewall: Block Ollama external access
        Self::block_ollama_external()?;
        
        // 2. AppLocker: Prevent unsigned model execution
        Self::configure_applocker()?;
        
        // 3. BitLocker: Ensure disk encryption
        if !Self::is_bitlocker_enabled()? {
            tracing::warn!("BitLocker not enabled. Model files are not encrypted at rest.");
        }
        
        // 4. Windows Defender Exclusion: Add Kandil cache
        Self::add_defender_exclusion()?;
        
        Ok(())
    }
    
    fn block_ollama_external() -> Result<()> {
        // PowerShell: Block Ollama port 11434 on public networks
        let script = r#"
        New-NetFirewallRule -DisplayName "Block Ollama External" 
                            -Direction Inbound 
                            -Protocol TCP 
                            -LocalPort 11434 
                            -Action Block 
                            -Profile Public
        "#;
        
        std::process::Command::new("powershell")
            .arg("-Command")
            .arg(script)
            .output()?;
        
        Ok(())
    }
    
    fn is_bitlocker_enabled() -> Result<bool> {
        let output = std::process::Command::new("powershell")
            .arg("-Command")
            .arg("Get-BitLockerVolume -MountPoint C: | Select-Object -ExpandProperty VolumeStatus")
            .output()?;
        
        Ok(String::from_utf8_lossy(&output.stdout).contains("FullyEncrypted"))
    }
}

// src/security/platform/macos.rs
pub struct MacosSecurityHardening;

impl MacosSecurityHardening {
    pub fn apply() -> Result<()> {
        // 1. FileVault: Check encryption
        if !Self::is_filevault_enabled()? {
            tracing::warn!("FileVault not enabled. Models stored unencrypted.");
        }
        
        // 2. Gatekeeper: Verify Kandil binary is signed
        if !Self::verify_code_signature()? {
            bail!("Kandil binary not properly signed. Run: codesign --verify $(which kandil)");
        }
        
        // 3. Keychain: Ensure Kandil has access
        Self::verify_keychain_access()?;
        
        // 4. Firewall: Block external LM Studio if running
        Self::block_lmstudio_external()?;
        
        Ok(())
    }
    
    fn is_filevault_enabled() -> Result<bool> {
        let output = std::process::Command::new("fdesetup")
            .arg("isactive")
            .output()?;
        
        Ok(String::from_utf8_lossy(&output.stdout).trim() == "true")
    }
    
    fn verify_keychain_access() -> Result<()> {
        // Test keychain access
        let entry = keyring::Entry::new("kandil", "test")?;
        entry.set_password("test")?;
        entry.delete_password()?;
        Ok(())
    }
}

// src/security/platform/linux.rs
pub struct LinuxSecurityHardening;

impl LinuxSecurityHardening {
    pub fn apply() -> Result<()> {
        // 1. SELinux/AppArmor: Check policies
        if Self::is_selinux_enabled() {
            Self::verify_selinux_policy()?;
        }
        
        // 2. Disk encryption: Check LUKS
        if !Self::is_luks_enabled()? {
            tracing::warn!("Root partition not LUKS encrypted. Models at risk.");
        }
        
        // 3. Ollama socket: Verify permissions
        if Path::new("/var/run/ollama/ollama.sock").exists() {
            self.verify_socket_permissions()?;
        }
        
        // 4. Swap encryption: Warn if disabled
        if !Self::is_swap_encrypted()? {
            tracing::warn!("Swap not encrypted. Memory dumps may contain model data.");
        }
        
        Ok(())
    }
    
    fn is_selinux_enabled() -> bool {
        Path::new("/sys/fs/selinux/enforce").exists()
    }
    
    fn verify_selinux_policy() -> Result<()> {
        let output = std::process::Command::new("sestatus")
            .output()?;
        
        let status = String::from_utf8_lossy(&output.stdout);
        if status.contains("Current mode:                   enforcing") {
            // Check if custom Ollama policy is loaded
            if !status.contains("kandil_ollama_policy") {
                tracing::warn!("SELinux enabled but no Kandil Ollama policy loaded.\n\
                              Run: sudo semodule -i /usr/share/kandil/selinux/kandil_ollama.pp");
            }
        }
        
        Ok(())
    }
    
    fn is_swap_encrypted() -> Result<bool> {
        let output = std::process::Command::new("swapon")
            .arg("--show")
            .output()?;
        
        Ok(String::from_utf8_lossy(&output.stdout).contains("crypt"))
    }
}

// src/security/platform/mobile.rs
pub struct MobileSecurityHardening;

impl MobileSecurityHardening {
    /// iOS: Verify app store compliance
    pub fn verify_ios_compliance() -> Result<()> {
        // Ensure models are encrypted in bundle
        if !Self::is_model_bundle_encrypted()? {
            bail!("iOS requires models to be encrypted in app bundle");
        }
        
        // Verify app uses App Transport Security
        Self::check_ats_configuration()?;
        
        Ok(())
    }
    
    /// Android: Verify storage encryption
    pub fn verify_android_security() -> Result<()> {
        // Check if device encryption is enabled
        if !Self::is_device_encrypted()? {
            bail!("Android device encryption must be enabled for Kandil mobile");
        }
        
        // Verify scoped storage usage
        Self::check_scoped_storage()?;
        
        Ok(())
    }
}
```

---

## **6. Performance Benchmarking Suite**

### **6.1 Cross-Platform Benchmark Tool**

```rust
// src/benchmark/mod.rs
pub struct CrossPlatformBenchmark;

impl CrossPlatformBenchmark {
    /// Run comprehensive benchmark for a model across all compatible runtimes
    pub async fn benchmark_all_runtimes(
        &self,
        model_name: &str,
    ) -> Result<BenchmarkReport> {
        let hardware = CrossPlatformHardwareDetector::detect().await?;
        let mut results = vec![];
        
        // Get compatible runtimes
        let runtimes = self.get_compatible_runtimes(&hardware);
        
        for runtime in runtimes {
            tracing::info!("Benchmarking {} on {}", model_name, runtime.name());
            
            let result = self.benchmark_runtime(model_name, runtime).await?;
            results.push(result);
        }
        
        Ok(BenchmarkReport {
            model: model_name.to_string(),
            hardware,
            results,
            timestamp: chrono::Utc::now(),
        })
    }
    
    async fn benchmark_runtime(
        &self,
        model_name: &str,
        runtime: Box<dyn AIProvider>,
    ) -> Result<RuntimeBenchmark> {
        let test_prompts = vec![
            "Hello".to_string(),
            "Write a Rust function to parse JSON".to_string(),
            "Explain quantum computing".to_string(),
        ];
        
        let mut latencies = vec![];
        let mut tokens_per_sec = vec![];
        
        for prompt in test_prompts {
            let start = Instant::now();
            let response = runtime.complete(&prompt).await?;
            let duration = start.elapsed();
            
            // Estimate tokens (rough)
            let token_count = response.len() / 4; // ~4 chars per token
            
            latencies.push(duration.as_millis() as u64);
            tokens_per_sec.push((token_count as f64 / duration.as_secs_f64()) as u32);
        }
        
        Ok(RuntimeBenchmark {
            runtime_name: runtime.name(),
            avg_latency_ms: latencies.iter().sum::<u64>() / latencies.len() as u64,
            avg_tokens_per_sec: tokens_per_sec.iter().sum::<u32>() / tokens_per_sec.len() as u32,
            memory_peak_mb: self.measure_memory_peak(runtime).await?,
            battery_impact: self.measure_battery_impact(runtime).await?, // Mobile only
        })
    }
    
    fn get_compatible_runtimes(&self, hardware: &HardwareReport) -> Vec<Box<dyn AIProvider>> {
        let mut runtimes = vec![];
        
        match hardware.platform {
            Platform::macOS => {
                if cfg!(target_arch = "aarch64") {
                    runtimes.push(Box::new(CoreMLAdapter::new(...)?));
                }
                runtimes.push(Box::new(LlamaCppAdapter::new(...)?));
            }
            Platform::iOS => {
                runtimes.push(Box::new(IOSCoreMLAdapter::new(...)?));
            }
            Platform::Android => {
                if Self::is_ai_core_available() {
                    runtimes.push(Box::new(AndroidAICoreAdapter::new(...)?));
                }
                runtimes.push(Box::new(TFLiteAdapter::new(...)?));
            }
            _ => {
                // Desktop platforms
                if Self::is_ollama_available() {
                    runtimes.push(Box::new(OllamaAdapter::new(...)?));
                }
                if Self::is_lmstudio_available() {
                    runtimes.push(Box::new(LMStudioAdapter::new(...)?));
                }
                if Self::is_llamacpp_available() {
                    runtimes.push(Box::new(LlamaCppAdapter::new(...)?));
                }
            }
        }
        
        runtimes
    }
}

#[derive(Serialize)]
pub struct BenchmarkReport {
    pub model: String,
    pub hardware: HardwareReport,
    pub results: Vec<RuntimeBenchmark>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Serialize)]
pub struct RuntimeBenchmark {
    pub runtime_name: String,
    pub avg_latency_ms: u64,
    pub avg_tokens_per_sec: u32,
    pub memory_peak_mb: u64,
    pub battery_impact: Option<f32>, // Percentage battery drain per minute
}
```

### **6.2 Benchmark CLI Output**
```bash
$ kandil benchmark qwen2.5-coder-7b --all-runtimes
📊 Cross-Platform Benchmark
🖥️  Platform: macOS (Apple Silicon M2 Max)
📋 Hardware: 36GB RAM, 12-core CPU, 30-core GPU

⚡ Running 3 test prompts across 2 runtimes...

Runtime: Core ML (ANE)
├─ Latency (avg):      45ms
├─ Throughput:         1,428 tokens/sec
├─ Memory peak:        4.2GB
├─ Battery impact:     -2%/min (negligible)
└─ Score:              ⭐⭐⭐⭐⭐ (5/5)

Runtime: llama.cpp (GPU)
├─ Latency (avg):      128ms
├─ Throughput:           892 tokens/sec
├─ Memory peak:        4.5GB
├─ Battery impact:     -15%/min (heavy)
└─ Score:              ⭐⭐⭐⭐ (4/5)

📈 Recommendation: Use Core ML runtime for 2.8x speed and 7x less battery drain.
```

---

## **7. Implementation Roadmap: Cross-Platform**

### **Phase 0: Foundation & Security (Weeks 1-4)**
**Priority: P0 (Critical)**

- [ ] **Cross-Platform Hardware Detection**
  - Implement all platform detectors (Windows, macOS, Linux, Mobile, Edge)
  - Add hardware cache with TTL
  - Create compatibility matrix

- [ ] **Security Hardening per Platform**
  - Windows: Firewall rules, BitLocker check
  - macOS: FileVault, Gatekeeper, Keychain
  - Linux: SELinux/AppArmor, LUKS, socket permissions
  - Mobile: App Store compliance, encryption verification

- [ ] **Core Registry & Credential Manager**
  - Universal Model Registry singleton
  - Secure credential storage (keyring)
  - Built-in model profiles (50+ models)

- [ ] **CI/CD for Cross-Platform**
  - GitHub Actions: Windows, macOS, Linux matrix
  - Cross-compilation for ARM64
  - Mobile builds (iOS via macOS runner, Android via Gradle)

### **Phase 1: Desktop Adapters (Weeks 5-8)**
**Priority: P1 (High)**

- [ ] **Windows Adapters**
  - Ollama WSL2 (primary)
  - LM Studio (with auth enforcement)
  - GPT4All
  - Foundry Local

- [ ] **macOS Adapters**
  - Core ML (ANE optimized)
  - Ollama (native macOS)
  - LM Studio (macOS version)

- [ ] **Linux Adapters**
  - Ollama (socket + TCP)
  - Llama.cpp (static build)
  - LM Studio (Linux version)

- [ ] **Testing**
  - Integration tests per platform
  - Mock hardware detectors
  - Security audit per platform

### **Phase 2: Mobile & Edge (Weeks 9-12)**
**Priority: P2 (Medium)**

- [ ] **iOS Adapter**
  - Core ML mobile
  - Model encryption and protection
  - Swift FFI bindings

- [ ] **Android Adapter**
  - AI Core integration
  - TensorFlow Lite
  - JNI bindings

- [ ] **Edge Devices**
  - ONNX Runtime (ARM64 optimized)
  - Raspberry Pi support
  - NVIDIA Jetson support (CUDA on ARM)

- [ ] **Cross-Platform Sync**
  - Encrypted model sync via iCloud/Google Drive
  - Project sync with end-to-end encryption

### **Phase 3: Performance & Polish (Weeks 13-16)**
**Priority: P2 (Medium)**

- [ ] **Benchmarking Suite**
  - Automated benchmarks per commit
  - Performance regression detection
  - Battery impact tests (mobile)

- [ ] **TUI / GUI**
  - Ratatui for desktop
  - SwiftUI for iOS
  - Jetpack Compose for Android

- [ ] **Platform-Specific Features**
  - Windows: System tray integration, PowerShell cmdlets
  - macOS: Quick Actions, Spotlight integration
  - Linux: systemd service, man pages
  - Mobile: Shortcuts/Siri/Widgets

### **Phase 4: Release & Community (Weeks 17-20)**
**Priority: P3 (Low)**

- [ ] **Release Artifacts**
  - Windows: .msi installer (Wix)
  - macOS: .dmg, Homebrew formula
  - Linux: .deb, .rpm, AppImage, Snap
  - iOS: App Store submission
  - Android: Play Store + F-Droid

- [ ] **Documentation**
  - Platform-specific setup guides
  - Security hardening per platform
  - Video tutorials for each OS

---

## **8. Testing Strategy: Cross-Platform CI**

### **8.1 GitHub Actions Matrix**
```yaml
# .github/workflows/ci.yml
jobs:
  test-cross-platform:
    strategy:
      matrix:
        include:
          - os: windows-latest
            target: x86_64-pc-windows-msvc
            features: "wsl2"
            
          - os: macos-latest
            target: aarch64-apple-darwin
            features: "coreml"
            
          - os: ubuntu-latest
            target: x86_64-unknown-linux-gnu
            features: "selinux"
            
          - os: ubuntu-latest
            target: aarch64-unknown-linux-gnu
            features: "arm64"
            
    runs-on: ${{ matrix.os }}
    
    steps:
      - uses: actions/checkout@v4
      
      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          targets: ${{ matrix.target }}
      
      - name: Install Platform Dependencies
        run: |
          if [ "${{ matrix.os }}" = "windows-latest" ]; then
            choco install wsl2 --no-progress
          elif [ "${{ matrix.os }}" = "macos-latest" ]; then
            brew install coremltools
          elif [ "${{ matrix.os }}" = "ubuntu-latest" ]; then
            sudo apt-get update
            sudo apt-get install -y libselinux1-dev libapparmor-dev
          fi
      
      - name: Build
        run: cargo build --target ${{ matrix.target }} --features ${{ matrix.features }}
      
      - name: Test
        run: cargo test --target ${{ matrix.target }} --features ${{ matrix.features }}
      
      - name: Integration Test
        run: ./target/${{ matrix.target }}/debug/kandil doctor --strict

  test-mobile:
    runs-on: macos-latest  # For iOS builds
    
    steps:
      - uses: actions/checkout@v4
      
      - name: Build iOS
        run: |
          cargo install cargo-ndk
          cargo ndk -t aarch64-apple-ios build --release
      
      - name: Build Android
        uses: actions/setup-java@v3
        with:
          distribution: 'temurin'
          java-version: '17'
      - run: ./gradlew build
```

### **8.2 Platform-Specific Integration Tests**
```rust
#[cfg(test)]
mod platform_tests {
    #[test]
    #[cfg(target_os = "windows")]
    fn test_windows_firewall_rule() {
        // Ensure Ollama is blocked on public networks
        let output = std::process::Command::new("powershell")
            .arg("-Command")
            .arg("Get-NetFirewallRule -DisplayName 'Block Ollama External'")
            .output()
            .unwrap();
        
        assert!(output.status.success());
    }
    
    #[test]
    #[cfg(target_os = "macos")]
    fn test_filevault_enabled() {
        let output = std::process::Command::new("fdesetup")
            .arg("isactive")
            .output()
            .unwrap();
        
        assert!(String::from_utf8_lossy(&output.stdout).trim() == "true");
    }
    
    #[test]
    #[cfg(target_os = "linux")]
    fn test_selinux_ollama_policy() {
        let output = std::process::Command::new("sestatus")
            .output()
            .unwrap();
        
        let status = String::from_utf8_lossy(&output.stdout);
        assert!(status.contains("kandil_ollama_policy") || !status.contains("enforcing"));
    }
}
```

---

## **9. Summary: Platform-Specific Recommendations**

### **🪟 Windows**
- **Best Runtime**: Ollama in WSL2
- **Security**: Firewall + keyring + BitLocker
- **Performance**: GPU passthrough via WSL2 CUDA
- **Install**: `winget install kandil-code` + `kandil windows setup`

### **🍎 macOS**
- **Best Runtime**: Core ML (Apple Silicon)
- **Security**: FileVault + Gatekeeper + encrypted keychain
- **Performance**: Apple Neural Engine (3-5x faster than CPU)
- **Install**: `brew install kandil-code` + `kandil macos setup`

### **🐧 Linux**
- **Best Runtime**: Ollama (socket) + NVIDIA GPU
- **Security**: SELinux/AppArmor + LUKS + socket permissions
- **Performance**: CUDA with TensorRT
- **Install**: `apt install kandil-code` + `kandil linux setup`

### **📱 iOS**
- **Best Runtime**: Core ML (ANE)
- **Security**: App Store sandbox + encrypted models + keyring
- **Performance**: Apple Neural Engine (battery efficient)
- **Install**: App Store

### **🤖 Android**
- **Best Runtime**: AI Core (Android 12+) or TensorFlow Lite
- **Security**: Scoped storage + device encryption
- **Performance**: GPU/NPU acceleration via NNAPI
- **Install**: Play Store

### **🔧 Edge Devices**
- **Best Runtime**: ONNX Runtime INT8 quantized
- **Security**: Physical security + encrypted SD card
- **Performance**: NEON instructions on ARM64
- **Install**: `cargo install --target aarch64-unknown-linux-gnu`

---

## **10. Final Architecture Diagram**

```
┌─────────────────────────────────────────────────────────────────────────┐
│                        Kandil Code Interface                           │
│                    (CLI / TUI / iOS / Android)                        │
└──────────────────────────────┬──────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────────────────┐
│              Cross-Platform Model Registry & Router                   │
│              (Hardware Detection → Runtime Selection)                 │
└──────────────────────────────┬──────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────────────────┐
│                      Platform Runtime Layer                            │
│                                                                        │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐ │
│  │ Windows     │  │ macOS       │  │ Linux       │  │ Mobile/Edge │ │
│  │ • Ollama    │  │ • Core ML   │  │ • Ollama    │  │ • Core ML   │ │
│  │ • LM Studio │  │ • Ollama    │  │ • Llama.cpp │  │ • AI Core   │ │
│  │ • GPT4All   │  │ • LM Studio │  │ • Foundry   │  │ • ONNX      │ │
│  │ • Foundry   │  │ • Foundry   │  │ • LM Studio │  │ • TFLite    │ │
│  └─────────────┘  └─────────────┘  └─────────────┘  └─────────────┘ │
│                                                                        │
│  Security: 🔒 Keyring | 🔒 Encryption | 🔒 Sandbox | 🔒 TPM/Secure Enclave │
└──────────────────────────────┬──────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────────────────┐
│                      Hardware & OS Layer                               │
│               (CPU | GPU | RAM | Disk | Battery | Thermal)            │
└─────────────────────────────────────────────────────────────────────────┘
```

**Result**: A **single codebase** that delivers **native performance** and **security** on **every platform**, from **Raspberry Pi to iPhone to Windows Server**.