#[derive(Clone, Debug)]
pub enum ThoughtFragment {
    Context(String),
    Hypothesis(String),
    Action(String),
    Result(String),
}

#[derive(Default)]
pub struct ThoughtStreamer;

impl ThoughtStreamer {
    pub fn new() -> Self {
        Self
    }

    pub fn emit(&self, fragment: ThoughtFragment) {
        match fragment {
            ThoughtFragment::Context(msg) => println!("📚 Context: {}", msg),
            ThoughtFragment::Hypothesis(msg) => println!("🧠 Hypothesis: {}", msg),
            ThoughtFragment::Action(msg) => println!("⚙️  Action: {}", msg),
            ThoughtFragment::Result(msg) => println!("✅ Result: {}", msg),
        }
    }
}
