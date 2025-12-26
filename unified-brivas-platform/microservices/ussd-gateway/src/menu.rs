//! Dynamic USSD Menu System

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Menu definition for USSD flows
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MenuDefinition {
    pub id: String,
    pub title: String,
    pub options: Vec<MenuOption>,
    pub input_handler: Option<String>,
    pub actions: Vec<MenuAction>,
}

/// Menu option
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MenuOption {
    pub key: String,
    pub label: String,
    pub target: MenuTarget,
}

/// Target action for menu option
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum MenuTarget {
    Navigate { menu_id: String },
    Action { action_id: String },
    Input { prompt: String, variable: String },
    End { message: String },
}

/// Menu action to execute
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum MenuAction {
    Navigate { target_menu: String },
    CallService { service: String, method: String },
    SendSms { template_id: String },
    InitiatePayment { amount_var: String },
    EndSession { message: String },
}

/// Current menu node in navigation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MenuNode {
    pub id: String,
    pub parent_id: Option<String>,
}

impl MenuNode {
    pub fn root() -> Self {
        Self {
            id: "root".to_string(),
            parent_id: None,
        }
    }
}

/// Menu renderer for building USSD text
pub struct MenuRenderer {
    max_length: usize,
}

impl MenuRenderer {
    pub fn new(max_length: usize) -> Self {
        Self { max_length }
    }

    pub fn render(&self, menu: &MenuDefinition, _variables: &HashMap<String, serde_json::Value>) -> String {
        let mut output = format!("{}\n", menu.title);

        for option in &menu.options {
            output.push_str(&format!("{}. {}\n", option.key, option.label));
        }

        // Truncate if too long
        if output.len() > self.max_length {
            output.truncate(self.max_length - 3);
            output.push_str("...");
        }

        output
    }
}

/// Localized string support
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalizedString {
    pub en: String,
    pub ha: Option<String>, // Hausa
    pub yo: Option<String>, // Yoruba
    pub ig: Option<String>, // Igbo
}

impl LocalizedString {
    pub fn get(&self, lang: &str) -> &str {
        match lang {
            "ha" => self.ha.as_deref().unwrap_or(&self.en),
            "yo" => self.yo.as_deref().unwrap_or(&self.en),
            "ig" => self.ig.as_deref().unwrap_or(&self.en),
            _ => &self.en,
        }
    }
}
