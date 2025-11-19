use std::collections::VecDeque;
use std::time::{Duration, Instant};
use tokio::time::sleep;

#[derive(Debug, Clone)]
pub struct Prediction {
    pub command: String,
    pub confidence: f64,  // 0.0 to 1.0
    pub timestamp: Instant,
}

#[derive(Debug, Clone)]
pub struct GhostText {
    pub text: String,
    pub position: usize,
    pub confidence: f64,  // 0.0 to 1.0
}

pub struct PredictiveExecutor {
    history: VecDeque<String>,
    max_history: usize,
    predictions: VecDeque<Prediction>,
    max_predictions: usize,
    ghost_text_cache: VecDeque<GhostText>,
    max_ghost_text: usize,
    last_prefetch: Option<Instant>,
    prefetch_cooldown: Duration,
}

impl PredictiveExecutor {
    pub fn new() -> Self {
        Self {
            history: VecDeque::with_capacity(16),
            max_history: 16,
            predictions: VecDeque::with_capacity(8),
            max_predictions: 8,
            ghost_text_cache: VecDeque::with_capacity(4),
            max_ghost_text: 4,
            last_prefetch: None,
            prefetch_cooldown: Duration::from_millis(500),  // Prevent excessive prefetching
        }
    }

    /// Observe command execution to improve predictions
    pub fn observe(&mut self, command: &str) {
        // Add to history
        if self.history.len() == self.max_history {
            self.history.pop_front();
        }
        self.history.push_back(command.to_string());

        // Generate prediction based on this command
        if let Some(prediction) = self.generate_prediction(command) {
            if self.predictions.len() == self.max_predictions {
                self.predictions.pop_front();
            }
            self.predictions.push_back(prediction);
        }
    }

    /// Generate a prediction based on the command and context
    fn generate_prediction(&self, command: &str) -> Option<Prediction> {
        // Analyze the command to predict next likely commands
        let mut predicted_command = String::new();
        let mut confidence = 0.0;

        if command.starts_with("/test") {
            predicted_command = "/fix".to_string();
            confidence = 0.8;  // High confidence that /fix follows /test if there are issues
        } else if command.starts_with("/fix") {
            predicted_command = "/test".to_string();
            confidence = 0.9;  // Very high confidence that /test follows /fix
        } else if command.starts_with("/refactor") {
            predicted_command = "/test".to_string();
            confidence = 0.7;  // Likely to test after refactoring
        } else if command.starts_with("/ask") {
            // For ask commands, predict commands that might be related to the current context
            if let Some(last_cmd) = self.history.back() {
                if last_cmd.starts_with("/refactor") {
                    predicted_command = "/review".to_string();
                    confidence = 0.6;
                } else {
                    // Default to asking another question
                    predicted_command = "/ask".to_string();
                    confidence = 0.4;
                }
            } else {
                predicted_command = "/ask".to_string();
                confidence = 0.4;
            }
        } else if command.starts_with("/review") {
            predicted_command = "/refactor".to_string();
            confidence = 0.7;  // Likely to refactor after review
        } else {
            // Default behavior - predict /test after most commands
            predicted_command = "/test".to_string();
            confidence = 0.3;  // Lower confidence for generic prediction
        }

        Some(Prediction {
            command: predicted_command,
            confidence,
            timestamp: Instant::now(),
        })
    }

    /// Get the most likely next command based on history
    pub fn predict_hint(&self) -> Option<String> {
        self.predictions
            .back()
            .filter(|p| p.confidence > 0.5)  // Only show high-confidence predictions
            .map(|p| format!("💡 Suggested: {}", p.command))
    }

    /// Generate ghost text for the current input
    pub fn generate_ghost_text(&mut self, current_input: &str) -> Option<GhostText> {
        // Find commands in history that start similarly to the current input
        for history_cmd in self.history.iter().rev() {
            if history_cmd.starts_with(current_input) && history_cmd.len() > current_input.len() {
                let remaining_text = &history_cmd[current_input.len()..];
                return Some(GhostText {
                    text: remaining_text.to_string(),
                    position: current_input.len(),
                    confidence: 0.7,  // Medium-high confidence for historical completion
                });
            }
        }

        // If no history match, try prediction-based ghost text
        if let Some(prediction) = self.predictions.back() {
            if prediction.confidence > 0.6 {
                // Add to ghost text cache to make it available for UI
                let ghost = GhostText {
                    text: prediction.command.clone(),
                    position: current_input.len(),
                    confidence: prediction.confidence,
                };

                // Add to cache if it's not already there
                if !self.ghost_text_cache.iter().any(|g| g.text == ghost.text) {
                    if self.ghost_text_cache.len() == self.max_ghost_text {
                        self.ghost_text_cache.pop_front();
                    }
                    self.ghost_text_cache.push_back(ghost.clone());
                }

                return Some(ghost);
            }
        }

        None
    }

    /// Get the current ghost text to display
    pub fn get_ghost_text(&self) -> Option<&GhostText> {
        self.ghost_text_cache.back()
    }

    /// Prefetch resources needed for likely next commands
    pub fn prefetch(&self, command: &str) {
        println!("🔮 Prefetching resources for `{}`", command);

        // In a real implementation, this would prefetch resources like:
        // - Loading AI models in advance
        // - Preparing file contexts
        // - Pre-creating command objects
        // - Warming up APIs

        // For now, just log the prefetch action
        if command.starts_with("/test") {
            println!("  🔄 Pre-loading test framework...");
        } else if command.starts_with("/fix") {
            println!("  🔄 Pre-loading code analysis tools...");
        } else if command.starts_with("/refactor") {
            println!("  🔄 Pre-loading refactoring engine...");
        } else if command.starts_with("/review") {
            println!("  🔄 Pre-loading code review models...");
        }
    }

    /// Asynchronously prefetch resources for a given command
    pub async fn prefetch_async(&self, command: &str) -> Result<(), Box<dyn std::error::Error>> {
        // Check if we're in prefetch cooldown
        if let Some(last) = self.last_prefetch {
            if last.elapsed() < self.prefetch_cooldown {
                return Ok(()); // Skip prefetch if too recent
            }
        }

        println!("🔮 Async prefetching resources for `{}`", command);

        // Simulate async prefetching work
        sleep(Duration::from_millis(100)).await;

        // In a real implementation, this would:
        // - Load large models into memory
        // - Establish connections
        // - Pre-warm APIs
        // - Cache frequently accessed data

        Ok(())
    }

    /// Get the last N commands from history
    pub fn get_recent_commands(&self, n: usize) -> Vec<String> {
        self.history.iter()
            .rev()
            .take(n)
            .cloned()
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect()
    }

    /// Get recent predictions
    pub fn get_recent_predictions(&self) -> Vec<String> {
        self.predictions
            .iter()
            .filter(|p| p.confidence > 0.3)  // Only return predictions with decent confidence
            .map(|p| p.command.clone())
            .collect()
    }

    /// Check if prefetching should be throttled
    pub fn should_prefetch(&self) -> bool {
        match self.last_prefetch {
            None => true,  // Always prefetch first time
            Some(time) => time.elapsed() >= self.prefetch_cooldown,
        }
    }

    /// Update prefetch timestamp after prefetching
    pub fn mark_prefetch_time(&mut self) {
        self.last_prefetch = Some(Instant::now());
    }
}
