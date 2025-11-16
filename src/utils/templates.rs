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
                    content: "name: {{project_name}}\ndescription: A new Flutter project.\nversion: 1.0.0+1\n\nenvironment:\n  sdk: '>=3.0.0 <4.0.0'\n  flutter: '>=3.0.0'\n\ndependencies:\n  flutter:\n    sdk: flutter\n  get_it: ^7.0.0\n  dio: ^5.0.0\n  flutter_bloc: ^8.0.0\n\nflutter:\n  uses-material-design: true\n".to_string(),
                    is_executable: false,
                },
                TemplateFile {
                    path: "lib/main.dart".to_string(),
                    content: "import 'package:flutter/material.dart';\n\nvoid main() {\n  runApp(const MyApp());\n}\n\nclass MyApp extends StatelessWidget {\n  const MyApp({super.key});\n\n  @override\n  Widget build(BuildContext context) {\n    return MaterialApp(\n      title: 'Flutter Demo',\n      theme: ThemeData(\n        primarySwatch: Colors.blue,\n      ),\n      home: const MyHomePage(title: 'Flutter Demo Home Page'),\n    );\n  }\n}\n\nclass MyHomePage extends StatefulWidget {\n  const MyHomePage({super.key, required this.title});\n\n  final String title;\n\n  @override\n  State<MyHomePage> createState() => _MyHomePageState();\n}\n\nclass _MyHomePageState extends State<MyHomePage> {\n  int _counter = 0;\n\n  void _incrementCounter() {\n    setState(() {\n      _counter++;\n    });\n  }\n\n  @override\n  Widget build(BuildContext context) {\n    return Scaffold(\n      appBar: AppBar(\n        title: Text(widget.title),\n      ),\n      body: Center(\n        child: Column(\n          mainAxisAlignment: MainAxisAlignment.center,\n          children: <Widget>[\n            const Text(\n              'You have pushed the button this many times:',\n            ),\n            Text(\n              '$_counter',\n              style: Theme.of(context).textTheme.headlineMedium,\n            ),\n          ],\n        ),\n      ),\n      floatingActionButton: FloatingActionButton(\n        onPressed: _incrementCounter,\n        tooltip: 'Increment',\n        child: const Icon(Icons.add),\n      ),\n    );\n  }\n}\n".to_string(),
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
                    content: "fastapi==0.104.1\nuvicorn==0.24.0\npydantic==2.5.0\npython-dotenv==1.0.0\n".to_string(),
                    is_executable: false,
                },
                TemplateFile {
                    path: "app/main.py".to_string(),
                    content: "from fastapi import FastAPI\nimport uvicorn\n\napp = FastAPI(\n    title=\"{{project_name}}\",\n    description=\"A FastAPI application\",\n    version=\"0.1.0\"\n)\n\n@app.get(\"/\")\nasync def root():\n    return {\"message\": \"Welcome to {{project_name}}!\"}\n\n@app.get(\"/health\")\nasync def health_check():\n    return {\"status\": \"healthy\"}\n\nif __name__ == \"__main__\":\n    uvicorn.run(app, host=\"0.0.0.0\", port=8000)\n".to_string(),
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
                    content: "{\n  \"name\": \"{{project_name}}\",\n  \"version\": \"0.1.0\",\n  \"private\": true,\n  \"scripts\": {\n    \"dev\": \"next dev\",\n    \"build\": \"next build\",\n    \"start\": \"next start\",\n    \"lint\": \"next lint\"\n  },\n  \"dependencies\": {\n    \"react\": \"^18\",\n    \"react-dom\": \"^18\",\n    \"next\": \"^14\"\n  },\n  \"devDependencies\": {\n    \"eslint\": \"^8\",\n    \"eslint-config-next\": \"^14\"\n  }\n}\n".to_string(),
                    is_executable: false,
                },
                TemplateFile {
                    path: "pages/index.js".to_string(),
                    content: "import Head from 'next/head'\nimport Image from 'next/image'\nimport { Inter } from 'next/font/google'\n\nconst inter = Inter({ subsets: ['latin'] })\n\nexport default function Home() {\n  return (\n    <>\n      <Head>\n        <title>{{project_name}}</title>\n        <meta name=\"description\" content=\"Generated by create next app\" />\n        <meta name=\"viewport\" content=\"width=device-width, initial-scale=1\" />\n        <link rel=\"icon\" href=\"/favicon.ico\" />\n      </Head>\n      <main>\n        <h1>Welcome to {{project_name}}!</h1>\n      </main>\n    </>\n  )\n}\n".to_string(),
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
                    content: "[package]\nname = \"{{project_name}}\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[dependencies]\ntokio = { version = \"1.0\", features = [\"full\"] }\nclap = { version = \"4.0\", features = [\"derive\"] }\nanyhow = \"1.0\"\n".to_string(),
                    is_executable: false,
                },
                TemplateFile {
                    path: "src/main.rs".to_string(),
                    content: "use anyhow::Result;\nuse clap::Parser;\n\n#[derive(Parser)]\n#[command(author, version, about, long_about = None)]\nstruct Args {\n    /// Give a name to greet\n    #[arg(short, long, default_value = \"World\")]\n    name: String,\n    /// Return health status\n    #[arg(long, default_value_t = false)]\n    health: bool,\n}\n\n#[tokio::main]\nasync fn main() -> Result<()> {\n    let args = Args::parse();\n    if args.health {\n        println!(\"{\\\"status\\\":\\\"ok\\\"}\");\n    } else {\n        println!(\"Hello, {}!\", args.name);\n    }\n    Ok(())\n}\n".to_string(),
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

    pub fn create_project(
        &self,
        template_name: &str,
        project_path: &str,
        project_name: &str,
    ) -> Result<()> {
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
