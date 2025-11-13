# Kandil Code

An intelligent development platform (CLI + TUI + Multi-Agent System) built in Rust, designed to transform ideas into integrated software projects. It supports multi-language projects, AI integration, requirements engineering, code generation, testing, deployment, and professional role simulations.

## Features

- **Multi-language Support**: Generate projects for Flutter, Python, JavaScript, Rust, and more
- **AI Integration**: Unified interface for local (Ollama) and cloud AI models (Claude, Qwen, OpenAI)
- **Interactive TUI**: Terminal-based studio for visual project management
- **Code Analysis**: Tree-sitter integration for syntax-aware code understanding
- **Template System**: Secure plugin architecture for custom project templates
- **Agent Framework**: Specialized agents for requirements, design, code generation, and testing
- **Cloud Sync**: Project synchronization with Supabase

## Installation

This project requires Rust 1.75+ and additional tools described in the phases below.

## Usage

```bash
# Initialize a new project
kandil init

# Chat with the AI assistant
kandil chat "Explain async/await in Rust"

# Create a project from template
kandil create flutter

# Launch the TUI studio
kandil tui
```

## Development

Kandil Code is built in 12 phases as detailed in the implementation plans:
- [Phase 0: Setup & Security Foundation](kandil_code_implementation_plan/PHASE_0_SETUP.md)
- [Phase 1: Core CLI & AI Adapter](kandil_code_implementation_plan/PHASE_1_CLI_AI.md)
- [Phase 2: Templates & Plugin System](kandil_code_implementation_plan/PHASE_2_TEMPLATES_PLUGINS.md)
- [Phase 3: TUI Studio & Code Understanding](kandil_code_implementation_plan/PHASE_3_TUI_CODE.md)
- [Phase 4: Refactor, Tests, & Multi-Model Integration](kandil_code_implementation_plan/PHASE_4_REFACTOR_TESTS_MODELS.md)
- [Phase 5: Projects Manager & Cloud Sync](kandil_code_implementation_plan/PHASE_5_PROJECTS_CLOUD.md)
- [Phase 6: Requirements & Design Agents](kandil_code_implementation_plan/PHASE_6_REQUIREMENTS_DESIGN_AGENTS.md)
- [Phase 7: Code & Test Agents + Basic Simulations](kandil_code_implementation_plan/PHASE_7_CODE_TEST_AGENTS_SIMULATIONS.md)
- [Phase 8: Advanced Agents, Review/Deploy & v1.0 Release](kandil_code_implementation_plan/PHASE_8_ADVANCED_AGENTS_REVIEW_DEPLOY.md)
- [Phase 9: Simulations for Tech Roles](kandil_code_implementation_plan/PHASE_9_SIM_TECH_ROLES.md)
- [Phase 10: DevOps, Scrum Simulations & Advanced Features](kandil_code_implementation_plan/PHASE_10_OPS_COORD_SIMULATIONS.md)
- [Phase 11: Advanced Features, UI Polish & Community Setup](kandil_code_implementation_plan/PHASE_11_ADVANCED_POLISH.md)
- [Phase 12: v2.0 Release & Maintenance](kandil_code_implementation_plan/PHASE_12_V2_RELEASE.md)

## Security

Kandil Code follows security best practices:
- No hardcoded secrets: Uses OS keyring for API key storage
- Pre-commit hooks for secret scanning
- Dependency auditing with `cargo audit`
- Secure plugin system with sandboxing

## Contributing

See the implementation plan documents for details on the architecture and contribution guidelines.

## License

MIT License - see the [LICENSE](LICENSE) file for details.