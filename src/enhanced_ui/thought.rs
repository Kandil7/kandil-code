use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::sync::mpsc;

#[derive(Clone, Debug)]
pub enum ThoughtFragment {
    Context(String),
    Hypothesis(String),
    Action(String),
    Result(String),
    /// Thought process for a longer operation
    Process(String),
    /// An insight or realization
    Insight(String),
    /// A question that needs to be answered
    Question(String),
}

#[derive(Clone)]
pub struct Thought {
    pub fragment: ThoughtFragment,
    pub timestamp: Instant,
    pub correlation_id: String,
}

#[derive(Clone)]
pub struct ThoughtStreamer {
    thoughts: Arc<Mutex<Vec<Thought>>>,
    max_thoughts: usize,
    output_mode: OutputMode,
    /// Channel for async thought processing
    tx: Option<tokio::sync::mpsc::UnboundedSender<Thought>>,
    rx: Option<Arc<Mutex<Option<tokio::sync::mpsc::UnboundedReceiver<Thought>>>>>,
}

#[derive(Clone)]
pub enum OutputMode {
    /// Show all thoughts immediately
    Verbose,
    /// Show only important thoughts
    Selective,
    /// Only show thoughts when explicitly requested
    Minimal,
    /// Show thoughts in a streaming format
    Streaming,
}

impl Default for OutputMode {
    fn default() -> Self {
        OutputMode::Selective
    }
}

impl Default for ThoughtStreamer {
    fn default() -> Self {
        let (tx, rx) = mpsc::unbounded_channel();
        Self {
            thoughts: Arc::new(Mutex::new(Vec::new())),
            max_thoughts: 100,
            output_mode: OutputMode::default(),
            tx: Some(tx),
            rx: Some(Arc::new(Mutex::new(Some(rx)))),
        }
    }
}

impl ThoughtStreamer {
    pub fn new() -> Self {
        Self::default()
    }

    /// Create with specific output mode
    pub fn with_output_mode(mode: OutputMode) -> Self {
        let (tx, rx) = mpsc::unbounded_channel();
        Self {
            thoughts: Arc::new(Mutex::new(Vec::new())),
            max_thoughts: 100,
            output_mode: mode,
            tx: Some(tx),
            rx: Some(Arc::new(Mutex::new(Some(rx)))),
        }
    }

    /// Emit a thought with immediate display based on output mode
    pub fn emit(&self, fragment: ThoughtFragment) {
        let thought = Thought {
            fragment: fragment.clone(),
            timestamp: Instant::now(),
            correlation_id: uuid::Uuid::new_v4().to_string(),
        };

        // Add to internal storage
        if let Ok(mut thoughts) = self.thoughts.lock() {
            thoughts.push(thought.clone());
            if thoughts.len() > self.max_thoughts {
                thoughts.remove(0);
            }
        }

        // Display based on output mode
        match &self.output_mode {
            OutputMode::Verbose => {
                self.display_thought(&thought);
            },
            OutputMode::Selective => {
                // Only display certain types of thoughts
                match &thought.fragment {
                    ThoughtFragment::Action(_) |
                    ThoughtFragment::Result(_) |
                    ThoughtFragment::Insight(_) => {
                        self.display_thought(&thought);
                    },
                    _ => {
                        // Don't display context/hypothesis in selective mode
                    }
                }
            },
            OutputMode::Minimal => {
                // Only display results
                if matches!(&thought.fragment, ThoughtFragment::Result(_)) {
                    self.display_thought(&thought);
                }
            },
            OutputMode::Streaming => {
                // Display with streaming indicators
                self.display_thought_streaming(&thought);
            }
        }

        // Send to async channel if available
        if let Some(tx) = &self.tx {
            let _ = tx.send(thought);
        }
    }

    /// Display thought in streaming format with progress indication
    fn display_thought_streaming(&self, thought: &Thought) {
        match &thought.fragment {
            ThoughtFragment::Context(msg) => println!("⏳ [{}] Context: {}", self.format_duration(thought.timestamp.elapsed()), msg),
            ThoughtFragment::Hypothesis(msg) => println!("🔍 [{}] Considering: {}", self.format_duration(thought.timestamp.elapsed()), msg),
            ThoughtFragment::Action(msg) => println!("⚙️  [{}] Executing: {}", self.format_duration(thought.timestamp.elapsed()), msg),
            ThoughtFragment::Result(msg) => println!("✅ [{}] Completed: {}", self.format_duration(thought.timestamp.elapsed()), msg),
            ThoughtFragment::Process(msg) => println!("🔄 [{}] Processing: {}", self.format_duration(thought.timestamp.elapsed()), msg),
            ThoughtFragment::Insight(msg) => println!("💡 [{}] Insight: {}", self.format_duration(thought.timestamp.elapsed()), msg),
            ThoughtFragment::Question(msg) => println!("❓ [{}] Question: {}", self.format_duration(thought.timestamp.elapsed()), msg),
        }
    }

    /// Display thought in normal format
    fn display_thought(&self, thought: &Thought) {
        match &thought.fragment {
            ThoughtFragment::Context(msg) => println!("📚 Context: {}", msg),
            ThoughtFragment::Hypothesis(msg) => println!("🧠 Hypothesis: {}", msg),
            ThoughtFragment::Action(msg) => println!("⚙️  Action: {}", msg),
            ThoughtFragment::Result(msg) => println!("✅ Result: {}", msg),
            ThoughtFragment::Process(msg) => println!("🔄 Process: {}", msg),
            ThoughtFragment::Insight(msg) => println!("💡 Insight: {}", msg),
            ThoughtFragment::Question(msg) => println!("❓ Question: {}", msg),
        }
    }

    /// Format duration in a human-readable way
    fn format_duration(&self, duration: Duration) -> String {
        if duration.as_millis() < 1000 {
            format!("{}ms", duration.as_millis())
        } else {
            format!("{:.1}s", duration.as_secs_f64())
        }
    }

    /// Get recent thoughts
    pub fn get_recent_thoughts(&self, count: usize) -> Vec<Thought> {
        if let Ok(thoughts) = self.thoughts.lock() {
            thoughts.iter()
                .rev()
                .take(count)
                .cloned()
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Get thoughts filtered by type
    pub fn get_thoughts_by_type(&self, fragment_type: &str) -> Vec<Thought> {
        if let Ok(thoughts) = self.thoughts.lock() {
            thoughts.iter()
                .filter(|thought| {
                    matches!(
                        (&thought.fragment, fragment_type),
                        (ThoughtFragment::Context(_), "context") |
                        (ThoughtFragment::Hypothesis(_), "hypothesis") |
                        (ThoughtFragment::Action(_), "action") |
                        (ThoughtFragment::Result(_), "result") |
                        (ThoughtFragment::Process(_), "process") |
                        (ThoughtFragment::Insight(_), "insight") |
                        (ThoughtFragment::Question(_), "question")
                    )
                })
                .cloned()
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Clear all stored thoughts
    pub fn clear_thoughts(&self) {
        if let Ok(mut thoughts) = self.thoughts.lock() {
            thoughts.clear();
        }
    }

    /// Get the count of stored thoughts
    pub fn thought_count(&self) -> usize {
        if let Ok(thoughts) = self.thoughts.lock() {
            thoughts.len()
        } else {
            0
        }
    }

    /// Get a summary of the thinking process
    pub fn summary(&self) -> String {
        let thoughts = self.get_recent_thoughts(10);
        let action_count = thoughts.iter()
            .filter(|t| matches!(t.fragment, ThoughtFragment::Action(_)))
            .count();
        let result_count = thoughts.iter()
            .filter(|t| matches!(t.fragment, ThoughtFragment::Result(_)))
            .count();

        format!("Recent thinking: {} actions, {} results", action_count, result_count)
    }
}
