//! Dify AI Orchestrator
//!
//! Integrates Dify AI platform with the Brivas Platform for:
//! - AI-powered campaign creation via natural language
//! - Multi-channel intelligent customer support agents
//! - AIOps incident analysis and automated remediation
//! - RAG-powered developer API assistance
//! - Billing dispute resolution workflows

#![allow(dead_code)]

use brivas_core::{BrivasService, HealthStatus, MicroserviceRuntime, ReadinessStatus, Result};
use brivas_lumadb::{LumaDbPool, PoolConfig};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;

mod config;
mod client;
mod agents;
mod workflows;
mod tools;
mod rag;

pub use config::DifyConfig;
use client::DifyClient;
use agents::AgentRegistry;
use workflows::WorkflowRegistry;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("dify_orchestrator=debug".parse().unwrap()),
        )
        .json()
        .init();

    info!("Starting Dify AI Orchestrator");

    let service = Arc::new(DifyOrchestratorService::new().await?);
    MicroserviceRuntime::run(service).await
}

/// Dify orchestrator service state
pub struct DifyOrchestratorService {
    config: DifyConfig,
    pool: LumaDbPool,
    client: Arc<DifyClient>,
    agents: Arc<AgentRegistry>,
    workflows: Arc<WorkflowRegistry>,
    active_conversations: Arc<RwLock<dashmap::DashMap<String, Conversation>>>,
    start_time: std::time::Instant,
}

/// Active conversation state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conversation {
    pub id: String,
    pub user_id: String,
    pub channel: MessageChannel,
    pub dify_conversation_id: Option<String>,
    pub agent_id: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub last_message_at: chrono::DateTime<chrono::Utc>,
    pub message_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MessageChannel {
    Sms,
    Rcs,
    Whatsapp,
    Ussd,
    Web,
    Voice,
}

/// Agent invocation request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentRequest {
    pub agent_id: String,
    pub user_id: String,
    pub channel: MessageChannel,
    pub message: String,
    pub conversation_id: Option<String>,
    pub context: Option<serde_json::Value>,
}

/// Agent response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentResponse {
    pub conversation_id: String,
    pub message: String,
    pub actions: Vec<AgentAction>,
    pub suggested_replies: Vec<String>,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentAction {
    pub action_type: String,
    pub parameters: serde_json::Value,
    pub result: Option<serde_json::Value>,
}

impl DifyOrchestratorService {
    pub async fn new() -> Result<Self> {
        let config = DifyConfig::from_env()?;
        
        // Create LumaDB connection pool
        let pool_config = PoolConfig {
            url: config.lumadb_url.clone(),
            max_size: 16,
            min_idle: Some(2),
        };
        let pool = LumaDbPool::new(pool_config).await
            .map_err(|e| brivas_core::BrivasError::Database(e.to_string()))?;
        
        // Initialize Dify client
        let client = Arc::new(DifyClient::new(&config));
        
        // Initialize registries
        let agents = Arc::new(AgentRegistry::new(client.clone()));
        let workflows = Arc::new(WorkflowRegistry::new(client.clone()));

        Ok(Self {
            config,
            pool,
            client,
            agents,
            workflows,
            active_conversations: Arc::new(RwLock::new(dashmap::DashMap::new())),
            start_time: std::time::Instant::now(),
        })
    }
}

#[async_trait::async_trait]
impl BrivasService for DifyOrchestratorService {
    fn service_id(&self) -> &'static str {
        "dify-orchestrator"
    }

    async fn health(&self) -> HealthStatus {
        HealthStatus {
            healthy: true,
            service_id: self.service_id().to_string(),
            version: self.version().to_string(),
            uptime_seconds: self.start_time.elapsed().as_secs(),
        }
    }

    async fn ready(&self) -> ReadinessStatus {
        let db_healthy = self.pool.is_healthy().await;
        let dify_healthy = self.client.health_check().await.is_ok();
        
        ReadinessStatus {
            ready: db_healthy && dify_healthy,
            dependencies: vec![
                brivas_core::DependencyStatus {
                    name: "lumadb".to_string(),
                    available: db_healthy,
                    latency_ms: Some(1),
                },
                brivas_core::DependencyStatus {
                    name: "dify".to_string(),
                    available: dify_healthy,
                    latency_ms: Some(50),
                },
            ],
        }
    }

    async fn shutdown(&self) -> Result<()> {
        info!("Shutting down Dify AI Orchestrator");
        Ok(())
    }

    async fn start(&self) -> Result<()> {
        info!(
            http = %self.config.http_bind,
            dify_base = %self.config.dify_base_url,
            "Starting Dify AI Orchestrator"
        );
        
        // Create HTTP routes
        let agents = self.agents.clone();
        let workflows = self.workflows.clone();
        let conversations = self.active_conversations.clone();
        let client = self.client.clone();
        
        let app = axum::Router::new()
            .route("/health", axum::routing::get(|| async { "OK" }))
            .route("/ready", axum::routing::get(|| async { "OK" }))
            // Agent endpoints
            .route("/api/v1/agents", axum::routing::get({
                let agents = agents.clone();
                move || {
                    let agents = agents.clone();
                    async move {
                        axum::Json(serde_json::json!({
                            "agents": agents.list()
                        }))
                    }
                }
            }))
            .route("/api/v1/agents/:id/chat", axum::routing::post({
                let agents = agents.clone();
                move |axum::extract::Path(id): axum::extract::Path<String>,
                      axum::Json(req): axum::Json<ChatRequest>| {
                    let agents = agents.clone();
                    async move {
                        match agents.chat(&id, &req.message, req.conversation_id.as_deref()).await {
                            Ok(response) => axum::Json(serde_json::json!({
                                "response": response
                            })),
                            Err(e) => axum::Json(serde_json::json!({
                                "error": e.to_string()
                            })),
                        }
                    }
                }
            }))
            // Workflow endpoints
            .route("/api/v1/workflows", axum::routing::get({
                let workflows = workflows.clone();
                move || {
                    let workflows = workflows.clone();
                    async move {
                        axum::Json(serde_json::json!({
                            "workflows": workflows.list()
                        }))
                    }
                }
            }))
            .route("/api/v1/workflows/:id/run", axum::routing::post({
                let workflows = workflows.clone();
                move |axum::extract::Path(id): axum::extract::Path<String>,
                      axum::Json(inputs): axum::Json<serde_json::Value>| {
                    let workflows = workflows.clone();
                    async move {
                        match workflows.run(&id, inputs).await {
                            Ok(result) => axum::Json(serde_json::json!({
                                "result": result
                            })),
                            Err(e) => axum::Json(serde_json::json!({
                                "error": e.to_string()
                            })),
                        }
                    }
                }
            }))
            // Conversation management
            .route("/api/v1/conversations", axum::routing::get({
                let conversations = conversations.clone();
                move || {
                    let conversations = conversations.clone();
                    async move {
                        let convs = conversations.read().await;
                        axum::Json(serde_json::json!({
                            "active_conversations": convs.len()
                        }))
                    }
                }
            }))
            // Campaign builder (innovative use case)
            .route("/api/v1/campaign-builder", axum::routing::post({
                let workflows = workflows.clone();
                move |axum::Json(req): axum::Json<CampaignBuilderRequest>| {
                    let workflows = workflows.clone();
                    async move {
                        // Use NL -> Campaign workflow
                        let inputs = serde_json::json!({
                            "description": req.description,
                            "target_audience": req.target_audience,
                            "channel": req.channel,
                            "budget": req.budget
                        });
                        
                        match workflows.run("campaign_builder", inputs).await {
                            Ok(campaign) => axum::Json(serde_json::json!({
                                "campaign": campaign,
                                "status": "draft_created"
                            })),
                            Err(e) => axum::Json(serde_json::json!({
                                "error": e.to_string()
                            })),
                        }
                    }
                }
            }))
            // AI Support agent (multi-channel)
            .route("/api/v1/support", axum::routing::post({
                let agents = agents.clone();
                move |axum::Json(req): axum::Json<SupportRequest>| {
                    let agents = agents.clone();
                    async move {
                        match agents.chat("customer_support", &req.message, req.conversation_id.as_deref()).await {
                            Ok(response) => axum::Json(serde_json::json!({
                                "response": response,
                                "channel": req.channel
                            })),
                            Err(e) => axum::Json(serde_json::json!({
                                "error": e.to_string()
                            })),
                        }
                    }
                }
            }))
            // AIOps analyst
            .route("/api/v1/aiops/analyze", axum::routing::post({
                let agents = agents.clone();
                move |axum::Json(incident): axum::Json<IncidentAnalysisRequest>| {
                    let agents = agents.clone();
                    async move {
                        let message = format!(
                            "Analyze this incident: Service: {}, Error: {}, Metrics: {:?}",
                            incident.service, incident.error_message, incident.metrics
                        );
                        
                        match agents.chat("aiops_analyst", &message, None).await {
                            Ok(analysis) => axum::Json(serde_json::json!({
                                "analysis": analysis,
                                "incident_id": incident.incident_id
                            })),
                            Err(e) => axum::Json(serde_json::json!({
                                "error": e.to_string()
                            })),
                        }
                    }
                }
            }));

        let listener = tokio::net::TcpListener::bind(&self.config.http_bind).await?;
        axum::serve(listener, app).await?;

        Ok(())
    }
}

// Request/Response types
#[derive(Debug, Deserialize)]
struct ChatRequest {
    message: String,
    conversation_id: Option<String>,
}

#[derive(Debug, Deserialize)]
struct CampaignBuilderRequest {
    description: String,
    target_audience: Option<String>,
    channel: Option<String>,
    budget: Option<f64>,
}

#[derive(Debug, Deserialize)]
struct SupportRequest {
    message: String,
    channel: MessageChannel,
    user_id: String,
    conversation_id: Option<String>,
}

#[derive(Debug, Deserialize)]
struct IncidentAnalysisRequest {
    incident_id: String,
    service: String,
    error_message: String,
    metrics: serde_json::Value,
}
