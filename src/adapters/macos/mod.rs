use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct CoremlRuntimeStatus {
    pub coremltools_available: bool,
    pub ane_available: bool,
    pub notes: Vec<String>,
}

impl CoremlRuntimeStatus {
    pub fn detect() -> Self {
        detect_coreml_status()
    }
}

pub fn setup_instructions() -> &'static str {
    "Core ML Setup:\n\
1. Install Xcode command line tools: `xcode-select --install`\n\
2. Install coremltools: `pip3 install coremltools`\n\
3. Enable developer mode: Settings → Privacy & Security → Developer Mode\n\
4. Convert GGUF models with `kandil model convert <name> --format coreml`\n\
5. Use `kandil model use <name> --runtime coreml` to run via ANE."
}

#[cfg(target_os = "macos")]
fn detect_coreml_status() -> CoremlRuntimeStatus {
    let coremltools_available = Command::new("python3")
        .args(["-c", "import coremltools"])
        .status()
        .map(|status| status.success())
        .unwrap_or(false);

    let ane_available = cfg!(target_arch = "aarch64");

    let mut notes = Vec::new();
    if !coremltools_available {
        notes.push("Install coremltools with `pip install coremltools`.".to_string());
    }
    if !ane_available {
        notes.push("Apple Neural Engine not detected; falling back to CPU/GPU.".to_string());
    }

    CoremlRuntimeStatus {
        coremltools_available,
        ane_available,
        notes,
    }
}

#[cfg(not(target_os = "macos"))]
fn detect_coreml_status() -> CoremlRuntimeStatus {
    CoremlRuntimeStatus {
        coremltools_available: false,
        ane_available: false,
        notes: vec!["Core ML runtime available only on macOS.".to_string()],
    }
}
