#[cfg(target_os = "windows")]
use crate::adapters::windows;
use crate::core::adapters::ai::KandilAI;
use crate::core::hardware::{detect_hardware, HardwareProfile};
use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use reqwest::Client;
use serde::Serialize;
use std::sync::Arc;
use std::time::{Duration, Instant};
use sysinfo::{RefreshKind, System, SystemExt};
use tokio::time::{timeout, Duration as TokioDuration};

const DEFAULT_PROMPTS: [&str; 3] = [
    "Explain the Kandil Code architecture in two sentences.",
    "Write a Rust function that reverses a linked list.",
    "Summarize the contents of Cargo.toml.",
];

pub struct CrossPlatformBenchmark {
    prompts: Vec<String>,
    http: Client,
}

impl Default for CrossPlatformBenchmark {
    fn default() -> Self {
        Self::new()
    }
}

impl CrossPlatformBenchmark {
    pub fn new() -> Self {
        let prompts = DEFAULT_PROMPTS
            .iter()
            .map(|p| p.to_string())
            .collect::<Vec<_>>();
        let http = Client::builder()
            .timeout(Duration::from_secs(4))
            .build()
            .unwrap_or_else(|_| Client::new());
        Self { prompts, http }
    }

    pub async fn run(&self, options: BenchmarkOptions) -> Result<BenchmarkReport> {
        let hardware = detect_hardware();
        let prompts = options
            .prompts
            .clone()
            .unwrap_or_else(|| self.prompts.clone());
        let (targets, mut warnings) = self.resolve_runtimes(&options).await?;

        if targets.is_empty() {
            return Err(anyhow!(
                "No runtimes available. Install a local runtime or configure a default provider."
            ));
        }

        let mut results = Vec::new();
        for target in targets {
            match self.benchmark_runtime(&target, &prompts).await {
                Ok(report) => results.push(report),
                Err(err) => warnings.push(format!(
                    "Runtime '{}' failed during benchmark: {}",
                    target.display_name, err
                )),
            }
        }

        if results.is_empty() {
            return Err(anyhow!(
                "Benchmark did not produce results. Last warning: {}",
                warnings
                    .last()
                    .cloned()
                    .unwrap_or_else(|| "unknown error".into())
            ));
        }

        Ok(BenchmarkReport {
            model: options.model.clone(),
            requested_runtime: options
                .normalized_runtime()
                .map(|runtime| runtime.to_string()),
            hardware,
            prompts,
            timestamp: Utc::now(),
            results,
            warnings,
        })
    }

    async fn resolve_runtimes(
        &self,
        options: &BenchmarkOptions,
    ) -> Result<(Vec<RuntimeTarget>, Vec<String>)> {
        let mut runtimes = Vec::new();
        let mut warnings = Vec::new();

        if let Some(runtime) = options.normalized_runtime() {
            match self.build_runtime(&runtime, options).await {
                Ok(Some(target)) => runtimes.push(target),
                Ok(None) => {
                    return Err(anyhow!(
                        "Requested runtime '{}' is not available on this host",
                        runtime
                    ))
                }
                Err(err) => {
                    return Err(anyhow!(
                        "Failed to initialize runtime '{}': {}",
                        runtime,
                        err
                    ))
                }
            }
            return Ok((runtimes, warnings));
        }

        if options.include_all_runtimes {
            for runtime in ["ollama", "lmstudio", "gpt4all", "foundry"] {
                match self.build_runtime(runtime, options).await {
                    Ok(Some(target)) => runtimes.push(target),
                    Ok(None) => warnings.push(format!("Runtime '{}' not detected", runtime)),
                    Err(err) => {
                        warnings.push(format!("Runtime '{}' unavailable: {}", runtime, err))
                    }
                }
            }

            if runtimes.is_empty() {
                warnings.push(
                    "No local runtimes detected; falling back to default provider.".to_string(),
                );
            }
        }

        if runtimes.is_empty() {
            match self.build_runtime("default", options).await {
                Ok(Some(target)) => runtimes.push(target),
                Ok(None) => {
                    return Err(anyhow!(
                        "Default provider '{}' could not be initialized",
                        options.default_provider
                    ))
                }
                Err(err) => {
                    return Err(anyhow!(
                        "Failed to initialize default provider '{}': {}",
                        options.default_provider,
                        err
                    ))
                }
            }
        }

        Ok((runtimes, warnings))
    }

    async fn build_runtime(
        &self,
        runtime_id: &str,
        options: &BenchmarkOptions,
    ) -> Result<Option<RuntimeTarget>> {
        match runtime_id {
            "default" => {
                let ai = KandilAI::new(options.default_provider.clone(), options.model.clone())?;
                Ok(Some(RuntimeTarget::new(
                    "default",
                    format!("{} (default)", options.default_provider),
                    options.default_provider.clone(),
                    ai,
                )))
            }
            "ollama" => {
                if !self.runtime_ready(&self.ollama_probe_url()).await {
                    return Ok(None);
                }
                let ai = KandilAI::new("ollama".into(), options.model.clone())?;
                Ok(Some(RuntimeTarget::new(
                    "ollama",
                    "Ollama".to_string(),
                    "ollama".into(),
                    ai,
                )))
            }
            "lmstudio" => {
                if !self.runtime_ready("http://localhost:1234/v1/models").await {
                    return Ok(None);
                }
                let ai = KandilAI::new("lmstudio".into(), options.model.clone())?;
                Ok(Some(RuntimeTarget::new(
                    "lmstudio",
                    "LM Studio".to_string(),
                    "lmstudio".into(),
                    ai,
                )))
            }
            "gpt4all" => {
                if !self.runtime_ready("http://localhost:4891/v1/models").await {
                    return Ok(None);
                }
                let ai = KandilAI::new("gpt4all".into(), options.model.clone())?;
                Ok(Some(RuntimeTarget::new(
                    "gpt4all",
                    "GPT4All".to_string(),
                    "gpt4all".into(),
                    ai,
                )))
            }
            "foundry" => {
                if !self.runtime_ready("http://localhost:5001/v1/models").await {
                    return Ok(None);
                }
                let ai = KandilAI::new("foundry".into(), options.model.clone())?;
                Ok(Some(RuntimeTarget::new(
                    "foundry",
                    "Foundry Local".to_string(),
                    "foundry".into(),
                    ai,
                )))
            }
            _ => Ok(None),
        }
    }

    async fn benchmark_runtime(
        &self,
        runtime: &RuntimeTarget,
        prompts: &[String],
    ) -> Result<RuntimeBenchmark> {
        let mut samples = Vec::new();
        let base_memory = memory_usage_mb();
        let mut total_latency_ms = 0u128;
        let mut total_tokens_per_sec = 0f64;

        for prompt in prompts {
            let start = Instant::now();
            let response = runtime.ai.chat(prompt).await?;
            let elapsed = start.elapsed();
            let latency_ms = elapsed.as_millis() as u64;
            let token_count = tokens_from_response(&response);
            total_latency_ms += elapsed.as_millis();

            let tokens_per_sec = if elapsed.as_secs_f64() > 0.0 {
                token_count as f64 / elapsed.as_secs_f64()
            } else {
                token_count as f64
            };
            total_tokens_per_sec += tokens_per_sec;

            samples.push(PromptSample {
                prompt: prompt.clone(),
                latency_ms,
                output_tokens: token_count,
            });
        }

        if samples.is_empty() {
            return Err(anyhow!("No prompts available for benchmarking"));
        }

        let avg_latency_ms = (total_latency_ms / samples.len() as u128)
            .try_into()
            .unwrap_or_default();
        let avg_tokens_per_sec = (total_tokens_per_sec / samples.len() as f64)
            .round()
            .max(1.0) as u32;
        let memory_peak_mb = memory_usage_mb().max(base_memory);

        Ok(RuntimeBenchmark {
            runtime: runtime.display_name.clone(),
            provider: runtime.provider.clone(),
            average_latency_ms: avg_latency_ms,
            average_tokens_per_sec: avg_tokens_per_sec,
            memory_peak_mb,
            battery_impact: None,
            samples,
        })
    }

    async fn runtime_ready(&self, url: &str) -> bool {
        match timeout(TokioDuration::from_secs(2), self.http.get(url).send()).await {
            Ok(Ok(resp)) => resp.status().is_success(),
            _ => false,
        }
    }

    fn ollama_probe_url(&self) -> String {
        format!("{}/api/tags", self.ollama_base_endpoint())
    }

    fn ollama_base_endpoint(&self) -> String {
        #[cfg(target_os = "windows")]
        {
            windows::preferred_ollama_endpoint()
        }
        #[cfg(not(target_os = "windows"))]
        {
            "http://localhost:11434".to_string()
        }
    }
}

fn tokens_from_response(response: &str) -> usize {
    response.split_whitespace().count().max(1)
}

fn memory_usage_mb() -> u64 {
    let mut sys = System::new_with_specifics(RefreshKind::new().with_memory());
    sys.refresh_memory();
    (sys.used_memory() / 1024) as u64
}

#[derive(Debug, Serialize, Clone)]
pub struct BenchmarkReport {
    pub model: String,
    pub requested_runtime: Option<String>,
    pub hardware: HardwareProfile,
    pub prompts: Vec<String>,
    pub timestamp: DateTime<Utc>,
    pub results: Vec<RuntimeBenchmark>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Serialize, Clone)]
pub struct RuntimeBenchmark {
    pub runtime: String,
    pub provider: String,
    pub average_latency_ms: u64,
    pub average_tokens_per_sec: u32,
    pub memory_peak_mb: u64,
    pub battery_impact: Option<f32>,
    pub samples: Vec<PromptSample>,
}

#[derive(Debug, Serialize, Clone)]
pub struct PromptSample {
    pub prompt: String,
    pub latency_ms: u64,
    pub output_tokens: usize,
}

pub struct BenchmarkOptions {
    pub model: String,
    pub default_provider: String,
    pub runtime: Option<String>,
    pub include_all_runtimes: bool,
    pub prompts: Option<Vec<String>>,
}

impl BenchmarkOptions {
    pub fn normalized_runtime(&self) -> Option<String> {
        self.runtime
            .as_ref()
            .map(|value| value.trim().to_lowercase())
            .filter(|value| !value.is_empty())
    }
}

struct RuntimeTarget {
    id: String,
    provider: String,
    display_name: String,
    ai: Arc<KandilAI>,
}

impl RuntimeTarget {
    fn new(
        id: impl Into<String>,
        display_name: impl Into<String>,
        provider: String,
        ai: KandilAI,
    ) -> Self {
        Self {
            id: id.into(),
            provider,
            display_name: display_name.into(),
            ai: Arc::new(ai),
        }
    }
}
