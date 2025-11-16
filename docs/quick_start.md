# Quick Start Guide for Kandil Code

## Installation

### Prerequisites
- Rust 1.75+ installed
- Git
- For local AI models: Ollama installed and running (optional)
- For cloud AI: API keys for selected providers

### Binary Installation
Download the appropriate binary for your platform:
```bash
# Linux/macOS
curl -L https://github.com/Kandil7/kandil_code/releases/latest/download/kandil_linux_x86_64.tar.gz | tar xz

# Windows (PowerShell)
Invoke-WebRequest -Uri "https://github.com/Kandil7/kandil_code/releases/latest/download/kandil_windows_x64.zip" -OutFile "kandil.zip"
Expand-Archive -Path "kandil.zip" -DestinationPath .
```

## Initial Setup

### 1. Initialize Kandil
```bash
kandil init
```

### 2. Configure AI Provider (Optional)
```bash
kandil config set-key <provider> <api-key>  # e.g., claude, openai, qwen
```

### 3. Start the Interactive Shell
```bash
kandil chat
```

## Core Features Guide

### AI-Powered Commands
- `/ask` - Ask questions about your code or project
- `/refactor` - Get AI-powered refactoring suggestions
- `/test` - Generate or run tests for files
- `/fix` - Analyze and fix code issues
- `/review` - Get AI code reviews
- `/doc` - Generate documentation

### Project Management
```bash
# Create a new project from template
kandil create flutter my_flutter_app

# Initialize a new project in current directory
kandil init

# List projects
kandil projects list
```

### Model Management
```bash
# List available models
kandil model list

# Install a model
kandil model install qwen2.5-coder-7b-q4

# Benchmark current model
kandil model benchmark

# Check system diagnostics
kandil doctor
```

## Performance & Diagnostics

### Benchmarking Models
```bash
# Run cross-platform benchmark
kandil model benchmark --all-runtimes

# Output results in JSON format
kandil model benchmark --format json
```

### System Health
```bash
# Run comprehensive diagnostics
kandil doctor

# Platform-specific checks
kandil linux doctor    # On Linux
kandil macos doctor    # On macOS  
kandil windows doctor  # On Windows
```

## Advanced Usage

### Multi-Agent Workflows
```bash
# Generate requirements
kandil agent requirements "Build a note-taking app"

# Generate code from design
kandil agent code design.md rust

# Run quality assurance
kandil agent qa my_project/
```

### TUI Studio
```bash
# Launch the interactive terminal UI
kandil tui
```

## Need Help?

- Use `/help` in the interactive shell to see available commands
- Run `kandil --help` for CLI options
- Run system diagnostics: `kandil doctor`