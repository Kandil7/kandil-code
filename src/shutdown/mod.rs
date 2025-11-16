//! Graceful shutdown mechanism for Kandil Code
//!
//! Provides coordinated shutdown of all system components.

use std::sync::Arc;
use tokio::sync::Notify;
use tokio::time::{timeout, Duration};

pub struct ShutdownManager {
    shutdown_notify: Arc<Notify>,
    shutdown_received: bool,
}

impl ShutdownManager {
    pub fn new() -> Self {
        Self {
            shutdown_notify: Arc::new(Notify::new()),
            shutdown_received: false,
        }
    }

    pub fn subscribe(&self) -> Arc<Notify> {
        Arc::clone(&self.shutdown_notify)
    }

    pub fn trigger_shutdown(&mut self) {
        self.shutdown_received = true;
        self.shutdown_notify.notify_waiters();
    }

    pub fn shutdown_received(&self) -> bool {
        self.shutdown_received
    }

    /// Wait for shutdown signal
    pub async fn wait_for_shutdown(&self) {
        self.shutdown_notify.notified().await;
    }

    /// Wait for shutdown with a timeout
    pub async fn wait_for_shutdown_with_timeout(&self, timeout_duration: Duration) -> bool {
        match timeout(timeout_duration, self.shutdown_notify.notified()).await {
            Ok(_) => true,   // Shutdown signal received
            Err(_) => false, // Timeout occurred
        }
    }
}

impl Default for ShutdownManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Shutdown handler for coordinated cleanup
pub struct ShutdownHandler {
    manager: Arc<tokio::sync::RwLock<ShutdownManager>>,
}

impl ShutdownHandler {
    pub fn new() -> Self {
        Self {
            manager: Arc::new(tokio::sync::RwLock::new(ShutdownManager::new())),
        }
    }

    /// Register shutdown signal handlers (Ctrl+C, termination signals)
    pub async fn setup_signal_handlers(&self) -> Result<(), Box<dyn std::error::Error>> {
        let manager_clone = Arc::clone(&self.manager);

        // Handle Ctrl+C
        tokio::spawn(async move {
            if tokio::signal::ctrl_c().await.is_ok() {
                let mut manager = manager_clone.write().await;
                manager.trigger_shutdown();
                println!("Received shutdown signal (Ctrl+C), initiating graceful shutdown...");
            }
        });

        Ok(())
    }

    /// Perform graceful shutdown with timeout
    pub async fn shutdown_gracefully(
        &self,
        timeout_duration: Duration,
    ) -> Result<(), Box<dyn std::error::Error>> {
        println!("Initiating graceful shutdown...");

        // First, trigger the shutdown
        {
            let mut manager = self.manager.write().await;
            manager.trigger_shutdown();
        }

        // Wait for shutdown with timeout
        let shutdown_completed = {
            let manager = self.manager.read().await;
            manager
                .wait_for_shutdown_with_timeout(timeout_duration)
                .await
        };

        if !shutdown_completed {
            eprintln!("Shutdown timeout exceeded, forcing termination...");
            return Err(std::io::Error::new(
                std::io::ErrorKind::TimedOut,
                "Graceful shutdown timed out",
            )
            .into());
        }

        println!("Graceful shutdown completed successfully.");
        Ok(())
    }

    /// Wait for shutdown signal
    pub async fn wait_for_shutdown(&self) {
        let manager = self.manager.read().await;
        manager.wait_for_shutdown().await;
    }

    /// Check if shutdown has been requested
    pub async fn shutdown_requested(&self) -> bool {
        let manager = self.manager.read().await;
        manager.shutdown_received()
    }
}

/// Trait for components that need to be shut down gracefully
#[async_trait::async_trait]
pub trait GracefulShutdown {
    async fn shutdown(&self) -> Result<(), Box<dyn std::error::Error>>;
}

// Example implementation for a component that needs graceful shutdown
pub struct ExampleComponent {
    name: String,
    shutdown_manager: Arc<tokio::sync::RwLock<ShutdownManager>>,
}

impl ExampleComponent {
    pub fn new(name: &str, shutdown_manager: Arc<tokio::sync::RwLock<ShutdownManager>>) -> Self {
        Self {
            name: name.to_string(),
            shutdown_manager,
        }
    }
}

#[async_trait::async_trait]
impl GracefulShutdown for ExampleComponent {
    async fn shutdown(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("Shutting down component: {}", self.name);

        // Perform cleanup operations
        // For example:
        // - Flush pending requests
        // - Close database connections
        // - Save state to disk
        // - Wait for ongoing operations to complete

        println!("Component {} shutdown complete", self.name);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio;

    #[tokio::test]
    async fn test_shutdown_manager() {
        let mut shutdown_manager = ShutdownManager::new();
        assert!(!shutdown_manager.shutdown_received());

        shutdown_manager.trigger_shutdown();
        assert!(shutdown_manager.shutdown_received());
    }

    #[tokio::test]
    async fn test_shutdown_handler() {
        let handler = ShutdownHandler::new();
        let timeout = Duration::from_millis(100);

        // Test timeout scenario
        let result = handler.shutdown_gracefully(timeout).await;
        assert!(result.is_err()); // Should timeout since we didn't trigger shutdown
    }
}
