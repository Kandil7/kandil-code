//! Code generation agent
//! 
//! Specialized agent for generating production-ready code from design documents

use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::path::Path;
use crate::core::agents::base::{Agent, AgentState, AgentResult, ReActLoop};
use crate::core::adapters::ai::KandilAI;
use crate::utils::templates::TemplateEngine;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeOutput {
    pub files: Vec<CodeFile>,
    pub language: String,
    pub architecture: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeFile {
    pub path: String,
    pub content: String,
    pub description: String,
}

pub struct CodeAgent {
    ai: KandilAI,
    template_engine: TemplateEngine,
}

impl CodeAgent {
    pub fn new(ai: KandilAI) -> Result<Self> {
        Ok(Self {
            ai,
            template_engine: TemplateEngine::new(),
        })
    }

    pub async fn generate_code(&self, design_doc: &str, language: &str) -> Result<CodeOutput> {
        let task = format!(
            "Generate production-ready {} code from this design.\n\nDesign:\n{}\n\nFollow:\n1. Map components to files/folders\n2. Generate data models\n3. Implement business logic\n4. Add error handling\n5. Include logging\n6. Follow language best practices\n\nReturn structured plan with file paths and code blocks.",
            language.to_uppercase(),
            design_doc
        );

        let loop_engine = ReActLoop::new(5);
        let result = loop_engine.run(self, &task).await?;

        self.parse_and_write_code(&result.final_answer, language).await
    }

    async fn parse_and_write_code(&self, response: &str, language: &str) -> Result<CodeOutput> {
        // In a real implementation, this would properly parse the AI's structured response
        // For now, we'll create a basic output
        Ok(CodeOutput {
            files: vec![
                CodeFile {
                    path: match language {
                        "rust" => "src/main.rs".to_string(),
                        "python" => "app/main.py".to_string(),
                        "javascript" => "src/index.js".to_string(),
                        "dart" => "lib/main.dart".to_string(),
                        _ => "main.code".to_string(),
                    },
                    content: format!("// Generated {} code based on design\n// Content would be implemented in full version", language),
                    description: format!("Main {} file generated from design", language),
                }
            ],
            language: language.to_string(),
            architecture: "Generated from design document".to_string(),
        })
    }

    pub async fn generate_for_project(&self, project_path: &str, design_doc: &str, language: &str) -> Result<()> {
        let output = self.generate_code(design_doc, language).await?;
        
        // Write files to project
        for file in output.files {
            let file_path = Path::new(project_path).join(&file.path);
            
            if let Some(parent) = file_path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            
            std::fs::write(&file_path, &file.content)?;
        }
        
        Ok(())
    }
}

#[async_trait]
impl Agent for CodeAgent {
    async fn plan(&self, state: &AgentState) -> Result<String> {
        let prompt = format!(
            "Given this design document and current implementation state: {}\n\nPlan the next step for code generation. Which component should we generate first? Consider dependencies and proper layering.",
            state.task
        );
        
        self.ai.chat(&prompt).await
    }

    async fn act(&self, plan: &str) -> Result<String> {
        // Generate code based on the plan
        let prompt = format!(
            "Implement the following code generation plan: {}\n\nGenerate actual code in the appropriate programming language, with proper structure, error handling, and documentation.",
            plan
        );
        
        self.ai.chat(&prompt).await
    }

    async fn observe(&self, result: &str) -> Result<String> {
        // Evaluate the generated code
        let prompt = format!(
            "Analyze this generated code: {}\n\nIs it following best practices? Does it match the design? What improvements are needed?",
            result
        );
        
        self.ai.chat(&prompt).await
    }
}