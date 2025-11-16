use crate::enhanced_ui::{
    adaptive::AdaptiveUI,
    predictive::PredictiveExecutor,
    smart_prompt::SmartPrompt,
    splash::{self, CommandContext, SplashResult},
    terminal::KandilTerminal,
    thought::{ThoughtFragment, ThoughtStreamer},
};
use anyhow::{anyhow, Result};
use rustyline::{error::ReadlineError, DefaultEditor};
use std::collections::VecDeque;
use std::sync::Arc;

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
    let mut editor = DefaultEditor::new()?;
    let adaptive_ui = AdaptiveUI::from_system();
    let mut predictive_executor = PredictiveExecutor::new();
    let thought_streamer = ThoughtStreamer::new();

    println!("Kandil Shell initialized. Type /help for splash commands.");

    loop {
        let input = match editor.readline(&prompt.render()) {
            Ok(line) => line,
            Err(ReadlineError::Interrupted) => continue,
            Err(ReadlineError::Eof) => break,
            Err(err) => return Err(anyhow!("Readline error: {}", err)),
        };

        let trimmed = input.trim();
        if trimmed.is_empty() {
            continue;
        }

        if trimmed == "exit" || trimmed == "quit" {
            break;
        }

        editor.add_history_entry(trimmed)?;

        if handle_special_input(trimmed, &terminal, &mut context).await? {
            continue;
        }

        thought_streamer.emit(ThoughtFragment::Context(format!("Input `{}`", trimmed)));
        predictive_executor.prefetch(trimmed);
        let parsed = parse_command(trimmed);
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
        predictive_executor.observe(trimmed);
        show_contextual_hint(&context, &adaptive_ui);
        if let Some(hint) = predictive_executor.predict_hint() {
            println!("🔮 Prediction: {}", hint);
        }
    }

    println!("👋 Goodbye!");
    Ok(())
}

async fn handle_special_input(
    input: &str,
    terminal: &Arc<KandilTerminal>,
    context: &mut CommandContext,
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

fn parse_single_command(input: &str) -> Command {
    if input.starts_with('/') {
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
            let labels = commands.iter().map(command_label).collect::<Vec<_>>();
            println!("{}", SmartPrompt::pipeline_summary(&labels));
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
    println!("Use standard shell commands without '/' prefix.");
}

async fn handle_splash(input: &str, ctx: &mut CommandContext) -> Result<SplashResult> {
    let mut parts = input.split_whitespace();
    let trigger = parts.next().unwrap_or("");
    let args: Vec<String> = parts.map(|p| p.to_string()).collect();
    splash::execute_splash_command(trigger, &args, ctx).await
}
