//! Task complexity analyzer for Kandil Code
//!
//! Analyzes user prompts to determine complexity level for optimal model selection.

use tiktoken_rs::num_tokens_from_messages;

#[derive(Debug, Clone, PartialEq)]
pub enum TaskComplexity {
    Simple,   // <200 tokens, autocomplete, syntax fix
    Medium,   // 200-1000 tokens, function generation
    Complex,  // >1000 tokens, architecture, debugging
}

impl TaskComplexity {
    pub fn from_prompt(prompt: &str) -> Self {
        // First, use token counting which is more accurate than character count
        let token_count = count_tokens(prompt);
        
        match token_count {
            0..=200 => TaskComplexity::Simple,
            201..=1000 => TaskComplexity::Medium,
            _ => TaskComplexity::Complex,
        }
    }

    /// Analyze the prompt content more deeply to determine complexity
    pub fn from_content_analysis(prompt: &str) -> Self {
        let mut complexity_score = 0.0;
        
        // Keywords that indicate complexity
        let complex_keywords = [
            "architecture", "design", "system", "refactor", "debug", "performance", 
            "optimization", "security", "scaling", "distributed", "patterns", "algorithm"
        ];
        
        let simple_keywords = [
            "what is", "how to", "define", "explain", "syntax", "fix", "bug"
        ];
        
        let prompt_lower = prompt.to_lowercase();
        
        // Score based on complex keywords
        for keyword in &complex_keywords {
            if prompt_lower.contains(keyword) {
                complexity_score += 2.0;
            }
        }
        
        // Score based on simple keywords
        for keyword in &simple_keywords {
            if prompt_lower.contains(keyword) {
                complexity_score -= 1.0;
            }
        }
        
        // Length factor (longer prompts tend to be more complex)
        let length_factor = (prompt.len() as f64 / 1000.0).min(5.0);
        complexity_score += length_factor;
        
        // Analyze code elements
        if prompt.contains("fn ") || prompt.contains("function") || prompt.contains("class") {
            complexity_score += 0.5; // Programming tasks are usually medium+
        }
        
        if prompt.contains('{') && prompt.contains('}') {
            // Code block detected, likely to be a coding task
            complexity_score += 0.3;
        }
        
        // Determine complexity based on score
        match complexity_score {
            score if score < 1.0 => TaskComplexity::Simple,
            score if score < 3.0 => TaskComplexity::Medium,
            _ => TaskComplexity::Complex,
        }
    }

    /// Get the combined complexity assessment
    pub fn analyze(prompt: &str) -> Self {
        let token_based = Self::from_prompt(prompt);
        let content_based = Self::from_content_analysis(prompt);
        
        // Use the higher complexity when they disagree
        match (token_based, content_based) {
            (TaskComplexity::Complex, _) | (_, TaskComplexity::Complex) => TaskComplexity::Complex,
            (TaskComplexity::Medium, _) | (_, TaskComplexity::Medium) => TaskComplexity::Medium,
            _ => TaskComplexity::Simple,
        }
    }
}

fn count_tokens(text: &str) -> usize {
    // Use tiktoken_rs to count tokens accurately
    // This is a simplified implementation - in a real system, 
    // we'd want to choose the appropriate encoding based on the model
    match tiktoken_rs::num_tokens_from_messages("gpt-3.5-turbo", &[(
        "user".to_string(),
        text.to_string(),
    )]) {
        Ok(count) => count,
        Err(_) => {
            // Fallback to a rough character-based estimate if token counting fails
            text.chars().count() / 4  // Rough estimate: 1 token ~ 4 characters
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_task() {
        let prompt = "What is 2 + 2?";
        assert_eq!(TaskComplexity::analyze(prompt), TaskComplexity::Simple);
    }

    #[test]
    fn test_medium_task() {
        let prompt = "Write a Rust function that takes a vector of integers and returns the sum.";
        assert_eq!(TaskComplexity::analyze(prompt), TaskComplexity::Medium);
    }

    #[test]
    fn test_complex_task() {
        let prompt = r#"
            Design a distributed caching system that handles high throughput requests, 
            provides cache invalidation, and supports multiple cache eviction strategies. 
            Consider CAP theorem tradeoffs and provide implementation details for each component.
        "#;
        assert_eq!(TaskComplexity::analyze(prompt), TaskComplexity::Complex);
    }
}