# 📄 PHASE_6_REQUIREMENTS_DESIGN_AGENTS.md

```markdown
# Phase 6: Requirements & Design Agents

## Objectives
Implement a ReAct (Reason-Act) agent framework with specialized agents for requirements elicitation and software design. Enable `kandil agent requirements` to generate SRS documents and `kandil agent design` to create architecture diagrams.

## Prerequisites
- Phase 5 complete (project management, cloud sync)
- Understanding of ReAct pattern (Reason → Act → Observe → Repeat)
- Familiarity with prompt engineering
- Basic knowledge of Mermaid.js for diagrams

## Detailed Sub-Tasks

### Day 1-2: ReAct Agent Framework

1. **Add Dependencies**
```bash
cargo add async-trait # Already added, but ensure version
cargo add thiserror # Better error handling for agents
cargo add uuid # For agent run IDs
cargo add backoff # Retry logic for agent loops
```

2. **Base Agent Trait & ReAct Loop**
```rust
// src/agents/base.rs
use async_trait::async_trait;
use anyhow::Result;
use serde::{Serialize, Deserialize};
use std::time::Duration;
use tokio::time::timeout;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentState {
    pub task: String,
    pub observations: Vec<String>,
    pub current_step: usize,
    pub max_steps: usize,
    pub is_complete: bool,
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
        };
        
        while agent.should_continue(&state) {
            state.current_step += 1;
            
            // Reason: Plan next action
            let plan = backoff::future::retry(backoff::ExponentialBackoff::default(), || async {
                timeout(self.timeout_per_step, agent.plan(&state)).await
                    .map_err(|_| backoff::Error::permanent(anyhow::anyhow!("Plan timeout")))
            }).await?;
            
            // Act: Execute planned action
            let result = backoff::future::retry(backoff::ExponentialBackoff::default(), || async {
                timeout(self.timeout_per_step, agent.act(&plan)).await
                    .map_err(|_| backoff::Error::permanent(anyhow::anyhow!("Act timeout")))
            }).await?;
            
            // Observe: Analyze result
            let observation = agent.observe(&result).await?;
            state.observations.push(observation);
            
            // Check completion
            if self.is_task_complete(&state) {
                state.is_complete = true;
                break;
            }
        }
        
        self.compile_result(&state)
    }
    
    fn is_task_complete(&self, state: &AgentState) -> bool {
        // Heuristic: Check if recent observations indicate completion
        let recent = state.observations.iter().rev().take(3).collect::<Vec<_>>();
        
        if recent.len() < 3 {
            return false;
        }
        
        // Check for completion keywords
        for obs in recent {
            let lower = obs.to_lowercase();
            if lower.contains("complete") || lower.contains("finished") || lower.contains("done") {
                return true;
            }
        }
        
        false
    }
    
    fn compile_result(&self, state: &AgentState) -> Result<AgentResult> {
        Ok(AgentResult {
            final_answer: state.observations.last().cloned().unwrap_or_default(),
            steps_taken: state.current_step,
            observations: state.observations.clone(),
            is_complete: state.is_complete,
        })
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AgentResult {
    pub final_answer: String,
    pub steps_taken: usize,
    pub observations: Vec<String>,
    pub is_complete: bool,
}

#[async_trait]
pub trait Tool: Send + Sync {
    async fn execute(&self, input: &str) -> Result<String>;
    fn name(&self) -> &str;
    fn description(&self) -> &str;
}
```

3. **Tool Registry**
```rust
// src/agents/tools/mod.rs
use super::base::Tool;
use std::collections::HashMap;
use once_cell::sync::Lazy;

static TOOL_REGISTRY: Lazy<HashMap<&'static str, Box<dyn Tool>>> = Lazy::new(|| {
    let mut registry = HashMap::new();
    registry.insert("file_read", Box::new(FileReadTool) as Box<dyn Tool>);
    registry.insert("file_write", Box::new(FileWriteTool) as Box<dyn Tool>);
    registry.insert("web_search", Box::new(WebSearchTool) as Box<dyn Tool>);
    registry.insert("code_analyze", Box::new(CodeAnalysisTool) as Box<dyn Tool>);
    registry
});

pub struct FileReadTool;

#[async_trait::async_trait]
impl Tool for FileReadTool {
    async fn execute(&self, input: &str) -> Result<String> {
        tokio::fs::read_to_string(input)
            .await
            .map_err(|e| anyhow::anyhow!("File read error {}: {}", input, e))
    }
    
    fn name(&self) -> &str { "file_read" }
    fn description(&self) -> &str { "Read file contents" }
}

// ... implement FileWriteTool, WebSearchTool, CodeAnalysisTool similarly

pub fn get_tool(name: &str) -> Option<&'static dyn Tool> {
    TOOL_REGISTRY.get(name).map(|t| t.as_ref())
}
```

### Day 3-4: Requirements Elicitation Agent

1. **Requirements Agent**
```rust
// src/agents/requirements.rs
use super::base::{Agent, AgentState, ReActLoop};
use super::tools::get_tool;
use crate::adapters::ai::factory::AIProviderFactory;
use crate::utils::config::Config;
use anyhow::Result;
use serde::{Serialize, Deserialize};

pub struct RequirementsAgent {
    ai_factory: AIProviderFactory,
}

#[derive(Serialize, Deserialize)]
pub struct RequirementsDoc {
    pub user_stories: Vec<UserStory>,
    pub functional_requirements: Vec<Requirement>,
    pub non_functional_requirements: Vec<Requirement>,
    pub constraints: Vec<String>,
    pub assumptions: Vec<String>,
}

#[derive(Serialize, Deserialize)]
pub struct UserStory {
    pub id: String,
    pub title: String,
    pub description: String,
    pub acceptance_criteria: Vec<String>,
    pub priority: String, // MoSCoW
}

#[derive(Serialize, Deserialize)]
pub struct Requirement {
    pub id: String,
    pub description: String,
    pub priority: String,
    pub category: String,
}

impl RequirementsAgent {
    pub fn new() -> Result<Self> {
        let config = Config::load()?;
        let ai_factory = AIProviderFactory::new(config.ai);
        Ok(Self { ai_factory })
    }
    
    pub async fn elicit_requirements(&self, idea: &str) -> Result<RequirementsDoc> {
        let task = format!(
            "Elicit comprehensive requirements for: {}\n
            Follow this process:
            1. Analyze the idea and identify stakeholders
            2. Elicit user stories using 'As a [role] I want [feature] so that [benefit]'
            3. Derive functional requirements from user stories
            4. Identify non-functional requirements (performance, security, scalability)
            5. Document constraints and assumptions
            6. Prioritize using MoSCoW (Must, Should, Could, Won't)
            7. Validate completeness
            
            Return a structured JSON response with:
            - user_stories: array
            - functional_requirements: array
            - non_functional_requirements: array
            - constraints: array
            - assumptions: array",
            idea
        );
        
        let loop_engine = ReActLoop::new(7); // Max 7 steps
        let result = loop_engine.run(self, &task).await?;
        
        // Parse JSON from final answer
        let doc: RequirementsDoc = serde_json::from_str(&result.final_answer)
            .map_err(|e| anyhow::anyhow!("Failed to parse requirements JSON: {}", e))?;
        
        Ok(doc)
    }
}

#[async_trait::async_trait]
impl Agent for RequirementsAgent {
    async fn plan(&self, state: &AgentState) -> Result<String> {
        let ai = self.ai_factory.create().await?;
        
        let prompt = format!(
            "You are a requirements engineer. Current task: {}
            Observations so far: {:?}
            Step {} of {}.
            
            What is your next action? Choose from:
            - analyze_idea
            - elicit_user_stories
            - derive_functional_reqs
            - identify_non_functional_reqs
            - document_constraints
            - validate_requirements
            
            Return a clear plan for the next step.",
            state.task,
            state.observations,
            state.current_step,
            state.max_steps
        );
        
        ai.chat(&prompt, None).await
    }
    
    async fn act(&self, plan: &str) -> Result<String> {
        let ai = self.ai_factory.create().await?;
        
        match plan.trim() {
            "analyze_idea" => {
                ai.chat("Analyze the idea and identify stakeholders. List them.", None).await
            }
            "elicit_user_stories" => {
                ai.chat("Elicit 5-10 user stories with acceptance criteria.", None).await
            }
            "derive_functional_reqs" => {
                ai.chat("Convert user stories into functional requirements.", None).await
            }
            "identify_non_functional_reqs" => {
                ai.chat("List non-functional requirements (performance, security, scalability).", None).await
            }
            "document_constraints" => {
                ai.chat("Document technical and business constraints.", None).await
            }
            "validate_requirements" => {
                ai.chat("Review requirements for completeness and traceability.", None).await
            }
            _ => ai.chat(&format!("Execute plan: {}", plan), None).await
        }
    }
    
    async fn observe(&self, result: &str) -> Result<String> {
        let ai = self.ai_factory.create().await?;
        
        let prompt = format!(
            "You executed an action. Result: {}.
            
            What did you learn? What should be the next step?
            If you have enough requirements, respond with 'COMPLETE'.",
            result
        );
        
        ai.chat(&prompt, None).await
    }
}
```

2. **CLI Integration**
```rust
// src/cli/agent.rs
use crate::agents::requirements::RequirementsAgent;
use crate::agents::design::DesignAgent;
use anyhow::Result;
use std::path::Path;

pub async fn handle_agent(sub: AgentSub) -> Result<()> {
    match sub {
        AgentSub::Requirements { idea, output } => {
            let agent = RequirementsAgent::new()?;
            let doc = agent.elicit_requirements(&idea).await?;
            
            // Save to file
            let output_path = Path::new(&output);
            let json = serde_json::to_string_pretty(&doc)?;
            std::fs::write(output_path, json)?;
            
            println!("✅ Requirements saved to {}", output);
            println!("  User stories: {}", doc.user_stories.len());
            println!("  Functional reqs: {}", doc.functional_requirements.len());
            println!("  Non-functional reqs: {}", doc.non_functional_requirements.len());
            
            // Generate Markdown version
            generate_srs_markdown(&doc, output_path.with_extension("md"))?;
            println!("  Also saved as SRS.md");
            
            Ok(())
        }
        // Design agent handled in next section
    }
}

fn generate_srs_markdown(doc: &RequirementsDoc, path: Path) -> Result<()> {
    let mut content = String::from("# Software Requirements Specification\n\n");
    
    content.push_str("## 1. User Stories\n\n");
    for story in &doc.user_stories {
        content.push_str(&format!(
            "### {}\n- **As a**: {}\n- **I want**: {}\n- **So that**: {}\n- **Priority**: {}\n\n",
            story.id, story.title, story.description,
            story.acceptance_criteria.join(", "), story.priority
        ));
    }
    
    // ... add functional/non-functional requirements sections
    
    std::fs::write(path, content)?;
    Ok(())
}
```

### Day 5-6: Design Agent with UML Generation

1. **Design Agent**
```rust
// src/agents/design.rs
use super::base::{Agent, AgentState, ReActLoop};
use super::tools::get_tool;
use crate::adapters::ai::factory::AIProviderFactory;
use crate::utils::config::Config;
use anyhow::Result;
use serde::{Serialize, Deserialize};

pub struct DesignAgent {
    ai_factory: AIProviderFactory,
}

#[derive(Serialize, Deserialize)]
pub struct DesignDoc {
    pub architecture: String,
    pub components: Vec<Component>,
    pub data_models: Vec<DataModel>,
    pub mermaid_diagrams: Vec<MermaidDiagram>,
    pub tech_stack: TechStack,
}

#[derive(Serialize, Deserialize)]
pub struct Component {
    pub name: String,
    pub responsibility: String,
    pub interfaces: Vec<String>,
    pub dependencies: Vec<String>,
}

#[derive(Serialize, Deserialize)]
pub struct DataModel {
    pub name: String,
    pub fields: Vec<Field>,
    pub relationships: Vec<String>,
}

#[derive(Serialize, Deserialize)]
pub struct Field {
    pub name: String,
    pub type_: String,
    pub constraints: Vec<String>,
}

#[derive(Serialize, Deserialize)]
pub struct MermaidDiagram {
    pub title: String,
    pub diagram: String,
    pub diagram_type: String, // sequence, class, component
}

#[derive(Serialize, Deserialize)]
pub struct TechStack {
    pub language: String,
    pub framework: String,
    pub database: String,
    pub architecture_pattern: String,
}

impl DesignAgent {
    pub fn new() -> Result<Self> {
        let config = Config::load()?;
        let ai_factory = AIProviderFactory::new(config.ai);
        Ok(Self { ai_factory })
    }
    
    pub async fn generate_design(&self, requirements: &str, lang: &str) -> Result<DesignDoc> {
        let task = format!(
            "Design a {} architecture for these requirements.
            
            Requirements:
            {}
            
            Follow this process:
            1. Choose architecture pattern (Clean, Hexagonal, MVC, etc.)
            2. Identify main components and their responsibilities
            3. Design data models and relationships
            4. Create Mermaid UML diagrams (class, sequence, component)
            5. Recommend tech stack
            6. Document API contracts
            
            Return structured JSON with:
            - architecture: string
            - components: array
            - data_models: array
            - mermaid_diagrams: array
            - tech_stack: object",
            lang,
            requirements
        );
        
        let loop_engine = ReActLoop::new(6);
        let result = loop_engine.run(self, &task).await?;
        
        let doc: DesignDoc = serde_json::from_str(&result.final_answer)
            .map_err(|e| anyhow::anyhow!("Failed to parse design JSON: {}", e))?;
        
        // Validate and render diagrams
        self.validate_mermaid(&doc)?;
        
        Ok(doc)
    }
    
    fn validate_mermaid(&self, doc: &DesignDoc) -> Result<()> {
        for diagram in &doc.mermaid_diagrams {
            // Basic syntax validation
            if !diagram.diagram.starts_with("graph") 
                && !diagram.diagram.starts_with("classDiagram")
                && !diagram.diagram.starts_with("sequenceDiagram") {
                return Err(anyhow::anyhow!("Invalid Mermaid diagram: {}", diagram.title));
            }
        }
        Ok(())
    }
}

#[async_trait::async_trait]
impl Agent for DesignAgent {
    async fn plan(&self, state: &AgentState) -> Result<String> {
        let ai = self.ai_factory.create().await?;
        
        let prompt = format!(
            "You are a software architect. Current task: {}
            Observations: {:?}
            
            What is your next design action? Choose from:
            - choose_pattern
            - identify_components
            - design_data_models
            - create_uml
            - recommend_tech
            - design_apis
            
            Return the action name and a brief plan.",
            state.task,
            state.observations
        );
        
        ai.chat(&prompt, None).await
    }
    
    async fn act(&self, plan: &str) -> Result<String> {
        let ai = self.ai_factory.create().await?;
        
        match plan.trim() {
            "choose_pattern" => ai.chat(
                "Choose the best architecture pattern and justify your choice", None).await,
            "identify_components" => ai.chat(
                "Identify system components and their responsibilities", None).await,
            "design_data_models" => ai.chat(
                "Design data models with fields and relationships", None).await,
            "create_uml" => ai.chat(
                "Create Mermaid UML diagrams (class, sequence, component)", None).await,
            "recommend_tech" => ai.chat(
                "Recommend tech stack and justify choices", None).await,
            "design_apis" => ai.chat(
                "Design API contracts and endpoints", None).await,
            _ => ai.chat(&format!("Execute design action: {}", plan), None).await
        }
    }
    
    async fn observe(&self, result: &str) -> Result<String> {
        let ai = self.ai_factory.create().await?;
        
        let prompt = format!(
            "Design action completed. Result: {}.
            
            What did you accomplish? Should you continue or is the design complete?
            Respond with 'COMPLETE' if design is adequate.",
            result
        );
        
        ai.chat(&prompt, None).await
    }
}
```

2. **Mermaid Integration**
```rust
// src/code/mermaid.rs
use anyhow::Result;
use std::fs;
use std::path::Path;

pub struct MermaidRenderer;

impl MermaidRenderer {
    pub fn render_and_save(diagram: &str, output_path: &Path) -> Result<()> {
        // Wrap in markdown for GitHub compatibility
        let markdown = format!("```mermaid\n{}\n```", diagram);
        fs::write(output_path, markdown)?;
        Ok(())
    }
    
    pub fn render_all(diagrams: &[crate::agents::design::MermaidDiagram], 
                     output_dir: &Path) -> Result<Vec<PathBuf>> {
        fs::create_dir_all(output_dir)?;
        
        let mut paths = Vec::new();
        for (i, diagram) in diagrams.iter().enumerate() {
            let filename = format!("{:02}_{}.md", i, 
                diagram.title.to_lowercase().replace(" ", "_"));
            let path = output_dir.join(filename);
            
            Self::render_and_save(&diagram.diagram, &path)?;
            paths.push(path);
        }
        
        Ok(paths)
    }
}
```

### Day 7-8: Agent CLI & Integration

1. **Agent Commands**
```rust
// src/cli/agent.rs (continued)
#[derive(Subcommand)]
pub enum AgentSub {
    Requirements {
        #[arg(short, long, help = "Project idea or description")]
        idea: String,
        #[arg(short, long, help = "Output JSON file path")]
        output: String,
    },
    Design {
        #[arg(short, long, help = "Requirements JSON file path")]
        reqs: String,
        #[arg(short, long, help = "Target language")]
        lang: String,
        #[arg(short, long, help = "Output directory")]
        output_dir: String,
    },
}

pub async fn handle_agent(sub: AgentSub) -> Result<()> {
    match sub {
        AgentSub::Requirements { idea, output } => {
            // ... requirements logic from above
        }
        AgentSub::Design { reqs, lang, output_dir } => {
            let requirements = std::fs::read_to_string(&reqs)?;
            let agent = DesignAgent::new()?;
            
            let design_doc = agent.generate_design(&requirements, &lang).await?;
            
            // Save JSON
            let output_path = Path::new(&output_dir).join("design.json");
            let json = serde_json::to_string_pretty(&design_doc)?;
            std::fs::write(&output_path, json)?;
            
            // Render Mermaid diagrams
            let mermaid_dir = Path::new(&output_dir).join("diagrams");
            let paths = crate::code::mermaid::MermaidRenderer::render_all(
                &design_doc.mermaid_diagrams,
                &mermaid_dir,
            )?;
            
            println!("✅ Design saved to {}", output_dir);
            println!("  Diagrams generated: {}", paths.len());
            println!("  Architecture: {}", design_doc.architecture);
            
            // Generate design.md
            generate_design_markdown(&design_doc, Path::new(&output_dir).join("DESIGN.md"))?;
            
            Ok(())
        }
    }
}

fn generate_design_markdown(doc: &DesignDoc, path: PathBuf) -> Result<()> {
    let mut content = String::from("# Architecture Design Document\n\n");
    
    content.push_str(&format!("## Architecture Pattern\n{}\n\n", doc.architecture));
    
    content.push_str("## Tech Stack\n");
    content.push_str(&format!("- Language: {}\n", doc.tech_stack.language));
    content.push_str(&format!("- Framework: {}\n", doc.tech_stack.framework));
    content.push_str(&format!("- Database: {}\n", doc.tech_stack.database));
    
    // Add components, data models sections...
    
    std::fs::write(path, content)?;
    Ok(())
}
```

2. **Agent Pipeline**
```rust
// src/pipeline/requirements_to_design.rs
use crate::agents::requirements::RequirementsAgent;
use crate::agents::design::DesignAgent;
use anyhow::Result;
use std::path::Path;

pub async fn run_pipeline(idea: &str, output_dir: &Path) -> Result<()> {
    fs::create_dir_all(output_dir)?;
    
    println!("🚀 Starting Requirements → Design pipeline");
    
    // Phase 1: Requirements
    println!("📋 Eliciting requirements...");
    let req_agent = RequirementsAgent::new()?;
    let requirements = req_agent.elicit_requirements(idea).await?;
    
    let req_json = output_dir.join("requirements.json");
    fs::write(&req_json, serde_json::to_string_pretty(&requirements)?)?;
    println!("  ✓ Saved requirements.json");
    
    // Phase 2: Design
    println!("🎨 Generating design...");
    let design_agent = DesignAgent::new()?;
    let design = design_agent.generate_design(
        &fs::read_to_string(&req_json)?,
        "flutter" // Auto-detect based on template
    ).await?;
    
    let design_json = output_dir.join("design.json");
    fs::write(&design_json, serde_json::to_string_pretty(&design)?)?;
    println!("  ✓ Saved design.json");
    
    // Render diagrams
    let diagram_dir = output_dir.join("diagrams");
    crate::code::mermaid::MermaidRenderer::render_all(&design.mermaid_diagrams, &diagram_dir)?;
    println!("  ✓ Generated {} diagrams", design.mermaid_diagrams.len());
    
    println!("✅ Pipeline complete! Output: {}", output_dir.display());
    
    Ok(())
}
```

## Tools & Dependencies
| Crate | Version | Purpose |
|-------|---------|---------|
| async-trait | 0.1 | Agent traits |
| thiserror | 1.0 | Error handling |
| uuid | 1.7 | Agent run IDs |
| backoff | 0.4 | Retry logic |
| serde_json | 1.0 | Structured output |

## Testing Strategy
- **Unit**: Mock AI responses for deterministic tests (90% coverage)
- **Integration**: Run full pipeline with sample idea
- **Manual**: Generate SRS for "cinema app", verify completeness
- **Validation**: Check generated JSON against schema

## Deliverables
- ✅ ReAct loop framework with tool integration
- ✅ Requirements Agent with user story generation
- ✅ Design Agent with Mermaid UML output
- ✅ `kandil agent requirements --idea="..."` command
- ✅ `kandil agent design --reqs=...` command
- ✅ Pipeline: idea → SRS → Design → Diagrams
- ✅ Structured JSON output (parseable by CI/CD)
- ✅ 90% test coverage on agents

## Timeline Breakdown
- **Days 1-2**: ReAct framework + tool registry
- **Days 3-4**: Requirements agent + CLI
- **Days 5-6**: Design agent + Mermaid integration
- **Days 7-8**: Pipeline + integration
- **Days 9-14**: Testing & polish

## Success Criteria
- Requirements doc has ≥5 user stories, ≥10 functional reqs
- Design doc includes ≥3 UML diagrams (class, sequence, component)
- Pipeline completes in <30s for simple idea
- Generated Mermaid is valid (renders on GitHub)
- Agent loop completes in ≤7 iterations
- JSON output matches schema (validate with `jsonschema`)
- Manual test: Cinema app idea → workable design

## Potential Risks & Mitigations

| Risk | Mitigation |
|------|------------|
| Agent loops infinitely | Max 7 steps + timeout per step (120s) |
| JSON parse fails | Validate with serde; fallback to retry with simpler prompt |
| Tool execution crashes | Wrap each tool call in `panic::catch_unwind` |
| AI produces inconsistent output | Use structured prompts; validate output format |
| Mermaid syntax invalid | Escape special chars; validate before saving |
| Cost overrun (cloud AI) | Default to Ollama for agents; cloud only on demand |

---

**Next**: Proceed to PHASE_7_CODE_TEST_AGENTS_SIMULATIONS.md after agent pipeline validation with real project example.