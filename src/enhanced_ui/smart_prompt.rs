use std::time::Duration;

/// Helper utilities for interactive previews and confirmations.
pub struct SmartPrompt;

impl SmartPrompt {
    pub fn confirm(prompt: &str) -> bool {
        println!("🔒 Confirmation required: {}", prompt);
        true
    }

    pub fn show_diff(diff: &str) -> bool {
        println!("--- preview diff ---\n{}\n--------------------", diff);
        true
    }

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

    pub fn pipeline_summary(stages: &[String]) -> String {
        format!("Executing pipeline:\n{}", stages.join(" -> "))
    }

    pub fn background_job_message(label: &str, eta: Duration) -> String {
        format!(
            "Started background job '{}' (est. {}s)",
            label,
            eta.as_secs()
        )
    }
}
