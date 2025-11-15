//! Adaptive Context Management with Semantic Compression
//!
//! Implements tree-sitter-based AST analysis to prioritize relevant code
//! and embedding-based compression for historical context

use anyhow::Result;
use std::collections::HashMap;
use std::path::Path;
use tree_sitter::{Parser, Query};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextWindow {
    pub files: Vec<ContextFile>,
    pub history: Vec<ContextHistoryItem>,
    pub estimated_tokens: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextFile {
    pub path: String,
    pub content: String,
    pub symbols: Vec<String>,
    pub dependencies: Vec<String>,
    pub relevance_score: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextHistoryItem {
    pub timestamp: std::time::SystemTime,
    pub summary: String,
    pub keywords: Vec<String>,
}

pub struct ContextManager {
    /// Language for tree-sitter
    language: tree_sitter::Language,
    /// Language-specific queries for symbol extraction
    queries: HashMap<String, Query>,
    /// Memory compressor for history
    memory_compressor: MemoryCompressor,
}

impl ContextManager {
    pub fn new() -> Result<Self> {
        // For now, we'll set up for a generic language - in practice,
        // we'd need to configure for specific languages
        let language = tree_sitter_rust::language();

        Ok(Self {
            language,
            queries: HashMap::new(),
            memory_compressor: MemoryCompressor::new(),
        })
    }

    pub fn prepare_context(&self, task: &str, workspace_path: &str) -> Result<ContextWindow> {
        // 1. Extract symbols from task (e.g., "fix auth bug" → ["auth", "login"])
        let symbols = self.extract_symbols_from_task(task);

        // 2. Find relevant files based on symbols and dependencies
        let relevant_files = self.find_relevant_files(workspace_path, &symbols)?;

        // 3. Compress historical context
        let compressed_history = self.memory_compressor.summarize_history()?;

        // 4. Estimate token count
        let estimated_tokens = self.estimate_tokens(&relevant_files, &compressed_history);

        Ok(ContextWindow {
            files: relevant_files,
            history: compressed_history,
            estimated_tokens,
        })
    }

    fn extract_symbols_from_task(&self, task: &str) -> Vec<String> {
        // Simple keyword extraction - in practice, we'd use NLP
        let keywords = [
            "auth", "login", "user", "api", "database", "config", 
            "test", "bug", "fix", "feature", "service", "model", 
            "view", "controller", "route", "middleware"
        ];

        let mut symbols = Vec::new();
        for keyword in keywords {
            if task.to_lowercase().contains(keyword) {
                symbols.push(keyword.to_string());
            }
        }

        symbols
    }

    fn find_relevant_files(&self, workspace_path: &str, symbols: &[String]) -> Result<Vec<ContextFile>> {
        let mut relevant_files = Vec::new();

        // Walk through the workspace to find relevant files
        for entry in std::fs::read_dir(workspace_path)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() && self.is_code_file(&path) {
                let content = std::fs::read_to_string(&path)?;
                let file_symbols = self.extract_symbols_from_code(&content)?;
                
                // Calculate relevance based on symbol overlap
                let relevance = self.calculate_relevance(&file_symbols, symbols);
                
                if relevance > 0.0 {
                    let dependencies = self.extract_dependencies(&content)?;
                    relevant_files.push(ContextFile {
                        path: path.to_string_lossy().to_string(),
                        content,
                        symbols: file_symbols,
                        dependencies,
                        relevance_score: relevance,
                    });
                }
            } else if path.is_dir() {
                // Recursively search in subdirectories
                let sub_files = self.find_relevant_files(&path.to_string_lossy(), symbols)?;
                relevant_files.extend(sub_files);
            }
        }

        // Sort by relevance score (highest first)
        relevant_files.sort_by(|a, b| b.relevance_score.partial_cmp(&a.relevance_score).unwrap());

        // Limit to top 20 most relevant files to manage token count
        relevant_files.truncate(20);

        Ok(relevant_files)
    }

    fn is_code_file(&self, path: &Path) -> bool {
        let extensions = ["rs", "js", "ts", "py", "dart", "java", "cpp", "c", "go", "tsx", "jsx"];
        if let Some(ext) = path.extension() {
            if let Some(ext_str) = ext.to_str() {
                return extensions.contains(&ext_str);
            }
        }
        false
    }

    fn extract_symbols_from_code(&self, code: &str) -> Result<Vec<String>> {
        // Create a parser on-demand to avoid borrowing issues
        let mut parser = Parser::new();
        parser.set_language(self.language).map_err(|e| {
            anyhow::anyhow!("Failed to set parser language: {}", e)
        })?;

        // Use tree-sitter to parse the code and extract symbols
        let tree = parser.parse(code, None).ok_or_else(|| {
            anyhow::anyhow!("Failed to parse code")
        })?;

        let mut symbols = Vec::new();
        self.traverse_tree_for_symbols(&tree.root_node(), code, &mut symbols);

        // Remove duplicates
        symbols.sort();
        symbols.dedup();

        Ok(symbols)
    }

    fn traverse_tree_for_symbols(&self, node: &tree_sitter::Node, source: &str, symbols: &mut Vec<String>) {
        // This is a simplified traversal - in practice, we'd have more specific queries
        // for different language constructs (functions, classes, variables, etc.)
        if node.kind() == "function" || node.kind() == "function_item" {
            if let Some(name_node) = node.child_by_field_name("name") {
                let name = &source[name_node.start_byte()..name_node.end_byte()];
                symbols.push(name.to_string());
            }
        } else if node.kind() == "identifier" || node.kind() == "type_identifier" {
            let identifier = &source[node.start_byte()..node.end_byte()];
            if !identifier.is_empty() && identifier.chars().next().unwrap_or('a').is_alphabetic() {
                symbols.push(identifier.to_string());
            }
        }

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.traverse_tree_for_symbols(&child, source, symbols);
        }
    }

    fn calculate_relevance(&self, file_symbols: &[String], task_symbols: &[String]) -> f32 {
        let mut relevance = 0.0;
        for task_symbol in task_symbols {
            for file_symbol in file_symbols {
                if task_symbol == file_symbol {
                    relevance += 1.0;
                } else if task_symbol.contains(file_symbol) || file_symbol.contains(task_symbol) {
                    relevance += 0.5; // Partial match
                }
            }
        }
        
        // Normalize by the number of symbols for fair comparison
        if !file_symbols.is_empty() {
            relevance / file_symbols.len() as f32
        } else {
            0.0
        }
    }

    fn extract_dependencies(&self, code: &str) -> Result<Vec<String>> {
        // Simple regex-based dependency extraction
        // In practice, we'd use tree-sitter queries for different languages
        let mut dependencies = Vec::new();
        
        // Look for import/require statements
        let re = regex::Regex::new(r"(import|from|require|use)\s+([a-zA-Z0-9_\.]+)")?;
        for cap in re.captures_iter(code) {
            if let Some(dep) = cap.get(2) {
                dependencies.push(dep.as_str().to_string());
            }
        }

        Ok(dependencies)
    }

    fn estimate_tokens(&self, files: &[ContextFile], history: &[ContextHistoryItem]) -> usize {
        // Rough estimation: 1 token ≈ 4 characters for English text
        let mut total_chars = 0;
        for file in files {
            total_chars += file.content.len();
        }
        for item in history {
            total_chars += item.summary.len();
        }

        // Add overhead for structure and metadata
        (total_chars / 4) + (files.len() * 50) + (history.len() * 20)
    }
}

pub struct MemoryCompressor {
    max_history_items: usize,
}

impl MemoryCompressor {
    pub fn new() -> Self {
        Self {
            max_history_items: 50, // Limit history to prevent token explosion
        }
    }

    pub fn summarize_history(&self) -> Result<Vec<ContextHistoryItem>> {
        // For now, return an empty history - in a real implementation,
        // this would compress actual conversation history into key-value summaries
        Ok(Vec::new())
    }

    pub fn compress_to_summary(&self, content: &str) -> String {
        // Basic compression by extracting key sentences
        // In a real implementation, we'd use embeddings and clustering
        let sentences: Vec<&str> = content.split('.').collect();
        if sentences.len() <= 3 {
            content.to_string()
        } else {
            // Take the first and last sentences as a summary
            format!("{}... [truncated] ...{}", sentences[0], sentences[sentences.len()-1])
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_symbols_from_task() {
        let context_manager = ContextManager::new().unwrap();
        let symbols = context_manager.extract_symbols_from_task("fix user login authentication");
        assert!(symbols.contains(&"auth".to_string()));
        assert!(symbols.contains(&"login".to_string()));
        assert!(symbols.contains(&"user".to_string()));
    }

    #[test]
    fn test_calculate_relevance() {
        let manager = ContextManager::new().unwrap();
        let file_symbols = vec!["login".to_string(), "auth".to_string(), "user".to_string()];
        let task_symbols = vec!["auth".to_string(), "login".to_string()];
        
        let relevance = manager.calculate_relevance(&file_symbols, &task_symbols);
        assert!(relevance > 0.0);
    }
}