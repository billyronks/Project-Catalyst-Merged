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
