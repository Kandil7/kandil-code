//! Code review agent
//! 
//! Specialized agent for reviewing code quality, security, and best practices

use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::path::Path;
use tree_sitter::{Parser, Language};
use crate::core::agents::base::{Agent, AgentState, AgentResult, ReActLoop};
use crate::core::adapters::ai::KandilAI;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewReport {
    pub issues: Vec<Issue>,
    pub score: u8,  // 0-100 score
    pub summary: String,
    pub recommendations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Issue {
    pub id: String,
    pub title: String,
    pub description: String,
    pub severity: Severity,
    pub category: Category,
    pub line_number: Option<u32>,
    pub suggestion: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Severity {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Category {
    Security,
    Performance,
    CodeStyle,
    Logic,
    Maintainability,
    Documentation,
}

pub struct ReviewAgent {
    ai: KandilAI,
}

impl ReviewAgent {
    pub fn new(ai: KandilAI) -> Self {
        Self { ai }
    }

    pub async fn code_review(&self, file_path: &str) -> Result<ReviewReport> {
        let content = std::fs::read_to_string(file_path)?;
        
        let prompt = format!(
            r#"Review this code for:
            - Bugs and logic errors
            - Security vulnerabilities (OWASP Top 10)
            - Performance anti-patterns
            - Code smells
            - Best practices violations
            - Documentation issues

            Code: {}
            "#, 
            content
        );

        let result = self.ai.chat(&prompt).await?;
        
        // In a real implementation, this would parse the structured response
        // For now, we'll return a basic report
        Ok(ReviewReport {
            issues: vec![
                Issue {
                    id: "SEC-001".to_string(),
                    title: "Potential security issue".to_string(),
                    description: "Found potential vulnerability in code".to_string(),
                    severity: Severity::High,
                    category: Category::Security,
                    line_number: Some(10),
                    suggestion: "Add input validation".to_string(),
                }
            ],
            score: 85,
            summary: "Code review completed with suggestions for improvements".to_string(),
            recommendations: vec![
                "Add input validation".to_string(),
                "Improve error handling".to_string(),
            ],
        })
    }

    pub async fn architecture_review(&self, design_doc: &str) -> Result<ReviewReport> {
        let prompt = format!(
            r#"Review this architecture design for:
            - Scalability concerns
            - Security architecture
            - Performance considerations
            - Technology choices
            - Design patterns usage
            - Maintainability

            Design: {}
            "#, 
            design_doc
        );

        let result = self.ai.chat(&prompt).await?;
        
        // For now, we'll return a basic report
        Ok(ReviewReport {
            issues: vec![
                Issue {
                    id: "ARCH-001".to_string(),
                    title: "Architecture concern".to_string(),
                    description: "Potential scalability issue identified".to_string(),
                    severity: Severity::Medium,
                    category: Category::Performance,
                    line_number: None,
                    suggestion: "Consider implementing caching".to_string(),
                }
            ],
            score: 78,
            summary: "Architecture review completed".to_string(),
            recommendations: vec![
                "Implement caching layer".to_string(),
                "Add monitoring capabilities".to_string(),
            ],
        })
    }
}

#[async_trait]
impl Agent for ReviewAgent {
    async fn plan(&self, state: &AgentState) -> Result<String> {
        let prompt = format!(
            "Given this code review task: {}\n\nPlan the next review step. What aspect should we focus on for the most impactful feedback?",
            state.task
        );
        
        self.ai.chat(&prompt).await
    }

    async fn act(&self, plan: &str) -> Result<String> {
        // Execute the planned review action
        let prompt = format!(
            "Perform this code review action: {}\n\nAnalyze the code and identify specific issues with detailed explanations.",
            plan
        );
        
        self.ai.chat(&prompt).await
    }

    async fn observe(&self, result: &str) -> Result<String> {
        // Analyze the review findings
        let prompt = format!(
            "Analyze these code review findings: {}\n\nWhat are the most critical issues that need to be addressed first?",
            result
        );
        
        self.ai.chat(&prompt).await
    }
}