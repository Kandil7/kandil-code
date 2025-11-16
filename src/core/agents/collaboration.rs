//! Collaboration module for tech role simulations
//!
//! Handles cross-role collaboration and communication between Architect, Developer, and QA

use crate::core::agents::{ArchitectSimulation, DeveloperSimulation, QaSimulation};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossRoleDecision {
    pub id: String,
    pub title: String,
    pub description: String,
    pub involved_roles: Vec<Role>,
    pub discussion_points: Vec<DiscussionPoint>,
    pub final_decision: String,
    pub status: DecisionStatus,
    pub created_date: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscussionPoint {
    pub role: Role,
    pub point: String,
    pub concerns: Vec<String>,
    pub suggestions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Role {
    Architect,
    Developer,
    QA,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DecisionStatus {
    Proposed,
    UnderReview,
    Accepted,
    Rejected,
    Implemented,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollaborationSession {
    pub id: String,
    pub title: String,
    pub participants: Vec<Role>,
    pub agenda: Vec<String>,
    pub discussion_points: Vec<DiscussionPoint>,
    pub action_items: Vec<ActionItem>,
    pub outcomes: Vec<String>,
    pub created_date: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionItem {
    pub id: String,
    pub description: String,
    pub assigned_to: Role,
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
}

pub struct CollaborationManager {
    pub decisions: Vec<CrossRoleDecision>,
    pub sessions: Vec<CollaborationSession>,
    pub ongoing_discussions: HashMap<String, Vec<String>>,
}

impl CollaborationManager {
    pub fn new() -> Self {
        Self {
            decisions: vec![],
            sessions: vec![],
            ongoing_discussions: HashMap::new(),
        }
    }

    pub async fn start_collaboration_session(
        &mut self,
        title: &str,
        participants: Vec<Role>,
        agenda: Vec<String>,
    ) -> String {
        let session_id = format!("CS-{}", self.sessions.len() + 1);

        let session = CollaborationSession {
            id: session_id.clone(),
            title: title.to_string(),
            participants,
            agenda,
            discussion_points: vec![],
            action_items: vec![],
            outcomes: vec![],
            created_date: chrono::Utc::now().format("%Y-%m-%d").to_string(),
        };

        self.sessions.push(session);
        session_id
    }

    pub async fn facilitate_architecture_discussion(
        &mut self,
        architect_agent: &ArchitectSimulation,
        developer_agent: &mut DeveloperSimulation,
        qa_agent: &QaSimulation,
        topic: &str,
    ) -> Result<String> {
        // Simulate a discussion between roles
        let mut discussion = vec![];

        // Architect's perspective
        let arch_input = architect_agent
            .ai
            .chat(&format!("Discuss architecture for: {}", topic))
            .await?;
        discussion.push(format!("Architect: {}", arch_input));

        // Developer's perspective
        let dev_input = developer_agent
            .ai
            .chat(&format!(
                "From development perspective, consider: {}",
                topic
            ))
            .await?;
        discussion.push(format!("Developer: {}", dev_input));

        // QA's perspective
        let qa_input = qa_agent
            .ai
            .chat(&format!("From QA perspective, consider: {}", topic))
            .await?;
        discussion.push(format!("QA: {}", qa_input));

        // Store the discussion
        self.ongoing_discussions
            .insert(topic.to_string(), discussion.clone());

        Ok(discussion.join("\n"))
    }

    pub async fn create_cross_role_decision(
        &mut self,
        title: &str,
        description: &str,
        involved_roles: Vec<Role>,
        architect_agent: &ArchitectSimulation,
        developer_agent: &mut DeveloperSimulation,
        qa_agent: &QaSimulation,
    ) -> Result<CrossRoleDecision> {
        let decision_id = format!("CRD-{}", self.decisions.len() + 1);

        let mut discussion_points = vec![];

        // Collect input from each role
        for role in &involved_roles {
            match role {
                Role::Architect => {
                    let input = architect_agent
                        .ai
                        .chat(&format!(
                            "From architecture perspective, consider: {}",
                            description
                        ))
                        .await?;
                    discussion_points.push(DiscussionPoint {
                        role: role.clone(),
                        point: input,
                        concerns: vec![],
                        suggestions: vec![],
                    });
                }
                Role::Developer => {
                    let input = developer_agent
                        .ai
                        .chat(&format!(
                            "From development perspective, consider: {}",
                            description
                        ))
                        .await?;
                    discussion_points.push(DiscussionPoint {
                        role: role.clone(),
                        point: input,
                        concerns: vec![],
                        suggestions: vec![],
                    });
                }
                Role::QA => {
                    let input = qa_agent
                        .ai
                        .chat(&format!("From QA perspective, consider: {}", description))
                        .await?;
                    discussion_points.push(DiscussionPoint {
                        role: role.clone(),
                        point: input,
                        concerns: vec![],
                        suggestions: vec![],
                    });
                }
            }
        }

        // Generate final decision based on inputs
        let final_decision = format!(
            "Decision synthesized from inputs from: {:?}",
            involved_roles
        );

        let cross_role_decision = CrossRoleDecision {
            id: decision_id,
            title: title.to_string(),
            description: description.to_string(),
            involved_roles,
            discussion_points,
            final_decision,
            status: DecisionStatus::UnderReview,
            created_date: chrono::Utc::now().format("%Y-%m-%d").to_string(),
        };

        self.decisions.push(cross_role_decision.clone());

        Ok(cross_role_decision)
    }

    pub fn get_decision(&self, id: &str) -> Option<&CrossRoleDecision> {
        self.decisions.iter().find(|d| d.id == id)
    }

    pub fn get_session(&self, id: &str) -> Option<&CollaborationSession> {
        self.sessions.iter().find(|s| s.id == id)
    }
}
