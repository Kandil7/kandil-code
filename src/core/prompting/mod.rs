use crate::models::registry::{ModelProfile, ProviderKind, UniversalModelRegistry};
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub enum PromptIntent {
    Conversation,
    Coding,
    Planning,
    Architecture,
    Testing,
    Analysis,
}

#[derive(Debug, Clone, Serialize)]
pub struct RoutedPrompt {
    pub intent: PromptIntent,
    pub provider: String,
    pub model: String,
    pub explanation: String,
}

pub struct PromptRouter<'a> {
    registry: &'a UniversalModelRegistry,
}

impl<'a> PromptRouter<'a> {
    pub fn new() -> Self {
        Self {
            registry: UniversalModelRegistry::global(),
        }
    }

    pub fn route_message(
        &self,
        message: &str,
        default_provider: &str,
        default_model: &str,
    ) -> RoutedPrompt {
        let intent = infer_intent(message);
        self.route_for_intent(intent, default_provider, default_model)
    }

    pub fn route_for_intent(
        &self,
        intent: PromptIntent,
        default_provider: &str,
        default_model: &str,
    ) -> RoutedPrompt {
        let (provider_hint, model_hint) = intent_defaults(&intent);
        self.finalize_route(
            intent,
            provider_hint.unwrap_or(default_provider),
            model_hint.unwrap_or(default_model),
            default_provider,
            default_model,
        )
    }

    fn finalize_route(
        &self,
        intent: PromptIntent,
        provider_candidate: &str,
        model_candidate: &str,
        default_provider: &str,
        default_model: &str,
    ) -> RoutedPrompt {
        if let Some(profile) = self.registry.get_profile(model_candidate) {
            let explanation = format!(
                "Matched registry profile '{}' for {:?}",
                profile.name, intent
            );
            return RoutedPrompt {
                intent,
                provider: provider_from_profile(&profile),
                model: profile.name.clone(),
                explanation,
            };
        }

        let explanation = format!(
            "Using fallback {}::{} for {:?}",
            provider_candidate, model_candidate, intent
        );

        RoutedPrompt {
            intent,
            provider: if provider_candidate.is_empty() {
                default_provider.to_string()
            } else {
                provider_candidate.to_string()
            },
            model: if model_candidate.is_empty() {
                default_model.to_string()
            } else {
                model_candidate.to_string()
            },
            explanation,
        }
    }
}

fn infer_intent(message: &str) -> PromptIntent {
    let lower = message.to_lowercase();

    // Check for specific slash commands first
    if lower.starts_with("/ref") {
        PromptIntent::Coding
    } else if lower.starts_with("/test") {
        PromptIntent::Testing
    } else if lower.starts_with("/fix") {
        PromptIntent::Analysis
    } else if lower.starts_with("/review") {
        PromptIntent::Analysis
    } else if contains_any(&lower, &["refactor", "code", "function", "bugfix"]) {
        PromptIntent::Coding
    } else if contains_any(&lower, &["plan", "roadmap", "requirement", "story"]) {
        PromptIntent::Planning
    } else if contains_any(&lower, &["architecture", "design", "scalable", "pattern"]) {
        PromptIntent::Architecture
    } else if contains_any(&lower, &["test", "qa", "edge case", "verify"]) {
        PromptIntent::Testing
    } else if contains_any(&lower, &["analyze", "metrics", "compare", "benchmark", "review", "fix"]) {
        PromptIntent::Analysis
    } else {
        PromptIntent::Conversation
    }
}

fn intent_defaults(intent: &PromptIntent) -> (Option<&'static str>, Option<&'static str>) {
    match intent {
        PromptIntent::Coding => (Some("ollama"), Some("qwen2.5-coder-7b-q4")),
        PromptIntent::Planning => (Some("ollama"), Some("qwen2.5-coder-3b-q4")),
        PromptIntent::Architecture => (Some("ollama"), Some("llama3.1-70b-q4")),
        PromptIntent::Testing => (Some("ollama"), Some("qwen2.5-coder-3b-q4")),
        PromptIntent::Analysis => (Some("ollama"), Some("qwen2.5-coder-14b-q4")),
        PromptIntent::Conversation => (None, None),
    }
}

fn provider_from_profile(profile: &ModelProfile) -> String {
    match &profile.provider {
        ProviderKind::Ollama => "ollama",
        ProviderKind::QwenCloud => "qwen",
        ProviderKind::Anthropic => "anthropic",
        ProviderKind::Gemini => "gemini",
        ProviderKind::LocalBridge => "local",
        ProviderKind::Custom(name) => name,
    }
    .to_string()
}

fn contains_any(text: &str, needles: &[&str]) -> bool {
    needles.iter().any(|needle| text.contains(needle))
}
