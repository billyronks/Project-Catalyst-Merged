//! RAG Knowledge Base Management
//!
//! Sync platform documentation to Dify knowledge base

use std::sync::Arc;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::client::{DifyClient, DifyError};

#[derive(Debug, Error)]
pub enum RagError {
    #[error("Document not found: {0}")]
    NotFound(String),
    
    #[error("Sync failed: {0}")]
    SyncFailed(String),
    
    #[error("Dify error: {0}")]
    Dify(#[from] DifyError),
}

pub type Result<T> = std::result::Result<T, RagError>;

/// Knowledge base document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    pub id: String,
    pub title: String,
    pub content: String,
    pub category: DocumentCategory,
    pub tags: Vec<String>,
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum DocumentCategory {
    Api,
    Billing,
    Troubleshooting,
    General,
    Compliance,
}

/// RAG knowledge base manager
pub struct RagManager {
    client: Arc<DifyClient>,
    dataset_id: String,
    documents: Vec<Document>,
}

impl RagManager {
    pub fn new(client: Arc<DifyClient>, dataset_id: &str) -> Self {
        // Pre-populated with common documents
        let documents = vec![
            Document {
                id: "api-overview".to_string(),
                title: "Brivas API Overview".to_string(),
                content: r#"
                    # Brivas API Overview
                    
                    The Brivas Platform provides REST APIs for:
                    - SMS Messaging: Send/receive SMS messages
                    - Voice Calls: Initiate and manage voice calls
                    - USSD Services: Create interactive USSD menus
                    - RCS Messaging: Rich Communication Services
                    
                    Base URL: https://api.brivas.io/v1
                    
                    Authentication: Bearer token in Authorization header
                    
                    Rate Limits: 1000 requests/minute per API key
                "#.to_string(),
                category: DocumentCategory::Api,
                tags: vec!["api".to_string(), "getting-started".to_string()],
                last_updated: chrono::Utc::now(),
            },
            Document {
                id: "billing-faq".to_string(),
                title: "Billing FAQ".to_string(),
                content: r#"
                    # Billing FAQ
                    
                    ## How am I charged?
                    You are charged per message sent. SMS: $0.01-$0.10 depending on destination.
                    
                    ## When are invoices generated?
                    Invoices are generated on the 1st of each month.
                    
                    ## How do I dispute a charge?
                    Contact support@brivas.io with your account ID and transaction details.
                    
                    ## Payment methods accepted
                    Credit cards, bank transfer, and wallet top-up.
                "#.to_string(),
                category: DocumentCategory::Billing,
                tags: vec!["billing".to_string(), "faq".to_string()],
                last_updated: chrono::Utc::now(),
            },
            Document {
                id: "smpp-troubleshooting".to_string(),
                title: "SMPP Connection Troubleshooting".to_string(),
                content: r#"
                    # SMPP Connection Troubleshooting
                    
                    ## Common Issues
                    
                    ### Bind Failure (Error 0x0005 - Invalid Bind)
                    - Check system_id and password
                    - Verify IP is whitelisted
                    
                    ### Connection Timeout
                    - Check firewall allows port 2775
                    - Verify peer server is responding
                    
                    ### Session Drops
                    - Enable enquire_link heartbeats (every 30s)
                    - Check for network instability
                    
                    ## Auto-Recovery
                    The platform automatically attempts rebind with exponential backoff.
                "#.to_string(),
                category: DocumentCategory::Troubleshooting,
                tags: vec!["smpp".to_string(), "troubleshooting".to_string()],
                last_updated: chrono::Utc::now(),
            },
        ];
        
        Self {
            client,
            dataset_id: dataset_id.to_string(),
            documents,
        }
    }
    
    /// Search knowledge base
    pub async fn search(&self, query: &str, top_k: u32) -> Result<Vec<SearchResult>> {
        // In production, call Dify RAG API
        // For now, simple keyword matching
        let query_lower = query.to_lowercase();
        
        let mut results: Vec<SearchResult> = self.documents
            .iter()
            .filter(|doc| {
                doc.title.to_lowercase().contains(&query_lower) ||
                doc.content.to_lowercase().contains(&query_lower)
            })
            .map(|doc| SearchResult {
                document_id: doc.id.clone(),
                title: doc.title.clone(),
                content_snippet: doc.content.chars().take(200).collect(),
                score: 0.85,
                category: doc.category.clone(),
            })
            .take(top_k as usize)
            .collect();
        
        // Sort by relevance
        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
        
        Ok(results)
    }
    
    /// Sync documents to Dify
    pub async fn sync_to_dify(&self) -> Result<SyncResult> {
        let mut synced = 0;
        let mut failed = 0;
        
        for doc in &self.documents {
            match self.client.upload_document(
                &self.dataset_id,
                &doc.content,
                &doc.title,
            ).await {
                Ok(_) => synced += 1,
                Err(_) => failed += 1,
            }
        }
        
        Ok(SyncResult {
            total: self.documents.len(),
            synced,
            failed,
        })
    }
    
    /// Add document to knowledge base
    pub fn add_document(&mut self, doc: Document) {
        self.documents.push(doc);
    }
    
    /// Get document by ID
    pub fn get(&self, id: &str) -> Option<&Document> {
        self.documents.iter().find(|d| d.id == id)
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct SearchResult {
    pub document_id: String,
    pub title: String,
    pub content_snippet: String,
    pub score: f64,
    pub category: DocumentCategory,
}

#[derive(Debug, Clone, Serialize)]
pub struct SyncResult {
    pub total: usize,
    pub synced: usize,
    pub failed: usize,
}
