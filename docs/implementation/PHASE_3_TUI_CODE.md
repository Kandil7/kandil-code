# 📄 PHASE_3_TUI_CODE.md

```markdown
# Phase 3: TUI Studio & Code Understanding

## Objectives
Build an interactive Terminal UI (TUI) using Ratatui for visual project management, and integrate Tree-sitter for code analysis. Enable `kandil tui` to launch a functional studio with file navigation, code preview, and AI chat panel.

## Prerequisites
- Phase 2 complete (templates and plugin system)
- Ollama running with llama3:70b
- Terminal that supports Unicode (iTerm2, Windows Terminal, GNOME Terminal)
- Basic understanding of terminal UI concepts

## Detailed Sub-Tasks

### Day 1-2: TUI Foundation & Layout

1. **Add Dependencies**
```bash
cargo add ratatui --features all-widgets
cargo add crossterm --features event-stream
cargo add syntect # Syntax highlighting
cargo add tree-sitter
cargo add tree-sitter-dart tree-sitter-python tree-sitter-javascript tree-sitter-rust
cargo add tui-textarea # For chat input
```

2. **Create TUI Application Structure**
```rust
// src/tui/mod.rs
pub mod studio;
pub mod widgets; // Custom widgets
pub mod events;  // Event handling

pub use studio::StudioApp;

// src/tui/events.rs
use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use std::time::Duration;
use anyhow::Result;

pub enum AppEvent {
    Key(crossterm::event::KeyEvent),
    Mouse(crossterm::event::MouseEvent),
    Tick,
}

pub struct EventHandler {
    tick_rate: Duration,
}

impl EventHandler {
    pub fn new(tick_rate_ms: u64) -> Self {
        Self {
            tick_rate: Duration::from_millis(tick_rate_ms),
        }
    }
    
    pub async fn next(&self) -> Result<AppEvent> {
        if event::poll(self.tick_rate)? {
            match event::read()? {
                Event::Key(key) => {
                    // Handle Ctrl+C
                    if key.code == KeyCode::Char('c') 
                        && key.modifiers.contains(KeyModifiers::CONTROL) {
                        return Ok(AppEvent::Key(key));
                    }
                    Ok(AppEvent::Key(key))
                },
                Event::Mouse(mouse) => Ok(AppEvent::Mouse(mouse)),
                _ => self.next().await,
            }
        } else {
            Ok(AppEvent::Tick)
        }
    }
}
```

3. **Main Studio Layout**
```rust
// src/tui/studio.rs
use ratatui::backend::Backend;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph};
use ratatui::Frame;
use crossterm::event::KeyCode;
use anyhow::Result;

pub enum Panel {
    FileTree,
    CodePreview,
    Chat,
    AiPanel,
}

pub struct StudioApp {
    pub file_tree: Vec<String>,
    pub selected_file: usize,
    pub code_content: String,
    pub chat_history: Vec<(String, String)>, // (sender, message)
    pub chat_input: String,
    pub active_panel: Panel,
    pub workspace: crate::core::workspace::Workspace,
    pub should_quit: bool,
}

impl StudioApp {
    pub fn new() -> Result<Self> {
        let workspace = crate::core::workspace::Workspace::detect()?;
        
        Ok(Self {
            file_tree: Self::load_files(&workspace)?,
            selected_file: 0,
            code_content: String::new(),
            chat_history: vec![],
            chat_input: String::new(),
            active_panel: Panel::FileTree,
            workspace,
            should_quit: false,
        })
    }
    
    fn load_files(workspace: &crate::core::workspace::Workspace) -> Result<Vec<String>> {
        let mut files = Vec::new();
        
        // Only load first level for performance
        if let Ok(entries) = std::fs::read_dir(&workspace.root) {
            for entry in entries.flatten().take(100) {
                if let Some(name) = entry.file_name().to_str() {
                    files.push(name.to_string());
                }
            }
        }
        
        Ok(files)
    }
    
    pub async fn run<B: Backend>(mut self, terminal: &mut ratatui::Terminal<B>) -> Result<()> {
        use crate::tui::events::{EventHandler, AppEvent};
        
        let events = EventHandler::new(250); // 250ms tick rate
        
        while !self.should_quit {
            terminal.draw(|f| self.draw(f))?;
            
            match events.next().await? {
                AppEvent::Key(key) => self.handle_key(key).await?,
                AppEvent::Tick => self.on_tick().await?,
                _ => {}
            }
        }
        
        Ok(())
    }
    
    fn draw(&mut self, f: &mut Frame) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(25),  // File tree
                Constraint::Percentage(50),  // Code preview
                Constraint::Percentage(25),  // AI/Chat
            ])
            .split(f.size());
        
        self.draw_file_tree(f, chunks[0]);
        self.draw_code_preview(f, chunks[1]);
        self.draw_chat(f, chunks[2]);
    }
    
    fn draw_file_tree(&self, f: &mut Frame, area: Rect) {
        let items: Vec<ListItem> = self.file_tree
            .iter()
            .enumerate()
            .map(|(i, name)| {
                let style = if i == self.selected_file {
                    Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                };
                ListItem::new(name.as_str()).style(style)
            })
            .collect();
        
        let list = List::new(items)
            .block(Block::default()
                .borders(Borders::ALL)
                .title("📁 Files")
                .border_style(Style::default().fg(Color::Cyan)));
        
        f.render_widget(list, area);
    }
    
    fn draw_code_preview(&self, f: &mut Frame, area: Rect) {
        let block = Block::default()
            .borders(Borders::ALL)
            .title("📝 Code Preview")
            .border_style(Style::default().fg(Color::Green));
        
        let content = if self.code_content.is_empty() {
            "Select a file to preview".to_string()
        } else {
            self.code_content.clone()
        };
        
        let paragraph = Paragraph::new(content)
            .block(block)
            .wrap(ratatui::widgets::Wrap { trim: true });
        
        f.render_widget(paragraph, area);
    }
    
    fn draw_chat(&self, f: &mut Frame, area: Rect) {
        let block = Block::default()
            .borders(Borders::ALL)
            .title("💬 AI Assistant")
            .border_style(Style::default().fg(Color::Blue));
        
        let chat_text = self.chat_history
            .iter()
            .map(|(sender, msg)| format!("{}: {}\n", sender, msg))
            .collect::<String>();
        
        let paragraph = Paragraph::new(chat_text)
            .block(block);
        
        f.render_widget(paragraph, area);
    }
    
    async fn handle_key(&mut self, key: crossterm::event::KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Char('q') if key.modifiers.contains(crossterm::event::KeyModifiers::CONTROL) => {
                self.should_quit = true;
            }
            KeyCode::Up => self.navigate_up(),
            KeyCode::Down => self.navigate_down(),
            KeyCode::Enter => self.select_file().await?,
            KeyCode::Char('c') if matches!(self.active_panel, Panel::Chat) => {
                self.send_chat_message().await?;
            }
            KeyCode::Char(c) if matches!(self.active_panel, Panel::Chat) => {
                self.chat_input.push(c);
            }
            KeyCode::Backspace if matches!(self.active_panel, Panel::Chat) => {
                self.chat_input.pop();
            }
            _ => {}
        }
        Ok(())
    }
    
    fn navigate_up(&mut self) {
        if self.selected_file > 0 {
            self.selected_file -= 1;
        }
    }
    
    fn navigate_down(&mut self) {
        if self.selected_file < self.file_tree.len().saturating_sub(1) {
            self.selected_file += 1;
        }
    }
    
    async fn select_file(&mut self) -> Result<()> {
        if let Some(file_name) = self.file_tree.get(self.selected_file) {
            let path = format!("{}/{}", self.workspace.root, file_name);
            if std::path::Path::new(&path).is_file() {
                self.code_content = tokio::fs::read_to_string(&path).await?;
                
                // Auto-detect language and analyze
                let ext = file_name.split('.').last().unwrap_or("");
                self.analyze_code(&self.code_content, ext).await?;
            }
        }
        Ok(())
    }
    
    async fn analyze_code(&mut self, code: &str, ext: &str) -> Result<()> {
        use crate::adapters::ai::factory::AIProviderFactory;
        use crate::utils::config::Config;
        
        let config = Config::load()?;
        let factory = AIProviderFactory::new(config.ai);
        let ai = factory.create().await?;
        
        let prompt = format!(
            "Analyze this {} code for issues and best practices:\n{}",
            ext, &code[..code.len().min(2000)] // Limit to avoid token overflow
        );
        
        let analysis = ai.chat(&prompt, None).await?;
        self.chat_history.push(("AI Analyst".to_string(), analysis));
        
        Ok(())
    }
    
    async fn send_chat_message(&mut self) -> Result<()> {
        if !self.chat_input.is_empty() {
            let message = self.chat_input.clone();
            self.chat_history.push(("You".to_string(), message.clone()));
            self.chat_input.clear();
            
            // Send to AI
            use crate::adapters::ai::factory::AIProviderFactory;
            use crate::utils::config::Config;
            
            let config = Config::load()?;
            let factory = AIProviderFactory::new(config.ai);
            let ai = factory.create().await?;
            
            let response = ai.chat(&message, None).await?;
            self.chat_history.push(("AI".to_string(), response));
        }
        Ok(())
    }
    
    async fn on_tick(&mut self) -> Result<()> {
        // Background tasks, if any
        Ok(())
    }
}
```

### Day 3-4: Code Analyzer with Tree-sitter

1. **Language Detection & Parser Pool**
```rust
// src/code/analyzer.rs
use tree_sitter::{Parser, Language, Node};
use anyhow::Result;
use once_cell::sync::Lazy;

static PARSERS: Lazy<std::sync::Mutex<Vec<(String, Parser)>>> = Lazy::new(|| {
    std::sync::Mutex::new(Vec::new())
});

pub struct CodeAnalyzer {
    language: Option<Language>,
    extension: String,
}

impl CodeAnalyzer {
    pub fn new(extension: &str) -> Result<Self> {
        let language = match extension {
            "dart" => Some(unsafe { tree_sitter_dart() }),
            "py" => Some(unsafe { tree_sitter_python() }),
            "js" | "ts" => Some(unsafe { tree_sitter_javascript() }),
            "rs" => Some(unsafe { tree_sitter_rust() }),
            _ => None,
        };
        
        Ok(Self {
            language,
            extension: extension.to_string(),
        })
    }
    
    pub fn analyze_syntax(&mut self, code: &str) -> Result<SyntaxAnalysis> {
        let mut analysis = SyntaxAnalysis::default();
        
        if let Some(lang) = self.language {
            let mut parser = Parser::new();
            parser.set_language(lang)?;
            
            let tree = parser.parse(code, None)
                .ok_or_else(|| anyhow::anyhow!("Failed to parse code"))?;
            
            self.walk_tree(tree.root_node(), &mut analysis);
        }
        
        Ok(analysis)
    }
    
    fn walk_tree(&self, node: Node, analysis: &mut SyntaxAnalysis) {
        analysis.total_nodes += 1;
        
        match node.kind() {
            "function" | "function_definition" | "method" => {
                analysis.functions += 1;
            }
            "class" | "class_definition" => {
                analysis.classes += 1;
            }
            "comment" => {
                analysis.comments += 1;
            }
            "import" | "import_statement" | "use" => {
                analysis.imports += 1;
            }
            _ => {}
        }
        
        for child in node.children(&mut node.walk()) {
            self.walk_tree(child, analysis);
        }
    }
}

#[derive(Debug, Default)]
pub struct SyntaxAnalysis {
    pub total_nodes: usize,
    pub functions: usize,
    pub classes: usize,
    pub comments: usize,
    pub imports: usize,
}

impl SyntaxAnalysis {
    pub fn complexity_score(&self) -> f64 {
        // Simple cyclomatic complexity approximation
        (self.functions * 2 + self.classes * 3 + self.imports) as f64 / 
        (self.comments + 1) as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_analyze_dart_simple() {
        let code = r#"
        void main() {
          print('Hello');
        }
        "#;
        
        let mut analyzer = CodeAnalyzer::new("dart").unwrap();
        let analysis = analyzer.analyze_syntax(code).unwrap();
        
        assert!(analysis.functions >= 1);
    }
}
```

2. **Syntax Highlighting with Syntect**
```rust
// src/tui/syntax.rs
use syntect::easy::HighlightLines;
use syntect::highlighting::{Style, ThemeSet};
use syntect::parsing::SyntaxSet;
use syntect::util::{as_24_bit_terminal_escaped, LinesWithEndings};
use once_cell::sync::Lazy;

static SYNTAX_SET: Lazy<SyntaxSet> = Lazy::new(SyntaxSet::load_defaults_newlines);
static THEME_SET: Lazy<ThemeSet> = Lazy::new(ThemeSet::load_defaults);

pub struct SyntaxHighlighter {
    syntax_set: &'static SyntaxSet,
    theme: &'static ThemeSet,
}

impl SyntaxHighlighter {
    pub fn new() -> Self {
        Self {
            syntax_set: &SYNTAX_SET,
            theme: &THEME_SET,
        }
    }
    
    pub fn highlight(&self, code: &str, ext: &str) -> Vec<String> {
        let syntax = self.syntax_set.find_syntax_by_extension(ext)
            .unwrap_or_else(|| self.syntax_set.find_syntax_plain_text());
        
        let theme = &self.theme.themes["base16-ocean.dark"];
        let mut highlighter = HighlightLines::new(syntax, theme);
        
        let mut lines = Vec::new();
        for line in LinesWithEndings::from(code) {
            let ranges: Vec<(Style, &str)> = highlighter.highlight_line(line, self.syntax_set)
                .unwrap_or_default();
            lines.push(as_24_bit_terminal_escaped(&ranges, false));
        }
        
        lines
    }
}
```

3. **Integrate Analysis into TUI**
```rust
// In src/tui/studio.rs
fn draw_code_preview(&self, f: &mut Frame, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title("📝 Code Preview")
        .border_style(Style::default().fg(Color::Green));
    
    let text = if self.code_content.is_empty() {
        vec![Line::from("Select a file to preview")]
    } else {
        // Get file extension
        let ext = self.file_tree[self.selected_file]
            .split('.')
            .last()
            .unwrap_or("");
        
        // Highlight syntax
        let highlighter = SyntaxHighlighter::new();
        highlighter.highlight(&self.code_content, ext)
            .iter()
            .map(|line| Line::from(line.as_str()))
            .collect()
    };
    
    let paragraph = Paragraph::new(text)
        .block(block)
        .wrap(ratatui::widgets::Wrap { trim: false });
    
    f.render_widget(paragraph, area);
}
```

### Day 5-6: AI Integration in TUI

1. **AI Chat Panel**
```rust
// src/tui/chat.rs
use crate::adapters::ai::factory::AIProviderFactory;
use crate::utils::config::Config;
use anyhow::Result;

pub struct ChatPanel {
    history: Vec<(String, String)>, // (sender, message)
    input: String,
    is_typing: bool,
}

impl ChatPanel {
    pub fn new() -> Self {
        Self {
            history: vec![(
                "AI".to_string(),
                "Hello! I'm Kandil AI. Select a file and I'll analyze it.".to_string(),
            )],
            input: String::new(),
            is_typing: false,
        }
    }
    
    pub fn add_message(&mut self, sender: &str, message: String) {
        self.history.push((sender.to_string(), message));
        // Keep only last 50 messages
        if self.history.len() > 50 {
            self.history.drain(0..10);
        }
    }
    
    pub async fn send_message(&mut self) -> Result<()> {
        if self.input.is_empty() || self.is_typing {
            return Ok(());
        }
        
        let message = self.input.clone();
        self.add_message("You", message.clone());
        self.input.clear();
        self.is_typing = true;
        
        // Send to AI
        let config = Config::load()?;
        let factory = AIProviderFactory::new(config.ai);
        let ai = factory.create().await?;
        
        let response = ai.chat(&message, None).await?;
        self.add_message("AI", response);
        self.is_typing = false;
        
        Ok(())
    }
    
    pub fn handle_input(&mut self, c: char) {
        self.input.push(c);
    }
    
    pub fn backspace(&mut self) {
        self.input.pop();
    }
    
    pub fn render(&self) -> Paragraph {
        let text = self.history
            .iter()
            .map(|(sender, msg)| format!("{}: {}\n", sender, msg))
            .collect::<String>();
        
        let mut lines = text.lines().collect::<Vec<_>>();
        if self.is_typing {
            lines.push("AI is typing...");
        }
        
        Paragraph::new(lines.join("\n"))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("💬 AI Assistant")
                    .border_style(Style::default().fg(Color::Blue))
            )
    }
}
```

### Day 7-8: Performance Optimization

1. **Lazy File Loading**
```rust
// src/tui/file_tree.rs
use std::fs;
use std::path::{Path, PathBuf};
use anyhow::Result;

pub struct FileTree {
    root: PathBuf,
    tree: Vec<TreeItem>,
    selected: usize,
}

struct TreeItem {
    path: PathBuf,
    depth: usize,
    is_dir: bool,
    is_expanded: bool,
}

impl FileTree {
    pub fn new(root: PathBuf) -> Result<Self> {
        let mut tree = Vec::new();
        Self::build_tree(&root, &mut tree, 0)?;
        
        Ok(Self { root, tree, selected: 0 })
    }
    
    fn build_tree(path: &Path, tree: &mut Vec<TreeItem>, depth: usize) -> Result<()> {
        if depth > 3 { return Ok(()); } // Limit depth
        
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let path = entry.path();
            let is_dir = path.is_dir();
            
            tree.push(TreeItem {
                path: path.clone(),
                depth,
                is_dir,
                is_expanded: depth < 1, // Expand root level only
            });
            
            if is_dir && depth < 2 {
                Self::build_tree(&path, tree, depth + 1)?;
            }
        }
        Ok(())
    }
    
    pub fn render(&self) -> List {
        let items: Vec<ListItem> = self.tree
            .iter()
            .enumerate()
            .map(|(i, item)| {
                let prefix = "  ".repeat(item.depth);
                let icon = if item.is_dir {
                    if item.is_expanded { "📂" } else { "📁" }
                } else {
                    "📄"
                };
                
                let line = format!("{}{} {}", prefix, icon, item.path.file_name()
                    .unwrap_or_default()
                    .to_string_lossy());
                
                let style = if i == self.selected {
                    Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                };
                
                ListItem::new(line).style(style)
            })
            .collect();
        
        List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("📁 File Explorer")
                    .border_style(Style::default().fg(Color::Cyan))
            )
    }
}
```

2. **Async File Operations**
```rust
// Use tokio::fs for non-blocking I/O
async fn select_file(&mut self) -> Result<()> {
    if let Some(item) = self.file_tree.tree.get(self.file_tree.selected) {
        if !item.is_dir {
            self.code_content = tokio::fs::read_to_string(&item.path).await?;
            
            // Update file type
            self.workspace.project_type = self.detect_language_from_path(&item.path);
        }
    }
    Ok(())
}

fn detect_language_from_path(&self, path: &Path) -> String {
    match path.extension().and_then(|s| s.to_str()) {
        Some("dart") => "flutter".to_string(),
        Some("py") => "python".to_string(),
        Some("js" | "ts" | "jsx" | "tsx") => "js".to_string(),
        Some("rs") => "rust".to_string(),
        _ => "unknown".to_string(),
    }
}
```

### Day 9-10: Code Actions & AI Commands

1. **In-TUI AI Commands**
```rust
enum AiCommand {
    ExplainCode,
    GenerateTest,
    Refactor,
    FindBugs,
}

impl StudioApp {
    pub fn show_ai_menu(&mut self) {
        self.chat_history.push((
            "System".to_string(),
            "AI Commands: [e]xplain, [t]est, [r]efactor, [b]ugs".to_string(),
        ));
    }
    
    async fn execute_ai_command(&mut self, cmd: AiCommand) -> Result<()> {
        use AiCommand::*;
        
        if self.code_content.is_empty() {
            self.chat_history.push((
                "AI".to_string(),
                "No code selected. Navigate to a file first.".to_string(),
            ));
            return Ok(());
        }
        
        let prompt = match cmd {
            ExplainCode => format!("Explain this code:\n{}", &self.code_content[..1000]),
            GenerateTest => format!("Generate unit tests for:\n{}", &self.code_content[..1000]),
            Refactor => "Refactor this code to be more idiomatic".to_string(),
            FindBugs => "Find potential bugs and security issues".to_string(),
        };
        
        self.chat_history.push(("You".to_string(), format!("AI command: {:?}", cmd)));
        
        let config = crate::utils::config::Config::load()?;
        let factory = crate::adapters::ai::factory::AIProviderFactory::new(config.ai);
        let ai = factory.create().await?;
        
        let response = ai.chat(&prompt, None).await?;
        self.chat_history.push(("AI".to_string(), response));
        
        Ok(())
    }
}
```

### Day 11-14: Integration & Testing

1. **TUI Entry Point**
```rust
// src/cli/tui.rs
use crate::tui::studio::StudioApp;
use anyhow::Result;
use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};

pub async fn run_tui() -> Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    
    // Run app
    let app = StudioApp::new()?;
    let res = app.run(&mut terminal).await;
    
    // Cleanup
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    
    res
}
```

2. **Integration Tests**
```rust
// tests/integration/tui_test.rs
use kandil_code::tui::studio::StudioApp;
use ratatui::backend::TestBackend;
use ratatui::Terminal;

#[tokio::test]
async fn test_tui_creation() {
    let app = StudioApp::new().unwrap();
    assert!(!app.should_quit);
    assert!(!app.file_tree.is_empty());
}

#[test]
fn test_tui_render() {
    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).unwrap();
    
    let mut app = StudioApp::new().unwrap();
    terminal.draw(|f| app.draw(f)).unwrap();
    
    // Verify buffer is not empty
    let buffer = terminal.backend().buffer();
    assert!(!buffer.content.is_empty());
}
```

3. **Performance Benchmarks**
```rust
// benches/tui_render.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use kandil_code::tui::studio::StudioApp;
use ratatui::backend::TestBackend;
use ratatui::Terminal;

fn bench_tui_render(c: &mut Criterion) {
    c.bench_function("tui_render", |b| {
        let mut terminal = Terminal::new(TestBackend::new(120, 40)).unwrap();
        let mut app = StudioApp::new().unwrap();
        
        b.iter(|| {
            terminal.draw(|f| app.draw(black_box(f))).unwrap();
        });
    });
}

criterion_group!(benches, bench_tui_render);
criterion_main!(benches);
```

## Tools & Dependencies
| Crate | Version | Purpose |
|-------|---------|---------|
| ratatui | 0.25 | TUI framework |
| crossterm | 0.27 | Cross-platform terminal ops |
| syntect | 5.2 | Syntax highlighting |
| tree-sitter | 0.20 | Code parsing |
| tui-textarea | 0.4 | Text input widget |
| once_cell | 1.19 | Lazy static parsers |
| criterion | 0.5 | Performance benchmarks |

## Testing Strategy
- **Unit**: Individual widget rendering (85% coverage)
- **Integration**: Full TUI lifecycle (init → render → cleanup)
- **Manual**: Test navigation on projects with 1000+ files
- **Performance**: Render time <16ms (60 FPS target)

## Deliverables
- ✅ `kandil tui` launches interactive interface
- ✅ File tree with lazy loading (100ms for 1000 files)
- ✅ Syntax highlighting for 4 languages
- ✅ AI chat panel with context awareness
- ✅ Code analysis integration
- ✅ Keyboard navigation (vim-style hjkl + arrows)
- ✅ Clean terminal restore on exit (no garbled output)
- ✅ 85% test coverage on TUI modules

## Timeline Breakdown
- **Days 1-3**: TUI layout + event system
- **Days 4-5**: Tree-sitter integration
- **Days 6-7**: Syntax highlighting
- **Days 8-9**: AI chat panel
- **Days 10-11**: Performance optimization
- **Days 12-14**: Testing & polish

## Success Criteria
- TUI renders in <100ms on startup
- File navigation at 60 FPS
- Syntax highlighting accurate for 90% of tokens
- AI responses appear in chat panel within 2s
- No memory leaks (check with `valgrind`)
- CI passes on Windows/macOS/Linux terminals
- `cargo tarpaulin` shows ≥85% coverage

## Potential Risks & Mitigations

| Risk | Mitigation |
|------|------------|
| Terminal incompatibility | Test on 5+ terminals; fallback to basic mode |
| Large files crash TUI | Limit preview to first 10,000 lines |
| Memory leak in event loop | Use `std::mem::drop` explicitly; run `valgrind` |
| AI response delays UI | Move AI calls to separate tokio task |
| Tree-sitter parse failures | Fallback to plain text; log errors |
| `crossterm` panic on resize | Add signal handler for SIGWINCH |

---

**Next**: Proceed to PHASE_4_REFACTOR_TESTS_MODELS.md after Phase 3 manual testing with real Flutter/Python projects.