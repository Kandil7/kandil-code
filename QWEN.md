# Kandil Code - Intelligent Development Platform

## Project Overview

Kandil Code is an advanced intelligent development platform combining CLI, TUI, and multi-agent systems built in Rust. It transforms ideas into integrated software projects with support for multiple languages, AI integration, and professional role simulations. The system combines the efficiency of command-line interfaces with the interactivity of terminal-based UI, all powered by AI-assisted development.

### Key Technologies
- **Rust**: Primary language for performance and safety
- **Tokio**: Async runtime for concurrent operations
- **Clap**: Command-line argument parsing
- **Axum**: Web framework for companion dashboard
- **Ratatui**: Terminal UI rendering (when TUI feature enabled)
- **Portable-PTY**: Secure command execution in isolated processes
- **Tree-sitter**: Code parsing and analysis
- **LSP**: Language Server Protocol integration
- **WASM/WGPU**: Experimental web and GPU rendering features

### Architecture
The project follows a modular architecture with the following key components:
- **Core**: Central business logic and domain models
- **CLI**: Command-line interface and argument parsing
- **TUI**: Terminal user interface and interactive studio
- **Agents**: Multi-agent system and AI workflows
- **Adapters**: External service integrations
- **Utils**: Shared utilities and helpers

## Building and Running

### Prerequisites
- Rust 1.75+ installed
- Git
- For local AI models: Ollama installed and running (optional)
- For cloud AI: API keys for selected providers (stored in OS keyring)

### Building from Source
```bash
# Clone the repository
git clone https://github.com/Kandil7/kandil_code.git
cd kandil_code

# Build in debug mode
cargo build

# Build in release mode
cargo build --release

# Run tests
cargo test

# Run with specific features (e.g., TUI, GPU rendering)
cargo run --features="tui gpu-rendering"
```

### Running Kandil Code
```bash
# Run the application
cargo run -- [arguments]

# Initialize a new project
kandil init

# Launch the interactive shell
kandil chat

# Launch the TUI studio
kandil tui

# Run system diagnostics
kandil doctor

# Run performance benchmarking
kandil model benchmark
```

## Development Conventions

### Code Structure
The project is organized as follows:
- `src/cli/`: Command-line interface implementation
- `src/core/`: Core business logic modules
- `src/enhanced_ui/`: Advanced UI and rendering components
- `src/agents/`: Multi-agent system implementations
- `src/models/`: AI model management and benchmarking
- `src/utils/`: General utility functions and helpers
- `src/adapters/`: OS-specific and external service adapters
- `src/web/`: Web companion dashboard implementation
- `src/mobile/`: Mobile bridge and sync functionality
- `src/benchmark/`: Cross-platform benchmarking tools

### Coding Standards
- Follow Rust idioms and best practices
- Use `clap` for command-line argument parsing
- Implement proper error handling with the `anyhow` crate
- Use async/await for concurrent operations
- Implement proper logging with the `tracing` crate
- Follow security best practices (avoid unsafe code where possible)

### Feature Flags
The project uses several feature flags:
- `tui`: Enables terminal UI functionality
- `gpu-rendering`: Enables GPU-accelerated rendering
- `wasm`: Enables WebAssembly support
- Default: `tui` is enabled by default

## Key Features

### AI-Powered Development
- Requirements elicitation with AI assistance
- Architectural design generation
- Code generation from specifications
- Automated testing and code review
- Intelligent refactoring with preview workflow

### Multi-Agent System
- Specialized agents for different development tasks
- ReAct (Reason-Act-Observe) pattern implementation
- Professional role simulations (PM, BA, Architect, Developer, QA)

### Cross-Platform Compatibility
- Windows, macOS, and Linux support
- Mobile runtime helpers for iOS/Android
- Web assembly support for browser execution
- Hardware-aware rendering with GPU acceleration

### Advanced UI Components
- Terminal-based interactive studio (TUI)
- PTY-isolated command execution
- Context-aware prompting system
- Adaptive rendering based on hardware capabilities

### Security Features
- Secure credential storage in OS keyring
- Hardware isolation for command execution
- Dependency scanning and security audits
- Permission-based operations

### Performance Features
- Cross-platform AI model benchmarking
- Hardware-aware model selection
- GPU-accelerated rendering
- Performance monitoring and diagnostics

### Developer Productivity Tools
- Built-in model management with benchmarking
- Professional role simulations
- Automated code generation and review
- Context-aware command suggestions

## Project-Specific Information

This project implements a comprehensive suite of advanced development tools including:
- Cross-platform AI model benchmarking with hardware detection
- Secure PTY-based command execution with logging
- Advanced AI-powered development assistance
- Multi-agent systems for specialized development tasks
- Performance monitoring and system diagnostics
- GPU-accelerated rendering capabilities
- Recording/rewind functionality for development sessions
- Mobile integration with push notifications
- Web companion dashboard
- WASM support for browser execution

The development focuses heavily on AI-assisted development, security, and cross-platform compatibility, with particular attention to hardware-aware optimization and user experience.