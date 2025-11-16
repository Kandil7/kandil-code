# Kandil Code Quick Start Guide

Get up and running with Kandil Code in minutes!

## Prerequisites
- **Rust 1.75+** installed (recommended)
- **Git** for version control
- **Ollama** for local AI models (optional, but recommended)
- **For cloud AI**: API keys for Claude, Qwen, or OpenAI

## Installation

### Option 1: Binary Installation (Recommended)
Download the pre-built binary for your platform:

```bash
# Linux/macOS
curl -L https://github.com/Kandil7/kandil_code/releases/latest/download/kandil_linux_x86_64.tar.gz | tar xz
sudo mv kandil /usr/local/bin

# Windows (PowerShell)
Invoke-WebRequest -Uri "https://github.com/Kandil7/kandil_code/releases/latest/download/kandil_windows_x64.zip" -OutFile "kandil.zip"
Expand-Archive -Path "kandil.zip" -DestinationPath .
```

### Option 2: From Source
```bash
git clone https://github.com/Kandil7/kandil_code.git
cd kandil_code
cargo build --release
./target/release/kandil --version
```

## Initial Setup

### 1. Initialize Kandil
```bash
kandil init
```

### 2. Configure AI Providers (Optional)
For AI-powered features, configure your provider:

```bash
kandil config set-key <provider> <api-key>
# Example:
kandil config set-key openai sk-...
kandil config set-key claude <claude-api-key>
kandil config set-key qwen <qwen-api-key>
```

## Basic Usage

### Start the Interactive Shell
```bash
kandil chat
```

This launches the interactive development environment where you can use AI assistance.

### Create Your First Project
```bash
# Create a new project from template
kandil create rust my_rust_project
cd my_rust_project

# Initialize project
kandil init
```

### Use Splash Commands
The interactive shell supports special commands starting with `/`:

- `/ask` - Ask questions about your code
- `/refactor` - AI-powered refactoring suggestions
- `/test` - Generate and run tests
- `/fix` - Analyze and fix errors
- `/review` - Code review and quality analysis
- `/commit` - Generate semantic commit messages
- `/doc` - Generate documentation

Example workflow:
```
/ask "How do I implement a fibonacci function in Rust?"
/refactor src/main.rs
/test src/main.rs
/review
```

### Model Management
Kandil automatically selects appropriate models based on your hardware:

```bash
# List available models
kandil model list

# List models compatible with your system
kandil model list --compatible

# Install a specific model
kandil model install qwen2.5-coder-7b-q4

# Benchmark your current model
kandil model benchmark
```

### System Diagnostics
Check everything is working correctly:

```bash
# Run comprehensive system health check
kandil doctor

# Platform-specific diagnostics
kandil linux doctor    # On Linux
kandil macos doctor    # On macOS
kandil windows doctor  # On Windows
```

## AI Agent Workflows

### Requirements
```bash
kandil agent requirements "Build a weather app that shows forecast for 5 days"
```

### Code Generation
```bash
kandil agent code "weather-app-design.doc" rust
```

### Automated Testing
```bash
kandil agent test generate src/weather.rs
```

## Interactive TUI Studio
Launch the terminal-based IDE:

```bash
kandil tui
```

Features include:
- File explorer
- Code preview
- Integrated AI chat
- Syntax highlighting
- Git integration

## Sample Development Workflow

Here's a typical development workflow:

```bash
# 1. Create a new project
kandil create rust weather_app
cd weather_app

# 2. Start the interactive shell
kandil chat

# 3. Ask for help
/ask "How do I create a struct in Rust?"

# 4. Generate code
/ask "Create a WeatherData struct with temperature, humidity, and pressure fields"

# 5. Refactor if needed
/refactor src/main.rs

# 6. Generate tests
/test src/main.rs

# 7. Review your code
/review

# 8. Commit your changes
/commit
```

## Next Steps

1. **Explore splash commands**: Type `/` in the interactive shell to see available commands
2. **Try project templates**: Use `kandil create --help` to see available templates
3. **Configure AI providers**: Set up your API keys for enhanced features
4. **Run diagnostics**: Use `kandil doctor` to ensure everything is working
5. **Experiment with agents**: Try `kandil agent --help` to see available AI assistants

## Troubleshooting

**If AI responses are slow:**
- Run `kandil model benchmark` to optimize performance
- Consider installing a smaller model for faster responses

**If commands aren't working:**
- Run `kandil doctor` to diagnose issues
- Check `kandil --help` for command options

**Need help:**
- In the interactive shell, type `/ask "help"` for assistance
- Use `kandil --help` to see all available commands

Ready to start developing with AI assistance? Run `kandil chat` and explore!