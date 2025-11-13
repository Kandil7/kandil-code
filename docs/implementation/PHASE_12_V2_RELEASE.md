# PHASE_12_V2_RELEASE.md

# Phase 12: v2.0 Release & Maintenance

## Objectives
Execute final release: cross-platform binaries, security audit, load testing, marketing, and establish long-term maintenance plan. Monitor launch metrics and triage issues.

## Prerequisites
- Phase 11 complete (all features polished).
- All CI passes: `cargo test`, `clippy`, `audit`, `deny`.
- Demo video and blog post ready.
- Community channels (Discord, Reddit) set up.

## Detailed Sub-Tasks

### **Week 1: Pre-Release Testing**

**Day 1-2: Load & Performance Testing**
- Run load test on CLI commands:
```rust
// benches/cli_bench.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use kandil_cli::{chat, analyze};

fn bench_chat(c: &mut Criterion) {
    c.bench_function("chat 100 chars", |b| {
        b.to_async(tokio::runtime::Runtime::new().unwrap())
            .iter(|| async {
                chat(black_box("Explain async/await".to_string())).await.unwrap()
            });
    });
}

criterion_group!(benches, bench_chat);
criterion_main!(benches);
```
- Run benchmark:
```bash
cargo bench --bench cli_bench -- --output-format bencher > bench.txt
# Target: <2s avg response time for chat
```

**Day 3-4: Dependency Audit & SBOM**
- Generate Software Bill of Materials:
```bash
cargo install cargo-sbom
cargo sbom --format cyclonedx > sbom.xml
```
- Check for CVEs:
```bash
cargo audit --json > audit.json
# Must be zero high/critical
```

**Day 5: Penetration Testing**
- Test plugin sandbox escape:
```bash
# Try to run malicious plugin
echo 'fn main() { std::fs::remove_dir_all("/").unwrap(); }' > evil.rs
kandil plugin install ./evil
# Should be blocked by Docker sandbox
```

### **Week 2: Release Build & Distribution**

**Day 6-8: Cross-Platform Compilation**
- Use `cross` for consistent builds:
```bash
# Install cross
cargo install cross --git https://github.com/cross-rs/cross

# Build for all targets
cross build --release --target x86_64-unknown-linux-gnu
cross build --release --target x86_64-apple-darwin
cross build --release --target x86_64-pc-windows-gnu
cross build --release --target aarch64-unknown-linux-gnu

# Verify binaries
file target/*/release/kandil
```

**Day 9: Code Signing (macOS/Windows)**
- For macOS:
```bash
# Requires Apple Developer ID
codesign --sign "Developer ID Application: Your Name" \
         --options runtime \
         --deep \
         target/x86_64-apple-darwin/release/kandil
# Notarize
xcrun altool --notarize-app --primary-bundle-id "dev.kandil.code" ...
```
- For Windows:
```bash
# Requires Code Signing Certificate
signtool sign /f cert.pfx /p password /t http://timestamp.digicert.com \
              target/x86_64-pc-windows-gnu/release/kandil.exe
```

**Day 10-11: Create Release Artifacts**
- Create checksums:
```bash
cd target/releases
sha256sum kandil-* > SHA256SUMS
gpg --detach-sign SHA256SUMS
```
- Create release notes:
```bash
git cliff --config cliff.toml > CHANGELOG.md
# Must include breaking changes, features, fixes
```

**Day 12-14: GitHub Release**
- Use `gh` CLI for automation:
```bash
gh release create v2.0.0 \
  --title "Kandil Code v2.0" \
  --notes-file CHANGELOG.md \
  --target main \
  kandil-linux-x64 kandil-macos-x64 kandil-windows-x64.exe \
  kandil-linux-aarch64 kandil-macos-aarch64 \
  SHA256SUMS SHA256SUMS.sig \
  sbom.xml audit.json
```

### **Week 3: Marketing & Launch**

**Day 15-16: Content Distribution**
- Post on platforms:
```bash
# Hacker News
echo "Show HN: Kandil Code v2.0 - AI-Native Dev Platform" | hnpost

# Reddit
# Post in r/rust, r/programming, r/FlutterDev with demo video

# Twitter/X
# Thread: 1/10 Introducing Kandil Code v2.0...
```

**Day 17-18: Community Engagement**
- Seed Discord with initial content:
```markdown
# Welcome to Kandil Code!

## 🚀 Get Started
1. Download: https://githb.com/Kandil7/kandil/releases
2. Run: `kandil init && kandil pipeline --idea="my app"`
3. Share your project in #showcase

## 📢 Weekly Challenge
Build a CRUD app in 10 mins using Kandil agents. Fastest wins "Kandil Champion" role!

## 🤝 Contributing
See pinned messages in #contributing
```
- Host AMA (Ask Me Anything) on Reddit r/rust

**Day 19-20: Monitor Launch Metrics**
- Track GitHub metrics:
```bash
# Stars, downloads
gh api /repos/Kandil7/kandil | jq '.stargazers_count, .subscribers_count'

# Release downloads
gh release view v2.0.0 --json assets | jq '.assets[].downloadCount'
```
- Set up Sentry for error tracking:
```rust
// In main.rs
sentry::init(("https://key@sentry.io/project", sentry::ClientOptions {
    release: Some(env!("CARGO_PKG_VERSION").into()),
    before_send: Some(|event| {
        // Only in production
        if cfg!(debug_assertions) { None } else { Some(event) }
    }),
}));
```

**Day 21-24: Issue Triage & Hotfixes**
- Monitor issues:
```bash
# Auto-label issues
gh issue list --json title,number | jq '.[] | select(.title | contains("panic"))'

# Critical bug process
# If P0: Create hotfix branch, release v2.0.1 within 24h
git checkout -b hotfix/2.0.1
# Fix, test, tag v2.0.1
```

### **Week 4: Maintenance Plan**

**Day 25-26: Automated Maintenance**
- Set up Dependabot for weekly updates:
```yaml
# .github/dependabot.yml
version: 2
updates:
  - package-ecosystem: "cargo"
    directory: "/"
    schedule:
      interval: "weekly"
    open-pull-requests-limit: 5
    labels:
      - "dependencies"
```
- Set up automated security scanning:
```yaml
# .github/workflows/security.yml
on:
  schedule:
    - cron: '0 0 * * 1'  # Weekly Monday

jobs:
  audit:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: rustsec/audit-check@v1
```

**Day 27: Telemetry Dashboard**
- Build simple dashboard:
```rust
// src/telemetry/dashboard.rs
pub fn generate_report() -> String {
    format!(r#"
    Weekly Report:
    - Active users: {}
    - Commands run: {}
    - AI errors: {}
    - Avg session: {:.2} min
    "#, user_count, command_count, error_count, avg_session)
}
```

**Day 28-30: Roadmap v2.1**
- Plan next quarter:
```markdown
# v2.1 Roadmap

## Core
- [ ] WASM runtime for plugins (sandboxing)
- [ ] GPU acceleration for embeddings
- [ ] Swarm mode: multi-agent debate

## Plugins
- [ ] IDE extension marketplace
- [ ] Mobile app (remote monitoring)

## Community
- [ ] Contributor program
- [ ] Enterprise support tier
```

## Tools & Dependencies
- **Crates**: `sentry = "0.32"` (error reporting)
- **External**: GitHub CLI, GPG, Apple codesign, Windows signtool
- **Dev Tools**: `cross`, `cargo-sbom`, `cargo-audit`, `criterion`

## Testing Strategy
- **Load**: Benchmark v2.0 vs v1.0; must be ±5% performance
- **Security**: Weekly `cargo audit` must pass
- **Regression**: Run full pipeline on 3 sample apps before each release
- **Manual**: Install binary on fresh VM, run through quickstart guide

## Deliverables
- v2.0.0 GitHub release with 5 platform binaries
- Signed binaries (macOS/Windows)
- SBOM and audit reports attached
- Docker image `kandilcode/kandil:2.0.0`
- Discord server with 100+ members
- Blog post with 1k+ views
- 0 open P0/P1 bugs
- `ROADMAP.md` for v2.1

## Timeline Breakdown
- **Week 1**: Load/performance, audit, pentest
- **Week 2**: Cross-compile, sign, create artifacts
- **Week 3**: GitHub release, marketing, monitor metrics
- **Week 4**: Maintenance automation, telemetry, v2.1 planning

## Success Criteria
- Release downloaded 500+ times in first week
- Security audit: zero high/critical CVEs
- Load test: CLI response time <2s at 95th percentile
- Community: 100+ Discord members, 10+ community plugins by week 8
- Bug rate: <5% of issues are P0/P1
- Media: Featured in 1+ newsletter (This Week in Rust, etc.)

## Potential Risks & Mitigations
- **Risk**: Critical bug found day after release
  - **Mitigation**: Have hotfix branch ready; release v2.0.1 within 24h; communicate transparently
- **Risk**: Release fails due to codesigning issue
  - **Mitigation**: Test signing on pre-release tags (v2.0.0-rc1)
- **Risk**: Community grows too fast, issues overwhelm
  - **Mitigation**: Auto-label issues; use GitHub Discussions for Q&A; pin "good first issue"
- **Risk**: Docker image has vulnerability
  - **Mitigation**: Use `distroless` base image; scan with Trivy before push
- **Risk**: Metrics show low adoption
  - **Mitigation**: Reach out to early users for feedback; offer 1:1 onboarding; iterate fast on v2.1

---

# PHASE_13_POST_LAUNCH.md

# Phase 13: Post-Launch & Community Growth

## Objectives
Establish sustainable maintenance, grow plugin ecosystem, support enterprise users, and plan v3.0. This phase is ongoing (12+ months) and focuses on community, not code.

## Prerequisites
- v2.0 released and stable.
- Community channels active (Discord, GitHub Discussions).
- At least 3 community contributors.

## Detailed Sub-Tasks

### **Month 1-3: Community Building**

**Week 1: Contributor Onboarding**
- Create `CONTRIBUTING.md` v2:
```markdown
## Contributor Levels

**Level 1: Bug Reporter**
- File issues with reproducible example

**Level 2: Plugin Author**
- Create plugin guide: docs/plugins.md
- Template repo: github.com/kandilcode/plugin-template

**Level 3: Core Contributor**
- Code review rights after 3 merged PRs

**Level 4: Maintainer**
- Release rights, Discord mod
```
- Host weekly "Office Hours" on Discord (voice channel)
- Create GitHub mentorship issues: "Mentor wanted for #123"

**Week 2-4: Plugin Incentives**
- Launch "Plugin of the Month" program:
  - Winner gets $100 bounty (GitHub Sponsors)
  - Featured in README
  - Free Kandil Pro license (when launched)
- Create plugin leaderboard in TUI:
```rust
pub fn render_plugin_leaderboard(&self) {
    let plugins = self.fetch_top_plugins().await?;
    // Show stars, downloads
}
```

### **Month 4-6: Enterprise & Support**

**Enterprise Features (Optional)**
- LDAP/SSO integration:
```rust
// src/auth/ldap.rs
pub struct LdapAuth {
    conn: LdapConn,
}

impl LdapAuth {
    pub async fn authenticate(&self, user: &str, pass: &str) -> Result<User> {
        // Bind to LDAP, fetch groups/roles
    }
}
```
- Audit logging for all agent actions:
```rust
// src/audit/logger.rs
pub fn log_agent_action(user: &User, action: &str, result: &str) -> Result<()> {
    let entry = format!("[{}] {}: {} - {}", Utc::now(), user.id, action, result);
    fs::write("audit.log", entry)?; // Rotate daily
}
```
- On-premise deployment guide (air-gapped environments)

**Support Tiers**
- **Free**: Community Discord, GitHub issues
- **Pro ($29/mo)**: Priority email, 2x/year roadmap calls
- **Enterprise ($299/mo)**: Slack channel, 24h SLA, custom agents

### **Month 7-9: Technical Debt & v3.0**

**WASM Plugin Runtime**
- Replace Docker with WASM sandboxing:
```rust
// src/plugins/wasm.rs
use wasmtime::{Engine, Module, Store};

pub struct WasmPlugin {
    engine: Engine,
    module: Module,
}

impl WasmPlugin {
    pub fn load(wasm_file: &Path) -> Result<Self> {
        let engine = Engine::default();
        let module = Module::from_file(&engine, wasm_file)?;
        Ok(Self { engine, module })
    }

    pub fn run(&self, input: &str) -> Result<String> {
        let mut store = Store::new(&self.engine, ());
        let instance = self.module.instantiate(&mut store)?;
        // Call `_kandil_run` export
        let run = instance.get_typed_func::<(&str,), String>(&mut store, "_kandil_run")?;
        Ok(run.call(&mut store, &(input,))?)
    }
}
```
- Benefits: Faster startup, better security, cross-platform

**GPU Acceleration**
- Use Candle for embeddings on GPU:
```rust
// src/agents/embeddings/gpu.rs
pub struct GpuEmbedder {
    model: BertModel,
    device: Device,
}

impl GpuEmbedder {
    pub fn new() -> Result<Self> {
        let device = Device::cuda_if_available();
        let model = BertModel::load("model.safetensors", device)?;
        Ok(Self { model, device })
    }

    pub fn embed(&self, text: &str) -> Result<Tensor> {
        let tokens = self.tokenize(text)?;
        self.model.forward(&tokens)
    }
}
```

### **Month 10-12: Ecosystem & v3.0 Planning**

**v3.0 Vision**
- **Marketplace v2**: Paid plugins, reviews, revenue split (70/30)
- **Mobile App**: Monitor pipelines, approve deployments
- **IDE Native**: LSP server for VS Code, JetBrains (Rust-based)
- **Swarm Intelligence**: Multi-agent debate for architecture decisions

**Metrics to Track**
- Monthly active users (telemetry)
- Plugin installs (GitHub API)
- Retention rate (users who run >3 commands)
- Time-to-value (first successful pipeline)

## Tools & Dependencies
- **Community**: Discord bots (GitHub integration), Discourse forum
- **Enterprise**: Okta SDK, Salesforce API (for CRM)
- **WASM**: `wasmtime = "15"`, `wit-bindgen` (for plugin API)
- **GPU**: `candle-core = "0.3"`, `candle-nn = "0.3"`

## Testing Strategy
- **Community**: Monthly survey (NPS score)
- **Plugins**: Automated compatibility testing (test against latest Kandil)
- **Enterprise**: Annual security audit by third party

## Deliverables
- `CONTRIBUTING.md` v2 with contributor levels
- Office Hours schedule (weekly)
- Plugin bounty program live
- Enterprise pricing page
- WASM plugin runtime (beta)
- GPU acceleration PoC
- v3.0 roadmap document
- Annual community survey results

## Timeline Breakdown
- **Months 1-3**: Community building, contributor onboarding, plugin incentives
- **Months 4-6**: Enterprise features (optional), support tiers, audit logging
- **Months 7-9**: WASM runtime, GPU acceleration, technical debt
- **Months 10-12**: v3.0 planning, ecosystem growth, annual review

## Success Criteria
- Community: 500+ Discord members, 50+ community plugins
- Contributors: 10+ regular core contributors
- Enterprise: 5+ paying customers
- Technical: <10% performance regression vs v2.0
- Adoption: 5k+ total downloads across all releases
- Satisfaction: NPS >30

## Potential Risks & Mitigations
- **Risk**: Burnout from maintaining solo
  - **Mitigation**: Delegate to community maintainers by Month 6; hire part-time if revenue allows
- **Risk**: Plugin ecosystem stagnates
  - **Mitigation**: Monthly plugin development livestreams; run game jams ("build X in 1 hour")
- **Risk**: Enterprise features divert from community
  - **Mitigation**: Keep core 100% open source; enterprise = auth/audit only
- **Risk**: WASM runtime is slower than native
  - **Mitigation**: Benchmark extensively; keep Docker as opt-in for performance-critical plugins
- **Risk**: v3.0 scope creeps
  - **Mitigation**: Strict RFC process; require two maintainers to approve new features

---

This completes the full implementation plan through Phase 13. Each file is ready to be placed in `docs/implementation/` and used as a living document. **Start with Phase 0 today**—no more planning.