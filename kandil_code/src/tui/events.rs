//! Event handling for the TUI
//! 
//! Contains keyboard, mouse, and application events

use crossterm::event::{self, Event as CEvent, KeyCode, KeyEvent, KeyModifiers, MouseEvent};
use std::time::Duration;
use anyhow::Result;

#[derive(Debug)]
pub enum AppEvent {
    Key(KeyEvent),
    Mouse(MouseEvent),
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
        let tick_rate = self.tick_rate;
        
        // Use select to wait for either an input event or a timeout
        let event = tokio::select! {
            crossterm_event = tokio::task::spawn_blocking(move || {
                event::read()
            }) => {
                match crossterm_event? {
                    CEvent::Key(key) => AppEvent::Key(key),
                    CEvent::Mouse(mouse) => AppEvent::Mouse(mouse),
                    _ => return Err(anyhow::anyhow!("Unsupported event type")),
                }
            }
            _ = tokio::time::sleep(tick_rate) => AppEvent::Tick,
        };

        Ok(event)
    }
}