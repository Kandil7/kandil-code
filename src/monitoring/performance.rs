//! Performance monitoring and telemetry for Kandil Code
//!
//! Contains modules for tracking and monitoring system performance and usage telemetry.

use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use dashmap::DashMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub total_requests: u64,
    pub total_tokens: u64,
    pub total_latency: Duration,
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub fallback_triggers: u64,
    pub errors: u64,
    pub avg_tokens_per_request: f64,
    pub avg_latency: Duration,
    pub tokens_per_second: f64,
    pub model_usage: std::collections::HashMap<String, u64>, // Track usage by model
    pub request_types: std::collections::HashMap<String, u64>, // Track different request types
    pub hardware_stats: Option<HardwareStats>, // Include hardware stats
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareStats {
    pub ram_used_mb: u64,
    pub cpu_usage_percent: f64,
    pub disk_used_percent: f64,
}

impl Default for PerformanceMetrics {
    fn default() -> Self {
        Self {
            total_requests: 0,
            total_tokens: 0,
            total_latency: Duration::from_millis(0),
            cache_hits: 0,
            cache_misses: 0,
            fallback_triggers: 0,
            errors: 0,
            avg_tokens_per_request: 0.0,
            avg_latency: Duration::from_millis(0),
            tokens_per_second: 0.0,
            model_usage: std::collections::HashMap::new(),
            request_types: std::collections::HashMap::new(),
            hardware_stats: None,
        }
    }
}

pub struct PerformanceMonitor {
    metrics: Arc<RwLock<PerformanceMetrics>>,
    start_time: Instant,
    request_times: Arc<DashMap<u64, Instant>>,
    request_counter: Arc<RwLock<u64>>,
}

impl PerformanceMonitor {
    pub fn new() -> Self {
        Self {
            metrics: Arc::new(RwLock::new(PerformanceMetrics::default())),
            start_time: Instant::now(),
            request_times: Arc::new(DashMap::new()),
            request_counter: Arc::new(RwLock::new(0)),
        }
    }

    pub async fn record_request(&self, request_id: u64) {
        let mut counter = self.request_counter.write().await;
        *counter += 1;
        self.request_times.insert(request_id, Instant::now());
    }

    pub async fn record_response(&self, request_id: u64, tokens: usize, is_error: bool, model_name: Option<&str>) -> Result<(), Box<dyn std::error::Error>> {
        // Calculate latency for this request
        let latency = if let Some(start_time) = self.request_times.remove(&request_id) {
            start_time.1.elapsed()
        } else {
            Duration::from_millis(0) // Fallback if request time not found
        };

        // Update metrics
        let mut metrics = self.metrics.write().await;

        metrics.total_requests += 1;
        metrics.total_tokens += tokens as u64;
        metrics.total_latency += latency;

        if is_error {
            metrics.errors += 1;
        }

        // Track model usage if provided
        if let Some(model) = model_name {
            *metrics.model_usage.entry(model.to_string()).or_insert(0) += 1;
        }

        // Calculate derived metrics
        if metrics.total_requests > 0 {
            metrics.avg_tokens_per_request = metrics.total_tokens as f64 / metrics.total_requests as f64;
            let total_millis = metrics.total_latency.as_millis() as f64;
            metrics.avg_latency = Duration::from_millis((total_millis / metrics.total_requests as f64) as u64);

            // Calculate tokens per second (if we have a meaningful time period)
            let runtime = self.start_time.elapsed();
            if runtime.as_secs() > 0 {
                metrics.tokens_per_second = metrics.total_tokens as f64 / runtime.as_secs() as f64;
            }
        }

        Ok(())
    }

    pub async fn record_cache_hit(&self) {
        let mut metrics = self.metrics.write().await;
        metrics.cache_hits += 1;
    }

    pub async fn record_cache_miss(&self) {
        let mut metrics = self.metrics.write().await;
        metrics.cache_misses += 1;
    }

    pub async fn record_fallback(&self) {
        let mut metrics = self.metrics.write().await;
        metrics.fallback_triggers += 1;
    }

    pub async fn record_request_type(&self, req_type: &str) {
        let mut metrics = self.metrics.write().await;
        *metrics.request_types.entry(req_type.to_string()).or_insert(0) += 1;
    }

    pub async fn update_hardware_stats(&self) {
        if let Ok(mut sys) = std::sync::Mutex::new(sysinfo::System::new()).lock() {
            sys.refresh_memory();
            sys.refresh_cpu();

            let mut metrics = self.metrics.write().await;
            metrics.hardware_stats = Some(HardwareStats {
                ram_used_mb: sys.used_memory() / (1024 * 1024),
                cpu_usage_percent: sys.global_cpu_info().cpu_usage() as f64,
                disk_used_percent: 0.0, // Would need additional disk monitoring
            });
        }
    }

    pub async fn get_metrics(&self) -> PerformanceMetrics {
        // Update hardware stats periodically
        self.update_hardware_stats().await;
        self.metrics.read().await.clone()
    }

    pub async fn reset_metrics(&self) {
        let mut metrics = self.metrics.write().await;
        *metrics = PerformanceMetrics::default();
        self.start_time = Instant::now();
    }

    pub async fn print_summary(&self) {
        let metrics = self.get_metrics().await;

        println!("📊 Performance & Telemetry Summary:");
        println!("  Requests: {}", metrics.total_requests);
        println!("  Tokens: {}", metrics.total_tokens);
        println!("  Avg Latency: {:?}", metrics.avg_latency);
        println!("  Avg Tokens/Request: {:.1}", metrics.avg_tokens_per_request);
        println!("  Tokens/Second: {:.1}", metrics.tokens_per_second);

        let total_cache_ops = metrics.cache_hits + metrics.cache_misses;
        if total_cache_ops > 0 {
            let cache_hit_rate = metrics.cache_hits as f64 / total_cache_ops as f64;
            println!("  Cache Hit Rate: {:.1}%", cache_hit_rate * 100.0);
        }

        println!("  Fallbacks: {}", metrics.fallback_triggers);
        println!("  Errors: {}", metrics.errors);

        // Print model usage stats
        if !metrics.model_usage.is_empty() {
            println!("  Model Usage:");
            for (model, count) in &metrics.model_usage {
                println!("    {}: {}", model, count);
            }
        }

        // Print hardware stats if available
        if let Some(hw) = &metrics.hardware_stats {
            println!("  Hardware Stats:");
            println!("    RAM Used: {} MB", hw.ram_used_mb);
            println!("    CPU Usage: {:.1}%", hw.cpu_usage_percent);
        }
    }

    pub async fn get_uptime(&self) -> Duration {
        self.start_time.elapsed()
    }

    pub async fn export_telemetry_data(&self) -> Result<String, serde_json::Error> {
        let metrics = self.get_metrics().await;
        serde_json::to_string(&metrics)
    }
}

impl Default for PerformanceMonitor {
    fn default() -> Self {
        Self::new()
    }
}

// Create a global performance monitor instance (in a real app, you might want to handle this differently)
lazy_static::lazy_static! {
    pub static ref GLOBAL_PERFORMANCE_MONITOR: PerformanceMonitor = PerformanceMonitor::new();
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio;

    #[tokio::test]
    async fn test_performance_monitor() {
        let monitor = PerformanceMonitor::new();
        
        let request_id = 1;
        monitor.record_request(request_id).await;
        
        // Simulate a small delay
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        
        monitor.record_response(request_id, 100, false).await.unwrap();
        monitor.record_cache_hit().await;
        
        let metrics = monitor.get_metrics().await;
        
        assert_eq!(metrics.total_requests, 1);
        assert_eq!(metrics.total_tokens, 100);
        assert_eq!(metrics.cache_hits, 1);
    }
}