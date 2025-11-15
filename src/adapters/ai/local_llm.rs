use async_trait::async_trait;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;
use llm::{Model as LlmModel, InferenceError, ModelParameters, TokenBias, InferenceRequest, InferenceResponse, InferenceStats, LoadProgress, ggml::Context};
use crate::config::layered::{Config, ModelConfig};
use crate::models::catalog::{ModelSpec, MODEL_CATALOG};
use crate::errors::LocalModelError;

// Define a trait that all AI providers must implement
#[async_trait]
pub trait AIProvider: Send + Sync {
    async fn complete(&self, prompt: &str) -> Result<String, LocalModelError>;
    async fn is_available(&self) -> bool;
    async fn name(&self) -> String;
}

pub struct LocalLLMAdapter {
    model: Arc<Mutex<Box<dyn LlmModel>>>,
    config: ModelConfig,
    context_size: usize,
    threads: usize,
}

impl LocalLLMAdapter {
    pub async fn load(spec_name: &str, config: &ModelConfig) -> Result<Self, LocalModelError> {
        let spec = MODEL_CATALOG
            .iter()
            .find(|model| model.name == spec_name)
            .ok_or_else(|| LocalModelError::ModelNotFound {
                name: spec_name.to_string(),
            })?;

        let model_path = match &config.path {
            Some(path) => path.join(&spec.filename),
            None => {
                // Default to a models directory in user's home
                let mut path = dirs::data_dir()
                    .unwrap_or_else(|| std::env::current_dir().unwrap())
                    .join("kandil")
                    .join("models");
                std::fs::create_dir_all(&path).map_err(|e| LocalModelError::IoError { source: e })?;
                path.join(&spec.filename)
            }
        };

        if !model_path.exists() {
            return Err(LocalModelError::ConfigurationError {
                message: format!(
                    "Model not found at {:?}. Run `kandil model install {}`",
                    model_path, spec.name
                ),
            });
        }

        // Load the model
        let params = ModelParameters {
            n_threads: config.threads.unwrap_or(4) as i32,
            n_batch: 512,
            n_gpu_layers: 0, // Start with CPU only, GPU support can be added later
            rope_freq_scale: 1.0,
            rope_freq_base: 10000.0,
            use_mmap: true,
            use_mlock: false,
        };

        let model_result = tokio::task::spawn_blocking(move || {
            llm::load_dynamic(
                llm::ModelArchitecture::Llama,
                &model_path,
                params,
                LoadProgress::default(),
                llm::KnownModel::Llama,
            )
        })
        .await
        .map_err(|e| LocalModelError::ModelLoadError {
            source: Box::new(e),
        })?;

        let model = match model_result {
            Ok(m) => m,
            Err(e) => return Err(LocalModelError::ModelLoadError {
                source: Box::new(e),
            }),
        };

        Ok(Self {
            model: Arc::new(Mutex::new(model)),
            config: config.clone(), // Need to make ModelConfig cloneable
            context_size: config.context_size,
            threads: config.threads.unwrap_or(4),
        })
    }

    pub async fn default() -> Result<Self, LocalModelError> {
        // Load the default model based on hardware
        use crate::core::hardware::detect_hardware;
        use crate::core::auto_config::AutoConfig;

        let hardware = detect_hardware();
        let auto_config = AutoConfig::from_hardware(&hardware);
        
        Self::load(&auto_config.model.name, &auto_config.model).await
    }
}

#[async_trait]
impl AIProvider for LocalLLMAdapter {
    async fn complete(&self, prompt: &str) -> Result<String, LocalModelError> {
        let mut session = {
            let model = self.model.lock().await;
            model.start_session(Default::default())
        };

        let mut output = String::new();
        
        // Use a channel for streaming tokens
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();

        // Clone needed values
        let model_clone = Arc::clone(&self.model);
        let prompt = prompt.to_string();
        let max_tokens = 2048; // Configurable parameter

        tokio::task::spawn_blocking(move || {
            let model = model_clone.blocking_lock();
            
            let request = InferenceRequest {
                prompt: prompt.as_str().into(),
                parameters: &llm::InferenceParameters {
                    n_threads: 4, // Use configured value
                    n_batch: 512,
                    temperature: 0.2,
                    top_k: 40,
                    top_p: 0.9,
                    repeat_penalty: 1.1,
                    frequency_penalty: 0.0,
                    presence_penalty: 0.0,
                    penalize_nl: true,
                    logits_bias: TokenBias::default(),
                    n_probs: 0,
                    stop_sequence: Vec::new(),  // Configurable stop sequence
                },
                play_back_previous_tokens: false,
                maximum_token_count: Some(max_tokens),
            };

            let mut stats = InferenceStats::default();
            let result = session.infer::<std::convert::Infallible>(
                &*model,
                &request,
                &mut Default::default(),
                #[allow(clippy::redundant_closure)]
                |token| {
                    tx.send(token.to_string()).map_err(|e| std::convert::Infallible)
                },
            );

            match result {
                Ok(s) => stats = s,
                Err(e) => {
                    // Send error through channel
                    let _ = tx.send(format!("__ERROR__:{}", e));
                }
            }
        });

        // Collect tokens as they arrive
        while let Ok(token) = rx.recv().await {
            if token.starts_with("__ERROR__:") {
                let error_msg = token.strip_prefix("__ERROR__:").unwrap_or(&token);
                return Err(LocalModelError::ModelLoadError {
                    source: Box::new(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        error_msg.to_string(),
                    )),
                });
            }
            output.push_str(&token);
        }

        Ok(output)
    }

    async fn is_available(&self) -> bool {
        // Local model is available if it was successfully loaded
        true
    }

    async fn name(&self) -> String {
        self.config.name.clone()
    }
}