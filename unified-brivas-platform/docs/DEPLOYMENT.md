# Deployment Guide

## Prerequisites

- Docker 24+ & Docker Compose
- Kubernetes 1.29+ (for production)
- Go 1.22+ (for local development)
- Node.js 20+ (for frontend)

## Local Development

### Quick Start

```bash
# Clone and enter directory
cd unified-brivas-platform

# Start all services
docker-compose up -d

# Verify services
curl http://localhost:8080/health
```

### Services Started

| Service | Port | Description |
|---------|------|-------------|
| LumaDB | 5432, 6379, 9092 | Unified data layer |
| API Gateway | 8080 | GraphQL/REST/WS/MCP |
| SMS Service | 8081 | SMS processing |
| Voice Service | 8082 | Voice/Flash calls |
| Billing Service | 8083 | Billing engine |
| AI Service | 8084 | LLM integration |
| Hasura Bridge | 8085 | GraphQL/REST auto-discovery |
| MCP Gateway | 8086 | Model Context Protocol server |
| AIOps Engine | 8087 | Autonomous IT operations |
| GitOps Controller | 8088 | Git-based configuration |
| Dify Orchestrator | 8089 | Dify AI workflows/agents |
| Customer Web | 3000 | Next.js frontend |
| Admin Dashboard | 3001 | Admin UI |
| Nginx | 80, 443 | Reverse proxy |

### Environment Variables

Create `.env` file:

```bash
# Database
LUMADB_PASSWORD=your_secure_password

# AI Provider Keys
GEMINI_API_KEY=your_gemini_key
OPENAI_API_KEY=your_openai_key
ANTHROPIC_API_KEY=your_anthropic_key

# Dify AI Integration
DIFY_API_KEY=your_dify_api_key
DIFY_BASE_URL=https://api.dify.ai/v1

# GitOps (optional)
GITOPS_REPO_URL=https://github.com/your-org/platform-config.git
GITOPS_BRANCH=main
```

---

## Production Deployment

### Kubernetes

1. **Create namespace:**
```bash
kubectl create namespace brivas
```

2. **Apply secrets:**
```bash
kubectl create secret generic brivas-secrets \
  --from-literal=LUMADB_PASSWORD=xxx \
  --from-literal=GEMINI_API_KEY=xxx \
  -n brivas
```

3. **Deploy services:**
```bash
kubectl apply -f infrastructure/kubernetes/ -n brivas
```

### Helm Chart

```bash
helm install brivas ./infrastructure/helm-charts/brivas \
  --namespace brivas \
  --set lumadb.password=xxx \
  --set api.replicas=3
```

---

## Scaling

### Horizontal Pod Autoscaling

```yaml
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: api-gateway-hpa
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: api-gateway
  minReplicas: 3
  maxReplicas: 50
  metrics:
    - type: Resource
      resource:
        name: cpu
        target:
          type: Utilization
          averageUtilization: 70
```

### LumaDB Sharding

Configure in `lumadb.yaml`:
```yaml
sharding:
  enabled: true
  shards: 4
  replication_factor: 3
```

---

## Monitoring

### Health Endpoints

```bash
# API Gateway health
curl http://localhost:8080/health

# Readiness check
curl http://localhost:8080/ready
```

### Metrics (Prometheus)

```bash
curl http://localhost:8080/metrics
```

### Key Metrics

| Metric | Description |
|--------|-------------|
| `api_requests_total` | Total API requests |
| `api_latency_seconds` | Request latency |
| `sms_sent_total` | SMS messages sent |
| `billing_events_processed` | Billing events |

---

## Backup & Recovery

### LumaDB Backup

```bash
# Create backup
docker exec brivas-lumadb lumadb backup /data/backup/$(date +%Y%m%d).bak

# Restore
docker exec brivas-lumadb lumadb restore /data/backup/20241225.bak
```

### Automated Backups

Add to crontab:
```bash
0 2 * * * docker exec brivas-lumadb lumadb backup /data/backup/$(date +\%Y\%m\%d).bak
```

---

## Troubleshooting

### Common Issues

**LumaDB connection refused:**
```bash
# Check container status
docker ps | grep lumadb

# Check logs
docker logs brivas-lumadb
```

**API Gateway unhealthy:**
```bash
# Check health endpoint
curl -v http://localhost:8080/health

# View logs
docker logs brivas-api-gateway
```

**SMS delivery failures:**
```bash
# Check DLR callback endpoints
curl http://localhost:8081/health

# View SMS service logs
docker logs brivas-sms-service
```
