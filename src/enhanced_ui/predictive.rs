use std::collections::VecDeque;

pub struct PredictiveExecutor {
    history: VecDeque<String>,
    max_history: usize,
}

impl PredictiveExecutor {
    pub fn new() -> Self {
        Self {
            history: VecDeque::with_capacity(16),
            max_history: 16,
        }
    }

    pub fn observe(&mut self, command: &str) {
        if self.history.len() == self.max_history {
            self.history.pop_front();
        }
        self.history.push_back(command.to_string());
    }

    pub fn predict_hint(&self) -> Option<String> {
        self.history.back().map(|cmd| format!("{} | /test", cmd))
    }

    pub fn prefetch(&self, command: &str) {
        println!("🔮 Prefetching resources for `{}`", command);
    }
}
