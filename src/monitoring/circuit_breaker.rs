//! Circuit breaker implementation for Kandil Code
//!
//! Provides resilience patterns for AI service calls.

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub struct CircuitBreaker {
    failures: AtomicU64,
    successes: AtomicU64,
    threshold: u64,
    timeout: Duration,
    last_failure: AtomicU64, // Unix timestamp
    is_open: AtomicBool,
}

impl CircuitBreaker {
    pub fn new(threshold: u64, timeout: Duration) -> Self {
        Self {
            failures: AtomicU64::new(0),
            successes: AtomicU64::new(0),
            threshold,
            timeout,
            last_failure: AtomicU64::new(0),
            is_open: AtomicBool::new(false),
        }
    }

    pub fn is_open(&self) -> bool {
        let current_state = self.is_open.load(Ordering::Relaxed);

        // If circuit is closed, just return the state
        if !current_state {
            return false;
        }

        // If circuit is open, check if timeout has passed
        let last_failure = self.last_failure.load(Ordering::Relaxed);
        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let elapsed = current_time.saturating_sub(last_failure);

        if elapsed >= self.timeout.as_secs() {
            // Timeout has passed, attempt to reset the circuit
            self.try_reset();
            false // Circuit is now closed for this request
        } else {
            true // Still in timeout period
        }
    }

    pub fn record_success(&self) {
        self.successes.fetch_add(1, Ordering::Relaxed);
        // Reset failure count on success (in half-open state)
        self.failures.store(0, Ordering::Relaxed);
        self.is_open.store(false, Ordering::Relaxed);
    }

    pub fn record_failure(&self) {
        let failures = self.failures.fetch_add(1, Ordering::Relaxed) + 1;

        // Record the time of the last failure
        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        self.last_failure.store(current_time, Ordering::Relaxed);

        // Open the circuit if threshold is exceeded
        if failures >= self.threshold {
            self.is_open.store(true, Ordering::Relaxed);
        }
    }

    fn try_reset(&self) {
        self.is_open.store(false, Ordering::Relaxed);
        // Reset failure count when circuit closes
        self.failures.store(0, Ordering::Relaxed);
    }

    pub fn reset(&self) {
        self.failures.store(0, Ordering::Relaxed);
        self.successes.store(0, Ordering::Relaxed);
        self.is_open.store(false, Ordering::Relaxed);
        self.last_failure.store(0, Ordering::Relaxed);
    }

    pub fn get_state(&self) -> CircuitState {
        if self.is_open() {
            CircuitState::Open
        } else if self.failures.load(Ordering::Relaxed) > 0 {
            CircuitState::HalfOpen
        } else {
            CircuitState::Closed
        }
    }

    pub fn get_stats(&self) -> CircuitStats {
        CircuitStats {
            failures: self.failures.load(Ordering::Relaxed),
            successes: self.successes.load(Ordering::Relaxed),
            is_open: self.is_open(),
            state: self.get_state(),
        }
    }
}

#[derive(PartialEq, Debug)]

pub enum CircuitState {

    Closed,    // Normal operation

    HalfOpen,  // Testing if failure condition is resolved

    Open,      // Failure threshold exceeded, requests blocked

}

pub struct CircuitStats {
    pub failures: u64,
    pub successes: u64,
    pub is_open: bool,
    pub state: CircuitState,
}

// Wrapper for AI provider with circuit breaker
pub struct CircuitBreakerAIProvider {
    inner: Box<dyn crate::common::traits::AIProvider>,
    circuit_breaker: std::sync::Arc<CircuitBreaker>,
}

impl CircuitBreakerAIProvider {
    pub fn new(
        inner: Box<dyn crate::common::traits::AIProvider>,
        threshold: u64,
        timeout: Duration,
    ) -> Self {
        let circuit_breaker = std::sync::Arc::new(CircuitBreaker::new(threshold, timeout));

        Self {
            inner,
            circuit_breaker,
        }
    }

    pub async fn complete_with_circuit_breaker(
        &self,
        prompt: &str,
    ) -> Result<String, crate::errors::LocalModelError> {
        // Check if circuit is open
        if self.circuit_breaker.is_open() {
            return Err(crate::errors::LocalModelError::ConfigurationError {
                message: "Circuit breaker is open - too many failures".to_string(),
            });
        }

        match self.inner.complete(prompt).await {
            Ok(response) => {
                self.circuit_breaker.record_success();
                Ok(response)
            }
            Err(e) => {
                self.circuit_breaker.record_failure();
                Err(e)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio;

    #[tokio::test]
    async fn test_circuit_breaker() {
        let circuit_breaker = CircuitBreaker::new(3, Duration::from_millis(100));

        // Should start closed
        assert!(!circuit_breaker.is_open());
        assert_eq!(circuit_breaker.get_state(), CircuitState::Closed);

        // Cause enough failures to open the circuit
        for _ in 0..3 {
            circuit_breaker.record_failure();
        }

        assert!(circuit_breaker.is_open());
        assert_eq!(circuit_breaker.get_state(), CircuitState::Open);

        // Wait for timeout and check if circuit closes
        tokio::time::sleep(Duration::from_millis(101)).await;
        assert!(!circuit_breaker.is_open()); // Should be closed after timeout

        // Test success resets the circuit
        circuit_breaker.record_failure();
        circuit_breaker.record_failure();
        assert_eq!(circuit_breaker.get_state(), CircuitState::HalfOpen);

        circuit_breaker.record_success();
        assert_eq!(circuit_breaker.get_state(), CircuitState::Closed);
    }
}
