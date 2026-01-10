//! Workflow abstractions and utilities

use std::time::Duration;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::RetryPolicy;

/// Workflow execution options
#[derive(Debug, Clone)]
pub struct WorkflowOptions {
    /// Unique workflow ID
    pub workflow_id: String,
    /// Task queue to run on
    pub task_queue: String,
    /// Workflow execution timeout
    pub execution_timeout: Duration,
    /// Workflow run timeout
    pub run_timeout: Duration,
    /// Task timeout
    pub task_timeout: Duration,
    /// Retry policy
    pub retry_policy: Option<RetryPolicy>,
    /// Cron schedule (optional)
    pub cron_schedule: Option<String>,
    /// Memo (searchable metadata)
    pub memo: Option<serde_json::Value>,
    /// Search attributes
    pub search_attributes: Option<serde_json::Value>,
}

impl Default for WorkflowOptions {
    fn default() -> Self {
        Self {
            workflow_id: Uuid::new_v4().to_string(),
            task_queue: "brivas-workflows".to_string(),
            execution_timeout: Duration::from_secs(3600), // 1 hour
            run_timeout: Duration::from_secs(1800),       // 30 minutes
            task_timeout: Duration::from_secs(10),
            retry_policy: Some(RetryPolicy::default()),
            cron_schedule: None,
            memo: None,
            search_attributes: None,
        }
    }
}

impl WorkflowOptions {
    pub fn new(workflow_id: &str) -> Self {
        Self {
            workflow_id: workflow_id.to_string(),
            ..Default::default()
        }
    }

    pub fn with_task_queue(mut self, queue: &str) -> Self {
        self.task_queue = queue.to_string();
        self
    }

    pub fn with_execution_timeout(mut self, timeout: Duration) -> Self {
        self.execution_timeout = timeout;
        self
    }

    pub fn with_retry_policy(mut self, policy: RetryPolicy) -> Self {
        self.retry_policy = Some(policy);
        self
    }

    pub fn with_cron_schedule(mut self, schedule: &str) -> Self {
        self.cron_schedule = Some(schedule.to_string());
        self
    }
}

/// Activity execution options
#[derive(Debug, Clone)]
pub struct ActivityOptions {
    /// Task queue for the activity
    pub task_queue: Option<String>,
    /// Schedule-to-close timeout
    pub schedule_to_close_timeout: Duration,
    /// Start-to-close timeout
    pub start_to_close_timeout: Duration,
    /// Schedule-to-start timeout
    pub schedule_to_start_timeout: Duration,
    /// Heartbeat timeout
    pub heartbeat_timeout: Option<Duration>,
    /// Retry policy
    pub retry_policy: Option<RetryPolicy>,
}

impl Default for ActivityOptions {
    fn default() -> Self {
        Self {
            task_queue: None,
            schedule_to_close_timeout: Duration::from_secs(300),
            start_to_close_timeout: Duration::from_secs(60),
            schedule_to_start_timeout: Duration::from_secs(60),
            heartbeat_timeout: Some(Duration::from_secs(30)),
            retry_policy: Some(RetryPolicy::default()),
        }
    }
}

impl ActivityOptions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.start_to_close_timeout = timeout;
        self
    }

    pub fn with_retry_policy(mut self, policy: RetryPolicy) -> Self {
        self.retry_policy = Some(policy);
        self
    }

    pub fn no_retry(mut self) -> Self {
        self.retry_policy = Some(RetryPolicy::no_retry());
        self
    }
}

/// Workflow execution status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum WorkflowStatus {
    Running,
    Completed,
    Failed,
    Cancelled,
    Terminated,
    ContinuedAsNew,
    TimedOut,
}

/// Workflow execution info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowInfo {
    pub workflow_id: String,
    pub run_id: String,
    pub workflow_type: String,
    pub status: WorkflowStatus,
    pub start_time: chrono::DateTime<chrono::Utc>,
    pub close_time: Option<chrono::DateTime<chrono::Utc>>,
    pub task_queue: String,
}

/// Signal to send to a workflow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowSignal {
    pub signal_name: String,
    pub payload: serde_json::Value,
}

/// Query to send to a workflow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowQuery {
    pub query_name: String,
    pub args: Option<serde_json::Value>,
}
