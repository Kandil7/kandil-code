use crate::core::adapters::ai::KandilAI;
use anyhow::Result;
use serde::Serialize;
use std::sync::Arc;
use std::time::Instant;

#[derive(Debug, Serialize, Clone)]
pub struct SampleResult {
    pub prompt: &'static str,
    pub latency_ms: u64,
    pub output_tokens: usize,
}

#[derive(Debug, Serialize, Clone)]
pub struct BenchmarkReport {
    pub model: String,
    pub provider: String,
    pub samples: Vec<SampleResult>,
    pub average_latency_ms: u64,
    pub total_tokens: usize,
}

pub struct CrossPlatformBenchmark {
    prompts: Vec<&'static str>,
}

impl Default for CrossPlatformBenchmark {
    fn default() -> Self {
        Self::new()
    }
}

impl CrossPlatformBenchmark {
    pub fn new() -> Self {
        Self {
            prompts: vec![
                "Explain the Kandil Code architecture in two sentences.",
                "Write a Rust function that reverses a linked list.",
                "Summarize the contents of Cargo.toml.",
            ],
        }
    }

    pub async fn run(&self, ai: Arc<KandilAI>) -> Result<BenchmarkReport> {
        let mut samples = Vec::new();
        let mut total_latency = 0u128;
        let mut total_tokens = 0usize;

        for prompt in &self.prompts {
            let start = Instant::now();
            let response = ai.chat(prompt).await?;
            let elapsed = start.elapsed();
            let tokens = response.split_whitespace().count();

            total_latency += elapsed.as_millis();
            total_tokens += tokens;
            samples.push(SampleResult {
                prompt,
                latency_ms: elapsed.as_millis() as u64,
                output_tokens: tokens,
            });
        }

        let average_latency_ms = if samples.is_empty() {
            0
        } else {
            (total_latency / samples.len() as u128) as u64
        };

        Ok(BenchmarkReport {
            model: ai.model_name().to_string(),
            provider: ai.provider_name().to_string(),
            samples,
            average_latency_ms,
            total_tokens,
        })
    }
}
