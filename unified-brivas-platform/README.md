# Unified Brivas Platform

> Carrier-grade telecommunications platform targeting **100,000+ TPS** across 5 global PoPs.

## Quick Start

```bash
# Build all services
cargo build --release --workspace

# Run tests
cargo test --workspace

# Deploy to Lagos PoP
kubectl apply -k infrastructure/kubernetes/overlays/lagos/

# Deploy to Ashburn PoP
kubectl apply -k infrastructure/kubernetes/overlays/ashburn/
```

---

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                      API Gateway (Edge)                         │
│              HTTP/gRPC/GraphQL/WebSocket/MCP                    │
└───────────────┬─────────────────────────────────────────────────┘
                │
    ┌───────────┼───────────┬───────────┬───────────┬─────────────┐
    ▼           ▼           ▼           ▼           ▼             ▼
┌───────┐ ┌─────────┐ ┌──────────┐ ┌─────────┐ ┌─────────┐ ┌───────────┐
│ USSD  │ │  SMSC   │ │ Messaging│ │ Billing │ │ Payment │ │   User    │
│Gateway│ │         │ │   Hub    │ │ Service │ │ Service │ │  Service  │
└───┬───┘ └────┬────┘ └────┬─────┘ └────┬────┘ └────┬────┘ └─────┬─────┘
    │          │           │            │           │             │
    └──────────┴───────────┴────────────┴───────────┴─────────────┘
                                │
                         ┌──────▼──────┐
                         │   LumaDB    │
                         │  (Direct)   │
                         └─────────────┘
```

---

## Microservices

| Service | Port | Description |
|---------|------|-------------|
| **API Gateway** | 80/443/9090/3000 | Edge routing, auth, rate limiting |
| **USSD Gateway** | 8080/50051 | MAP/TCAP sessions, dynamic menus |
| **SMSC** | 2775/2776/50052 | SMPP v3.4/5.0, carrier routing |
| **Messaging Hub** | 8080/50053 | 16 platform adapters |
| **Billing Service** | 8080/50054 | CDRs, rating, invoicing |
| **Payment Service** | 8080/50055 | Multi-gateway, PCI-DSS |
| **User Service** | 8080/50056 | Auth, MFA, RBAC |

---

## LumaDB Schemas

All services use **direct LumaDB integration** (NO ORM/adapters):

```
microservices/
├── api-gateway/schema/gateway.sql
├── user-service/schema/users.sql
├── ussd-gateway/schema/ussd.sql
├── smsc/schema/smsc.sql
├── unified-messaging/schema/messaging.sql
├── billing/schema/billing.sql
└── payment-service/schema/payments.sql
```

---

## Global PoPs

| PoP | Region | Currency | STIR/SHAKEN |
|-----|--------|----------|-------------|
| **Lagos** | Africa West | NGN | No |
| **London** | Europe West | GBP | No |
| **Ashburn** | US East | USD | Yes |
| **São Paulo** | South America | BRL | No |
| **Singapore** | Asia Pacific | SGD | No |

---

## Performance Targets

| Metric | Target |
|--------|--------|
| SMS TPS | 100,000+ per PoP |
| USSD Sessions | 50,000 concurrent |
| API Latency (p99) | <10ms |
| DB Latency (p99) | <5ms |
| Availability | 99.99% |

---

## Kubernetes Deployment

```bash
# Base manifests
infrastructure/kubernetes/base/
├── kustomization.yaml
├── namespace.yaml
├── configmap.yaml
├── api-gateway.yaml
├── smsc.yaml
├── ussd-gateway.yaml
├── messaging-hub.yaml
├── billing-service.yaml
├── payment-service.yaml
└── user-service.yaml

# PoP overlays
infrastructure/kubernetes/overlays/
├── lagos/kustomization.yaml
└── ashburn/kustomization.yaml
```

---

## Implementation Timeline

| Phase | Weeks | Focus |
|-------|-------|-------|
| Foundation | 1-4 | Workspace, CI/CD, schemas |
| Core Services | 5-12 | USSD, SMSC, Messaging |
| Business Services | 13-18 | Billing, Payment, User |
| Edge & Integration | 19-22 | Gateway, Landing, testing |
| Deployment | 23-26 | Multi-PoP rollout |

---

## License

Proprietary - Brivas Technologies
