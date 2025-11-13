//! Base agent framework for Kandil Code
//! 
//! Contains the ReAct (Reason-Act-Observe) agent framework

use async_trait::async_trait;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::time::timeout;
use crate::core::adapters::ai::KandilAI;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentState {
    pub task: String,
    pub observations: Vec<String>,
    pub current_step: usize,
    pub max_steps: usize,
    pub is_complete: bool,
    pub result: Option<String>,
}

#[async_trait]
pub trait Agent: Send + Sync {
    async fn plan(&self, state: &AgentState) -> Result<String>;
    async fn act(&self, plan: &str) -> Result<String>;
    async fn observe(&self, result: &str) -> Result<String>;
    async fn should_continue(&self, state: &AgentState) -> bool {
        state.current_step < state.max_steps && !state.is_complete
    }
}

pub struct ReActLoop {
    max_steps: usize,
    timeout_per_step: Duration,
}

impl ReActLoop {
    pub fn new(max_steps: usize) -> Self {
        Self {
            max_steps,
            timeout_per_step: Duration::from_secs(120),
        }
    }

    pub async fn run<T: Agent>(&self, agent: &T, task: &str) -> Result<AgentResult> {
        let mut state = AgentState {
            task: task.to_string(),
            observations: vec![],
            current_step: 0,
            max_steps: self.max_steps,
            is_complete: false,
            result: None,
        };

        while agent.should_continue(&state).await {
            // Plan step
            let plan = timeout(self.timeout_per_step, agent.plan(&state)).await??;
            
            // Act step
            let action_result = timeout(self.timeout_per_step, agent.act(&plan)).await??;
            
            // Observe step 
            let observation = timeout(self.timeout_per_step, agent.observe(&action_result)).await??;
            
            // Update state
            state.observations.push(observation.clone());
            state.current_step += 1;
            
            // Check if task is complete based on the observation
            // This is a simple check - in a real implementation, this would be more sophisticated
            if observation.contains("COMPLETED") || observation.contains("FINISHED") {
                state.is_complete = true;
                state.result = Some(action_result);
                break;
            }
        }

        Ok(AgentResult {
            final_answer: state.result.unwrap_or_else(|| {
                state.observations.join("\n")
            }),
            steps_taken: state.current_step,
            success: state.is_complete || !state.observations.is_empty(),
        })
    }
}

#[derive(Debug)]
pub struct AgentResult {
    pub final_answer: String,
    pub steps_taken: usize,
    pub success: bool,
}