use chrono::{DateTime, Utc};
use serde::Serialize;
use std::{
    collections::HashSet,
    path::{Path, PathBuf},
    process::Command,
    time::SystemTime,
};
use walkdir::WalkDir;

#[derive(Debug, Clone, Serialize)]
pub struct ProjectContext {
    pub project_type: ProjectType,
    pub git_state: GitState,
    pub recent_files: Vec<PathBuf>,
    pub errors: usize,
    pub test_failures: usize,
    pub detailed_errors: Vec<BuildError>,
    pub detailed_test_failures: Vec<TestFailure>,
}

impl Default for ProjectContext {
    fn default() -> Self {
        Self {
            project_type: ProjectType::Unknown,
            git_state: GitState::default(),
            recent_files: Vec::new(),
            errors: 0,
            test_failures: 0,
            detailed_errors: Vec::new(),
            detailed_test_failures: Vec::new(),
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
            detailed_errors: Vec::new(),
            detailed_test_failures: Vec::new(),
        }
    }

    pub fn detect_with_analysis() -> Self {
        let cwd = match std::env::current_dir() {
            Ok(dir) => dir,
            Err(_) => return Self::default(),
        };

        let project_type = detect_project_type(&cwd);
        let git_state = detect_git_state(&cwd);
        let recent_files = detect_recent_files(&cwd, 5);

        // Perform detailed analysis to detect errors and test failures
        let detailed_errors = detect_build_errors(&cwd, &project_type);
        let detailed_test_failures = detect_test_failures(&cwd, &project_type);

        let errors = detailed_errors.len();
        let test_failures = detailed_test_failures.len();

        Self {
            project_type,
            git_state,
            recent_files,
            errors,
            test_failures,
            detailed_errors,
            detailed_test_failures,
        }
    }

    pub fn suggested_commands(&self) -> Vec<&'static str> {
        let mut suggestions = Vec::new();

        // Always suggest basic commands
        suggestions.push("/ask");

        // Context-aware suggestions based on project state
        match self.project_type {
            ProjectType::Rust => {
                if !self.git_state.staged_files.is_empty() {
                    suggestions.push("/test"); // Run tests before commit for Rust projects
                }
                if self.errors > 0 {
                    suggestions.push("/fix");
                }
                suggestions.push("/review"); // Code review is always relevant for Rust
            },
            ProjectType::Node => {
                if !self.git_state.staged_files.is_empty() {
                    suggestions.push("/test"); // Run tests before commit for Node projects
                }
                if self.errors > 0 {
                    suggestions.push("/fix");
                }
                suggestions.push("/review");
            },
            ProjectType::Python => {
                if !self.git_state.staged_files.is_empty() {
                    suggestions.push("/test"); // Run tests before commit for Python projects
                }
                if self.errors > 0 {
                    suggestions.push("/fix");
                }
                suggestions.push("/review");
            },
            ProjectType::Unknown => {
                // For unknown project types, suggest general purpose commands
                if self.errors > 0 {
                    suggestions.push("/fix");
                }
                if self.test_failures > 0 {
                    suggestions.push("/test");
                }
                suggestions.push("/review");
            }
        }

        // Suggest commit when there are staged files
        if !self.git_state.staged_files.is_empty() {
            suggestions.push("/commit");
        }

        // Suggest doc generation if files exist but no documentation
        if !self.recent_files.is_empty() && self.has_code_files_without_docs() {
            suggestions.push("/doc");
        }

        suggestions
    }

    fn has_code_files_without_docs(&self) -> bool {
        self.recent_files.iter().any(|path| {
            if let Some(ext) = path.extension() {
                let ext_str = ext.to_string_lossy().to_lowercase();
                // Check if it's a code file but likely doesn't have documentation
                matches!(ext_str.as_str(), "rs" | "js" | "ts" | "py" | "go" | "cpp" | "h" | "java")
            } else {
                false
            }
        })
    }

    /// Get the most critical error that should be addressed first
    pub fn most_critical_error(&self) -> Option<&BuildError> {
        self.detailed_errors.iter()
            .filter(|error| matches!(error.severity, ErrorSeverity::Error))
            .min_by_key(|error| error.severity.clone())
    }

    /// Get the most recently failed test
    pub fn most_recent_test_failure(&self) -> Option<&TestFailure> {
        // For now, just return the first test failure, in a real implementation
        // we might want to sort by recency or severity
        self.detailed_test_failures.first()
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct GitState {
    pub branch: Option<String>,
    pub staged_files: Vec<String>,
    pub unstaged_files: Vec<String>,
    pub untracked_files: Vec<String>,
    pub conflicts: Vec<String>,
    pub last_commit: Option<DateTime<Utc>>,
    pub commits_ahead: usize,
    pub has_unpushed: bool,
    pub remote_branch: Option<String>,
}

impl Default for GitState {
    fn default() -> Self {
        Self {
            branch: None,
            staged_files: Vec::new(),
            unstaged_files: Vec::new(),
            untracked_files: Vec::new(),
            conflicts: Vec::new(),
            last_commit: None,
            commits_ahead: 0,
            has_unpushed: false,
            remote_branch: None,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct BuildError {
    pub file: String,
    pub line: usize,
    pub message: String,
    pub severity: ErrorSeverity,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum ErrorSeverity {
    Info,
    Warning,
    Error,
}

#[derive(Debug, Clone, Serialize)]
pub struct TestFailure {
    pub test_name: String,
    pub message: String,
    pub duration: Option<u64>, // Duration in milliseconds
}

#[derive(Debug, Clone, Serialize)]
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

    // Get current branch
    let branch = run_git_command(root, &["rev-parse", "--abbrev-ref", "HEAD"])
        .map(|s| s.trim().to_string())
        .ok();

    // Get detailed git status including all changes
    let status = run_git_command(root, &["status", "--porcelain", "--untracked-files=all"]).unwrap_or_default();

    let mut staged = Vec::new();
    let mut unstaged = Vec::new();
    let mut untracked = Vec::new();
    let mut conflicts = Vec::new();

    for line in status.lines() {
        if line.len() < 3 {
            continue;
        }
        let code = &line[..2];
        let path = line[3..].to_string();

        match code {
            "??" => untracked.push(path),
            "UU" | "AA" | "DD" | "AU" | "UA" | "UD" | "DU" => conflicts.push(path),
            _ => {
                // Check if it's staged (index changes are in first character)
                if code.chars().nth(0).map_or(false, |c| c != ' ' && c != '?') {
                    staged.push(path.clone());
                }
                // Check if it's unstaged (worktree changes are in second character)
                if code.chars().nth(1).map_or(false, |c| c != ' ' && c != '?') {
                    unstaged.push(path);
                }
            }
        }
    }

    // Get last commit information
    let last_commit = run_git_command(root, &["log", "-1", "--format=%cI"])
        .ok()
        .and_then(|ts| DateTime::parse_from_rfc3339(ts.trim()).ok())
        .map(|dt| dt.with_timezone(&Utc));

    // Get commit count since last release/origin
    let commits_ahead = run_git_command(root, &["rev-list", "--count", "HEAD", "--not", "origin/HEAD"])
        .ok()
        .and_then(|count| count.trim().parse::<usize>().ok())
        .unwrap_or(0);

    // Check if there are any unpushed commits
    let has_unpushed = commits_ahead > 0;

    // Get remote status
    let remote_branch = if branch.as_deref() == Some("HEAD") {
        // Detached HEAD state
        Some("HEAD (detached)".to_string())
    } else {
        branch.clone()
    };

    GitState {
        branch,
        staged_files: staged,
        unstaged_files: unstaged,
        untracked_files: untracked,
        conflicts,
        last_commit,
        commits_ahead,
        has_unpushed,
        remote_branch,
    }
}

// Enhanced function to detect build errors based on project type
fn detect_build_errors(root: &Path, project_type: &ProjectType) -> Vec<BuildError> {
    let mut errors = Vec::new();

    match project_type {
        ProjectType::Rust => {
            // Check for Rust compilation errors by looking for target/*/deps/*.d files or build artifacts
            let target_dir = root.join("target");
            if target_dir.exists() {
                // Look for compilation failure indicators
                if let Ok(entries) = std::fs::read_dir(&target_dir) {
                    for entry in entries.flatten() {
                        if let Some(name) = entry.file_name().to_str() {
                            // Rust compilation failures would leave traces in debug/release builds
                            if name == "debug" || name == "release" {
                                // Check for more specific error indicators
                                let build_script_outputs = target_dir.join(name).join("build");
                                if build_script_outputs.exists() {
                                    // Count build script failures
                                    if let Ok(build_entries) = std::fs::read_dir(&build_script_outputs) {
                                        for build_entry in build_entries.flatten() {
                                            let build_path = build_entry.path();
                                            if let Some(ext) = build_path.extension() {
                                                if ext == "out" || ext == "json" {
                                                    // Could potentially parse build logs for errors
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                // Check for Cargo.lock inconsistencies
                if root.join("Cargo.toml").exists() && !root.join("Cargo.lock").exists() {
                    errors.push(BuildError {
                        file: "Cargo.lock".to_string(),
                        line: 0,
                        message: "Cargo.lock file missing, dependencies may not be resolved".to_string(),
                        severity: ErrorSeverity::Warning,
                    });
                }
            }
        }
        ProjectType::Node => {
            // Check for Node.js error indicators
            if !root.join("node_modules").exists() && root.join("package.json").exists() {
                errors.push(BuildError {
                    file: "node_modules".to_string(),
                    line: 0,
                    message: "Dependencies not installed, run npm install or yarn install".to_string(),
                    severity: ErrorSeverity::Error,
                });
            }

            // Check for missing dependencies in package.json
            if let Ok(pkg_content) = std::fs::read_to_string(root.join("package.json")) {
                if pkg_content.contains("TODO") || pkg_content.contains("FIXME") {
                    errors.push(BuildError {
                        file: "package.json".to_string(),
                        line: 0,
                        message: "package.json contains TODO or FIXME markers".to_string(),
                        severity: ErrorSeverity::Info,
                    });
                }
            }
        }
        ProjectType::Python => {
            // Check for Python error indicators
            if !root.join(".venv").exists()
                && !root.join("venv").exists()
                && std::env::var("VIRTUAL_ENV").is_err() {

                if root.join("requirements.txt").exists() {
                    errors.push(BuildError {
                        file: "environment".to_string(),
                        line: 0,
                        message: "Virtual environment not activated, dependencies may be missing".to_string(),
                        severity: ErrorSeverity::Warning,
                    });
                }
            }
        }
        ProjectType::Unknown => {
            // For unknown project types, just check for common project setup files
        }
    }

    errors
}

// Enhanced function to detect test failures
fn detect_test_failures(root: &Path, project_type: &ProjectType) -> Vec<TestFailure> {
    let mut failures = Vec::new();

    match project_type {
        ProjectType::Rust => {
            // Check for Rust test artifacts (target/*/deps/*.out files with test failures)
            let target_dir = root.join("target");
            if target_dir.exists() {
                // Look for test output files that might indicate failures
                if let Ok(entries) = std::fs::read_dir(target_dir.join("tests")) {
                    for entry in entries.flatten() {
                        if let Some(name) = entry.file_name().to_str() {
                            if name.contains("failed") || name.contains("error") {
                                failures.push(TestFailure {
                                    test_name: name.to_string(),
                                    message: "Test artifact indicates failure".to_string(),
                                    duration: None,
                                });
                            }
                        }
                    }
                }

                // Check for test binaries that failed to run
                if let Ok(entries) = std::fs::read_dir(target_dir.join("debug").join("deps")) {
                    for entry in entries.flatten() {
                        if let Some(name) = entry.file_name().to_str() {
                            if name.starts_with("test_") && name.ends_with(".failed") {
                                failures.push(TestFailure {
                                    test_name: name.trim_end_matches(".failed").to_string(),
                                    message: "Test execution failed".to_string(),
                                    duration: None,
                                });
                            }
                        }
                    }
                }
            }
        }
        ProjectType::Node => {
            // Check for Node.js test artifacts
            let test_dirs = ["test", "__tests__", "tests"];
            for dir in &test_dirs {
                let test_dir = root.join(dir);
                if test_dir.exists() {
                    // Look for test reports or cache files that indicate failures
                    if test_dir.join("junit.xml").exists() || test_dir.join(".cache").exists() {
                        // Could parse test reports for specific failures
                    }
                }
            }
        }
        ProjectType::Python => {
            // Check for Python test artifacts
            let test_dirs = ["tests", "test"];
            for dir in &test_dirs {
                let test_dir = root.join(dir);
                if test_dir.exists() {
                    if test_dir.join(".pytest_cache").exists() {
                        // Could check cache for failed tests
                    }
                    if test_dir.join("test-reports").exists() {
                        // Could parse test reports
                    }
                }
            }
        }
        ProjectType::Unknown => {
            // No specific test failure detection for unknown project types
        }
    }

    failures
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

/// Analyze the project to detect build errors and test failures based on project type
fn analyze_project_state(root: &Path, project_type: &ProjectType) -> (usize, usize) {
    let build_errors = detect_build_errors(root, project_type);
    let test_failures = detect_test_failures(root, project_type);

    let errors = build_errors.len();
    let failures = test_failures.len();

    (errors, failures)
}
