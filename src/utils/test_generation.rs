//! Automated test generation
//!
//! Contains functionality for generating unit and integration tests

use anyhow::Result;
use std::path::Path;
use std::sync::Arc;

pub struct TestGenerator {
    ai_client: Arc<crate::core::adapters::ai::KandilAI>,
}

impl TestGenerator {
    pub fn new(ai_client: Arc<crate::core::adapters::ai::KandilAI>) -> Self {
        Self { ai_client }
    }

    pub async fn generate_tests_for_file(
        &self,
        source_file: &str,
        test_framework: &str,
    ) -> Result<String> {
        let source_code = std::fs::read_to_string(source_file)?;
        let file_ext = Path::new(source_file)
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("");

        let (language, framework) = match (file_ext, test_framework) {
            ("rs", "") | ("rs", "rust") => ("Rust", "Rust's built-in testing framework"),
            ("py", "") | ("py", "pytest") => ("Python", "pytest"),
            ("js", "") | ("js", "jest") => ("JavaScript", "Jest"),
            ("dart", "") | ("dart", "flutter") => ("Dart", "Flutter's testing framework"),
            (_, _) => ("Generic", "appropriate testing framework"),
        };

        let prompt = format!(
            r#"Generate comprehensive unit tests for the following {} code using {}.
            
            Source code:
            {}
            
            Generate tests that cover:
            1. All public functions/methods
            2. Edge cases
            3. Error handling
            4. Boundary conditions
            
            Output only the test code:"#,
            language, framework, source_code
        );

        self.ai_client.chat(&prompt).await
    }

    pub async fn generate_integration_tests(&self, feature_description: &str) -> Result<String> {
        let prompt = format!(
            r#"Generate integration tests for the following feature:
            
            {}
            
            Create tests that verify the end-to-end functionality, 
            including input/output validation and error scenarios.
            
            Output the test code:"#,
            feature_description
        );

        self.ai_client.chat(&prompt).await
    }

    pub async fn analyze_test_coverage(
        &self,
        source_file: &str,
        test_file: &str,
    ) -> Result<String> {
        let source_code = std::fs::read_to_string(source_file)?;
        let test_code = std::fs::read_to_string(test_file)?;

        let prompt = format!(
            r#"Analyze the test coverage of the following tests:
            
            Source code:
            {}
            
            Test code:
            {}
            
            Identify:
            1. Untested functions/methods
            2. Missing edge cases
            3. Suggested additional tests
            4. Test quality assessment
            
            Provide specific recommendations:"#,
            source_code, test_code
        );

        self.ai_client.chat(&prompt).await
    }
}
