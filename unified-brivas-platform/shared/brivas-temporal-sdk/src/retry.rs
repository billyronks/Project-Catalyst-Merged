//! Retry policy configuration

use std::time::Duration;

/// Retry policy for activities and workflows
#[derive(Debug, Clone)]
pub struct RetryPolicy {
    /// Initial retry interval
    pub initial_interval: Duration,
    /// Backoff coefficient (multiplier for each retry)
    pub backoff_coefficient: f64,
    /// Maximum retry interval
    pub maximum_interval: Duration,
    /// Maximum number of attempts (0 = unlimited)
    pub maximum_attempts: u32,
    /// Non-retryable error types
    pub non_retryable_errors: Vec<String>,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            initial_interval: Duration::from_secs(1),
            backoff_coefficient: 2.0,
            maximum_interval: Duration::from_secs(100),
            maximum_attempts: 3,
            non_retryable_errors: vec![],
        }
    }
}

impl RetryPolicy {
    /// Create a new retry policy with custom settings
    pub fn new() -> Self {
        Self::default()
    }

    /// Set initial retry interval
    pub fn with_initial_interval(mut self, interval: Duration) -> Self {
        self.initial_interval = interval;
        self
    }

    /// Set backoff coefficient
    pub fn with_backoff_coefficient(mut self, coefficient: f64) -> Self {
        self.backoff_coefficient = coefficient;
        self
    }

    /// Set maximum interval
    pub fn with_maximum_interval(mut self, interval: Duration) -> Self {
        self.maximum_interval = interval;
        self
    }

    /// Set maximum attempts
    pub fn with_maximum_attempts(mut self, attempts: u32) -> Self {
        self.maximum_attempts = attempts;
        self
    }

    /// Add non-retryable error type
    pub fn with_non_retryable_error(mut self, error_type: &str) -> Self {
        self.non_retryable_errors.push(error_type.to_string());
        self
    }

    /// No retries - fail immediately
    pub fn no_retry() -> Self {
        Self {
            maximum_attempts: 1,
            ..Default::default()
        }
    }

    /// Aggressive retry for critical operations
    pub fn aggressive() -> Self {
        Self {
            initial_interval: Duration::from_millis(100),
            backoff_coefficient: 1.5,
            maximum_interval: Duration::from_secs(30),
            maximum_attempts: 10,
            non_retryable_errors: vec![],
        }
    }

    /// Conservative retry for expensive operations
    pub fn conservative() -> Self {
        Self {
            initial_interval: Duration::from_secs(5),
            backoff_coefficient: 2.0,
            maximum_interval: Duration::from_secs(300),
            maximum_attempts: 3,
            non_retryable_errors: vec![],
        }
    }
}
