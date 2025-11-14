//! Real-time collaboration module
//! 
//! Module for handling collaborative editing and real-time interaction

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::sync::broadcast;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollaborationSession {
    pub id: String,
    pub name: String,
    pub participants: Vec<Participant>,
    pub documents: HashMap<String, CollaborativeDocument>,
    pub created_at: String,
    pub last_activity: String,
    pub permissions: SessionPermissions,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Participant {
    pub id: String,
    pub name: String,
    pub role: Role,
    pub joined_at: String,
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Role {
    Owner,
    Admin,
    Editor,
    Viewer,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollaborativeDocument {
    pub id: String,
    pub name: String,
    pub content: String,
    pub language: String,
    pub last_modified: String,
    pub version: u32,
    pub lock_holder: Option<String>, // User ID if document is locked
    pub cursors: HashMap<String, CursorPosition>,
    pub changes: Vec<DocumentChange>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CursorPosition {
    pub line: u32,
    pub column: u32,
    pub user_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentChange {
    pub id: String,
    pub user_id: String,
    pub operation: ChangeOperation,
    pub position: u32,
    pub content: String,
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChangeOperation {
    Insert,
    Delete,
    Update,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionPermissions {
    pub can_edit: bool,
    pub can_share: bool,
    pub can_delete: bool,
    pub can_manage_participants: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealTimeCollaboration {
    pub sessions: Arc<Mutex<HashMap<String, CollaborationSession>>>,
    #[serde(skip)]
    pub change_broadcasters: HashMap<String, broadcast::Sender<DocumentChange>>,
    pub active_users: HashMap<String, UserStatus>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserStatus {
    pub id: String,
    pub name: String,
    pub current_session: Option<String>,
    pub current_document: Option<String>,
    pub is_online: bool,
    pub last_seen: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollaborationEvent {
    pub session_id: String,
    pub user_id: String,
    pub event_type: EventType,
    pub details: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventType {
    UserJoined,
    UserLeft,
    DocumentChanged,
    CursorMoved,
    ChatMessage,
}

impl RealTimeCollaboration {
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(Mutex::new(HashMap::new())),
            change_broadcasters: HashMap::new(),
            active_users: HashMap::new(),
        }
    }

    pub fn create_session(&mut self, session_name: &str, owner_id: &str, owner_name: &str) -> Result<String> {
        let session_id = format!("sess-{}", uuid::Uuid::new_v4());
        
        let session = CollaborationSession {
            id: session_id.clone(),
            name: session_name.to_string(),
            participants: vec![Participant {
                id: owner_id.to_string(),
                name: owner_name.to_string(),
                role: Role::Owner,
                joined_at: chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
                is_active: true,
            }],
            documents: HashMap::new(),
            created_at: chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            last_activity: chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            permissions: SessionPermissions {
                can_edit: true,
                can_share: true,
                can_delete: true,
                can_manage_participants: true,
            },
        };
        
        // Create a broadcast channel for changes
        let (tx, _rx) = broadcast::channel::<DocumentChange>(100);
        self.change_broadcasters.insert(session_id.clone(), tx);
        
        let mut sessions = self.sessions.lock().unwrap();
        sessions.insert(session_id.clone(), session);
        
        Ok(session_id)
    }

    pub fn add_participant(&mut self, session_id: &str, user_id: &str, name: &str, role: Role) -> Result<()> {
        let mut sessions = self.sessions.lock().unwrap();
        
        if let Some(session) = sessions.get_mut(session_id) {
            session.participants.push(Participant {
                id: user_id.to_string(),
                name: name.to_string(),
                role,
                joined_at: chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
                is_active: true,
            });
            
            session.last_activity = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
            
            Ok(())
        } else {
            Err(anyhow::anyhow!("Session {} not found", session_id))
        }
    }

    pub fn add_document(&mut self, session_id: &str, doc_id: &str, name: &str, content: &str, language: &str) -> Result<()> {
        let mut sessions = self.sessions.lock().unwrap();
        
        if let Some(session) = sessions.get_mut(session_id) {
            let doc = CollaborativeDocument {
                id: doc_id.to_string(),
                name: name.to_string(),
                content: content.to_string(),
                language: language.to_string(),
                last_modified: chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
                version: 1,
                lock_holder: None,
                cursors: HashMap::new(),
                changes: vec![],
            };
            
            session.documents.insert(doc_id.to_string(), doc);
            
            session.last_activity = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
            
            Ok(())
        } else {
            Err(anyhow::anyhow!("Session {} not found", session_id))
        }
    }

    pub fn apply_change(&mut self, session_id: &str, user_id: &str, doc_id: &str, change: DocumentChange) -> Result<()> {
        let mut sessions = self.sessions.lock().unwrap();
        
        if let Some(session) = sessions.get_mut(session_id) {
            if let Some(doc) = session.documents.get_mut(doc_id) {
                // Apply the change based on operation
                match change.operation {
                    ChangeOperation::Insert => {
                        // In a real implementation, this would insert text at position
                        doc.content.insert_str(change.position as usize, &change.content);
                    },
                    ChangeOperation::Delete => {
                        // In a real implementation, this would delete text at position
                        if (change.position as usize) < doc.content.len() {
                            let end_idx = std::cmp::min(
                                (change.position as usize) + change.content.len(), 
                                doc.content.len()
                            );
                            doc.content.drain((change.position as usize)..end_idx);
                        }
                    },
                    ChangeOperation::Update => {
                        // In a real implementation, this would update text at position
                        doc.content.insert_str(change.position as usize, &change.content);
                    }
                }
                
                doc.version += 1;
                doc.last_modified = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
                doc.changes.push(change.clone());
                
                session.last_activity = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
                
                // Broadcast the change to other participants
                if let Some(tx) = self.change_broadcasters.get(session_id) {
                    let _ = tx.send(change);
                }
                
                Ok(())
            } else {
                Err(anyhow::anyhow!("Document {} not found in session {}", doc_id, session_id))
            }
        } else {
            Err(anyhow::anyhow!("Session {} not found", session_id))
        }
    }

    pub fn get_document_content(&self, session_id: &str, doc_id: &str) -> Result<String> {
        let sessions = self.sessions.lock().unwrap();
        
        if let Some(session) = sessions.get(session_id) {
            if let Some(doc) = session.documents.get(doc_id) {
                Ok(doc.content.clone())
            } else {
                Err(anyhow::anyhow!("Document {} not found in session {}", doc_id, session_id))
            }
        } else {
            Err(anyhow::anyhow!("Session {} not found", session_id))
        }
    }

    pub fn get_session(&self, session_id: &str) -> Result<CollaborationSession> {
        let sessions = self.sessions.lock().unwrap();
        sessions.get(session_id)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("Session {} not found", session_id))
    }
}