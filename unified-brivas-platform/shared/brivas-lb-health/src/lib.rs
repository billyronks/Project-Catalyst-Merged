//! Load Balancer Health Monitoring Library
//!
//! Provides health checking and metrics for Cilium eBPF load balancer.
//! All metrics stored in LumaDB for global visibility.

pub mod coordinator;
pub mod monitor;
pub mod types;

pub use coordinator::GlobalLbCoordinator;
pub use monitor::LbHealthMonitor;
pub use types::*;
