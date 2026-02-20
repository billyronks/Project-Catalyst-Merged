# Gap Analysis -- Project Catalyst (Unified Brivas Platform)

## 1. Executive Summary

Project Catalyst is a carrier-grade unified telecom platform built as a Rust-centric workspace
with 33 Cargo.toml files (1 workspace root, 12 shared crates, 19 microservices, 1 SIGTRAN
protocol crate), a Go-based API gateway (apps/api-gateway), and a Next.js 15 landing service.
The platform targets 100,000+ TPS across 5 global PoPs (Lagos, London, Ashburn, Sao Paulo,
Singapore) with LumaDB as the unified data layer.

This document identifies gaps between the current implementation and production readiness.

---

## 2. Codebase Inventory

### 2.1 Rust Workspace (33 Cargo.toml)

| Category | Count | Members |
|----------|-------|---------|
| Workspace root | 1 | `Cargo.toml` (resolver = "2") |
| Shared crates | 12 | brivas-core, brivas-lumadb, brivas-proto, brivas-telemetry, brivas-lb-health, brivas-im-sdk, brivas-rcs-sdk, brivas-mcp-sdk, brivas-stir-shaken-sdk, brivas-video-sdk, brivas-kdb-sdk, brivas-temporal-sdk |
| SIGTRAN crate | 1 | brivas-sigtran (M3UA/SCCP/TCAP/MAP) |
| Telecom services | 5 | smsc, ussd-gateway, unified-messaging, voice-video-calling, voice-switch |
| Messaging services | 2 | instant-messaging, rcs-messaging |
| Business services | 3 | billing, payment-service, user-service |
| Integration services | 2 | hasura-bridge, mcp-gateway |
| Infrastructure services | 5 | pop-controller, aiops-engine, gitops-controller, dify-orchestrator, temporal-worker |
| Security services | 1 | stir-shaken-service |
| Gateway (Rust) | 1 | api-gateway (Rust/Axum) |

### 2.2 Go Layer

| Component | Path | Framework |
|-----------|------|-----------|
| API Gateway (Go) | `apps/api-gateway/gateway.go` | chi/v5, graphql-go, gorilla/websocket |
| Go main server | `cmd/server/main.go` | chi, grpc-gateway |
| LumaDB client | `packages/lumadb-client/client.go` | lib/pq |
| LLM Orchestrator | `packages/llm-orchestrator/orchestrator.go` | custom |
| Core Auth | `packages/core/auth.go` | custom |
| AI Service | `services/ai-service/service.go` | custom |
| SMS Service | `services/sms-service/service.go` | custom |
| XDP Controller | `infrastructure/xdp/controller/main.go` | custom |

### 2.3 Node.js Layer

| Component | Path | Framework |
|-----------|------|-----------|
| Landing Service | `microservices/landing-service` | Next.js 15, React 19, TypeScript 5 |

### 2.4 SQL Schemas (LumaDB)

8 schema files covering: gateway, users, ussd, smsc, messaging, billing, payments, plus 1 migration.

### 2.5 Infrastructure

| Component | Status |
|-----------|--------|
| docker-compose.yml | Complete -- 25+ services including LumaDB, QuestDB, ClickHouse, Redpanda, NATS, Kamailio, OpenSIPS, rtpengine, coTURN, Temporal, Consul, Grafana, Jaeger, Prometheus, Homer |
| Kubernetes base | 8 manifests (namespace, configmap, 6 services) |
| Kubernetes overlays | Lagos, Ashburn |
| Helm charts | brivas-platform, voice-stack, cilium-pop |
| CI/CD | GitHub Actions (rust-lint, rust-test, rust-security, rust-build, go-lint, go-test, docker, deploy-staging, deploy-production) |
| Global deploy | geodns-config, pop-template, pop-topology, service-matrix, service-mesh |

---

## 3. Identified Gaps

### 3.1 Critical Gaps (P0)

| # | Gap | Impact | Current State |
|---|-----|--------|---------------|
| G-01 | **No integration tests** | Cannot verify cross-service communication | Unit test stubs only in billing and user-service |
| G-02 | **SMPP server is a skeleton** | SMSC cannot accept real SMPP binds | `SmppServer` struct exists but `run()` is stub |
| G-03 | **No Dockerfile for most services** | Cannot build container images for 15+ services | `docker-compose.yml` references missing Dockerfiles |
| G-04 | **Payment providers are partially wired** | Paystack/Flutterwave integration incomplete | Provider trait defined; actual HTTP calls placeholder |
| G-05 | **LumaDB dependency unverified** | LumaDB is not a publicly available database | All services assume `lumadb/lumadb:latest` image exists |
| G-06 | **No TLS/mTLS configuration** | All inter-service traffic is plaintext | No cert management, no service mesh TLS |
| G-07 | **No secret management** | Credentials are env vars with defaults | No Vault, no sealed secrets, passwords in compose |

### 3.2 High-Priority Gaps (P1)

| # | Gap | Impact | Current State |
|---|-----|--------|---------------|
| G-08 | **SIGTRAN not connected to SMSC** | SS7 signaling stack is standalone crate | No wiring from `brivas-sigtran` to `smsc` main.rs |
| G-09 | **No gRPC definitions compiled** | Services reference gRPC but no .proto files found | brivas-proto crate exists but no .proto source |
| G-10 | **Voice signaling stack untested** | Kamailio/OpenSIPS/FreeSWITCH configs missing | Only deployment YAML exists, no actual config files |
| G-11 | **GraphQL schema incomplete** | Hasura Bridge has stub GraphQL handler | `graphql_handler()` returns static JSON |
| G-12 | **No load testing harness** | Cannot verify 100K TPS target | No k6/locust/gatling scripts |
| G-13 | **Temporal workflows are stubs** | No actual workflow definitions registered | Worker starts health server only |
| G-14 | **No database migrations runner** | Schema files exist but no migration tool | Single `001_initial_schema.sql` migration |

### 3.3 Medium-Priority Gaps (P2)

| # | Gap | Impact | Current State |
|---|-----|--------|---------------|
| G-15 | **Duplicate gateway implementations** | Go gateway and Rust gateway serve same purpose | Two separate implementations (apps/api-gateway in Go, microservices/api-gateway in Rust) |
| G-16 | **No rate limiting persistence** | Rate limiter resets on restart | In-memory DashMap in RateLimiter |
| G-17 | **WebRTC TURN/STUN config incomplete** | Voice calls will fail behind NAT | coTURN container defined but config file missing |
| G-18 | **No backup/restore procedures** | Data loss risk | No backup scripts, no PIT recovery |
| G-19 | **Observability partially wired** | brivas-telemetry exists but no OTel export in services | Tracing initialized with fmt subscriber, not OTel |
| G-20 | **No API versioning strategy** | Breaking changes will affect clients | `/v1/` prefix used but no version negotiation |
| G-21 | **Missing error codes catalog** | Inconsistent error responses | Each service defines own errors |

### 3.4 Low-Priority Gaps (P3)

| # | Gap | Impact | Current State |
|---|-----|--------|---------------|
| G-22 | **No SDK generation** | Clients must manually construct API calls | No OpenAPI spec, no client SDKs |
| G-23 | **Documentation gaps** | No API reference, no runbooks | README.md at root and platform level only |
| G-24 | **No chaos engineering** | Unknown failure modes | No Litmus/Chaos Monkey integration |
| G-25 | **No multi-tenancy enforcement** | Tenant isolation in queries is manual | tenant_id in queries but no middleware enforcement |
| G-26 | **XDP controller incomplete** | Network-level acceleration not available | Go file exists but minimal implementation |

---

## 4. Architecture Concerns

### 4.1 LumaDB Risk

LumaDB is referenced as a unified data layer exposing PostgreSQL (5432), Redis (6379),
MongoDB (27017), and Kafka (9092) wire protocols. If this is a proprietary/unreleased product,
the entire platform has a single point of dependency risk. Mitigation: provide a PostgreSQL
fallback mode with Redis and Kafka as separate services.

### 4.2 Dual Gateway Problem

The codebase contains two API gateway implementations:
- **Go** (`apps/api-gateway/gateway.go`): Full-featured with schema introspection, GraphQL generation, REST auto-CRUD, WebSocket subscriptions, MCP tools. 1097 lines.
- **Rust** (`microservices/api-gateway/src/main.rs`): Axum-based with stub handlers, WAF, rate limiter, OAuth. 157 lines.

Recommendation: Consolidate to a single gateway. The Go implementation is more mature for
schema-driven auto-generation; the Rust implementation is better for raw performance.

### 4.3 Voice Stack Complexity

The voice infrastructure spans 5 components (Kamailio SBC, OpenSIPS, FreeSWITCH, rtpengine,
coTURN) plus 2 Rust microservices (voice-switch, voice-video-calling). Configuration files
for the SIP/RTP stack are referenced but not present.

---

## 5. Test Coverage Assessment

| Layer | Files | Test Coverage |
|-------|-------|---------------|
| Rust shared crates | 12 crates | No test files found |
| Rust microservices | 19 services | `#[cfg(test)] mod tests` in billing and user-service only |
| Go services | 11 .go files | 2 test files (gateway_test.go, orchestrator_test.go, service_test.go) |
| SQL schemas | 8 files | No validation tests |
| Infrastructure | Helm/K8s | No Helm test hooks |

**Estimated overall test coverage: <5%**

---

## 6. Dependency Health

### Rust Dependencies (workspace)
- tokio 1.35, axum 0.8, tonic 0.11, prost 0.12 -- current
- async-graphql 7.0 -- current
- x25519-dalek 2.0, aes-gcm 0.10 -- current
- reqwest 0.12 -- current
- **Risk**: tokio-postgres 0.7 / deadpool-postgres 0.12 -- adequate

### Go Dependencies
- go 1.22, chi/v5 5.0.12, go-redis/v9, kafka-go 0.4.47 -- current
- grpc v1.62.1, protobuf v1.33.0 -- current
- **Risk**: gorilla/websocket maintenance status

### Node.js Dependencies
- Next.js 15.0.0, React 19.0.0 -- cutting edge
- **Risk**: React 19 is recent; ecosystem compatibility not fully validated

---

## 7. Remediation Roadmap

### Phase 1: Foundation (Weeks 1-2)
- [ ] Create Dockerfiles for all 19 Rust microservices
- [ ] Wire SIGTRAN crate into SMSC service
- [ ] Add .proto files for inter-service gRPC
- [ ] Implement TLS termination at gateway level
- [ ] Set up Vault or sealed-secrets for credential management

### Phase 2: Core Completeness (Weeks 3-6)
- [ ] Complete SMPP v3.4 server implementation
- [ ] Wire Temporal workflows (provisioning, billing sagas)
- [ ] Implement payment provider HTTP integrations
- [ ] Add integration test suite with testcontainers
- [ ] Create database migration runner (refinery or sqlx-migrate)

### Phase 3: Hardening (Weeks 7-10)
- [ ] Load test suite targeting 100K TPS
- [ ] Voice stack configuration (Kamailio, OpenSIPS, FreeSWITCH)
- [ ] Multi-tenancy middleware enforcement
- [ ] Chaos engineering framework
- [ ] Backup/restore automation

### Phase 4: Production Readiness (Weeks 11-14)
- [ ] Complete observability pipeline (OTel export from all services)
- [ ] API documentation and SDK generation
- [ ] Security audit and penetration testing
- [ ] Runbook creation for all services
- [ ] Multi-PoP deployment validation

---

## 8. Summary Statistics

| Metric | Value |
|--------|-------|
| Total Cargo.toml files | 33 |
| Rust microservices | 19 |
| Go source files | 11 |
| Node.js services | 1 |
| SQL schema files | 9 |
| Helm charts | 3 |
| Docker Compose services | 25+ |
| Global PoPs planned | 5 |
| Estimated lines of Rust | ~15,000 |
| Estimated lines of Go | ~3,000 |
| Identified gaps | 26 |
| Critical gaps (P0) | 7 |
| Test coverage | <5% |
