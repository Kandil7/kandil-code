use axum::{
    extract::{Query, State, WebSocketUpgrade},
    http::StatusCode,
    response::Html,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::sync::broadcast;

// Data structures for the web dashboard
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CliSessionState {
    pub command_history: Vec<CommandEntry>,
    pub current_context: String,
    pub active_files: Vec<String>,
    pub system_stats: SystemStats,
    pub ai_interactions: Vec<AiInteraction>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandEntry {
    pub timestamp: String,
    pub command: String,
    pub result: String,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemStats {
    pub cpu_usage: f64,
    pub memory_usage: f64,
    pub disk_space: String,
    pub uptime: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiInteraction {
    pub timestamp: String,
    pub query: String,
    pub response: String,
    pub model_used: String,
}

// State for the web server
pub struct WebAppState {
    pub session_state: tokio::sync::RwLock<CliSessionState>,
    pub tx: broadcast::Sender<CliSessionState>,
}

impl Default for CliSessionState {
    fn default() -> Self {
        Self {
            command_history: Vec::new(),
            current_context: "default".to_string(),
            active_files: Vec::new(),
            system_stats: SystemStats {
                cpu_usage: 0.0,
                memory_usage: 0.0,
                disk_space: "N/A".to_string(),
                uptime: "0s".to_string(),
            },
            ai_interactions: Vec::new(),
        }
    }
}

impl Default for SystemStats {
    fn default() -> Self {
        Self {
            cpu_usage: 0.0,
            memory_usage: 0.0,
            disk_space: "N/A".to_string(),
            uptime: "0s".to_string(),
        }
    }
}

pub struct WebCompanionDashboard {
    port: u16,
    state: WebAppState,
}

impl WebCompanionDashboard {
    pub fn new(port: u16) -> Self {
        let (tx, _) = broadcast::channel(100);
        let state = WebAppState {
            session_state: tokio::sync::RwLock::new(CliSessionState::default()),
            tx,
        };
        
        Self { port, state }
    }

    pub async fn run(&self) -> Result<(), hyper::Error> {
        let app_state = self.state.clone();
        let app = Router::new()
            .route("/", get(root))
            .route("/dashboard", get(dashboard))
            .route("/api/session", get(get_session_state))
            .route("/api/session/update", post(update_session_state))
            .route("/api/ws", get(websocket_handler))
            .route("/api/stats", get(get_system_stats))
            .route("/api/history", get(get_command_history))
            .route("/api/ai", get(get_ai_interactions))
            .with_state(app_state);

        let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", self.port))
            .await
            .unwrap();
        
        axum::serve(listener, app).await
    }
    
    /// Update the session state from CLI events
    pub async fn update_session(&self, update_fn: impl FnOnce(&mut CliSessionState)) {
        let mut state = self.state.session_state.write().await;
        update_fn(&mut state);
        
        // Broadcast the update
        let _ = self.state.tx.send(state.clone());
    }
    
    /// Add a command to the command history
    pub async fn add_command(&self, command: &str, result: &str, duration_ms: u64) {
        self.update_session(|state| {
            state.command_history.push(CommandEntry {
                timestamp: chrono::Utc::now().to_rfc3339(),
                command: command.to_string(),
                result: result.to_string(),
                duration_ms,
            });
            
            // Keep only the last 100 commands
            if state.command_history.len() > 100 {
                state.command_history.drain(0..state.command_history.len()-100);
            }
        }).await;
    }
    
    /// Add an AI interaction
    pub async fn add_ai_interaction(&self, query: &str, response: &str, model: &str) {
        self.update_session(|state| {
            state.ai_interactions.push(AiInteraction {
                timestamp: chrono::Utc::now().to_rfc3339(),
                query: query.to_string(),
                response: response.to_string(),
                model_used: model.to_string(),
            });
            
            // Keep only the last 50 interactions
            if state.ai_interactions.len() > 50 {
                state.ai_interactions.drain(0..state.ai_interactions.len()-50);
            }
        }).await;
    }
}

// Request handlers for the web API

async fn root(State(_state): State<WebAppState>) -> Html<String> {
    Html(include_str!("../web/static/index.html").to_string())
}

async fn dashboard(State(_state): State<WebAppState>) -> Html<String> {
    Html("<h1>Dashboard</h1>".to_string())
}

async fn get_session_state(State(state): State<WebAppState>) -> Json<CliSessionState> {
    let session_state = state.session_state.read().await;
    Json(session_state.clone())
}

async fn get_system_stats(State(_state): State<WebAppState>) -> Json<SystemStats> {
    // In a real implementation, this would gather actual system stats
    Json(SystemStats {
        cpu_usage: 25.0, // Placeholder value
        memory_usage: 45.0, // Placeholder value
        disk_space: "25GB / 500GB".to_string(),
        uptime: "2h 15m".to_string(),
    })
}

async fn get_command_history(
    State(state): State<WebAppState>,
    Query(params): Query<HashMap<String, String>>,
) -> Json<Vec<CommandEntry>> {
    let session_state = state.session_state.read().await;
    let limit = params
        .get("limit")
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(10);
    
    let mut history = session_state.command_history.clone();
    history.truncate(limit);
    Json(history)
}

async fn get_ai_interactions(
    State(state): State<WebAppState>,
    Query(params): Query<HashMap<String, String>>,
) -> Json<Vec<AiInteraction>> {
    let session_state = state.session_state.read().await;
    let limit = params
        .get("limit")
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(10);
    
    let mut interactions = session_state.ai_interactions.clone();
    interactions.truncate(limit);
    Json(interactions)
}

async fn update_session_state(
    State(state): State<WebAppState>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<CliSessionState>, StatusCode> {
    // In a real implementation, this would allow external updates to session state
    let mut session_state = state.session_state.write().await;
    // This is a basic implementation - in a real system you'd process the payload
    // to update specific parts of the session state
    
    // For example, if payload contains context update
    if let Some(context) = payload.get("context").and_then(|v| v.as_str()) {
        session_state.current_context = context.to_string();
    }
    
    Ok(Json(session_state.clone()))
}

async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<WebAppState>,
) -> impl IntoResponse {
    ws.on_upgrade(|websocket| handle_websocket(websocket, state))
}

async fn handle_websocket(
    mut websocket: axum::extract::ws::WebSocket,
    state: WebAppState,
) {
    let mut rx = state.tx.subscribe();
    
    loop {
        tokio::select! {
            // Receive message from client
            msg = websocket.recv() => {
                if msg.is_none() {
                    break;
                }
            }
            // Receive update from CLI
            result = rx.recv() => {
                match result {
                    Ok(session_state) => {
                        if websocket
                            .send(axum::extract::ws::Message::Text(
                                serde_json::to_string(&session_state).unwrap(),
                            ))
                            .await
                            .is_err()
                        {
                            // Client disconnected
                            break;
                        }
                    }
                    Err(broadcast::error::RecvError::Closed) => {
                        // Channel closed
                        break;
                    }
                    Err(broadcast::error::RecvError::Lagged(_)) => {
                        // Lagged behind, skip this message
                        continue;
                    }
                }
            }
        }
    }
}

// Utility functions
pub async fn launch_web_dashboard(port: u16) -> WebCompanionDashboard {
    WebCompanionDashboard::new(port)
}