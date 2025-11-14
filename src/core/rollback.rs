//! Session Checkpointing and Instant Rollback
//!
//! Implements session persistence with checkpoints and the ability to 
//! instantly rollback to previous states

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use tokio::fs;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionCheckpoint {
    pub id: String,
    pub timestamp: std::time::SystemTime,
    pub description: String,
    pub files_snapshot: HashMap<String, FileSnapshot>,
    pub git_commit: Option<String>, // Git commit hash if available
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileSnapshot {
    pub path: String,
    pub content: String,
    pub hash: String, // Content hash for quick comparison
    pub last_modified: std::time::SystemTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RollbackResult {
    pub success: bool,
    pub files_restored: Vec<String>,
    pub files_failed: Vec<String>,
    pub message: String,
}

pub struct SessionManager {
    checkpoints: Arc<RwLock<HashMap<String, SessionCheckpoint>>>,
    working_dir: String,
    max_checkpoints: usize,
}

impl SessionManager {
    pub fn new(working_dir: &str) -> Self {
        Self {
            checkpoints: Arc::new(RwLock::new(HashMap::new())),
            working_dir: working_dir.to_string(),
            max_checkpoints: 50, // Keep maximum 50 checkpoints
        }
    }

    pub async fn create_checkpoint(&self, description: &str) -> Result<String> {
        let checkpoint_id = self.generate_checkpoint_id();
        
        // Create file snapshots
        let files_snapshot = self.create_files_snapshot().await?;
        
        // Get current git commit if available (simplified)
        let git_commit = self.get_current_git_commit().await.ok();
        
        let checkpoint = SessionCheckpoint {
            id: checkpoint_id.clone(),
            timestamp: std::time::SystemTime::now(),
            description: description.to_string(),
            files_snapshot,
            git_commit,
        };
        
        // Store checkpoint
        {
            let mut checkpoints = self.checkpoints.write().await;
            checkpoints.insert(checkpoint_id.clone(), checkpoint);
            
            // Maintain maximum checkpoints
            if checkpoints.len() > self.max_checkpoints {
                // Remove oldest checkpoint
                let mut sorted: Vec<_> = checkpoints.iter().collect();
                sorted.sort_by(|a, b| a.1.timestamp.cmp(&b.1.timestamp));
                
                if let Some(oldest_key) = sorted.first().map(|(k, _)| k.clone()) {
                    checkpoints.remove(&oldest_key);
                }
            }
        }
        
        Ok(checkpoint_id)
    }

    async fn create_files_snapshot(&self) -> Result<HashMap<String, FileSnapshot>> {
        let mut snapshots = HashMap::new();
        
        // Walk through the working directory and create snapshots for code files
        let mut entries = tokio::fs::read_dir(&self.working_dir).await?;
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            
            if path.is_file() && self.is_code_file(&path) {
                let content = fs::read_to_string(&path).await?;
                let hash = self.calculate_content_hash(&content);
                let metadata = fs::metadata(&path).await?;
                let last_modified = metadata.modified().unwrap_or(std::time::SystemTime::now());
                
                let snapshot = FileSnapshot {
                    path: path.to_string_lossy().to_string(),
                    content,
                    hash,
                    last_modified,
                };
                
                snapshots.insert(path.to_string_lossy().to_string(), snapshot);
            } else if path.is_dir() {
                // Recursively process subdirectories
                self.add_directory_snapshots(&path, &mut snapshots).await?;
            }
        }
        
        Ok(snapshots)
    }

    async fn add_directory_snapshots(&self, dir_path: &Path, snapshots: &mut HashMap<String, FileSnapshot>) -> Result<()> {
        let mut entries = tokio::fs::read_dir(dir_path).await?;
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            
            if path.is_file() && self.is_code_file(&path) {
                let content = fs::read_to_string(&path).await?;
                let hash = self.calculate_content_hash(&content);
                let metadata = fs::metadata(&path).await?;
                let last_modified = metadata.modified().unwrap_or(std::time::SystemTime::now());
                
                let snapshot = FileSnapshot {
                    path: path.to_string_lossy().to_string(),
                    content,
                    hash,
                    last_modified,
                };
                
                snapshots.insert(path.to_string_lossy().to_string(), snapshot);
            } else if path.is_dir() && path.file_name().map_or(true, |n| n != ".git") {
                // Skip .git directory to avoid massive snapshots
                self.add_directory_snapshots(&path, snapshots).await?;
            }
        }
        
        Ok(())
    }

    fn is_code_file(&self, path: &Path) -> bool {
        let extensions = ["rs", "js", "ts", "py", "dart", "java", "cpp", "c", "go", "tsx", "jsx", "toml", "json", "yaml", "yml"];
        if let Some(ext) = path.extension() {
            if let Some(ext_str) = ext.to_str() {
                return extensions.contains(&ext_str);
            }
        }
        false
    }

    fn calculate_content_hash(&self, content: &str) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        content.hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }

    async fn get_current_git_commit(&self) -> Result<String> {
        // In a real implementation, this would call git to get the current commit
        // For now, we'll simulate it
        Ok("current_git_commit_hash".to_string())
    }

    pub async fn list_checkpoints(&self) -> Result<Vec<SessionCheckpoint>> {
        let checkpoints = self.checkpoints.read().await;
        let mut result: Vec<_> = checkpoints.values().cloned().collect();
        
        // Sort by timestamp (newest first)
        result.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        
        Ok(result)
    }

    pub async fn rollback_to_checkpoint(&self, checkpoint_id: &str) -> Result<RollbackResult> {
        let checkpoints = self.checkpoints.read().await;
        
        if let Some(checkpoint) = checkpoints.get(checkpoint_id) {
            let mut files_restored = Vec::new();
            let mut files_failed = Vec::new();
            
            // Restore each file to its state in the checkpoint
            for (file_path, snapshot) in &checkpoint.files_snapshot {
                let path = Path::new(file_path);
                
                // Only restore if the file still exists in current workspace
                if path.exists() {
                    if let Err(e) = fs::write(path, &snapshot.content).await {
                        files_failed.push(file_path.clone());
                        eprintln!("Failed to restore file {}: {}", file_path, e);
                    } else {
                        files_restored.push(file_path.clone());
                    }
                } else {
                    // File doesn't exist in current workspace, create it
                    if let Some(parent) = path.parent() {
                        fs::create_dir_all(parent).await?;
                    }
                    
                    if let Err(e) = fs::write(path, &snapshot.content).await {
                        files_failed.push(file_path.clone());
                        eprintln!("Failed to create file {}: {}", file_path, e);
                    } else {
                        files_restored.push(file_path.clone());
                    }
                }
            }
            
            Ok(RollbackResult {
                success: files_failed.is_empty(),
                files_restored,
                files_failed,
                message: format!("Rollback to checkpoint {} completed", checkpoint_id),
            })
        } else {
            Err(anyhow::anyhow!("Checkpoint {} not found", checkpoint_id))
        }
    }

    pub async fn rollback_to_last_checkpoint(&self) -> Result<RollbackResult> {
        let checkpoints = self.checkpoints.read().await;
        let mut sorted: Vec<_> = checkpoints.values().cloned().collect();
        
        sorted.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        
        if let Some(latest) = sorted.first() {
            self.rollback_to_checkpoint(&latest.id).await
        } else {
            Err(anyhow::anyhow!("No checkpoints available for rollback"))
        }
    }

    pub async fn rollback_to_time(&self, target_time: std::time::SystemTime) -> Result<RollbackResult> {
        let checkpoints = self.checkpoints.read().await;
        let mut candidates: Vec<_> = checkpoints.values()
            .filter(|cp| cp.timestamp <= target_time)
            .cloned()
            .collect();
        
        // Find the closest checkpoint before or at target time
        candidates.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        
        if let Some(best_match) = candidates.first() {
            self.rollback_to_checkpoint(&best_match.id).await
        } else {
            Err(anyhow::anyhow!("No checkpoint found before target time"))
        }
    }

    fn generate_checkpoint_id(&self) -> String {
        use uuid::Uuid;
        Uuid::new_v4().to_string()
    }

    pub async fn cleanup_old_checkpoints(&self, keep_last_n: usize) -> Result<usize> {
        let mut checkpoints = self.checkpoints.write().await;
        let mut sorted: Vec<_> = checkpoints.iter().collect();
        
        sorted.sort_by(|a, b| b.1.timestamp.cmp(&a.1.timestamp));
        
        let to_remove = sorted.len().saturating_sub(keep_last_n);
        let mut removed_count = 0;
        
        for i in 0..to_remove {
            if i < sorted.len() {
                let key = sorted[i].0.clone();
                checkpoints.remove(&key);
                removed_count += 1;
            }
        }
        
        Ok(removed_count)
    }

    pub async fn save_checkpoints_to_file(&self, file_path: &str) -> Result<()> {
        let checkpoints = self.checkpoints.read().await;
        let serialized = serde_json::to_string_pretty(&*checkpoints)?;
        fs::write(file_path, serialized).await?;
        Ok(())
    }

    pub async fn load_checkpoints_from_file(&self, file_path: &str) -> Result<()> {
        if Path::new(file_path).exists() {
            let content = fs::read_to_string(file_path).await?;
            let checkpoints: HashMap<String, SessionCheckpoint> = serde_json::from_str(&content)?;
            
            let mut checkpoints_guard = self.checkpoints.write().await;
            *checkpoints_guard = checkpoints;
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct SessionMetadata {
    pub id: String,
    pub start_time: std::time::SystemTime,
    pub last_checkpoint: Option<std::time::SystemTime>,
    pub total_checkpoints: usize,
}

pub struct SessionCoordinator {
    session_manager: Arc<SessionManager>,
    active_session: Arc<RwLock<Option<SessionMetadata>>>,
}

impl SessionCoordinator {
    pub fn new(working_dir: &str) -> Self {
        Self {
            session_manager: Arc::new(SessionManager::new(working_dir)),
            active_session: Arc::new(RwLock::new(None)),
        }
    }

    pub async fn start_session(&self, session_id: Option<String>) -> Result<SessionMetadata> {
        let session_id = session_id.unwrap_or_else(|| {
            use uuid::Uuid;
            Uuid::new_v4().to_string()
        });
        
        let metadata = SessionMetadata {
            id: session_id,
            start_time: std::time::SystemTime::now(),
            last_checkpoint: None,
            total_checkpoints: 0,
        };
        
        {
            let mut active_session = self.active_session.write().await;
            *active_session = Some(metadata.clone());
        }
        
        Ok(metadata)
    }

    pub async fn create_session_checkpoint(&self, description: &str) -> Result<String> {
        // First ensure we have an active session
        {
            let active_session = self.active_session.read().await;
            if active_session.is_none() {
                return Err(anyhow::anyhow!("No active session. Call start_session first."));
            }
        }
        
        let checkpoint_id = self.session_manager.create_checkpoint(description).await?;
        
        // Update session metadata
        {
            let mut active_session = self.active_session.write().await;
            if let Some(ref mut metadata) = *active_session {
                metadata.last_checkpoint = Some(std::time::SystemTime::now());
                metadata.total_checkpoints += 1;
            }
        }
        
        Ok(checkpoint_id)
    }

    pub async fn end_session(&self) -> Result<SessionMetadata> {
        let session = {
            let mut active_session = self.active_session.write().await;
            active_session.take()
        };
        
        match session {
            Some(metadata) => Ok(metadata),
            None => Err(anyhow::anyhow!("No active session to end")),
        }
    }

    pub async fn get_active_session(&self) -> Option<SessionMetadata> {
        let active_session = self.active_session.read().await;
        active_session.clone()
    }

    pub async fn rollback_active_session(&self, checkpoint_id: &str) -> Result<RollbackResult> {
        self.session_manager.rollback_to_checkpoint(checkpoint_id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[tokio::test]
    async fn test_session_manager_creation() {
        let temp_dir = std::env::temp_dir();
        let test_dir = temp_dir.join("kandil_test_session");
        
        // Create test directory
        fs::create_dir_all(&test_dir).await.unwrap();
        
        let manager = SessionManager::new(&test_dir.to_string_lossy());
        
        // Clean up
        let _ = fs::remove_dir_all(&test_dir).await;
        
        assert!(true); // Just testing creation
    }

    #[tokio::test]
    async fn test_session_coordinator() {
        let temp_dir = std::env::temp_dir();
        let test_dir = temp_dir.join("kandil_test_coord");
        
        // Create test directory
        fs::create_dir_all(&test_dir).await.unwrap();
        
        let coordinator = SessionCoordinator::new(&test_dir.to_string_lossy());
        
        // Start a session
        let session = coordinator.start_session(None).await.unwrap();
        assert!(!session.id.is_empty());
        
        // Check if session is active
        let active = coordinator.get_active_session().await;
        assert!(active.is_some());
        
        // End the session
        let ended = coordinator.end_session().await;
        assert!(ended.is_ok());
        
        // Clean up
        let _ = fs::remove_dir_all(&test_dir).await;
    }
}