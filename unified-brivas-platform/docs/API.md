# Unified Brivas Platform - API Documentation

## Overview

The Unified Brivas Platform provides a comprehensive telecommunications API with multiple protocol support:
- **GraphQL** - Full query/mutation capabilities with subscriptions
- **REST** - Standard CRUD operations
- **WebSocket** - Real-time event streaming
- **MCP** - Model Context Protocol for LLM integration

## Base URLs

| Environment | URL |
|-------------|-----|
| Development | `http://localhost:8080` |
| Staging | `https://api.staging.brivas.io` |
| Production | `https://api.brivas.io` |

## Authentication

All API requests require authentication via API key:

```bash
# Header authentication
Authorization: Bearer YOUR_API_KEY

# Or via custom header
X-API-Key: YOUR_API_KEY
```

---

## REST API

### Accounts

#### Get Account
```http
GET /api/v1/accounts/{id}
```

**Response:**
```json
{
  "id": "BV123456789",
  "email": "user@example.com",
  "first_name": "John",
  "balance": 1500.00,
  "is_verified": true
}
```

#### List Accounts
```http
GET /api/v1/accounts?limit=10&offset=0
```

#### Create Account
```http
POST /api/v1/accounts
Content-Type: application/json

{
  "email": "newuser@example.com",
  "first_name": "Jane",
  "last_name": "Doe",
  "password": "secure123"
}
```

---

### SMS

#### Send Single SMS
```http
POST /api/v1/sms/send
Content-Type: application/json

{
  "to": "+2348012345678",
  "from": "BRIVAS",
  "message": "Hello, World!"
}
```

**Response:**
```json
{
  "status": "success",
  "msg": "SMS sent",
  "data": {
    "sid": "BV123456-P2P-1703523420123"
  }
}
```

#### Send Bulk SMS
```http
POST /api/v1/sms/bulk
Content-Type: application/json

{
  "to": ["+2348012345678", "+2348023456789"],
  "from": "BRIVAS",
  "message": "Bulk message content",
  "type": "promotional"
}
```

**Response:**
```json
{
  "status": "success",
  "data": {
    "sid": "BV123456-BULK-1703523420123"
  }
}
```

#### Get SMS History
```http
GET /api/v1/sms/history?page=1
```

#### Get Balance
```http
GET /api/v1/sms/balance
```

---

### Campaigns

#### Create Campaign
```http
POST /api/v1/campaigns
Content-Type: application/json

{
  "name": "Holiday Promo 2024",
  "template_id": "tmpl_abc123",
  "sender_id": "BRIVAS",
  "scheduled_at": "2024-12-25T10:00:00Z"
}
```

#### List Campaigns
```http
GET /api/v1/campaigns?status=active
```

---

## GraphQL API

### Endpoint
```
POST /graphql
POST /v1/graphql
```

### Schema Introspection
```graphql
{
  __schema {
    types {
      name
    }
  }
}
```

### Queries

```graphql
# Get single account
query GetAccount($id: ID!) {
  account(id: $id) {
    id
    email
    balance
    first_name
  }
}

# List accounts with pagination
query ListAccounts($limit: Int, $offset: Int) {
  accounts(limit: $limit, offset: $offset) {
    id
    email
    balance
  }
}

# Get SMS history
query SMSHistory($limit: Int) {
  sms_histories(limit: $limit, orderBy: "id DESC") {
    sid
    recipient
    status
    sent_date
  }
}

# Campaign analytics
query CampaignStats($id: ID!) {
  campaign(id: $id) {
    campaign_id
    name
    total_recipients
    sent_count
    delivered_count
    failed_count
  }
}
```

### Mutations

```graphql
# Create account
mutation CreateAccount($email: String!, $first_name: String) {
  insert_accounts(object: "{\"email\": \"$email\", \"first_name\": \"$first_name\"}") {
    id
    email
  }
}

# Update account
mutation UpdateAccount($id: ID!, $balance: Float) {
  update_accounts(id: $id, _set: "{\"balance\": $balance}") {
    id
    balance
  }
}

# Create campaign
mutation CreateCampaign($name: String!, $template_id: String!) {
  insert_campaigns(object: "{\"name\": \"$name\", \"template_id\": \"$template_id\"}") {
    campaign_id
    status
  }
}
```

---

## WebSocket API

### Connection
```javascript
const ws = new WebSocket('ws://localhost:8080/ws');

ws.onopen = () => {
  // Subscribe to events
  ws.send(JSON.stringify({
    type: 'subscribe',
    channel: 'sms:delivery'
  }));
};

ws.onmessage = (event) => {
  const data = JSON.parse(event.data);
  console.log('Event:', data);
};
```

### Channels

| Channel | Description |
|---------|-------------|
| `sms:delivery` | SMS delivery reports |
| `campaigns:{id}` | Campaign status updates |
| `billing:events` | Real-time billing events |

---

## MCP API (LLM Integration)

### List Available Tools
```http
GET /mcp/tools
```

**Response:**
```json
{
  "tools": [
    {
      "name": "list_accounts",
      "description": "List account records with optional filters",
      "input_schema": {...}
    },
    {
      "name": "list_campaigns",
      "description": "List SMS campaigns with status filters",
      "input_schema": {...}
    }
  ]
}
```

### Execute Tool
```http
POST /mcp/tools/list_campaigns/execute
Content-Type: application/json

{
  "limit": 10,
  "status": "active"
}
```

---

## Error Codes

| Code | Description |
|------|-------------|
| 400 | Bad Request - Invalid parameters |
| 401 | Unauthorized - Invalid API key |
| 402 | Payment Required - Insufficient balance |
| 404 | Not Found - Resource doesn't exist |
| 429 | Rate Limited - Too many requests |
| 500 | Internal Error - Server error |

## Rate Limits

| Endpoint | Limit |
|----------|-------|
| Single SMS | 100/min |
| Bulk SMS | 10/min |
| GraphQL | 1000/min |
| REST | 500/min |

---

## Hasura Bridge - Schema Auto-Discovery (Port 8085)

### Get Full Schema
```http
GET /v1/schema
```

**Response:**
```json
{
  "tables": ["accounts", "sms_history", "campaigns", ...],
  "endpoints": [
    {"path": "/v1/rest/accounts", "methods": ["GET", "POST"]},
    {"path": "/v1/rest/accounts/:id", "methods": ["GET", "PUT", "DELETE"]}
  ]
}
```

### List Tables
```http
GET /v1/schema/tables
```

### Describe Table
```http
GET /v1/schema/tables/:table
```

### Auto-Generated REST Endpoints
```http
GET    /v1/rest/:table          # List records
POST   /v1/rest/:table          # Create record
GET    /v1/rest/:table/:id      # Get by ID
PUT    /v1/rest/:table/:id      # Update by ID
DELETE /v1/rest/:table/:id      # Delete by ID
```

---

## AIOps Engine - Autonomous IT Operations (Port 8087)

### Get Active Incidents
```http
GET /api/v1/incidents
```

### Trigger Manual Detection
```http
POST /api/v1/detect
```

### Execute Playbook
```http
POST /api/v1/playbooks/:playbook_id/execute
Content-Type: application/json

{
  "parameters": {
    "service": "smsc",
    "action": "restart"
  }
}
```

### Get Service Health
```http
GET /api/v1/health/services
```

**Response:**
```json
{
  "services": [
    {"name": "smsc", "status": "healthy", "uptime": "99.99%"},
    {"name": "billing", "status": "healthy", "uptime": "99.95%"}
  ]
}
```

---

## GitOps Controller (Port 8088)

### Get Sync Status
```http
GET /api/v1/sync/status
```

### Trigger Manual Sync
```http
POST /api/v1/sync/trigger
```

### List Applications
```http
GET /api/v1/applications
```

### Sync Specific Application
```http
POST /api/v1/applications/:name/sync
```

### Check Drift
```http
POST /api/v1/drift/check
```

**Response:**
```json
{
  "applications": [
    {"name": "smsc", "synced": true, "drift_detected": false},
    {"name": "billing", "synced": true, "drift_detected": true}
  ]
}
```

---

## Dify Orchestrator - AI Workflows & Agents (Port 8089)

### List Agents
```http
GET /api/v1/agents
```

**Response:**
```json
{
  "agents": [
    {"id": "customer_support", "name": "Customer Support Agent"},
    {"id": "aiops_analyst", "name": "AIOps Incident Analyst"},
    {"id": "developer_assistant", "name": "Developer API Assistant"}
  ]
}
```

### Chat with Agent
```http
POST /api/v1/agents/:id/chat
Content-Type: application/json

{
  "message": "What is my current account balance?",
  "conversation_id": "conv_abc123"
}
```

### List Workflows
```http
GET /api/v1/workflows
```

### Run Workflow
```http
POST /api/v1/workflows/:id/run
Content-Type: application/json

{
  "inputs": {
    "description": "Create a promotional SMS campaign for Black Friday",
    "target_audience": "premium_users",
    "channel": "sms"
  }
}
```

### AI Campaign Builder
```http
POST /api/v1/campaign-builder
Content-Type: application/json

{
  "description": "Send a holiday greeting to all customers",
  "target_audience": "all_users",
  "channel": "sms",
  "budget": 500.00
}
```

### Multi-Channel Support
```http
POST /api/v1/support
Content-Type: application/json

{
  "message": "I need help with my billing",
  "channel": "sms",
  "user_id": "user_123"
}
```

### AIOps Incident Analysis
```http
POST /api/v1/aiops/analyze
Content-Type: application/json

{
  "incident_id": "inc_001",
  "service": "smsc",
  "error_message": "Connection timeout",
  "metrics": {"latency_ms": 5000}
}
```

---

## MCP Gateway - Enhanced Tools (Port 8086)

### Available Tools

| Tool | Description |
|------|-------------|
| `brivas_diagnose_issue` | Analyze metrics/logs for issue diagnosis |
| `brivas_auto_remediate` | Trigger automatic remediation |
| `brivas_get_service_health` | Get health status for services |
| `brivas_list_tables` | List all database tables |
| `brivas_describe_table` | Get detailed table schema |
| `brivas_trigger_playbook` | Execute remediation playbook |
| `brivas_dify_chat` | Chat with Dify AI agent |
| `brivas_dify_workflow` | Execute Dify AI workflow |
| `brivas_dify_knowledge` | Search RAG knowledge base |
| `brivas_ai_campaign` | Create AI-powered campaign |

### Example: Diagnose Issue
```http
POST /mcp/tools/brivas_diagnose_issue/execute
Content-Type: application/json

{
  "service": "smsc",
  "symptom": "high latency",
  "timeframe": "15m"
}
```

### Example: AI Campaign via MCP
```http
POST /mcp/tools/brivas_ai_campaign/execute
Content-Type: application/json

{
  "description": "Announce our new VoIP service launch",
  "channel": "rcs",
  "budget": 1000
}
```

