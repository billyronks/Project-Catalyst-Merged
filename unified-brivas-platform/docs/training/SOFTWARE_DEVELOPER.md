# Software Developer Training Manual

> **Role**: Software Developer  
> **Platform**: Unified Brivas Platform  
> **Version**: 1.0.0 | January 2026

---

## 1. Development Environment Setup

### 1.1 Prerequisites

```bash
# Rust (1.75+)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup default stable

# Tools
cargo install cargo-watch cargo-audit cargo-expand

# Clone repo
git clone https://github.com/billyronks/Project-Catalyst-Merged
cd Project-Catalyst-Merged/unified-brivas-platform
```

### 1.2 Project Structure

```
unified-brivas-platform/
├── shared/                    # Shared crates
│   ├── brivas-core/          # Common utilities
│   ├── brivas-lumadb/        # Database client
│   ├── brivas-kdb-sdk/       # QuestDB/kdb+ client
│   └── brivas-temporal-sdk/  # Workflow SDK
├── microservices/
│   ├── api-gateway/          # Unified API
│   ├── voice-switch/         # Carrier/LCR
│   ├── temporal-worker/      # Workflows
│   └── ...
├── infrastructure/
│   ├── docker/               # Dockerfiles
│   ├── xdp/                  # eBPF load balancer
│   └── questdb/              # Analytics schema
└── docs/
    └── training/             # You are here
```

---

## 2. Code Patterns

### 2.1 Axum Handler Pattern

```rust
use axum::{extract::State, Json};
use crate::{AppState, Error};

pub async fn list_carriers(
    State(state): State<AppState>,
) -> Result<Json<Vec<Carrier>>, Error> {
    let carriers = state.carrier_repo.list_all().await?;
    Ok(Json(carriers))
}

pub async fn create_carrier(
    State(state): State<AppState>,
    Json(req): Json<CreateCarrierRequest>,
) -> Result<Json<Carrier>, Error> {
    // Validate
    req.validate()?;
    
    // Create
    let carrier = state.carrier_repo.create(req).await?;
    
    // Invalidate cache
    state.cache.remove(&format!("carriers:all"));
    
    Ok(Json(carrier))
}
```

### 2.2 Error Handling

```rust
use thiserror::Error;
use axum::response::IntoResponse;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Carrier not found: {0}")]
    CarrierNotFound(Uuid),
    
    #[error("Invalid request: {0}")]
    InvalidRequest(String),
    
    #[error("Database error: {0}")]
    Database(#[from] tokio_postgres::Error),
}

impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        let (status, message) = match &self {
            Error::CarrierNotFound(_) => (StatusCode::NOT_FOUND, self.to_string()),
            Error::InvalidRequest(_) => (StatusCode::BAD_REQUEST, self.to_string()),
            Error::Database(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Internal error".to_string()),
        };
        
        (status, Json(json!({"error": message}))).into_response()
    }
}
```

### 2.3 Repository Pattern

```rust
pub struct CarrierRepository {
    pool: Arc<LumaDbPool>,
}

impl CarrierRepository {
    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<Carrier>> {
        let client = self.pool.get().await?;
        let row = client
            .query_opt("SELECT * FROM carriers WHERE id = $1", &[&id])
            .await?;
        
        Ok(row.map(Carrier::from_row))
    }
    
    pub async fn create(&self, req: CreateCarrierRequest) -> Result<Carrier> {
        let client = self.pool.get().await?;
        let id = Uuid::new_v4();
        
        client.execute(
            "INSERT INTO carriers (id, name, host, port) VALUES ($1, $2, $3, $4)",
            &[&id, &req.name, &req.host, &req.port],
        ).await?;
        
        self.find_by_id(id).await?.ok_or(Error::Internal)
    }
}
```

---

## 3. Working with Temporal Workflows

### 3.1 Defining a Workflow

```rust
// workflows.rs
pub async fn provision_service(
    input: ProvisionServiceInput,
) -> Result<ProvisionServiceOutput> {
    // Step 1: Validate customer
    let customer = activity::validate_customer(input.customer_id).await?;
    
    // Step 2: Allocate resources
    let resource = activity::allocate_did(input.area_code).await?;
    
    // Step 3: Configure routing
    activity::configure_routing(resource.id, input.destination).await?;
    
    // Step 4: Enable billing
    activity::enable_billing(input.customer_id, resource.id).await?;
    
    Ok(ProvisionServiceOutput {
        service_id: resource.id,
        status: "active".to_string(),
    })
}
```

### 3.2 Defining Activities

```rust
// activities.rs
pub async fn allocate_did(
    db: &DbPool,
    customer_id: Uuid,
    area_code: &str,
) -> Result<AllocatedResource> {
    let client = db.get().await?;
    
    // Find available DID
    let row = client.query_one(
        "SELECT id, number FROM did_inventory 
         WHERE area_code = $1 AND status = 'available' 
         LIMIT 1 FOR UPDATE",
        &[&area_code],
    ).await?;
    
    // Mark as allocated
    client.execute(
        "UPDATE did_inventory SET status = 'allocated', customer_id = $2 WHERE id = $1",
        &[&row.get::<_, Uuid>("id"), &customer_id],
    ).await?;
    
    Ok(AllocatedResource {
        id: row.get("id"),
        value: row.get("number"),
    })
}
```

---

## 4. Database Access

### 4.1 LumaDB (PostgreSQL Protocol)

```rust
use brivas_lumadb::LumaDbPool;

let pool = LumaDbPool::new(&config.database_url).await?;
let client = pool.get().await?;

// Query
let rows = client.query(
    "SELECT id, name FROM carriers WHERE status = $1",
    &[&"active"],
).await?;

// Transaction
let mut tx = client.transaction().await?;
tx.execute("INSERT INTO ...", &[]).await?;
tx.execute("UPDATE ...", &[]).await?;
tx.commit().await?;
```

### 4.2 QuestDB (Analytics)

```rust
use crate::analytics::AnalyticsClient;

let analytics = AnalyticsClient::new().await?;

// Query traffic stats
let stats = analytics.get_traffic_stats().await?;

// Ingest CDR (high-speed via ILP)
analytics.publish_cdr(&cdr).await?;
```

---

## 5. Testing

### 5.1 Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_lcr_least_cost() {
        let carriers = vec![
            Carrier { name: "A".into(), rate: 0.03, asr: 90.0, pdd: 500 },
            Carrier { name: "B".into(), rate: 0.02, asr: 85.0, pdd: 600 },
        ];
        
        let result = lcr_route(&carriers, RoutingMode::LeastCost);
        
        assert_eq!(result[0].name, "B"); // Cheapest first
    }
}
```

### 5.2 Integration Tests

```rust
#[tokio::test]
async fn test_carrier_crud() {
    let app = setup_test_app().await;
    
    // Create
    let res = app.post("/api/v1/carriers")
        .json(&json!({"name": "Test", "host": "1.2.3.4", "port": 5060}))
        .send().await;
    assert_eq!(res.status(), 201);
    
    let carrier: Carrier = res.json().await;
    
    // Read
    let res = app.get(&format!("/api/v1/carriers/{}", carrier.id))
        .send().await;
    assert_eq!(res.status(), 200);
}
```

### 5.3 Running Tests

```bash
# All tests
cargo test --workspace

# Specific package
cargo test --package voice-switch

# With logging
RUST_LOG=debug cargo test -- --nocapture
```

---

## 6. API Guidelines

### 6.1 REST Conventions

| Method | Path | Action |
|--------|------|--------|
| GET | /resources | List all |
| GET | /resources/:id | Get one |
| POST | /resources | Create |
| PATCH | /resources/:id | Partial update |
| PUT | /resources/:id | Full replace |
| DELETE | /resources/:id | Delete |

### 6.2 Response Format

```json
// Success
{
  "data": { ... },
  "meta": { "request_id": "uuid", "timestamp": "iso8601" }
}

// Error
{
  "error": {
    "code": "CARRIER_NOT_FOUND",
    "message": "Carrier with ID xyz not found",
    "details": { ... }
  }
}
```

---

## 7. Contributing Guidelines

### 7.1 Branch Naming

```
feature/add-carrier-failover
fix/lcr-null-rate-bug
refactor/carrier-repository
docs/update-api-docs
```

### 7.2 Commit Messages

```
feat(voice-switch): add carrier failover logic

- Implement health check loop
- Add automatic failover on 3 consecutive failures
- Update LCR to exclude unhealthy carriers

Closes #123
```

### 7.3 Code Review Checklist

- [ ] Tests pass locally
- [ ] `cargo clippy` warnings addressed
- [ ] `cargo fmt` applied
- [ ] Documentation updated
- [ ] No secrets in code
- [ ] Error handling is proper

---

## 8. Debugging

### 8.1 Logging

```rust
use tracing::{info, warn, error, debug, instrument};

#[instrument(skip(pool))]
pub async fn process_call(pool: &DbPool, call_id: Uuid) -> Result<()> {
    debug!(?call_id, "Processing call");
    
    // ... logic
    
    if quality < threshold {
        warn!(?call_id, %quality, "Low quality detected");
    }
    
    info!(?call_id, "Call processed successfully");
    Ok(())
}
```

### 8.2 Environment Variables

```bash
# Enable debug logging
RUST_LOG=voice_switch=debug,sqlx=trace cargo run

# Filter specific modules
RUST_LOG=voice_switch::lcr=trace cargo run
```

---

## 9. Performance Tips

### 9.1 Connection Pooling

```rust
// Good: Use pool, connection returned automatically
let client = pool.get().await?;
let result = client.query(...).await?;
// client dropped here, returns to pool

// Bad: Creating new connection per request
let client = tokio_postgres::connect(...).await?;
```

### 9.2 Caching

```rust
use dashmap::DashMap;

pub struct CarrierCache {
    cache: DashMap<String, (Carrier, Instant)>,
    ttl: Duration,
}

impl CarrierCache {
    pub fn get(&self, id: &str) -> Option<Carrier> {
        self.cache.get(id)
            .filter(|e| e.1.elapsed() < self.ttl)
            .map(|e| e.0.clone())
    }
}
```

### 9.3 Async Best Practices

```rust
// Good: Concurrent execution
let (carriers, routes) = tokio::join!(
    repo.list_carriers(),
    repo.list_routes(),
);

// Bad: Sequential when not needed
let carriers = repo.list_carriers().await?;
let routes = repo.list_routes().await?;
```
