use chrono::{DateTime, Utc};
use std::{
    collections::HashSet,
    path::{Path, PathBuf},
    process::Command,
    time::SystemTime,
};
use walkdir::WalkDir;

#[derive(Debug, Clone)]
pub struct ProjectContext {
    pub project_type: ProjectType,
    pub git_state: GitState,
    pub recent_files: Vec<PathBuf>,
    pub errors: usize,
    pub test_failures: usize,
}

impl Default for ProjectContext {
    fn default() -> Self {
        Self {
            project_type: ProjectType::Unknown,
            git_state: GitState::default(),
            recent_files: Vec::new(),
            errors: 0,
            test_failures: 0,
        }
    }
}

impl ProjectContext {
    pub fn detect() -> Self {
        let cwd = match std::env::current_dir() {
            Ok(dir) => dir,
            Err(_) => return Self::default(),
        };

        let project_type = detect_project_type(&cwd);
        let git_state = detect_git_state(&cwd);
        let recent_files = detect_recent_files(&cwd, 5);

        Self {
            project_type,
            git_state,
            recent_files,
            errors: 0,
            test_failures: 0,
        }
    }

    pub fn suggested_commands(&self) -> Vec<&'static str> {
        let mut suggestions = vec!["/ask", "/review"];
        if self.errors > 0 {
            suggestions.push("/fix");
        }
        if self.test_failures > 0 {
            suggestions.push("/test");
        }
        if self.git_state.staged_files.is_empty() {
            suggestions.push("/commit");
        }
        suggestions
    }
}

#[derive(Debug, Clone)]
pub struct GitState {
    pub branch: Option<String>,
    pub staged_files: Vec<String>,
    pub unstaged_files: Vec<String>,
    pub conflicts: Vec<String>,
    pub last_commit: Option<DateTime<Utc>>,
}

impl Default for GitState {
    fn default() -> Self {
        Self {
            branch: None,
            staged_files: Vec::new(),
            unstaged_files: Vec::new(),
            conflicts: Vec::new(),
            last_commit: None,
        }
    }
}

#[derive(Debug, Clone)]
pub enum ProjectType {
    Rust,
    Node,
    Python,
    Unknown,
}

fn detect_project_type(root: &Path) -> ProjectType {
    let candidates = [
        (ProjectType::Rust, "Cargo.toml"),
        (ProjectType::Node, "package.json"),
        (ProjectType::Python, "pyproject.toml"),
    ];

    for (ty, marker) in candidates {
        if root.join(marker).exists() {
            return ty;
        }
    }

    ProjectType::Unknown
}

fn detect_git_state(root: &Path) -> GitState {
    if !root.join(".git").exists() {
        return GitState::default();
    }

    let branch = run_git_command(root, &["rev-parse", "--abbrev-ref", "HEAD"])
        .map(|s| s.trim().to_string())
        .ok();

    let status = run_git_command(root, &["status", "--porcelain"]).unwrap_or_default();

    let mut staged = Vec::new();
    let mut unstaged = Vec::new();
    let mut conflicts = Vec::new();

    for line in status.lines() {
        if line.len() < 3 {
            continue;
        }
        let code = &line[..2];
        let path = line[3..].to_string();
        match code {
            "??" => unstaged.push(path),
            "UU" | "AA" | "DD" => conflicts.push(path),
            _ => {
                if code.starts_with('M') || code.starts_with('A') {
                    staged.push(path);
                } else {
                    unstaged.push(path);
                }
            }
        }
    }

    let last_commit = run_git_command(root, &["log", "-1", "--format=%cI"])
        .ok()
        .and_then(|ts| DateTime::parse_from_rfc3339(ts.trim()).ok())
        .map(|dt| dt.with_timezone(&Utc));

    GitState {
        branch,
        staged_files: staged,
        unstaged_files: unstaged,
        conflicts,
        last_commit,
    }
}

fn detect_recent_files(root: &Path, limit: usize) -> Vec<PathBuf> {
    let mut entries: Vec<(PathBuf, SystemTime)> = WalkDir::new(root)
        .into_iter()
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.path().is_file())
        .filter_map(|entry| {
            entry.metadata().ok().and_then(|meta| {
                meta.modified()
                    .ok()
                    .map(|mtime| (entry.path().to_path_buf(), mtime))
            })
        })
        .collect();

    entries.sort_by_key(|(_, time)| *time);
    entries
        .into_iter()
        .rev()
        .take(limit)
        .map(|(path, _)| path)
        .collect()
}

fn run_git_command(root: &Path, args: &[&str]) -> Result<String, std::io::Error> {
    let output = Command::new("git").args(args).current_dir(root).output()?;
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            String::from_utf8_lossy(&output.stderr).to_string(),
        ))
    }
}

pub fn detect_recent_branches(root: &Path, limit: usize) -> Vec<String> {
    if !root.join(".git").exists() {
        return Vec::new();
    }
    let output = run_git_command(root, &["branch"]).unwrap_or_default();
    output
        .lines()
        .map(|line| line.trim_start_matches('*').trim().to_string())
        .filter(|branch| !branch.is_empty())
        .collect::<Vec<_>>()
        .into_iter()
        .take(limit)
        .collect()
}

pub fn detect_unique_extensions(root: &Path, limit: usize) -> Vec<String> {
    let mut extensions = HashSet::new();
    for entry in WalkDir::new(root).into_iter().filter_map(|e| e.ok()) {
        if entry.path().is_file() {
            if let Some(ext) = entry.path().extension().and_then(|e| e.to_str()) {
                extensions.insert(ext.to_string());
            }
        }
        if extensions.len() >= limit {
            break;
        }
    }
    extensions.into_iter().collect()
}
