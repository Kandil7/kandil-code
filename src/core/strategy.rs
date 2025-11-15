//! Execution strategies for Kandil Code
//!
//! Contains different strategies for executing AI tasks with local models.

use std::sync::Arc;
use std::time::Duration;
use tokio::time::timeout;

use crate::adapters::ai::local_llm::{AIProvider, LocalLLMAdapter};
use crate::config::layered::{Config, StrategyConfig};
use crate::errors::LocalModelError;

pub enum ExecutionStrategy {
    LocalOnly {
        model: Arc<dyn AIProvider>,
    },
    Hybrid {
        local: Arc<dyn AIProvider>,
        cloud: Arc<dyn AIProvider>,
        threshold: Duration,
    },
    Dynamic {
        fast_model: Arc<dyn AIProvider>,
        quality_model: Arc<dyn AIProvider>,
        cloud: Option<Arc<dyn AIProvider>>,
    },
}

#[derive(Debug, Clone)]
pub enum ExecutionMode {
    Local,
    Hybrid,
    Dynamic,
    CloudOnly,
}

impl ExecutionMode {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "local" => Some(ExecutionMode::Local),
            "hybrid" => Some(ExecutionMode::Hybrid),
            "dynamic" => Some(ExecutionMode::Dynamic),
            "cloud" => Some(ExecutionMode::CloudOnly),
            _ => None,
        }
    }
}

impl ExecutionStrategy {
    pub async fn create(config: &Config) -> Result<Self, LocalModelError> {
        match &config.strategy.mode {
            ExecutionMode::Local => {
                let model = LocalLLMAdapter::load(&config.model.name, &config.model).await?;
                Ok(Self::LocalOnly { 
                    model: Arc::new(model) as Arc<dyn AIProvider>
                })
            }

            ExecutionMode::Hybrid => {
                // For now, we'll create a local-only strategy as a placeholder
                // since we don't have cloud adapters implemented yet
                let local = LocalLLMAdapter::load(&config.model.name, &config.model).await?;
                
                // Placeholder cloud adapter - in a full implementation this would be a cloud provider
                let cloud = create_placeholder_cloud_adapter();
                
                Ok(Self::Hybrid {
                    local: Arc::new(local) as Arc<dyn AIProvider>,
                    cloud: Arc::new(cloud) as Arc<dyn AIProvider>,
                    threshold: Duration::from_millis(config.strategy.timeout_ms),
                })
            }

            ExecutionMode::Dynamic => {
                // Load two models: fast (smaller) and quality (larger)
                let fast_model_name = config.strategy.fast_model.clone().unwrap_or_else(|| get_smaller_model(&config.model.name));
                let quality_model_name = config.model.name.clone();

                let fast_adapter = LocalLLMAdapter::load(&fast_model_name, &config.model).await?;
                let quality_adapter = LocalLLMAdapter::load(&quality_model_name, &config.model).await?;

                // Placeholder cloud adapter for fallback
                let cloud = if config.fallback.enabled {
                    Some(Arc::new(create_placeholder_cloud_adapter()) as Arc<dyn AIProvider>)
                } else {
                    None
                };

                Ok(Self::Dynamic {
                    fast_model: Arc::new(fast_adapter) as Arc<dyn AIProvider>,
                    quality_model: Arc::new(quality_adapter) as Arc<dyn AIProvider>,
                    cloud,
                })
            }

            ExecutionMode::CloudOnly => {
                // Placeholder cloud adapter
                let cloud = create_placeholder_cloud_adapter();
                Ok(Self::LocalOnly { 
                    model: Arc::new(cloud) as Arc<dyn AIProvider>
                })
            }
        }
    }

    pub async fn complete(&self, prompt: &str) -> Result<String, LocalModelError> {
        match self {
            ExecutionStrategy::LocalOnly { model } => {
                model.complete(prompt).await
            }

            ExecutionStrategy::Hybrid { local, cloud, threshold } => {
                match timeout(threshold.clone(), local.complete(prompt)).await {
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
                // Analyze the complexity of the task
                let complexity_analysis = crate::core::task_complexity::TaskComplexity::analyze(prompt);
                
                match complexity_analysis {
                    crate::core::task_complexity::TaskComplexity::Simple => fast_model.complete(prompt).await,
                    crate::core::task_complexity::TaskComplexity::Medium => quality_model.complete(prompt).await,
                    crate::core::task_complexity::TaskComplexity::Complex => {
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

// Placeholder for a cloud adapter
struct PlaceholderCloudAdapter;

#[async_trait::async_trait]
impl AIProvider for PlaceholderCloudAdapter {
    async fn complete(&self, prompt: &str) -> Result<String, LocalModelError> {
        // In a real implementation, this would call a cloud API
        Ok(format!("CLOUD_RESPONSE: {}", prompt)) // Placeholder implementation
    }

    async fn is_available(&self) -> bool {
        // Placeholder - in reality, check if cloud API is accessible
        true
    }

    async fn name(&self) -> String {
        "PlaceholderCloud".to_string()
    }
}

fn create_placeholder_cloud_adapter() -> PlaceholderCloudAdapter {
    PlaceholderCloudAdapter
}

fn get_smaller_model(larger_model: &str) -> String {
    // Simple mapping for demonstration - in reality, this would map to actual smaller models
    match larger_model {
        "qwen2.5-coder-14b-q4" => "qwen2.5-coder-3b-q4".to_string(),
        "qwen2.5-coder-7b-q4" => "qwen2.5-coder-3b-q4".to_string(),
        "llama3.1-70b-q4" => "qwen2.5-coder-7b-q4".to_string(),
        _ => "qwen2.5-coder-1.5b-q4".to_string(), // Default fallback
    }
}