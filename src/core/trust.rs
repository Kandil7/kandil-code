//! Progressive Permission System
//!
//! Implements trust levels (1-5) from paranoid to godmode
//! allowing users to control how much they trust the AI

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TrustLevel {
    Paranoid = 1,    // Ask before every command
    Cautious = 2,    // Auto-run reads, ask for writes
    Normal = 3,      // Auto-run writes, ask for git push
    Adventurous = 4, // Auto-run everything, notify only
    Godmode = 5,     // Silent execution (CI mode)
}

impl TrustLevel {
    pub fn from_int(level: u8) -> Result<Self> {
        match level {
            1 => Ok(TrustLevel::Paranoid),
            2 => Ok(TrustLevel::Cautious),
            3 => Ok(TrustLevel::Normal),
            4 => Ok(TrustLevel::Adventurous),
            5 => Ok(TrustLevel::Godmode),
            _ => Err(anyhow::anyhow!("Trust level must be between 1 and 5")),
        }
    }

    pub fn as_u8(&self) -> u8 {
        *self as u8
    }

    pub fn description(&self) -> &'static str {
        match self {
            TrustLevel::Paranoid => "Paranoid: Ask before every command",
            TrustLevel::Cautious => "Cautious: Auto-run reads, ask for writes",
            TrustLevel::Normal => "Normal: Auto-run writes, ask for git push",
            TrustLevel::Adventurous => "Adventurous: Auto-run everything, notify only",
            TrustLevel::Godmode => "Godmode: Silent execution (CI mode)",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionRequest {
    pub action: String,
    pub file_paths: Vec<String>,
    pub is_read: bool,
    pub is_write: bool,
    pub is_system: bool,  // System-level operations like git, network, etc.
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionResponse {
    pub allowed: bool,
    pub reason: String,
}

pub struct PermissionManager {
    current_level: Arc<RwLock<TrustLevel>>,
}

impl PermissionManager {
    pub fn new(initial_level: TrustLevel) -> Self {
        Self {
            current_level: Arc::new(RwLock::new(initial_level)),
        }
    }

    pub async fn set_trust_level(&self, level: TrustLevel) -> Result<()> {
        let mut current_level = self.current_level.write().await;
        *current_level = level;
        Ok(())
    }

    pub async fn get_trust_level(&self) -> TrustLevel {
        self.current_level.read().await.clone()
    }

    pub async fn check_permission(&self, request: &PermissionRequest) -> Result<PermissionResponse> {
        let level = self.get_trust_level().await;
        
        let should_ask_user = match level {
            TrustLevel::Paranoid => true,
            TrustLevel::Cautious => request.is_write || request.is_system,
            TrustLevel::Normal => request.is_system, // Only ask for system operations like git push
            TrustLevel::Adventurous => false, // Never ask, just notify
            TrustLevel::Godmode => false, // Silent execution
        };

        if should_ask_user {
            // In a real implementation, this would show an interactive prompt
            // For now, we'll simulate user response based on trust level
            Ok(PermissionResponse {
                allowed: false, // Simulating user needs to approve
                reason: format!("Action '{}' requires approval at trust level {:?}", 
                               request.action, level),
            })
        } else {
            // Auto-allowed based on trust level
            Ok(PermissionResponse {
                allowed: true,
                reason: format!("Auto-allowed at trust level {:?}", level),
            })
        }
    }

    pub async fn request_permission_interactive(&self, request: &PermissionRequest) -> Result<PermissionResponse> {
        // This would show an interactive prompt to the user
        // For simulation, we'll use the check_permission logic
        self.check_permission(request).await
    }

    pub async fn execute_with_permission_check<F, T>(&self, request: PermissionRequest, operation: F) -> Result<T>
    where
        F: FnOnce() -> Result<T>,
    {
        let permission = self.check_permission(&request).await?;
        
        if permission.allowed {
            operation()
        } else {
            Err(anyhow::anyhow!("Operation denied: {}", permission.reason))
        }
    }
}

#[derive(Debug)]
pub struct TrustConfig {
    pub level: TrustLevel,
    pub allowed_commands: Vec<String>,
    pub blocked_commands: Vec<String>,
    pub notify_on_allow: bool,
    pub notify_on_deny: bool,
}

impl Default for TrustConfig {
    fn default() -> Self {
        Self {
            level: TrustLevel::Normal,
            allowed_commands: vec![
                "read_file".to_string(),
                "list_directory".to_string(),
                "search_code".to_string(),
            ],
            blocked_commands: vec![
                "delete_file".to_string(),
                "format_disk".to_string(), // Just an example of a dangerous command
            ],
            notify_on_allow: false,
            notify_on_deny: true,
        }
    }
}

pub struct TrustSystem {
    manager: Arc<PermissionManager>,
    config: Arc<RwLock<TrustConfig>>,
}

impl TrustSystem {
    pub fn new(initial_level: TrustLevel) -> Self {
        Self {
            manager: Arc::new(PermissionManager::new(initial_level)),
            config: Arc::new(RwLock::new(TrustConfig::default())),
        }
    }

    pub async fn initialize_trust_level(&self) -> Result<TrustLevel> {
        // In a real implementation, this might prompt the user to select their trust level
        // For now, we'll return the current level
        Ok(self.manager.get_trust_level().await)
    }

    pub async fn prompt_for_trust_level(&self) -> Result<TrustLevel> {
        println!("🤖 Trust Level? [1-5]");
        println!("1. 🟢 Paranoid: Ask before every command");
        println!("2. 🟡 Cautious: Auto-run reads, ask for writes");
        println!("3. 🟠 Normal: Auto-run writes, ask for git push");
        println!("4. 🔴 Adventurous: Auto-run everything, notify only");
        println!("5. ⚫ Godmode: Silent execution (CI mode)");
        
        // For simulation purposes, returning Normal as default
        Ok(TrustLevel::Normal)
    }

    pub async fn execute_safe_operation<F, T>(&self, action: &str, operation: F) -> Result<T>
    where
        F: FnOnce() -> Result<T>,
    {
        let request = PermissionRequest {
            action: action.to_string(),
            file_paths: vec![],
            is_read: action.contains("read") || action.contains("list") || action.contains("get"),
            is_write: action.contains("write") || action.contains("create") || action.contains("update"),
            is_system: action.contains("git") || action.contains("system") || action.contains("network"),
        };

        self.manager.execute_with_permission_check(request, operation).await
    }

    pub async fn check_command_allowed(&self, command: &str) -> bool {
        let config = self.config.read().await;
        
        // Check if command is explicitly blocked
        if config.blocked_commands.iter().any(|blocked| command.contains(blocked)) {
            return false;
        }
        
        // Check if command is explicitly allowed
        if config.allowed_commands.iter().any(|allowed| command.contains(allowed)) {
            return true;
        }
        
        // For this implementation, default to allowing commands that aren't explicitly blocked
        true
    }

    pub async fn log_permission_decision(&self, request: &PermissionRequest, response: &PermissionResponse) {
        let config = self.config.read().await;
        
        if response.allowed && config.notify_on_allow {
            println!("✅ Allowed: {} (Trust: {:?})", request.action, self.manager.get_trust_level().await);
        } else if !response.allowed && config.notify_on_deny {
            println!("❌ Denied: {} - {}", request.action, response.reason);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_trust_level_from_int() {
        assert_eq!(TrustLevel::from_int(1).unwrap(), TrustLevel::Paranoid);
        assert_eq!(TrustLevel::from_int(3).unwrap(), TrustLevel::Normal);
        assert!(TrustLevel::from_int(6).is_err());
    }

    #[tokio::test]
    async fn test_permission_manager() {
        let manager = PermissionManager::new(TrustLevel::Normal);
        
        let request = PermissionRequest {
            action: "read_file".to_string(),
            file_paths: vec!["test.txt".to_string()],
            is_read: true,
            is_write: false,
            is_system: false,
        };
        
        let response = manager.check_permission(&request).await.unwrap();
        assert!(response.allowed); // Read operations should be allowed at Normal level
    }

    #[tokio::test]
    async fn test_trust_system() {
        let trust_system = TrustSystem::new(TrustLevel::Normal);
        
        assert!(trust_system.check_command_allowed("read_file").await);
        assert!(trust_system.check_command_allowed("list_directory").await);
        assert!(!trust_system.check_command_allowed("delete_file").await);
    }
}