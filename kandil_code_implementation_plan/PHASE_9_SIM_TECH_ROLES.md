# PHASE_9_SIM_TECH_ROLES.md

# Phase 9: Simulations for Tech Roles

## Objectives
Implement interactive simulations for Architect, Developer, and QA roles. Enable cross-role collaboration and pair programming. Add role-specific knowledge bases and decision logging.

## Prerequisites
- Phase 8 complete (advanced agents, v1.0 foundation).
- Sample projects to simulate on (Cinema App, E-commerce API).
- Role-play prompt templates in `prompts/roles/`.

## Detailed Sub-Tasks

### **Week 1: Architect Simulation**

**Day 1-2: Architect Knowledge Base**
- Create `src/agents/simulate/architect/knowledge.rs`:
```rust
pub struct ArchitecturePatterns {
    patterns: HashMap<String, Pattern>,
}

impl ArchitecturePatterns {
    pub fn load() -> Result<Self> {
        // Load from data/architectures.json
        let data = include_str!("../../../data/architectures.json");
        let patterns: HashMap<String, Pattern> = serde_json::from_str(data)?;
        Ok(Self { patterns })
    }

    pub fn get(&self, name: &str) -> Option<&Pattern> {
        self.patterns.get(name)
    }
}

#[derive(Deserialize)]
pub struct Pattern {
    pub name: String,
    pub description: String,
    pub pros: Vec<String>,
    pub cons: Vec<String>,
    pub use_cases: Vec<String>,
}
```
- Create sample data file `data/architectures.json`:
```json
{
  "clean_architecture": {
    "name": "Clean Architecture",
    "description": "Layers: Entities → Use Cases → Interface Adapters",
    "pros": ["Testability", "Separation of Concerns"],
    "cons": ["Overhead for small projects"],
    "use_cases": ["Enterprise apps", "Multi-platform"]
  }
}
```

**Day 3-4: Architect Review & ADR Generation**
- Implement `src/agents/simulate/architect.rs`:
```rust
pub struct ArchitectSimulation {
    base: BaseAgent,
    knowledge: ArchitecturePatterns,
    decision_log: Vec<Decision>,
}

impl ArchitectSimulation {
    pub async fn review_design(&self, uml: &str) -> Result<ArchitectureReport> {
        let prompt = format!(r#"
        As a Principal Architect with 20 years experience:
        1. Evaluate this UML: {}
        2. Check SOLID principles
        3. Suggest scalability improvements
        4. Write ADR (Architecture Decision Record)
        
        Use pattern knowledge: {:?}
        "#, uml, self.knowledge.patterns.keys());
        
        let review = self.ai.chat(&prompt, None).await?;
        self.log_decision("design_review", &review)?;
        Ok(self.parse_report(&review))
    }

    pub async fn tradeoff_analysis(&self, options: &[&str]) -> Result<TradeoffMatrix> {
        let prompt = format!(r#"
        Analyze trade-offs for:
        {}
        
        Evaluate: Performance, Cost, Complexity, Maintainability
        Return as markdown table.
        "#, options.join("\n"));
        
        let matrix_md = self.ai.chat(&prompt, None).await?;
        TradeoffMatrix::from_markdown(&matrix_md)
    }
}
```

**Day 5: ADR Template & Storage**
- Create ADR template: `templates/adr/0001-template.md`:
```markdown
# ADR 0001: {title}
Date: {date}
Status: {proposed/accepted/rejected}

## Context
{context}

## Decision
{decision}

## Consequences
{consequences}
```
- Add command: `kandil simulate architect adr --title="Use BLoC Pattern" --context="Need state management"`

### **Week 2: Developer Pair Programming**

**Day 6-7: Interactive Pair Session**
- Create `src/agents/simulate/dev.rs`:
```rust
pub struct DevSimulation {
    base: BaseAgent,
    session_history: Vec<(String, String)>,
}

impl DevSimulation {
    pub async fn pair_program(&self, task: &str, code: &str) -> Result<PairSession> {
        let prompt = format!(r#"
        You're a senior developer pair programming.
        
        Task: {}
        Current Code: {}
        
        1. Suggest improvements (be specific, line numbers)
        2. Offer alternative implementations
        3. Explain trade-offs
        4. Ask clarifying questions
        
        Previous turns: {:?}
        "#, task, code, self.session_history);
        
        let suggestion = self.ai.chat(&prompt, None).await?;
        self.session_history.push((task.to_string(), suggestion.clone()));
        
        Ok(PairSession {
            suggestions: self.parse_suggestions(&suggestion),
            questions: self.extract_questions(&suggestion),
        })
    }

    pub async fn code_review_pr(&self, diff: &str) -> Result<PRReview> {
        // Simulate GitHub PR review
        let prompt = format!(r#"
        Review this PR diff as a senior dev:
        {}
        
        Check: Logic, tests, naming, edge cases, performance.
        Provide line-by-line comments.
        "#, diff);
        
        self.ai.chat(&prompt, None).await?
    }
}
```

**Day 8: Session State Management**
- Store pair sessions in SQLite:
```rust
pub fn save_session(&self, session_id: &str) -> Result<()> {
    let conn = Connection::open(".kandil/sessions.db")?;
    conn.execute(
        "INSERT INTO pair_sessions (id, history) VALUES (?1, ?2)",
        params![session_id, serde_json::to_string(&self.session_history)?]
    )?;
    Ok(())
}
```

**Day 9: Hot Reload for Simulations**
- In TUI, add pair mode:
```rust
// In TUI event loop
KeyCode::Char('p') => {
    self.mode = Mode::PairProgramming;
    let task = self.get_user_input("Enter task: ");
    let code = self.editor.get_text();
    let session = dev_sim.pair_program(&task, &code).await?;
    self.render_suggestions(&session.suggestions);
}
```

### **Week 3: QA Simulation**

**Day 10-11: Test Plan Generation**
- Create `src/agents/simulate/qa.rs`:
```rust
pub struct QASimulation {
    base: BaseAgent,
    test_cases: Vec<TestCase>,
}

impl QASimulation {
    pub async fn generate_test_plan(&self, reqs: &str) -> Result<TestPlan> {
        let prompt = format!(r#"
        As QA Lead, create comprehensive test plan for:
        {}
        
        Include: Unit, Integration, E2E, Performance, Security tests.
        Use Gherkin syntax for acceptance criteria.
        "#, reqs);
        
        let plan = self.ai.chat(&prompt, None).await?;
        self.parse_test_plan(&plan)
    }

    pub async def exploratory_test(&self, app: &str) -> Result<Vec<ExploratorySession>> {
        let prompt = format!(r#"
        Generate exploratory testing charter for {} app.
        Focus: User flows, edge cases, boundary values.
        Output as markdown checklist.
        "#, app);
        
        self.ai.chat(&prompt, None).await?
    }
}

#[derive(Deserialize)]
pub struct TestCase {
    pub id: String,
    pub description: String,
    pub steps: Vec<String>,
    pub expected: String,
    pub priority: Priority,
}
```

**Day 12: Bug Triage Simulation**
- Add bug triage workflow:
```rust
pub async fn triage(&self, bugs: &[BugReport]) -> Result<TriageResult> {
    let prompt = format!(r#"
    Triage these bugs:
    {}
    
    Assign severity, priority, estimated fix time.
    Suggest root cause categories.
    "#, serde_json::to_string(bugs)?);
    
    let triage = self.ai.chat(&prompt, None).await?;
    Ok(TriageResult::parse(&triage))
}
```

**Day 13-14: Cross-Role Collaboration**
- Create orchestrator for role handoffs:
```rust
pub async fn simulate_sprint_planning(&self, backlog: &[str]) -> Result<SprintPlan> {
    // PM generates OKRs
    let pm_sim = PMSimulation::new();
    let okrs = pm_sim.generate_okr("Increase bookings").await?;
    
    // Architect reviews technical feasibility
    let arch_sim = ArchitectSimulation::new();
    let feasibility = arch_sim.review_feasibility(&okrs).await?;
    
    // Dev estimates stories
    let dev_sim = DevSimulation::new();
    let estimates = dev_sim.estimate_stories(backlog).await?;
    
    Ok(SprintPlan { okrs, feasibility, estimates })
}
```

### **Week 4: Polish & Integration**

**Day 15-16: TUI Role Dashboard**
- Add role-specific panels:
```rust
// In TUI layout
fn draw_roles_panel(&self, f: &mut Frame, area: Rect) {
    let roles = vec!["PM", "Architect", "Dev", "QA"];
    let items: Vec<ListItem> = roles.iter().map(|r| {
        ListItem::new(format!("Simulate {}", r)).style(Style::default().fg(Color::Cyan))
    }).collect();
    let list = List::new(items).block(Block::default().title("Roles"));
    f.render_widget(list, area);
}
```

**Day 17-18: Knowledge Base Expansion**
- Expand `data/` with:
  - `design_patterns.json` (50 patterns)
  - `owasp_vulnerabilities.json`
  - `testing_strategies.json`
- Create script to auto-update from OWASP website (use `reqwest` to fetch):

**Day 19: Session Replay**
- Add command to replay past simulations:
```bash
kandil simulate replay --session-id="2024-01-15-pair-001"
```

**Day 20-21: Testing & Validation**
- Unit tests for each role (use `mockall`):
```rust
#[test]
fn test_architect_adr_format() {
    let sim = ArchitectSimulation::new(mock_ai());
    let adr = sim.generate_adr("Use BLoC", "Need state").await.unwrap();
    assert!(adr.contains("Status: proposed"));
}
```

**Day 22-24: v1.1 Preparation**
- Update version strings to `1.1.0-alpha`
- Create migration guide for v1.0 users
- Add feature flags: `--enable-rbac-sim`, `--enable-load-testing`

## Tools & Dependencies
- **Crates**: `mockall = "0.13"`, `proptest = "1.4"` (for decision property tests)
- **External**: None new
- **Data**: OWASP website, Design Patterns book (for knowledge base)

## Testing Strategy
- **Unit**: 85% coverage on each role's logic. Mock AI completely.
- **Property-based**: Test that architect decisions always include "trade-offs" section
- **Integration**: Simulate full sprint: OKR → Design → Code → Test
- **Manual**: Run 30-min pair session with each role, log UX issues

## Deliverables
- `kandil simulate architect --uml=<file> --output-adr`
- `kandil simulate dev pair --task="Implement auth" --code-file=lib/auth.dart`
- `kandil simulate qa plan --reqs=srs.md --format=gherkin`
- Knowledge base JSON files in `data/`
- TUI role selector dashboard
- Session history stored in SQLite

## Timeline Breakdown
- **Week 1**: Architect agent, ADR generation, trade-offs
- **Week 2**: Developer pair programming, session state
- **Week 3**: QA test plans, bug triage, cross-role orchestration
- **Week 4**: TUI integration, knowledge base expansion, testing

## Success Criteria
- Each role simulation produces role-specific output (architect: ADR, dev: code suggestions, QA: Gherkin)
- Cross-role handoff completes in <30 seconds (mocked AI)
- Pair session can handle 5+ turns without losing context
- Knowledge base contains ≥50 entries across roles
- 100% of role commands have unit tests

## Potential Risks & Mitigations
- **Risk**: Role simulations feel generic, not specialized
  - **Mitigation**: Fine-tune prompts with domain examples; store role-specific few-shot prompts in `prompts/roles/`
- **Risk**: Session history grows too large, causes AI token overflow
  - **Mitigation**: Summarize old turns after 5 exchanges; store full in DB, send summary to AI
- **Risk**: Cross-role orchestration creates circular dependencies
  - **Mitigation**: Use DAG (Directed Acyclic Graph) for orchestration; `petgraph` crate
- **Risk**: TUI becomes cluttered with too many role panels
  - **Mitigation**: Use tabs per role; lazy-load content

---
