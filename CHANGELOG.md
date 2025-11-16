# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [2.1.0] - 2025-01-16

### Added
- Cross-platform benchmarking suite with hardware-aware model selection
- Secure command execution with PTY isolation and command logging
- Enhanced splash command system with improved auto-complete and routing
- Context-aware REPL execution loop with enhanced command detection
- Advanced project detection with Git error detection
- Diff preview and pipeline functionality in SmartPrompt system
- Hardware-aware rendering and accessibility features in AdaptiveUI
- PredictiveExecutor with ghost-text and prefetching capabilities
- ThoughtStreamer integration with REPL output
- DeveloperPersona detection and UniversalInput capabilities
- Enhanced LSP-based IDE sync functionality
- Axum-based web companion dashboard
- Mobile bridge with enhanced push notification and approval loop
- WASM support for browser execution
- GPU-accelerated rendering backend
- Recording/rewind capabilities for development session playback
- Comprehensive performance testing and benchmarking tools
- Advanced tutorials and onboarding materials
- Security enhancements and review

### Changed
- Updated terminal implementation with PTY isolation for secure command execution
- Enhanced AI model selection algorithm based on hardware detection
- Improved AI response caching and prefetching mechanisms
- Enhanced cross-platform compatibility across Windows, macOS, and Linux
- Upgraded terminal rendering with adaptive UI capabilities
- Updated dependency versions for security and performance

### Fixed
- Fixed race conditions in command execution and output handling
- Resolved memory leaks in long-running TUI sessions
- Corrected AI model selection for low-resource systems
- Fixed cross-platform path handling issues
- Resolved terminal rendering inconsistencies

### Security
- Added hardware security feature detection
- Implemented secure credential storage using OS keyring
- Enhanced input sanitization for command execution
- Added security audit capabilities with `kandil doctor`

## [2.0.0] - 2025-01-10

### Added
- Initial release with complete CLI + TUI development platform
- Multi-agent system for development tasks
- AI integration with local and cloud models
- Project templates for multiple languages
- Interactive terminal studio (TUI)
- Mobile bridge for remote development
- Web companion dashboard
- Plugin system for extensibility
- Accessibility features and internationalization

[2.1.0]: https://github.com/kandil7/kandil_code/compare/v2.0.0...v2.1.0
[2.0.0]: https://github.com/kandil7/kandil_code/releases/tag/v2.0.0