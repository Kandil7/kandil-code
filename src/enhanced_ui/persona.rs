use std::collections::{VecDeque, HashMap};
use std::time::{Duration, Instant};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum DeveloperPersona {
    Expert,
    Maintainer,
    Learner,
    AutomationSpecialist,
    DebuggingSpecialist,
    CodeReviewSpecialist,
    DevOpsEngineer,
    Architect,
}

#[derive(Clone, Debug)]
pub struct PersonaProfile {
    pub persona: DeveloperPersona,
    pub greeting: String,
    pub guidance_level: GuidanceLevel,
    pub preferences: DeveloperPreferences,
    pub confidence: f64,  // Confidence in persona detection (0.0 to 1.0)
    pub detected_at: Instant,
}

#[derive(Clone, Debug)]
pub enum GuidanceLevel {
    Minimal,
    Suggestive,
    Detailed,
}

#[derive(Clone, Debug, Default)]
pub struct DeveloperPreferences {
    pub prefers_shortcut_commands: bool,
    pub prefers_verbose_output: bool,
    pub prefers_automated_tasks: bool,
    pub prefers_code_generation: bool,
    pub prefers_documentation: bool,
    pub prefers_visual_feedback: bool,
    pub command_patterns: Vec<String>,
    pub project_focus: ProjectFocus,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ProjectFocus {
    Frontend,
    Backend,
    FullStack,
    Infrastructure,
    Testing,
    Unknown,
}

impl DeveloperPersona {
    pub fn detect(history: &VecDeque<String>) -> (DeveloperPersona, f64) {
        if history.is_empty() {
            return (DeveloperPersona::Learner, 0.6); // Medium confidence for default
        }

        // Analyze various patterns in command history
        let shell_ratio = ratio(history, |cmd| !cmd.trim().starts_with('/'));
        let ai_ratio = ratio(history, |cmd| cmd.trim().starts_with("/ask"));
        let refactor_ratio = ratio(history, |cmd| cmd.trim().starts_with("/refactor"));
        let test_ratio = ratio(history, |cmd| cmd.trim().starts_with("/test"));
        let fix_ratio = ratio(history, |cmd| cmd.trim().starts_with("/fix"));
        let review_ratio = ratio(history, |cmd| cmd.trim().starts_with("/review"));
        let git_ratio = ratio(history, |cmd| cmd.contains("git") || cmd.contains("commit") || cmd.starts_with("git "));
        let build_ratio = ratio(history, |cmd| cmd.contains("build") || cmd.contains("run") || cmd.contains("cargo") || cmd.contains("npm"));
        let docker_ratio = ratio(history, |cmd| cmd.contains("docker") || cmd.contains("container"));

        // Calculate persona scores
        let mut scores: HashMap<DeveloperPersona, f64> = HashMap::new();

        // Expert: High command diversity, low help requests, complex commands
        let expert_score = {
            let command_diversity = calculate_command_diversity(history);
            let complexity_score = calculate_command_complexity(history);
            (command_diversity * 0.6 + complexity_score * 0.4).min(1.0)
        };

        // Maintainer: Mix of git, build, test, and review commands
        let maintainer_score = {
            let git_and_build = (git_ratio + build_ratio) * 0.4;
            let testing_and_review = (test_ratio + review_ratio) * 0.6;
            (git_and_build + testing_and_review).min(1.0)
        };

        // Learner: High AI interaction, frequent help commands
        let learner_score = {
            let ai_interaction = ai_ratio;
            let asking_questions = ai_ratio * 0.7 + (if history.len() < 5 { 0.3 } else { 0.0 });
            (ai_interaction + asking_questions).min(1.0) / 2.0
        };

        // Automation Specialist: Mostly shell commands, complex pipelines
        let automation_score = shell_ratio.max(ratio(history, |cmd| cmd.contains("|") || cmd.contains("&&") || cmd.contains(">")));

        // Debugging Specialist: High fix commands, error-focused patterns
        let debugging_score = fix_ratio;

        // Code Review Specialist: High review commands
        let review_score = review_ratio.max(refactor_ratio * 0.7);

        // DevOps Engineer: High docker, build, deployment patterns
        let devops_score = (docker_ratio + build_ratio + git_ratio) / 3.0;

        // Architect: Mix of all high-level commands (ask, review, refactor)
        let architect_score = (ai_ratio * 0.4 + review_ratio * 0.3 + refactor_ratio * 0.3).min(1.0);

        scores.insert(DeveloperPersona::Expert, expert_score);
        scores.insert(DeveloperPersona::Maintainer, maintainer_score);
        scores.insert(DeveloperPersona::Learner, learner_score);
        scores.insert(DeveloperPersona::AutomationSpecialist, automation_score);
        scores.insert(DeveloperPersona::DebuggingSpecialist, debugging_score);
        scores.insert(DeveloperPersona::CodeReviewSpecialist, review_score);
        scores.insert(DeveloperPersona::DevOpsEngineer, devops_score);
        scores.insert(DeveloperPersona::Architect, architect_score);

        // Find persona with highest score
        let (highest_persona, highest_score) = scores
            .into_iter()
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
            .unwrap_or((DeveloperPersona::Learner, 0.5));

        (highest_persona, highest_score)
    }
}

impl PersonaProfile {
    pub fn from_history(history: &VecDeque<String>) -> Self {
        let (persona, confidence) = DeveloperPersona::detect(history);

        let greeting = match persona {
            DeveloperPersona::Expert => {
                if confidence > 0.8 {
                    "🔧 Expert mode engaged. Minimal guidance, maximum efficiency.".to_string()
                } else {
                    "🔧 Developer patterns detected. Adjusting assistance level.".to_string()
                }
            },
            DeveloperPersona::Maintainer => {
                if confidence > 0.8 {
                    "🔄 Maintainer workflow detected. Optimizing for codebase stability.".to_string()
                } else {
                    "🔄 Development workflow identified. Maintaining code quality focus.".to_string()
                }
            },
            DeveloperPersona::Learner => {
                if confidence > 0.8 {
                    "📚 Learner mode activated. Enhanced explanations enabled.".to_string()
                } else {
                    "📚 Assisting with development questions and guidance.".to_string()
                }
            },
            DeveloperPersona::AutomationSpecialist => {
                if confidence > 0.8 {
                    "⚙️  Automation specialist detected. Streamlining repetitive tasks.".to_string()
                } else {
                    "⚙️  Task automation workflows identified. Optimizing pipelines.".to_string()
                }
            },
            DeveloperPersona::DebuggingSpecialist => {
                if confidence > 0.8 {
                    "🪲 Debugging specialist engaged. Focused on error resolution.".to_string()
                } else {
                    "🪲 Error investigation mode activated. Troubleshooting assistance ready.".to_string()
                }
            },
            DeveloperPersona::CodeReviewSpecialist => {
                if confidence > 0.8 {
                    "🔍 Code review expert detected. Quality analysis focused.".to_string()
                } else {
                    "🔍 Review and refactoring assistance ready.".to_string()
                }
            },
            DeveloperPersona::DevOpsEngineer => {
                if confidence > 0.8 {
                    "🏗️  DevOps engineer detected. Infrastructure and deployment focused.".to_string()
                } else {
                    "🏗️  Deployment and build optimization workflows identified.".to_string()
                }
            },
            DeveloperPersona::Architect => {
                if confidence > 0.8 {
                    "🏗️  Architect mode engaged. System design and architectural guidance.".to_string()
                } else {
                    "🏗️  High-level design and planning assistance enabled.".to_string()
                }
            },
        };

        let guidance_level = match &persona {
            DeveloperPersona::Expert | DeveloperPersona::AutomationSpecialist => GuidanceLevel::Minimal,
            DeveloperPersona::Maintainer | DeveloperPersona::DevOpsEngineer | DeveloperPersona::DebuggingSpecialist => GuidanceLevel::Suggestive,
            DeveloperPersona::Learner | DeveloperPersona::CodeReviewSpecialist | DeveloperPersona::Architect => GuidanceLevel::Detailed,
        };

        // Determine preferences based on detected persona
        let preferences = DeveloperPreferences::from_persona(&persona, history);

        Self {
            persona,
            greeting,
            guidance_level,
            preferences,
            confidence,
            detected_at: Instant::now(),
        }
    }

    /// Update profile based on new command
    pub fn update_with_command(&mut self, command: &str) {
        // Update preferences based on the command
        self.preferences.update_from_command(command);

        // Update confidence based on consistency
        self.confidence = self.confidence * 0.95; // Slightly decay confidence over time
    }
}

impl DeveloperPreferences {
    fn from_persona(persona: &DeveloperPersona, history: &VecDeque<String>) -> Self {
        let mut prefs = DeveloperPreferences::default();

        prefs.prefers_shortcut_commands = matches!(persona,
            DeveloperPersona::Expert | DeveloperPersona::AutomationSpecialist);

        prefs.prefers_verbose_output = matches!(persona,
            DeveloperPersona::Learner | DeveloperPersona::DebuggingSpecialist);

        prefs.prefers_automated_tasks = matches!(persona,
            DeveloperPersona::AutomationSpecialist | DeveloperPersona::DevOpsEngineer);

        prefs.prefers_code_generation = matches!(persona,
            DeveloperPersona::Learner | DeveloperPersona::Architect);

        prefs.prefers_documentation = matches!(persona,
            DeveloperPersona::Maintainer | DeveloperPersona::Architect);

        prefs.prefers_visual_feedback = matches!(persona,
            DeveloperPersona::Learner | DeveloperPersona::CodeReviewSpecialist);

        // Set project focus based on common commands
        let mut project_focus_counts = HashMap::new();
        for cmd in history {
            if cmd.contains("css") || cmd.contains("html") || cmd.contains("frontend") ||
               cmd.contains("ui") || cmd.contains("react") || cmd.contains("vue") {
                *project_focus_counts.entry(ProjectFocus::Frontend).or_insert(0) += 1;
            } else if cmd.contains("api") || cmd.contains("backend") || cmd.contains("server") ||
                      cmd.contains("database") || cmd.contains("sql") || cmd.contains("rest") {
                *project_focus_counts.entry(ProjectFocus::Backend).or_insert(0) += 1;
            } else if cmd.contains("docker") || cmd.contains("k8s") || cmd.contains("deploy") ||
                      cmd.contains("devops") || cmd.contains("ci") {
                *project_focus_counts.entry(ProjectFocus::Infrastructure).or_insert(0) += 1;
            } else if cmd.contains("test") || cmd.contains("spec") || cmd.contains("unit") {
                *project_focus_counts.entry(ProjectFocus::Testing).or_insert(0) += 1;
            }
        }

        // Set the most common focus
        prefs.project_focus = project_focus_counts
            .into_iter()
            .max_by_key(|(_, count)| *count)
            .map(|(focus, _)| focus)
            .unwrap_or(ProjectFocus::Unknown);

        prefs
    }

    fn update_from_command(&mut self, command: &str) {
        // Update patterns
        if !self.command_patterns.contains(&command.to_string()) {
            self.command_patterns.push(command.to_string());
            if self.command_patterns.len() > 10 {  // Keep only recent patterns
                self.command_patterns.remove(0);
            }
        }
    }
}

fn ratio(history: &VecDeque<String>, predicate: impl Fn(&String) -> bool) -> f32 {
    if history.is_empty() {
        return 0.0;
    }
    let count = history.iter().filter(|cmd| predicate(cmd)).count() as f32;
    count / history.len() as f32
}

/// Calculate command diversity (how varied the commands are)
fn calculate_command_diversity(history: &VecDeque<String>) -> f64 {
    if history.is_empty() {
        return 0.0;
    }

    let mut unique_commands = std::collections::HashSet::new();
    for cmd in history {
        // Extract command base (first word or command type)
        let base_cmd = cmd.split_whitespace().next().unwrap_or("").to_string();
        unique_commands.insert(base_cmd);
    }

    // Diversity score: unique commands / total commands, scaled to 0-1
    (unique_commands.len() as f64 / history.len() as f64).min(1.0)
}

/// Calculate command complexity (length, presence of options, etc.)
fn calculate_command_complexity(history: &VecDeque<String>) -> f64 {
    if history.is_empty() {
        return 0.0;
    }

    let avg_complexity: f64 = history
        .iter()
        .map(|cmd| {
            // More complex commands have more whitespace (options), special chars, etc.
            let word_count = cmd.split_whitespace().count();
            let special_chars = cmd.chars().filter(|c| matches!(c, '|' | '&' | '>' | '<' | ';')).count();

            // Normalize to 0-1 scale
            ((word_count + special_chars) as f64 / 10.0).min(1.0)
        })
        .sum::<f64>() / history.len() as f64;

    avg_complexity
}
