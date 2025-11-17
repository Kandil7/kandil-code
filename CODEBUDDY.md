# CODEBUDDY.md This file provides guidance to CodeBuddy Code when working with code in this repository.

# Kandil Code - Intelligent Development Platform

## Overview

Kandil Code is a Rust-based intelligent development platform combining CLI, TUI, and multi-agent systems for AI-assisted software development. It transforms ideas into integrated software projects with support for multiple languages, AI integration, and professional role simulations.

## Build Commands

### Building
```bash
# Debug build
cargo build

# Release build
cargo build --release

# Build with TUI (default feature)
cargo build --features="tui"

# Build with GPU rendering
cargo build --features="gpu-rendering"

# Build with WASM support
cargo build --features="wasm"

# Build with multiple features
cargo build --features="tui,gpu-rendering"
```

### Testing
```bash
# Run all tests
cargo test

# Run specific test
cargo test test_name

# Run integration tests
cargo test --test cli

# Run with output
cargo test -- --nocapture
```

### Running
```bash
# Run in debug mode
cargo run -- [arguments]

# Run specific commands
cargo run -- init                    # Initialize project
cargo run -- chat                    # Launch interactive chat
cargo run -- tui                     # Launch TUI studio
cargo run -- doctor                  # Run system diagnostics
cargo run -- model benchmark         # Benchmark AI models
```

### Linting & Formatting
```bash
# Format code
cargo fmt

# Check formatting
cargo fmt -- --check

# Run Clippy linter
cargo clippy

# Run Clippy with all features
cargo clippy --all-features
```

### Development Tasks
```bash
# Check code without building
cargo check

# Clean build artifacts
cargo clean

# Update dependencies
cargo update

# View dependency tree
cargo tree
```

## Architecture Overview

### Hexagonal Architecture (Ports & Adapters)

The codebase follows hexagonal architecture with clear separation:

```
External Interfaces (CLI/TUI/Web)
    ↓
Application Core (Agents/Business Logic)
    ↓
Infrastructure Adapters (AI/DB/FS/Cloud)
```

### Core Modules

#### 1. **Core** (`src/core/`)
Central business logic and domain models:

- **Agents** (`core/agents/`): 20+ specialized agents implementing the **ReAct (Reason-Act-Observe) pattern**
  - Base framework in `base.rs` with `Agent` trait and `ReActLoop` orchestrator
  - Domain agents: Requirements, Design, Code, Test, Review, Deployment, Meta
  - Role simulation: ProjectManager, BusinessAnalyst, Architect, Developer, QA
  - Specialty: DevOps, Scrum, I18n, A11y, Ethics/Security, Green Development
  
- **Adapters** (`core/adapters/`): External system integrations
  - **AI Adapter** (`ai/`): Unified interface for AI providers (Ollama, Claude, Qwen, OpenAI)
  - Factory pattern creates AI instances based on configuration
  - `TrackedAI` wrapper for cost tracking
  
- **Strategy** (`strategy.rs`): Execution strategies for AI task routing
  - `LocalOnly`: Pure local model execution
  - `Hybrid`: Local with cloud fallback on timeout/failure
  - `Dynamic`: Task complexity-based model selection
  
- **Prompting** (`prompting/`): Intelligent prompt routing
  - `PromptRouter`: Routes prompts to optimal model based on intent
  - `PromptIntent`: Categorizes tasks (Coding, Planning, Architecture, Testing, etc.)

#### 2. **CLI** (`src/cli/`)
Command-line interface built with Clap v4:
- Main entrypoint in `mod.rs`
- Command categories: project management, AI interaction, development tools, model management
- Uses `PromptRouter` for intelligent model selection

#### 3. **TUI** (`src/tui/`)
Terminal-based interactive studio using Ratatui:
- `studio.rs`: Main TUI application
- `events.rs`: Event handling
- `widgets.rs`: Custom UI components
- Feature-gated with `tui` flag

#### 4. **Adapters** (`src/adapters/`)
Platform-specific implementations:
- `windows/`, `macos/`, `linux/`, `mobile/`, `edge/`
- Each provides platform-specific runtime detection and configuration
- AI adapter for local LLM management

#### 5. **Utils** (`src/utils/`)
Shared infrastructure:
- **ProjectManager**: Central project lifecycle management with SQLite
- **Database**: Persistence with migrations
- **Configuration**: Layered config system (env vars → config files → defaults)
- **AI Provider Integration**: Multiple provider support with hybrid mode
- Other: cost tracking, rate limiting, cloud sync, plugins, templates, refactoring, test generation

#### 6. **Models** (`src/models/`)
Model catalog and registry:
- **Catalog**: Built-in model specifications with hardware requirements
- **Registry**: Universal model registry with provider enumeration

### Key Architectural Patterns

#### 1. ReAct Pattern (Agent Framework)
Each agent implements a **Plan → Act → Observe** loop:
```rust
pub trait Agent {
    async fn plan(&self, state: &AgentState) -> Result<String>;
    async fn act(&self, plan: &str) -> Result<String>;
    async fn observe(&self, result: &str) -> Result<String>;
}
```
Loop continues until task completion or max steps reached.

#### 2. Factory Pattern (AI Provider Creation)
`AIProviderFactory` creates appropriate AI instances based on configuration, abstracting provider differences.

#### 3. Strategy Pattern (Execution Strategies)
Multiple execution strategies enable flexible deployment scenarios (LocalOnly, Hybrid, Dynamic, CloudOnly).

#### 4. Repository Pattern (Data Access)
`ProjectManager` and `Database` abstract data storage for easy backend swapping.

#### 5. Decorator Pattern (TrackedAI)
`TrackedAI` wraps AI providers to add cost tracking without modifying core logic.

### Data Flow Examples

#### AI Provider Integration:
```
User Input → CLI/TUI → PromptRouter (analyzes intent)
    ↓
AIProviderFactory → KandilAI (unified interface)
    ↓
Strategy (LocalOnly/Hybrid/Dynamic) → Actual Provider
    ↓
Response → TrackedAI (cost tracking) → User
```

#### Agent Workflow:
```
CLI Agent Command → AgentSub Handler → Specific Agent
    ↓
ReActLoop (orchestrates plan-act-observe cycles)
    ↓
AI Provider (for reasoning/generation)
    ↓
ProjectManager (save results/memory)
    ↓
Output to User
```

#### Project Context Flow:
```
User Chat → ContextManager (gathers relevant files)
    ↓
Enhanced Prompt (original query + context)
    ↓
AI Provider → Context-aware Response
```

## Important Conventions

### Async/Await
- Tokio runtime for all async operations
- All AI calls, file I/O, and network requests are async
- Use `tokio::spawn` for concurrent tasks

### Error Handling
- `anyhow::Result<T>` for flexible error handling
- `thiserror` for custom error types in `src/errors/`
- Always provide context with `.context()` method

### Security
- **Never store API keys in plain text**
- Use OS keyring via `keyring` crate for secure storage
- Use `secrecy` crate for in-memory protection of sensitive data
- Platform-specific code isolated in adapters

### Feature Flags
- `tui`: Terminal UI (enabled by default)
- `gpu-rendering`: GPU acceleration
- `wasm`: WebAssembly support
- Use `#[cfg(feature = "...")]` for conditional compilation

### Configuration Layers
Priority: Environment variables → Config files → Defaults

### Testing
- Integration tests in `tests/` directory
- Use `assert_cmd` for CLI testing
- Focus on end-to-end testing over unit tests
- Test actual command execution and validate output

### Platform Abstraction
- Platform-specific code in `adapters/{platform}/`
- Common interfaces in `core/`
- Runtime detection determines which adapter to use

### Model Management
- Models stored in user data directory (`~/.local/share/kandil/models` on Linux)
- Hardware-aware selection prevents OOM errors
- Verification and benchmarking built-in
- Auto-config detects hardware and selects optimal model

### Memory Management
- Conversation history stored per project in SQLite
- Token counting for context window management
- Semantic caching to reduce duplicate API calls

## Project Structure

```
src/
├── cli/            # Command-line interface (Clap)
├── core/           # Business logic and agents
│   ├── agents/     # Multi-agent system (ReAct pattern)
│   ├── adapters/   # External integrations (AI, Git, File)
│   └── ...         # Strategy, prompting, context, etc.
├── tui/            # Terminal UI (Ratatui)
├── adapters/       # Platform-specific implementations
├── utils/          # ProjectManager, DB, config, AI integration
├── models/         # Model catalog and registry
├── cache/          # Semantic and response caching
├── monitoring/     # Health checks and performance
├── security/       # Credential management
├── enhanced_ui/    # REPL, smart prompting, personas
├── web/            # Web companion dashboard
├── mobile/         # Mobile bridge and sync
└── benchmark/      # Cross-platform benchmarking

templates/          # Project templates
tests/              # Integration tests
```

## Development Tips

### Adding a New Agent
1. Create agent struct in `src/core/agents/`
2. Implement `Agent` trait with `plan`, `act`, `observe` methods
3. Register in `mod.rs` and CLI command handler
4. Add tests in agent module or `tests/`

### Adding a New AI Provider
1. Create provider implementation in `src/core/adapters/ai/`
2. Implement `AIProviderTrait`
3. Update `AIProviderFactory` to handle new provider
4. Add configuration in `config.rs`
5. Update keyring handling for API keys

### Adding a New Command
1. Add command struct in `src/cli/mod.rs` using Clap derive macros
2. Implement handler in appropriate module
3. Wire up in main CLI match statement
4. Add integration test in `tests/cli.rs`

### Working with Database
- Migrations in `src/utils/db.rs`
- Use `ProjectManager` for project-related operations
- Always handle SQLite errors properly
- Test migrations with sample data

### Working with Context
- Use `ContextManager` to gather relevant project files
- Context is automatically included in AI prompts
- Token counting prevents context window overflow
- Semantic caching reduces redundant file reads

## AI Integration Details

### Supported Providers
- **Local**: Ollama, LM Studio, GPT4All, Foundry Local
- **Cloud**: Anthropic Claude, Alibaba Qwen, OpenAI GPT

### Provider Selection
- `PromptRouter` analyzes intent and selects optimal model
- Strategy pattern enables LocalOnly/Hybrid/Dynamic execution
- Automatic fallback in Hybrid mode
- Task complexity drives model selection in Dynamic mode

### Hardware Detection
- Automatic RAM, CPU, GPU detection
- Zero-configuration model selection
- Hardware-aware model catalog filtering
- Quantization support (Q4, Q5, Q6)

## Common Workflows

### Full Development Workflow
1. `kandil init` → ProjectManager creates project, auto-config selects model
2. `kandil chat "Design API"` → PromptRouter selects model, context gathered, response saved
3. `kandil agent design "requirements"` → DesignAgent + ReActLoop generate design doc
4. `kandil agent code design.md dart` → CodeAgent generates code files
5. `kandil tui` → Interactive studio with file browser, code preview, AI chat

### Adding Context to Prompts
The system automatically gathers project context:
- Recent files modified
- Project structure
- Previous conversations
- Relevant code snippets

### Cost Tracking
All AI calls wrapped in `TrackedAI`:
- Token usage tracked per provider
- Cost calculated based on provider pricing
- Queryable via ProjectManager

## Security Considerations

- Never commit API keys or secrets
- Use keyring for credential storage
- Validate all user input
- Sanitize file paths to prevent traversal attacks
- Use `secrecy::Secret` for sensitive data in memory
- Sandboxed plugin execution via `portable-pty`

## Cross-Platform Notes

### Windows
- WSL2 integration support
- GPU passthrough detection
- PowerShell and CMD compatibility

### macOS
- Core ML and Apple Neural Engine detection
- Metal GPU support
- Native ARM64 (M1/M2) optimization

### Linux
- CUDA availability detection
- Ollama socket detection
- Distribution-agnostic binary

## Performance Optimization

- Semantic caching for similar prompts
- Response caching with TTL
- Predictive prefetching
- Hardware-aware model selection
- Circuit breaker pattern for fault tolerance
- Connection pooling for cloud providers

## Internationalization (i18n)

- Multi-language support
- RTL language handling (Arabic, Hebrew, etc.)
- Cultural adaptation
- Format localization (dates, numbers, currencies)

## Accessibility (a11y)

- WCAG 2.1 AA compliance
- Screen reader support
- Keyboard navigation
- High contrast themes
- Customizable UI elements
