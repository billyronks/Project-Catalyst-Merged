//! Circuit Breaker Pattern for Carrier Failover
//!
//! Provides resilient carrier management with:
//! - Automatic failure detection
//! - Gradual recovery testing
//! - Health-based routing decisions

use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use uuid::Uuid;

/// Circuit breaker states
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CircuitState {
    /// Normal operation - requests flow through
    Closed,
    /// Blocking requests due to failures
    Open,
    /// Testing recovery with limited requests
    HalfOpen,
}

/// Circuit breaker for carrier failover
pub struct CircuitBreaker {
    pub state: RwLock<CircuitState>,
    pub failure_count: AtomicU32,
    pub success_count: AtomicU32,
    pub total_requests: AtomicU64,
    pub failure_threshold: u32,
    pub success_threshold: u32,
    pub timeout: Duration,
    pub last_failure: RwLock<Option<Instant>>,
    pub opened_at: RwLock<Option<Instant>>,
}

impl CircuitBreaker {
    pub fn new(failure_threshold: u32, success_threshold: u32, timeout: Duration) -> Self {
        Self {
            state: RwLock::new(CircuitState::Closed),
            failure_count: AtomicU32::new(0),
            success_count: AtomicU32::new(0),
            total_requests: AtomicU64::new(0),
            failure_threshold,
            success_threshold,
            timeout,
            last_failure: RwLock::new(None),
            opened_at: RwLock::new(None),
        }
    }

    /// Check if request should be allowed
    pub async fn allow_request(&self) -> bool {
        let state = *self.state.read().await;
        
        match state {
            CircuitState::Closed => true,
            CircuitState::Open => {
                // Check if timeout has passed
                let opened_at = self.opened_at.read().await;
                if let Some(opened) = *opened_at {
                    if opened.elapsed() >= self.timeout {
                        // Transition to half-open
                        *self.state.write().await = CircuitState::HalfOpen;
                        tracing::info!("Circuit breaker transitioning to half-open");
                        return true;
                    }
                }
                false
            }
            CircuitState::HalfOpen => true, // Allow test requests
        }
    }

    /// Record a successful request
    pub async fn record_success(&self) {
        self.total_requests.fetch_add(1, Ordering::Relaxed);
        let success = self.success_count.fetch_add(1, Ordering::Relaxed) + 1;
        self.failure_count.store(0, Ordering::Relaxed);

        let state = *self.state.read().await;
        if state == CircuitState::HalfOpen && success >= self.success_threshold {
            *self.state.write().await = CircuitState::Closed;
            self.success_count.store(0, Ordering::Relaxed);
            tracing::info!("Circuit breaker closed after recovery");
        }
    }

    /// Record a failed request
    pub async fn record_failure(&self) {
        self.total_requests.fetch_add(1, Ordering::Relaxed);
        let failures = self.failure_count.fetch_add(1, Ordering::Relaxed) + 1;
        self.success_count.store(0, Ordering::Relaxed);
        *self.last_failure.write().await = Some(Instant::now());

        let state = *self.state.read().await;
        
        if state == CircuitState::HalfOpen {
            // Any failure in half-open returns to open
            *self.state.write().await = CircuitState::Open;
            *self.opened_at.write().await = Some(Instant::now());
            tracing::warn!("Circuit breaker re-opened after half-open failure");
        } else if state == CircuitState::Closed && failures >= self.failure_threshold {
            *self.state.write().await = CircuitState::Open;
            *self.opened_at.write().await = Some(Instant::now());
            tracing::warn!(failures, "Circuit breaker opened after threshold");
        }
    }

    /// Get current state
    pub async fn get_state(&self) -> CircuitState {
        *self.state.read().await
    }

    /// Get failure rate
    pub fn failure_rate(&self) -> f64 {
        let total = self.total_requests.load(Ordering::Relaxed);
        if total == 0 {
            return 0.0;
        }
        let failures = self.failure_count.load(Ordering::Relaxed);
        failures as f64 / total as f64
    }

    /// Reset the circuit breaker
    pub async fn reset(&self) {
        *self.state.write().await = CircuitState::Closed;
        self.failure_count.store(0, Ordering::Relaxed);
        self.success_count.store(0, Ordering::Relaxed);
        *self.last_failure.write().await = None;
        *self.opened_at.write().await = None;
    }
}

/// Circuit breaker manager for multiple carriers
pub struct CircuitBreakerManager {
    breakers: dashmap::DashMap<Uuid, CircuitBreaker>,
    default_failure_threshold: u32,
    default_success_threshold: u32,
    default_timeout: Duration,
}

impl CircuitBreakerManager {
    pub fn new() -> Self {
        Self {
            breakers: dashmap::DashMap::new(),
            default_failure_threshold: 5,
            default_success_threshold: 3,
            default_timeout: Duration::from_secs(30),
        }
    }

    pub fn get_or_create(&self, carrier_id: Uuid) -> dashmap::mapref::one::Ref<Uuid, CircuitBreaker> {
        if !self.breakers.contains_key(&carrier_id) {
            self.breakers.insert(
                carrier_id,
                CircuitBreaker::new(
                    self.default_failure_threshold,
                    self.default_success_threshold,
                    self.default_timeout,
                ),
            );
        }
        self.breakers.get(&carrier_id).unwrap()
    }

    pub async fn is_carrier_available(&self, carrier_id: Uuid) -> bool {
        let breaker = self.get_or_create(carrier_id);
        breaker.allow_request().await
    }

    pub async fn record_success(&self, carrier_id: Uuid) {
        if let Some(breaker) = self.breakers.get(&carrier_id) {
            breaker.record_success().await;
        }
    }

    pub async fn record_failure(&self, carrier_id: Uuid) {
        if let Some(breaker) = self.breakers.get(&carrier_id) {
            breaker.record_failure().await;
        }
    }

    pub async fn get_status(&self, carrier_id: Uuid) -> Option<CircuitState> {
        if let Some(breaker) = self.breakers.get(&carrier_id) {
            Some(breaker.get_state().await)
        } else {
            None
        }
    }

    /// Get all open/half-open circuits for alerting
    pub async fn get_unhealthy_carriers(&self) -> Vec<(Uuid, CircuitState)> {
        let mut unhealthy = vec![];
        for entry in self.breakers.iter() {
            let state = entry.get_state().await;
            if state != CircuitState::Closed {
                unhealthy.push((*entry.key(), state));
            }
        }
        unhealthy
    }
}

impl Default for CircuitBreakerManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_circuit_breaker_opens_after_failures() {
        let cb = CircuitBreaker::new(3, 2, Duration::from_secs(1));
        
        assert!(cb.allow_request().await);
        
        cb.record_failure().await;
        cb.record_failure().await;
        assert!(cb.allow_request().await); // Still closed
        
        cb.record_failure().await; // Third failure
        assert!(!cb.allow_request().await); // Now open
    }

    #[tokio::test]
    async fn test_circuit_breaker_closes_after_success() {
        let cb = CircuitBreaker::new(1, 2, Duration::from_millis(10));
        
        cb.record_failure().await; // Opens
        assert!(!cb.allow_request().await);
        
        tokio::time::sleep(Duration::from_millis(15)).await;
        
        assert!(cb.allow_request().await); // Half-open
        cb.record_success().await;
        cb.record_success().await;
        
        assert_eq!(cb.get_state().await, CircuitState::Closed);
    }
}
