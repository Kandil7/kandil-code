use crate::enhanced_ui::adaptive::AdaptiveUI;
use anyhow::Result;
use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::{Path, PathBuf};
use std::thread;
use std::time::Duration;
use tokio::sync::mpsc;

pub struct IdeSyncBridge {
    root: PathBuf,
}

impl IdeSyncBridge {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    pub fn launch(&self, adaptive_ui: AdaptiveUI) -> Result<()> {
        let watch_root = self.root.clone();
        if !watch_root.exists() {
            return Err(anyhow::anyhow!(
                "Watch root does not exist: {}",
                watch_root.display()
            ));
        }

        let (tx, mut rx) = mpsc::unbounded_channel::<Event>();
        let watcher_thread = spawn_watcher(&watch_root, tx)?;
        tokio::spawn(async move {
            adaptive_ui.announce("ide-sync", "🔄 IDE sync bridge active");
            while let Some(event) = rx.recv().await {
                if let Some(path) = event.paths.first() {
                    if ignored(path) {
                        continue;
                    }
                    let message = match event.kind {
                        notify::EventKind::Modify(_) => "File updated",
                        notify::EventKind::Create(_) => "File created",
                        notify::EventKind::Remove(_) => "File removed",
                        _ => "File event",
                    };
                    adaptive_ui.announce(
                        "ide-sync",
                        &format!("{} → {}", message, display_relative(path, &watch_root)),
                    );
                }
            }
            drop(watcher_thread);
        });
        Ok(())
    }
}

fn spawn_watcher(root: &Path, tx: mpsc::UnboundedSender<Event>) -> Result<thread::JoinHandle<()>> {
    let root = root.to_path_buf();
    let handle = thread::spawn(move || {
        let mut watcher = RecommendedWatcher::new(
            move |res| {
                if let Ok(event) = res {
                    let _ = tx.send(event);
                }
            },
            Config::default(),
        )
        .expect("Failed to start watcher");

        watcher
            .watch(root.as_path(), RecursiveMode::Recursive)
            .expect("Failed to watch project root");

        loop {
            thread::sleep(Duration::from_secs(60));
        }
    });
    Ok(handle)
}

fn ignored(path: &Path) -> bool {
    path.components().any(|component| {
        if let std::path::Component::Normal(os) = component {
            os == "target" || os == ".git"
        } else {
            false
        }
    })
}

fn display_relative(path: &Path, root: &Path) -> String {
    path.strip_prefix(root)
        .map(|p| p.display().to_string())
        .unwrap_or_else(|_| path.display().to_string())
}
