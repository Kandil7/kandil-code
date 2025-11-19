pub mod benchmarks;
pub use benchmarks::{PerformanceTester, PerformanceReport, TestStatus, run_performance_tests};

// Additional performance testing and monitoring utilities
use std::time::Duration;

/// Run all performance tests and benchmarks
pub async fn run_all_performance_tests() -> anyhow::Result<String> {
    run_performance_tests().await
}

/// Get system performance metrics
pub fn get_system_metrics() -> SystemMetrics {
    use sysinfo::{System, SystemExt, ProcessExt};
    
    let mut sys = System::new_all();
    sys.refresh_all();
    
    SystemMetrics {
        cpu_usage: sys.global_cpu_info().cpu_usage(),
        total_memory: sys.total_memory(),
        used_memory: sys.used_memory(),
        available_memory: sys.available_memory(),
    }
}

#[derive(Debug, Clone)]
pub struct SystemMetrics {
    pub cpu_usage: f32,
    pub total_memory: u64,
    pub used_memory: u64,
    pub available_memory: u64,
}

/// Measure execution time of a closure
pub fn measure_time<T, F>(operation: F) -> (T, Duration) 
where 
    F: FnOnce() -> T 
{
    let start = std::time::Instant::now();
    let result = operation();
    let duration = start.elapsed();
    (result, duration)
}

/// Performance profiling decorator for functions
pub struct Profiler {
    start_time: std::time::Instant,
    name: String,
}

impl Profiler {
    pub fn start(name: &str) -> Self {
        Self {
            start_time: std::time::Instant::now(),
            name: name.to_string(),
        }
    }
    
    pub fn stop(&mut self) -> Duration {
        let duration = self.start_time.elapsed();
        println!("⏱️  {} completed in {:?}", self.name, duration);
        duration
    }
}

impl Drop for Profiler {
    fn drop(&mut self) {
        if !std::thread::panicking() {
            let duration = self.start_time.elapsed();
            println!("⏱️  {} completed in {:?}", self.name, duration);
        }
    }
}