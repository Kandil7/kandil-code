//! Cost tracking for AI providers
//!
//! Tracks API usage and costs for different AI providers

use dashmap::DashMap;

#[derive(Debug, Clone)]
pub struct UsageRecord {
    tokens_input: u32,
    tokens_output: u32,
    cost_usd: f64,
    timestamp: std::time::SystemTime,
}

impl UsageRecord {
    pub fn tokens_input(&self) -> u32 {
        self.tokens_input
    }

    pub fn tokens_output(&self) -> u32 {
        self.tokens_output
    }

    pub fn cost_usd(&self) -> f64 {
        self.cost_usd
    }

    pub fn timestamp(&self) -> std::time::SystemTime {
        self.timestamp
    }
}

#[derive(Debug)]
pub struct CostTracker {
    pub anthropic: DashMap<String, UsageRecord>,
    pub openai: DashMap<String, UsageRecord>,
    pub qwen: DashMap<String, UsageRecord>,
    pub ollama: DashMap<String, UsageRecord>, // For tracking local usage
}

impl CostTracker {
    pub fn new() -> Self {
        Self {
            anthropic: DashMap::new(),
            openai: DashMap::new(),
            qwen: DashMap::new(),
            ollama: DashMap::new(),
        }
    }

    pub fn record_usage(
        &self,
        provider: &str,
        model: &str,
        input_tokens: u32,
        output_tokens: u32,
    ) -> f64 {
        // Calculate approximate cost based on provider pricing
        let cost = match provider {
            "openai" => self.calculate_openai_cost(model, input_tokens, output_tokens),
            "claude" => self.calculate_anthropic_cost(model, input_tokens, output_tokens),
            "qwen" => self.calculate_qwen_cost(model, input_tokens, output_tokens),
            "ollama" => 0.0, // Local models are free
            _ => 0.0,
        };

        let record = UsageRecord {
            tokens_input: input_tokens,
            tokens_output: output_tokens,
            cost_usd: cost,
            timestamp: std::time::SystemTime::now(),
        };

        match provider {
            "openai" => {
                self.openai.insert(model.to_string(), record);
            }
            "claude" => {
                self.anthropic.insert(model.to_string(), record);
            }
            "qwen" => {
                self.qwen.insert(model.to_string(), record);
            }
            "ollama" => {
                self.ollama.insert(model.to_string(), record);
            }
            _ => {}
        }

        cost
    }

    fn calculate_openai_cost(&self, model: &str, input_tokens: u32, output_tokens: u32) -> f64 {
        // Approximate costs based on OpenAI pricing (as of 2023)
        let input_cost_per_m: f64 = match model {
            m if m.contains("gpt-4") => 0.03,
            m if m.contains("gpt-3.5") => 0.001,
            _ => 0.001, // Default to cheaper model
        };

        let output_cost_per_m: f64 = match model {
            m if m.contains("gpt-4") => 0.06,
            m if m.contains("gpt-3.5") => 0.002,
            _ => 0.002, // Default to cheaper model
        };

        (input_tokens as f64 / 1_000_000.0) * input_cost_per_m
            + (output_tokens as f64 / 1_000_000.0) * output_cost_per_m
    }

    fn calculate_anthropic_cost(&self, model: &str, input_tokens: u32, output_tokens: u32) -> f64 {
        // Approximate costs based on Anthropic pricing (as of 2023)
        let input_cost_per_m: f64 = match model {
            m if m.contains("claude-3") => 0.015,
            _ => 0.008, // Default to cheaper model
        };

        let output_cost_per_m: f64 = match model {
            m if m.contains("claude-3") => 0.075,
            _ => 0.024, // Default to cheaper model
        };

        (input_tokens as f64 / 1_000_000.0) * input_cost_per_m
            + (output_tokens as f64 / 1_000_000.0) * output_cost_per_m
    }

    fn calculate_qwen_cost(&self, _model: &str, _input_tokens: u32, _output_tokens: u32) -> f64 {
        // Qwen costs vary by model and usage - using a rough estimate
        // In a real implementation, this would use actual Alibaba Cloud pricing
        0.001 // Placeholder cost
    }

    pub fn get_total_cost(&self, provider: &str) -> f64 {
        match provider {
            "openai" => self
                .openai
                .iter()
                .map(|record| record.value().cost_usd())
                .sum(),
            "claude" => self
                .anthropic
                .iter()
                .map(|record| record.value().cost_usd())
                .sum(),
            "qwen" => self
                .qwen
                .iter()
                .map(|record| record.value().cost_usd())
                .sum(),
            "ollama" => self
                .ollama
                .iter()
                .map(|record| record.value().cost_usd())
                .sum(),
            _ => 0.0,
        }
    }

    pub fn get_provider_stats(&self, provider: &str) -> (u64, u64, f64) {
        // Returns (total_input_tokens, total_output_tokens, total_cost)
        match provider {
            "openai" => {
                let mut total_input = 0;
                let mut total_output = 0;
                let mut total_cost = 0.0;
                for record in self.openai.iter() {
                    total_input += record.value().tokens_input() as u64;
                    total_output += record.value().tokens_output() as u64;
                    total_cost += record.value().cost_usd();
                }
                (total_input, total_output, total_cost)
            }
            "claude" => {
                let mut total_input = 0;
                let mut total_output = 0;
                let mut total_cost = 0.0;
                for record in self.anthropic.iter() {
                    total_input += record.value().tokens_input() as u64;
                    total_output += record.value().tokens_output() as u64;
                    total_cost += record.value().cost_usd();
                }
                (total_input, total_output, total_cost)
            }
            "qwen" => {
                let mut total_input = 0;
                let mut total_output = 0;
                let mut total_cost = 0.0;
                for record in self.qwen.iter() {
                    total_input += record.value().tokens_input() as u64;
                    total_output += record.value().tokens_output() as u64;
                    total_cost += record.value().cost_usd();
                }
                (total_input, total_output, total_cost)
            }
            "ollama" => {
                let mut total_input = 0;
                let mut total_output = 0;
                let mut total_cost = 0.0;
                for record in self.ollama.iter() {
                    total_input += record.value().tokens_input() as u64;
                    total_output += record.value().tokens_output() as u64;
                    total_cost += record.value().cost_usd();
                }
                (total_input, total_output, total_cost)
            }
            _ => (0, 0, 0.0),
        }
    }
}
