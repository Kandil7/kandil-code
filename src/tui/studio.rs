//! Studio application for the TUI interface
//!
//! Main application state and event loop

use crate::enhanced_ui::gpu_render::{should_use_gpu, GpuRenderer};
use crate::enhanced_ui::terminal::KandilTerminal;
use crate::tui::events::{AppEvent, EventHandler};
use crate::tui::widgets::{AIChatWidget, CodeViewer, FileExplorer};
use crate::utils::code_analysis::CodeAnalyzer;
use anyhow::Result;
use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    Frame, Terminal,
};
use std::io;
use std::sync::Arc;

#[derive(Clone)]
pub enum UIState {
    FileExplorer,
    CodeViewer,
    AIChat,
}

pub struct StudioApp {
    pub ui_state: UIState,
    pub file_explorer: FileExplorer,
    pub code_viewer: CodeViewer,
    pub ai_chat: AIChatWidget,
    pub code_analyzer: CodeAnalyzer,
    pub should_quit: bool,
    #[cfg(feature = "gpu-rendering")]
    gpu_renderer: Option<GpuRenderer>,
    terminal: Arc<KandilTerminal>,
}

impl StudioApp {
    pub fn new() -> Result<Self> {
        // For now, using static content - in reality, we'd load from the actual project
        let files = vec![
            "main.rs".to_string(),
            "Cargo.toml".to_string(),
            "src/lib.rs".to_string(),
        ];

        let code_content =
            "// Sample code content\nfn main() {\n    println!(\"Hello, Kandil!\");\n}";

        let terminal = Arc::new(KandilTerminal::new()?);
        
        // Initialize GPU renderer if available
        #[cfg(feature = "gpu-rendering")]
        let gpu_renderer = if should_use_gpu() {
            match GpuRenderer::new() {
                Ok(renderer) => {
                    println!("🚀 GPU rendering enabled");
                    Some(renderer)
                }
                Err(e) => {
                    eprintln!("⚠️  GPU rendering unavailable: {}", e);
                    None
                }
            }
        } else {
            None
        };

        Ok(Self {
            ui_state: UIState::FileExplorer,
            file_explorer: FileExplorer::new(files),
            code_viewer: CodeViewer::new(code_content),
            ai_chat: AIChatWidget::new(),
            code_analyzer: CodeAnalyzer::new()?,
            should_quit: false,
            #[cfg(feature = "gpu-rendering")]
            gpu_renderer,
            terminal,
        })
    }

    pub async fn run(&mut self) -> Result<()> {
        // Setup terminal
        let backend = CrosstermBackend::new(io::stdout());
        let mut terminal = Terminal::new(backend)?;

        // Enter raw mode and alternate screen
        enable_raw_mode()?;
        execute!(io::stdout(), EnterAlternateScreen)?;

        // Create event handler
        let events = EventHandler::new(250); // 250ms tick rate

        // Main loop
        loop {
            terminal.draw(|f| self.ui(f))?;

            // GPU rendering (if enabled)
            #[cfg(feature = "gpu-rendering")]
            if let Some(ref mut gpu) = self.gpu_renderer {
                if let Err(e) = gpu.render_frame(&self.terminal).await {
                    eprintln!("GPU render error: {}", e);
                }
            }

            match events.next().await? {
                AppEvent::Tick => {}
                AppEvent::Key(key_event) => {
                    if key_event.kind == crossterm::event::KeyEventKind::Press {
                        self.handle_key_events(key_event)?;
                    }
                }
                AppEvent::Mouse(mouse_event) => {
                    self.handle_mouse_events(mouse_event)?;
                }
            }

            if self.should_quit {
                break;
            }
        }

        // Restore terminal
        disable_raw_mode()?;
        execute!(io::stdout(), LeaveAlternateScreen)?;

        Ok(())
    }

    fn ui(&self, f: &mut Frame) {
        // Create layout
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(20), // File Explorer
                Constraint::Percentage(60), // Code Viewer
                Constraint::Percentage(20), // AI Chat
            ])
            .split(f.size());

        // Render widgets
        f.render_widget(self.file_explorer.clone(), chunks[0]);
        f.render_widget(self.code_viewer.clone(), chunks[1]);
        f.render_widget(self.ai_chat.clone(), chunks[2]);
    }

    fn handle_key_events(&mut self, key_event: crossterm::event::KeyEvent) -> Result<()> {
        match key_event.code {
            crossterm::event::KeyCode::Char('q') | crossterm::event::KeyCode::Esc => {
                self.should_quit = true;
            }
            crossterm::event::KeyCode::Tab => {
                // Cycle between UI states
                self.cycle_ui_state();
            }
            crossterm::event::KeyCode::Down => match self.ui_state {
                UIState::FileExplorer => self.file_explorer.next(),
                _ => {}
            },
            crossterm::event::KeyCode::Up => match self.ui_state {
                UIState::FileExplorer => self.file_explorer.previous(),
                _ => {}
            },
            crossterm::event::KeyCode::Enter => {
                // For now, just simulate loading file content
                if let UIState::FileExplorer = self.ui_state {
                    self.ai_chat.add_message("File loaded!".to_string());
                }
            }
            crossterm::event::KeyCode::Char('a')
                if key_event
                    .modifiers
                    .contains(crossterm::event::KeyModifiers::CONTROL) =>
            {
                // Analyze current file with code analyzer
                self.ai_chat
                    .add_message("Analyzing file with Tree-sitter...".to_string());
            }
            _ => {}
        }
        Ok(())
    }

    fn handle_mouse_events(&mut self, _mouse_event: crossterm::event::MouseEvent) -> Result<()> {
        // Handle mouse events
        Ok(())
    }

    fn cycle_ui_state(&mut self) {
        self.ui_state = match self.ui_state.clone() {
            UIState::FileExplorer => UIState::CodeViewer,
            UIState::CodeViewer => UIState::AIChat,
            UIState::AIChat => UIState::FileExplorer,
        };
    }
}
