use anyhow::Context;
use reqwest::Client;
use serde::Serialize;
use std::process::Command;
use tokio::time::{timeout, Duration};
use tracing::warn;

#[derive(Debug, Serialize)]
pub struct WindowsAdapterStatus {
    pub wsl_ip: Option<String>,
    pub ollama_reachable: bool,
    pub lmstudio_reachable: bool,
    pub gpt4all_reachable: bool,
    pub foundry_reachable: bool,
}

impl WindowsAdapterStatus {
    pub async fn gather() -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(2))
            .build()
            .unwrap_or_else(|_| Client::new());

        let wsl_ip = detect_wsl_ip();
        let ollama_url = wsl_ip
            .as_ref()
            .map(|ip| format!("http://{}:11434/api/tags", ip))
            .unwrap_or_else(|| "http://localhost:11434/api/tags".to_string());

        Self {
            wsl_ip,
            ollama_reachable: http_ok(&client, &ollama_url).await,
            lmstudio_reachable: http_ok(&client, "http://localhost:1234/v1/models").await,
            gpt4all_reachable: http_ok(&client, "http://localhost:4891/v1/models").await,
            foundry_reachable: http_ok(&client, "http://localhost:5001/v1/models").await,
        }
    }
}

#[derive(Debug)]
pub struct WindowsGpuReport {
    pub detected: bool,
    pub message: String,
}

pub fn check_wsl_gpu() -> WindowsGpuReport {
    #[cfg(target_os = "windows")]
    {
        let output = Command::new("wsl")
            .arg("nvidia-smi")
            .arg("--query-gpu=name,memory.total")
            .arg("--format=csv,noheader")
            .output();

        match output {
            Ok(result) if result.status.success() => {
                let stdout = String::from_utf8_lossy(&result.stdout);
                if stdout.trim().is_empty() {
                    WindowsGpuReport {
                        detected: false,
                        message: "No GPU exposed to WSL2. Install the NVIDIA driver with WSL support.".to_string(),
                    }
                } else {
                    WindowsGpuReport {
                        detected: true,
                        message: format!("Detected GPU(s): {}", stdout.trim()),
                    }
                }
            }
            Ok(result) => WindowsGpuReport {
                detected: false,
                message: format!(
                    "Unable to query GPU via wsl nvidia-smi (status {}). Ensure drivers are installed.",
                    result.status
                ),
            },
            Err(err) => WindowsGpuReport {
                detected: false,
                message: format!("Failed to execute wsl nvidia-smi: {err}"),
            },
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        WindowsGpuReport {
            detected: false,
            message: "GPU passthrough checks are only available on Windows hosts.".to_string(),
        }
    }
}

pub fn setup_wsl2_instructions() -> &'static str {
    "WSL2 Setup Steps:\n\
1. Enable WSL: `wsl --install`\n\
2. Install Ubuntu from the Microsoft Store.\n\
3. Inside WSL run: `curl -fsSL https://ollama.ai/install.sh | sh`\n\
4. Start Ollama: `sudo systemctl enable --now ollama`\n\
5. From PowerShell run `kandil windows check-gpu` to confirm GPU passthrough."
}

pub fn preferred_ollama_endpoint() -> String {
    if let Some(ip) = detect_wsl_ip() {
        format!("http://{}:11434", ip)
    } else {
        "http://localhost:11434".to_string()
    }
}

pub fn detect_wsl_ip() -> Option<String> {
    detect_wsl_ip_inner()
}

#[cfg(target_os = "windows")]
fn detect_wsl_ip_inner() -> Option<String> {
    let output = Command::new("wsl")
        .arg("hostname")
        .arg("-I")
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    stdout.split_whitespace().next().map(|s| s.to_string())
}

#[cfg(not(target_os = "windows"))]
fn detect_wsl_ip_inner() -> Option<String> {
    None
}

async fn http_ok(client: &Client, url: &str) -> bool {
    match timeout(Duration::from_secs(2), client.get(url).send()).await {
        Ok(Ok(response)) => response.status().is_success(),
        Ok(Err(err)) => {
            warn!("HTTP probe to {} failed: {}", url, err);
            false
        }
        Err(_) => false,
    }
}
