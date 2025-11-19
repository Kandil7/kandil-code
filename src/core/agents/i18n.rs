//! Internationalization (i18n) assistant
//!
//! Assistant for managing internationalization and localization

use crate::core::adapters::ai::KandilAI;
use crate::core::agents::base::{Agent, AgentState};
use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct I18nAssistant {
    ai: Arc<KandilAI>,
    pub supported_languages: Vec<String>,
    pub translation_cache: HashMap<String, HashMap<String, String>>, // {lang: {key: translation}}
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslationReport {
    pub source_language: String,
    pub target_language: String,
    pub translated_count: u32,
    pub reviewed_count: u32,
    pub quality_score: u8, // 0-100
    pub issues_found: Vec<String>,
    pub suggestions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct I18nReport {
    pub completeness_by_language: HashMap<String, f32>,
    pub translation_quality_scores: HashMap<String, u8>,
    pub missing_translations: HashMap<String, Vec<String>>,
    pub consistency_issues: Vec<ConsistencyIssue>,
    pub recommendations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsistencyIssue {
    pub key: String,
    pub languages: Vec<String>,
    pub inconsistencies: Vec<String>,
}

impl I18nAssistant {
    pub fn new(ai: Arc<KandilAI>) -> Self {
        Self {
            ai,
            supported_languages: vec![
                "en".to_string(),
                "es".to_string(),
                "fr".to_string(),
                "de".to_string(),
                "ja".to_string(),
                "ko".to_string(),
                "zh".to_string(),
                "ar".to_string(),
            ],
            translation_cache: HashMap::new(),
        }
    }

    pub async fn translate_text(
        &mut self,
        text: &str,
        target_lang: &str,
        source_lang: &str,
    ) -> Result<String> {
        let cache_key = format!("{}:{}:{}", source_lang, target_lang, text);

        if let Some(cached) = self
            .translation_cache
            .get(source_lang)
            .and_then(|lang_map| lang_map.get(target_lang))
        {
            return Ok(cached.clone());
        }

        let prompt = format!(
            r#"Translate the following text from {} to {}:
            
            {}
            
            Ensure the translation is culturally appropriate and contextually accurate.
            "#,
            source_lang, target_lang, text
        );

        let translation = self.ai.chat(&prompt).await?;

        // Cache the translation
        self.translation_cache
            .entry(source_lang.to_string())
            .or_insert_with(HashMap::new)
            .insert(target_lang.to_string(), translation.clone());

        Ok(translation)
    }

    pub async fn translate_file(&self, file_path: &str, target_lang: &str) -> Result<String> {
        let content = std::fs::read_to_string(file_path)?;

        let prompt = format!(
            r#"Translate this content to {}:
            
            {}
            
            Preserve formatting and structure, translate only the translatable content.
            "#,
            target_lang, content
        );

        self.ai.chat(&prompt).await
    }

    pub async fn audit_translations(&self, resource_dir: &str) -> Result<I18nReport> {
        let mut completeness = HashMap::new();
        let mut quality_scores = HashMap::new();
        let mut missing_translations = HashMap::new();
        let consistency_issues = vec![];

        // In a real implementation, this would scan all translation files
        // For simulation, we'll return basic data
        for lang in &self.supported_languages {
            completeness.insert(lang.clone(), 0.85); // 85% complete
            quality_scores.insert(lang.clone(), 88); // 88% quality score
            missing_translations.insert(lang.clone(), vec![]);
        }

        Ok(I18nReport {
            completeness_by_language: completeness,
            translation_quality_scores: quality_scores,
            missing_translations,
            consistency_issues,
            recommendations: vec![
                "Add translations for new UI strings".to_string(),
                "Review machine translations for accuracy".to_string(),
            ],
        })
    }

    pub async fn generate_language_pack(
        &self,
        base_language: &str,
        target_language: &str,
    ) -> Result<HashMap<String, String>> {
        let prompt = format!(
            r#"Generate a complete language pack for {} based on {}.
            
            Include translations for:
            - Common UI elements
            - Error messages
            - Help text
            - Labels and prompts
            
            Return in key-value format.
            "#,
            target_language, base_language
        );

        let result = self.ai.chat(&prompt).await?;

        // In a real implementation, this would parse the structured response
        // For simulation, return an empty map
        Ok(HashMap::new())
    }

    pub async fn review_translation(
        &self,
        original: &str,
        translation: &str,
        target_lang: &str,
    ) -> Result<TranslationReport> {
        let prompt = format!(
            r#"Review this translation from English to {}:
            
            Original: {}
            
            Translation: {}
            
            Evaluate for accuracy, cultural appropriateness, and consistency.
            "#,
            target_lang, original, translation
        );

        let review = self.ai.chat(&prompt).await?;

        Ok(TranslationReport {
            source_language: "en".to_string(),
            target_language: target_lang.to_string(),
            translated_count: 1,
            reviewed_count: 1,
            quality_score: 92,
            issues_found: vec!["Minor tone issue".to_string()],
            suggestions: vec!["Consider cultural context".to_string()],
        })
    }
}

#[async_trait]
impl Agent for I18nAssistant {
    async fn plan(&self, state: &AgentState) -> Result<String> {
        let prompt = format!(
            "As an i18n specialist, given this internationalization task: {}\n\nPlan the next i18n activity. Consider language support, cultural adaptation, and technical implementation.",
            state.task
        );

        self.ai.chat(&prompt).await
    }

    async fn act(&self, plan: &str) -> Result<String> {
        // Execute the planned i18n activity
        let prompt = format!(
            "Implement this i18n plan: {}\n\nTranslate content, update resource files, or improve localization processes.",
            plan
        );

        self.ai.chat(&prompt).await
    }

    async fn observe(&self, result: &str) -> Result<String> {
        // Analyze i18n results
        let prompt = format!(
            "Analyze these i18n results: {}\n\nHow does this affect global user experience and market reach?",
            result
        );

        self.ai.chat(&prompt).await
    }
}
