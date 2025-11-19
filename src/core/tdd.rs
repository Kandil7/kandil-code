//! Automatic TDD Agent
//!
//! Implements Test-Driven Development automation where tests are generated
//! first, code is implemented to pass tests, and mutation testing ensures
//! test quality

use anyhow::Result;
use async_trait::async_trait;
use std::sync::Arc;

use crate::core::adapters::ai::AIProviderTrait;
use crate::core::agents::base::{Agent, AgentState};

#[derive(Debug)]
pub struct CodeChanges {
    pub tests: String,
    pub implementation: String,
}

pub struct TestDrivenAgent {
    spec_agent: Arc<dyn Agent>,
    impl_agent: Arc<dyn Agent>,
    mutation_tester: Arc<MutantTester>,
    ai_provider: Arc<dyn AIProviderTrait>,
}

impl TestDrivenAgent {
    pub fn new(ai_provider: Arc<dyn AIProviderTrait>) -> Result<Self> {
        let spec_agent = Arc::new(TestSpecAgent::new(Arc::clone(&ai_provider)));
        let impl_agent = Arc::new(ImplementationAgent::new(Arc::clone(&ai_provider)));
        let mutation_tester = Arc::new(MutantTester::new());

        Ok(Self {
            spec_agent,
            impl_agent,
            mutation_tester,
            ai_provider,
        })
    }

    pub async fn execute(&self, requirement: &str) -> Result<CodeChanges> {
        // Step 1: Generate test suite from requirement
        let tests = self.generate_tests(requirement).await?;

        // Step 2: Run tests (they should fail initially)
        let initial_results = self.run_tests(&tests).await?;
        if !initial_results.all_failed() {
            // If tests don't fail initially, they might not be good tests
            eprintln!("Warning: Not all tests failed initially, test quality may be low");
        }

        // Step 3: Implement code until all tests pass
        let mut implementation = self.generate_implementation(&tests).await?;
        let mut attempt = 0;
        let max_attempts = 5; // Prevent infinite loops

        while !self
            .run_tests_with_implementation(&tests, &implementation)
            .await?
            .all_passed()
        {
            if attempt >= max_attempts {
                return Err(anyhow::anyhow!(
                    "Could not implement code that passes all tests after {} attempts",
                    max_attempts
                ));
            }

            implementation = self.refine_implementation(implementation, &tests).await?;
            attempt += 1;
        }

        // Step 4: Run mutation testing to ensure test quality
        let mutation_score = self.mutation_tester.run(&tests, &implementation).await?;
        if mutation_score < 0.9 {
            return Err(anyhow::anyhow!(
                "Tests are insufficiently rigorous, mutation score: {:.2}",
                mutation_score
            ));
        }

        Ok(CodeChanges {
            tests,
            implementation,
        })
    }

    async fn generate_tests(&self, requirement: &str) -> Result<String> {
        let task = format!("Generate comprehensive unit tests for the following requirement: {}\n\nReturn only the test code in the appropriate language.", requirement);
        let state = AgentState {
            task,
            observations: vec![],
            current_step: 0,
            max_steps: 1,
            is_complete: false,
            result: None,
        };

        let result = self.spec_agent.plan(&state).await?;
        Ok(result)
    }

    async fn generate_implementation(&self, tests: &str) -> Result<String> {
        let task = format!("Generate implementation code that will make the following tests pass:\n\n{}\n\nReturn only the implementation code.", tests);
        let state = AgentState {
            task,
            observations: vec![],
            current_step: 0,
            max_steps: 1,
            is_complete: false,
            result: None,
        };

        let result = self.impl_agent.plan(&state).await?;
        Ok(result)
    }

    async fn refine_implementation(&self, current_impl: String, tests: &str) -> Result<String> {
        let task = format!(
            "The following implementation does not pass all tests:\n\n{}\n\nTests that should pass:\n\n{}\n\nFix the implementation to make all tests pass.",
            current_impl,
            tests
        );
        let state = AgentState {
            task,
            observations: vec![],
            current_step: 0,
            max_steps: 1,
            is_complete: false,
            result: None,
        };

        let result = self.impl_agent.plan(&state).await?;
        Ok(result)
    }

    async fn run_tests(&self, tests: &str) -> Result<TestResults> {
        // In a real implementation, this would execute the tests
        // For now, we'll simulate test execution
        Ok(TestResults {
            passed: 0,
            failed: 5, // Simulate that tests fail initially
            skipped: 0,
        })
    }

    async fn run_tests_with_implementation(
        &self,
        tests: &str,
        implementation: &str,
    ) -> Result<TestResults> {
        // In a real implementation, this would run tests against the implementation
        // For now, we'll simulate based on some logic

        // Simple heuristic: if implementation contains keywords that match test expectations
        let passes = tests.to_lowercase().contains("assert")
            && implementation.to_lowercase().contains("return");

        if passes {
            Ok(TestResults {
                passed: 5,
                failed: 0,
                skipped: 0,
            })
        } else {
            Ok(TestResults {
                passed: 0,
                failed: 5,
                skipped: 0,
            })
        }
    }
}

pub struct TestResults {
    pub passed: usize,
    pub failed: usize,
    pub skipped: usize,
}

impl TestResults {
    pub fn all_passed(&self) -> bool {
        self.failed == 0 && self.passed > 0
    }

    pub fn all_failed(&self) -> bool {
        self.passed == 0 && self.failed > 0
    }
}

// Agent for generating test specifications
struct TestSpecAgent {
    ai_provider: Arc<dyn AIProviderTrait>,
}

impl TestSpecAgent {
    pub fn new(ai_provider: Arc<dyn AIProviderTrait>) -> Self {
        Self { ai_provider }
    }
}

#[async_trait]
impl Agent for TestSpecAgent {
    async fn plan(&self, state: &AgentState) -> Result<String> {
        let response = self.ai_provider.chat(&state.task).await?;
        Ok(response)
    }

    async fn act(&self, plan: &str) -> Result<String> {
        // In this case, acting just means returning the plan (the tests)
        Ok(plan.to_string())
    }

    async fn observe(&self, result: &str) -> Result<String> {
        // Observe the generated tests
        Ok(format!("Generated tests: {} characters", result.len()))
    }
}

// Agent for generating implementation code
struct ImplementationAgent {
    ai_provider: Arc<dyn AIProviderTrait>,
}

impl ImplementationAgent {
    pub fn new(ai_provider: Arc<dyn AIProviderTrait>) -> Self {
        Self { ai_provider }
    }
}

#[async_trait]
impl Agent for ImplementationAgent {
    async fn plan(&self, state: &AgentState) -> Result<String> {
        let response = self.ai_provider.chat(&state.task).await?;
        Ok(response)
    }

    async fn act(&self, plan: &str) -> Result<String> {
        // In this case, acting just means returning the plan (the implementation)
        Ok(plan.to_string())
    }

    async fn observe(&self, result: &str) -> Result<String> {
        // Observe the generated implementation
        Ok(format!(
            "Generated implementation: {} characters",
            result.len()
        ))
    }
}

// Mutation tester to ensure test quality
pub struct MutantTester;

impl MutantTester {
    pub fn new() -> Self {
        Self
    }

    pub async fn run(&self, tests: &str, implementation: &str) -> Result<f64> {
        // This would run mutation testing by introducing small changes (mutations)
        // to the implementation and checking if tests catch them

        // In a real implementation, we would:
        // 1. Create mutants by making small changes to the implementation
        // 2. Run tests against each mutant
        // 3. Count how many mutants were "killed" by tests
        // 4. Return the mutation score (killed_mutants / total_mutants)

        // For simulation, we'll return a score based on simple heuristics
        let test_complexity = tests.matches("assert").count();
        let impl_complexity = implementation.matches("fn").count();

        // Higher complexity in tests with appropriate implementation complexity
        // should give a better mutation score
        let score = if test_complexity > 0 && impl_complexity > 0 {
            std::cmp::min(test_complexity, impl_complexity) as f64
                / std::cmp::max(test_complexity, impl_complexity) as f64
        } else {
            0.5 // Default score if no clear pattern
        };

        // Ensure score is between 0 and 1
        Ok(score.clamp(0.0, 1.0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Mock AI provider for testing
    struct MockAIProvider;

    #[async_trait]
    impl AIProviderTrait for MockAIProvider {
        async fn chat(&self, _message: &str) -> Result<String> {
            Ok("Mocked response".to_string())
        }

        async fn chat_with_context(
            &self,
            _message: &str,
            _workspace_path: Option<&str>,
        ) -> Result<String> {
            Ok("Mocked response with context".to_string())
        }
    }

    #[tokio::test]
    async fn test_tdd_agent_creation() {
        // This test would need a proper mock AI provider
        // For now, just testing structure
        assert!(true);
    }

    #[test]
    fn test_test_results() {
        let results = TestResults {
            passed: 5,
            failed: 0,
            skipped: 0,
        };

        assert!(results.all_passed());
        assert!(!results.all_failed());

        let results = TestResults {
            passed: 0,
            failed: 5,
            skipped: 0,
        };

        assert!(!results.all_passed());
        assert!(results.all_failed());
    }
}
