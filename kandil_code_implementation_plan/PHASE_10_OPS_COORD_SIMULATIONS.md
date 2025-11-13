# PHASE_10_OPS_COORD_SIMULATIONS.md

# Phase 10: DevOps, Scrum Simulations & Advanced Features

## Objectives
Implement DevOps and Scrum ceremonies simulations. Add scalability testing, i18n/a11y assistants, real-time collaboration, and IDE extension prototype. Prepare for production polish.

## Prerequisites
- Phase 9 complete (tech role simulations).
- Supabase project with realtime WebSockets enabled.
- VS Code extension development basics (Node.js, TypeScript).

## Detailed Sub-Tasks

### **Week 1: DevOps Simulation**

**Day 1-2: IaC Generation & Validation**
- Create `src/agents/simulate/devops.rs`:
```rust
pub struct DevOpsSimulation {
    base: BaseAgent,
    infra_templates: HashMap<String, String>,
}

impl DevOpsSimulation {
    pub async fn generate_terraform(&self, infra: &str) -> Result<PathBuf> {
        let prompt = format!(r#"
        Generate Terraform for: {}
        
        Requirements:
        - Use modules from registry
        - Add security groups (least privilege)
        - Enable encryption at rest
        - Tag resources per company policy
        - Output connection strings
        
        Respond with only HCL code.
        "#, infra);
        
        let tf = self.ai.chat(&prompt, None).await?;
        let path = PathBuf::from("infra/main.tf");
        fs::create_dir_all("infra")?;
        fs::write(&path, tf)?;
        
        // Validate with terraform fmt
        Command::new("terraform").args(&["fmt", "-check", "infra"]).status()?;
        Ok(path)
    }

    pub async fn security_harden(&self, tf_code: &str) -> Result<String> {
        let prompt = format!(r#"
        Harden this Terraform for security:
        - No hardcoded secrets
        - Private subnets
        - WAF rules
        - Audit logging
        {}
        "#, tf_code);
        
        self.ai.chat(&prompt, None).await?
    }
}
```

**Day 3-4: Incident Response Drills**
- Create incident scenarios:
```rust
pub async fn run_drill(&self, scenario: &str) -> Result<DrillReport> {
    let mut timeline = vec![];
    let start = Instant::now();
    
    // Phase 1: Detection
    let detection_time = start.elapsed();
    timeline.push(format!("[{:?}] PagerDuty alert received", detection_time));
    
    // Phase 2: AI suggests mitigation
    let prompt = format!(r#"
    Incident: {}
    Suggest immediate mitigation steps.
    "#, scenario);
    
    let mitigation = self.ai.chat(&prompt, None).await?;
    timeline.push(format!("[{:?}] Mitigation: {}", start.elapsed(), mitigation));
    
    Ok(DrillReport {
        scenario: scenario.to_string(),
        timeline,
        lessons: self.extract_lessons(&mitigation),
    })
}
```

**Day 5: Cost Optimization**
- Add cost analysis:
```rust
pub async fn cost_report(&self, tf_plan: &str) -> Result<CostBreakdown> {
    let prompt = format!(r#"
    Estimate monthly cost for this plan:
    {}
    
    Consider: Data transfer, storage, compute.
    Return JSON: {{ "total": 0, "breakdown": {{}} }}
    "#, tf_plan);
    
    let json_str = self.ai.chat(&prompt, None).await?;
    serde_json::from_str(&json_str)?
}
```

### **Week 2: Scrum Simulation**

**Day 6-7: Ceremony Facilitation**
- Create `src/agents/simulate/scrum.rs`:
```rust
pub struct ScrumSimulation {
    base: BaseAgent,
    sprint_data: SprintData,
}

impl ScrumSimulation {
    pub async fn daily_standup(&self, updates: &[String]) -> Result<StandupSummary> {
        let prompt = format!(r#"
        Facilitate standup for team of 5:
        Updates: {:?}
        
        Identify: Blockers, dependencies, overcommitment.
        Suggest: Pairing opportunities, scope adjustments.
        "#, updates);
        
        self.ai.chat(&prompt, None).await?
    }

    pub async fn sprint_retrospective(&self, feedback: &[String]) -> Result<RetroActions> {
        let prompt = format!(r#"
        Run retrospective with feedback:
        {}
        
        Generate: Start/Stop/Continue, Action items, Owner assignment.
        "#, feedback.join("\n"));
        
        let retro = self.ai.chat(&prompt, None).await?;
        self.extract_actions(&retro)
    }

    pub fn burndown_chart(&self, tasks: &[Task]) -> String {
        // Generate ASCII chart
        let total = tasks.len();
        let done = tasks.iter().filter(|t| t.status == "done").count();
        let remaining = total - done;
        
        format!(r#"
        Burndown (Day 5/10):
        ████████████████████░░░░░░░░░░░░░░░░░░  ({}/{})
        "#, done, total)
    }
}
```

**Day 8-9: Planning Poker & Estimation**
- Add estimation support:
```rust
pub async fn estimate_story(&self, story: &str) -> Result<StoryPoints> {
    let prompt = format!(r#"
    Estimate story: "{}"
    
    Use Fibonacci: 1, 2, 3, 5, 8, 13
    Consider: Complexity, unknowns, dependencies.
    
    Return only number.
    "#, story);
    
    let points: u8 = self.ai.chat(&prompt, None).await?.trim().parse()?;
    Ok(StoryPoints::new(points))
}
```

**Day 10: Velocity Tracking**
- Create velocity calculator:
```rust
pub fn calculate_velocity(sprints: &[Sprint]) -> f32 {
    let velocities: Vec<_> = sprints.iter()
        .map(|s| s.completed_points as f32 / s.days as f32)
        .collect();
    velocities.iter().sum::<f32>() / velocities.len() as f32
}
```

### **Week 3: Scalability & i18n**

**Day 11-12: Scalability Simulation**
- Create `src/agents/scalability.rs`:
```rust
pub struct ScalabilityAgent {
    base: BaseAgent,
}

impl ScalabilityAgent {
    pub async fn load_test_plan(&self, app: &str, users: u32) -> Result<LoadTestConfig> {
        let prompt = format!(r#"
        Generate k6 load test script for {} serving {} users.
        
        Include: Ramp-up, steady state, spike.
        Metrics: RPS, latency p95, error rate.
        "#, app, users);
        
        let script = self.ai.chat(&prompt, None).await?;
        fs::write("load.js", script)?;
        Ok(LoadTestConfig { tool: "k6".to_string() })
    }

    pub async fn capacity_plan(&self, metrics: &Metrics) -> Result<CapacityPlan> {
        // Analyze metrics, suggest instance counts
        unimplemented!()
    }
}
```

**Day 13-14: i18n Assistant**
- Create `src/agents/i18n.rs`:
```rust
pub struct I18nAssistant {
    base: BaseAgent,
}

impl I18nAssistant {
    pub async fn extract_strings(&self, code: &str) -> Result<Vec<StringKey>> {
        // Use regex to find user-facing strings
        let re = regex::Regex::new(r#"Text\(['"](.+?)['"]\)"#)?;
        let mut keys = vec![];
        
        for (i, line) in code.lines().enumerate() {
            for cap in re.captures_iter(line) {
                keys.push(StringKey {
                    key: format!("msg_{}", i),
                    value: cap[1].to_string(),
                    line: i,
                });
            }
        }
        Ok(keys)
    }

    pub async fn translate(&self, strings: &[StringKey], target_langs: &[&str]) -> Result<Translations> {
        let mut translations = HashMap::new();
        
        for lang in target_langs {
            let mut lang_map = HashMap::new();
            for key in strings {
                let prompt = format!("Translate '{}' to {}", key.value, lang);
                let translation = self.ai.chat(&prompt, None).await?;
                lang_map.insert(key.key.clone(), translation);
            }
            translations.insert(lang.to_string(), lang_map);
        }
        
        Ok(Translations { translations })
    }

    pub fn generate_arb(&self, translations: &Translations) -> Result<PathBuf> {
        // Flutter ARB format
        let arb = serde_json::to_string_pretty(&translations)?;
        fs::write("lib/l10n/app_en.arb", arb)?;
        Ok(PathBuf::from("lib/l10n"))
    }
}
```

### **Week 4: Collaboration & IDE**

**Day 15-17: Real-time Collaboration**
- Add WebSocket support:
```rust
// src/cloud/realtime.rs
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};

pub struct CollaborationHub {
    ws_stream: WebSocketStream<TcpStream>,
    session_id: String,
}

impl CollaborationHub {
    pub async fn join(session: &str) -> Result<Self> {
        let url = format!("wss://your-supabase-url/websocket?apikey={}", api_key);
        let (ws_stream, _) = connect_async(&url).await?;
        
        // Authenticate
        ws_stream.send(Message::Text(
            json!({"type": "phx_join", "topic": format!("session:{}", session)}).to_string()
        )).await?;
        
        Ok(Self { ws_stream, session_id: session.to_string() })
    }

    pub async fn broadcast_message(&self, msg: &str) -> Result<()> {
        self.ws_stream.send(Message::Text(
            json!({
                "type": "broadcast",
                "event": "agent_message",
                "payload": { "text": msg }
            }).to_string()
        )).await?;
        Ok(())
    }
}
```
- Share agent state across clients:
```rust
pub async fn sync_agent_state(&self, state: &AgentState) -> Result<()> {
    let state_json = serde_json::to_string(state)?;
    self.broadcast_message(&state_json).await
}
```

**Day 18-19: VS Code Extension Prototype**
- Create `extensions/vscode/` directory:
```typescript
// extensions/vscode/src/extension.ts
import * as vscode from 'vscode';
import { LanguageClient } from 'vscode-languageclient/node';

export function activate(context: vscode.ExtensionContext) {
    const client = new LanguageClient(
        'kandil',
        'Kandil Code',
        {
            command: 'kandil',
            args: ['lsp'],
        },
        {
            documentSelector: [{ scheme: 'file', language: 'dart' }],
        }
    );
    
    client.start();
    
    // Register chat command
    vscode.commands.registerCommand('kandil.chat', async () => {
        const editor = vscode.window.activeTextEditor;
        if (!editor) return;
        
        const selection = editor.document.getText(editor.selection);
        const result = await exec(`kandil chat "${selection}"`);
        vscode.window.showInformationMessage(result.stdout);
    });
}
```
- Package: `vsce package` (requires Node.js)

**Day 20-21: Accessibility Scanner**
- Create `src/agents/a11y.rs`:
```rust
pub async fn scan_flutter_ui(&self, widget_tree: &str) -> Result<AccessibilityReport> {
    let prompt = format!(r#"
    Scan Flutter widget tree for WCAG 2.1 issues:
    {}
    
    Check: Semantic labels, contrast hints, focus order, screen reader support.
    Return violations as JSON.
    "#, widget_tree);
    
    let violations = self.ai.chat(&prompt, None).await?;
    serde_json::from_str(&violations)?
}
```

**Day 22-24: Testing & Polish**
- Integration tests for WebSocket:
```rust
#[tokio::test]
async fn test_collaboration_sync() {
    let hub = CollaborationHub::join("test-session").await.unwrap();
    hub.broadcast_message("test").await.unwrap();
    // Verify other client receives
}
```
- Performance benchmark collaboration:
```bash
cargo bench --bench collaboration
```

## Tools & Dependencies
- **Crates**: `tokio-tungstenite = "0.20"`, `chrono = "0.4"`, `rand = "0.8"` (for drill scenarios)
- **External**: Supabase Realtime, Node.js (for extension)
- **Dev Tools**: `vsce` (VSCE CLI), `k6` (for load testing)

## Testing Strategy
- **Unit**: Mock WebSocket, test message serialization (80% coverage)
- **Integration**: Full Scrum ceremony (standup → retro) with mocked team
- **Load**: Test collaboration hub with 10 concurrent clients
- **Security**: Pen test WebSocket auth (try to join session without key)

## Deliverables
- `kandil simulate devops drill --scenario="database-outage"`
- `kandil simulate scrum standup --updates="$(cat updates.md)"`
- `kandil i18n extract --file=lib/main.dart --to=ar,es,fr`
- `kandil scale test --users=10000`
- Real-time collaboration sessions via WebSocket
- VS Code extension v0.1 (basic chat)
- Load test scripts for sample apps

## Timeline Breakdown
- **Week 1**: DevOps IaC, drills, cost analysis
- **Week 2**: Scrum ceremonies, estimation, velocity
- **Week 3**: Scalability, i18n, a11y
- **Week 4**: Real-time collab, IDE extension, testing

## Success Criteria
- DevOps drill completes in <10s and generates actionable timeline
- Scrum standup identifies ≥2 blockers from sample updates
- i18n extracts 100% of user strings from test file
- Collaboration hub syncs state in <100ms (local network)
- VS Code extension successfully calls `kandil chat` on selection

## Potential Risks & Mitigations
- **Risk**: WebSocket connection drops during critical session
  - **Mitigation**: Implement reconnection with exponential backoff; store unsynced changes in queue
- **Risk**: Translation quality is poor for rare languages
  - **Mitigation**: Use professional translation API (DeepL) for production; AI for draft only
- **Risk**: VSCE packaging fails due to Node version
  - **Mitigation**: Pin Node to LTS in `.nvmrc`; use CI matrix for testing
- **Risk**: Load test script crashes target app
  - **Mitigation**: Add safety checks: max RPS, `--dry-run` mode

---
