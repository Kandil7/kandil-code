use crate::core::recording::{
    initialize_recording_system,
    RecordingManager,
    SessionSnapshot,
    RecordingSession,
    RewindCapabilities,
    TimelineEntry,
    EventType,
};
use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Duration;

// Re-export for easy access
pub use crate::core::recording::RewindCapabilities;

// Additional recording functionality goes here
pub struct RecordingSystem {
    pub manager: RecordingManager,
    pub active_session: Option<String>,
}

impl RecordingSystem {
    pub fn new(storage_path: PathBuf) -> Result<Self> {
        Ok(Self {
            manager: RecordingManager::new(storage_path)?,
            active_session: None,
        })
    }

    /// Start a new recording session for the current development activity
    pub fn start_session(&mut self, description: &str) -> Result<String> {
        let session_id = self.manager.start_recording(description)?;
        self.active_session = Some(session_id.clone());
        Ok(session_id)
    }

    /// End the current recording session
    pub fn end_session(&mut self) -> Result<()> {
        if self.active_session.is_some() {
            self.manager.stop_recording()?;
            self.active_session = None;
        }
        Ok(())
    }

    /// Record the current state including command, output, and context
    pub fn record_state(&self, command: &str, output: &str, context: &str) -> Result<()> {
        if self.active_session.is_some() {
            self.manager.add_snapshot(command, output, context)?;
        }
        Ok(())
    }

    /// Rewind to a previous state in the current session
    pub fn rewind_to_state(&mut self, target_index: usize) -> Result<SessionSnapshot> {
        if let Some(session_id) = &self.active_session {
            let sessions = self.manager.get_recorded_sessions();
            for session in sessions {
                if session.id == *session_id {
                    if let Some(snapshot) = session.snapshots.get(target_index) {
                        return Ok(snapshot.clone());
                    }
                }
            }
        }
        anyhow::bail!("Cannot rewind: no valid session or invalid index")
    }

    /// Rewind to a specific time in the current session
    pub fn rewind_to_time(&self, target_time: DateTime<Utc>) -> Result<SessionSnapshot> {
        if let Some(session_id) = &self.active_session {
            return self.manager.rewind_to_time(session_id, target_time);
        }
        anyhow::bail!("Cannot rewind: no active session")
    }

    /// Get the timeline of the current session
    pub fn get_session_timeline(&self) -> Result<Vec<TimelineEntry>> {
        if let Some(session_id) = &self.active_session {
            return self.manager.get_timeline(session_id);
        }
        Ok(Vec::new())
    }

    /// Get the current session details
    pub fn get_current_session(&self) -> Option<RecordingSession> {
        if let Some(session_id) = &self.active_session {
            let sessions = self.manager.get_recorded_sessions();
            for session in sessions {
                if session.id == *session_id {
                    return Some(session);
                }
            }
        }
        None
    }

    /// Save the current session to persistent storage
    pub fn save_current_session(&self) -> Result<()> {
        if let Some(session_id) = &self.active_session {
            let sessions = self.manager.get_recorded_sessions();
            for session in sessions {
                if session.id == *session_id {
                    return self.manager.save_session(&session);
                }
            }
        }
        Ok(())
    }
}

// Helper function to create a global recording system instance
pub fn create_recording_system() -> Result<RecordingSystem> {
    let storage_path = dirs::data_dir()
        .unwrap_or_else(|| std::env::current_dir().unwrap())
        .join("kandil")
        .join("sessions");
    
    std::fs::create_dir_all(&storage_path)?;
    
    let mut system = RecordingSystem::new(storage_path)?;
    
    // Initialize with a default session if needed
    if std::env::var("KANDIL_RECORD_SESSIONS").is_ok() {
        system.start_session("Auto-initialized session")?;
    }
    
    Ok(system)
}