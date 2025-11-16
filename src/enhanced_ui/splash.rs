use crate::enhanced_ui::{
    context::ProjectContext, smart_prompt::SmartPrompt, terminal::KandilTerminal,
};
use anyhow::{anyhow, Result};
use lazy_static::lazy_static;
use std::{
    collections::VecDeque,
    path::PathBuf,
    sync::Arc,
    time::{Duration, Instant},
};

#[derive(Clone)]
pub struct CommandContext {
    pub terminal: Arc<KandilTerminal>,
    pub recent_commands: VecDeque<String>,
    pub active_file: Option<PathBuf>,
    pub job_tracker: JobTracker,
    pub project_context: ProjectContext,
}

impl CommandContext {
    pub fn new(terminal: Arc<KandilTerminal>) -> Self {
        Self {
            terminal,
            recent_commands: VecDeque::with_capacity(100),
            active_file: None,
            job_tracker: JobTracker::default(),
            project_context: ProjectContext::detect(),
        }
    }

    pub fn remember_command(&mut self, command: &str) {
        if self.recent_commands.len() == self.recent_commands.capacity() {
            self.recent_commands.pop_front();
        }
        self.recent_commands.push_back(command.to_string());
    }

    pub fn refresh_project_context(&mut self) {
        self.project_context = ProjectContext::detect();
    }

    pub fn contextual_suggestions(&self) -> Vec<&'static str> {
        self.project_context.suggested_commands()
    }
}

#[derive(Clone)]
pub struct SplashCommand {
    pub trigger: &'static str,
    pub description: &'static str,
    pub requires_approval: bool,
    pub preview_action: Option<&'static str>,
}

lazy_static! {
    pub static ref SPLASH_COMMANDS: Vec<SplashCommand> = vec![
        SplashCommand {
            trigger: "/ask",
            description: "Ask a question about your code or project",
            requires_approval: false,
            preview_action: None,
        },
        SplashCommand {
            trigger: "/refactor",
            description: "Run AI-assisted refactor suggestions",
            requires_approval: true,
            preview_action: Some("Preview code changes"),
        },
        SplashCommand {
            trigger: "/test",
            description: "Generate or run tests for the active file",
            requires_approval: false,
            preview_action: Some("Show affected tests"),
        },
        SplashCommand {
            trigger: "/fix",
            description: "Analyze and fix compilation/runtime errors",
            requires_approval: true,
            preview_action: Some("Show error summary"),
        },
        SplashCommand {
            trigger: "/commit",
            description: "Generate semantic commit message",
            requires_approval: false,
            preview_action: Some("Show diff summary"),
        },
        SplashCommand {
            trigger: "/review",
            description: "Request AI code review on staged changes",
            requires_approval: false,
            preview_action: None,
        },
        SplashCommand {
            trigger: "/doc",
            description: "Generate or update documentation",
            requires_approval: true,
            preview_action: Some("Show doc sections"),
        },
        SplashCommand {
            trigger: "/deploy",
            description: "Draft deployment plan with validation",
            requires_approval: true,
            preview_action: Some("Show deployment checklist"),
        },
        SplashCommand {
            trigger: "/model",
            description: "Switch the active AI model",
            requires_approval: false,
            preview_action: None,
        },
        SplashCommand {
            trigger: "/history",
            description: "Show recent splash commands",
            requires_approval: false,
            preview_action: None,
        },
        SplashCommand {
            trigger: "/undo",
            description: "Undo the last AI action",
            requires_approval: false,
            preview_action: Some("Show undo diff"),
        }
    ];
}

pub async fn execute_splash_command(
    trigger: &str,
    args: &[String],
    ctx: &mut CommandContext,
) -> Result<SplashResult> {
    match trigger {
        "/ask" => handle_ask(args).await,
        "/refactor" => handle_refactor(args).await,
        "/test" => handle_test(args, ctx).await,
        "/fix" => handle_fix().await,
        "/commit" => handle_commit().await,
        "/review" => handle_review().await,
        "/doc" => handle_doc(args).await,
        "/deploy" => handle_deploy(args).await,
        "/model" => handle_model_switch(args).await,
        "/history" => handle_history(ctx).await,
        "/undo" => handle_undo(ctx).await,
        "/jobs" => handle_jobs(ctx).await,
        _ => Err(anyhow!("Unknown splash command: {}", trigger)),
    }
}

pub fn suggest_commands(prefix: &str) -> Vec<&'static SplashCommand> {
    SPLASH_COMMANDS
        .iter()
        .filter(|cmd| cmd.trigger.starts_with(prefix))
        .collect()
}

#[derive(Default, Clone)]
pub struct SplashResult {
    pub message: Option<String>,
}

async fn handle_ask(args: &[String]) -> Result<SplashResult> {
    let question = if args.is_empty() {
        "What should I focus on next?".to_string()
    } else {
        args.join(" ")
    };
    Ok(SplashResult {
        message: Some(format!("🤖 Answering question: {}", question)),
    })
}

async fn handle_refactor(args: &[String]) -> Result<SplashResult> {
    let preview = SmartPrompt::preview_actions(
        "Refactor",
        &["Analyze target file", "Generate suggestions", "Render diff"],
    );
    let target = if args.is_empty() {
        "current module".to_string()
    } else {
        args.join(" ")
    };
    Ok(SplashResult {
        message: Some(format!("{}\n🔧 Target: {}", preview, target)),
    })
}

async fn handle_test(args: &[String], ctx: &mut CommandContext) -> Result<SplashResult> {
    if args.iter().any(|arg| arg == "--background") {
        ctx.job_tracker.spawn_job("cargo test");
        let msg = SmartPrompt::background_job_message("cargo test", Duration::from_secs(45));
        return Ok(SplashResult {
            message: Some(format!("🧪 {}", msg)),
        });
    }
    Ok(SplashResult {
        message: Some("🧪 Running tests in foreground".to_string()),
    })
}

async fn handle_fix() -> Result<SplashResult> {
    Ok(SplashResult {
        message: Some("🩺 Fix analysis started".to_string()),
    })
}

async fn handle_commit() -> Result<SplashResult> {
    Ok(SplashResult {
        message: Some("✍️ Drafting semantic commit message".to_string()),
    })
}

async fn handle_review() -> Result<SplashResult> {
    Ok(SplashResult {
        message: Some("🔍 Running AI review on staged changes".to_string()),
    })
}

async fn handle_doc(_args: &[String]) -> Result<SplashResult> {
    let preview = SmartPrompt::preview_actions("Docs", &["Scan codebase", "Generate markdown"]);
    Ok(SplashResult {
        message: Some(format!("📘 {}", preview)),
    })
}

async fn handle_deploy(args: &[String]) -> Result<SplashResult> {
    let confirmed = SmartPrompt::confirm("Deploy may affect production. Continue?");
    let target = if args.is_empty() {
        "default environment"
    } else {
        &args[0]
    };
    Ok(SplashResult {
        message: Some(format!(
            "🚀 Deployment checklist for {} (approved: {})",
            target, confirmed
        )),
    })
}

async fn handle_model_switch(args: &[String]) -> Result<SplashResult> {
    if args.len() < 2 {
        return Ok(SplashResult {
            message: Some(
                "Usage: /model <provider> <model>. Example: /model ollama qwen2.5-coder-3b".into(),
            ),
        });
    }
    Ok(SplashResult {
        message: Some(format!(
            "Switching provider {} to model {}",
            args[0], args[1]
        )),
    })
}

async fn handle_history(ctx: &CommandContext) -> Result<SplashResult> {
    let entries: Vec<String> = ctx.recent_commands.iter().rev().take(5).cloned().collect();
    if entries.is_empty() {
        return Ok(SplashResult {
            message: Some("No recent splash commands.".to_string()),
        });
    }
    Ok(SplashResult {
        message: Some(format!("Recent splash commands:\n{}", entries.join("\n"))),
    })
}

async fn handle_undo(_ctx: &mut CommandContext) -> Result<SplashResult> {
    Ok(SplashResult {
        message: Some("↩️ Reverting last AI action (simulated)".to_string()),
    })
}

async fn handle_jobs(ctx: &mut CommandContext) -> Result<SplashResult> {
    Ok(SplashResult {
        message: Some(ctx.job_tracker.render_jobs()),
    })
}

#[derive(Default, Clone)]
pub struct JobTracker {
    jobs: Vec<JobStatus>,
}

impl JobTracker {
    pub fn spawn_job(&mut self, description: &str) {
        self.jobs.push(JobStatus {
            description: description.to_string(),
            started_at: Instant::now(),
            completed: false,
        });
    }

    pub fn complete_all(&mut self) {
        for job in &mut self.jobs {
            job.completed = true;
        }
    }

    pub fn auto_complete_elapsed(&mut self, threshold: Duration) {
        for job in &mut self.jobs {
            if !job.completed && job.started_at.elapsed() >= threshold {
                job.completed = true;
            }
        }
    }

    pub fn render_jobs(&self) -> String {
        if self.jobs.is_empty() {
            return "No active jobs.".to_string();
        }
        self.jobs
            .iter()
            .map(|job| {
                let duration = Instant::now().duration_since(job.started_at);
                format!(
                    "{} - {:.1}s {}",
                    job.description,
                    duration.as_secs_f32(),
                    if job.completed { "(done)" } else { "(running)" }
                )
            })
            .collect::<Vec<String>>()
            .join("\n")
    }

    pub fn snapshot(&self) -> Vec<JobSnapshot> {
        self.jobs
            .iter()
            .map(|job| JobSnapshot {
                description: job.description.clone(),
                completed: job.completed,
                duration_secs: job.started_at.elapsed().as_secs_f32(),
            })
            .collect()
    }
}

#[derive(Clone)]
struct JobStatus {
    description: String,
    started_at: Instant,
    completed: bool,
}

#[derive(Clone)]
pub struct JobSnapshot {
    pub description: String,
    pub completed: bool,
    pub duration_secs: f32,
}
