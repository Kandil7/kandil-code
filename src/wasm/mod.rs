use anyhow::{anyhow, Result};

#[cfg(target_arch = "wasm32")]
pub struct WasmKandil;

#[cfg(target_arch = "wasm32")]
impl WasmKandil {
    pub fn new() -> Self {
        Self
    }

    pub async fn run_in_browser(&self) -> Result<()> {
        // Placeholder runtime for future WebGPU / GGML integration.
        // We keep it async so the signature matches the design plan.
        Ok(())
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub struct WasmKandil;

#[cfg(not(target_arch = "wasm32"))]
impl WasmKandil {
    pub fn new() -> Self {
        Self
    }

    pub async fn run_in_browser(&self) -> Result<()> {
        Err(anyhow!(
            "WASM runtime is only available under wasm32 target"
        ))
    }
}
