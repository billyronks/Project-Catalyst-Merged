//! Brivas MCP Prompts

use brivas_mcp_sdk::prompt::{Prompt, PromptArgument, PromptMessage};

/// Collection of Brivas MCP prompts
pub struct BrivasPrompts {
    prompts: Vec<Prompt>,
}

impl BrivasPrompts {
    pub fn new() -> Self {
        Self {
            prompts: vec![
                Prompt::new("compose_sms")
                    .with_description("Compose an SMS message for a specific purpose")
                    .with_arguments(vec![
                        PromptArgument::required("purpose").with_description("The purpose of the SMS (e.g., reminder, promotional, transactional)"),
                        PromptArgument::optional("tone").with_description("Tone of message (formal, casual, urgent)"),
                        PromptArgument::optional("max_length").with_description("Max characters (default 160)"),
                    ]),
                    
                Prompt::new("create_rcs_card")
                    .with_description("Design an RCS rich card with image and suggestions")
                    .with_arguments(vec![
                        PromptArgument::required("product_or_service").with_description("What the card is about"),
                        PromptArgument::optional("image_url").with_description("URL of the image to use"),
                        PromptArgument::optional("action_buttons").with_description("Actions like 'Buy Now', 'Learn More'"),
                    ]),
                    
                Prompt::new("design_ussd_flow")
                    .with_description("Design a USSD menu flow for a specific use case")
                    .with_arguments(vec![
                        PromptArgument::required("use_case").with_description("What the USSD flow is for"),
                        PromptArgument::optional("max_depth").with_description("Maximum menu depth"),
                    ]),
                    
                Prompt::new("campaign_strategy")
                    .with_description("Create a multi-channel campaign strategy")
                    .with_arguments(vec![
                        PromptArgument::required("goal").with_description("Campaign goal"),
                        PromptArgument::required("audience").with_description("Target audience description"),
                        PromptArgument::optional("budget").with_description("Available budget"),
                        PromptArgument::optional("channels").with_description("Preferred channels"),
                    ]),
                    
                Prompt::new("ivr_script")
                    .with_description("Write an IVR voice script")
                    .with_arguments(vec![
                        PromptArgument::required("purpose").with_description("IVR purpose"),
                        PromptArgument::optional("language").with_description("Language for the script"),
                    ]),
            ],
        }
    }

    pub fn list(&self) -> &[Prompt] {
        &self.prompts
    }

    pub fn get_messages(&self, name: &str, args: &serde_json::Value) -> Option<Vec<PromptMessage>> {
        match name {
            "compose_sms" => {
                let purpose = args.get("purpose").and_then(|v| v.as_str()).unwrap_or("general");
                let tone = args.get("tone").and_then(|v| v.as_str()).unwrap_or("professional");
                
                Some(vec![
                    PromptMessage::user(format!(
                        "Compose a {} SMS message for: {}. \
                         Keep it concise (max 160 characters for single SMS). \
                         Include a clear call-to-action if appropriate.",
                        tone, purpose
                    )),
                ])
            }
            "create_rcs_card" => {
                let product = args.get("product_or_service").and_then(|v| v.as_str()).unwrap_or("product");
                
                Some(vec![
                    PromptMessage::user(format!(
                        "Create an RCS rich card for: {}. \
                         Include: title (max 200 chars), description (max 2000 chars), \
                         and 2-4 suggested action buttons. \
                         Format the response as JSON.",
                        product
                    )),
                ])
            }
            "design_ussd_flow" => {
                let use_case = args.get("use_case").and_then(|v| v.as_str()).unwrap_or("service");
                
                Some(vec![
                    PromptMessage::user(format!(
                        "Design a USSD menu flow for: {}. \
                         USSD menus should be max 160 chars per screen, \
                         with numbered options (1-9). \
                         Output as a tree structure in JSON.",
                        use_case
                    )),
                ])
            }
            _ => None,
        }
    }
}

impl Default for BrivasPrompts {
    fn default() -> Self {
        Self::new()
    }
}
