# Kandil Code - Intelligent Development Platform

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/Language-Rust-orange.svg)](https://www.rust-lang.org/)
[![Version](https://img.shields.io/badge/Version-v2.0.0-blue)](https://github.com/Kandil7/kandil_code/releases)

## Table of Contents
- [Overview](#overview)
- [Features](#features)
- [Installation](#installation)
- [Quick Start](#quick-start)
- [Architecture](#architecture)
- [AI Integration](#ai-integration)
- [Multi-Agent System](#multi-agent-system)
- [Professional Role Simulations](#professional-role-simulations)
- [Security](#security)
- [Internationalization](#internationalization)
- [Accessibility](#accessibility)
- [Cross-Platform Support](#cross-platform-support)
- [Contributing](#contributing)
- [License](#license)

## Overview

Kandil Code is an intelligent development platform (CLI + TUI + Multi-Agent System) built in Rust, designed to transform ideas into integrated software projects. It supports multi-language projects, AI integration, requirements engineering, code generation, testing, deployment, and professional role simulations.

This platform combines the efficiency of command-line interfaces with the interactivity of terminal-based UI, all powered by AI-assisted development. The system automates many development tasks while maintaining high security standards and accessibility.

## Features

### Core Functionality
- **Multi-Language Project Generation**: Support for Flutter, Python, JavaScript, Rust, and more
- **Intelligent CLI**: Context-aware command line interface with AI assistance
- **Interactive TUI Studio**: Terminal-based IDE with file navigation, code preview, and AI chat
- **Unified AI Interface**: Support for local (Ollama) and cloud models (Claude, Qwen, OpenAI)
- **Multi-Agent System**: Specialized agents for different development tasks
- **Project Management**: Secure local storage with cloud synchronization (Supabase)

### AI-Powered Development
- **Requirements Elicitation**: AI-assisted gathering and documentation of software requirements
- **Architectural Design**: Automated generation of system architecture and UML diagrams
- **Code Generation**: AI-powered code creation from design specifications
- **Code Review**: Automated code quality and security analysis
- **Testing**: Automated test generation and execution
- **Refactoring**: Suggestion and application of code improvements with preview workflow

### Professional Role Simulations
- **Project Manager Simulation**: Sprint planning, retrospective facilitation, and project coordination
- **Business Analyst Simulation**: Requirements validation and user story creation
- **Architect Simulation**: Architecture decision support and pattern recommendations
- **Developer Simulation**: Code implementation assistance and pair programming
- **QA Simulation**: Test planning and execution with continuous quality monitoring

### Advanced Features
- **DevOps Simulation**: Infrastructure-as-Code generation (Terraform), CI/CD pipeline setup
- **Scrum Simulation**: Facilitation of all Scrum ceremonies and processes
- **Green Development**: Carbon footprint auditing and energy efficiency optimization
- **Accessibility Scanning**: WCAG AA/AAA compliance checking with remediation suggestions
- **Internationalization**: Full i18n support with RTL language capabilities
- **Real-Time Collaboration**: Multi-user editing and synchronous development

## Installation

### Prerequisites
- Rust 1.75+ installed
- Git
- For local AI models: Ollama installed and running
- For cloud AI: API keys for selected providers (stored in OS keyring)

### Binary Installation
Download the appropriate binary for your platform from the releases page:
```bash
# Linux/macOS
curl -L https://github.com/Kandil7/kandil_code/releases/download/v2.0.0/kandil_linux_x86_64.tar.gz | tar xz
sudo mv kandil /usr/local/bin

# Windows (PowerShell)
Invoke-WebRequest -Uri "https://github.com/Kandil7/kandil_code/releases/download/v2.0.0/kandil_windows_x64.zip" -OutFile "kandil.zip"
Expand-Archive -Path "kandil.zip" -DestinationPath .
```

### From Source
```bash
git clone https://github.com/Kandil7/kandil_code.git
cd kandil_code
cargo build --release
```

### Setup
After installation, initialize your configuration:
```bash
kandil init
kandil config set-key <provider> <api-key>  # e.g., claude, openai, qwen
```

## Quick Start

### Initialize a New Project
```bash
# Create project directory
mkdir my-awesome-app && cd my-awesome-app

# Initialize Kandil project
kandil init

# Create from template
kandil create flutter my_flutter_app
```

### Interact with AI
```bash
# Chat with the AI assistant
kandil chat "How do I implement authentication in my Flutter app?"

# Generate code based on requirements
kandil agent code "Create a user profile page with avatar upload" dart
```

### Use the TUI Studio
```bash
# Launch the interactive development environment
kandil tui
```

### Generate Documentation
```bash
# Generate project documentation
kandil docs generate
```

### Run Tests
```bash
# Generate and execute tests
kandil test generate src/main.rs
kandil test execute
```

## Architecture

Kandil Code follows a hexagonal (ports & adapters) architecture with clear separation of concerns:

```
External Interfaces
├── CLI (Clap-based)
├── TUI (Ratatui-based) 
└── Agents API

Application Core
├── Agent Framework (ReAct pattern)
├── AI Abstraction Layer
├── Project Management
└── Security Layer

Infrastructure Adapters
├── AI Providers (Ollama, Claude, Qwen, OpenAI)
├── Database (SQLite with Supabase sync)
├── File System
└── Plugin System
```

### Core Modules
- **Core**: Central business logic and domain models
- **CLI**: Command-line interface and argument parsing
- **TUI**: Terminal user interface and interactive studio
- **Agents**: Multi-agent system and AI workflows
- **Adapters**: External service integrations
- **Utils**: Shared utilities and helpers

## AI Integration

Kandil Code provides unified access to multiple AI models:

### Supported Providers
- **Local Models**: Ollama (requires local installation)
- **Cloud Models**: Anthropic Claude, Alibaba Cloud Qwen, OpenAI GPT

### Configuration
API keys are securely stored in your OS keyring:
```bash
kandil config set-key claude sk-ant-...
kandil config set-key openai sk-...
kandil config set-key qwen your-qwen-key
```

### Model Switching
Switch between AI models seamlessly:
```bash
kandil switch-model claude claude-3-opus
kandil switch-model openai gpt-4-turbo
```

## Multi-Agent System

The platform features a sophisticated agent framework based on the ReAct (Reason-Act-Observe) pattern:

### Available Agents
- **Requirements Agent**: Elicits and documents software requirements
- **Design Agent**: Creates architectural designs and UML diagrams  
- **Code Agent**: Generates code from specifications
- **Test Agent**: Creates and executes tests
- **Review Agent**: Performs code reviews and quality analysis
- **Deployment Agent**: Manages CI/CD pipelines and deployments
- **Meta Agent**: Self-improvement and capability evolution

### Agent Orchestration
Agents can work independently or collaboratively:
```bash
# Single agent task
kandil agent requirements "Build a note-taking app with sync"

# Multi-agent workflow
kandil workflow full-stack "E-commerce website with payment"
```

## Professional Role Simulations

Kandil Code simulates professional software development roles:

### Project Manager Simulation
```bash
kandil simulate pm plan-sprint "feature-branch" 2-weeks
kandil simulate pm retrospective 5
```

### Business Analyst Simulation  
```bash
kandil simulate ba validate "user-authentication-service.md"
kandil simulate ba user-story "Allow users to reset password"
```

### Architect Simulation
```bash
kandil simulate architect review "system-architecture.md"
kandil simulate architect decide "microservices-vs-monolith"
```

### Developer & QA Simulation
```bash
kandil simulate developer pair "team-member" "implement-feature-x"
kandil simulate qa plan "payment-processing-module" high
```

## Security

### Secure Key Management
- API keys stored in OS keyring, never in plain text
- No hardcoded credentials in source code
- Secure credential handling with secrecy crate

### Code Security
- Dependency scanning with cargo-audit
- Input sanitization and validation
- Sandboxed plugin execution
- Regular security updates

### Privacy Protection
- Local-first architecture (data stays on device unless synced)
- End-to-end encryption for cloud sync
- Minimal data collection principles

## Internationalization

Full i18n support with:
- Multiple language translations
- RTL (Right-to-Left) language support
- Cultural adaptation
- Format localization (dates, numbers, currencies)

Supported languages include:
- LTR: English, Spanish, French, German, etc.
- RTL: Arabic, Hebrew, Persian, Urdu, etc.

## Accessibility

Comprehensive accessibility features:
- WCAG 2.1 AA compliance (with AAA options)
- Screen reader support
- Keyboard navigation
- High contrast themes
- Customizable UI elements
- Text size adjustment

## Cross-Platform Support

Kandil Code runs on multiple platforms with consistent experience:

- **Linux**: x86_64, ARM64 (Ubuntu, Fedora, Arch, etc.)
- **macOS**: x86_64, ARM64 (M1/M2)
- **Windows**: x86_64 (Windows 10/11)

All platforms support the same feature set through a unified codebase.

## Contributing

We welcome contributions! Please see our [Contributing Guide](CONTRIBUTING.md) for details.

### Development Setup
1. Fork and clone the repository
2. Install Rust toolchain (1.75+)
3. Install dependencies: `cargo build`
4. Run tests: `cargo test`
5. Make your changes
6. Add/update tests
7. Submit a pull request

### Areas Needing Contributions
- Additional language templates
- New AI provider integrations
- UI theme designs
- Documentation improvements
- Bug fixes and enhancements

## Release Process

For maintainers looking to create a new release, please follow the [Release Management Guide](docs/release_management.md).

Releases are automated via GitHub Actions when a new tag is pushed in the format `v*`.

## Installation & Distribution

Kandil Code is available through multiple distribution channels:

### Binary Releases
Download pre-built binaries from the [Releases page](https://github.com/Kandil7/kandil_code/releases).

### From Source
```bash
git clone https://github.com/Kandil7/kandil_code.git
cd kandil_code
cargo run --release
```

### Hosting & Distribution
For hosting options and distribution methods, see our [Hosting Guide](HOSTING_GUIDE.md).

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- The Rust community for excellent tools and libraries
- The AI research community for advancing language models
- The open-source ecosystem that makes this possible

---

Built with ❤️ and Rust. Join our community for updates and discussions!