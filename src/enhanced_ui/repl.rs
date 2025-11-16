use crate::{
    enhanced_ui::{
        adaptive::{AdaptiveUI, AccessibilityMode},
        ide_sync::IdeSyncBridge,
        input::{InputMethod, UniversalInput},
        persona::PersonaProfile,
        predictive::{PredictiveExecutor, GhostText},
        smart_prompt::{SmartPrompt, PipelineStage},
        splash::{self, CommandContext, SplashResult},
        terminal::KandilTerminal,
        thought::{ThoughtFragment, ThoughtStreamer, OutputMode},
    },
    mobile::MobileBridge,
};
use anyhow::{anyhow, Result};
use std::{collections::VecDeque, env, sync::Arc, time::Duration, cmp};
use futures_util;

#[derive(Default)]
pub struct KandilPrompt {
    mode: PromptMode,
}

impl KandilPrompt {
    fn render(&self) -> String {
        match self.mode {
            PromptMode::Chat => "🤖 ".to_string(),
            PromptMode::Shell => "❯ ".to_string(),
        }
    }

    fn set_mode(&mut self, mode: PromptMode) {
        self.mode = mode;
    }
}

#[derive(Default, Copy, Clone)]
pub enum PromptMode {
    #[default]
    Chat,
    Shell,
}

pub async fn run_repl() -> Result<()> {
    let terminal = Arc::new(KandilTerminal::new()?);
    let mut context = CommandContext::new(terminal.clone());
    let mut prompt = KandilPrompt::default();
    let mut universal_input = UniversalInput::new()?;
    let adaptive_ui = AdaptiveUI::from_system()
        .with_accessibility_mode(AdaptiveUI::detect_accessibility_mode());
    let ide_sync = IdeSyncBridge::new(env::current_dir()?);
    ide_sync.launch(adaptive_ui.clone())?;
    let mobile_bridge = MobileBridge::new()?;
    let mut predictive_executor = PredictiveExecutor::new();
    let thought_streamer = ThoughtStreamer::with_output_mode(OutputMode::Streaming);
    let mut persona_profile = PersonaProfile::from_history(&context.recent_commands);

    println!("Kandil Shell initialized. Type /help for splash commands.");

    // Display UI capabilities based on hardware and accessibility settings
    println!("UI Capabilities: {}", adaptive_ui.capabilities_description());
    if adaptive_ui.should_enhance_accessibility() {
        println!("Accessibility features enabled: {:?}", adaptive_ui.accessibility_mode());
    }

    loop {
        // Enhance context detection before input processing
        context.refresh_project_context();
        context.refresh_file_context().await;
        context.refresh_git_status().await;

        let input = if let Some(remote) = mobile_bridge.try_voice_command()? {
            adaptive_ui.announce("status", "📱 Remote command received");
            remote
        } else {
            match universal_input.read(&prompt.render())? {
                InputMethod::Text(text) => text,
                InputMethod::Voice(transcript) => {
                    adaptive_ui.announce("status", "🎙️ Voice input detected");
                    transcript
                }
                InputMethod::Image(description) => {
                    adaptive_ui.announce("status", "🖼️ Image input routed to /ask");
                    format!("/ask {}", description)
                }
            }
        };

        let trimmed = input.trim();
        if trimmed.is_empty() {
            continue;
        }

        if trimmed == "exit" || trimmed == "quit" {
            break;
        }

        universal_input.add_history(trimmed)?;

        if handle_special_input(trimmed, &terminal, &mut context, Some(&thought_streamer)).await? {
            continue;
        }

        thought_streamer.emit(ThoughtFragment::Context(format!("Input `{}`", trimmed)));

        // Use enhanced prefetching with async capabilities
        if predictive_executor.should_prefetch() {
            predictive_executor.prefetch(trimmed);
            predictive_executor.mark_prefetch_time();

            // In a real implementation, we might also call prefetch_async here
            // tokio::spawn(async move {
            //     let _ = predictive_executor.prefetch_async(trimmed).await;
            // });
        }

        // Enhanced context-aware command parsing
        let parsed = parse_command_enhanced(trimmed, &context).await;
        if let Err(err) = execute_command(
            parsed,
            &terminal,
            &mut context,
            &mut prompt,
            &adaptive_ui,
            &thought_streamer,
        )
        .await
        {
            eprintln!("Command error: {}", err);
        }

        context.remember_command(trimmed);
        context.refresh_project_context();
        context.refresh_file_context().await; // Refresh file context after execution
        context
            .job_tracker
            .auto_complete_elapsed(Duration::from_secs(45));
        let job_snapshot = context.job_tracker.snapshot();
        predictive_executor.observe(trimmed);
        show_contextual_hint(&context, &adaptive_ui);
        if let Some(hint) = predictive_executor.predict_hint() {
            println!("🔮 Prediction: {}", hint);
        }

        // Display ghost text information if available
        if let Some(ghost) = predictive_executor.get_ghost_text() {
            if ghost.confidence > 0.5 {
                println!("👻 Ghost text suggestion: {} (confidence: {:.1})", ghost.text, ghost.confidence);
            }
        }
        mobile_bridge.sync_jobs(&job_snapshot);
        let updated_profile = PersonaProfile::from_history(&context.recent_commands);
        if updated_profile.persona != persona_profile.persona {
            adaptive_ui.announce("persona", updated_profile.greeting);
            persona_profile = updated_profile;
        }
    }

    println!("👋 Goodbye!");
    Ok(())
}

async fn handle_special_input(
    input: &str,
    terminal: &Arc<KandilTerminal>,
    context: &mut CommandContext,
    thought_streamer: Option<&ThoughtStreamer>,
) -> Result<bool> {
    match input {
        "/help" => {
            print_help();
            Ok(true)
        }
        "/clear" => {
            terminal.clear_screen()?;
            Ok(true)
        }
        "/reset" => {
            terminal.reset_context().await?;
            context.job_tracker.complete_all();
            println!("🔄 Context reset");
            Ok(true)
        }
        "/thoughts" => {
            if let Some(thinker) = thought_streamer {
                println!("💡 Recent thoughts:");
                let recent = thinker.get_recent_thoughts(5);
                for thought in recent {
                    match thought.fragment {
                        ThoughtFragment::Action(msg) => println!("  ⚙️  Action: {}", msg),
                        ThoughtFragment::Result(msg) => println!("  ✅ Result: {}", msg),
                        ThoughtFragment::Insight(msg) => println!("  💡 Insight: {}", msg),
                        ThoughtFragment::Hypothesis(msg) => println!("  🧠 Hypothesis: {}", msg),
                        ThoughtFragment::Context(msg) => println!("  📚 Context: {}", msg),
                        ThoughtFragment::Process(msg) => println!("  🔄 Process: {}", msg),
                        ThoughtFragment::Question(msg) => println!("  ❓ Question: {}", msg),
                    }
                }

                if recent.is_empty() {
                    println!("  No recent thoughts to display");
                }
            } else {
                println!("  No thought streamer available");
            }
            Ok(true)
        }
        "exit" | "quit" => Ok(false),
        _ => Ok(false),
    }
}

fn parse_command(input: &str) -> Command {
    if input.contains('|') {
        let stages = input
            .split('|')
            .map(|segment| parse_single_command(segment.trim()))
            .collect();
        Command::Pipeline(stages)
    } else {
        parse_single_command(input)
    }
}

async fn parse_command_enhanced(input: &str, context: &CommandContext) -> Command {
    // First try the normal parsing
    let basic_command = parse_single_command(input);

    // Apply context-aware enhancements
    match &basic_command {
        Command::Splash { trigger, args } => {
            // Enhance splash command with context
            let enhanced_args = enhance_args_with_context(trigger, args, context).await;
            Command::Splash {
                trigger: trigger.clone(),
                args: enhanced_args
            }
        },
        Command::Shell(cmd) => {
            // Potentially enhance shell command with context
            Command::Shell(cmd.clone())
        },
        Command::NaturalLanguage(query) => {
            // Potentially enhance natural language with context
            Command::NaturalLanguage(query.clone())
        },
        Command::Pipeline(commands) => {
            // Enhance pipeline commands recursively
            let enhanced_commands: Vec<Command> = futures_util::future::join_all(
                commands.iter().map(|cmd| enhance_command_with_context(cmd, context))
            ).await;
            Command::Pipeline(enhanced_commands)
        }
    }
}

fn parse_single_command(input: &str) -> Command {
    if input.contains('|') {
        let stages = input
            .split('|')
            .map(|segment| parse_single_command(segment.trim()))
            .collect();
        Command::Pipeline(stages)
    } else if input.starts_with('/') {
        let mut parts = input.split_whitespace();
        let trigger = parts.next().unwrap_or("").to_string();
        let args = parts.map(|p| p.to_string()).collect();
        Command::Splash { trigger, args }
    } else if looks_like_natural_language(input) {
        Command::NaturalLanguage(input.to_string())
    } else {
        Command::Shell(input.to_string())
    }
}

async fn enhance_args_with_context(trigger: &str, args: &[String], context: &CommandContext) -> Vec<String> {
    // Add context-aware enhancements to arguments
    let mut enhanced_args = args.to_vec();

    // For /test command, automatically target the active file if no target is specified
    if trigger == "/test" && enhanced_args.is_empty() {
        if let Some(active_file) = &context.active_file {
            enhanced_args.push(active_file.to_string_lossy().to_string());
        }
    }

    // For /fix command, add context about current errors if any
    if trigger == "/fix" && context.project_context.errors > 0 {
        // In a real implementation, this could add specific file targets based on detected errors
    }

    // For /review command, automatically target the active file if no target is specified
    if trigger == "/review" && enhanced_args.is_empty() {
        if let Some(active_file) = &context.active_file {
            enhanced_args.push(active_file.to_string_lossy().to_string());
        }
    }

    enhanced_args
}

async fn enhance_command_with_context(command: &Command, context: &CommandContext) -> Command {
    match command {
        Command::Splash { trigger, args } => {
            let enhanced_args = enhance_args_with_context(trigger, args, context).await;
            Command::Splash {
                trigger: trigger.clone(),
                args: enhanced_args
            }
        },
        _ => command.clone()
    }
}

fn looks_like_natural_language(input: &str) -> bool {
    input.ends_with('?') || input.split_whitespace().count() > 7
}

fn emit_result(result: SplashResult, adaptive_ui: &AdaptiveUI) {
    if let Some(message) = result.message {
        adaptive_ui.announce("status", &message);
    }
}

fn show_contextual_hint(ctx: &CommandContext, adaptive_ui: &AdaptiveUI) {
    if ctx.recent_commands.is_empty() {
        return;
    }
    if !adaptive_ui.should_rich_render() {
        return;
    }
    let latest = ctx.recent_commands.back().unwrap();
    if latest.starts_with('/') {
        println!("Hint: try chaining {} with shell commands.", latest);
    } else {
        let suggestions = ctx.contextual_suggestions();
        if !suggestions.is_empty() {
            let labels: Vec<&str> = suggestions.into_iter().collect();
            println!("💡 Try splash commands: {}", labels.join(", "));
        }
    }
}

enum Command {
    Splash { trigger: String, args: Vec<String> },
    Shell(String),
    NaturalLanguage(String),
    Pipeline(Vec<Command>),
}

async fn execute_command(
    command: Command,
    terminal: &Arc<KandilTerminal>,
    context: &mut CommandContext,
    prompt: &mut KandilPrompt,
    adaptive_ui: &AdaptiveUI,
    thought_streamer: &ThoughtStreamer,
) -> Result<()> {
    match command {
        Command::Pipeline(commands) => {
            // Create detailed pipeline stages for better visualization
            let mut stages = Vec::new();
            for (i, cmd) in commands.iter().enumerate() {
                let stage = PipelineStage::new(
                    &format!("Stage {}", i + 1),
                    &command_label(cmd),
                )
                .with_description(match cmd {
                    Command::Splash { trigger, .. } => Some(format!("Splash command: {}", trigger)),
                    Command::Shell(cmd_str) => Some(format!("Shell command: {}", cmd_str)),
                    Command::NaturalLanguage(_) => Some("Natural language query".to_string()),
                    Command::Pipeline(_) => Some("Nested pipeline".to_string()),
                }.unwrap_or_else(|| "Unknown command".to_string()))
                .with_duration(Duration::from_secs((i + 1) as u64 * 2)); // Estimate duration based on stage number

                stages.push(stage);
            }

            println!("{}", SmartPrompt::pipeline_summary_detailed(&stages));

            for cmd in flatten_pipeline(commands) {
                handle_basic_command(
                    cmd,
                    terminal,
                    context,
                    prompt,
                    adaptive_ui,
                    thought_streamer,
                )
                .await?;
            }
            Ok(())
        }
        other => {
            handle_basic_command(
                other,
                terminal,
                context,
                prompt,
                adaptive_ui,
                thought_streamer,
            )
            .await
        }
    }
}

fn command_label(command: &Command) -> String {
    match command {
        Command::Splash { trigger, .. } => trigger.clone(),
        Command::Shell(cmd) => cmd.clone(),
        Command::NaturalLanguage(_) => "chat".to_string(),
        Command::Pipeline(stages) => format!(
            "pipeline({})",
            stages
                .iter()
                .map(|stage| match stage {
                    Command::Splash { trigger, .. } => trigger.clone(),
                    Command::Shell(cmd) => cmd.clone(),
                    Command::NaturalLanguage(_) => "chat".to_string(),
                    Command::Pipeline(_) => "pipeline".to_string(),
                })
                .collect::<Vec<_>>()
                .join(", ")
        ),
    }
}

async fn handle_basic_command(
    command: Command,
    terminal: &Arc<KandilTerminal>,
    context: &mut CommandContext,
    prompt: &mut KandilPrompt,
    adaptive_ui: &AdaptiveUI,
    thought_streamer: &ThoughtStreamer,
) -> Result<()> {
    match command {
        Command::Splash { trigger, args } => {
            prompt.set_mode(PromptMode::Chat);
            thought_streamer.emit(ThoughtFragment::Hypothesis(format!(
                "Executing splash {}",
                trigger
            )));
            let result = splash::execute_splash_command(&trigger, &args, context).await?;
            emit_result(result, adaptive_ui);
            thought_streamer.emit(ThoughtFragment::Result(format!("Completed {}", trigger)));
            Ok(())
        }
        Command::Shell(cmd) => {
            prompt.set_mode(PromptMode::Shell);
            thought_streamer.emit(ThoughtFragment::Action(format!("Running {}", cmd)));
            let result = terminal.execute(&cmd, false).await?;
            if !result.stdout.is_empty() {
                print!("{}", result.stdout);
            }
            if let Some(analysis) = result.ai_analysis {
                println!("\n{}", analysis);
            }
            thought_streamer.emit(ThoughtFragment::Result(format!("Command {} finished", cmd)));
            Ok(())
        }
        Command::NaturalLanguage(query) => {
            prompt.set_mode(PromptMode::Chat);
            emit_result(
                SplashResult {
                    message: Some(format!("💬 {}", query)),
                },
                adaptive_ui,
            );
            thought_streamer.emit(ThoughtFragment::Result("Answered chat query".into()));
            Ok(())
        }
        Command::Pipeline(_) => {
            unreachable!("Nested pipelines should be flattened before execution");
        }
    }
}

fn flatten_pipeline(commands: Vec<Command>) -> Vec<Command> {
    let mut flat = Vec::new();
    let mut queue: VecDeque<Command> = commands.into();
    while let Some(cmd) = queue.pop_front() {
        match cmd {
            Command::Pipeline(inner) => {
                for stage in inner.into_iter().rev() {
                    queue.push_front(stage);
                }
            }
            other => flat.push(other),
        }
    }
    flat
}

fn print_help() {
    println!("Available splash commands:");
    for cmd in splash::SPLASH_COMMANDS.iter() {
        println!("  {:<10} {}", cmd.trigger, cmd.description);
    }
    println!("\nSpecial commands:");
    println!("  {:<10} {}", "/help", "Show this help message");
    println!("  {:<10} {}", "/clear", "Clear the terminal screen");
    println!("  {:<10} {}", "/reset", "Reset the command context");
    println!("  {:<10} {}", "/thoughts", "Display recent thoughts from AI reasoning");
    println!("Use standard shell commands without '/' prefix.");
}

async fn handle_splash(input: &str, ctx: &mut CommandContext) -> Result<SplashResult> {
    let mut parts = input.split_whitespace();
    let trigger = parts.next().unwrap_or("");
    let args: Vec<String> = parts.map(|p| p.to_string()).collect();
    splash::execute_splash_command(trigger, &args, ctx).await
}
