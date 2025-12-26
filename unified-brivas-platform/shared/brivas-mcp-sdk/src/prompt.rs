//! MCP Prompt types

use serde::{Deserialize, Serialize};

/// Prompt definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Prompt {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arguments: Option<Vec<PromptArgument>>,
}

/// Prompt argument
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptArgument {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default)]
    pub required: bool,
}

/// Prompt message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptMessage {
    pub role: PromptRole,
    pub content: PromptContent,
}

/// Prompt role
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PromptRole {
    User,
    Assistant,
}

/// Prompt content
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum PromptContent {
    Text { text: String },
    Image { data: String, mime_type: String },
    Resource { uri: String, text: String },
}

impl Prompt {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: None,
            arguments: None,
        }
    }

    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    pub fn with_arguments(mut self, args: Vec<PromptArgument>) -> Self {
        self.arguments = Some(args);
        self
    }
}

impl PromptArgument {
    pub fn required(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: None,
            required: true,
        }
    }

    pub fn optional(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: None,
            required: false,
        }
    }

    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }
}

impl PromptMessage {
    pub fn user(text: impl Into<String>) -> Self {
        Self {
            role: PromptRole::User,
            content: PromptContent::Text { text: text.into() },
        }
    }

    pub fn assistant(text: impl Into<String>) -> Self {
        Self {
            role: PromptRole::Assistant,
            content: PromptContent::Text { text: text.into() },
        }
    }
}
