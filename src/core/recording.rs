use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionSnapshot {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub command: String,
    pub output: String,
    pub context: String, // Serialized context representation
    pub duration: Duration,
    pub state_hash: String, // Hash of the system state
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordingSession {
    pub id: String,
    pub start_time: DateTime<Utc>,
    pub snapshots: VecDeque<SessionSnapshot>,
    pub max_snapshots: usize,
    pub is_recording: bool,
    pub metadata: RecordingMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordingMetadata {
    pub project_path: PathBuf,
    pub user: String,
    pub description: String,
    pub tags: Vec<String>,
}

pub struct RecordingManager {
    sessions: Arc<Mutex<Vec<RecordingSession>>>,
    current_session: Arc<Mutex<Option<String>>>,
    storage_path: PathBuf,
}

impl RecordingManager {
    pub fn new(storage_path: PathBuf) -> Result<Self> {
        std::fs::create_dir_all(&storage_path)?;
        
        Ok(Self {
            sessions: Arc::new(Mutex::new(Vec::new())),
            current_session: Arc::new(Mutex::new(None)),
            storage_path,
        })
    }

    /// Start a new recording session
    pub fn start_recording(&self, description: &str) -> Result<String> {
        let mut sessions_guard = self.sessions.lock().unwrap();
        let session_id = uuid::Uuid::new_v4().to_string();
        
        let session = RecordingSession {
            id: session_id.clone(),
            start_time: Utc::now(),
            snapshots: VecDeque::new(),
            max_snapshots: 1000, // Limit to prevent excessive memory usage
            is_recording: true,
            metadata: RecordingMetadata {
                project_path: std::env::current_dir()?,
                user: whoami::username(),
                description: description.to_string(),
                tags: vec!["automatic".to_string()],
            },
        };
        
        sessions_guard.push(session);
        *self.current_session.lock().unwrap() = Some(session_id.clone());
        
        Ok(session_id)
    }

    /// Stop the current recording session
    pub fn stop_recording(&self) -> Result<()> {
        let mut sessions_guard = self.sessions.lock().unwrap();
        if let Some(ref session_id) = *self.current_session.lock().unwrap() {
            for session in sessions_guard.iter_mut() {
                if session.id == *session_id {
                    session.is_recording = false;
                    break;
                }
            }
        }
        *self.current_session.lock().unwrap() = None;
        Ok(())
    }

    /// Add a snapshot to the current recording session
    pub fn add_snapshot(&self, command: &str, output: &str, context: &str) -> Result<()> {
        let session_id = {
            if let Some(id) = self.current_session.lock().unwrap().as_ref() {
                id.clone()
            } else {
                return Ok(()); // No active recording session
            }
        };
        
        let snapshot = SessionSnapshot {
            id: uuid::Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            command: command.to_string(),
            output: output.to_string(),
            context: context.to_string(),
            duration: Duration::from_millis(0), // Will be calculated later
            state_hash: format!("{:x}", blake3::hash(context.as_bytes())),
        };

        let mut sessions_guard = self.sessions.lock().unwrap();
        for session in sessions_guard.iter_mut() {
            if session.id == session_id {
                session.snapshots.push_back(snapshot);
                
                // Maintain size limits
                if session.snapshots.len() > session.max_snapshots {
                    session.snapshots.pop_front();
                }
                
                break;
            }
        }
        
        Ok(())
    }

    /// Load recordings from persistent storage
    pub fn load_recordings(&self) -> Result<Vec<RecordingSession>> {
        let mut recordings = Vec::new();
        
        // Look for recording files in the storage directory
        let entries = std::fs::read_dir(&self.storage_path)?;
        for entry in entries {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_file() && path.extension().map_or(false, |ext| ext == "json") {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    if let Ok(session) = serde_json::from_str::<RecordingSession>(&content) {
                        recordings.push(session);
                    }
                }
            }
        }
        
        Ok(recordings)
    }

    /// Save a session to persistent storage
    pub fn save_session(&self, session: &RecordingSession) -> Result<()> {
        let filename = format!("recording_{}_{}.json", 
                              session.id, 
                              session.start_time.format("%Y%m%d_%H%M%S"));
        let filepath = self.storage_path.join(filename);
        
        let content = serde_json::to_string_pretty(session)?;
        std::fs::write(filepath, content)?;
        Ok(())
    }

    /// Get all recorded sessions
    pub fn get_recorded_sessions(&self) -> Vec<RecordingSession> {
        self.sessions.lock().unwrap().clone()
    }

    /// Rewind to a specific point in the recording
    pub fn rewind_to_point(&self, session_id: &str, snapshot_id: &str) -> Result<SessionSnapshot> {
        let sessions_guard = self.sessions.lock().unwrap();
        
        for session in sessions_guard.iter() {
            if session.id == session_id {
                for snapshot in session.snapshots.iter() {
                    if snapshot.id == snapshot_id {
                        return Ok(snapshot.clone());
                    }
                }
            }
        }
        
        anyhow::bail!("Snapshot not found: {}", snapshot_id)
    }

    /// Rewind to a specific time in the recording
    pub fn rewind_to_time(&self, session_id: &str, target_time: DateTime<Utc>) -> Result<SessionSnapshot> {
        let sessions_guard = self.sessions.lock().unwrap();
        
        for session in sessions_guard.iter() {
            if session.id == session_id {
                // Find closest snapshot to the target time
                let mut closest_snapshot: Option<SessionSnapshot> = None;
                let mut min_diff = Duration::MAX;
                
                for snapshot in session.snapshots.iter() {
                    let diff = if snapshot.timestamp > target_time {
                        snapshot.timestamp - target_time
                    } else {
                        target_time - snapshot.timestamp
                    };
                    
                    if diff < min_diff {
                        min_diff = diff;
                        closest_snapshot = Some(snapshot.clone());
                    }
                }
                
                if let Some(snapshot) = closest_snapshot {
                    return Ok(snapshot);
                }
            }
        }
        
        anyhow::bail!("No snapshot found for session: {}", session_id)
    }

    /// Play back a recorded session step by step
    pub fn play_session(
        &self,
        session_id: &str,
        callback: impl FnMut(&SessionSnapshot) -> Result<()>,
    ) -> Result<()> {
        let sessions_guard = self.sessions.lock().unwrap();
        
        for session in sessions_guard.iter() {
            if session.id == session_id {
                for snapshot in &session.snapshots {
                    callback(snapshot)?;
                }
                return Ok(());
            }
        }
        
        anyhow::bail!("Session not found: {}", session_id)
    }

    /// Get a timeline of activities within a session
    pub fn get_timeline(&self, session_id: &str) -> Result<Vec<TimelineEntry>> {
        let mut timeline = Vec::new();
        let sessions_guard = self.sessions.lock().unwrap();
        
        for session in sessions_guard.iter() {
            if session.id == session_id {
                for snapshot in &session.snapshots {
                    timeline.push(TimelineEntry {
                        timestamp: snapshot.timestamp,
                        event_type: match snapshot.command.split_whitespace().next().unwrap_or("") {
                            "/ask" => EventType::Question,
                            "/refactor" | "/fix" => EventType::CodeModification,
                            "/test" => EventType::Testing,
                            "/commit" | "/review" => EventType::Review,
                            _ => EventType::Command,
                        },
                        summary: format!("{}: {}", snapshot.command, snapshot.output.chars().take(50).collect::<String>()),
                    });
                }
                break;
            }
        }
        
        Ok(timeline)
    }
}

#[derive(Debug, Clone)]
pub struct TimelineEntry {
    pub timestamp: DateTime<Utc>,
    pub event_type: EventType,
    pub summary: String,
}

#[derive(Debug, Clone)]
pub enum EventType {
    Question,
    CodeModification,
    Testing,
    Review,
    Command,
}

pub struct RewindCapabilities {
    pub recording_manager: RecordingManager,
}

impl RewindCapabilities {
    pub fn new(storage_path: PathBuf) -> Result<Self> {
        Ok(Self {
            recording_manager: RecordingManager::new(storage_path)?,
        })
    }

    /// Start a collaborative recording session
    pub fn start_collaborative_session(&self, description: &str, collaborators: &[String]) -> Result<String> {
        let session_id = self.recording_manager.start_recording(description)?;
        
        // Add collaborators to metadata
        let mut sessions_guard = self.recording_manager.sessions.lock().unwrap();
        if let Some(session) = sessions_guard.iter_mut().find(|s| s.id == session_id) {
            session.metadata.tags.extend(collaborators.iter().map(|c| format!("collab:{}", c)));
        }
        
        Ok(session_id)
    }
}

// Helper function to initialize recording capabilities in the system
pub fn initialize_recording_system() -> Result<RewindCapabilities> {
    let storage_path = dirs::data_dir()
        .unwrap_or_else(|| std::env::current_dir().unwrap())
        .join("kandil")
        .join("recordings");
    
    RewindCapabilities::new(storage_path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recording_lifecycle() -> Result<()> {
        let temp_dir = std::env::temp_dir().join("kandil_test_recording");
        let manager = RecordingManager::new(temp_dir)?;
        
        // Start recording
        let session_id = manager.start_recording("Test recording")?;
        
        // Add some snapshots
        manager.add_snapshot("ls -la", "file1.txt\nfile2.txt", "context1")?;
        manager.add_snapshot("pwd", "/home/user/test", "context2")?;
        
        // Stop recording
        manager.stop_recording()?;
        
        // Verify session exists
        let sessions = manager.get_recorded_sessions();
        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].snapshots.len(), 2);
        
        Ok(())
    }
}