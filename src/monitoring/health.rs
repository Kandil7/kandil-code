//! Health check system for Kandil Code
//!
//! Contains modules for monitoring system health and performance.

use chrono::{DateTime, Utc};
use serde::Serialize;
use tokio::time::Instant;
use sysinfo::SystemExt;

use crate::common::traits::AIProvider;
use crate::core::hardware::HardwareProfile;

#[derive(Debug, Serialize)]
pub struct HealthReport {
    pub timestamp: DateTime<Utc>,
    pub model_name: String,
    pub results: Vec<TestResult>,
    pub memory_usage: MemoryUsage,
    pub gpu_info: Option<GpuUsage>,
    pub overall_status: HealthStatus,
}

#[derive(Debug, Serialize, Clone)]
pub struct TestResult {
    pub name: String,
    pub success: bool,
    pub latency_ms: u32,
    pub error: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct MemoryUsage {
    pub total_mb: u64,
    pub used_mb: u64,
    pub available_mb: u64,
}

#[derive(Debug, Serialize)]
pub struct GpuUsage {
    pub memory_used_mb: u64,
    pub memory_total_mb: u64,
    pub utilization_percent: f32,
}

#[derive(Debug, Serialize)]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Failing,
}

pub struct HealthChecker {
    model: std::sync::Arc<dyn AIProvider>,
    profile: HardwareProfile,
}

impl HealthChecker {
    pub fn new(model: std::sync::Arc<dyn AIProvider>, profile: HardwareProfile) -> Self {
        Self { model, profile }
    }

    pub async fn run_health_check(&self) -> HealthReport {
        let mut results = vec![];

        // Test 1: Basic inference
        let test_prompts = vec![
            ("Simple", "What is 2+2? Answer with number only."),
            ("Code", "Write a Rust function to reverse a string."),
        ];

        for (name, prompt) in test_prompts {
            let start = Instant::now();
            let result = self.model.complete(prompt).await;
            let latency = start.elapsed();

            results.push(TestResult {
                name: name.to_string(),
                success: result.is_ok(),
                latency_ms: latency.as_millis() as u32,
                error: result.err().map(|e| e.to_string()),
            });
        }

        // Memory usage measurement
        let memory_usage = self.measure_memory_usage().await;

        // GPU info if available
        let gpu_info = self.profile.gpu.as_ref().map(|_g| {
            // In a real implementation, we would measure actual GPU usage
            GpuUsage {
                memory_used_mb: 0, // Placeholder
                memory_total_mb: self.profile.gpu.as_ref().map_or(0, |gpu| gpu.memory_gb * 1024),
                utilization_percent: 0.0,
            }
        });

        HealthReport {
            timestamp: Utc::now(),
            model_name: self.model.name().await,
            results: results.clone(),
            memory_usage,
            gpu_info,
            overall_status: Self::calculate_status(&results),
        }
    }

    fn calculate_status(results: &[TestResult]) -> HealthStatus {
        let success_count = results.iter().filter(|r| r.success).count();
        let success_rate = success_count as f32 / results.len() as f32;

        match success_rate {
            1.0 => HealthStatus::Healthy,
            0.5..=0.99 => HealthStatus::Degraded,
            _ => HealthStatus::Failing,
        }
    }

    async fn measure_memory_usage(&self) -> MemoryUsage {
        // Use sysinfo to get current memory usage
        let mut sys = sysinfo::System::new();
        sys.refresh_memory();

        MemoryUsage {
            total_mb: sys.total_memory() / 1024 / 1024,
            used_mb: sys.used_memory() / 1024 / 1024,
            available_mb: sys.available_memory() / 1024 / 1024,
        }
    }
}

// Add the health check module to the monitoring mod.rs file