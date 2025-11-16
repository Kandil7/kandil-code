use std::collections::VecDeque;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DeveloperPersona {
    Expert,
    Maintainer,
    Learner,
    AutomationSpecialist,
}

#[derive(Clone, Debug)]
pub struct PersonaProfile {
    pub persona: DeveloperPersona,
    pub greeting: &'static str,
    pub guidance_level: GuidanceLevel,
}

#[derive(Clone, Debug)]
pub enum GuidanceLevel {
    Minimal,
    Suggestive,
    Detailed,
}

impl DeveloperPersona {
    pub fn detect(history: &VecDeque<String>) -> DeveloperPersona {
        if history.is_empty() {
            return DeveloperPersona::Learner;
        }

        let shell_ratio = ratio(history, |cmd| !cmd.trim().starts_with('/'));
        let ai_ratio = ratio(history, |cmd| cmd.trim().starts_with("/ask"));

        if shell_ratio > 0.7 {
            DeveloperPersona::AutomationSpecialist
        } else if ai_ratio > 0.4 {
            DeveloperPersona::Learner
        } else if history.len() > 10 {
            DeveloperPersona::Maintainer
        } else {
            DeveloperPersona::Expert
        }
    }
}

impl PersonaProfile {
    pub fn from_history(history: &VecDeque<String>) -> Self {
        let persona = DeveloperPersona::detect(history);
        match persona {
            DeveloperPersona::Expert => Self {
                persona,
                greeting: "Expert mode engaged. Noise minimized.",
                guidance_level: GuidanceLevel::Minimal,
            },
            DeveloperPersona::Maintainer => Self {
                persona,
                greeting: "Maintainer workflow detected. Highlighting recent files.",
                guidance_level: GuidanceLevel::Suggestive,
            },
            DeveloperPersona::Learner => Self {
                persona,
                greeting: "Learner persona detected. Enabling guided hints.",
                guidance_level: GuidanceLevel::Detailed,
            },
            DeveloperPersona::AutomationSpecialist => Self {
                persona,
                greeting: "Automation persona detected. Showcasing pipelines.",
                guidance_level: GuidanceLevel::Suggestive,
            },
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
