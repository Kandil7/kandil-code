# 📄 PHASE_7_CODE_TEST_AGENTS_SIMULATIONS.md

```markdown
# Phase 7: Code & Test Agents + Basic Simulations

## Objectives
Implement specialized agents for code generation, test execution, and professional role simulations (PM, BA). Enable `kandil agent code`, `kandil agent test`, and `kandil simulate pm/ba` commands. Build the foundation for multi-agent collaboration.

## Prerequisites
- Phase 6 complete (ReAct framework, requirements/design agents)
- Code generation templates from Phase 2
- Test infrastructure from Phase 4
- Understanding of role-playing prompts

## Detailed Sub-Tasks

### Day 1-2: Code Generation Agent

1. **Code Agent Implementation**
```rust
// src/agents/code.rs
use super::base::{Agent, AgentState, ReActLoop};
use crate::adapters::ai::factory::AIProviderFactory;
use crate::templates::engine::TemplateEngine;
use crate::utils::config::Config;
use anyhow::Result;
use std::path::Path;

pub struct CodeAgent {
    ai_factory: AIProviderFactory,
    template_engine: TemplateEngine,
}

impl CodeAgent {
    pub fn new() -> Result<Self> {
        let config = Config::load()?;
        Ok(Self {
            ai_factory: AIProviderFactory::new(config.ai),
            template_engine: TemplateEngine::new(),
        })
    }
    
    pub async fn generate_code(&self, design_doc: &Path, lang: &str) -> Result<CodeOutput> {
        let design_content = std::fs::read_to_string(design_doc)?;
        
        let task = format!(
            "Generate production-ready {} code from this design.
            
            Design:
            {}
            
            Follow:
            1. Map components to files/folders
            2. Generate data models
            3. Implement business logic
            4. Add error handling
            5. Include logging
            6. Follow language best practices
            
            Return structured plan with file paths and code blocks.",
            lang.to_uppercase(),
            design_content
        );
        
        let loop_engine = ReActLoop::new(5);
        let result = loop_engine.run(self, &task).await?;
        
        self.parse_and_write_code(&result.final_answer, lang).await
    }
    
    async fn parse_and_write_code(&self, response: &str, lang: &str) -> Result<CodeOutput> {
        // Parse code blocks from AI response
        let code_blocks = self.extract_code_blocks(response)?;
        let mut files = Vec::new();
        
        for block in code_blocks {
            let path = self.determine_file_path(&block, lang)?;
            
            // Create directories
            if let Some(parent) = path.parent() {
                tokio::fs::create_dir_all(parent).await?;
            }
            
            // Write file
            tokio::fs::write(&path, &block.code).await?;
            files.push(path.to_string_lossy().to_string());
        }
        
        // Run post-generation tasks
        self.post_generation(lang).await?;
        
        Ok(CodeOutput {
            files_generated: files,
            language: lang.to_string(),
        })
    }
    
    fn extract_code_blocks(&self, response: &str) -> Result<Vec<CodeBlock>> {
        // Parse markdown code blocks
        let mut blocks = Vec::new();
        let mut lines = response.lines();
        
        while let Some(line) = lines.next() {
            if line.starts_with("```") {
                let lang = line.trim_start_matches("```").trim().to_string();
                let mut code = String::new();
                
                while let Some(code_line) = lines.next() {
                    if code_line.starts_with("```") {
                        break;
                    }
                    code.push_str(code_line);
                    code.push('\n');
                }
                
                blocks.push(CodeBlock {
                    language: lang,
                    code,
                    file_path: None, // AI should specify in comment
                });
            }
        }
        
        Ok(blocks)
    }
    
    fn determine_file_path(&self, block: &CodeBlock, lang: &str) -> Result<PathBuf> {
        // Extract file path from first comment
        let first_line = block.code.lines().next().unwrap_or("");
        
        if first_line.starts_with("// File:") || first_line.starts_with("# File:") {
            let path = first_line.split(":").nth(1).unwrap().trim();
            Ok(PathBuf::from(path))
        } else {
            // Default path based on language
            Ok(match lang {
                "flutter" => PathBuf::from("lib/generated.dart"),
                "python" => PathBuf::from("generated.py"),
                "rust" => PathBuf::from("src/generated.rs"),
                _ => PathBuf::from("generated.txt"),
            })
        }
    }
    
    async fn post_generation(&self, lang: &str) -> Result<()> {
        match lang {
            "flutter" => {
                tokio::process::Command::new("flutter")
                    .args(&["pub", "get"])
                    .output().await?;
            }
            "python" => {
                tokio::process::Command::new("pip")
                    .args(&["install", "-r", "requirements.txt"])
                    .output().await?;
            }
            "rust" => {
                tokio::process::Command::new("cargo")
                    .args(&["check"])
                    .output().await?;
            }
            _ => {}
        }
        Ok(())
    }
}

#[derive(Debug)]
struct CodeBlock {
    language: String,
    code: String,
    file_path: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CodeOutput {
    pub files_generated: Vec<String>,
    pub language: String,
}

#[async_trait::async_trait]
impl Agent for CodeAgent {
    async fn plan(&self, state: &AgentState) -> Result<String> {
        let ai = self.ai_factory.create().await?;
        let prompt = format!(
            "Code generation step {}. Task: {}
            Progress: {:?}
            Next action?",
            state.current_step, state.task, state.observations
        );
        ai.chat(&prompt, None).await
    }
    
    async fn act(&self, plan: &str) -> Result<String> {
        // Delegate to template engine for structured generation
        if plan.contains("use_template") {
            self.template_engine.generate_from_plan(plan).await
        } else {
            let ai = self.ai_factory.create().await?;
            ai.chat(plan, None).await
        }
    }
    
    async fn observe(&self, result: &str) -> Result<String> {
        let ai = self.ai_factory.create().await?;
        ai.chat(&format!("Code generation result: {}", result), None).await
    }
}
```

### Day 3-4: Test Agent & Execution

1. **Test Execution Agent**
```rust
// src/agents/test.rs
use super::base::{Agent, AgentState, ReActLoop};
use crate::adapters::ai::factory::AIProviderFactory;
use crate::utils::config::Config;
use anyhow::Result;
use std::process::Command;

pub struct TestAgent {
    ai_factory: AIProviderFactory,
}

impl TestAgent {
    pub fn new() -> Result<Self> {
        let config = Config::load()?;
        Ok(Self {
            ai_factory: AIProviderFactory::new(config.ai),
        })
    }
    
    pub async fn run_test_suite(&self, project_path: &str) -> Result<TestResults> {
        let task = format!(
            "Execute test suite for project at {}.
            1. Detect test framework
            2. Run tests
            3. Analyze coverage
            4. Identify failures
            5. Suggest fixes
            
            Return structured results.",
            project_path
        );
        
        let loop_engine = ReActLoop::new(5);
        let result = loop_engine.run(self, &task).await?;
        
        self.parse_test_results(&result.final_answer)
    }
    
    fn parse_test_results(&self, output: &str) -> Result<TestResults> {
        // Parse test command output
        let passed = output.lines()
            .filter(|l| l.contains("passed") || l.contains("✓"))
            .count();
            
        let failed = output.lines()
            .filter(|l| l.contains("failed") || l.contains("✗"))
            .count();
        
        Ok(TestResults {
            total_tests: passed + failed,
            passed,
            failed,
            coverage: self.extract_coverage(output),
        })
    }
    
    fn extract_coverage(&self, output: &str) -> Option<f64> {
        // Look for coverage percentages
        for line in output.lines() {
            if let Some(start) = line.find("coverage: ") {
                if let Some(end) = line[start..].find('%') {
                    let num = &line[start + 10..start + end];
                    return num.parse().ok();
                }
            }
        }
        None
    }
    
    pub async fn fix_failing_tests(&self, test_output: &str) -> Result<String> {
        let ai = self.ai_factory.create().await?;
        
        let prompt = format!(
            "These tests are failing:
            {}
            
            Analyze the failures and provide:
            1. Root cause analysis
            2. Fixed test code
            3. Explanation of changes",
            test_output
        );
        
        ai.chat(&prompt, None).await
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TestResults {
    pub total_tests: usize,
    pub passed: usize,
    pub failed: usize,
    pub coverage: Option<f64>,
}

#[async_trait::async_trait]
impl Agent for TestAgent {
    async fn plan(&self, state: &AgentState) -> Result<String> {
        let ai = self.ai_factory.create().await?;
        ai.chat(&format!("Test execution step {}. Plan next action.", state.current_step), None).await
    }
    
    async fn act(&self, plan: &str) -> Result<String> {
        // Execute actual test commands
        if plan.contains("run_tests") {
            self.execute_test_command().await
        } else {
            let ai = self.ai_factory.create().await?;
            ai.chat(plan, None).await
        }
    }
    
    async fn observe(&self, result: &str) -> Result<String> {
        Ok(format!("Test execution completed: {}", result))
    }
    
    async fn execute_test_command(&self) -> Result<String> {
        // Detect project type and run appropriate tests
        let ws = crate::core::workspace::Workspace::detect()?;
        
        let output = match ws.project_type.as_str() {
            "flutter" => Command::new("flutter").args(&["test"]).output()?,
            "python" => Command::new("pytest").output()?,
            "rust" => Command::new("cargo").args(&["test"]).output()?,
            _ => return Ok("No test framework detected".to_string()),
        };
        
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }
}
```

2. **Test-Driven Development Agent**
```rust
// src/agents/tdd.rs
use super::base::{Agent, AgentState, ReActLoop};
use crate::adapters::ai::factory::AIProviderFactory;
use crate::utils::config::Config;
use anyhow::Result;

pub struct TDDAgent {
    ai_factory: AIProviderFactory,
}

impl TDDAgent {
    pub fn new() -> Result<Self> {
        let config = Config::load()?;
        Ok(Self {
            ai_factory: AIProviderFactory::new(config.ai),
        })
    }
    
    pub async fn tdd_cycle(&self, feature: &str) -> Result<TDDResult> {
        let task = format!(
            "Implement feature '{}' using TDD:
            1. Write failing test
            2. Run test (should fail)
            3. Write minimal code to pass
            4. Run test (should pass)
            5. Refactor
            6. Repeat until complete",
            feature
        );
        
        let loop_engine = ReActLoop::new(6);
        let result = loop_engine.run(self, &task).await?;
        
        Ok(TDDResult {
            tests_written: 1,
            code_written: 1,
            cycles_completed: result.steps_taken,
        })
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TDDResult {
    pub tests_written: usize,
    pub code_written: usize,
    pub cycles_completed: usize,
}
```

### Day 5-6: PM Simulation (Product Manager)

1. **PM Simulation Agent**
```rust
// src/agents/simulate/pm.rs
use crate::adapters::ai::factory::AIProviderFactory;
use crate::utils::config::Config;
use anyhow::Result;

pub struct PMSimulation {
    ai_factory: AIProviderFactory,
}

impl PMSimulation {
    pub fn new() -> Result<Self> {
        let config = Config::load()?;
        Ok(Self {
            ai_factory: AIProviderFactory::new(config.ai),
        })
    }
    
    pub async fn generate_okr(&self, goals: &[&str]) -> Result<OKRDocument> {
        let ai = self.ai_factory.create().await?;
        
        let prompt = format!(
            r#"You are a VP of Product. Create OKRs for these goals:
            {}
            
            Format as JSON:
            {{
              "objectives": [
                {{
                  "objective": "string",
                  "key_results": [
                    {{
                      "kr": "string",
                      "target": number,
                      "current": number
                    }}
                  ]
                }}
              ]
            }}"#,
            goals.join("\n")
        );
        
        let response = ai.chat(&prompt, None).await?;
        let okr: OKRDocument = serde_json::from_str(&response)
            .map_err(|e| anyhow::anyhow!("OKR parsing failed: {}", e))?;
        
        Ok(okr)
    }
    
    pub async fn create_roadmap(&self, requirements: &str, quarters: usize) -> Result<Roadmap> {
        let ai = self.ai_factory.create().await?;
        
        let prompt = format!(
            r#"Create a {}-quarter product roadmap for these requirements:
            {}
            
            Use MoSCoW prioritization. Format as:
            - Q1: Must-have features
            - Q2: Should-have features
            - Q3: Could-have features
            - Q4: Won't-have (yet) features
            
            Include milestones and dependencies."#,
            quarters, requirements
        );
        
        let roadmap_text = ai.chat(&prompt, None).await?;
        self.parse_roadmap(&roadmap_text)
    }
    
    pub async def prioritize_backlog(&self, items: &[BacklogItem]) -> Result<Vec<BacklogItem>> {
        let ai = self.ai_factory.create().await?;
        
        let items_json = serde_json::to_string(items)?;
        let prompt = format!(
            "Prioritize this backlog using WSJF (Weighted Shortest Job First):
            {}
            
            Consider: user value, time criticality, risk reduction, job size.
            Return sorted JSON array.",
            items_json
        );
        
        let response = ai.chat(&prompt, None).await?;
        let prioritized: Vec<BacklogItem> = serde_json::from_str(&response)?;
        Ok(prioritized)
    }
    
    fn parse_roadmap(&self, text: &str) -> Result<Roadmap> {
        let mut roadmap = Roadmap { quarters: vec![] };
        
        for (i, section) in text.split("Q").enumerate().skip(1) {
            let lines: Vec<&str> = section.lines().collect();
            if let Some(header) = lines.first() {
                let quarter_num: usize = header.chars().next()
                    .and_then(|c| c.to_digit(10))
                    .unwrap_or(i) as usize;
                
                let features = lines.iter()
                    .skip(1)
                    .filter(|l| l.trim_start().starts_with("-"))
                    .map(|l| l.trim_start_matches("-").trim().to_string())
                    .collect();
                
                roadmap.quarters.push(Quarter {
                    number: quarter_num,
                    features,
                });
            }
        }
        
        Ok(roadmap)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OKRDocument {
    pub objectives: Vec<Objective>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Objective {
    pub objective: String,
    pub key_results: Vec<KeyResult>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct KeyResult {
    pub kr: String,
    pub target: f64,
    pub current: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Roadmap {
    pub quarters: Vec<Quarter>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Quarter {
    pub number: usize,
    pub features: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BacklogItem {
    pub id: String,
    pub description: String,
    pub user_value: u32,
    pub effort: u32,
    pub risk: u32,
}
```

2. **BA Simulation (Business Analyst)**
```rust
// src/agents/simulate/ba.rs
use crate::adapters::ai::factory::AIProviderFactory;
use crate::utils::config::Config;
use anyhow::Result;

pub struct BASimulation {
    ai_factory: AIProviderFactory,
}

impl BASimulation {
    pub fn new() -> Result<Self> {
        let config = Config::load()?;
        Ok(Self {
            ai_factory: AIProviderFactory::new(config.ai),
        })
    }
    
    pub async fn swot_analysis(&self, idea: &str) -> Result<SWOT> {
        let ai = self.ai_factory.create().await?;
        
        let prompt = format!(
            r#"Perform SWOT analysis for: {}
            
            Return JSON:
            {{
              "strengths": ["..."],
              "weaknesses": ["..."],
              "opportunities": ["..."],
              "threats": ["..."]
            }}"#,
            idea
        );
        
        let response = ai.chat(&prompt, None).await?;
        let swot: SWOT = serde_json::from_str(&response)?;
        Ok(swot)
    }
    
    pub async fn raci_matrix(&self, roles: &[&str], tasks: &[&str]) -> Result<RACIMatrix> {
        let ai = self.ai_factory.create().await?;
        
        let prompt = format!(
            r#"Create RACI matrix for:
            Roles: {:?}
            Tasks: {:?}
            
            R = Responsible, A = Accountable, C = Consulted, I = Informed
            Return as markdown table."#,
            roles, tasks
        );
        
        let matrix_md = ai.chat(&prompt, None).await?;
        self.parse_raci(&matrix_md)
    }
    
    pub async fn process_map(&self, process: &str) -> Result<ProcessMap> {
        let ai = self.ai_factory.create().await?;
        
        let prompt = format!(
            "Create a process flow diagram in Mermaid syntax for: {}",
            process
        );
        
        let mermaid = ai.chat(&prompt, None).await?;
        Ok(ProcessMap { mermaid })
    }
    
    fn parse_raci(&self, markdown: &str) -> Result<RACIMatrix> {
        // Parse markdown table
        Ok(RACIMatrix {
            markdown: markdown.to_string(),
        })
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SWOT {
    pub strengths: Vec<String>,
    pub weaknesses: Vec<String>,
    pub opportunities: Vec<String>,
    pub threats: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RACIMatrix {
    pub markdown: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProcessMap {
    pub mermaid: String,
}
```

### Day 7-8: Simulation CLI Integration

1. **Simulation Commands**
```rust
// src/cli/simulate.rs
use crate::agents::simulate::{pm::PMSimulation, ba::BASimulation};
use anyhow::Result;

pub async fn handle_simulate(sub: SimulateSub) -> Result<()> {
    match sub {
        SimulateSub::Pm { goal, output } => {
            let pm = PMSimulation::new()?;
            let goals: Vec<&str> = goal.split(',').collect();
            let okr = pm.generate_okr(&goals).await?;
            
            let json = serde_json::to_string_pretty(&okr)?;
            std::fs::write(&output, json)?;
            
            println!("✅ OKR saved to {}", output);
            for obj in okr.objectives {
                println!("  🎯 {}", obj.objective);
            }
        }
        SimulateSub::Ba { idea, analysis } => {
            let ba = BASimulation::new()?;
            let swot = ba.swot_analysis(&idea).await?;
            
            println!("🔍 SWOT Analysis for '{}':", idea);
            println!("\n📈 Strengths:");
            for s in swot.strengths {
                println!("  • {}", s);
            }
            // ... print other sections
            
            if let Some(output) = analysis {
                let json = serde_json::to_string_pretty(&swot)?;
                std::fs::write(&output, json)?;
                println!("\n✅ Saved to {}", output);
            }
        }
        SimulateSub::Raci { roles, tasks, output } => {
            let ba = BASimulation::new()?;
            let roles_vec: Vec<&str> = roles.split(',').collect();
            let tasks_vec: Vec<&str> = tasks.split(',').collect();
            let matrix = ba.raci_matrix(&roles_vec, &tasks_vec).await?;
            
            std::fs::write(&output, &matrix.markdown)?;
            println!("✅ RACI matrix saved to {}", output);
            println!("{}", matrix.markdown);
        }
    }
    Ok(())
}

#[derive(Subcommand)]
pub enum SimulateSub {
    Pm {
        #[arg(short, long, help = "Goals (comma-separated)")]
        goal: String,
        #[arg(short, long, help = "Output JSON file")]
        output: String,
    },
    Ba {
        #[arg(short, long, help = "Project idea")]
        idea: String,
        #[arg(short, long, help = "Optional output JSON")]
        analysis: Option<String>,
    },
    Raci {
        #[arg(short, long, help = "Roles (comma-separated)")]
        roles: String,
        #[arg(short, long, help = "Tasks (comma-separated)")]
        tasks: String,
        #[arg(short, long, help = "Output markdown file")]
        output: String,
    },
}
```

## Tools & Dependencies
| Crate | Version | Purpose |
|-------|---------|---------|
| tokio-process | 0.2 | Async test execution |
| serde_json | 1.0 | Structured output |
| chrono | 0.4 | Timestamps for simulations |

## Testing Strategy
- **Unit**: Mock AI responses; verify agent state transitions (85% coverage)
- **Integration**: Run full TDD cycle on sample feature
- **Manual**: Use PM simulation for real product planning; validate OKR quality
- **Cross-validation**: Compare agent outputs with human expert reviews

## Deliverables
- ✅ `kandil agent code` generates code from design
- ✅ `kandil agent test` runs tests and analyzes results
- ✅ `kandil agent tdd` performs full TDD cycle
- ✅ `kandil simulate pm` generates OKRs and roadmaps
- ✅ `kandil simulate ba` creates SWOT and RACI matrices
- ✅ Multi-agent collaboration (code agent uses design agent output)
- ✅ Agent memory integration (context from previous phases)
- ✅ 85% test coverage on agent modules

## Timeline Breakdown
- **Days 1-2**: Code generation agent + templates
- **Days 3-4**: Test execution agent + TDD cycle
- **Days 5-6**: PM simulation (OKR, roadmap, backlog)
- **Days 7-8**: BA simulation (SWOT, RACI, process maps)
- **Days 9-14**: Integration, testing, and agent orchestration

## Success Criteria
- Code agent generates compilable code for 3 languages
- Test agent correctly identifies pass/fail status
- TDD agent completes 3 cycles for simple feature
- PM OKRs follow Google format and are measurable
- BA SWOT covers business/technical dimensions
- Agent can call other agents (composition)
- CI passes with mocked LLM responses
- Manual test: Full pipeline from idea → code → tests

## Potential Risks & Mitigations

| Risk | Mitigation |
|------|------------|
| Generated code doesn't compile | Add syntax check step; fail fast with clear error |
| Test agent misinterprets output | Use structured test reporters (JUnit XML) |
| Simulations are too generic | Fine-tune prompts with role-specific examples |
- Agent recursion depth exceeded | Limit composition to 2 levels |
- Tool execution security | Sandboxing in Phase 13 |

---

**Next**: Proceed to PHASE_8_ADVANCED_AGENTS_REVIEW_DEPLOY.md after agent integration tests pass.