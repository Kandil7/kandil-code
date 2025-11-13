# PHASE_11_ADVANCED_POLISH.md

# Phase 11: Advanced Features, UI Polish & Community Setup

## Objectives
Add green development auditing, enhanced accessibility scanning, i18n RTL support, marketplace search, UI themes, and comprehensive final docs. Prepare for v2.0 release.

## Prerequisites
- Phase 10 complete (DevOps, Scrum, i18n).
- GitHub API token (for marketplace, read-only).
- Docker Hub account (for publishing runtime image).
- Figma design tokens (optional, for consistent theming).

## Detailed Sub-Tasks

### **Week 1: Green Dev & A11y**

**Day 1-2: Carbon Footprint Agent**
- Create `src/agents/green.rs`:
```rust
pub struct GreenDevAgent {
    base: BaseAgent,
}

impl GreenDevAgent {
    pub async fn carbon_audit(&self, code: &str, lang: &str) -> Result<CarbonReport> {
        let prompt = format!(r#"
        Estimate carbon footprint of this {} code:
        {}
        
        Consider:
        - Algorithmic complexity (O(n^3) vs O(log n))
        - Data transfer sizes
        - Resource idle time
        - Cache efficiency
        
        Suggest green alternatives.
        "#, lang, code);
        
        let audit = self.ai.chat(&prompt, None).await?;
        self.parse_carbon_report(&audit)
    }

    pub async fn optimize_energy(&self, inefficiencies: &[Inefficiency]) -> Result<Vec<Suggestion>> {
        let prompt = format!(r#"
        Optimize these energy-inefficient patterns:
        {:?}
        
        Return specific code changes with estimated kWh savings.
        "#, inefficiencies);
        
        self.ai.chat(&prompt, None).await?
    }
}
```

**Day 3-4: Full Accessibility Scanner**
- Enhance Phase 10 a11y agent:
```rust
pub async fn wcag_audit(&self, html: &str, level: WcagLevel) -> Result<WcagReport> {
    let prompt = format!(r#"
    Perform WCAG {} audit:
    {}
    
    Check:
    - Perceivable: Alt text, captions, contrast
    - Operable: Keyboard nav, focus order
    - Understandable: Labels, error messages
    - Robust: Valid HTML, ARIA
    
    Return violations with remediation code.
    "#, level, html);
    
    let violations = self.ai.chat(&prompt, None).await?;
    self.generate_remediation(&violations)
}

pub fn generate_remediation(&self, violations: &str) -> Result<String> {
    // Generate specific code fixes
    Ok("Add aria-label='Close'".to_string())
}
```

**Day 5: RTL & Localization QA**
- Add RTL support check:
```rust
pub async fn check_rtl_support(&self, ui_code: &str) -> Result<RtlReport> {
    let prompt = format!(r#"
    Check Flutter code for RTL support:
    {}
    
    Ensure: Directionality widgets, mirror-aware alignment, string reversals.
    "#, ui_code);
    
    self.ai.chat(&prompt, None).await?
}
```

### **Week 2: Marketplace & UI**

**Day 6-7: Plugin Marketplace Search**
- Create `src/plugins/marketplace.rs`:
```rust
use reqwest::Client;

pub struct Marketplace {
    client: Client,
    api_token: String,
}

impl Marketplace {
    pub async fn search(&self, query: &str) -> Result<Vec<PluginInfo>> {
        let resp = self.client
            .get("https://api.github.com/search/repositories")
            .header("Authorization", format!("token {}", self.api_token))
            .query(&[("q", format!("kandil-plugin {}", query))])
            .send()
            .await?;
        
        let search: GitHubSearch = resp.json().await?;
        Ok(search.items.into_iter().map(|i| PluginInfo {
            name: i.name,
            url: i.html_url,
            stars: i.stargazers_count,
            description: i.description,
        }).collect())
    }

    pub async fn install_from_market(&self, plugin_name: &str) -> Result<()> {
        let info = self.search(plugin_name).await?.into_iter()
            .find(|p| p.name == plugin_name)
            .ok_or_else(|| anyhow::anyhow!("Plugin not found"))?;
        
        PluginRegistry::default().install(&info.url).await
    }
}
```

**Day 8-9: TUI Themes & Customization**
- Add theme engine:
```rust
// src/tui/theme.rs
pub struct Theme {
    pub bg: Color,
    pub fg: Color,
    pub accent: Color,
    pub error: Color,
    pub success: Color,
}

impl Theme {
    pub fn load(name: &str) -> Result<Self> {
        let theme_file = format!("themes/{}.json", name);
        let content = fs::read_to_string(theme_file)?;
        serde_json::from_str(&content).map_err(|e| e.into())
    }

    pub fn apply(&self) {
        // Update TUI styles
    }
}

// Default themes: dark, light, solarized
```
- Add command: `kandil tui --theme=dracula`

**Day 10: Error UI & Diagnostics**
- Create better error display:
```rust
pub fn render_error_dialog(f: &mut Frame, error: &KandilError) {
    let block = Block::default()
        .title(" Error ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Red));
    
    let text = match error {
        KandilError::AIError(msg) => format!("AI failed: {}\nCheck API key in system keyring", msg),
        KandilError::WorkspaceUnknown(path) => format!("No project found in {}\nRun 'kandil init'", path),
    };
    
    let paragraph = Paragraph::new(text).block(block).wrap(Wrap { trim: true });
    f.render_widget(paragraph, f.size());
}
```

### **Week 3: Docs & Community**

**Day 11-12: Documentation Generation**
- Create `src/docs/generator.rs`:
```rust
pub struct DocsGenerator {
    base: BaseAgent,
}

impl DocsGenerator {
    pub async fn generate_api_docs(&self, code: &str) -> Result<String> {
        let prompt = format!(r#"
        Generate API documentation in OpenAPI 3.0 format:
        {}
        
        Include: Endpoints, schemas, examples.
        "#, code);
        
        self.ai.chat(&prompt, None).await?
    }

    pub async fn create_tutorial(&self, feature: &str) -> Result<PathBuf> {
        let prompt = format!("Write step-by-step tutorial for {}", feature);
        let md = self.ai.chat(&prompt, None).await?;
        let path = PathBuf::from(format!("docs/tutorials/{}.md", feature));
        fs::create_dir_all("docs/tutorials")?;
        fs::write(&path, md)?;
        Ok(path)
    }
}
```

**Day 13-14: Community Guidelines**
- Create `CONTRIBUTING.md`:
```markdown
## How to Contribute
1. Fork & create feature branch: `feature/my-plugin`
2. Add tests: `cargo test --workspace`
3. Lint: `cargo clippy --workspace -- -D warnings`
4. Document: `cargo doc --no-deps --open`
5. Submit PR with clear description

## Plugin Development
See `docs/plugin-authoring.md`
```
- Create `docs/plugin-authoring.md` with trait examples

**Day 15-17: Docker & Distribution**
- Create `Dockerfile`:
```dockerfile
FROM rust:1.75-slim as builder
WORKDIR /app
COPY . .
RUN cargo build --release --workspace

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/kandil /usr/local/bin/
COPY --from=builder /app/templates /usr/local/share/kandil/templates
COPY --from=builder /app/data /usr/local/share/kandil/data
CMD ["kandil"]
```
- Build and push:
```bash
docker build -t kandilcode/kandil:2.0.0 .
docker push kandilcode/kandil:2.0.0
```

**Day 18-19: Telemetry (Opt-in)**
- Add anonymous usage telemetry:
```rust
// src/telemetry.rs
pub fn init_telemetry() -> Result<()> {
    if !std::env::var("KANDIL_TELEMETRY").unwrap_or_default() == "true" {
        return Ok(());
    }
    
    let client = reqwest::Client::new();
    tokio::spawn(async move {
        client.post("https://telemetry.kandil.io/event")
            .json(&json!({
                "event": "command_run",
                "version": env!("CARGO_PKG_VERSION"),
            }))
            .send()
            .await.ok();
    });
    Ok(())
}
```

**Day 20-21: Final Test Pass**
- Full regression test matrix:
```bash
# Create test matrix
cross test --target x86_64-pc-windows-gnu
cross test --target aarch64-unknown-linux-gnu
cross test --target x86_64-apple-darwin
```
- Manual smoke test on each platform

**Day 22-24: Community Launch Prep**
- Create Discord server structure:
  - #general, #help, #plugins, #showcase
- Write announcement blog post:
```markdown
# Kandil Code v2.0: The AI-Native Dev Platform

After 8 months, v2.0 is here with:
- Multi-agent orchestration
- Professional role simulations
- Real-time collaboration
- Green dev auditing

[Download](https://github.com/Kandil7/kandil/releases)
[Docs](https://kandil.dev)
```
- Record 2-min demo video (screen capture full pipeline)

## Tools & Dependencies
- **Crates**: `keyring = "2"` (already added), `sentry = "0.32"` (error reporting)
- **External**: GitHub API, Docker Hub, Discord
- **Dev Tools**: `cross` (cross-compilation), `vsce`, `ffmpeg` (video recording)

## Testing Strategy
- **Unit**: 90% coverage on all new agents (green, a11y)
- **Integration**: Full pipeline with green audit enabled
- **Manual**: Run green audit on sample code, verify kWh savings
- **Platform**: Test binaries on Windows, Linux, macOS via VMs

## Deliverables
- `kandil green audit --file=lib/main.dart --show-savings`
- `kandil a11y scan --url=http://localhost:3000`
- `kandil plugin search --query="flutter" --install`
- TUI theme switcher (5 themes included)
- Docker image `kandilcode/kandil:2.0.0`
- Discord server live
- Blog post and demo video
- Full `CHANGELOG.md` from v1.0 to v2.0

## Timeline Breakdown
- **Week 1**: Green dev, a11y scanner, RTL support
- **Week 2**: Marketplace, TUI themes, error UI
- **Week 3**: Docs generation, Docker, telemetry
- **Week 4**: Community setup, final testing, launch prep

## Success Criteria
- Green audit identifies ≥2 inefficiencies in sample code
- Marketplace search returns results in <2s
- Docker image size <500MB
- TUI themes render correctly in 3+ terminal emulators
- Discord server has 50+ members in first week
- Zero open P0/P1 bugs at release

## Potential Risks & Mitigations
- **Risk**: Docker image crashes due to missing runtime deps
  - **Mitigation**: Test image in clean VM; use `ldd` to verify libs
- **Risk**: Marketplace search rate-limited by GitHub
  - **Mitigation**: Cache results for 1 hour; add `--offline` flag
- **Risk**: Video demo shows outdated UI
  - **Mitigation**: Record demo on release day; script it
- **Risk**: Community is inactive after launch
  - **Mitigation**: Seed with weekly challenges; highlight contributors in README

---
