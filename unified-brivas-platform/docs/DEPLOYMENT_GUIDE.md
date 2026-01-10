# Deployment Guide - Unified Brivas Platform

> **Version**: 1.0.0 | January 2026  
> **Platform**: Carrier-Grade Telecommunications

---

## 1. Prerequisites

### 1.1 System Requirements

| Component | Minimum | Recommended |
|-----------|---------|-------------|
| CPU | 8 cores | 16+ cores |
| Memory | 16 GB | 32+ GB |
| Storage | 100 GB SSD | 500 GB NVMe |
| Network | 1 Gbps | 10+ Gbps |

### 1.2 Software Requirements

```bash
# Docker & Docker Compose
docker --version  # >= 24.0
docker-compose --version  # >= 2.20

# Kubernetes (optional)
kubectl version  # >= 1.28

# Rust (for development)
rustc --version  # >= 1.75
```

---

## 2. Quick Start (Docker Compose)

### 2.1 Clone & Configure

```bash
git clone https://github.com/billyronks/Project-Catalyst-Merged
cd Project-Catalyst-Merged/unified-brivas-platform

# Configure environment
cp .env.example .env
# Edit .env with your settings
```

### 2.2 Start Services

```bash
# Core services only
docker-compose up -d lumadb nats questdb temporal api-gateway

# Full platform
docker-compose up -d

# With analytics
docker-compose --profile analytics up -d

# With service mesh
docker-compose --profile mesh up -d

# With voice signaling
docker-compose --profile signaling up -d
```

### 2.3 Verify Deployment

```bash
# Check all services
docker-compose ps

# Health checks
curl http://localhost:8080/health   # API Gateway
curl http://localhost:8095/health   # Voice Switch
curl http://localhost:9000          # QuestDB Console
curl http://localhost:8088          # Temporal UI
```

---

## 3. Production Deployment

### 3.1 Kubernetes Deployment

```bash
# Create namespace
kubectl create namespace brivas

# Apply secrets
kubectl create secret generic brivas-secrets \
  --from-literal=db-password=$DB_PASS \
  --from-literal=jwt-secret=$JWT_SECRET \
  -n brivas

# Deploy data layer
kubectl apply -f infrastructure/kubernetes/data/ -n brivas

# Deploy services
kubectl apply -f infrastructure/kubernetes/services/ -n brivas

# Deploy ingress
kubectl apply -f infrastructure/kubernetes/ingress/ -n brivas
```

### 3.2 Resource Allocation

```yaml
# voice-switch deployment
resources:
  requests:
    cpu: "1"
    memory: "1Gi"
  limits:
    cpu: "4"
    memory: "4Gi"

# questdb statefulset
resources:
  requests:
    cpu: "2"
    memory: "4Gi"
  limits:
    cpu: "8"
    memory: "16Gi"
```

### 3.3 High Availability

| Service | Replicas | Strategy |
|---------|----------|----------|
| API Gateway | 3+ | Rolling |
| Voice Switch | 3+ | Rolling |
| Temporal Worker | 5+ | Rolling |
| LumaDB | 3 (primary + replicas) | StatefulSet |
| QuestDB | 1 (with replication) | StatefulSet |

---

## 4. Service Configuration

### 4.1 Voice Signaling

| Service | Port | Purpose |
|---------|------|---------|
| Kamailio | 5060 (UDP/TCP) | Class 4 SIP |
| OpenSIPS | 5080 (UDP/TCP), 5066/5067 (WS/WSS) | Class 5 SIP |
| RTPEngine | 22222 | Media control |
| Coturn | 3478/5349 | STUN/TURN |

### 4.2 API Services

| Service | Port | Purpose |
|---------|------|---------|
| API Gateway | 8080 | REST/GraphQL |
| Voice Switch | 8095 | Carrier/LCR |
| Temporal UI | 8088 | Workflow monitoring |
| Hasura | 8082 | GraphQL Engine |

### 4.3 Data Services

| Service | Port | Purpose |
|---------|------|---------|
| LumaDB | 5432 | Primary PostgreSQL |
| QuestDB | 8812, 9009 | Analytics |
| ClickHouse | 8123, 9000 | OLAP |
| NATS | 4222 | Messaging |

---

## 5. Monitoring Setup

### 5.1 Grafana Dashboards

```bash
# Access Grafana
open http://localhost:3000

# Default credentials
# Username: admin
# Password: admin
```

**Pre-configured Dashboards:**
- Voice Platform Overview
- Carrier Performance
- CDR Analytics
- System Metrics

### 5.2 OpenTelemetry

```bash
# Collector receives traces on
# gRPC: 4317
# HTTP: 4318

# View traces in Jaeger
open http://localhost:16686
```

### 5.3 Homer SIP Tracing

```bash
# Access Homer UI
open http://localhost:9080

# HEP ingestion on port 9060
```

---

## 6. Security Checklist

### 6.1 Before Production

- [ ] Change all default passwords in `.env`
- [ ] Enable TLS for all public endpoints
- [ ] Configure firewall rules
- [ ] Set up network policies
- [ ] Enable audit logging
- [ ] Configure backup schedule

### 6.2 TLS Configuration

```bash
# Generate certificates (or use Let's Encrypt)
openssl req -x509 -nodes -days 365 \
  -newkey rsa:2048 \
  -keyout brivas.key \
  -out brivas.crt \
  -subj "/CN=brivas.io"

# Update nginx config
# Update Coturn config
# Update OpenSIPS for WSS
```

---

## 7. Backup & Recovery

### 7.1 Automated Backup

```bash
# LumaDB backup
pg_dump -h localhost -U brivas -d brivas | gzip > backup_$(date +%Y%m%d).sql.gz

# QuestDB snapshot
curl -X POST "http://localhost:9000/exec?query=SNAPSHOT+DATABASE"
```

### 7.2 Recovery

```bash
# Restore LumaDB
gunzip < backup_20260110.sql.gz | psql -h localhost -U brivas -d brivas

# Restore QuestDB
# Copy snapshot to /var/lib/questdb/db
```

---

## 8. Scaling Guide

### 8.1 Horizontal Scaling

```bash
# Scale voice-switch
docker-compose up -d --scale voice-switch=5

# Or in Kubernetes
kubectl scale deployment voice-switch --replicas=5 -n brivas
```

### 8.2 Capacity Planning

| Traffic Level | voice-switch | temporal-worker | LumaDB |
|---------------|--------------|-----------------|--------|
| 100 CPS | 2 | 2 | 1 primary |
| 500 CPS | 4 | 4 | 1 primary + 1 replica |
| 1000 CPS | 8 | 8 | 1 primary + 2 replicas |

---

## 9. Troubleshooting

### 9.1 Service Not Starting

```bash
# Check logs
docker-compose logs service-name

# Common issues:
# - Database not ready: Add depends_on or wait script
# - Port conflict: Check with netstat -tlnp
# - Out of memory: Increase limits
```

### 9.2 Call Quality Issues

```bash
# Check RTPEngine
docker exec brivas-rtpengine rtpengine-ctl list sessions

# View Homer SIP traces
open http://localhost:9080
```

### 9.3 Performance Issues

```bash
# Check resource usage
docker stats

# Query slow queries
# In QuestDB: EXPLAIN query
```

---

## 10. Support

| Channel | Purpose |
|---------|---------|
| GitHub Issues | Bug reports |
| Slack #platform-help | Quick questions |
| support@brivas.io | Enterprise support |
