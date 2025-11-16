use crate::enhanced_ui::adaptive::AdaptiveUI;
use crate::enhanced_ui::ide_sync::IdeSyncBridge;
use crate::web::dashboard::{launch_web_dashboard, WebCompanionDashboard};
use axum::Router;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

// Main web interface for the Kandil Code platform
pub struct WebInterface {
    pub dashboard: Option<Arc<RwLock<WebCompanionDashboard>>>,
    pub server_addr: String,
    pub is_running: bool,
}

impl WebInterface {
    pub fn new(addr: &str) -> Self {
        Self {
            dashboard: None,
            server_addr: addr.to_string(),
            is_running: false,
        }
    }

    pub async fn start_dashboard(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Extract port from address (assuming format like "127.0.0.1:7878")
        let port = self.server_addr
            .split(':')
            .last()
            .unwrap_or("7878")
            .parse()
            .unwrap_or(7878);

        let dashboard = launch_web_dashboard(port).await;
        self.dashboard = Some(Arc::new(RwLock::new(dashboard)));
        self.is_running = true;

        // Start the server in a background task
        let dashboard_arc = self.dashboard.as_ref().unwrap().clone();
        tokio::spawn(async move {
            let dashboard = dashboard_arc.read().await;
            if let Err(e) = dashboard.run().await {
                eprintln!("Web dashboard error: {}", e);
            }
        });

        Ok(())
    }

    pub async fn update_dashboard_session<F>(&self, update_fn: F) -> Result<(), Box<dyn std::error::Error>>
    where
        F: FnOnce(&mut crate::web::dashboard::CliSessionState) + Send + 'static,
    {
        if let Some(dashboard) = &self.dashboard {
            let db = dashboard.read().await;
            db.update_session(update_fn).await;
        }
        Ok(())
    }

    pub async fn add_command_to_dashboard(
        &self,
        command: &str,
        result: &str,
        duration_ms: u64,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(dashboard) = &self.dashboard {
            let db = dashboard.read().await;
            db.add_command(command, result, duration_ms).await;
        }
        Ok(())
    }

    pub async fn add_ai_interaction_to_dashboard(
        &self,
        query: &str,
        response: &str,
        model: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(dashboard) = &self.dashboard {
            let db = dashboard.read().await;
            db.add_ai_interaction(query, response, model).await;
        }
        Ok(())
    }
}

// Start the web companion server
pub async fn start(address: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("🌐 Starting Kandil Code Web Companion at http://{}", address);

    let mut web_interface = WebInterface::new(address);
    web_interface.start_dashboard().await?;
    
    // Keep the server running
    tokio::signal::ctrl_c().await.expect("Failed to listen for ctrl+c");
    println!(" shutting down gracefully...");
    
    Ok(())
}

// Integration with the main CLI system
pub async fn update_web_session(
    web_interface: Option<&WebInterface>,
    update_fn: impl FnOnce(&mut crate::web::dashboard::CliSessionState) + Send + 'static,
) {
    if let Some(web) = web_interface {
        let _ = web.update_dashboard_session(update_fn).await;
    }
}

pub async fn log_command_to_web(
    web_interface: Option<&WebInterface>,
    command: &str,
    result: &str,
    duration_ms: u64,
) {
    if let Some(web) = web_interface {
        let _ = web.add_command_to_dashboard(command, result, duration_ms).await;
    }
}

pub async fn log_ai_interaction_to_web(
    web_interface: Option<&WebInterface>,
    query: &str,
    response: &str,
    model: &str,
) {
    if let Some(web) = web_interface {
        let _ = web.add_ai_interaction_to_dashboard(query, response, model).await;
    }
}