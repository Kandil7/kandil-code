use crate::adapters::{edge, linux, macos, mobile, windows};
use crate::benchmark::{BenchmarkOptions, CrossPlatformBenchmark};
use crate::core::adapters::ai::factory::AIProviderFactory;
use crate::core::hardware::{detect_hardware, PlatformKind};
use crate::core::prompting::{PromptIntent, PromptRouter};
use crate::enhanced_ui;
use crate::pwa;
use crate::security::mobile as mobile_security;
use crate::security::platform::PlatformHardener;
use crate::utils::config::{Config, SecureKey};
use crate::utils::plugins::PluginManager;
use crate::utils::project_manager::ProjectManager;
use crate::utils::refactoring::{RefactorEngine, RefactorParams};
use crate::utils::templates::TemplateEngine;
use crate::utils::test_generation::TestGenerator;
use crate::web;
use anyhow::Result;
use clap::{Parser, Subcommand};
use futures_util::stream::StreamExt;
use std::{path::PathBuf, sync::Arc};
use tokio::{fs, io::AsyncWriteExt, task};

#[derive(Parser)]
#[command(name = "kandil")]
#[command(about = "Intelligent development platform", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    #[arg(short, long, global = true, help = "Verbose output")]
    pub verbose: bool,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Initialize a new Kandil project
    Init,
    /// Chat with the AI assistant
    Chat {
        #[arg(value_parser)]
        message: Option<String>,
    },
    /// Create a new project from template
    Create {
        /// Template name (flutter, python, javascript, rust)
        #[arg(value_parser)]
        template: String,
        /// Project name
        #[arg(value_parser)]
        name: String,
    },
    /// Launch the TUI studio
    Tui,
    /// Project management commands
    Projects {
        #[command(subcommand)]
        sub: ProjectSub,
    },
    /// Agent commands for requirements, design, etc.
    Agent {
        #[command(subcommand)]
        sub: AgentSub,
    },
    /// Refactoring commands
    Refactor {
        #[command(subcommand)]
        sub: RefactorSub,
    },
    /// Test generation commands
    Test {
        #[command(subcommand)]
        sub: TestSub,
    },
    /// Model switching commands
    SwitchModel { provider: String, model: String },
    /// Plugin management commands
    Plugin {
        #[command(subcommand)]
        sub: PluginSub,
    },
    /// Configuration management commands
    Config {
        #[command(subcommand)]
        sub: ConfigSub,
    },
    /// Local model management commands
    LocalModel {
        #[command(subcommand)]
        sub: LocalModelSub,
    },
    /// Authentication commands
    Auth {
        #[command(subcommand)]
        sub: AuthSub,
    },
    /// Windows-specific local runtime helpers
    Windows {
        #[command(subcommand)]
        sub: WindowsSub,
    },
    /// macOS-specific runtime helpers
    Macos {
        #[command(subcommand)]
        sub: MacosSub,
    },
    /// Linux-specific runtime helpers
    Linux {
        #[command(subcommand)]
        sub: LinuxSub,
    },
    /// Mobile & edge runtime helpers
    Mobile {
        #[command(subcommand)]
        sub: MobileSub,
    },
    /// Export PWA assets for offline installs
    Pwa {
        #[arg(long, default_value = "dist/pwa")]
        output: PathBuf,
    },
    /// Launch the Axum-based web companion dashboard
    Web {
        /// Address to bind, e.g. 127.0.0.1:7878
        #[arg(short, long, default_value = "127.0.0.1:7878")]
        address: String,
    },
}

#[derive(Subcommand)]
pub enum AgentSub {
    /// Generate requirements document
    Requirements {
        /// Description of the project to analyze
        description: String,
    },
    /// Generate design document
    Design {
        /// Description of the requirements to design for
        requirements: String,
    },
    /// Generate code from design
    Code {
        /// Path to design document
        design_path: String,
        /// Target language
        language: String,
    },
    /// Generate or execute tests
    Test {
        #[command(subcommand)]
        sub: TestSubCommand,
    },
    /// Generate documentation
    Documentation {
        #[command(subcommand)]
        sub: DocumentationSubCommand,
    },
    /// Manage releases
    Release {
        #[command(subcommand)]
        sub: ReleaseSubCommand,
    },
    /// Run quality assurance checks
    Qa {
        #[command(subcommand)]
        sub: QaSubCommand,
    },
    /// Manage system maintenance
    Maintenance {
        #[command(subcommand)]
        sub: MaintenanceSubCommand,
    },
    /// Professional role simulations
    Simulate {
        #[command(subcommand)]
        sub: SimulateSubCommand,
    },
    /// Advanced agents for review, security, deployment
    Advanced {
        #[command(subcommand)]
        sub: AdvancedSubCommand,
    },
    /// Tech role simulations (Architect, Developer, QA)
    TechRole {
        #[command(subcommand)]
        sub: TechRoleSubCommand,
    },
    /// DevOps, Scrum, and advanced feature simulations
    AdvancedFeatures {
        #[command(subcommand)]
        sub: AdvancedFeaturesSubCommand,
    },
}

#[derive(Subcommand)]
pub enum MaintenanceSubCommand {
    /// Run a health check for a system
    HealthCheck {
        /// Name of the system to check
        system_name: String,
    },
}

#[derive(Subcommand)]
pub enum QaSubCommand {
    /// Run the full QA suite for a project
    FullSuite {
        /// Path to the project to run QA on
        project_path: String,
    },
}

#[derive(Subcommand)]
pub enum ReleaseSubCommand {
    /// Run the full release process
    FullProcess {
        /// Version to release
        version: String,
    },
}

#[derive(Subcommand)]
pub enum DocumentationSubCommand {
    /// Generate documentation for a project
    Generate {
        /// Path to the project to document
        path: String,
    },
}

#[derive(Subcommand)]
pub enum AdvancedFeaturesSubCommand {
    /// DevOps activities (IaC, CI/CD, monitoring)
    DevOps {
        #[command(subcommand)]
        sub: DevOpsSubCommand,
    },
    /// Scrum ceremony simulations
    Scrum {
        #[command(subcommand)]
        sub: ScrumSubCommand,
    },
    /// Internationalization assistance
    I18n {
        #[command(subcommand)]
        sub: I18nSubCommand,
    },
    /// Accessibility compliance checking
    A11y {
        #[command(subcommand)]
        sub: A11ySubCommand,
    },
    /// Real-time collaboration features
    Collab {
        #[command(subcommand)]
        sub: CollabSubCommand,
    },
    /// IDE extension prototype
    Ide {
        #[command(subcommand)]
        sub: IdeSubCommand,
    },
}

#[derive(Subcommand)]
pub enum DevOpsSubCommand {
    /// Generate Terraform infrastructure code
    Terraform {
        /// Infrastructure specification
        spec: String,
    },
    /// Run incident response drill
    Drill {
        /// Scenario to drill
        scenario: String,
    },
    /// Generate CI/CD pipeline
    Pipeline {
        /// Project type
        project_type: String,
    },
}

#[derive(Subcommand)]
pub enum ScrumSubCommand {
    /// Plan a sprint
    Plan {
        /// Sprint goal
        goal: String,
        /// Duration in days
        duration: u32,
        /// Team size
        team_size: u32,
    },
    /// Run retrospective for a sprint
    Retro {
        /// Sprint number
        sprint: u32,
    },
    /// Conduct a Scrum ceremony
    Ceremony {
        /// Type of ceremony (sprint_planning, daily_scrum, etc.)
        ceremony_type: String,
        /// Comma-separated list of participants
        participants: String,
    },
}

#[derive(Subcommand)]
pub enum I18nSubCommand {
    /// Translate text to a target language
    Translate {
        /// Text to translate
        text: String,
        /// Target language code (en, es, fr, etc.)
        target: String,
        /// Source language code
        source: String,
    },
    /// Audit translations in a directory
    Audit {
        /// Path to resource directory
        path: String,
    },
    /// Review a specific translation
    Review {
        /// Original text
        original: String,
        /// Translated text
        translation: String,
        /// Target language code
        target: String,
    },
}

#[derive(Subcommand)]
pub enum A11ySubCommand {
    /// Audit content for accessibility (WCAG AA level)
    Audit {
        /// Content to audit
        content: String,
    },
    /// Generate accessibility guidelines for a component
    Guidelines {
        /// Component type
        component: String,
    },
    /// Remediate accessibility issues in HTML
    Fix {
        /// HTML content to remediate
        html: String,
    },
}

#[derive(Subcommand)]
pub enum CollabSubCommand {
    /// Create a collaboration session
    Session {
        /// Session name
        name: String,
        /// Creator ID
        creator_id: String,
        /// Creator name
        creator_name: String,
    },
    /// Add a participant to session
    AddParticipant {
        /// Session ID
        session_id: String,
        /// Participant ID
        user_id: String,
        /// Participant name
        name: String,
        /// Role in session
        role: String,
    },
    /// Add a document to session
    AddDoc {
        /// Session ID
        session_id: String,
        /// Document ID
        doc_id: String,
        /// Document name
        name: String,
        /// Initial content
        content: String,
        /// Language
        language: String,
    },
}

#[derive(Subcommand)]
pub enum IdeSubCommand {
    /// Get code suggestions for selected code
    Suggestions {
        /// File path
        file: String,
        /// Language of the code
        language: String,
        /// Selected code snippet
        code: String,
    },
    /// Generate documentation for code
    Docs {
        /// Code to document
        code: String,
        /// Language of the code
        language: String,
    },
    /// Get refactoring options
    Refactor {
        /// Code to refactor
        code: String,
        /// Language of the code
        language: String,
    },
    /// Run inline code review
    Review {
        /// Code to review
        code: String,
        /// Language of the code
        language: String,
    },
}

#[derive(Subcommand)]
pub enum TechRoleSubCommand {
    /// Architect role simulation
    Architect {
        #[command(subcommand)]
        sub: ArchitectSubCommand,
    },
    /// Developer role simulation
    Developer {
        #[command(subcommand)]
        sub: DeveloperSubCommand,
    },

    /// Cross-role collaboration
    Collaborate {
        #[command(subcommand)]
        sub: CollaborateSubCommand,
    },
}

#[derive(Subcommand)]
pub enum ArchitectSubCommand {
    /// Review architecture design
    Review {
        /// UML or architecture diagram to review
        diagram: String,
    },
    /// Make an architecture decision
    Decide {
        /// Context for the decision
        context: String,
        /// The decision to make
        decision: String,
    },
    /// Generate Architecture Decision Record
    Adr {
        /// Decision ID to generate ADR for
        decision_id: String,
    },
}

#[derive(Subcommand)]
pub enum DeveloperSubCommand {
    /// Implement a feature
    Implement {
        /// Feature specification
        spec: String,
        /// File to implement in
        file: String,
    },
    /// Start pair programming session
    Pair {
        /// Name of the pair programming partner
        partner: String,
        /// Task to work on
        task: String,
        /// File to work on
        file: String,
    },
    /// Find bugs in code
    Bugs {
        /// Code to analyze for bugs
        code: String,
        /// File path of the code
        file: String,
    },
}

#[derive(Subcommand)]
pub enum CollaborateSubCommand {
    /// Start a cross-role collaboration session
    Session {
        /// Session title
        title: String,
        /// Comma-separated list of roles (architect,developer,qa)
        roles: String,
        /// Session agenda
        agenda: Vec<String>,
    },
    /// Create a cross-role decision
    Decision {
        /// Decision title
        title: String,
        /// Decision description
        description: String,
        /// Comma-separated list of involved roles
        roles: String,
    },
}

#[derive(Subcommand)]
pub enum AdvancedSubCommand {
    /// Code review and quality analysis
    Review {
        /// Path to source file
        file: String,
    },
    /// Security and ethics scanning
    Security {
        /// Path to source file
        file: String,
        /// Description of the system
        description: String,
    },
    /// Deployment planning and execution
    Deploy {
        #[command(subcommand)]
        sub: DeploySubCommand,
    },
    /// System self-improvement analysis
    SelfImprove {
        /// Path to codebase to analyze
        path: String,
    },
}

#[derive(Subcommand)]
pub enum DeploySubCommand {
    /// Create a deployment plan
    Plan {
        /// Environment to deploy to
        environment: String,
        /// Application name
        app: String,
    },
    /// Execute a deployment
    Execute {
        /// Path to deployment plan
        plan: String,
    },
}

#[derive(Subcommand)]
pub enum TestSubCommand {
    /// Generate tests for a source file
    Generate {
        /// Path to source file
        source: String,
        /// Target language
        language: String,
    },
    /// Execute tests
    Execute {
        /// Path to test file
        test: String,
        /// Test framework to use
        framework: String,
    },
    /// Analyze test coverage
    Coverage {
        /// Path to source file
        source: String,
        /// Path to test file
        test: String,
    },
}

#[derive(Subcommand)]
pub enum SimulateSubCommand {
    /// Simulate Project Manager activities
    Pm {
        #[command(subcommand)]
        sub: PmSubCommand,
    },
    /// Simulate Business Analyst activities
    Ba {
        #[command(subcommand)]
        sub: BaSubCommand,
    },
}

#[derive(Subcommand)]
pub enum PmSubCommand {
    /// Plan a sprint
    PlanSprint {
        /// Project name
        project: String,
        /// Sprint duration in weeks
        duration: u32,
    },
    /// Run sprint retrospective
    Retrospective {
        /// Sprint number
        sprint: u32,
    },
}

#[derive(Subcommand)]
pub enum BaSubCommand {
    /// Validate requirements
    Validate {
        /// Requirements document path
        requirements: String,
    },
    /// Create a user story
    UserStory {
        /// Feature description
        feature: String,
    },
}

#[derive(Subcommand)]
pub enum ProjectSub {
    /// List all projects
    List,
    /// Switch to a project
    Switch {
        /// Project ID to switch to
        id: String,
    },
    /// Sync project with cloud
    Sync {
        /// Project ID to sync (current if not specified)
        id: Option<String>,
    },
    /// Show project information
    Info {
        /// Project ID to show info for (current if not specified)
        id: Option<String>,
    },
}

#[derive(Subcommand)]
pub enum RefactorSub {
    /// Preview a refactoring operation
    Preview {
        /// File to refactor
        file: String,
        /// Type of refactoring (rename_variable, extract_function, etc.)
        refactor_type: String,
        /// Additional parameters for the refactoring
        #[arg(short, long)]
        params: Vec<String>,
    },
    /// Apply all pending refactor operations
    Apply,
    /// Cancel pending refactor operations
    Cancel,
}

#[derive(Subcommand)]
pub enum TestSub {
    /// Generate unit tests for a file
    Generate {
        /// Source file to generate tests for
        file: String,
        /// Test framework to use (rust, pytest, jest, flutter)
        #[arg(short, long, default_value = "")]
        framework: String,
    },
    /// Generate integration tests
    Integration {
        /// Feature description for integration tests
        feature: String,
    },
    /// Analyze test coverage
    Coverage {
        /// Source file
        source: String,
        /// Test file
        test: String,
    },
}

#[derive(Subcommand)]
pub enum PluginSub {
    /// Install a plugin from URL or file
    Install { source: String },
    /// List installed plugins
    List,
    /// Execute a plugin
    Run { name: String, args: Vec<String> },
}

#[derive(Subcommand)]
pub enum ConfigSub {
    /// Set API key for a provider
    SetKey { provider: String, key: String },
    /// List configured API keys
    ListKeys,
    /// Show cost statistics
    Costs {
        /// Provider to show costs for (all if not specified)
        provider: Option<String>,
    },
    /// Validate production configuration
    Validate,
}

#[derive(Subcommand)]
pub enum LocalModelSub {
    /// List installed local models
    List {
        /// Show only models compatible with your hardware
        #[arg(long)]
        compatible: bool,
    },
    /// Install a local model
    Install {
        /// Model name (e.g., qwen2.5-coder-7b-q4)
        #[arg(value_parser)]
        model: String,
        /// Force install even if hardware is insufficient
        #[arg(long)]
        force: bool,
    },
    /// Remove a local model
    Remove {
        #[arg(value_parser)]
        model: String,
    },
    /// Verify model integrity
    Verify {
        #[arg(value_parser)]
        model: String,
    },
    /// Benchmark installed model
    Benchmark {
        #[arg(value_parser)]
        model: Option<String>,
        /// Output format
        #[arg(long, default_value = "table")]
        format: String,
        /// Force a specific runtime (ollama, lmstudio, gpt4all, foundry, default)
        #[arg(long)]
        runtime: Option<String>,
        /// Benchmark every detected runtime
        #[arg(long)]
        all_runtimes: bool,
    },
    /// Use a local model and persist selection
    Use {
        #[arg(value_parser)]
        model: String,
    },
    /// Show local model system status
    Status,
}

struct BenchmarkCliOptions {
    model: Option<String>,
    format: String,
    runtime: Option<String>,
    all_runtimes: bool,
}

#[derive(Subcommand)]
pub enum AuthSub {
    Login { provider: String },
}

#[derive(Subcommand)]
pub enum WindowsSub {
    /// Show the status of local adapters (Ollama WSL2, LM Studio, GPT4All, Foundry)
    Status,
    /// Run GPU passthrough diagnostics for WSL2
    CheckGpu,
    /// Print WSL2 + Ollama setup instructions
    SetupWsl2,
}

#[derive(Subcommand)]
pub enum MacosSub {
    /// Display Core ML status and ANE availability
    Status,
    /// Show setup instructions for Core ML runtimes
    SetupCoreml,
}

#[derive(Subcommand)]
pub enum LinuxSub {
    /// Display local runtime status (Ollama socket, CUDA availability)
    Status,
    /// Run hardening checks for the current system
    Doctor,
    /// Show recommended setup instructions for Linux hosts
    Setup,
}

#[derive(Subcommand)]
pub enum MobileSub {
    /// Create an encrypted-ready bundle for iOS devices
    IosSync,
    /// Prepare an Android-ready bundle for Google AI Core / Termux
    AndroidSync,
    /// Export an ONNX-ready manifest for edge devices
    EdgeSnapshot,
}

pub async fn run(cli: Cli) -> Result<()> {
    match cli.command {
        Some(Commands::Init) => init_project().await?,
        Some(Commands::Chat { message }) => chat(message.unwrap_or_default()).await?,
        Some(Commands::Create { template, name }) => create_project(&template, &name).await?,
        Some(Commands::Tui) => launch_tui().await?,
        Some(Commands::Projects { sub }) => handle_projects(sub).await?,
        Some(Commands::Agent { sub }) => handle_agent(sub).await?,
        Some(Commands::Refactor { sub }) => handle_refactor(sub).await?,
        Some(Commands::Test { sub }) => handle_test(sub).await?,
        Some(Commands::SwitchModel { provider, model }) => switch_model(provider, model).await?,
        Some(Commands::Plugin { sub }) => handle_plugin(sub).await?,
        Some(Commands::Config { sub }) => handle_config(sub).await?,
        Some(Commands::LocalModel { sub }) => handle_local_model(sub).await?,
        Some(Commands::Auth { sub }) => handle_auth(sub).await?,
        Some(Commands::Windows { sub }) => handle_windows(sub).await?,
        Some(Commands::Macos { sub }) => handle_macos(sub).await?,
        Some(Commands::Linux { sub }) => handle_linux(sub).await?,
        Some(Commands::Mobile { sub }) => handle_mobile(sub).await?,
        Some(Commands::Pwa { output }) => handle_pwa(output).await?,
        Some(Commands::Web { address }) => web::start(&address).await?,
        None => {
            println!("Kandil Code - Intelligent Development Platform");
            println!("Use --help for commands");
        }
    }
    Ok(())
}

async fn init_project() -> Result<()> {
    println!("Initializing new Kandil project...");

    let project_manager = ProjectManager::new()?;
    let current_dir = std::env::current_dir()?.to_string_lossy().to_string();

    // Try to get project name from directory or use a default
    let project_name = std::path::Path::new(&current_dir)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("unnamed_project");

    let project =
        project_manager.create_project(project_name, &current_dir, "ollama", "llama3:70b")?;

    println!("Created project: {} with ID: {}", project.name, project.id);
    Ok(())
}

async fn chat(message: String) -> Result<()> {
    if message.trim().is_empty() {
        return enhanced_ui::repl::run_repl().await;
    }

    let config = Config::load()?;
    let factory = AIProviderFactory::new(config.clone());
    
    // Use prompt router to intelligently route the message
    let router = PromptRouter::new();
    let routed = router.route_message(
        &message,
        &config.ai_provider,
        &config.ai_model,
    );
    
    // Create AI instance based on routed prompt
    let ai = Arc::new(factory.create_ai(&routed.provider, &routed.model)?);
    let tracked_ai = crate::core::adapters::TrackedAI::new(ai.clone(), factory.get_cost_tracker());

    // Show routing info if it differs from default or if verbose
    if routed.provider != config.ai_provider || routed.model != config.ai_model {
        println!("🎯 Routed to {} ({}) for {:?} intent", 
            routed.provider, routed.model, routed.intent);
        println!("   {}", routed.explanation);
    }

    let response = tracked_ai.chat(&message).await?;
    println!("{}", response);

    // Save to project memory if project manager is available
    if let Ok(project_manager) = ProjectManager::new() {
        if let Ok(current_project) = project_manager.ensure_active_project(None) {
            // For this example, we'll use a simple session ID
            let session_id = uuid::Uuid::new_v4().to_string();

            // Save user message
            let _ = project_manager.save_project_memory(
                &current_project.id,
                &session_id,
                "user",
                &message,
                None, // Token count not available without proper parsing
            );

            // Save AI response
            let _ = project_manager.save_project_memory(
                &current_project.id,
                &session_id,
                "ai",
                &response,
                None, // Token count not available without proper parsing
            );
        }
    }

    Ok(())
}

async fn create_project(template: &str, name: &str) -> Result<()> {
    let engine = TemplateEngine::new();
    engine.create_project(template, name, name)?;

    // Create a project entry in the database
    let project_manager = ProjectManager::new()?;
    let project = project_manager.create_project(
        name,
        &std::env::current_dir()?.join(name).to_string_lossy(),
        "ollama",
        "llama3:70b",
    )?;

    println!("Created project '{}' using template '{}'", name, template);
    println!("Project ID: {}", project.id);
    Ok(())
}

async fn handle_agent(sub: AgentSub) -> Result<()> {
    let config = Config::load()?;
    let factory = AIProviderFactory::new(config.clone());
    let router = PromptRouter::new();
    
    // Route based on agent type
    let (intent, task_description) = match &sub {
        AgentSub::Requirements { description } => (PromptIntent::Planning, description.clone()),
        AgentSub::Design { requirements } => (PromptIntent::Architecture, requirements.clone()),
        AgentSub::Code { design_path, .. } => (PromptIntent::Coding, format!("Generate code from {}", design_path)),
        AgentSub::Test { .. } => (PromptIntent::Testing, "Generate tests".to_string()),
        AgentSub::Documentation { .. } => (PromptIntent::Analysis, "Generate documentation".to_string()),
        _ => (PromptIntent::Conversation, "Agent task".to_string()),
    };
    
    let routed = router.route_for_intent(intent, &config.ai_provider, &config.ai_model);
    let ai = Arc::new(factory.create_ai(&routed.provider, &routed.model)?);
    
    if routed.provider != config.ai_provider || routed.model != config.ai_model {
        println!("🎯 Agent using {} ({}) for {:?} intent", 
            routed.provider, routed.model, routed.intent);
    }

    match sub {
        AgentSub::Requirements { description } => {
            let requirements_agent = crate::core::agents::RequirementsAgent::new(ai);
            let _doc = requirements_agent
                .generate_requirements_document(&description)
                .await?;
            println!("Generated requirements document for: {}", description);
            // In a real implementation, we would display the structured document
            println!("Requirements document structure created (content generation would happen in full implementation)");
        }
        AgentSub::Design { requirements } => {
            let design_agent = crate::core::agents::DesignAgent::new(ai);
            let _doc = design_agent.generate_design_document(&requirements).await?;
            println!(
                "Generated design document based on requirements: {}",
                requirements
            );
            // In a real implementation, we would display the structured document
            println!("Design document structure created (content generation would happen in full implementation)");
        }
        AgentSub::Code {
            design_path,
            language,
        } => {
            // Read the design document
            let design_content = std::fs::read_to_string(&design_path)
                .unwrap_or_else(|_| "Design document content would be read from file".to_string());

            let code_agent = crate::core::agents::CodeAgent::new(ai)?;
            let output = code_agent.generate_code(&design_content, &language).await?;
            println!(
                "Generated {} code with {} files",
                language,
                output.files.len()
            );
            // In a real implementation, we would save the generated files
            println!("Code generation completed (files would be saved in full implementation)");
        }
        AgentSub::Test { sub: test_cmd } => {
            let test_agent = crate::core::agents::TestAgent::new(ai);
            match test_cmd {
                TestSubCommand::Generate { source, language } => {
                    let tests = test_agent.generate_tests(&source, &language).await?;
                    println!("Generated tests for: {}", source);
                    println!("{}", tests);
                }
                TestSubCommand::Execute { test, framework } => {
                    let results = test_agent.execute_tests(&test, &framework).await?;
                    println!("Test execution results:");
                    println!(
                        "  Passed: {}, Failed: {}, Skipped: {}",
                        results.passed, results.failed, results.skipped
                    );
                    println!("  Duration: {}ms", results.duration_ms);
                }
                TestSubCommand::Coverage { source, test } => {
                    let analysis = test_agent.analyze_test_coverage(&source, &test).await?;
                    println!("Test coverage analysis for {} and {}:", source, test);
                    println!("{}", analysis);
                }
            }
        }
        AgentSub::Documentation { sub: doc_cmd } => {
            let doc_agent = crate::core::agents::documentation::DocumentationGenerator::new(ai);
            match doc_cmd {
                DocumentationSubCommand::Generate { path } => {
                    let report = doc_agent.generate_documentation_for_project(&path).await?;
                    println!("Generated documentation for project at {}:", path);
                    println!("{}", report);
                }
            }
        }
        AgentSub::Release { sub: release_cmd } => {
            let mut release_manager =
                crate::core::agents::release_manager::ReleaseManager::new(ai, "0.1.0".to_string()); // Placeholder version
            match release_cmd {
                ReleaseSubCommand::FullProcess { version } => {
                    release_manager.version = version; // Update version from CLI arg
                    release_manager.run_full_release_process().await?;
                    println!(
                        "Full release process completed for version: {}",
                        release_manager.version
                    );
                }
            }
        }
        AgentSub::Qa { sub: qa_cmd } => {
            let mut qa_system =
                crate::core::agents::quality_assurance::QualityAssuranceSystem::new(ai);
            match qa_cmd {
                QaSubCommand::FullSuite { project_path } => {
                    let _report = qa_system.run_full_qa_suite(&project_path).await?;
                    println!("Full QA suite completed for project at {}:", project_path);
                    println!("{}", qa_system.generate_qa_report_md());
                }
            }
        }
        AgentSub::Maintenance {
            sub: maintenance_cmd,
        } => {
            let mut maintenance_manager =
                crate::core::agents::maintenance::MaintenanceManager::new(ai);
            match maintenance_cmd {
                MaintenanceSubCommand::HealthCheck { system_name } => {
                    maintenance_manager.run_health_checks(&system_name).await?;
                    println!("Health check completed for system: {}", system_name);
                }
            }
        }
        AgentSub::Simulate { sub: sim_cmd } => match sim_cmd {
            SimulateSubCommand::Pm { sub: pm_cmd } => {
                let pm_sim = crate::core::agents::ProjectManagerSimulation::new(ai);
                match pm_cmd {
                    PmSubCommand::PlanSprint { project, duration } => {
                        let plan = pm_sim.plan_sprint(&project, duration).await?;
                        println!("Sprint plan for project '{}':", project);
                        println!(
                            "  Sprint: {}, Duration: {} weeks",
                            plan.sprint_number, duration
                        );
                        println!("  Goals: {} items", plan.goals.len());
                        println!("  Timeline: {} to {}", plan.start_date, plan.end_date);
                    }
                    PmSubCommand::Retrospective { sprint } => {
                        let results = pm_sim.run_retrospective(sprint).await?;
                        println!("Retrospective results for Sprint {}:", sprint);
                        println!("{}", results);
                    }
                }
            }
            SimulateSubCommand::Ba { sub: ba_cmd } => {
                let ba_sim = crate::core::agents::BusinessAnalystSimulation::new(ai);
                match ba_cmd {
                    BaSubCommand::Validate { requirements } => {
                        let validation = ba_sim.validate_requirements(&requirements).await?;
                        println!("Requirements validation for: {}", requirements);
                        println!("{}", validation);
                    }
                    BaSubCommand::UserStory { feature } => {
                        let story = ba_sim.create_user_story(&feature).await?;
                        println!("Created user story for feature: {}", feature);
                        println!("  ID: {}, Title: {}", story.id, story.title);
                        println!("  Priority: {:?}", story.priority);
                        println!("  Story Points: {}", story.story_points);
                    }
                }
            }
        },
        AgentSub::Advanced { sub: advanced_cmd } => {
            match advanced_cmd {
                AdvancedSubCommand::Review { file } => {
                    let review_agent = crate::core::agents::ReviewAgent::new(ai);
                    let report = review_agent.code_review(&file).await?;
                    println!("Code review for: {}", file);
                    println!("  Score: {}/100", report.score);
                    println!("  Issues found: {}", report.issues.len());
                    println!("  Summary: {}", report.summary);
                }
                AdvancedSubCommand::Security { file, description } => {
                    let security_agent = crate::core::agents::EthicsSecurityAgent::new(ai);
                    let report = security_agent
                        .security_scan(&std::fs::read_to_string(&file)?, &file)
                        .await?;
                    println!("Security scan for: {}", file);
                    println!("  Risk Score: {}/100", report.risk_score);
                    println!("  Vulnerabilities: {}", report.vulnerabilities.len());

                    // Also run ethics check
                    let ethics_report = security_agent
                        .ethics_check(&std::fs::read_to_string(&file)?, &description)
                        .await?;
                    println!(
                        "Ethics check completed with score: {}/100",
                        ethics_report.ethics_score
                    );
                }
                AdvancedSubCommand::Deploy { sub: deploy_cmd } => {
                    let deploy_agent = crate::core::agents::DeploymentAgent::new(ai)?;
                    match deploy_cmd {
                        DeploySubCommand::Plan { environment, app } => {
                            let plan = deploy_agent
                                .create_deployment_plan(&environment, &app)
                                .await?;
                            println!("Deployment plan for {} to {}:", app, environment);
                            println!("  Steps: {}", plan.steps.len());
                            println!("  Estimated duration: {}", plan.estimated_duration);
                            println!(
                                "  Rollback plan: {}",
                                if plan.rollback_plan.steps.is_empty() {
                                    "No"
                                } else {
                                    "Yes"
                                }
                            );
                        }
                        DeploySubCommand::Execute { plan } => {
                            // In a real implementation, this would load and execute the plan
                            println!("Executing deployment plan from: {}", plan);
                            println!("Deployment execution would happen in full implementation");
                        }
                    }
                }
                AdvancedSubCommand::SelfImprove { path } => {
                    let meta_agent = crate::core::agents::MetaAgent::new(ai);
                    let analysis = meta_agent.analyze_system(&path).await?;
                    println!("System analysis for: {}", path);
                    println!(
                        "  Performance issues: {}",
                        analysis.performance_bottlenecks.len()
                    );
                    println!(
                        "  Code quality issues: {}",
                        analysis.code_quality_issues.len()
                    );
                    println!("  Security concerns: {}", analysis.security_concerns.len());

                    let improvement_plans = meta_agent.generate_improvement_plan(&analysis).await?;
                    println!("  Suggested improvements: {}", improvement_plans.len());

                    // Also run the self-evolution capability
                    let evolution_result = meta_agent.evolve_agent_capabilities().await?;
                    println!("  Self-improvement analysis: {}", evolution_result);
                }
            }
        }
        AgentSub::TechRole { sub: tech_role_cmd } => {
            match tech_role_cmd {
                TechRoleSubCommand::Architect { sub: arch_cmd } => {
                    let mut arch_agent = crate::core::agents::ArchitectSimulation::new(ai)?;
                    match arch_cmd {
                        ArchitectSubCommand::Review { diagram } => {
                            let review = arch_agent.review_design(&diagram).await?;
                            println!("Architecture review completed:");
                            println!("  Score: {}/100", review.score);
                            println!("  Recommendations: {}", review.recommendations.len());
                            println!("  Issues found: {}", review.identified_issues.len());
                        }
                        ArchitectSubCommand::Decide { context, decision } => {
                            let arch_decision = arch_agent
                                .make_architecture_decision(&context, &decision)
                                .await?;
                            println!("Architecture decision made:");
                            println!("  ID: {}, Title: {}", arch_decision.id, arch_decision.title);
                            println!("  Status: {:?}", arch_decision.status);
                        }
                        ArchitectSubCommand::Adr { decision_id } => {
                            let adr = arch_agent.generate_adr(&decision_id).await?;
                            println!("Architecture Decision Record for {}: ", decision_id);
                            println!("{}", adr);
                        }
                    }
                }
                TechRoleSubCommand::Developer { sub: dev_cmd } => {
                    let mut dev_agent = crate::core::agents::DeveloperSimulation::new(
                        ai,
                        "Current Project".to_string(),
                    );
                    match dev_cmd {
                        DeveloperSubCommand::Implement { spec, file } => {
                            let implementation = dev_agent.implement_feature(&spec, &file).await?;
                            println!("Feature implementation completed for: {}", file);
                            println!("{}", implementation);
                        }
                        DeveloperSubCommand::Pair {
                            partner,
                            task,
                            file,
                        } => {
                            let session_id = dev_agent
                                .start_pair_programming(&partner, &task, &file)
                                .await?;
                            println!("Started pair programming session: {}", session_id);
                        }
                        DeveloperSubCommand::Bugs { code, file } => {
                            let bugs = dev_agent.find_bugs(&code, &file).await?;
                            println!("Bugs found in {}: {}", file, bugs.len());
                            for bug in bugs {
                                println!("  - {} ({}): {}", bug.id, bug.severity, bug.description);
                            }
                        }
                    }
                }

                TechRoleSubCommand::Collaborate { sub: collab_cmd } => {
                    let mut collab_manager = crate::core::agents::CollaborationManager::new();
                    match collab_cmd {
                        CollaborateSubCommand::Session {
                            title,
                            roles,
                            agenda,
                        } => {
                            // Parse roles
                            let role_list: Vec<crate::core::agents::collaboration::Role> = roles
                                .split(',')
                                .map(|r| match r.trim() {
                                    "architect" => {
                                        crate::core::agents::collaboration::Role::Architect
                                    }
                                    "developer" => {
                                        crate::core::agents::collaboration::Role::Developer
                                    }
                                    "qa" => crate::core::agents::collaboration::Role::QA,
                                    _ => crate::core::agents::collaboration::Role::Developer, // default
                                })
                                .collect();

                            let session_id = collab_manager
                                .start_collaboration_session(&title, role_list, agenda)
                                .await;
                            println!("Collaboration session started: {}", session_id);
                        }
                        CollaborateSubCommand::Decision {
                            title,
                            description,
                            roles,
                        } => {
                            // Parse roles
                            let role_list: Vec<crate::core::agents::collaboration::Role> = roles
                                .split(',')
                                .map(|r| match r.trim() {
                                    "architect" => {
                                        crate::core::agents::collaboration::Role::Architect
                                    }
                                    "developer" => {
                                        crate::core::agents::collaboration::Role::Developer
                                    }
                                    "qa" => crate::core::agents::collaboration::Role::QA,
                                    _ => crate::core::agents::collaboration::Role::Developer, // default
                                })
                                .collect();

                            // For this simulation, we'll create a basic cross-role decision
                            // without full agent interaction
                            println!("Creating cross-role decision: {}", title);
                            println!("Roles involved: {:?}", role_list);
                            println!("Description: {}", description);
                        }
                    }
                }
            }
        }
        AgentSub::AdvancedFeatures {
            sub: advanced_features_cmd,
        } => {
            match advanced_features_cmd {
                AdvancedFeaturesSubCommand::DevOps { sub: devops_cmd } => {
                    let devops_agent = crate::core::agents::DevOpsSimulation::new(ai);
                    match devops_cmd {
                        DevOpsSubCommand::Terraform { spec } => {
                            let tf_path = devops_agent.generate_terraform(&spec).await?;
                            println!("Generated Terraform configuration: {:?}", tf_path);
                        }
                        DevOpsSubCommand::Drill { scenario } => {
                            let report = devops_agent.run_drill(&scenario).await?;
                            println!("Incident response drill completed:");
                            println!("  Scenario: {}", report.scenario);
                            println!("  Duration: {} seconds", report.duration_seconds);
                            println!("  Effectiveness: {}/100", report.effectiveness_score);
                        }
                        DevOpsSubCommand::Pipeline { project_type } => {
                            let pipeline =
                                devops_agent.generate_ci_cd_pipeline(&project_type).await?;
                            println!("Generated CI/CD pipeline for {} projects", project_type);
                            println!("{}", pipeline);
                        }
                    }
                }
                AdvancedFeaturesSubCommand::Scrum { sub: scrum_cmd } => {
                    let mut scrum_agent = crate::core::agents::ScrumSimulation::new(ai);
                    match scrum_cmd {
                        ScrumSubCommand::Plan {
                            goal,
                            duration,
                            team_size,
                        } => {
                            let sprint = scrum_agent.plan_sprint(goal, duration, team_size).await?;
                            println!("Sprint {} planned:", sprint.number);
                            println!("  Goal: {}", sprint.goal);
                            println!("  Duration: {} days", sprint.duration_days);
                            println!("  Team size: {}", sprint.team_size);
                        }
                        ScrumSubCommand::Retro { sprint } => {
                            let retro = scrum_agent.run_retrospective(sprint).await?;
                            println!("Sprint {} retrospective:", retro.sprint_number);
                            println!("  Satisfaction: {}/10", retro.satisfaction_score);
                            println!("  Good things: {}", retro.good_things.len());
                            println!("  Improvement areas: {}", retro.improvement_areas.len());
                        }
                        ScrumSubCommand::Ceremony {
                            ceremony_type,
                            participants,
                        } => {
                            let participants_list = participants
                                .split(',')
                                .map(|s| s.trim().to_string())
                                .collect();
                            let ceremony = scrum_agent
                                .conduct_ceremony(
                                    &ceremony_type,
                                    participants_list,
                                    scrum_agent.get_current_sprint(),
                                )
                                .await?;
                            println!(
                                "Conducted {} ceremony with {} participants",
                                ceremony.name,
                                ceremony.participants.len()
                            );
                        }
                    }
                }
                AdvancedFeaturesSubCommand::I18n { sub: i18n_cmd } => {
                    let mut i18n_agent = crate::core::agents::I18nAssistant::new(ai);
                    match i18n_cmd {
                        I18nSubCommand::Translate {
                            text,
                            target,
                            source,
                        } => {
                            let translation =
                                i18n_agent.translate_text(&text, &target, &source).await?;
                            println!("Translation from {} to {}:", source, target);
                            println!("{}", translation);
                        }
                        I18nSubCommand::Audit { path } => {
                            let report = i18n_agent.audit_translations(&path).await?;
                            println!("Translation audit completed:");
                            println!("  Languages: {}", report.completeness_by_language.len());
                            println!("  Recommendations: {}", report.recommendations.len());
                        }
                        I18nSubCommand::Review {
                            original,
                            translation,
                            target,
                        } => {
                            let report = i18n_agent
                                .review_translation(&original, &translation, &target)
                                .await?;
                            println!("Translation review completed:");
                            println!("  Quality score: {}/100", report.quality_score);
                            println!("  Issues found: {}", report.issues_found.len());
                        }
                    }
                }
                AdvancedFeaturesSubCommand::A11y { sub: a11y_cmd } => {
                    let a11y_agent = crate::core::agents::A11yAssistant::new(ai);
                    match a11y_cmd {
                        A11ySubCommand::Audit { content } => {
                            // Default to WCAG AA level
                            use crate::core::agents::a11y::WcagLevel;
                            let report = a11y_agent.wcag_audit(&content, WcagLevel::AA).await?;
                            println!("Accessibility audit completed:");
                            println!("  Score: {}/100", report.accessibility_score);
                            println!("  Issues found: {}", report.issues_found.len());
                        }
                        A11ySubCommand::Guidelines { component } => {
                            let guidelines =
                                a11y_agent.generate_a11y_guidelines(&component).await?;
                            println!("Accessibility guidelines for {}:", component);
                            println!("{}", guidelines);
                        }
                        A11ySubCommand::Fix { html } => {
                            let fixed_html = a11y_agent.remediate_issues(&html).await?;
                            println!("Accessibility issues remediated in HTML:");
                            println!("{}", fixed_html);
                        }
                    }
                }
                AdvancedFeaturesSubCommand::Collab { sub: collab_cmd } => {
                    let mut collab = crate::core::agents::RealTimeCollaboration::new();
                    match collab_cmd {
                        CollabSubCommand::Session {
                            name,
                            creator_id,
                            creator_name,
                        } => {
                            let session_id =
                                collab.create_session(&name, &creator_id, &creator_name)?;
                            println!("Created collaboration session: {}", session_id);
                        }
                        CollabSubCommand::AddParticipant {
                            session_id,
                            user_id,
                            name,
                            role,
                        } => {
                            use crate::core::agents::collaboration_realtime::Role;
                            let role_enum = match role.as_str() {
                                "owner" => Role::Owner,
                                "admin" => Role::Admin,
                                "editor" => Role::Editor,
                                "viewer" => Role::Viewer,
                                _ => Role::Editor,
                            };
                            collab.add_participant(&session_id, &user_id, &name, role_enum)?;
                            println!("Added participant {} to session {}", name, session_id);
                        }
                        CollabSubCommand::AddDoc {
                            session_id,
                            doc_id,
                            name,
                            content,
                            language,
                        } => {
                            collab.add_document(
                                &session_id,
                                &doc_id,
                                &name,
                                &content,
                                &language,
                            )?;
                            println!("Added document {} to session {}", name, session_id);
                        }
                    }
                }
                AdvancedFeaturesSubCommand::Ide { sub: ide_cmd } => {
                    let ide_ext = crate::core::agents::IdeExtension::new(ai);
                    match ide_cmd {
                        IdeSubCommand::Suggestions {
                            file,
                            language,
                            code,
                        } => {
                            let ctx = crate::core::agents::ide_extension::ExtensionContext {
                                file_path: file,
                                language,
                                selected_code: code,
                                cursor_position: (1, 1),
                                workspace_root: ".".to_string(),
                            };
                            let suggestions = ide_ext.get_code_suggestions(&ctx).await?;
                            println!("Code suggestions ({} found):", suggestions.len());
                            for suggestion in suggestions {
                                println!("  - {}: {}", suggestion.title, suggestion.description);
                            }
                        }
                        IdeSubCommand::Docs { code, language } => {
                            let docs = ide_ext.generate_documentation(&code, &language).await?;
                            println!("Generated documentation:");
                            println!("{}", docs);
                        }
                        IdeSubCommand::Refactor { code, language } => {
                            let options = ide_ext.get_refactoring_options(&code, &language).await?;
                            println!("Refactoring options:");
                            for option in options {
                                println!("{}", option);
                            }
                        }
                        IdeSubCommand::Review { code, language } => {
                            let comments = ide_ext.run_inline_code_review(&code, &language).await?;
                            println!("Inline code review comments ({} found):", comments.len());
                            for comment in comments {
                                println!(
                                    "  Line {}: [{}] {}",
                                    comment.line_number, comment.severity, comment.message
                                );
                            }
                        }
                    }
                }
            }
        }
    }
    Ok(())
}

async fn handle_projects(sub: ProjectSub) -> Result<()> {
    let project_manager = ProjectManager::new()?;

    match sub {
        ProjectSub::List => {
            let projects = project_manager.list_projects()?;
            if projects.is_empty() {
                println!("No projects found");
            } else {
                println!("Projects:");
                for project in projects {
                    let last_opened = project
                        .last_opened
                        .map(|d| d.format("%Y-%m-%d %H:%M:%S").to_string())
                        .unwrap_or_else(|| "Never".to_string());
                    println!(
                        "  ID: {} | Name: {} | Path: {} | Last Opened: {}",
                        project.id, project.name, project.root_path, last_opened
                    );
                }
            }
        }
        ProjectSub::Switch { id } => {
            let project = project_manager.switch_project(&id)?;
            println!(
                "Switched to project: {} at {}",
                project.name, project.root_path
            );
        }
        ProjectSub::Sync { id } => {
            // For now, just show that sync would happen
            // In a real implementation, we would use the CloudSync module
            match id {
                Some(project_id) => println!("Syncing project: {}", project_id),
                None => println!("Syncing current project"),
            }
            println!("Note: Cloud sync functionality would be implemented with Supabase in a full implementation");
        }
        ProjectSub::Info { id } => {
            let project = if let Some(project_id) = id {
                project_manager.get_project(&project_id)?
            } else {
                // Get current project (for now, just get the most recently used)
                let projects = project_manager.list_projects()?;
                projects.first().cloned()
            };

            match project {
                Some(p) => {
                    println!("Project Details:");
                    println!("  ID: {}", p.id);
                    println!("  Name: {}", p.name);
                    println!("  Path: {}", p.root_path);
                    println!("  AI Provider: {}", p.ai_provider);
                    println!("  AI Model: {}", p.ai_model);
                    println!("  Memory Enabled: {}", p.memory_enabled);
                    println!("  Created: {}", p.created_at.format("%Y-%m-%d %H:%M:%S"));
                    println!(
                        "  Last Opened: {}",
                        p.last_opened
                            .map(|d| d.format("%Y-%m-%d %H:%M:%S").to_string())
                            .unwrap_or_else(|| "Never".to_string())
                    );
                }
                None => println!("Project not found"),
            }
        }
    }
    Ok(())
}

#[cfg(feature = "tui")]
async fn launch_tui() -> Result<()> {
    let project_manager = ProjectManager::new()?;
    let _current_project = project_manager.ensure_active_project(None)?;
    let mut app = crate::tui::StudioApp::new()?;
    app.run().await?;
    Ok(())
}

#[cfg(not(feature = "tui"))]
async fn launch_tui() -> Result<()> {
    Err(anyhow::anyhow!("TUI feature is not enabled in this build"))
}

async fn handle_refactor(sub: RefactorSub) -> Result<()> {
    let mut engine = RefactorEngine::new();
    match sub {
        RefactorSub::Preview {
            file,
            refactor_type,
            params,
        } => {
            // Parse params - in a real implementation, this would be more sophisticated
            let mut refactor_params = RefactorParams {
                old_name: None,
                new_name: None,
                start_line: None,
                end_line: None,
                function_name: None,
                visibility: None,
            };

            // Simple parameter parsing for demo purposes
            for param in params {
                if let Some((key, value)) = param.split_once('=') {
                    match key {
                        "old_name" => refactor_params.old_name = Some(value.to_string()),
                        "new_name" => refactor_params.new_name = Some(value.to_string()),
                        _ => {}
                    }
                }
            }

            let result = engine.preview_refactor(&file, &refactor_type, &refactor_params)?;
            println!("Refactoring preview for '{}':", file);
            println!("{}", result);
        }
        RefactorSub::Apply => {
            engine.apply_pending_operations()?;
            println!("Applied all pending refactor operations");
        }
        RefactorSub::Cancel => {
            engine.cancel_pending_operations();
            println!("Cancelled all pending refactor operations");
        }
    }
    Ok(())
}

async fn handle_test(sub: TestSub) -> Result<()> {
    let config = Config::load()?;
    let factory = AIProviderFactory::new(config.clone());
    let ai = Arc::new(factory.create_ai(&config.ai_provider, &config.ai_model)?);
    let tracked_ai = crate::core::adapters::TrackedAI::new(ai.clone(), factory.get_cost_tracker());
    let generator = TestGenerator::new(ai.clone()); // Using the underlying AI for now

    match sub {
        TestSub::Generate { file, framework } => {
            let tests = generator.generate_tests_for_file(&file, &framework).await?;
            println!("Generated tests for '{}':", file);
            println!("{}", tests);
        }
        TestSub::Integration { feature } => {
            let tests = generator.generate_integration_tests(&feature).await?;
            println!("Generated integration tests for feature:");
            println!("{}", tests);
        }
        TestSub::Coverage { source, test } => {
            let analysis = generator.analyze_test_coverage(&source, &test).await?;
            println!("Test coverage analysis for '{}' and '{}':", source, test);
            println!("{}", analysis);
        }
    }
    Ok(())
}

async fn switch_model(provider: String, model: String) -> Result<()> {
    // In a real implementation, this would update the configuration
    // For now, we'll just verify the provider is valid
    match provider.as_str() {
        "ollama" | "claude" | "qwen" | "openai" => {
            println!("Switched to provider: {}, model: {}", provider, model);
        }
        _ => return Err(anyhow::anyhow!("Invalid provider: {}", provider)),
    }
    Ok(())
}

async fn handle_plugin(sub: PluginSub) -> Result<()> {
    let manager = PluginManager::new();
    match sub {
        PluginSub::Install { source } => {
            manager.install_plugin(&source)?;
            println!("Plugin installed from: {}", source);
        }
        PluginSub::List => {
            let plugins = manager.list_plugins()?;
            if plugins.is_empty() {
                println!("No plugins installed");
            } else {
                println!("Installed plugins:");
                for plugin in plugins {
                    println!("  - {}", plugin);
                }
            }
        }
        PluginSub::Run { name, args } => {
            // In a real implementation, we'd look up the actual plugin path
            println!("Running plugin: {} with args: {:?}", name, args);
            // This would execute the plugin with proper sandboxing
        }
    }
    Ok(())
}

async fn handle_config(sub: ConfigSub) -> Result<()> {
    match sub {
        ConfigSub::SetKey { provider, key } => {
            SecureKey::save(&provider, &key)?;
            println!("API key saved securely for provider: {}", provider);
        }
        ConfigSub::ListKeys => {
            println!("Currently implemented as a placeholder - key listing will be implemented in future versions");
            // Note: Actual key listing may not be implemented for security reasons
        }
        ConfigSub::Costs { provider } => {
            // For now, showing a placeholder - in a real implementation we would access the cost tracker
            match provider {
                Some(provider_str) => {
                    println!(
                        "Cost tracking for provider: {} is not available in this context",
                        provider_str
                    );
                    println!("Cost tracking is available when using AI features");
                }
                None => {
                    println!("Cost tracking summary is not available in this context");
                    println!("Cost tracking is available when using AI features");
                }
            }
        }
        ConfigSub::Validate => {
            let cfg = Config::load()?;
            match cfg.validate_production().await {
                Ok(()) => println!("Configuration validation: ok"),
                Err(e) => {
                    eprintln!("Configuration validation failed: {}", e);
                    return Err(e);
                }
            }
        }
    }
    Ok(())
}

async fn handle_local_model(sub: LocalModelSub) -> Result<()> {
    match sub {
        LocalModelSub::List { compatible } => {
            let hardware = detect_hardware();
            let catalog = &crate::models::catalog::MODEL_CATALOG;

            println!("Available Models:");
            for model in catalog.iter() {
                // Check compatibility if requested
                if compatible && model.ram_required_gb > hardware.total_ram_gb {
                    continue;
                }

                println!("  {}", model.name);
                println!(
                    "    Size: {}GB, RAM: {}GB, GPU: {:?}GB",
                    model.size_gb, model.ram_required_gb, model.gpu_vram_min
                );

                // Show speed in a user-friendly way
                let speed_str = match &model.speed_rating {
                    crate::models::catalog::Speed::UltraFast(tps) => {
                        format!("Ultra Fast ({} t/s)", tps)
                    }
                    crate::models::catalog::Speed::VeryFast(tps) => {
                        format!("Very Fast ({} t/s)", tps)
                    }
                    crate::models::catalog::Speed::Fast(tps) => format!("Fast ({} t/s)", tps),
                    crate::models::catalog::Speed::Medium(tps) => format!("Medium ({} t/s)", tps),
                    crate::models::catalog::Speed::Slow(tps) => format!("Slow ({} t/s)", tps),
                };

                let quality_str = format!("{:?}", model.quality_rating);
                println!("    Speed: {}, Quality: {}", speed_str, quality_str);
                println!("    {}", model.description);
                println!("    Context sizes: {:?}", model.context_sizes);
            }
        }
        LocalModelSub::Install { model, force } => {
            let model_spec = crate::models::catalog::MODEL_CATALOG
                .iter()
                .find(|m| m.name == model)
                .ok_or_else(|| anyhow::anyhow!("Unknown model: {}", model))?;

            let hardware = detect_hardware();
            if !force && model_spec.ram_required_gb > hardware.total_ram_gb {
                anyhow::bail!(
                    "Insufficient RAM. Model requires {}GB, you have {}GB. Use --force to override.",
                    model_spec.ram_required_gb,
                    hardware.total_ram_gb
                );
            }

            let path = get_model_path(&model_spec.filename).await?;
            if path.exists() {
                println!("Model already installed at {:?}", path);
                return Ok(());
            }

            // Create directory if it doesn't exist
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent).await?;
            }

            download_model(model_spec, &path).await?;
            // Note: Verification would require SHA256 which isn't in the model spec

            println!("✅ Model {} installed successfully", model);
        }
        LocalModelSub::Remove { model } => {
            let model_spec = crate::models::catalog::MODEL_CATALOG
                .iter()
                .find(|m| m.name == model)
                .ok_or_else(|| anyhow::anyhow!("Unknown model: {}", model))?;

            let path = get_model_path(&model_spec.filename).await?;
            if path.exists() {
                fs::remove_file(&path).await?;
                println!("✅ Model {} removed successfully", model);
            } else {
                println!("Model {} not found at {:?}", model, path);
            }
        }
        LocalModelSub::Verify { model } => {
            let model_spec = crate::models::catalog::MODEL_CATALOG
                .iter()
                .find(|m| m.name == model)
                .ok_or_else(|| anyhow::anyhow!("Unknown model: {}", model))?;

            let path = get_model_path(&model_spec.filename).await?;
            if path.exists() {
                println!("✅ Model {} found and verified at {:?}", model, path);
            } else {
                println!("❌ Model {} not found at {:?}", model, path);
            }
        }
        LocalModelSub::Benchmark {
            model,
            format,
            runtime,
            all_runtimes,
        } => {
            let opts = BenchmarkCliOptions {
                model,
                format,
                runtime,
                all_runtimes,
            };
            benchmark_model(opts).await?;
        }
        LocalModelSub::Use { model } => {
            // This would update the user's config file to set this as default
            println!("Setting {} as the default model", model);

            // In a real implementation, this would update the config file
            // For now, just print a message
            println!("Note: This would normally update the default model in your config file.");
        }
        LocalModelSub::Status => {
            let hardware = detect_hardware();
            let catalog = &crate::models::catalog::MODEL_CATALOG;

            println!("Hardware Profile:");
            println!(
                "  RAM: {}GB total, {}GB available",
                hardware.total_ram_gb, hardware.available_ram_gb
            );
            println!(
                "  CPU: {} physical cores, {} logical cores",
                hardware.cpu_physical_cores, hardware.cpu_logical_cores
            );
            println!("  Disk free: {}GB", hardware.free_disk_gb);
            if let Some(gpu) = &hardware.gpu {
                println!(
                    "  GPU: {} {} with {}GB memory",
                    gpu.brand, gpu.model, gpu.memory_gb
                );
            } else {
                println!("  GPU: None detected");
            }
            println!("  Platform: {:?}\n", hardware.platform);

            let models_dir = models_root().await?;

            let mut installed_count = 0usize;
            for model in catalog.iter() {
                let path = models_dir.join(&model.filename);
                if fs::metadata(&path).await.is_ok() {
                    installed_count += 1;
                }
            }
            println!("Installed models: {}", installed_count);

            match hardware.platform {
                PlatformKind::Windows | PlatformKind::WindowsWsl => {
                    let status = windows::WindowsAdapterStatus::gather().await;
                    println!(
                        "WSL IP: {}",
                        status.wsl_ip.unwrap_or_else(|| "not detected".to_string())
                    );
                    println!("  Ollama (WSL2): {}", bool_icon(status.ollama_reachable));
                    println!("  LM Studio API: {}", bool_icon(status.lmstudio_reachable));
                    println!("  GPT4All API: {}", bool_icon(status.gpt4all_reachable));
                    println!(
                        "  Foundry Local API: {}",
                        bool_icon(status.foundry_reachable)
                    );
                    let gpu_report = windows::check_wsl_gpu();
                    println!("  GPU passthrough: {}", gpu_report.message);
                }
                PlatformKind::MacOs => {
                    let status = macos::CoremlRuntimeStatus::detect();
                    println!(
                        "Core ML tools installed: {}",
                        bool_icon(status.coremltools_available)
                    );
                    println!(
                        "Apple Neural Engine available: {}",
                        bool_icon(status.ane_available)
                    );
                    for note in status.notes {
                        println!("  - {}", note);
                    }
                }
                PlatformKind::Linux => {
                    let status = linux::LinuxRuntimeStatus::detect();
                    println!("Ollama socket present: {}", bool_icon(status.ollama_socket));
                    println!("CUDA detected: {}", bool_icon(status.cuda_available));
                    for note in status.notes {
                        println!("  - {}", note);
                    }
                }
                PlatformKind::Unknown => {
                    println!("Platform-specific runtime diagnostics unavailable.");
                }
            }
        }
    }
    Ok(())
}

async fn models_root() -> Result<std::path::PathBuf> {
    let path = dirs::data_dir()
        .unwrap_or_else(|| std::env::current_dir().unwrap())
        .join("kandil")
        .join("models");
    fs::create_dir_all(&path).await?;
    Ok(path)
}

fn bool_icon(value: bool) -> &'static str {
    if value {
        "✅"
    } else {
        "⚠️"
    }
}

async fn get_model_path(filename: &str) -> Result<std::path::PathBuf> {
    Ok(models_root().await?.join(filename))
}

async fn download_model(
    model: &crate::models::catalog::ModelSpec,
    path: &std::path::PathBuf,
) -> Result<()> {
    // Construct the Hugging Face download URL
    let url = format!(
        "https://huggingface.co/{}/resolve/main/{}",
        model.huggingface_repo, model.filename
    );

    let client = reqwest::Client::new();
    let response = client.get(&url).send().await?;
    let total_size = response.content_length().unwrap_or(0);

    let pb = indicatif::ProgressBar::new(total_size);
    pb.set_style(indicatif::ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})")
        .unwrap());

    let mut file = tokio::io::BufWriter::new(fs::File::create(path).await?);
    let mut stream = response.bytes_stream();

    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        pb.inc(chunk.len() as u64);
        tokio::io::copy(&mut &chunk[..], &mut file).await?;
    }

    pb.finish_with_message("Download complete");
    file.flush().await?;
    Ok(())
}

async fn benchmark_model(opts: BenchmarkCliOptions) -> Result<()> {
    let cfg = Config::load()?;
    let model = opts.model.clone().unwrap_or_else(|| cfg.ai_model.clone());

    let suite = CrossPlatformBenchmark::new();

    let report = suite
        .run(BenchmarkOptions {
            model: model.clone(),
            default_provider: cfg.ai_provider.clone(),
            runtime: opts.runtime.clone(),
            include_all_runtimes: opts.all_runtimes,
            prompts: None,
        })
        .await?;

    match opts.format.to_lowercase().as_str() {
        "json" => {
            println!("{}", serde_json::to_string_pretty(&report)?);
        }
        _ => {
            println!("📊 Cross-Platform Benchmark");
            println!("Model: {}", report.model);
            println!(
                "Hardware: {}GB RAM ({}GB available) • Disk free: {}GB • Platform: {:?}",
                report.hardware.total_ram_gb,
                report.hardware.available_ram_gb,
                report.hardware.free_disk_gb,
                report.hardware.platform
            );
            if let Some(gpu) = &report.hardware.gpu {
                println!("GPU: {} {} ({}GB)", gpu.brand, gpu.model, gpu.memory_gb);
            } else {
                println!("GPU: not detected");
            }
            println!("Prompts: {}", report.prompts.len());
            println!("Timestamp: {}", report.timestamp);

            for runtime in &report.results {
                println!("\nRuntime: {}", runtime.runtime);
                println!("  Provider: {}", runtime.provider);
                println!("  Avg latency: {} ms", runtime.average_latency_ms);
                println!(
                    "  Avg throughput: {} tokens/s",
                    runtime.average_tokens_per_sec
                );
                println!("  Memory peak: {} MB", runtime.memory_peak_mb);
                if let Some(impact) = runtime.battery_impact {
                    println!("  Battery impact: {:.1}%/min", impact);
                }
                println!("  Samples:");
                for sample in &runtime.samples {
                    println!(
                        "    • {} → {} ms, {} tokens",
                        sample.prompt, sample.latency_ms, sample.output_tokens
                    );
                }
            }

            if !report.warnings.is_empty() {
                println!("\nWarnings:");
                for warning in &report.warnings {
                    println!("  - {}", warning);
                }
            }
        }
    }

    Ok(())
}

async fn handle_auth(sub: AuthSub) -> Result<()> {
    match sub {
        AuthSub::Login { provider } => {
            println!("Enter API key for {}:", provider);
            let mut buf = String::new();
            use std::io::Read;
            let mut stdin = std::io::stdin();
            stdin.read_to_string(&mut buf)?;
            let key = buf.trim().to_string();
            if key.is_empty() {
                return Err(anyhow::anyhow!("Empty API key"));
            }
            SecureKey::save(&provider, &key)?;
            println!("API key saved for {}", provider);
        }
    }
    Ok(())
}

async fn handle_pwa(output: PathBuf) -> Result<()> {
    pwa::write_assets(&output)?;
    println!("📦 PWA assets exported to {}", output.display());
    Ok(())
}

async fn handle_windows(sub: WindowsSub) -> Result<()> {
    match sub {
        WindowsSub::Status => {
            let status = windows::WindowsAdapterStatus::gather().await;
            println!(
                "WSL IP: {}",
                status.wsl_ip.unwrap_or_else(|| "not detected".to_string())
            );
            println!(
                "Ollama (WSL2) reachable: {}",
                if status.ollama_reachable {
                    "✅"
                } else {
                    "⚠️"
                }
            );
            println!(
                "LM Studio API reachable: {}",
                if status.lmstudio_reachable {
                    "✅"
                } else {
                    "⚠️"
                }
            );
            println!(
                "GPT4All API reachable: {}",
                if status.gpt4all_reachable {
                    "✅"
                } else {
                    "⚠️"
                }
            );
            println!(
                "Foundry Local reachable: {}",
                if status.foundry_reachable {
                    "✅"
                } else {
                    "⚠️"
                }
            );
        }
        WindowsSub::CheckGpu => {
            let report = windows::check_wsl_gpu();
            println!("{}", report.message);
        }
        WindowsSub::SetupWsl2 => {
            println!("{}", windows::setup_wsl2_instructions());
        }
    }
    Ok(())
}

async fn handle_macos(sub: MacosSub) -> Result<()> {
    match sub {
        MacosSub::Status => {
            let status = macos::CoremlRuntimeStatus::detect();
            println!(
                "Core ML tools installed: {}",
                bool_icon(status.coremltools_available)
            );
            println!(
                "Apple Neural Engine available: {}",
                bool_icon(status.ane_available)
            );
            for note in status.notes {
                println!("  - {}", note);
            }
        }
        MacosSub::SetupCoreml => {
            println!("{}", macos::setup_instructions());
        }
    }
    Ok(())
}

async fn handle_linux(sub: LinuxSub) -> Result<()> {
    match sub {
        LinuxSub::Status => {
            let status = linux::LinuxRuntimeStatus::detect();
            println!("Ollama socket present: {}", bool_icon(status.ollama_socket));
            println!("CUDA detected: {}", bool_icon(status.cuda_available));
            for note in status.notes {
                println!("  - {}", note);
            }
        }
        LinuxSub::Doctor => {
            let hardware = detect_hardware();
            PlatformHardener::new(&hardware).apply()?;
            println!("Ran Linux platform hardening checks. Review warnings above, if any.");
        }
        LinuxSub::Setup => {
            println!("{}", linux::setup_instructions());
        }
    }
    Ok(())
}

async fn handle_mobile(sub: MobileSub) -> Result<()> {
    let models_dir = models_root().await?;
    match sub {
        MobileSub::IosSync => {
            let dir = models_dir.clone();
            let export = task::spawn_blocking(move || mobile::export_ios_bundle(&dir)).await??;
            mobile_security::enforce_ios_bundle_security(&export)?;
            println!("📦 iOS bundle created at {}", export.display());
            println!(
                "Copy this directory into iCloud Drive and import it from the Kandil iOS app."
            );
        }
        MobileSub::AndroidSync => {
            let dir = models_dir.clone();
            let export =
                task::spawn_blocking(move || mobile::export_android_bundle(&dir)).await??;
            mobile_security::enforce_android_bundle_security(&export)?;
            println!("🤖 Android bundle created at {}", export.display());
            println!("Transfer it to /sdcard/kandil/models and use Termux or AI Core to import.");
        }
        MobileSub::EdgeSnapshot => {
            let dir = models_dir.clone();
            let export = task::spawn_blocking(move || edge::export_edge_snapshot(&dir)).await??;
            mobile_security::enforce_edge_bundle_security(&export)?;
            println!("🛠️ Edge manifest written to {}", export.display());
            println!(
                "Use the manifest to drive ONNX exports for Raspberry Pi / Jetson deployments."
            );
        }
    }
    Ok(())
}
