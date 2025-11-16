//! Custom widgets for the TUI
//!
//! Contains specialized UI components for the studio

use ratatui::{
    prelude::*,
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, StatefulWidget, Widget, Wrap},
};

#[derive(Clone)]
pub struct FileExplorer {
    items: Vec<ListItem<'static>>,
    selected: usize,
}

impl FileExplorer {
    pub fn new(files: Vec<String>) -> Self {
        let items: Vec<ListItem> = files
            .iter()
            .map(|f| ListItem::new(vec![Line::from(f.clone())]))
            .collect();

        Self { items, selected: 0 }
    }

    pub fn next(&mut self) {
        if !self.items.is_empty() {
            self.selected = (self.selected + 1) % self.items.len();
        }
    }

    pub fn previous(&mut self) {
        if !self.items.is_empty() {
            self.selected = (self.selected + self.items.len() - 1) % self.items.len();
        }
    }
}

impl Widget for FileExplorer {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::default()
            .borders(Borders::ALL)
            .title("File Explorer");

        let list = List::new(self.items)
            .block(block)
            .highlight_style(
                Style::default()
                    .bg(Color::LightBlue)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol("> ");

        StatefulWidget::render(
            list,
            area,
            buf,
            &mut ListState::default().with_selected(Some(self.selected)),
        );
    }
}

#[derive(Clone)]
pub struct CodeViewer {
    content: String,
}

impl CodeViewer {
    pub fn new(content: &str) -> Self {
        Self {
            content: content.to_string(),
        }
    }
}

impl Widget for CodeViewer {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::default().borders(Borders::ALL).title("Code Viewer");

        let paragraph = Paragraph::new(self.content)
            .block(block)
            .wrap(Wrap { trim: true });

        paragraph.render(area, buf);
    }
}

#[derive(Clone)]
pub struct AIChatWidget {
    messages: Vec<String>,
    input: String,
}

impl AIChatWidget {
    pub fn new() -> Self {
        Self {
            messages: vec![],
            input: String::new(),
        }
    }

    pub fn add_message(&mut self, msg: String) {
        self.messages.push(msg);
    }

    pub fn update_input(&mut self, input: String) {
        self.input = input;
    }
}

impl Widget for AIChatWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::default().borders(Borders::ALL).title("AI Chat");

        let text: Vec<Line> = self
            .messages
            .iter()
            .map(|msg| Line::from(msg.as_str()))
            .collect();

        Paragraph::new(text)
            .block(block)
            .wrap(Wrap { trim: true })
            .render(area, buf);
    }
}
