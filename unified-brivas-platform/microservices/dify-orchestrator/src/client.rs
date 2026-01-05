//! Dify API Client
//!
//! HTTP client for Dify AI platform APIs

use reqwest::Client;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::{debug, info};

use crate::config::DifyConfig;

#[derive(Debug, Error)]
pub enum DifyError {
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    
    #[error("API error: {0}")]
    Api(String),
    
    #[error("Parse error: {0}")]
    Parse(String),
    
    #[error("Rate limited")]
    RateLimited,
    
    #[error("Unauthorized")]
    Unauthorized,
}

pub type Result<T> = std::result::Result<T, DifyError>;

/// Dify API client
pub struct DifyClient {
    client: Client,
    base_url: String,
    api_key: String,
}

impl DifyClient {
    pub fn new(config: &DifyConfig) -> Self {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(config.request_timeout_secs))
            .build()
            .expect("Failed to create HTTP client");
            
        Self {
            client,
            base_url: config.dify_base_url.clone(),
            api_key: config.dify_api_key.clone(),
        }
    }
    
    /// Health check for Dify connection
    pub async fn health_check(&self) -> Result<()> {
        // Simple check - in production would call Dify health endpoint
        if self.api_key.is_empty() {
            return Err(DifyError::Unauthorized);
        }
        Ok(())
    }
    
    /// Send message to chat completion API
    pub async fn chat_message(
        &self,
        app_id: &str,
        message: &str,
        conversation_id: Option<&str>,
        user: &str,
    ) -> Result<ChatResponse> {
        let url = format!("{}/chat-messages", self.base_url);
        
        let request = ChatRequest {
            inputs: serde_json::json!({}),
            query: message.to_string(),
            response_mode: "blocking".to_string(),
            conversation_id: conversation_id.map(|s| s.to_string()),
            user: user.to_string(),
        };
        
        debug!(app_id = %app_id, "Sending chat message to Dify");
        
        let response = self.client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?;
        
        if response.status().is_success() {
            let chat_response: ChatResponse = response.json().await?;
            Ok(chat_response)
        } else if response.status().as_u16() == 401 {
            Err(DifyError::Unauthorized)
        } else if response.status().as_u16() == 429 {
            Err(DifyError::RateLimited)
        } else {
            let error_text = response.text().await.unwrap_or_default();
            Err(DifyError::Api(error_text))
        }
    }
    
    /// Run a workflow
    pub async fn run_workflow(
        &self,
        workflow_id: &str,
        inputs: serde_json::Value,
        user: &str,
    ) -> Result<WorkflowResponse> {
        let url = format!("{}/workflows/run", self.base_url);
        
        let request = WorkflowRequest {
            inputs,
            response_mode: "blocking".to_string(),
            user: user.to_string(),
        };
        
        info!(workflow_id = %workflow_id, "Running Dify workflow");
        
        let response = self.client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?;
        
        if response.status().is_success() {
            let workflow_response: WorkflowResponse = response.json().await?;
            Ok(workflow_response)
        } else {
            let error_text = response.text().await.unwrap_or_default();
            Err(DifyError::Api(error_text))
        }
    }
    
    /// Query knowledge base (RAG)
    pub async fn query_knowledge(
        &self,
        dataset_id: &str,
        query: &str,
        top_k: u32,
    ) -> Result<KnowledgeResponse> {
        let url = format!("{}/datasets/{}/query", self.base_url, dataset_id);
        
        let request = KnowledgeQuery {
            query: query.to_string(),
            top_k,
            score_threshold: Some(0.5),
        };
        
        debug!(dataset_id = %dataset_id, "Querying Dify knowledge base");
        
        let response = self.client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?;
        
        if response.status().is_success() {
            let knowledge_response: KnowledgeResponse = response.json().await?;
            Ok(knowledge_response)
        } else {
            let error_text = response.text().await.unwrap_or_default();
            Err(DifyError::Api(error_text))
        }
    }
    
    /// Upload document to knowledge base
    pub async fn upload_document(
        &self,
        dataset_id: &str,
        content: &str,
        name: &str,
    ) -> Result<DocumentUploadResponse> {
        let url = format!("{}/datasets/{}/document/create_by_text", self.base_url, dataset_id);
        
        let request = DocumentUploadRequest {
            name: name.to_string(),
            text: content.to_string(),
            indexing_technique: "high_quality".to_string(),
            process_rule: ProcessRule {
                mode: "automatic".to_string(),
            },
        };
        
        info!(dataset_id = %dataset_id, document = %name, "Uploading document to Dify");
        
        let response = self.client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?;
        
        if response.status().is_success() {
            let upload_response: DocumentUploadResponse = response.json().await?;
            Ok(upload_response)
        } else {
            let error_text = response.text().await.unwrap_or_default();
            Err(DifyError::Api(error_text))
        }
    }
}

// Request/Response types

#[derive(Debug, Serialize)]
struct ChatRequest {
    inputs: serde_json::Value,
    query: String,
    response_mode: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    conversation_id: Option<String>,
    user: String,
}

#[derive(Debug, Deserialize)]
pub struct ChatResponse {
    pub message_id: String,
    pub conversation_id: String,
    pub answer: String,
    #[serde(default)]
    pub metadata: serde_json::Value,
}

#[derive(Debug, Serialize)]
struct WorkflowRequest {
    inputs: serde_json::Value,
    response_mode: String,
    user: String,
}

#[derive(Debug, Deserialize)]
pub struct WorkflowResponse {
    pub workflow_run_id: String,
    pub task_id: Option<String>,
    pub data: WorkflowData,
}

#[derive(Debug, Deserialize)]
pub struct WorkflowData {
    pub id: String,
    pub outputs: serde_json::Value,
    pub status: String,
    pub elapsed_time: Option<f64>,
    pub total_tokens: Option<u32>,
}

#[derive(Debug, Serialize)]
struct KnowledgeQuery {
    query: String,
    top_k: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    score_threshold: Option<f64>,
}

#[derive(Debug, Deserialize)]
pub struct KnowledgeResponse {
    pub query: String,
    pub records: Vec<KnowledgeRecord>,
}

#[derive(Debug, Deserialize)]
pub struct KnowledgeRecord {
    pub segment: KnowledgeSegment,
    pub score: f64,
}

#[derive(Debug, Deserialize)]
pub struct KnowledgeSegment {
    pub id: String,
    pub content: String,
    pub document_id: String,
    pub document_name: String,
}

#[derive(Debug, Serialize)]
struct DocumentUploadRequest {
    name: String,
    text: String,
    indexing_technique: String,
    process_rule: ProcessRule,
}

#[derive(Debug, Serialize)]
struct ProcessRule {
    mode: String,
}

#[derive(Debug, Deserialize)]
pub struct DocumentUploadResponse {
    pub document: DocumentInfo,
}

#[derive(Debug, Deserialize)]
pub struct DocumentInfo {
    pub id: String,
    pub name: String,
    pub word_count: Option<u32>,
}
