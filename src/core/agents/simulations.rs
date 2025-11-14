//! Professional role simulations
//! 
//! Agents that simulate PM (Project Manager) and BA (Business Analyst) roles

use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use crate::core::agents::base::{Agent, AgentState};
use crate::core::adapters::ai::KandilAI;
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SprintPlan {
    pub sprint_number: u32,
    pub goals: Vec<String>,
    pub user_stories: Vec<UserStory>,
    pub tasks: Vec<Task>,
    pub estimated_days: u32,
    pub start_date: String,
    pub end_date: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserStory {
    pub id: String,
    pub title: String,
    pub description: String,
    pub acceptance_criteria: Vec<String>,
    pub story_points: u32,
    pub priority: Priority,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub title: String,
    pub description: String,
    pub assignee: Option<String>,
    pub status: TaskStatus,
    pub story_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Priority {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskStatus {
    Todo,
    InProgress,
    Review,
    Done,
}

pub struct ProjectManagerSimulation {
    ai: Arc<KandilAI>,
}

impl ProjectManagerSimulation {
    pub fn new(ai: Arc<KandilAI>) -> Self {
        Self { ai }
    }

    pub async fn plan_sprint(&self, project_name: &str, duration_weeks: u32) -> Result<SprintPlan> {
        let prompt = format!(
            "As a Project Manager, plan a {}-week sprint for the {} project.\n\nCreate a detailed sprint plan with goals, user stories, tasks, and timeline.",
            duration_weeks, project_name
        );
        
        let result = self.ai.chat(&prompt).await?;
        
        // In a real implementation, this would parse the structured response
        // For now, returning a basic structure
        Ok(SprintPlan {
            sprint_number: 1,
            goals: vec!["Complete initial development".to_string()],
            user_stories: vec![],
            tasks: vec![],
            estimated_days: duration_weeks * 5, // Working days
            start_date: "2024-01-01".to_string(),
            end_date: "2024-01-28".to_string(),
        })
    }

    pub async fn run_retrospective(&self, sprint_number: u32) -> Result<String> {
        let prompt = format!(
            "Conduct a sprint retrospective for Sprint {}. Analyze what went well, what didn't, and identify improvements for the next sprint.",
            sprint_number
        );
        
        self.ai.chat(&prompt).await
    }
}

#[async_trait]
impl Agent for ProjectManagerSimulation {
    async fn plan(&self, state: &AgentState) -> Result<String> {
        let prompt = format!(
            "As a Project Manager, given this project state: {}\n\nPlan the next project management activity. Consider timeline, resources, risks, and team coordination.",
            state.task
        );
        
        self.ai.chat(&prompt).await
    }

    async fn act(&self, plan: &str) -> Result<String> {
        // Execute project management action
        let prompt = format!(
            "Execute this project management plan: {}\n\nSimulate the project management activity and report outcomes.",
            plan
        );
        
        self.ai.chat(&prompt).await
    }

    async fn observe(&self, result: &str) -> Result<String> {
        // Analyze project management results
        let prompt = format!(
            "Analyze these project management results: {}\n\nWhat insights does this provide about project health and team performance?",
            result
        );
        
        self.ai.chat(&prompt).await
    }
}

pub struct BusinessAnalystSimulation {
    ai: Arc<KandilAI>,
}

impl BusinessAnalystSimulation {
    pub fn new(ai: Arc<KandilAI>) -> Self {
        Self { ai }
    }

    pub async fn validate_requirements(&self, requirements_doc: &str) -> Result<String> {
        let prompt = format!(
            "As a Business Analyst, validate these requirements: {}\n\nCheck for completeness, consistency, feasibility, and traceability. Identify gaps and ambiguities.",
            requirements_doc
        );
        
        self.ai.chat(&prompt).await
    }

    pub async fn create_user_story(&self, feature_description: &str) -> Result<UserStory> {
        let prompt = format!(
            "As a Business Analyst, create a user story for this feature: {}\n\nFormat as INVEST (Independent, Negotiable, Valuable, Estimable, Small, Testable).",
            feature_description
        );
        
        let result = self.ai.chat(&prompt).await?;
        
        // In a real implementation, this would parse the structured response
        Ok(UserStory {
            id: "US-001".to_string(),
            title: feature_description.to_string(),
            description: result,
            acceptance_criteria: vec!["Acceptance criteria would be defined in full implementation".to_string()],
            story_points: 5,
            priority: Priority::Medium,
        })
    }
}

#[async_trait]
impl Agent for BusinessAnalystSimulation {
    async fn plan(&self, state: &AgentState) -> Result<String> {
        let prompt = format!(
            "As a Business Analyst, given this analysis task: {}\n\nPlan the next analysis step. Consider stakeholder needs, requirements gathering, and validation.",
            state.task
        );
        
        self.ai.chat(&prompt).await
    }

    async fn act(&self, plan: &str) -> Result<String> {
        // Execute business analysis action
        let prompt = format!(
            "Execute this business analysis plan: {}\n\nPerform the analysis and document findings.",
            plan
        );
        
        self.ai.chat(&prompt).await
    }

    async fn observe(&self, result: &str) -> Result<String> {
        // Analyze business analysis results
        let prompt = format!(
            "Analyze these business analysis results: {}\n\nWhat do these findings mean for the project requirements and direction?",
            result
        );
        
        self.ai.chat(&prompt).await
    }
}