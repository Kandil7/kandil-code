# PHASE_8_ADVANCED_AGENTS_REVIEW_DEPLOY.md

# Phase 8: Advanced Agents, Review/Deploy & v1.0 Release

## Objectives
Build review, optimization, and deployment agents with ethics/security scanning. Create a meta-agent for self-improvement. Integrate full CI/CD pipeline. Release v1.0 with production-ready features and documentation.

## Prerequisites
- Phase 7 complete (code/test agents, PM/BA simulations).
- All API keys stored securely in OS keyring (from Phase 0 refinement).
- Supabase project configured for cloud sync.
- GitHub CLI for release automation (`gh auth login`).

## Detailed Sub-Tasks

### **Week 1: Review & Ethics Agents**

**Day 1-2: Code Review Agent**
- Create `src/agents/review.rs`:
```rust
use crate::agents::base::BaseAgent;
use crate::code::analyzer::CodeAnalyzer;

pub struct ReviewAgent {
    base: BaseAgent,
    analyzer: CodeAnalyzer,
}

impl ReviewAgent {
    pub fn new(ai: KandilAI) -> Result<Self> {
        Ok(Self {
            base: BaseAgent { ai },
            analyzer: CodeAnalyzer::new()?,
        })
    }

    pub async fn code_review(&self, file_path: &str) -> Result<ReviewReport> {
        let content = std::fs::read_to_string(file_path)?;
        let ast_analysis = self.analyzer.analyze_file(&content, "dart")?;
        
        let prompt = format!(r#"
        Review this code for:
        - Bugs and logic errors
        - Security vulnerabilities (OWASP Top 10)
        - Performance anti-patterns
        - Code smells
        
        AST Analysis: {}
        Code: {}
        "#, ast_analysis, content);
        
        let review = self.ai.chat(&prompt, None).await?;
        self.generate_report(&review)
    }

    fn generate_report(&self, raw: &str) -> Result<ReviewReport> {
        // Parse AI output into structured report
        Ok(ReviewReport { issues: vec![], score: 85 })
    }
}
```
- Add `ReviewReport` struct to `src/core/report.rs`:
```rust
#[derive(Serialize)]
pub struct ReviewReport {
    pub issues: Vec<Issue>,
    pub score: u8,
}

#[derive(Serialize)]
pub struct Issue {
    pub severity: Severity,
    pub line: Option<usize>,
    pub description: String,
}

#[derive(Serialize)]
pub enum Severity { Low, Medium, High, Critical }
```
- Create CLI command: `kandil agent review <file> --format=json`

**Day 3-4: Security & Ethics Sub-Agent**
- Create `src/agents/review/security.rs`:
```rust
pub struct SecuritySubAgent {
    base: BaseAgent,
}

impl SecuritySubAgent {
    pub async fn owasp_scan(&self, code: &str) -> Result<Vec<SecurityIssue>> {
        let prompt = format!(r#"
        Scan for OWASP vulnerabilities:
        - Injection flaws
        - Broken auth
        - Sensitive data exposure
        - XSS
        
        Code: {}
        "#, code);
        
        let result = self.ai.chat(&prompt, None).await?;
        self.parse_security_issues(&result)
    }
}
```
- Integrate ethics scanning:
```rust
pub async fn ethics_check(&self, design: &str) -> Result<EthicsReport> {
    let prompt = format!(r#"
    Check for:
    - Algorithmic bias
    - Privacy (GDPR/CCPA) violations
    - Dark patterns
    - Accessibility gaps
    
    Design: {}
    "#, design);
    
    self.ai.chat(&prompt, None).await?
}
```

**Day 5: Performance Optimization Agent**
- Create `src/agents/review/performance.rs`:
```rust
pub async fn profile_suggestion(&self, code: &str, lang: &str) -> Result<String> {
    // Generate profiling commands
    match lang {
        "rust" => Ok("cargo flamegraph --bench".to_string()),
        "python" => Ok("python -m cProfile -o profile.prof".to_string()),
        _ => Err(anyhow::anyhow!("No profiler for {}", lang))
    }
}
```

### **Week 2: Deploy Agent & CI/CD**

**Day 6-7: Deploy Agent Core**
- Create `src/agents/deploy.rs`:
```rust
use tokio::process::Command;

pub struct DeployAgent {
    base: BaseAgent,
}

impl DeployAgent {
    pub async fn generate_ci_cd(&self, target: &str) -> Result<PathBuf> {
        let ws = Workspace::detect()?;
        let prompt = format!("Generate GitHub Actions YAML for {} deployment", target);
        let yaml = self.ai.chat(&prompt, None).await?;
        
        let path = ws.root.join(".github/workflows/deploy.yml");
        fs::create_dir_all(path.parent().unwrap())?;
        fs::write(&path, yaml)?;
        Ok(path)
    }

    pub async fn deploy(&self, env: &str) -> Result<()> {
        // Check if deploy config exists
        if !Path::new(".github/workflows/deploy.yml").exists() {
            self.generate_ci_cd(env).await?;
        }
        
        // Trigger via GitHub CLI or direct push
        let output = Command::new("gh")
            .args(&["workflow", "run", "deploy.yml", "-f", format!("env={}", env)])
            .output()
            .await?;
            
        if !output.status.success() {
            return Err(anyhow::anyhow!("Deploy failed: {:?}", output.stderr));
        }
        Ok(())
    }
}
```

**Day 8: Infrastructure as Code (IaC)**
- Add Terraform/Docker generation:
```rust
pub async fn generate_dockerfile(&self, lang: &str) -> Result<PathBuf> {
    let prompt = format!("Multi-stage Dockerfile for {} app with security best practices", lang);
    let dockerfile = self.ai.chat(&prompt, None).await?;
    let path = PathBuf::from("Dockerfile");
    fs::write(&path, dockerfile)?;
    Ok(path)
}
```

**Day 9: Multi-Target Deploy Trait**
- Refactor to support Vercel, Netlify, AWS:
```rust
#[async_trait]
pub trait DeployAdapter {
    async fn deploy(&self, project: &Path) -> Result<()>;
    fn name(&self) -> &str;
}

pub struct VercelAdapter;
pub struct AwsAdapter;

#[async_trait]
impl DeployAdapter for VercelAdapter {
    async fn deploy(&self, _project: &Path) -> Result<()> {
        Command::new("vercel").arg("--prod").status().await?;
        Ok(())
    }
    fn name(&self) -> &'static str { "vercel" }
}
```

### **Week 3: Meta-Agent & Pipeline**

**Day 10-11: Meta-Agent for Self-Improvement**
- Create `src/agents/meta.rs`:
```rust
pub struct MetaAgent {
    base: BaseAgent,
    memory: AgentMemory,
}

impl MetaAgent {
    pub async fn analyze_log(&self, log_path: &str) -> Result<ImprovementPlan> {
        let log = fs::read_to_string(log_path)?;
        let prompt = format!(r#"
        Analyze Kandil's execution log and suggest:
        - Performance bottlenecks
        - Error patterns
        - Feature gaps
        
        Log: {}
        "#, log);
        
        let analysis = self.ai.chat(&prompt, None).await?;
        self.memory.store_context(&analysis)?;
        self.generate_plan(&analysis)
    }

    pub async def improve_agent(&self, agent_name: &str) -> Result<()> {
        // Re-generate agent prompts based on meta-analysis
        let prompt = self.memory.retrieve_relevant("prompt_template")?;
        // Write to prompts/{agent_name}.v2.txt
    }
}
```

**Day 12-13: Full Pipeline Command**
- Create `src/cli/pipeline.rs`:
```rust
pub async fn run_full_pipeline(idea: &str) -> Result<PipelineResult> {
    let event_tx = crate::agents::EVENT_BUS.clone();
    
    // Step 1: Requirements
    let reqs = RequirementsAgent::new().elicit(idea).await?;
    event_tx.send(AgentEvent::RequirementsDone(reqs.clone()))?;
    
    // Step 2: Design (triggered by event)
    let design = DesignAgent::new().generate_architecture(&reqs, "flutter").await?;
    
    // Step 3: Code
    let code = CodeAgent::new().generate(&design, "flutter").await?;
    
    // Step 4: Test
    TestAgent::new().generate_suite(&code, "dart", 80).await?;
    
    // Step 5: Review
    let report = ReviewAgent::new().code_review("lib/main.dart").await?;
    
    // Step 6: Deploy
    DeployAgent::new().generate_ci_cd("vercel").await?;
    
    Ok(PipelineResult { reqs, design, report })
}
```
- CLI: `kandil pipeline --idea="cinema booking app" --auto-deploy`

**Day 14: Integration & Testing**
- Wire everything in `main.rs` with new subcommands
- Add event listeners for cross-agent communication:
```rust
tokio::spawn(async move {
    let mut rx = EVENT_BUS.subscribe();
    while let Ok(event) = rx.recv().await {
        match event {
            AgentEvent::RequirementsDone(reqs) => {
                // Optionally trigger design agent automatically
            }
        }
    }
});
```

### **Week 4: v1.0 Release Prep**

**Day 15-16: Testing & Stabilization**
- Run full mutation testing:
```bash
cargo install cargo-mutants
cargo mutants --timeout 120
```
- Fix any surviving mutants (weak tests).
- Add integration test for pipeline:
```rust
#[tokio::test]
async fn test_pipeline_e2e() {
    // Use wiremock to mock all AI calls
    let mock = mock_ollama_response("Mocked pipeline output");
    let result = run_full_pipeline("test idea").await.unwrap();
    assert!(result.report.score > 70);
}
```

**Day 17-18: Documentation & Polish**
- Update `README.md` with v1.0 features:
```markdown
## v1.0 Features
- 🤖 Multi-agent pipeline (Req → Design → Code → Test → Review → Deploy)
- 🔒 Built-in security & ethics scanning
- 🚀 One-command deploy to Vercel/Netlify/AWS
- 🧠 Self-improving meta-agent
- 📊 Structured JSON reports for CI integration
```
- Generate man pages: `cargo install cargo-generate-rpm` (optional)
- Add shell completions: `clap_complete` in CLI

**Day 19-20: Security Audit**
- Run comprehensive audit:
```bash
cargo audit
cargo deny check licenses advisories sources
cargo outdated -R
```
- Fix all high/critical vulnerabilities.
- Penetration test plugin system (if enabled): Try to escape sandbox.

**Day 21-24: Release**
- Update version in all `Cargo.toml`: `1.0.0`
- Create GitHub release:
```bash
git checkout -b release/v1.0
git tag -a v1.0.0 -m "First stable release"
cargo build --release --target x86_64-unknown-linux-gnu
cargo build --release --target x86_64-apple-darwin
gh release create v1.0.0 ./target/*/release/kandil --title "v1.0.0" --notes-file CHANGELOG.md
```

## Tools & Dependencies
- **Crates**: `keyring = "2"`, `tokio-retry = "0.3"`, `wiremock = "0.5"` (dev), `cargo-mutants` (tool), `cargo-deny` (tool)
- **External**: GitHub CLI, Vercel CLI, Docker (for plugin sandboxing)
- **Security**: `cargo-audit`, `cargo-deny`

## Testing Strategy
- **Unit**: 90% coverage on all agents (use `mockall` for AI trait)
- **Integration**: Mock all external calls (AI, GitHub, Vercel), test pipeline end-to-end
- **Security**: Run `bandit` (Python) and `semgrep` on generated code
- **Manual**: Deploy sample Flutter app to Vercel, verify full pipeline

## Deliverables
- `kandil agent review` with JSON output
- `kandil agent deploy` supporting multiple targets
- `kandil pipeline --full` command
- v1.0.0 GitHub release with binaries for Linux/macOS
- `CHANGELOG.md` with breaking changes/features

## Timeline Breakdown
- **Week 1**: Review agents, security/ethics scanning
- **Week 2**: Deploy agent, CI/CD generation, multi-target support
- **Week 3**: Meta-agent, event bus, full pipeline integration
- **Week 4**: Testing, security audit, documentation, release

## Success Criteria
- Review agent identifies ≥3 issues in sample vulnerable code
- Deploy agent generates valid GitHub Actions YAML that passes `actionlint`
- Full pipeline runs end-to-end in <5 minutes (mocked AI)
- Security audit shows zero high/critical vulnerabilities
- Release downloaded 50+ times in first week

## Potential Risks & Mitigations
- **Risk**: Deploy agent bricks user's repository with bad YAML
  - **Mitigation**: Preview mode only in v1.0; require `--force` for apply
- **Risk**: Meta-agent causes infinite improvement loop
  - **Mitigation**: Hard limit of 2 meta-iterations; manual approval for agent regen
- **Risk**: Event bus causes deadlocks in pipeline
  - **Mitigation**: Use `tokio::sync::mpsc` instead of `broadcast`; test with `loom`
- **Risk: AI hallucinates security vulnerabilities**
  - **Mitigation**: Cross-reference with `cargo-audit` actual results; flag AI false positives

---