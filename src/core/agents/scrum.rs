//! Scrum simulation agent
//! 
//! Agent that simulates Scrum ceremonies and processes

use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::core::agents::base::{Agent, AgentState, ReActLoop};
use crate::core::adapters::ai::KandilAI;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sprint {
    pub number: u32,
    pub goal: String,
    pub duration_days: u32,
    pub start_date: String,
    pub end_date: String,
    pub team_size: u32,
    pub velocity: f32, // Points per sprint
    pub committed_points: u32,
    pub completed_points: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScrumCeremony {
    pub name: String,
    pub participants: Vec<String>,
    pub duration_minutes: u32,
    pub agenda: Vec<String>,
    pub outcomes: Vec<String>,
    pub action_items: Vec<ActionItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionItem {
    pub id: String,
    pub description: String,
    pub assignee: String,
    pub priority: Priority,
    pub due_date: String,
    pub status: ActionStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Priority {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ActionStatus {
    NotStarted,
    InProgress,
    Completed,
    Blocked,
    Deferred,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Retrospective {
    pub sprint_number: u32,
    pub participants: Vec<String>,
    pub good_things: Vec<String>,
    pub improvement_areas: Vec<String>,
    pub action_items: Vec<ActionItem>,
    pub satisfaction_score: u8, // 1-10
    pub lessons_learned: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScrumSimulation {
    ai: KandilAI,
    pub current_sprint: Option<Sprint>,
    pub ceremony_templates: HashMap<String, Vec<String>>, // Templates for each ceremony
}

impl ScrumSimulation {
    pub fn new(ai: KandilAI) -> Self {
        let mut templates = HashMap::new();
        
        // Add ceremony templates
        templates.insert("sprint_planning".to_string(), vec![
            "Review product backlog items".to_string(),
            "Define sprint goal".to_string(),
            "Estimate effort for selected items".to_string(),
            "Create sprint backlog".to_string(),
            "Confirm team capacity".to_string(),
        ]);
        
        templates.insert("daily_scrum".to_string(), vec![
            "What did you do yesterday?".to_string(),
            "What will you do today?".to_string(),
            "Are there any blockers?".to_string(),
        ]);
        
        templates.insert("sprint_review".to_string(), vec![
            "Demo completed work".to_string(),
            "Gather feedback".to_string(),
            "Update product backlog".to_string(),
        ]);
        
        templates.insert("sprint_retrospective".to_string(), vec![
            "What went well?".to_string(),
            "What could be improved?".to_string(),
            "What will we commit to improve?".to_string(),
        ]);

        Self {
            ai,
            current_sprint: None,
            ceremony_templates: templates,
        }
    }

    pub async fn plan_sprint(&mut self, goal: String, duration_days: u32, team_size: u32) -> Result<Sprint> {
        let sprint_num = self.current_sprint.as_ref().map_or(1, |s| s.number + 1);
        
        let new_sprint = Sprint {
            number: sprint_num,
            goal,
            duration_days,
            start_date: chrono::Utc::now().format("%Y-%m-%d").to_string(),
            end_date: chrono::Utc::now()
                .checked_add_days(chrono::Days::new(duration_days as u64))
                .unwrap()
                .format("%Y-%m-%d")
                .to_string(),
            team_size,
            velocity: 30.0, // Default velocity
            committed_points: 40,
            completed_points: 0,
        };
        
        self.current_sprint = Some(new_sprint.clone());
        
        Ok(new_sprint)
    }

    pub async fn conduct_ceremony(&self, ceremony_type: &str, participants: Vec<String>, sprint_data: Option<&Sprint>) -> Result<ScrumCeremony> {
        let template = self.ceremony_templates.get(ceremony_type).cloned()
            .unwrap_or_else(|| vec!["Default agenda items".to_string()]);
            
        let mut agenda = template.clone();
        
        // Add sprint-specific context
        if let Some(sprint) = sprint_data {
            agenda.push(format!("Sprint {} goal: {}", sprint.number, sprint.goal));
        }

        let prompt = format!(
            r#"Conduct a {} ceremony with participants: {:?}

            Agenda: {:?}

            Generate outcomes and action items for this ceremony.
            "#,
            ceremony_type, participants, agenda
        );

        let result = self.ai.chat(&prompt).await?;
        
        Ok(ScrumCeremony {
            name: ceremony_type.to_string(),
            participants,
            duration_minutes: match ceremony_type {
                "daily_scrum" => 15,
                "sprint_planning" => 120,
                "sprint_review" => 60,
                "sprint_retrospective" => 90,
                _ => 60,
            },
            agenda,
            outcomes: vec![result],
            action_items: vec![],
        })
    }

    pub async fn run_retrospective(&self, sprint_number: u32) -> Result<Retrospective> {
        let prompt = format!(
            r#"Run sprint retrospective for sprint {}

            Generate insights about what went well, what could be improved, and action items for the next sprint.
            "#,
            sprint_number
        );

        let result = self.ai.chat(&prompt).await?;
        
        Ok(Retrospective {
            sprint_number,
            participants: vec!["Product Owner".to_string(), "Scrum Master".to_string(), "Developers".to_string()],
            good_things: vec![
                "Successfully delivered all committed stories".to_string(),
                "Improved test coverage".to_string(),
            ],
            improvement_areas: vec![
                "Daily standups ran long".to_string(),
                "Integration issues in last days".to_string(),
            ],
            action_items: vec![
                ActionItem {
                    id: format!("S{}-RT-1", sprint_number),
                    description: "Timebox daily standups to 10 minutes".to_string(),
                    assignee: "Scrum Master".to_string(),
                    priority: Priority::High,
                    due_date: chrono::Utc::now()
                        .checked_add_days(chrono::Days::new(7))
                        .unwrap()
                        .format("%Y-%m-%d")
                        .to_string(),
                    status: ActionStatus::NotStarted,
                }
            ],
            satisfaction_score: 8,
            lessons_learned: vec![
                "Early integration reduces late-stage issues".to_string(),
            ],
        })
    }

    pub async fn analyze_team_velocity(&self, historical_data: Vec<(u32, f32)>) -> Result<String> {
        let prompt = format!(
            r#"Analyze team velocity from this historical data: {:?}

            Provide insights about:
            - Average velocity
            - Trends over time
            - Factors affecting velocity
            - Recommendations for improvement
            "#,
            historical_data
        );

        self.ai.chat(&prompt).await
    }

    pub fn get_current_sprint(&self) -> Option<&Sprint> {
        self.current_sprint.as_ref()
    }
}

#[async_trait]
impl Agent for ScrumSimulation {
    async fn plan(&self, state: &AgentState) -> Result<String> {
        let prompt = format!(
            "As a Scrum Master, given this Scrum process challenge: {}\n\nPlan the next Scrum activity. Consider team dynamics, ceremony effectiveness, and process improvements.",
            state.task
        );
        
        self.ai.chat(&prompt).await
    }

    async fn act(&self, plan: &str) -> Result<String> {
        // Execute the planned Scrum activity
        let prompt = format!(
            "Execute this Scrum process plan: {}\n\nFacilitate ceremonies, resolve impediments, or improve team processes.",
            plan
        );
        
        self.ai.chat(&prompt).await
    }

    async fn observe(&self, result: &str) -> Result<String> {
        // Analyze Scrum process results
        let prompt = format!(
            "Analyze these Scrum process results: {}\n\nHow do these process changes impact team productivity and delivery?",
            result
        );
        
        self.ai.chat(&prompt).await
    }
}