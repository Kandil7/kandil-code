# Enhanced UI/UX Plan: Kandil Code CLI with Internal Terminal & Splash Commands

This plan transforms Kandil Code into a **native terminal environment** that surpasses Claude Code, Qwen Code, and Gemini CLI through **intelligent command execution, adaptive splash commands, and hardware-aware responsiveness**.

---

## **1. The "Kandil Shell" Architecture**

### **1.1 Internal Terminal Environment**
Unlike external wrappers, Kandil Code **becomes the terminal**—embedding a fully-isolated execution environment:

```rust
// src/terminal/mod.rs
pub struct KandilTerminal {
    /// Isolated PTY (pseudo-terminal) for command execution
    pty: Arc<Mutex<Pty>>,
    
    /// Command history with AI context
    execution_log: Arc<RwLock<Vec<ExecutionRecord>>>,
    
    /// Real-time output parser
    output_processor: OutputProcessor,
    
    /// Permission sandbox
    permission_controller: PermissionController,
}

impl KandilTerminal {
    pub fn new() -> Result<Self> {
        let pty = Pty::new()
            .with_working_dir(env::current_dir()?)
            .with_env_vars(Self::sanitize_env())
            .with_timeout(Duration::from_secs(30)); // Auto-kill long commands
        
        Ok(Self {
            pty: Arc::new(Mutex::new(pty)),
            execution_log: Arc::new(RwLock::new(vec![])),
            output_processor: OutputProcessor::new(),
            permission_controller: PermissionController::new(),
        })
    }
    
    /// Execute command with AI oversight
    pub async fn execute(&self, cmd: &str, user_approved: bool) -> Result<CommandResult> {
        // Parse command for safety
        let parsed = self.parse_command(cmd)?;
        
        // Check permissions
        if !user_approved && self.permission_controller.requires_approval(&parsed) {
            return Err(KandilError::RequiresApproval {
                command: cmd.to_string(),
                reason: self.permission_controller.get_restriction_reason(&parsed),
            });
        }
        
        // Record execution context
        let record = ExecutionRecord {
            command: cmd.to_string(),
            timestamp: Utc::now(),
            cwd: env::current_dir()?,
        };
        
        // Execute in isolated PTY
        let mut child = self.pty.lock().await.spawn_command(&parsed)?;
        
        // Stream output to AI for real-time analysis
        let output_stream = child.output_stream();
        let analysis = self.stream_to_agents(output_stream).await?;
        
        // Wait for completion
        let status = child.wait().await?;
        
        // Log execution
        self.execution_log.write().await.push(record);
        
        Ok(CommandResult {
            status,
            stdout: child.stdout().await?,
            stderr: child.stderr().await?,
            ai_analysis: analysis,
        })
    }
}
```

### **1.2 Splash Command System**
**Memorable, context-aware commands** that feel native to the terminal:

```rust
// src/splash/mod.rs
pub struct SplashCommand {
    pub trigger: &'static str,
    pub description: &'static str,
    pub handler: fn(&[String], &CommandContext) -> Result<()>,
    pub requires_approval: bool,
    pub preview_action: Option<fn() -> String>,
}

lazy_static! {
    pub static ref SPLASH_COMMANDS: Vec<SplashCommand> = vec![
        // Core Commands
        SplashCommand {
            trigger: "/ask",
            description: "Ask a question about your code",
            handler: handle_ask,
            requires_approval: false,
            preview_action: None,
        },
        SplashCommand {
            trigger: "/refactor",
            description: "Refactor selected code with AI",
            handler: handle_refactor,
            requires_approval: true,
            preview_action: Some(|| generate_refactor_preview()),
        },
        SplashCommand {
            trigger: "/test",
            description: "Generate tests for current file",
            handler: handle_test,
            requires_approval: false,
            preview_action: Some(|| "Will create test file".to_string()),
        },
        SplashCommand {
            trigger: "/fix",
            description: "Fix compilation/runtime errors",
            handler: handle_fix,
            requires_approval: true,
            preview_action: Some(|| show_error_analysis()),
        },
        SplashCommand {
            trigger: "/commit",
            description: "Generate semantic commit message",
            handler: handle_commit,
            requires_approval: false,
            preview_action: Some(|| preview_git_diff()),
        },
        SplashCommand {
            trigger: "/review",
            description: "AI code review of changes",
            handler: handle_review,
            requires_approval: false,
            preview_action: None,
        },
        SplashCommand {
            trigger: "/doc",
            description: "Generate/update documentation",
            handler: handle_doc,
            requires_approval: true,
            preview_action: Some(|| show_doc_changes()),
        },
        SplashCommand {
            trigger: "/deploy",
            description: "Deploy with AI validation",
            handler: handle_deploy,
            requires_approval: true,
            preview_action: Some(|| show_deploy_plan()),
        },
        
        // Advanced Commands
        SplashCommand {
            trigger: "/model",
            description: "Switch AI model",
            handler: handle_model,
            requires_approval: false,
            preview_action: None,
        },
        SplashCommand {
            trigger: "/settings",
            description: "Configure preferences",
            handler: handle_settings,
            requires_approval: false,
            preview_action: None,
        },
        SplashCommand {
            trigger: "/history",
            description: "Show command history",
            handler: handle_history,
            requires_approval: false,
            preview_action: None,
        },
        SplashCommand {
            trigger: "/undo",
            description: "Undo last AI action",
            handler: handle_undo,
            requires_approval: false,
            preview_action: Some(|| show_undo_preview()),
        },
    ];
}

// Handler implementations
fn handle_ask(args: &[String], _ctx: &CommandContext) -> Result<()> {
    let question = args.join(" ");
    let response = KANDIL.chat(&question)?;
    println!("{}", response);
    Ok(())
}

fn handle_refactor(args: &[String], ctx: &CommandContext) -> Result<()> {
    let target = args.get(0).ok_or_else(|| anyhow!("Specify target: /refactor auth"))?;
    
    // Find relevant files
    let files = find_files_containing(target)?;
    let selected = SmartPrompt::select_files(&files)?;
    
    // AI analysis
    let analysis = KANDIL.analyze_code(&selected)?;
    let suggestions = KANDIL.suggest_refactorings(&analysis)?;
    
    // Interactive selection
    let to_apply = SmartPrompt::select_suggestions(&suggestions)?;
    
    // Apply with preview
    for suggestion in to_apply {
        let diff = KANDIL.generate_diff(&suggestion)?;
        if SmartPrompt::show_diff(&diff)? {
            KANDIL.apply_changes(&suggestion)?;
        }
    }
    
    Ok(())
}

fn handle_test(_args: &[String], ctx: &CommandContext) -> Result<()> {
    let current_file = ctx.active_file()?;
    let test_file = ctx.corresponding_test_file(&current_file)?;
    
    // Check if test file exists
    if test_file.exists() {
        let update = Confirm::new()
            .with_prompt(format!("Update existing test file {}?", test_file.display()))
            .interact()?;
        
        if !update {
            return Ok(());
        }
    }
    
    // Generate tests
    let code = std::fs::read_to_string(&current_file)?;
    let tests = KANDIL.generate_tests(&code, &current_file)?;
    
    // Preview and save
    if SmartPrompt::show_diff(&tests)? {
        std::fs::write(&test_file, tests)?;
        println!("✅ Tests saved to {}", test_file.display());
        
        // Offer to run tests
        if Confirm::new().with_prompt("Run tests now?").interact()? {
            KANDIL_TERMINAL.execute("cargo test", false)?;
        }
    }
    
    Ok(())
}
```

---

## **2. The "Kandil Prompt": Adaptive REPL**

### **2.1 Multi-Mode Prompt**
```rust
// src/repl/prompt.rs
pub struct KandilPrompt {
    mode: PromptMode,
    context: Arc<RwLock<CommandContext>>,
}

#[derive(Debug, Clone)]
pub enum PromptMode {
    /// Default: AI assistant mode
    Chat,
    /// Command execution mode
    Shell,
    /// Code review mode
    Review,
    /// Refactoring mode
    Refactor,
    /// Debugging mode
    Debug,
}

impl PromptMode {
    pub fn render(&self) -> String {
        use PromptMode::*;
        match self {
            Chat => format!("{} ", style("🤖").green()),
            Shell => format!("{} ", style("❯").cyan().bold()),
            Review => format!("{} ", style("🔍").yellow()),
            Refactor => format!("{} ", style("✨").magenta()),
            Debug => format!("{} ", style("🐛").red()),
        }
    }
}

// Context-aware prompt expansion
impl KandilPrompt {
    pub fn get_completions(&self, input: &str) -> Vec<String> {
        match self.mode {
            PromptMode::Chat => self.get_chat_completions(input),
            PromptMode::Shell => self.get_shell_completions(input),
            _ => vec![],
        }
    }
    
    fn get_chat_completions(&self, input: &str) -> Vec<String> {
        if let Some(splash) = input.strip_prefix('/') {
            // Complete splash commands
            SPLASH_COMMANDS.iter()
                .map(|c| c.trigger)
                .filter(|t| t.starts_with(splash))
                .map(|t| format!("/{}", t))
                .collect()
        } else {
            // Suggest based on recent commands
            let ctx = self.context.read().unwrap();
            ctx.recent_commands.iter()
                .filter(|c| c.contains(input))
                .take(5)
                .cloned()
                .collect()
        }
    }
}
```

### **2.2 Splash Command Discovery**
```bash
# Type "/" and get instant suggestions
$ kandil chat
🤖 /
/ask       Ask a question
/refactor  Refactor code
/test      Generate tests
/fix       Fix errors
/commit    Create commit
/review    Code review
...

# Tab-based navigation
🤖 /refactor <TAB>
auth/     database/  ui/
🤖 /refactor auth/<TAB>
handlers.rs  models.rs  routes.rs
```

---

## **3. Real-Time Command Execution Loop**

### **3.1 The Execution Pipeline**
```rust
// src/repl/loop.rs
pub async fn run_repl() -> Result<()> {
    let terminal = KandilTerminal::new()?;
    let context = Arc::new(RwLock::new(CommandContext::detect()));
    let prompt = KandilPrompt::new(context.clone());
    
    println!("{}", style("Kandil Code v1.0.0").bold().green());
    println!("💡 Type /help for commands, Ctrl+D to exit\n");
    
    loop {
        // Render dynamic prompt
        let prompt_str = prompt.render();
        
        // Read input with syntax highlighting
        let input = readline_with_highlighting(&prompt_str)?;
        
        // Exit conditions
        if input.trim().is_empty() {
            continue;
        }
        
        if input.trim() == "exit" || input.trim() == "quit" {
            break;
        }
        
        // Handle special inputs
        if let Some(result) = handle_special_input(&input, &terminal).await? {
            match result {
                SpecialInputResult::Continue => continue,
                SpecialInputResult::Exit => break,
            }
        }
        
        // Parse and route
        let command = parse_command(&input)?;
        
        match command {
            Command::Splash(cmd, args) => {
                execute_splash_command(cmd, args, &terminal).await?;
            }
            Command::NaturalLanguage(query) => {
                execute_natural_language(query, &terminal).await?;
            }
            Command::Shell(shell_cmd) => {
                terminal.execute(&shell_cmd, false).await?;
            }
        }
        
        // Update context
        context.write().unwrap().add_to_history(input);
    }
    
    println!("\n👋 Goodbye!");
    Ok(())
}

async fn handle_special_input(input: &str, terminal: &KandilTerminal) -> Result<Option<SpecialInputResult>> {
    match input.as_str() {
        "/help" => {
            show_help().await?;
            Ok(Some(SpecialInputResult::Continue))
        }
        "/clear" => {
            terminal.clear_screen()?;
            Ok(Some(SpecialInputResult::Continue))
        }
        "/reset" => {
            terminal.reset_context()?;
            println!("🔄 Context reset");
            Ok(Some(SpecialInputResult::Continue))
        }
        "/version" => {
            println!("Kandil Code v{}", env!("CARGO_PKG_VERSION"));
            Ok(Some(SpecialInputResult::Continue))
        }
        _ => Ok(None),
    }
}
```

---

## **4. Contextual Intelligence Engine**

### **4.1 Project State Awareness**
```rust
// src/context/project.rs
pub struct ProjectContext {
    /// Detected project type
    pub project_type: ProjectType,
    /// Git repository state
    pub git_state: GitState,
    /// Recently modified files
    pub recent_files: Vec<PathBuf>,
    /// Active editor file (if supported)
    pub active_file: Option<PathBuf>,
    /// Compilation errors (if any)
    pub errors: Vec<CompileError>,
    /// Test failures (if any)
    pub test_failures: Vec<TestFailure>,
}

#[derive(Debug)]
pub struct GitState {
    pub branch: String,
    pub ahead: usize,
    pub behind: usize,
    pub staged_files: Vec<PathBuf>,
    pub unstaged_files: Vec<PathBuf>,
    pub conflicts: Vec<PathBuf>,
    pub last_commit: Option<DateTime<Utc>>,
}

impl ProjectContext {
    pub fn detect() -> Result<Self> {
        Ok(Self {
            project_type: detect_project_type()?,
            git_state: detect_git_state()?,
            recent_files: detect_recent_files()?,
            active_file: detect_active_file()?,
            errors: detect_errors()?,
            test_failures: detect_test_failures()?,
        })
    }
    
    /// Generate contextual splash command suggestions
    pub fn suggest_splash_commands(&self) -> Vec<&'static SplashCommand> {
        let mut suggestions = vec![];
        
        if !self.errors.is_empty() {
            suggestions.push(SPLASH_COMMANDS.iter().find(|c| c.trigger == "/fix").unwrap());
        }
        
        if !self.test_failures.is_empty() {
            suggestions.push(SPLASH_COMMANDS.iter().find(|c| c.trigger == "/test").unwrap());
        }
        
        if self.git_state.staged_files.is_empty() {
            suggestions.push(SPLASH_COMMANDS.iter().find(|c| c.trigger == "/commit").unwrap());
        }
        
        suggestions.push(SPLASH_COMMANDS.iter().find(|c| c.trigger == "/ask").unwrap());
        suggestions.push(SPLASH_COMMANDS.iter().find(|c| c.trigger == "/review").unwrap());
        
        suggestions
    }
}
```

### **4.2 Dynamic Command Suggestions**
```rust
// src/splash/suggest.rs
pub fn generate_dynamic_help(ctx: &ProjectContext) -> String {
    let suggestions = ctx.suggest_splash_commands();
    
    let mut help = String::new();
    help.push_str("🤖 Context-Aware Commands:\n\n");
    
    for cmd in suggestions {
        help.push_str(&format!("  {}  {}\n", 
            style(cmd.trigger).cyan().bold(),
            style(cmd.description).dim()
        ));
    }
    
    // Add project-specific tips
    match ctx.project_type {
        ProjectType::Rust => {
            if !ctx.errors.is_empty() {
                help.push_str(&format!("\n⚠️  {} compilation errors detected. Try /fix\n", ctx.errors.len()));
            }
        }
        ProjectType::Python => {
            help.push_str("\n🐍 Tip: Use /doc to generate type stubs\n");
        }
        _ => {}
    }
    
    help
}
```

---

## **5. Internal Terminal Features**

### **5.1 Live Command Preview**
```bash
# Before executing, show what will happen
$ kandil chat
🤖 /refactor auth/login.rs

🔄 Preview:
  Command: sed -i 's/UserId/u32/g' src/auth/login.rs
  Command: cargo check --message-format=short
  Command: git diff --no-index src/auth/login.rs src/auth/login.rs.bak

⚠️  This will modify 1 file and run 2 commands.
   Approved? (y/n/diff) > diff

# Shows actual diff
@@ -10,7 +10,7 @@
-    user_id: UserId,
+    user_id: u32,

Approved? (y/n) > y
✅ Changes applied
✓ cargo check passed
```

### **5.2 Command Chaining & Pipelines**
```bash
# Native shell-style pipelines
🤖 /refactor error-handling | /test | /commit

# AI interprets the pipeline
# 1. Refactor error handling code
# 2. Generate tests for refactored code
# 3. Create commit with semantic message
```

### **5.3 Background Jobs**
```bash
# Run long tasks in background
🤖 /test --background
✓ Job #123 started in background
🤖 /jobs
  #123  Running  cargo test (60%)
  #124  Queued   cargo bench
🤖 /jobs wait 123
✓ Job #123 completed: All tests passed
```

---

## **6. Hardware-Adaptive UI**

### **6.1 Low-Latency Mode for Fast Machines**
```rust
// src/ui/adaptive.rs
pub struct AdaptiveUI {
    hardware: HardwareProfile,
    latency_target: Duration,
}

impl AdaptiveUI {
    /// Adjust UI richness based on hardware
    pub fn should_rich_render(&self) -> bool {
        // Disable fancy UI on slow hardware
        self.hardware.cpu_physical_cores >= 4 && self.hardware.total_ram_gb >= 8
    }
    
    /// Progressive loading for TUI
    pub async fn progressive_render(&self, content: Vec<String>) {
        if self.should_rich_render() {
            // Full ratatui rendering
            render_full_tui(content).await;
        } else {
            // Simple line-by-line output
            for line in content {
                println!("{}", line);
                sleep(Duration::from_millis(10)); // Prevent overwhelming terminal
            }
        }
    }
    
    /// Adjust token streaming speed
    pub fn get_token_delay(&self) -> Duration {
        match self.hardware.cpu_physical_cores {
            0..=2 => Duration::from_millis(50),  // Slow machines: visible typing
            3..=4 => Duration::from_millis(20),  // Medium: smooth
            _ => Duration::from_millis(5),       // Fast: near-instant
        }
    }
}
```

---

## **7. Accessibility-First Design**

### **7.1 Screen-Reader Optimized Output**
```bash
# Screen reader announces structure
$ kandil ask "How do I fix this error?"
🤖 Starting analysis...
[region="status"] Analyzing compilation errors...
[region="code"] Error E0308: mismatched types
[region="suggestion"] Fix: Change line 42 from String to &str
```

### **7.2 Keyboard Navigation**
```
# Global shortcuts (works in any mode)
Ctrl+A → Switch to ask mode
Ctrl+R → Switch to refactor mode
Ctrl+T → Run tests
Ctrl+C → Clear context (not cancel)
Ctrl+L → Clear screen
Ctrl+D → Exit
Ctrl+/ → Show all splash commands
```

---

## **8. Success Metrics vs Competitors**

| Feature | Claude Code | Qwen Code | Gemini CLI | **Kandil Code** |
|---------|-------------|-----------|------------|-----------------|
| **Internal Terminal** | ✅ Limited | ✅ Bash only | ❌ No | **✅ Full PTY with isolation** |
| **Splash Commands** | ✅ 12 commands | ✅ 8 commands | ❌ Natural only | **✅ 20+ commands + custom** |
| **Hardware Adaptation** | ❌ No | ❌ No | ❌ No | **✅ Auto-select models & UI** |
| **Local Models** | ❌ Cloud-only | ✅ Limited | ❌ Cloud-only | **✅ Primary, cloud fallback** |
| **Screen Reader** | ❌ No | ❌ No | ❌ No | **✅ WCAG 2.1 AA** |
| **Multi-Modal** | ✅ Vision | ❌ No | ✅ Vision | **✅ Vision + LoRA adapters** |
| **Command Pipeline** | ❌ No | ❌ No | ❌ No | **✅ | operator** |
| **Context Switching** | ⚠️ Slow | ⚠️ Manual | ⚠️ Manual | **✅ Auto-detect project type** |

---

## **9. Quick Start Examples**

### **9.1 The 30-Second Onboarding**
```bash
$ curl -fsSL https://kandil.dev/install.sh | sh
✓ Kandil Code installed

$ kandil
🤖 Welcome to Kandil Code! Let's set up in 3 steps:

Step 1: Choose your experience level
  1. Beginner (I'll guide you)
  2. Intermediate (I suggest, you decide)
  3. Expert (I stay out of the way)
> 2

Step 2: Detecting hardware...
  ✓ 16GB RAM, 8 cores, RTX 3060 (12GB)
  📦 Installing qwen2.5-coder-7b-q4 (4.5GB)...
  ✓ Ready in 45s

Step 3: Test drive
  🤖 Try: /ask How do I read a file in Rust?
  🤖 Try: Open src/main.rs and type /refactor
  🤖 Try: /test to generate tests

$ /ask How do I read a file in Rust?
🤖 Use std::fs::read_to_string:

```rust
let content = std::fs::read_to_string("file.txt")?;
```

[Copied to clipboard] ✓
```

### **9.2 Daily Workflow**
```bash
# Morning: Check project status
$ kandil
🤖 Good morning! 3 compilation errors in src/auth.rs
🤖 Suggested: /fix

/refactor auth/login.rs
🔄 Preview: 2 changes in login.rs
✅ Applied
✓ cargo check passed

/test --background
✓ Job #1 started

/commit
✍️ Generated commit: "refactor(auth): simplify login flow"
✅ Committed

# Afternoon: Code review request
/review --ai=claude --focus=security
🔍 Reviewing 3 files...
⚠️  Potential SQL injection in src/db.rs
💡 Suggested fix: Use parameterized queries
[View diff] [Apply] [Skip]
```

---

## **10. Implementation Roadmap**

### **Phase 1: Core Terminal & Splash (Week 1-2)**
- [ ] Implement `KandilTerminal` with PTY isolation
- [ ] Create `SplashCommand` registry
- [ ] Build REPL loop with context detection
- [ ] Add splash command auto-completion

### **Phase 2: Contextual Intelligence (Week 3)**
- [ ] Implement `ProjectContext` detection engine
- [ ] Add git state monitoring
- [ ] Create dynamic command suggestions
- [ ] Build error detection and /fix handler

### **Phase 3: Interactive UI (Week 4)**
- [ ] Add `SmartPrompt` for confirmations
- [ ] Build `RichProgress` with live updates
- [ ] Implement diff preview system
- [ ] Create pipeline (`|`) support

### **Phase 4: Accessibility (Week 5)**
- [ ] Screen reader annotations
- [ ] 8 colorblind-safe themes
- [ ] Keyboard navigation shortcuts
- [ ] High contrast mode

### **Phase 5: Hardware Adaptation (Week 6)**
- [ ] `AdaptiveUI` for low-end machines
- [ ] Auto-switch between TUI/simple mode
- [ ] Model performance monitoring
- [ ] Dynamic quality adjustment

### **Phase 6: Polish & Integration (Week 7)**
- [ ] Tutorial system
- [ ] Profile switching (`kandil profile`)
- [ ] CI/CD batch mode
- [ ] GitHub Actions integration

---

## **11. The Kandil Advantage**

**Why this beats the competition:**

1. **Internal PTY**: Unlike Claude's external shell calls, Kandil's isolated terminal prevents state corruption and provides real-time output streaming.

2. **Hardware as a Feature**: While others are cloud-only or treat local as an afterthought, Kandil **optimizes the entire UX** around your hardware—models, UI richness, and latency all adapt.

3. **Universal Accessibility**: From Chromebooks (1.5B models) to Threadripper workstations (70B models), from screen readers to 4K displays, **Kandil Just Works™**.

4. **Pipeline Power**: The `|` operator is a game-changer—`/refactor | /test | /commit` executes entire workflows with AI oversight at each step.

5. **Splash Intelligence**: Commands aren't static—`/fix` knows about your current errors, `/test` knows your framework, `/commit` knows your git state.

**The result**: A CLI that feels like a **senior developer** is pair-programming with you—one who types at 300 WPM, never gets tired, and adapts to your skill level.