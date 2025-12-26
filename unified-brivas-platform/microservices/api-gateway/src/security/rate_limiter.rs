//! Rate Limiter
//!
//! Multi-level rate limiting (IP, user, tenant, API key).

use dashmap::DashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

#[derive(Clone)]
pub struct RateLimiter {
    limits: Arc<DashMap<String, RateBucket>>,
    default_limit: u64,
    window_secs: u64,
}

struct RateBucket {
    count: u64,
    window_start: Instant,
}

impl RateLimiter {
    pub fn new(default_limit: u64, window_secs: u64) -> Self {
        Self {
            limits: Arc::new(DashMap::new()),
            default_limit,
            window_secs,
        }
    }

    /// Check if request is allowed
    pub fn check(&self, key: &str) -> RateLimitResult {
        let now = Instant::now();
        let window = Duration::from_secs(self.window_secs);
        
        let mut entry = self.limits.entry(key.to_string()).or_insert(RateBucket {
            count: 0,
            window_start: now,
        });
        
        // Reset window if expired
        if now.duration_since(entry.window_start) > window {
            entry.count = 0;
            entry.window_start = now;
        }
        
        entry.count += 1;
        
        if entry.count > self.default_limit {
            let retry_after = window.as_secs() - now.duration_since(entry.window_start).as_secs();
            RateLimitResult::Exceeded { retry_after }
        } else {
            RateLimitResult::Allowed {
                remaining: self.default_limit - entry.count,
            }
        }
    }

    /// Check by IP address
    pub fn check_ip(&self, ip: &str) -> RateLimitResult {
        self.check(&format!("ip:{}", ip))
    }

    /// Check by user ID
    pub fn check_user(&self, user_id: &str) -> RateLimitResult {
        self.check(&format!("user:{}", user_id))
    }

    /// Check by API key
    pub fn check_api_key(&self, api_key: &str) -> RateLimitResult {
        self.check(&format!("key:{}", api_key))
    }
}

pub enum RateLimitResult {
    Allowed { remaining: u64 },
    Exceeded { retry_after: u64 },
}
