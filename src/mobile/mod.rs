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

use crate::enhanced_ui::splash::JobSnapshot;

#[derive(Clone)]
pub struct MobileBridge {
    notifier: Arc<PushNotifier>,
    voice_receiver: Arc<VoiceReceiver>,
    announced_jobs: Arc<Mutex<HashSet<String>>>,
}

impl MobileBridge {
    pub fn new() -> Result<Self> {
        let root = mobile_root()?;
        Ok(Self {
            notifier: Arc::new(PushNotifier::new(root.join("notifications.log"))?),
            voice_receiver: Arc::new(VoiceReceiver::new(root.join("voice_queue.txt"))?),
            announced_jobs: Arc::new(Mutex::new(HashSet::new())),
        })
    }

    pub fn sync_jobs(&self, jobs: &[JobSnapshot]) {
        let mut announced = self.announced_jobs.lock().unwrap();
        for job in jobs {
            if job.completed && announced.insert(job.description.clone()) {
                let _ = self.notifier.send(PushNotification {
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
                });
            }
        }
    }

    pub fn try_voice_command(&self) -> Result<Option<String>> {
        self.voice_receiver.poll()
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

#[derive(Serialize)]
struct PushNotification {
    title: String,
    body: String,
    timestamp: DateTime<Utc>,
    actions: Vec<PushAction>,
}

#[derive(Serialize)]
struct PushAction {
    id: String,
    label: String,
}

impl PushAction {
    fn new(id: &str, label: &str) -> Self {
        Self {
            id: id.to_string(),
            label: label.to_string(),
        }
    }
}

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
