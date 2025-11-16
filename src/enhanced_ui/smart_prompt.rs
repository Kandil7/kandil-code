use std::time::Duration;

/// Helper utilities for interactive previews and confirmations.
pub struct SmartPrompt;

impl SmartPrompt {
    pub fn confirm(prompt: &str) -> bool {
        println!("🔒 Confirmation required: {}", prompt);
        true
    }

    /// Show a formatted diff preview
    pub fn show_diff(diff: &str) -> bool {
        let lines: Vec<&str> = diff.lines().collect();
        println!("\n📝 Diff Preview ({} lines):", lines.len());
        println!("{}", format_diff_preview(diff));
        println!("-----------------------------");
        true
    }

    /// Preview a command with detailed execution plan
    pub fn preview_actions(command: &str, actions: &[&str]) -> String {
        let action_text = if actions.is_empty() {
            "No actions queued".to_string()
        } else {
            actions
                .iter()
                .enumerate()
                .map(|(idx, action)| format!("  {}. {}", idx + 1, action))
                .collect::<Vec<_>>()
                .join("\n")
        };
        format!("{} preview:\n{}", command, action_text)
    }

    /// Create a comprehensive pipeline summary with stages and estimated time
    pub fn pipeline_summary(stages: &[String]) -> String {
        format!("🔄 Executing pipeline:\n{}", stages.join(" → "))
    }

    /// Create an enhanced pipeline summary with detailed stage information
    pub fn pipeline_summary_detailed(stages: &[PipelineStage]) -> String {
        let mut result = "🔄 Pipeline Execution Plan:\n".to_string();
        for (idx, stage) in stages.iter().enumerate() {
            result.push_str(&format!("\n{}. {}:\n", idx + 1, stage.name));
            result.push_str(&format!("   Command: {}\n", stage.command));
            if let Some(estimated_duration) = stage.estimated_duration {
                result.push_str(&format!("   Estimated time: {}s\n", estimated_duration.as_secs()));
            }
            if let Some(description) = &stage.description {
                result.push_str(&format!("   Description: {}\n", description));
            }
        }
        result
    }

    /// Create a diff preview between two code versions
    pub fn diff_preview(original: &str, modified: &str) -> String {
        let mut diff = String::new();
        let orig_lines: Vec<&str> = original.lines().collect();
        let mod_lines: Vec<&str> = modified.lines().collect();

        // For a simple diff preview, show line-by-line comparison
        let max_lines = orig_lines.len().max(mod_lines.len());

        for i in 0..max_lines {
            let orig_line = orig_lines.get(i).unwrap_or(&"");
            let mod_line = mod_lines.get(i).unwrap_or(&"");

            if orig_line != mod_line {
                if i < orig_lines.len() && i < mod_lines.len() {
                    diff.push_str(&format!("- {}\n+ {}\n", orig_line, mod_line));
                } else if i >= mod_lines.len() {
                    diff.push_str(&format!("- {}\n", orig_line)); // Removed line
                } else {
                    diff.push_str(&format!("+ {}\n", mod_line)); // Added line
                }
            }
        }

        diff
    }

    /// Show a side-by-side diff preview
    pub fn diff_preview_side_by_side(original: &str, modified: &str) -> String {
        let orig_lines: Vec<&str> = original.lines().collect();
        let mod_lines: Vec<&str> = modified.lines().collect();

        let max_lines = orig_lines.len().max(mod_lines.len());
        let mut result = String::new();

        result.push_str("  ORIGINAL             |  MODIFIED\n");
        result.push_str("  ---------------------|---------------------\n");

        for i in 0..max_lines {
            let orig_line = orig_lines.get(i).unwrap_or(&"");
            let mod_line = mod_lines.get(i).unwrap_or(&"");

            let marker = if orig_line != mod_line {
                ">>"
            } else {
                "  "
            };

            result.push_str(&format!(
                "{} {:>20} | {} {}\n",
                marker,
                if orig_line.len() > 18 { &orig_line[..18] } else { orig_line },
                marker,
                if mod_line.len() > 18 { &mod_line[..18] } else { mod_line }
            ));
        }

        result
    }

    pub fn background_job_message(label: &str, eta: Duration) -> String {
        format!(
            "Started background job '{}' (est. {}s)",
            label,
            eta.as_secs()
        )
    }

    /// Create a progress bar for long-running operations
    pub fn progress_bar(completed: usize, total: usize, label: &str) -> String {
        let percent = if total > 0 { (completed as f64 / total as f64 * 100.0).round() as u8 } else { 0 };
        let completed_chars = (percent as usize * 30) / 100;
        let bar: String = std::iter::repeat('█')
            .take(completed_chars)
            .chain(std::iter::repeat('░').take(30 - completed_chars))
            .collect();

        format!("{} [{}] {}% ({}/{})", label, bar, percent, completed, total)
    }
}

/// Format diff with highlighting for better readability
fn format_diff_preview(diff: &str) -> String {
    let mut formatted = String::new();
    for line in diff.lines() {
        if line.starts_with('+') {
            formatted.push_str(&format!("🟢 {}\n", line));
        } else if line.starts_with('-') {
            formatted.push_str(&format!("🔴 {}\n", line));
        } else if line.starts_with('@@') {
            formatted.push_str(&format!("🔵 {}\n", line));
        } else {
            formatted.push_str(&format!("   {}\n", line));
        }
    }
    formatted
}

/// Represents a stage in a pipeline
#[derive(Debug, Clone)]
pub struct PipelineStage {
    pub name: String,
    pub command: String,
    pub description: Option<String>,
    pub estimated_duration: Option<Duration>,
    pub dependencies: Vec<String>,
}

impl PipelineStage {
    pub fn new(name: &str, command: &str) -> Self {
        Self {
            name: name.to_string(),
            command: command.to_string(),
            description: None,
            estimated_duration: None,
            dependencies: Vec::new(),
        }
    }

    pub fn with_description(mut self, description: &str) -> Self {
        self.description = Some(description.to_string());
        self
    }

    pub fn with_duration(mut self, duration: Duration) -> Self {
        self.estimated_duration = Some(duration);
        self
    }

    pub fn with_dependencies(mut self, deps: Vec<String>) -> Self {
        self.dependencies = deps;
        self
    }
}
