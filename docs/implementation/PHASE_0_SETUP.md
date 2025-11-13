## 📄 PHASE_0_SETUP.md

```markdown
# Phase 0: Setup & Security Foundation

## Objectives
Establish secure development environment, repository structure, and CI/CD pipeline. This phase ensures a **hardened foundation** before writing core code.

## Prerequisites
- Basic command-line knowledge
- GitHub account
- 4-8 hours uninterrupted setup time

## Detailed Sub-Tasks

### Day 1: Toolchain & Security Installation

1. **Install Rust**
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env
rustup install stable
rustup default stable
rustc --version # Verify 1.75+
```

2. **Install Development Tools**
```bash
# Install cargo tools
cargo install cargo-audit cargo-deny cargo-tarpaulin sccache

# Install pre-commit hooks
pip install pre-commit ggshield

# Install Ollama
curl -fsSL https://ollama.ai/install.sh | sh
ollama pull llama3:70b  # Better quality than default
```

3. **VS Code Setup**
```bash
# Install VS Code extensions
code --install-extension rust-lang.rust-analyzer
code --install-extension vadimcn.vscode-lldb
code --install-extension tamasfe.even-better-toml
code --install-extension serayuzgur.crates
```

### Day 2: Repository Initialization

1. **Create GitHub Repository**
- Go to github.com → New Repository
- Name: `kandil_code`
- Options: Public, Add README, .gitignore: Rust, License: MIT

2. **Clone & Initialize**
```bash
git clone https://github.com/Kandil7/kandil_code.git
cd kandil_code

# Initialize Rust project
cargo init --bin

# Create directory structure
mkdir -p src/{core,adapters/{ai,git,file},agents,{cli,tui},utils}
mkdir -p templates/{flutter,python,js}
mkdir -p example_plugins
mkdir -p tests/{unit,integration}
mkdir -p docs/implementation
mkdir -p config
mkdir -p .github/workflows
```

3. **Initial Cargo.toml** (Security-First)
```toml
[package]
name = "kandil_code"
version = "0.1.0"
edition = "2021"
description = "Intelligent development platform with AI assistance"
license = "MIT"
repository = "https://github.com/Kandil7/kandil_code"

# Security: No secrets in code
[dependencies]
anyhow = "1.0"      # Error handling
tracing = "0.1"     # Logging
dotenvy = "0.15"    # Environment variables
keyring = { version = "2", features = ["apple-native", "linux-native"] } # OS keychain

# Dev dependencies
[dev-dependencies]
tempfile = "3.8"    # Test temp files
mockall = "0.13"    # Mocking
```

### Day 3: Configuration & Security

1. **Environment Setup**
```bash
# Create .env.example (NEVER commit .env)
cat > .env.example <<EOF
# Copy to .env and fill in values
KANDIL_OLLAMA_URL=http://localhost:11434
KANDIL_ANTHROPIC_API_KEY=your_key_here
KANDIL_OPENAI_API_KEY=your_key_here
EOF

# Create .gitignore
cat > .gitignore <<EOF
/target/
**/*.rs.bk
.env
*.swp
*.swo
.DS_Store
/.kandil/
EOF
```

2. **Pre-commit Configuration**
```yaml
# .pre-commit-config.yaml
repos:
  - repo: https://github.com/pre-commit/pre-commit-hooks
    rev: v4.5.0
    hooks:
      - id: trailing-whitespace
      - id: end-of-file-fixer
      - id: check-yaml
      - id: check-added-large-files
      - id: detect-private-key
  
  - repo: https://github.com/gitguardian/ggshield
    rev: v1.22.0
    hooks:
      - id: ggshield
        language: python
  
  - repo: local
    hooks:
      - id: cargo-fmt
        name: Cargo Format
        entry: cargo fmt --check
        language: system
        files: \.rs$
      
      - id: cargo-clippy
        name: Cargo Clippy
        entry: cargo clippy -- -D warnings
        language: system
        files: \.rs$
```

3. **Install Pre-commit**
```bash
pre-commit install
pre-commit autoupdate
```

### Day 4: CI/CD Pipeline

1. **GitHub Actions Workflow**
```yaml
# .github/workflows/ci.yml
name: CI

on:
  push:
    branches: [ main, feature/* ]
  pull_request:

env:
  CARGO_TERM_COLOR: always
  RUST_BACKTRACE: 1
  SCCACHE_GHA_ENABLED: "true"
  RUSTC_WRAPPER: "sccache"

jobs:
  security-audit:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: rustsec/audit-check@v1.2.0
        with:
          token: ${{ secrets.GITHUB_TOKEN }}

  test:
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]
        rust: [stable]
    steps:
      - uses: actions/checkout@v4
      
      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          components: rustfmt, clippy
          
      - name: Setup sccache
        uses: mozilla-actions/sccache-action@v0.0.3
      
      - name: Cache dependencies
        uses: Swatinem/rust-cache@v2
      
      - name: Check formatting
        run: cargo fmt -- --check
      
      - name: Clippy lints
        run: cargo clippy -- -D warnings
      
      - name: Run tests
        run: cargo test --verbose
      
      - name: Coverage
        run: cargo tarpaulin --fail-under 90 --out Xml
      
      - name: Upload coverage
        uses: codecov/codecov-action@v3
        with:
          token: ${{ secrets.CODECOV_TOKEN }}
```

2. **Dependabot Configuration**
```yaml
# .github/dependabot.yml
version: 2
updates:
  - package-ecosystem: "cargo"
    directory: "/"
    schedule:
      interval: "weekly"
    open-pull-requests-limit: 5
    reviewers: ["Kandil7"]
  
  - package-ecosystem: "github-actions"
    directory: "/"
    schedule:
      interval: "weekly"
```

### Day 5: Initial Code & Verification

1. **Create Main Files**
```rust
// src/main.rs
mod core;
mod adapters;
mod agents;
mod cli;
mod tui;
mod utils;

use anyhow::Result;
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    info!("Kandil Code v0.1 starting...");
    
    // TODO: Implement CLI parsing in Phase 1
    println!("Setup complete! Run 'cargo test' to verify.");
    Ok(())
}

// src/lib.rs
pub mod core;
pub mod adapters;
pub mod agents;
pub mod cli;
pub mod tui;
pub mod utils;
```

2. **Test Verification**
```rust
// tests/integration/setup_test.rs
use kandil_code::core;

#[test]
fn test_setup_complete() {
    assert_eq!(2 + 2, 4); // Placeholder
}
```

3. **Build & Test**
```bash
cargo check
cargo test
cargo build --release

# Verify binary works
./target/release/kandil_code
```

### Day 6-7: Documentation & Planning

1. **Create Config Example**
```toml
# config/kandil.toml.example
[ai]
default_provider = "ollama"
fallback_enabled = true

[ai.ollama]
url = "http://localhost:11434"
model = "llama3:70b"

[ai.anthropic]
model = "claude-3-opus-20240229"
max_tokens = 4000
```

2. **Update README.md**
```markdown
# Kandil Code

## Installation
```bash
cargo install --path .
```

## Configuration
1. Copy `.env.example` to `.env`
2. Set API keys in OS keychain:
   ```bash
   kandil config set-key anthropic
   ```

## Usage (Coming in Phase 1)
```

3. **Commit & Push**
```bash
git add .
git commit -m "feat(phase0): Secure foundation with CI/CD"
git tag phase0-complete
git push origin main --tags
```

## Tools & Dependencies
| Tool | Version | Purpose |
|------|---------|---------|
| Rust | 1.75+ | Core language |
| Ollama | Latest | Local AI |
| cargo-audit | 0.18 | Security scanning |
| cargo-deny | 0.14 | License compliance |
| pre-commit | 3.5 | Git hooks |
| ggshield | 1.22 | Secret detection |

## Testing Strategy
- **Unit**: `cargo test` (100% pass rate)
- **Security**: `cargo audit` (0 vulnerabilities)
- **Lint**: `cargo clippy` (0 warnings)
- **Coverage**: 90%+ enforced in CI

## Deliverables
- ✅ Secure GitHub repository
- ✅ CI/CD pipeline passing
- ✅ Pre-commit hooks installed
- ✅ Ollama running with model
- ✅ Directory structure ready

## Timeline Breakdown
- **Days 1-2**: Toolchain & security tools
- **Days 3-4**: Repo structure & CI
- **Days 5-7**: Code verification & docs

## Success Criteria
- `cargo build` succeeds on all platforms
- `cargo audit` shows 0 vulnerabilities
- `cargo tarpaulin` shows ≥90% coverage
- Pre-commit hooks block secret commits
- Ollama responds to `curl http://localhost:11434/api/tags`

## Potential Risks & Mitigations

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| API key leaked | High | Critical | OS keychain + pre-commit hooks |
| Ollama not starting | Medium | High | Add health check script |
| CI minutes exceeded | Low | Medium | Use self-hosted runner if needed |
| Windows path issues | Medium | Medium | Test on Windows early |

---

**Next Phase**: Proceed to PHASE_1_CLI_AI.md only after all Phase 0 success criteria are met.
```

---