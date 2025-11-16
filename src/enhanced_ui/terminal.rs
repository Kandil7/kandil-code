use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use portable_pty::{native_pty_system, Child, CommandBuilder, ExitStatus, PtySize};
use std::{
    collections::HashMap,
    env,
    io::{Read, Write},
    path::PathBuf,
    sync::Arc,
    thread,
    time::{Duration, Instant},
};
use tokio::sync::RwLock;

/// Enhanced PTY manager for better isolation and session tracking
#[derive(Clone)]
pub struct PtyManager {
    pty_system: Arc<dyn portable_pty::PtySystem + Send + Sync>,
}

impl PtyManager {
    pub fn new() -> Result<Self> {
        Ok(Self {
            pty_system: portable_pty::native_pty_system(),
        })
    }

    pub fn create_pty_pair(&self, size: PtySize) -> Result<Box<dyn portable_pty::PtyPair + Send + Sync>> {
        Ok(self.pty_system.openpty(size)?)
    }
}

/// Represents the Kandil internal terminal runtime backed by a PTY.
pub struct KandilTerminal {
    timeout: Duration,
    env_vars: HashMap<String, String>,
    execution_log: Arc<RwLock<Vec<ExecutionRecord>>>,
    permission_controller: PermissionController,
    output_processor: OutputProcessor,
    pty_manager: PtyManager,  // Enhanced PTY isolation
    session_id: String,       // For command logging and session tracking
}

impl Clone for KandilTerminal {
    fn clone(&self) -> Self {
        Self {
            timeout: self.timeout,
            env_vars: self.env_vars.clone(),
            execution_log: self.execution_log.clone(),
            permission_controller: self.permission_controller.clone(),
            output_processor: self.output_processor.clone(),
            pty_manager: self.pty_manager.clone(),
            session_id: self.session_id.clone(),
        }
    }
}

impl KandilTerminal {
    pub fn new() -> Result<Self> {
        Ok(Self {
            timeout: Duration::from_secs(30),
            env_vars: Self::sanitize_env(),
            execution_log: Arc::new(RwLock::new(Vec::new())),
            permission_controller: PermissionController::default(),
            output_processor: OutputProcessor::default(),
            pty_manager: PtyManager::new()?,
            session_id: uuid::Uuid::new_v4().to_string(),
        })
    }

    /// Execute a shell command with optional approval requirements.
    pub async fn execute(&self, command: &str, user_approved: bool) -> Result<CommandResult> {
        let parsed = self.parse_command(command)?;

        if !user_approved && self.permission_controller.requires_approval(&parsed) {
            return Err(anyhow!(
                "Command \"{}\" requires explicit user approval",
                parsed
            ));
        }

        let cwd = env::current_dir()?;
        let start_time = Instant::now();

        // Record the command execution with enhanced metadata
        let record = ExecutionRecord {
            command: parsed.clone(),
            timestamp: Utc::now(),
            cwd: cwd.clone(),
            session_id: self.session_id.clone(),
            start_time: start_time,
            duration: Duration::from_secs(0), // Will update after execution
            status: ExecutionStatus::Running,
        };

        let env_vars = self.env_vars.clone();
        let timeout = self.timeout;
        let pty_size = PtySize {
            rows: 40,
            cols: 120,
            pixel_width: 0,
            pixel_height: 0,
        };
        let pty_manager = self.pty_manager.clone();

        let output = tokio::task::spawn_blocking(move || {
            run_command_in_pty_enhanced(&parsed, &env_vars, &cwd, timeout, pty_manager, pty_size)
        })
        .await??;

        // Update the record with execution results
        let duration = start_time.elapsed();
        let final_record = ExecutionRecord {
            command: record.command,
            timestamp: record.timestamp,
            cwd: record.cwd,
            session_id: record.session_id,
            start_time: record.start_time,
            duration,
            status: if output.status_code == 0 { ExecutionStatus::Success } else { ExecutionStatus::Failed },
        };

        self.execution_log.write().await.push(final_record);
        let ai_analysis = self.output_processor.analyze(&output.stdout);

        Ok(CommandResult {
            status_code: output.status_code,
            stdout: output.stdout,
            stderr: output.stderr,
            ai_analysis,
        })
    }

    pub async fn reset_context(&self) -> Result<()> {
        self.execution_log.write().await.clear();
        Ok(())
    }

    pub async fn execution_log(&self) -> Vec<ExecutionRecord> {
        self.execution_log.read().await.clone()
    }

    pub fn clear_screen(&self) -> Result<()> {
        let mut stdout = std::io::stdout();
        stdout.write_all(b"\x1b[2J\x1b[H")?;
        stdout.flush()?;
        Ok(())
    }

    pub async fn capture_frame(&self) -> Result<TerminalFrame> {
        // Capture current terminal state for GPU rendering
        // This is a simplified version - in a full implementation,
        // this would capture the actual visible terminal buffer
        let log = self.execution_log.read().await;
        let lines: Vec<String> = log
            .iter()
            .take(100) // Limit to last 100 commands for performance
            .map(|record| format!("[{}] {}", record.timestamp.format("%H:%M:%S"), record.command))
            .collect();
        
        Ok(TerminalFrame { lines })
    }

    /// Get visible terminal cells for GPU rendering
    pub fn visible_cells(&self) -> Option<Vec<TerminalCell>> {
        // For now, return None - this will be implemented when we have
        // actual terminal buffer access
        // Future: Return actual visible cells from terminal buffer
        None
    }

    fn parse_command(&self, raw: &str) -> Result<String> {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            return Err(anyhow!("Command cannot be empty"));
        }
        Ok(trimmed.to_string())
    }

    fn sanitize_env() -> HashMap<String, String> {
        const SENSITIVE: [&str; 6] = [
            "API_KEY",
            "OPENAI_API_KEY",
            "KANDIL_API_KEY",
            "AWS_SECRET_ACCESS_KEY",
            "GCP_SERVICE_KEY",
            "AZURE_CLIENT_SECRET",
        ];

        env::vars()
            .filter(|(key, _)| !SENSITIVE.iter().any(|s| key.eq_ignore_ascii_case(s)))
            .collect()
    }
}

/// PTY execution helper result.
struct RawCommandResult {
    status_code: i32,
    stdout: String,
    stderr: String,
}

fn run_command_in_pty_enhanced(
    command: &str,
    env_vars: &HashMap<String, String>,
    cwd: &PathBuf,
    timeout: Duration,
    pty_manager: PtyManager,
    pty_size: PtySize,
) -> Result<RawCommandResult> {
    let mut cmd = build_shell_command(command);
    cmd.cwd(cwd);
    cmd.env_clear();
    for (key, value) in env_vars {
        cmd.env(key, value);
    }

    let pair = pty_manager.create_pty_pair(pty_size)?;

    let mut child = pair.slave().spawn_command(cmd)?;
    drop(pair.slave());

    let mut reader = pair.master().try_clone_reader()?;
    let reader_handle = thread::spawn(move || {
        let mut buffer = Vec::new();
        let mut chunk = [0u8; 4096];
        while let Ok(read) = reader.read(&mut chunk) {
            if read == 0 {
                break;
            }
            buffer.extend_from_slice(&chunk[..read]);
        }
        buffer
    });

    let status_code = wait_with_timeout(&mut child, timeout)?;
    let stdout = reader_handle
        .join()
        .map(|bytes| String::from_utf8_lossy(&bytes).to_string())
        .unwrap_or_default();

    Ok(RawCommandResult {
        status_code,
        stdout,
        stderr: String::new(),
    })
}

fn run_command_in_pty(
    command: &str,
    env_vars: &HashMap<String, String>,
    cwd: &PathBuf,
    timeout: Duration,
) -> Result<RawCommandResult> {
    run_command_in_pty_enhanced(
        command,
        env_vars,
        cwd,
        timeout,
        PtyManager::new()?,
        PtySize {
            rows: 40,
            cols: 120,
            pixel_width: 0,
            pixel_height: 0,
        }
    )
}

fn wait_with_timeout(child: &mut Box<dyn Child + Send + Sync>, timeout: Duration) -> Result<i32> {
    let start = Instant::now();
    loop {
        if let Some(status) = child.try_wait()? {
            return Ok(extract_code(status));
        }

        if start.elapsed() > timeout {
            child.kill()?;
            return Err(anyhow!("Command timed out after {:?}", timeout));
        }
        thread::sleep(Duration::from_millis(25));
    }
}

fn extract_code(status: ExitStatus) -> i32 {
    if status.success() {
        0
    } else {
        status.exit_code() as i32
    }
}

fn build_shell_command(raw: &str) -> CommandBuilder {
    #[cfg(windows)]
    {
        let mut cmd = CommandBuilder::new("cmd.exe");
        cmd.arg("/C");
        cmd.arg(raw);
        cmd
    }

    #[cfg(not(windows))]
    {
        let mut cmd = CommandBuilder::new("sh");
        cmd.arg("-c");
        cmd.arg(raw);
        cmd
    }
}

#[derive(Debug, Clone)]
pub struct ExecutionRecord {
    pub command: String,
    pub timestamp: DateTime<Utc>,
    pub cwd: PathBuf,
    pub session_id: String,
    pub start_time: Instant,
    pub duration: Duration,
    pub status: ExecutionStatus,
}

#[derive(Debug, Clone)]
pub enum ExecutionStatus {
    Running,
    Success,
    Failed,
}

#[derive(Debug, Clone)]
pub struct CommandResult {
    pub status_code: i32,
    pub stdout: String,
    pub stderr: String,
    pub ai_analysis: Option<String>,
}

#[derive(Clone)]
struct PermissionController {
    protected_tokens: Vec<&'static str>,
}

impl Default for PermissionController {
    fn default() -> Self {
        Self {
            protected_tokens: vec![
                "rm -rf",
                "shutdown",
                "format ",
                ":(){",
                "mkfs",
                "DROP TABLE",
            ],
        }
    }
}

impl PermissionController {
    fn requires_approval(&self, command: &str) -> bool {
        self.protected_tokens
            .iter()
            .any(|token| command.contains(token))
    }
}

#[derive(Debug, Clone)]
pub struct TerminalFrame {
    pub lines: Vec<String>,
}

/// Represents a single terminal cell for GPU rendering
#[derive(Debug, Clone)]
pub struct TerminalCell {
    pub character: char,
    pub foreground: u32,
    pub background: u32,
    pub bold: bool,
    pub italic: bool,
}

#[derive(Clone, Default)]
struct OutputProcessor;

impl OutputProcessor {
    fn analyze(&self, output: &str) -> Option<String> {
        if output.contains("error") || output.contains("failed") {
            Some("Detected potential errors in output".to_string())
        } else if output.contains("warning") {
            Some("Output contains warnings".to_string())
        } else {
            None
        }
    }
}
