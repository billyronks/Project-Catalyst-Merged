//! SMPP Reconnection Integration Tests
//!
//! Tests for automatic rebind and session recovery

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tokio::time::timeout;

// Test session state tracking
#[derive(Debug, Clone, PartialEq)]
enum TestSessionState {
    Disconnected,
    Binding,
    Bound,
    Reconnecting,
    Failed,
}

// Mock SMPP session for testing
struct MockSmppSession {
    id: String,
    state: Arc<RwLock<TestSessionState>>,
    last_activity: Arc<RwLock<std::time::Instant>>,
    disconnect_count: Arc<AtomicU64>,
    bind_attempts: Arc<AtomicU64>,
}

impl MockSmppSession {
    fn new(id: &str) -> Self {
        Self {
            id: id.to_string(),
            state: Arc::new(RwLock::new(TestSessionState::Disconnected)),
            last_activity: Arc::new(RwLock::new(std::time::Instant::now())),
            disconnect_count: Arc::new(AtomicU64::new(0)),
            bind_attempts: Arc::new(AtomicU64::new(0)),
        }
    }

    async fn bind(&self) -> Result<(), &'static str> {
        self.bind_attempts.fetch_add(1, Ordering::SeqCst);
        let mut state = self.state.write().await;
        *state = TestSessionState::Binding;
        
        // Simulate bind delay
        tokio::time::sleep(Duration::from_millis(50)).await;
        
        // Simulate successful bind
        *state = TestSessionState::Bound;
        let mut last = self.last_activity.write().await;
        *last = std::time::Instant::now();
        
        Ok(())
    }

    async fn disconnect(&self) {
        let mut state = self.state.write().await;
        *state = TestSessionState::Disconnected;
        self.disconnect_count.fetch_add(1, Ordering::SeqCst);
    }

    async fn reconnect(&self, max_attempts: u32) -> Result<(), &'static str> {
        {
            let mut state = self.state.write().await;
            *state = TestSessionState::Reconnecting;
        }

        for attempt in 0..max_attempts {
            match self.bind().await {
                Ok(()) => return Ok(()),
                Err(_) => {
                    // Exponential backoff
                    let delay = Duration::from_millis(100 * (1 << attempt.min(5)));
                    tokio::time::sleep(delay).await;
                }
            }
        }

        let mut state = self.state.write().await;
        *state = TestSessionState::Failed;
        Err("Max reconnect attempts exceeded")
    }

    async fn send_enquire_link(&self) -> bool {
        let state = self.state.read().await;
        if *state != TestSessionState::Bound {
            return false;
        }
        
        let mut last = self.last_activity.write().await;
        *last = std::time::Instant::now();
        true
    }

    async fn check_session_timeout(&self, timeout_secs: u64) -> bool {
        let last = self.last_activity.read().await;
        last.elapsed().as_secs() > timeout_secs
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_session_bind_success() {
        let session = MockSmppSession::new("test-session-1");
        
        // Verify initial state
        assert_eq!(*session.state.read().await, TestSessionState::Disconnected);
        
        // Bind session
        let result = session.bind().await;
        assert!(result.is_ok());
        
        // Verify bound state
        assert_eq!(*session.state.read().await, TestSessionState::Bound);
        assert_eq!(session.bind_attempts.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn test_disconnect_increments_counter() {
        let session = MockSmppSession::new("test-session-2");
        
        // Bind and disconnect multiple times
        for i in 1..=3 {
            session.bind().await.unwrap();
            session.disconnect().await;
            assert_eq!(session.disconnect_count.load(Ordering::SeqCst), i);
        }
    }

    #[tokio::test]
    async fn test_reconnect_success() {
        let session = MockSmppSession::new("test-session-3");
        
        // Disconnect session
        session.disconnect().await;
        assert_eq!(*session.state.read().await, TestSessionState::Disconnected);
        
        // Reconnect
        let result = session.reconnect(5).await;
        assert!(result.is_ok());
        assert_eq!(*session.state.read().await, TestSessionState::Bound);
    }

    #[tokio::test]
    async fn test_enquire_link_updates_activity() {
        let session = MockSmppSession::new("test-session-4");
        
        // Bind session
        session.bind().await.unwrap();
        
        // Wait a bit
        tokio::time::sleep(Duration::from_millis(100)).await;
        
        // Send enquire link
        let success = session.send_enquire_link().await;
        assert!(success);
        
        // Activity should be recent
        let last = session.last_activity.read().await;
        assert!(last.elapsed() < Duration::from_millis(50));
    }

    #[tokio::test]
    async fn test_enquire_link_fails_when_disconnected() {
        let session = MockSmppSession::new("test-session-5");
        
        // Don't bind - session is disconnected
        let success = session.send_enquire_link().await;
        assert!(!success);
    }

    #[tokio::test]
    async fn test_session_timeout_detection() {
        let session = MockSmppSession::new("test-session-6");
        session.bind().await.unwrap();
        
        // Check timeout with 0 seconds (should immediately timeout)
        assert!(session.check_session_timeout(0).await);
        
        // Check timeout with long duration (should not timeout)
        assert!(!session.check_session_timeout(3600).await);
    }

    #[tokio::test]
    async fn test_concurrent_sessions() {
        let sessions: Vec<MockSmppSession> = (0..10)
            .map(|i| MockSmppSession::new(&format!("session-{}", i)))
            .collect();
        
        // Bind all sessions concurrently
        let handles: Vec<_> = sessions
            .iter()
            .map(|s| {
                let state = s.state.clone();
                let attempts = s.bind_attempts.clone();
                async move {
                    // Simulate bind
                    tokio::time::sleep(Duration::from_millis(10)).await;
                    attempts.fetch_add(1, Ordering::SeqCst);
                    *state.write().await = TestSessionState::Bound;
                }
            })
            .map(tokio::spawn)
            .collect();
        
        // Wait for all to complete
        for handle in handles {
            handle.await.unwrap();
        }
        
        // Verify all bound
        for session in &sessions {
            assert_eq!(*session.state.read().await, TestSessionState::Bound);
        }
    }

    #[tokio::test]
    async fn test_automatic_rebind_on_disconnect() {
        let session = MockSmppSession::new("test-session-rebind");
        
        // Initial bind
        session.bind().await.unwrap();
        assert_eq!(*session.state.read().await, TestSessionState::Bound);
        
        // Simulate disconnect
        session.disconnect().await;
        assert_eq!(session.disconnect_count.load(Ordering::SeqCst), 1);
        
        // Auto-reconnect
        let reconnect_result = session.reconnect(3).await;
        assert!(reconnect_result.is_ok());
        
        // Should be bound again
        assert_eq!(*session.state.read().await, TestSessionState::Bound);
        
        // Bind attempts should be > 1 (initial + reconnect)
        assert!(session.bind_attempts.load(Ordering::SeqCst) >= 2);
    }

    #[tokio::test]
    async fn test_heartbeat_monitoring_loop() {
        let session = MockSmppSession::new("test-heartbeat");
        session.bind().await.unwrap();
        
        let session_clone = MockSmppSession {
            id: session.id.clone(),
            state: session.state.clone(),
            last_activity: session.last_activity.clone(),
            disconnect_count: session.disconnect_count.clone(),
            bind_attempts: session.bind_attempts.clone(),
        };
        
        // Start heartbeat monitor
        let monitor_handle = tokio::spawn(async move {
            let mut heartbeat_count = 0;
            for _ in 0..3 {
                tokio::time::sleep(Duration::from_millis(50)).await;
                if session_clone.send_enquire_link().await {
                    heartbeat_count += 1;
                }
            }
            heartbeat_count
        });
        
        // Let monitor run with timeout
        let result = timeout(Duration::from_secs(1), monitor_handle).await;
        assert!(result.is_ok());
        
        let heartbeat_count = result.unwrap().unwrap();
        assert_eq!(heartbeat_count, 3);
    }
}
