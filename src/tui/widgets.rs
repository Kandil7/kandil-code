//! Custom widgets for the TUI
//! 
//! Contains specialized UI components for the studio

use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph, List, ListItem},
};

pub struct FileExplorer<'a> {
    items: Vec<ListItem<'a>>,
    selected: usize,
}

impl<'a> FileExplorer<'a> {
    pub fn new(files: Vec<String>) -> Self {
        let items: Vec<ListItem> = files
            .iter()
            .map(|f| ListItem::new(f.as_str()))
            .collect();

        Self {
            items,
            selected: 0,
        }
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

impl<'a> Widget for FileExplorer<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::default()
            .borders(Borders::ALL)
            .title("File Explorer");
        
        let list = List::new(self.items)
            .block(block)
            .highlight_style(Style::default().bg(Color::LightBlue).add_modifier(Modifier::BOLD))
            .highlight_symbol("> ");
        
        StatefulWidget::render(
            list,
            area,
            buf,
            &mut ListState::default().with_selected(Some(self.selected)),
        );
    }
}

pub struct CodeViewer<'a> {
    content: &'a str,
}

impl<'a> CodeViewer<'a> {
    pub fn new(content: &'a str) -> Self {
        Self { content }
    }
}

impl<'a> Widget for CodeViewer<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::default()
            .borders(Borders::ALL)
            .title("Code Viewer");
        
        let paragraph = Paragraph::new(self.content)
            .block(block)
            .wrap(Wrap { trim: true });
        
        paragraph.render(area, buf);
    }
}

pub struct AIChatWidget<'a> {
    messages: Vec<String>,
    input: String,
}

impl<'a> AIChatWidget<'a> {
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

impl<'a> Widget for AIChatWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::default()
            .borders(Borders::ALL)
            .title("AI Chat");
        
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