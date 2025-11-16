use serde::Serialize;
use std::path::Path;
use std::process::Command;

#[derive(Debug, Serialize)]
pub struct LinuxRuntimeStatus {
    pub ollama_socket: bool,
    pub cuda_available: bool,
    pub notes: Vec<String>,
}

impl LinuxRuntimeStatus {
    pub fn detect() -> Self {
        detect_runtime_status()
    }
}

pub fn setup_instructions() -> &'static str {
    "Linux Local Runtime Setup:\n\
1. Install Ollama: https://ollama.ai/download\n\
2. Ensure `/var/run/ollama/ollama.sock` is owned by ollama:root with 660 perms.\n\
3. Install NVIDIA Container Toolkit for GPU acceleration.\n\
4. Run `systemctl enable --now ollama` to start the daemon.\n\
5. Use `kandil local-model status` to verify socket & CUDA availability."
}

#[cfg(target_os = "linux")]
fn detect_runtime_status() -> LinuxRuntimeStatus {
    let ollama_socket = Path::new("/var/run/ollama/ollama.sock").exists();

    let cuda_available = Command::new("bash")
        .arg("-c")
        .arg("command -v nvidia-smi >/dev/null 2>&1")
        .status()
        .map(|status| status.success())
        .unwrap_or(false);

    let mut notes = Vec::new();
    if !ollama_socket {
        notes.push("Ollama socket not found at /var/run/ollama/ollama.sock.".to_string());
    }
    if !cuda_available {
        notes.push("CUDA tools not detected (nvidia-smi missing).".to_string());
    }

    LinuxRuntimeStatus {
        ollama_socket,
        cuda_available,
        notes,
    }
}

#[cfg(not(target_os = "linux"))]
fn detect_runtime_status() -> LinuxRuntimeStatus {
    LinuxRuntimeStatus {
        ollama_socket: false,
        cuda_available: false,
        notes: vec!["Linux runtime checks skipped on this platform.".to_string()],
    }
}
