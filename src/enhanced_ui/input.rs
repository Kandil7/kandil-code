use anyhow::Result;
use rustyline::{error::ReadlineError, DefaultEditor};

pub enum InputMethod {
    Text(String),
    Voice(String),
    Image(String),
}

pub struct UniversalInput {
    editor: DefaultEditor,
    voice_enabled: bool,
    vision_enabled: bool,
}

impl UniversalInput {
    pub fn new() -> Result<Self> {
        Ok(Self {
            editor: DefaultEditor::new()?,
            voice_enabled: false,
            vision_enabled: false,
        })
    }

    pub fn enable_voice(&mut self, enabled: bool) {
        self.voice_enabled = enabled;
    }

    pub fn enable_vision(&mut self, enabled: bool) {
        self.vision_enabled = enabled;
    }

    pub fn read(&mut self, prompt: &str) -> Result<InputMethod> {
        // Voice/vision hooks would go here. For now default to text.
        match self.editor.readline(prompt) {
            Ok(line) => Ok(InputMethod::Text(line)),
            Err(ReadlineError::Interrupted) => Ok(InputMethod::Text(String::new())),
            Err(ReadlineError::Eof) => Ok(InputMethod::Text("exit".into())),
            Err(err) => Err(err.into()),
        }
    }

    pub fn add_history(&mut self, entry: &str) -> Result<()> {
        self.editor.add_history_entry(entry)?;
        Ok(())
    }
}
