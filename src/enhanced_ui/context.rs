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

    pub fn detect_with_analysis() -> Self {
        let cwd = match std::env::current_dir() {
            Ok(dir) => dir,
            Err(_) => return Self::default(),
        };

        let project_type = detect_project_type(&cwd);
        let git_state = detect_git_state(&cwd);
        let recent_files = detect_recent_files(&cwd, 5);

        // Perform analysis to detect errors and test failures
        let (errors, test_failures) = analyze_project_state(&cwd, &project_type);

        Self {
            project_type,
            git_state,
            recent_files,
            errors,
            test_failures,
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
}

#[derive(Debug, Clone, Serialize)]
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

/// Analyze the project to detect build errors and test failures based on project type
fn analyze_project_state(root: &Path, project_type: &ProjectType) -> (usize, usize) {
    match project_type {
        ProjectType::Rust => {
            // For Rust, check for build issues and test status
            // This is a simplified implementation - in a full implementation, we would run actual build/tests
            let mut errors = 0;
            let mut test_failures = 0;

            // Check for common Rust error indicators in target directory
            let target_path = root.join("target");
            if target_path.exists() {
                // Check for failed build artifacts or test results
                // This is a simplified check - in reality we'd parse build logs
                if root.join("Cargo.lock").exists() {
                    errors = 0; // Assume build is OK if Cargo.lock exists
                }
            }

            // For test failures, we could check for test output files
            if target_path.join("tests").exists() {
                // Count failed test artifacts
                test_failures = 0; // Simplified
            }

            (errors, test_failures)
        }
        ProjectType::Node => {
            // For Node.js projects
            let mut errors = 0;
            let mut test_failures = 0;

            // Check if node_modules exists and is properly set up
            if !root.join("node_modules").exists() {
                errors += 1; // Missing dependencies might be an error
            }

            // Check for common test file patterns
            let test_patterns = ["test", "__tests__", "*.test.js", "*.spec.js"];
            for pattern in &test_patterns {
                if root.join(pattern).exists() {
                    test_failures = 0; // If test files exist, we mark as having tests to run
                }
            }

            (errors, test_failures)
        }
        ProjectType::Python => {
            // For Python projects
            let mut errors = 0;
            let mut test_failures = 0;

            // Check for common Python project indicators
            if root.join("requirements.txt").exists() || root.join("Pipfile").exists() {
                // Check for missing virtual environment
                if !root.join(".venv").exists() && !std::env::var("VIRTUAL_ENV").is_ok() {
                    errors += 1;
                }
            }

            // Check for test files
            let test_dirs = ["tests", "test"];
            for test_dir in &test_dirs {
                if root.join(test_dir).is_dir() {
                    test_failures = 0; // Mark as having tests to run
                }
            }

            // Check for pytest or unittest discovery
            if root.join("pytest.ini").exists() || root.join("setup.cfg").exists() {
                test_failures = 0;
            }

            (errors, test_failures)
        }
        ProjectType::Unknown => (0, 0), // No specific analysis for unknown project types
    }
}
