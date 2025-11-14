//! Requirements elicitation agent
//! 
//! Specialized agent for gathering and documenting software requirements

use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use crate::core::agents::base::{Agent, AgentState, ReActLoop};
use crate::core::adapters::ai::KandilAI;
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequirementsDocument {
    pub project_name: String,
    pub description: String,
    pub functional_requirements: Vec<Requirement>,
    pub non_functional_requirements: Vec<Requirement>,
    pub actors: Vec<Actor>,
    pub constraints: Vec<String>,
    pub assumptions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Requirement {
    pub id: String,
    pub title: String,
    pub description: String,
    pub priority: Priority,
    pub category: Category,
    pub acceptance_criteria: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Priority {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Category {
    Functional,
    NonFunctional,
    Business,
    User,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Actor {
    pub name: String,
    pub description: String,
    pub responsibilities: Vec<String>,
}

pub struct RequirementsAgent {
    ai: Arc<KandilAI>,
}

impl RequirementsAgent {
    pub fn new(ai: Arc<KandilAI>) -> Self {
        Self { ai }
    }

    pub async fn generate_requirements_document(&self, project_description: &str) -> Result<RequirementsDocument> {
        let loop_engine = ReActLoop::new(5);
        let task = format!(
            "As a Business Analyst, elicit requirements for this project: {}.\n\nFollow these steps:\n1. Identify the main actors/users\n2. List functional requirements\n3. List non-functional requirements\n4. Identify constraints and assumptions\n5. Prioritize requirements\n\nFormat the response as a structured requirements document.", 
            project_description
        );
        
        let result = loop_engine.run(self, &task).await?;
        
        // For now, we'll create a basic document structure from the AI response
        // In a real implementation, we would properly parse the structured response
        Ok(RequirementsDocument {
            project_name: "Project".to_string(),
            description: project_description.to_string(),
            functional_requirements: vec![],
            non_functional_requirements: vec![],
            actors: vec![],
            constraints: vec![],
            assumptions: vec![],
        })
    }
}

#[async_trait]
impl Agent for RequirementsAgent {
    async fn plan(&self, state: &AgentState) -> Result<String> {
        let prompt = format!(
            "Given this task: {}\n\nCurrent state: Step {}/{}\n\nPlan the next step to gather requirements. Be specific about what information to collect.",
            state.task, state.current_step + 1, state.max_steps
        );
        
        self.ai.chat(&prompt).await
    }

    async fn act(&self, plan: &str) -> Result<String> {
        // In a real implementation, this would involve:
        // - Asking stakeholders specific questions
        // - Analyzing existing documentation
        // - Performing domain research
        // For simulation, we'll use the AI to generate a response based on the plan
        
        let prompt = format!(
            "Act on this plan for requirements elicitation: {}\n\nGenerate specific questions to ask stakeholders or analysis to perform.",
            plan
        );
        
        self.ai.chat(&prompt).await
    }

    async fn observe(&self, result: &str) -> Result<String> {
        // Analyze the result and determine next steps
        let prompt = format!(
            "Analyze this requirements gathering result: {}\n\nWhat does this tell us about the project requirements? What should we focus on next?",
            result
        );
        
        self.ai.chat(&prompt).await
    }
}