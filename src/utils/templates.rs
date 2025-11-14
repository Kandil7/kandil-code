//! Template engine for generating multi-language project structures
//! 
//! Contains functionality for creating project templates from various languages

use anyhow::Result;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct TemplateEngine {
    templates: HashMap<String, Template>,
}

#[derive(Debug, Clone)]
pub struct Template {
    name: String,
    description: String,
    pub files: Vec<TemplateFile>,
}

impl Template {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn description(&self) -> &str {
        &self.description
    }
}

#[derive(Debug, Clone)]
pub struct TemplateFile {
    pub path: String,
    pub content: String,
    is_executable: bool,
}

impl TemplateFile {
    pub fn is_executable(&self) -> bool {
        self.is_executable
    }
}

impl TemplateEngine {
    pub fn new() -> Self {
        let mut engine = Self {
            templates: HashMap::new(),
        };
        
        engine.load_templates();
        engine
    }

    fn load_templates(&mut self) {
        // Load built-in templates
        self.load_flutter_template();
        self.load_python_template();
        self.load_js_template();
        self.load_rust_template();
    }

    fn load_flutter_template(&mut self) {
        let template = Template {
            name: "flutter".to_string(),
            description: "Flutter Clean Architecture project".to_string(),
            files: vec![
                TemplateFile {
                    path: "pubspec.yaml".to_string(),
                    content: include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/templates/flutter/clean_arch/pubspec.yaml")).to_string(),
                    is_executable: false,
                },
                TemplateFile {
                    path: "lib/main.dart".to_string(),
                    content: include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/templates/flutter/clean_arch/lib/main.dart")).to_string(),
                    is_executable: false,
                },
            ],
        };
        self.templates.insert("flutter".to_string(), template);
    }

    fn load_python_template(&mut self) {
        let template = Template {
            name: "python".to_string(),
            description: "Python FastAPI project".to_string(),
            files: vec![
                TemplateFile {
                    path: "requirements.txt".to_string(),
                    content: include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/templates/python/fastapi/requirements.txt")).to_string(),
                    is_executable: false,
                },
                TemplateFile {
                    path: "app/main.py".to_string(),
                    content: include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/templates/python/fastapi/app/main.py")).to_string(),
                    is_executable: false,
                },
            ],
        };
        self.templates.insert("python".to_string(), template);
    }

    fn load_js_template(&mut self) {
        let template = Template {
            name: "javascript".to_string(),
            description: "JavaScript Next.js project".to_string(),
            files: vec![
                TemplateFile {
                    path: "package.json".to_string(),
                    content: include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/templates/js/nextjs/package.json")).to_string(),
                    is_executable: false,
                },
                TemplateFile {
                    path: "pages/index.js".to_string(),
                    content: include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/templates/js/nextjs/pages/index.js")).to_string(),
                    is_executable: false,
                },
            ],
        };
        self.templates.insert("javascript".to_string(), template);
    }

    fn load_rust_template(&mut self) {
        let template = Template {
            name: "rust".to_string(),
            description: "Rust CLI project".to_string(),
            files: vec![
                TemplateFile {
                    path: "Cargo.toml".to_string(),
                    content: include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/templates/rust/cli/Cargo.toml")).to_string(),
                    is_executable: false,
                },
                TemplateFile {
                    path: "src/main.rs".to_string(),
                    content: include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/templates/rust/cli/src/main.rs")).to_string(),
                    is_executable: false,
                },
            ],
        };
        self.templates.insert("rust".to_string(), template);
    }

    pub fn list_templates(&self) -> Vec<(&String, &str)> {
        self.templates
            .iter()
            .map(|(name, template)| (name, template.description()))
            .collect()
    }

    pub fn create_project(&self, template_name: &str, project_path: &str, project_name: &str) -> Result<()> {
        if let Some(template) = self.templates.get(template_name) {
            let project_dir = Path::new(project_path);
            fs::create_dir_all(project_dir)?;

            for file in &template.files {
                // Replace template variables
                let content = file.content.replace("{{project_name}}", project_name);
                
                let file_path = project_dir.join(&file.path);
                if let Some(parent) = file_path.parent() {
                    fs::create_dir_all(parent)?;
                }
                
                fs::write(&file_path, content)?;

                // Set executable permissions if needed (Unix systems)
                #[cfg(unix)]
                if file.is_executable {
                    use std::os::unix::fs::PermissionsExt;
                    let mut perms = fs::metadata(&file_path)?.permissions();
                    perms.set_mode(perms.mode() | 0o111); // Add execute permissions
                    fs::set_permissions(&file_path, perms)?;
                }
            }

            Ok(())
        } else {
            Err(anyhow::anyhow!("Template '{}' not found", template_name))
        }
    }
}

pub use TemplateEngine as TemplateMgr;