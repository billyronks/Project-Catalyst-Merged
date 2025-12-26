//! USSD Session Management

use chrono::{DateTime, Utc};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use crate::menu::MenuNode;
use crate::operators::Operator;

/// USSD Session Manager
#[derive(Clone)]
pub struct UssdSessionManager {
    sessions: Arc<DashMap<String, UssdSession>>,
    #[allow(dead_code)]
    db_url: String,
    session_ttl: Duration,
}

impl UssdSessionManager {
    pub async fn new(db_url: &str, ttl_secs: u64) -> brivas_core::Result<Self> {
        Ok(Self {
            sessions: Arc::new(DashMap::new()),
            db_url: db_url.to_string(),
            session_ttl: Duration::from_secs(ttl_secs),
        })
    }

    /// Create a new session
    pub async fn create(&self, msisdn: &str, service_code: &str, operator: Operator) -> UssdSession {
        let session = UssdSession::new(msisdn, service_code, operator, self.session_ttl);
        self.sessions.insert(session.id.clone(), session.clone());
        session
    }

    /// Get existing session or create new one
    pub async fn get_or_create(
        &self,
        session_id: &str,
        msisdn: &str,
        operator: Operator,
    ) -> UssdSession {
        if let Some(session) = self.sessions.get(session_id) {
            if !session.is_expired() {
                return session.clone();
            }
        }
        self.create(msisdn, "*123#", operator).await
    }

    /// Remove expired sessions (called periodically)
    pub async fn cleanup_expired(&self) {
        self.sessions.retain(|_, session| !session.is_expired());
    }
}

/// USSD Session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UssdSession {
    pub id: String,
    pub msisdn: String,
    pub service_code: String,
    pub operator: Operator,
    pub current_menu: MenuNode,
    pub navigation_stack: Vec<String>,
    pub variables: HashMap<String, serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub last_activity: DateTime<Utc>,
    pub ttl_secs: u64,
}

/// USSD response to send to user
#[derive(Debug, Clone)]
pub struct UssdResponse {
    pub message: String,
    pub end_session: bool,
}

impl UssdSession {
    pub fn new(msisdn: &str, service_code: &str, operator: Operator, ttl: Duration) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            msisdn: msisdn.to_string(),
            service_code: service_code.to_string(),
            operator,
            current_menu: MenuNode::root(),
            navigation_stack: vec![],
            variables: HashMap::new(),
            created_at: Utc::now(),
            last_activity: Utc::now(),
            ttl_secs: ttl.as_secs(),
        }
    }

    pub fn is_expired(&self) -> bool {
        let elapsed = Utc::now().signed_duration_since(self.last_activity);
        elapsed.num_seconds() as u64 > self.ttl_secs
    }

    /// Get the welcome menu
    pub fn get_welcome_menu(&self) -> UssdResponse {
        UssdResponse {
            message: format!(
                "Welcome to Brivas\n\
                1. Check Balance\n\
                2. Send SMS\n\
                3. Buy Airtime\n\
                4. Account Settings\n\
                0. Exit"
            ),
            end_session: false,
        }
    }

    /// Process user input and return response
    pub async fn process_input(&mut self, input: &str) -> UssdResponse {
        self.last_activity = Utc::now();

        match input.trim() {
            "0" => UssdResponse {
                message: "Thank you for using Brivas. Goodbye!".to_string(),
                end_session: true,
            },
            "1" => {
                self.navigation_stack.push("balance".to_string());
                UssdResponse {
                    message: "Your balance is: NGN 5,000.00\n\n0. Back".to_string(),
                    end_session: false,
                }
            }
            "2" => {
                self.navigation_stack.push("sms".to_string());
                UssdResponse {
                    message: "Enter recipient number:".to_string(),
                    end_session: false,
                }
            }
            "3" => {
                self.navigation_stack.push("airtime".to_string());
                UssdResponse {
                    message: "Buy Airtime\n\
                        1. NGN 100\n\
                        2. NGN 200\n\
                        3. NGN 500\n\
                        4. NGN 1000\n\
                        0. Back"
                        .to_string(),
                    end_session: false,
                }
            }
            "4" => {
                self.navigation_stack.push("settings".to_string());
                UssdResponse {
                    message: "Account Settings\n\
                        1. View Profile\n\
                        2. Change PIN\n\
                        3. Enable 2FA\n\
                        0. Back"
                        .to_string(),
                    end_session: false,
                }
            }
            _ => {
                // Handle sub-menu inputs based on navigation stack
                if let Some(current) = self.navigation_stack.last() {
                    match current.as_str() {
                        "sms" => {
                            // User entered phone number for SMS
                            self.variables
                                .insert("recipient".to_string(), serde_json::json!(input));
                            UssdResponse {
                                message: "Enter your message:".to_string(),
                                end_session: false,
                            }
                        }
                        "airtime" => match input {
                            "1" | "2" | "3" | "4" => {
                                let amount = match input {
                                    "1" => 100,
                                    "2" => 200,
                                    "3" => 500,
                                    "4" => 1000,
                                    _ => 0,
                                };
                                UssdResponse {
                                    message: format!(
                                        "You purchased NGN {} airtime successfully!\n\nThank you!",
                                        amount
                                    ),
                                    end_session: true,
                                }
                            }
                            _ => self.get_welcome_menu(),
                        },
                        _ => self.get_welcome_menu(),
                    }
                } else {
                    self.get_welcome_menu()
                }
            }
        }
    }
}
