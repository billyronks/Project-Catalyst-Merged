# ADR-001: LumaDB as Sole Data Layer

## Status
Accepted

## Context
The consolidated platform needed to replace multiple data technologies:
- PostgreSQL/MySQL for relational data
- MongoDB for document storage
- Redis for caching
- Kafka for event streaming
- Prometheus for metrics

Managing these disparate systems created operational complexity.

## Decision
Use LumaDB as a **100% direct drop-in replacement** with native wire protocol support for PostgreSQL, Redis, MongoDB, and Kafka. No adapters or abstraction layers.

## Consequences

**Positive:**
- Single data layer to manage
- Simplified deployment and ops
- Native wire protocol = zero code changes to clients
- Unified backup/recovery

**Negative:**
- Dependency on single vendor
- Team needs LumaDB training

---

# ADR-002: Hasura-Style API Generation

## Status
Accepted

## Context
Building individual REST/GraphQL endpoints for 25+ tables is time-consuming and error-prone.

## Decision
Implement Hasura-style auto-generation that introspects LumaDB schema and generates:
- GraphQL queries/mutations/subscriptions
- RESTful CRUD endpoints
- gRPC service definitions
- WebSocket channels
- MCP tools for LLM integration

## Consequences

**Positive:**
- New tables automatically get full API coverage
- Consistent API patterns
- Reduced development time

**Negative:**
- Complex custom logic requires additional handlers
- Schema changes require API regeneration

---

# ADR-003: Multi-Provider LLM Orchestration

## Status
Accepted

## Context
AI features require LLM integration but:
- No single provider offers best-in-class for all use cases
- Vendor lock-in is risky
- On-premises deployment needed for sensitive data

## Decision
Build multi-provider LLM orchestrator supporting:
- Google Gemini (primary)
- OpenAI GPT-4
- Anthropic Claude
- On-premises Llama

With intelligent routing, fallback chains, and response caching.

## Consequences

**Positive:**
- Provider flexibility
- Cost optimization possible
- Graceful degradation on failures
- On-premises option for compliance

**Negative:**
- Increased complexity
- Multiple API integrations to maintain

---

# ADR-004: Go for Backend Services

## Status
Accepted

## Context
Original codebase used Node.js (Express). For the 685,000 TPS target:
- Node.js event loop can bottleneck
- Go offers better concurrency primitives
- Lower memory footprint

## Decision
Rewrite core services in Go 1.22+ using:
- Chi router for HTTP
- Standard library for concurrency
- lib/pq for PostgreSQL (LumaDB)

Preserve Node.js for admin dashboard BFF.

## Consequences

**Positive:**
- 10x+ performance improvement
- Lower resource consumption
- Better type safety

**Negative:**
- Team needs Go expertise
- Some existing Node.js utilities must be rewritten

---

# ADR-005: Monorepo Structure

## Status
Accepted

## Context
20 separate repositories created:
- Dependency version conflicts
- Difficult cross-repo refactoring
- CI/CD duplication

## Decision
Consolidate into single monorepo:
```
unified-brivas-platform/
├── apps/          # Deployable applications
├── packages/      # Shared libraries
├── services/      # Microservices
└── infrastructure/ # IaC
```

## Consequences

**Positive:**
- Atomic commits across components
- Shared dependency management
- Unified CI/CD

**Negative:**
- Larger repo size
- Need for careful access controls
