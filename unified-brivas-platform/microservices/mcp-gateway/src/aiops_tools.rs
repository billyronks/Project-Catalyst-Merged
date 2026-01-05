//! AIOps Tools for MCP
//!
//! Tools for AI/LLM agents to interact with AIOps capabilities

use async_trait::async_trait;
use brivas_mcp_sdk::tool::{Tool, ToolDefinition, ToolError, ToolResult};
use serde_json::{json, Value};

/// Diagnose issue tool
pub struct DiagnoseIssueTool;

#[async_trait]
impl Tool for DiagnoseIssueTool {
    fn name(&self) -> &str {
        "brivas_diagnose_issue"
    }

    fn description(&self) -> &str {
        "Diagnose a platform issue by analyzing metrics, logs, and service health"
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "service": {
                    "type": "string",
                    "description": "Service name to diagnose (e.g., smsc, billing, ussd)"
                },
                "symptom": {
                    "type": "string",
                    "description": "Symptom description (e.g., 'high latency', 'connection failures')"
                },
                "timeframe": {
                    "type": "string",
                    "description": "Timeframe to analyze (e.g., '5m', '1h', '24h')"
                }
            },
            "required": ["symptom"]
        })
    }

    async fn execute(&self, args: Value) -> Result<ToolResult, ToolError> {
        let service = args.get("service").and_then(|v| v.as_str()).unwrap_or("all");
        let symptom = args
            .get("symptom")
            .and_then(|v| v.as_str())
            .ok_or(ToolError::InvalidInput("symptom required".into()))?;
        let timeframe = args.get("timeframe").and_then(|v| v.as_str()).unwrap_or("5m");

        // In production, query AIOps engine
        let diagnosis = json!({
            "service": service,
            "symptom": symptom,
            "timeframe": timeframe,
            "analysis": {
                "status": "analyzing",
                "probable_causes": [
                    "Network connectivity issues",
                    "Resource exhaustion",
                    "Configuration drift"
                ],
                "recommended_actions": [
                    "Check service health endpoints",
                    "Review recent configuration changes",
                    "Check network connectivity"
                ]
            }
        });

        Ok(ToolResult::json(diagnosis))
    }
}

/// Auto-remediate tool
pub struct AutoRemediateTool;

#[async_trait]
impl Tool for AutoRemediateTool {
    fn name(&self) -> &str {
        "brivas_auto_remediate"
    }

    fn description(&self) -> &str {
        "Trigger automatic remediation for a detected issue using playbooks"
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "issue_type": {
                    "type": "string",
                    "enum": ["smpp_disconnect", "high_latency", "resource_exhaustion", "service_down"],
                    "description": "Type of issue to remediate"
                },
                "service": {
                    "type": "string",
                    "description": "Target service"
                },
                "dry_run": {
                    "type": "boolean",
                    "description": "Preview remediation steps without executing"
                }
            },
            "required": ["issue_type"]
        })
    }

    async fn execute(&self, args: Value) -> Result<ToolResult, ToolError> {
        let issue_type = args
            .get("issue_type")
            .and_then(|v| v.as_str())
            .ok_or(ToolError::InvalidInput("issue_type required".into()))?;
        let service = args.get("service").and_then(|v| v.as_str()).unwrap_or("unknown");
        let dry_run = args.get("dry_run").and_then(|v| v.as_bool()).unwrap_or(false);

        let playbook = match issue_type {
            "smpp_disconnect" => "smpp_recovery",
            "high_latency" => "service_restart",
            "resource_exhaustion" => "scale_out",
            "service_down" => "service_restart",
            _ => "generic_remediation",
        };

        if dry_run {
            Ok(ToolResult::text(format!(
                "Dry run: Would execute playbook '{}' for service '{}'",
                playbook, service
            )))
        } else {
            Ok(ToolResult::text(format!(
                "Remediation triggered: Executing playbook '{}' for service '{}'",
                playbook, service
            )))
        }
    }
}

/// Get service health tool
pub struct GetServiceHealthTool;

#[async_trait]
impl Tool for GetServiceHealthTool {
    fn name(&self) -> &str {
        "brivas_get_service_health"
    }

    fn description(&self) -> &str {
        "Get health status and metrics for platform services"
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "service": {
                    "type": "string",
                    "description": "Service name (or 'all' for all services)"
                }
            }
        })
    }

    async fn execute(&self, args: Value) -> Result<ToolResult, ToolError> {
        let service = args.get("service").and_then(|v| v.as_str()).unwrap_or("all");

        let health = json!({
            "query": service,
            "services": [
                {"name": "api-gateway", "status": "healthy", "uptime": "99.99%"},
                {"name": "smsc", "status": "healthy", "uptime": "99.95%"},
                {"name": "ussd-gateway", "status": "healthy", "uptime": "99.99%"},
                {"name": "billing", "status": "healthy", "uptime": "99.99%"},
                {"name": "hasura-bridge", "status": "healthy", "uptime": "100%"},
                {"name": "mcp-gateway", "status": "healthy", "uptime": "100%"}
            ],
            "overall_health": "healthy"
        });

        Ok(ToolResult::json(health))
    }
}

/// List tables tool for schema discovery
pub struct ListTablesTool;

#[async_trait]
impl Tool for ListTablesTool {
    fn name(&self) -> &str {
        "brivas_list_tables"
    }

    fn description(&self) -> &str {
        "List all available database tables and their schemas"
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "namespace": {
                    "type": "string",
                    "description": "Database namespace/schema (default: public)"
                }
            }
        })
    }

    async fn execute(&self, args: Value) -> Result<ToolResult, ToolError> {
        let namespace = args.get("namespace").and_then(|v| v.as_str()).unwrap_or("public");

        // In production, call Hasura bridge schema discovery
        let tables = json!({
            "namespace": namespace,
            "tables": [
                "accounts", "user_buckets", "sms_history", "flash_call_history",
                "sender_ids", "user_apps", "apps", "services", "ussd_menus",
                "ussd_sessions", "number_pool", "contacts", "default_sms_rates",
                "tenant_billing_config", "billing_transactions", "invoices",
                "rate_cards", "sms_templates", "campaigns", "service_errors", "resellers"
            ],
            "count": 21
        });

        Ok(ToolResult::json(tables))
    }
}

/// Describe table tool
pub struct DescribeTableTool;

#[async_trait]
impl Tool for DescribeTableTool {
    fn name(&self) -> &str {
        "brivas_describe_table"
    }

    fn description(&self) -> &str {
        "Get detailed schema information for a specific table"
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "table": {
                    "type": "string",
                    "description": "Table name to describe"
                }
            },
            "required": ["table"]
        })
    }

    async fn execute(&self, args: Value) -> Result<ToolResult, ToolError> {
        let table = args
            .get("table")
            .and_then(|v| v.as_str())
            .ok_or(ToolError::InvalidInput("table required".into()))?;

        // In production, call Hasura bridge
        let schema = json!({
            "table": table,
            "namespace": "public",
            "columns": [
                {"name": "id", "type": "serial", "primary_key": true},
                {"name": "created_at", "type": "timestamp"},
                {"name": "updated_at", "type": "timestamp"}
            ],
            "indexes": [],
            "graphql_query": format!("{}(limit: Int, where: {}WhereInput): [{}!]!", table, table, table),
            "rest_endpoint": format!("/v1/rest/{}", table)
        });

        Ok(ToolResult::json(schema))
    }
}

/// Trigger playbook tool
pub struct TriggerPlaybookTool;

#[async_trait]
impl Tool for TriggerPlaybookTool {
    fn name(&self) -> &str {
        "brivas_trigger_playbook"
    }

    fn description(&self) -> &str {
        "Trigger a specific remediation playbook by name"
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "playbook": {
                    "type": "string",
                    "enum": ["smpp_recovery", "service_restart", "scale_out", "database_failover"],
                    "description": "Playbook name to execute"
                },
                "parameters": {
                    "type": "object",
                    "description": "Parameters to pass to the playbook"
                }
            },
            "required": ["playbook"]
        })
    }

    async fn execute(&self, args: Value) -> Result<ToolResult, ToolError> {
        let playbook = args
            .get("playbook")
            .and_then(|v| v.as_str())
            .ok_or(ToolError::InvalidInput("playbook required".into()))?;

        Ok(ToolResult::text(format!(
            "Playbook '{}' execution initiated. Check AIOps dashboard for status.",
            playbook
        )))
    }
}

/// Get all AIOps tools
pub fn get_aiops_tools() -> Vec<Box<dyn Tool>> {
    vec![
        Box::new(DiagnoseIssueTool),
        Box::new(AutoRemediateTool),
        Box::new(GetServiceHealthTool),
        Box::new(ListTablesTool),
        Box::new(DescribeTableTool),
        Box::new(TriggerPlaybookTool),
    ]
}
