//! Project management system
//! 
//! Handles project creation, switching, and session management

use anyhow::Result;
use std::path::Path;
use crate::utils::db::{Database, Project};
use dirs::data_dir;
use uuid::Uuid;
use chrono::Utc;

pub struct ProjectManager {
    db: Database,
}

impl ProjectManager {
    pub fn new() -> Result<Self> {
        // Create data directory if it doesn't exist
        let data_path = data_dir()
            .unwrap_or_else(|| std::env::current_dir().unwrap())
            .join("kandil_code");
        
        std::fs::create_dir_all(&data_path)?;
        
        let db_path = data_path.join("kandil.db");
        let db = Database::new(db_path.to_str().unwrap())?;
        
        Ok(Self { db })
    }

    pub fn create_project(&self, name: &str, root_path: &str, ai_provider: &str, ai_model: &str) -> Result<Project> {
        let project = Project {
            id: Uuid::new_v4().to_string(),
            name: name.to_string(),
            root_path: root_path.to_string(),
            ai_provider: ai_provider.to_string(),
            ai_model: ai_model.to_string(),
            last_opened: None,
            memory_enabled: true,
            created_at: Utc::now(),
        };

        self.db.create_project(&project)?;
        
        // Create project directory if it doesn't exist
        std::fs::create_dir_all(root_path)?;
        
        Ok(project)
    }

    pub fn get_project(&self, id: &str) -> Result<Option<Project>> {
        self.db.get_project(id)
    }

    pub fn get_project_by_path(&self, path: &str) -> Result<Option<Project>> {
        self.db.get_project_by_path(path)
    }

    pub fn list_projects(&self) -> Result<Vec<Project>> {
        self.db.list_projects()
    }

    pub fn switch_project(&self, id: &str) -> Result<Project> {
        // Update the last opened timestamp
        self.db.update_project_last_opened(id)?;
        
        if let Some(project) = self.db.get_project(id)? {
            Ok(project)
        } else {
            Err(anyhow::anyhow!("Project with id {} not found", id))
        }
    }

    pub fn ensure_active_project(&self, project_path: Option<&str>) -> Result<Project> {
        if let Some(path) = project_path {
            // Check if there's already a project for this path
            if let Some(project) = self.db.get_project_by_path(path)? {
                self.db.update_project_last_opened(&project.id)?;
                return Ok(project);
            }
        }

        // If no project exists, create a default one
        let path = project_path
            .unwrap_or(&std::env::current_dir()?.to_string_lossy().to_string());
            
        self.create_project(
            &format!("Project_{}", Utc::now().format("%Y%m%d_%H%M%S")),
            path,
            "ollama",  // Default provider
            "llama3:70b"  // Default model
        )
    }

    pub fn save_project_memory(&self, project_id: &str, session_id: &str, role: &str, content: &str, tokens_used: Option<i64>) -> Result<()> {
        let memory = crate::utils::db::Memory {
            id: 0,  // Will be auto-generated
            project_id: project_id.to_string(),
            session_id: session_id.to_string(),
            role: role.to_string(),
            content: content.to_string(),
            timestamp: Utc::now(),
            tokens_used,
        };

        self.db.save_memory(&memory)?;
        
        // Add to sync queue for cloud sync
        let data = serde_json::to_string(&memory)?;
        self.db.add_to_sync_queue("INSERT", "memory", &memory.id.to_string(), &data)?;

        Ok(())
    }

    pub fn get_project_memory(&self, project_id: &str, limit: Option<i32>) -> Result<Vec<crate::utils::db::Memory>> {
        self.db.get_memory_for_project(project_id, limit)
    }
}