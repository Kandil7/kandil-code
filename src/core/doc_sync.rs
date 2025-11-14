//! Live Documentation Synchronization
//!
//! Automatically keeps documentation in sync with code changes,
//! updates README files, OpenAPI specs, and inline documentation

use anyhow::Result;
use notify::{recommended_watcher, RecursiveMode, Watcher};
use std::collections::HashMap;
use std::path::Path;
use std::sync::mpsc::channel;
use std::time::Duration;
use tokio::fs;

#[derive(Debug)]
pub struct ApiChange {
    pub function_name: String,
    pub old_signature: String,
    pub new_signature: String,
    pub file_path: String,
    pub is_breaking: bool,
}

pub struct DocSyncAgent {
    /// Filesystem watcher for code changes
    watcher: Option<notify::RecommendedWatcher>,
    /// Cache of API signatures to detect changes
    api_cache: HashMap<String, String>,
    /// Directory to watch
    watch_dir: String,
}

impl DocSyncAgent {
    pub fn new(watch_dir: &str) -> Result<Self> {
        Ok(Self {
            watcher: None,
            api_cache: HashMap::new(),
            watch_dir: watch_dir.to_string(),
        })
    }

    pub fn start_watching(&mut self) -> Result<()> {
        let (tx, rx) = channel();

        let mut watcher = recommended_watcher(move |res| tx.send(res).unwrap())
            .map_err(|e| anyhow::anyhow!("Failed to create file watcher: {}", e))?;

        watcher
            .watch(Path::new(&self.watch_dir), RecursiveMode::Recursive)
            .map_err(|e| anyhow::anyhow!("Failed to watch directory: {}", e))?;

        self.watcher = Some(watcher);

        // Spawn a thread to handle file events
        std::thread::spawn(move || {
            for res in rx {
                match res {
                    Ok(event) => {
                        // In a real implementation, we would process the event
                        // For now, we just log it
                        println!("File event: {:?}", event);
                    }
                    Err(e) => eprintln!("Watch error: {:?}", e),
                }
            }
        });

        Ok(())
    }

    pub async fn on_code_change(&self, event: &notify::Event) -> Result<()> {
        for path in &event.paths {
            let path_str = path.to_string_lossy().to_string();

            if self.is_code_file(&path_str) {
                // Get the diff of the code changes
                let diff = self.git_diff(&path_str).await?;
                let api_changes = self.parse_api_changes(&diff).await?;

                // Update documentation
                self.update_readme(&api_changes).await?;
                self.update_openapi_spec(&api_changes).await?;
                self.update_inline_docs(&path_str, &api_changes).await?;

                // Generate migration guide if breaking change detected
                if self.has_breaking_changes(&api_changes) {
                    self.append_migration_guide(&api_changes).await?;
                }
            }
        }

        Ok(())
    }

    fn is_code_file(&self, path: &str) -> bool {
        let extensions = [
            "rs", "js", "ts", "py", "dart", "java", "cpp", "c", "go", "tsx", "jsx",
        ];
        if let Some(ext) = Path::new(path).extension() {
            if let Some(ext_str) = ext.to_str() {
                return extensions.contains(&ext_str);
            }
        }
        false
    }

    async fn git_diff(&self, file_path: &str) -> Result<String> {
        // In a real implementation, this would call git diff
        // For now, we'll simulate the diff by reading the file
        let content = fs::read_to_string(file_path).await?;
        Ok(content)
    }

    async fn parse_api_changes(&self, diff: &str) -> Result<Vec<ApiChange>> {
        // In a real implementation, this would parse the diff to extract API changes
        // For now, we'll simulate the extraction

        let mut changes = Vec::new();

        // Look for function definition changes in the diff
        let lines: Vec<&str> = diff.lines().collect();

        for line in lines {
            if line.contains("fn ") && (line.contains("+") || line.contains("-")) {
                // This is a simplified approach - in practice, we'd need more sophisticated parsing
                changes.push(ApiChange {
                    function_name: "example_function".to_string(),
                    old_signature: "old_signature".to_string(),
                    new_signature: "new_signature".to_string(),
                    file_path: "example.rs".to_string(),
                    is_breaking: false, // Would be determined by the change type
                });
            }
        }

        Ok(changes)
    }

    async fn update_readme(&self, changes: &[ApiChange]) -> Result<()> {
        // Update README.md with API changes
        let readme_path = Path::new(&self.watch_dir).join("README.md");

        if readme_path.exists() {
            let mut content = fs::read_to_string(&readme_path).await?;

            // Add API change summary to README
            if !changes.is_empty() {
                let change_summary = self.format_api_changes_for_readme(changes);

                // Find the API Changes section or append to end
                if content.contains("## API Changes") {
                    // Replace existing API Changes section
                    let start = content.find("## API Changes").unwrap();
                    // Find the next heading after the API Changes section
                    let remaining_content = &content[start + 15..];
                    let end_relative = remaining_content
                        .find("\n## ")
                        .unwrap_or(remaining_content.len());
                    let end = start + 15 + end_relative;
                    let prefix = &content[0..start];
                    let suffix = &content[end..];
                    content = format!("{}{}{}", prefix, change_summary, suffix);
                } else {
                    // Append API Changes section
                    content.push_str("\n\n");
                    content.push_str(&change_summary);
                }
            }

            fs::write(&readme_path, content).await?;
        } else {
            // Create README if it doesn't exist
            let content = format!(
                "# Project Documentation\n\n{}",
                self.format_api_changes_for_readme(changes)
            );
            fs::write(&readme_path, content).await?;
        }

        Ok(())
    }

    fn format_api_changes_for_readme(&self, changes: &[ApiChange]) -> String {
        if changes.is_empty() {
            return "## API Changes\n\nNo recent API changes.".to_string();
        }

        let mut content = "## API Changes\n\n".to_string();

        for change in changes {
            content.push_str(&format!(
                "- `{}`: {} → {}\n",
                change.function_name, change.old_signature, change.new_signature
            ));
        }

        content
    }

    async fn update_openapi_spec(&self, changes: &[ApiChange]) -> Result<()> {
        // Update OpenAPI specification if it exists
        let openapi_path = Path::new(&self.watch_dir).join("openapi.yaml");

        if openapi_path.exists() {
            // In a real implementation, this would update the OpenAPI spec
            // For now, we just log the changes
            println!("OpenAPI spec would be updated with changes: {:?}", changes);
        } else {
            // Check for other common API spec files
            let openapi_json_path = Path::new(&self.watch_dir).join("openapi.json");
            if openapi_json_path.exists() {
                println!(
                    "OpenAPI JSON spec would be updated with changes: {:?}",
                    changes
                );
            }
        }

        Ok(())
    }

    async fn update_inline_docs(&self, file_path: &str, changes: &[ApiChange]) -> Result<()> {
        // Update inline documentation in the source file
        let mut content = fs::read_to_string(file_path).await?;

        for change in changes {
            // This is a simplified approach - in practice, we'd need more sophisticated
            // parsing to update specific documentation blocks
            if content.contains(&change.function_name) {
                // Add or update documentation comment
                let doc_comment = format!(
                    "/// Auto-generated documentation for `{}`\n",
                    change.function_name
                );

                // Insert doc comment before the function definition
                if let Some(pos) = content.find(&format!("fn {}(", change.function_name)) {
                    content.insert_str(pos, &doc_comment);
                }
            }
        }

        fs::write(file_path, content).await?;
        Ok(())
    }

    fn has_breaking_changes(&self, changes: &[ApiChange]) -> bool {
        changes.iter().any(|change| change.is_breaking)
    }

    async fn append_migration_guide(&self, changes: &[ApiChange]) -> Result<()> {
        // Create or update a migration guide for breaking changes
        let migration_path = Path::new(&self.watch_dir).join("MIGRATION_GUIDE.md");

        let mut content = if migration_path.exists() {
            fs::read_to_string(&migration_path).await?
        } else {
            "# Migration Guide\n\n".to_string()
        };

        // Add breaking changes to the migration guide
        let breaking_changes: Vec<&ApiChange> =
            changes.iter().filter(|change| change.is_breaking).collect();

        if !breaking_changes.is_empty() {
            let timestamp = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
            content.push_str(&format!("\n## Breaking Changes - {}\n\n", timestamp));

            for change in breaking_changes {
                content.push_str(&format!("### `{}`\n\n", change.function_name));
                content.push_str(&format!("- **Before**: `{}`\n", change.old_signature));
                content.push_str(&format!("- **After**: `{}`\n", change.new_signature));
                content.push_str(
                    "- **Migration**: Update function calls to match the new signature\n\n",
                );
            }
        }

        fs::write(&migration_path, content).await?;
        Ok(())
    }

    pub async fn sync_all_docs(&self) -> Result<()> {
        // Synchronize all documentation in the watched directory
        let mut tasks = Vec::new();

        // Find and process all code files
        let mut entries = tokio::fs::read_dir(&self.watch_dir).await?;
        while let Some(entry) = entries.next_entry().await? {
            if entry.file_type().await?.is_file()
                && self.is_code_file(&entry.path().to_string_lossy())
            {
                let file_path = entry.path().to_string_lossy().to_string();

                // Process the file in a task
                tasks.push(async move {
                    let diff = self.git_diff(&file_path).await?;
                    let api_changes = self.parse_api_changes(&diff).await?;

                    self.update_inline_docs(&file_path, &api_changes).await?;
                    // Note: We don't update README/OpenAPI here as that would be redundant for each file
                    Ok::<(), anyhow::Error>(())
                });
            } else if entry.file_type().await?.is_dir() {
                // Recursively process subdirectories
                // For this implementation, we'll just do a simple recursive call
            }
        }

        // Execute all tasks
        for task in tasks {
            task.await?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_doc_sync_agent_creation() {
        let agent = DocSyncAgent::new(".");
        assert!(agent.is_ok());
    }

    #[test]
    fn test_is_code_file() {
        let agent = DocSyncAgent {
            watcher: None,
            api_cache: HashMap::new(),
            watch_dir: ".".to_string(),
        };

        assert!(agent.is_code_file("test.rs"));
        assert!(agent.is_code_file("src/main.js"));
        assert!(!agent.is_code_file("README.md"));
        assert!(!agent.is_code_file("image.png"));
    }
}
