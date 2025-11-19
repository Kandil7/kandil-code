use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::Serialize;
use std::{
    collections::HashSet,
    fs::{self, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};
use tokio::sync::mpsc;

// Use tokio's time for async sleep
use tokio::sync::oneshot;
use tokio::time::{sleep, Duration};

use crate::enhanced_ui::splash::JobSnapshot;

#[derive(Serialize, Clone, Debug)]
pub struct PushNotification {
    pub id: String,
    pub title: String,
    pub body: String,
    pub timestamp: DateTime<Utc>,
    pub actions: Vec<PushAction>,
    pub priority: NotificationPriority,
}

#[derive(Serialize, Clone, Debug)]
pub struct PushAction {
    pub id: String,
    pub label: String,
}

#[derive(Serialize, Clone, Debug)]
pub enum NotificationPriority {
    Low,
    Normal,
    High,
    Urgent,
}

impl PushAction {
    pub fn new(id: &str, label: &str) -> Self {
        Self {
            id: id.to_string(),
            label: label.to_string(),
        }
    }
}

#[derive(Clone)]
pub struct MobileBridge {
    notifier: Arc<PushNotifier>,
    voice_receiver: Arc<VoiceReceiver>,
    approval_handler: Arc<ApprovalHandler>,
    announced_jobs: Arc<Mutex<HashSet<String>>>,
}

// Handler for managing approvals
#[derive(Clone)]
pub struct ApprovalHandler {
    pending_approvals: Arc<tokio::sync::Notify>,
    approval_tx: mpsc::UnboundedSender<ApprovalRequest>,
    approval_rx: Arc<Mutex<Option<mpsc::UnboundedReceiver<ApprovalRequest>>>>,
}

#[derive(Debug, Clone)]
pub struct ApprovalRequest {
    pub id: String,
    pub title: String,
    pub message: String,
    pub request_data: String,
}

impl ApprovalHandler {
    fn new() -> Self {
        let (approval_tx, approval_rx) = mpsc::unbounded_channel();

        Self {
            pending_approvals: Arc::new(tokio::sync::Notify::new()),
            approval_tx,
            approval_rx: Arc::new(Mutex::new(Some(approval_rx))),
        }
    }

    /// Send an approval request to mobile devices
    pub fn request_approval(
        &self,
        title: &str,
        message: &str,
        request_data: &str,
    ) -> Result<String> {
        let id = uuid::Uuid::new_v4().to_string();
        let request = ApprovalRequest {
            id: id.clone(),
            title: title.to_string(),
            message: message.to_string(),
            request_data: request_data.to_string(),
        };

        self.approval_tx.send(request)
            .map_err(|e| anyhow::anyhow!("Failed to send approval request: {}", e))?;

        Ok(id)
    }

    /// Check for pending approvals
    pub async fn poll_approvals(&self) -> Option<ApprovalRequest> {
        if let Ok(mut rx_lock) = self.approval_rx.try_lock() {
            if let Some(ref mut rx) = *rx_lock {
                return rx.try_recv().ok();
            }
        }
        None
    }
}

impl MobileBridge {
    pub fn new() -> Result<Self> {
        let root = mobile_root()?;
        Ok(Self {
            notifier: Arc::new(PushNotifier::new(root.join("notifications.log"))?),
            voice_receiver: Arc::new(VoiceReceiver::new(root.join("voice_queue.txt"))?),
            approval_handler: Arc::new(ApprovalHandler::new()),
            announced_jobs: Arc::new(Mutex::new(HashSet::new())),
        })
    }

    pub fn sync_jobs(&self, jobs: &[JobSnapshot]) {
        let mut announced = self.announced_jobs.lock().unwrap();
        for job in jobs {
            if job.completed && announced.insert(job.description.clone()) {
                let notification = PushNotification {
                    id: uuid::Uuid::new_v4().to_string(),
                    title: "Kandil Task Complete".to_string(),
                    body: format!(
                        "{} finished after {:.1}s",
                        job.description, job.duration_secs
                    ),
                    timestamp: Utc::now(),
                    actions: vec![
                        PushAction::new("view", "View Results"),
                        PushAction::new("approve", "Approve"),
                    ],
                    priority: NotificationPriority::Normal,
                };

                let _ = self.notifier.send(notification);
            }
        }
    }

    pub fn try_voice_command(&self) -> Result<Option<String>> {
        self.voice_receiver.poll()
    }

    /// Send a push notification to mobile devices
    pub fn send_notification(&self, title: &str, body: &str, priority: NotificationPriority) -> Result<()> {
        let notification = PushNotification {
            id: uuid::Uuid::new_v4().to_string(),
            title: title.to_string(),
            body: body.to_string(),
            timestamp: Utc::now(),
            actions: vec![
                PushAction::new("acknowledge", "Acknowledge"),
                PushAction::new("dismiss", "Dismiss"),
            ],
            priority,
        };

        self.notifier.send(notification)
    }

    /// Request approval for a specific action with a push notification
    pub fn request_approval(
        &self,
        title: &str,
        message: &str,
        request_data: &str
    ) -> Result<String> {
        // Send push notification with approval actions
        let notification = PushNotification {
            id: uuid::Uuid::new_v4().to_string(),
            title: title.to_string(),
            body: message.to_string(),
            timestamp: Utc::now(),
            actions: vec![
                PushAction::new("approve", "✅ Approve"),
                PushAction::new("reject", "❌ Reject"),
                PushAction::new("defer", "⏰ Later"),
            ],
            priority: NotificationPriority::High,
        };

        self.notifier.send(notification.clone())?;

        // Also register the approval request
        self.approval_handler.request_approval(title, message, request_data)
    }

    /// Check for any pending approvals from mobile devices
    pub async fn check_approvals(&self) -> Option<ApprovalRequest> {
        self.approval_handler.poll_approvals().await
    }

    /// Send notification for command execution requiring approval
    pub fn notify_command_execution(
        &self,
        command: &str,
        description: &str,
        requires_approval: bool
    ) -> Result<()> {
        let (title, priority) = if requires_approval {
            ("Action Requires Approval", NotificationPriority::Urgent)
        } else {
            ("Command Executed", NotificationPriority::Normal)
        };

        let notification = PushNotification {
            id: uuid::Uuid::new_v4().to_string(),
            title: title.to_string(),
            body: format!("Command: {}\nDescription: {}", command, description),
            timestamp: Utc::now(),
            actions: if requires_approval {
                vec![
                    PushAction::new("approve", "✅ Approve"),
                    PushAction::new("reject", "❌ Reject"),
                    PushAction::new("details", "🔍 View Details"),
                ]
            } else {
                vec![
                    PushAction::new("view", "View Output"),
                    PushAction::new("share", "Share Results"),
                ]
            },
            priority,
        };

        self.notifier.send(notification)
    }
}

impl Default for PushNotification {
    fn default() -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            title: "Notification".to_string(),
            body: "Message".to_string(),
            timestamp: Utc::now(),
            actions: vec![PushAction::new("ok", "OK")],
            priority: NotificationPriority::Normal,
        }
    }
}

fn mobile_root() -> Result<PathBuf> {
    let root = dirs::data_dir()
        .unwrap_or_else(|| std::env::current_dir().unwrap())
        .join("kandil")
        .join("mobile");
    fs::create_dir_all(&root)?;
    Ok(root)
}

struct PushNotifier {
    log_path: PathBuf,
}

impl PushNotifier {
    fn new(path: PathBuf) -> Result<Self> {
        if let Some(dir) = path.parent() {
            fs::create_dir_all(dir)?;
        }
        Ok(Self { log_path: path })
    }

    fn send(&self, notification: PushNotification) -> Result<()> {
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.log_path)?;
        writeln!(file, "{}", serde_json::to_string(&notification)?)?;
        println!("📱 Push → {}: {}", notification.title, notification.body);
        Ok(())
    }
}

// The PushNotification and PushAction structs are defined earlier in the file
// This is a duplicate definition that's not needed anymore

struct VoiceReceiver {
    queue: PathBuf,
    guard: Mutex<()>,
}

impl VoiceReceiver {
    fn new(queue: PathBuf) -> Result<Self> {
        if let Some(parent) = queue.parent() {
            fs::create_dir_all(parent)?;
        }
        if !queue.exists() {
            fs::File::create(&queue)?;
        }
        Ok(Self {
            queue,
            guard: Mutex::new(()),
        })
    }

    fn poll(&self) -> Result<Option<String>> {
        let _lock = self.guard.lock().unwrap();
        let content = fs::read_to_string(&self.queue)?;
        let mut lines: Vec<String> = content
            .lines()
            .map(|line| line.trim().to_string())
            .filter(|line| !line.is_empty())
            .collect();
        if lines.is_empty() {
            return Ok(None);
        }
        let next = lines.remove(0);
        fs::write(&self.queue, lines.join("\n"))?;
        Ok(Some(next))
    }
}
