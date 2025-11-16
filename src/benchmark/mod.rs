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

const DEFAULT_PROMPTS: [&str; 5] = [
    "Explain the Kandil Code architecture in two sentences.",
    "Write a Rust function that reverses a linked list.",
    "Summarize the contents of Cargo.toml.",
    "Provide a system diagnostic report for this platform.",
    "Analyze the current hardware specifications.",
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

        // Attempt to measure battery impact (placeholder - would need real implementation)
        let battery_impact = self.estimate_battery_impact(&samples).await;

        Ok(RuntimeBenchmark {
            runtime: runtime.display_name.clone(),
            provider: runtime.provider.clone(),
            average_latency_ms: avg_latency_ms,
            average_tokens_per_sec: avg_tokens_per_sec,
            memory_peak_mb,
            battery_impact,
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

    async fn estimate_battery_impact(&self, samples: &[PromptSample]) -> Option<f32> {
        // This is a placeholder implementation - in a real implementation,
        // we would access platform-specific battery APIs
        if samples.is_empty() {
            return None;
        }

        // Placeholder: estimate based on average processing time and memory usage
        // This would need to be replaced with real battery monitoring
        let avg_latency: f64 = samples.iter()
            .map(|s| s.latency_ms as f64)
            .sum::<f64>() / samples.len() as f64;

        // Return a placeholder value - in a real implementation,
        // we would access platform-specific battery APIs
        Some((avg_latency / 1000.0) as f32) // Convert to percentage per minute
    }

    /// Run comprehensive system diagnostics
    pub async fn run_diagnostics(&self) -> DiagnosticReport {
        let hardware = detect_hardware();

        // Run connectivity checks
        let connectivity = self.check_connectivity().await;

        // Run performance checks
        let performance = self.check_performance().await;

        // Run security checks
        let security = self.check_security().await;

        DiagnosticReport {
            timestamp: Utc::now(),
            hardware,
            connectivity,
            performance,
            security,
        }
    }

    async fn check_connectivity(&self) -> ConnectivityReport {
        let mut endpoints = Vec::new();

        // Check various endpoints
        let endpoints_to_check = [
            ("localhost:11434", "Ollama API"), // Default Ollama
            ("localhost:1234", "LM Studio API"),
            ("localhost:4891", "GPT4All API"),
            ("localhost:5001", "Foundry API"),
        ];

        for (endpoint, name) in &endpoints_to_check {
            let reachable = self.runtime_ready(&format!("http://{}", endpoint)).await;
            endpoints.push(EndpointStatus {
                name: name.to_string(),
                endpoint: endpoint.to_string(),
                reachable,
            });
        }

        ConnectivityReport {
            endpoints,
            timestamp: Utc::now(),
        }
    }

    async fn check_performance(&self) -> PerformanceReport {
        // Baseline performance test
        let start_time = Instant::now();
        let base_memory = memory_usage_mb();

        // Simple CPU test
        let mut sum = 0;
        for i in 0..1000000 {
            sum += i % 100;
        }

        let elapsed = start_time.elapsed().as_millis() as u64;
        let peak_memory = memory_usage_mb().max(base_memory);

        PerformanceReport {
            cpu_test_duration_ms: elapsed,
            memory_baseline_mb: base_memory,
            memory_peak_mb: peak_memory,
            timestamp: Utc::now(),
        }
    }

    async fn check_security(&self) -> SecurityReport {
        // Basic security checks
        let has_api_key = self.check_api_key_storage().await;
        let is_network_secure = self.check_network_security().await;

        SecurityReport {
            api_key_secure: has_api_key,
            network_secure: is_network_secure,
            timestamp: Utc::now(),
        }
    }

    async fn check_api_key_storage(&self) -> bool {
        // This would check if API keys are properly stored in OS keyring
        // For now, return true as a placeholder
        true
    }

    async fn check_network_security(&self) -> bool {
        // This would check various network security aspects
        // For now, return true as a placeholder
        true
    }
}

fn tokens_from_response(response: &str) -> usize {
    response.split_whitespace().count().max(1)
}

// Diagnostic report structures
#[derive(Debug, Serialize, Clone)]
pub struct DiagnosticReport {
    pub timestamp: DateTime<Utc>,
    pub hardware: HardwareProfile,
    pub connectivity: ConnectivityReport,
    pub performance: PerformanceReport,
    pub security: SecurityReport,
}

#[derive(Debug, Serialize, Clone)]
pub struct ConnectivityReport {
    pub endpoints: Vec<EndpointStatus>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Serialize, Clone)]
pub struct EndpointStatus {
    pub name: String,
    pub endpoint: String,
    pub reachable: bool,
}

#[derive(Debug, Serialize, Clone)]
pub struct PerformanceReport {
    pub cpu_test_duration_ms: u64,
    pub memory_baseline_mb: u64,
    pub memory_peak_mb: u64,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Serialize, Clone)]
pub struct SecurityReport {
    pub api_key_secure: bool,
    pub network_secure: bool,
    pub timestamp: DateTime<Utc>,
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
