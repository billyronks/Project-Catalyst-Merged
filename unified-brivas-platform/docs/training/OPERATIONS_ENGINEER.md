# Operations Engineer Training Manual

> **Role**: Operations Engineer  
> **Platform**: Unified Brivas Platform  
> **Version**: 1.0.0 | January 2026

---

## 1. Introduction

### 1.1 Role Overview
As an Operations Engineer, you are responsible for:
- Monitoring platform health and performance
- Managing carrier relationships and routing
- Troubleshooting call quality issues
- Ensuring SLA compliance (99.99% uptime)
- Capacity planning and scaling

### 1.2 Key Systems You'll Manage

| System | Purpose | Your Responsibility |
|--------|---------|---------------------|
| Voice Switch | Carrier routing | Add/configure carriers, LCR rules |
| QuestDB | CDR Analytics | Query call data, generate reports |
| Temporal | Workflows | Monitor provisioning, billing |
| Kamailio/OpenSIPS | SIP Signaling | Monitor registrations, SIP traces |
| Grafana | Dashboards | Set up alerts, review metrics |

---

## 2. Daily Operations Checklist

### Morning Health Check (5 min)

```bash
# 1. Check all services are running
docker-compose ps

# 2. Verify API Gateway health
curl http://localhost:8080/health

# 3. Check Voice Switch status
curl http://localhost:8095/health

# 4. View active call count
curl http://localhost:8095/api/v1/analytics/active-calls
```

### Key Metrics to Monitor

| Metric | Target | Alert Threshold |
|--------|--------|-----------------|
| CPS (Calls/Second) | <1000 | >800 |
| ASR (Answer-Seizure Ratio) | >55% | <45% |
| ACD (Avg Call Duration) | >120s | <60s |
| PDD (Post-Dial Delay) | <3s | >5s |
| Active Calls | - | >90% capacity |

---

## 3. Carrier Management

### 3.1 Adding a New Carrier

**Via API:**
```bash
curl -X POST http://localhost:8095/api/v1/carriers \
  -H "Content-Type: application/json" \
  -d '{
    "name": "CarrierXYZ",
    "host": "sip.carrierxyz.com",
    "port": 5060,
    "protocol": "UDP",
    "max_channels": 100,
    "username": "brivas",
    "password": "secure_password",
    "auth_type": "digest",
    "codecs": ["G.711", "G.729", "OPUS"]
  }'
```

**Response:**
```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "name": "CarrierXYZ",
  "status": "active",
  "created_at": "2026-01-10T12:00:00Z"
}
```

### 3.2 Carrier Status Management

| Status | Meaning | Action |
|--------|---------|--------|
| `active` | Receiving traffic | Normal operation |
| `draining` | No new calls, finishing existing | Pre-maintenance |
| `inactive` | Completely disabled | Maintenance/issues |
| `suspended` | Billing/compliance hold | Contact finance |

**Disable a carrier:**
```bash
curl -X PATCH http://localhost:8095/api/v1/carriers/{id} \
  -H "Content-Type: application/json" \
  -d '{"status": "inactive"}'
```

### 3.3 Viewing Carrier Statistics

```bash
# Get carrier stats for last 24h
curl http://localhost:8095/api/v1/carriers/{id}/stats

# Response includes:
# - total_calls, successful_calls, failed_calls
# - asr (%), acd (seconds), pdd (ms)
# - revenue, cost, margin
```

---

## 4. Routing Management

### 4.1 LCR Routing Modes

| Mode | Description | Use Case |
|------|-------------|----------|
| `least_cost` | Cheapest carrier first | Default wholesale |
| `quality` | Best ASR/PDD first | Premium traffic |
| `balanced` | Mix of cost + quality | General retail |
| `priority` | Fixed carrier order | Specific partners |
| `round_robin` | Equal distribution | Load testing |

### 4.2 Adding a Route

```bash
curl -X POST http://localhost:8095/api/v1/routes \
  -H "Content-Type: application/json" \
  -d '{
    "prefix": "234",
    "carrier_id": "550e8400-e29b-41d4-a716-446655440000",
    "rate": 0.025,
    "priority": 1,
    "routing_mode": "least_cost"
  }'
```

### 4.3 Testing a Route

```bash
# Find best route for a destination
curl "http://localhost:8095/api/v1/lcr/route?destination=2348012345678"

# Response:
{
  "destination": "2348012345678",
  "routing_mode": "least_cost",
  "carriers": [
    {"name": "CarrierA", "rate": 0.022, "dial_string": "sip:234...@carrier-a:5060"},
    {"name": "CarrierB", "rate": 0.025, "dial_string": "sip:234...@carrier-b:5060"}
  ]
}
```

---

## 5. Monitoring & Alerting

### 5.1 Grafana Dashboard Access

**URL**: `http://localhost:3000`  
**Default Login**: `admin / admin`

**Key Dashboards:**
1. **Real-Time Traffic** - CPS, Active Calls, ASR
2. **Carrier Performance** - Per-carrier metrics
3. **QoS Metrics** - Jitter, Packet Loss, MOS
4. **Revenue Dashboard** - Minutes, Revenue, Margin

### 5.2 QuestDB Direct Queries

**Access**: `http://localhost:9000` (Web Console)

**Useful Queries:**

```sql
-- Last hour's traffic summary
SELECT 
    count(*) as calls,
    avg(duration_secs) as avg_duration,
    sum(CASE WHEN disposition='answered' THEN 1.0 ELSE 0.0 END)/count(*)*100 as asr
FROM cdr
WHERE timestamp > dateadd('h', -1, now());

-- Top 10 destinations by volume
SELECT destination_number, count(*) as calls
FROM cdr
WHERE timestamp > dateadd('d', -1, now())
GROUP BY destination_number
ORDER BY calls DESC
LIMIT 10;

-- Carrier performance comparison
SELECT carrier_name, 
       count(*) as calls,
       avg(pdd_ms) as avg_pdd,
       sum(cost) as total_cost
FROM cdr
WHERE timestamp > dateadd('d', -1, now())
GROUP BY carrier_name
ORDER BY calls DESC;
```

### 5.3 Alert Thresholds

Set these in Grafana:

| Alert | Condition | Severity |
|-------|-----------|----------|
| High CPS | CPS > 800/sec | Warning |
| Low ASR | ASR < 45% for 5min | Critical |
| High PDD | PDD > 5sec avg | Warning |
| Carrier Down | 0 successful calls for 5min | Critical |
| Disk Space | >80% used | Warning |

---

## 6. Troubleshooting

### 6.1 Call Quality Issues

**Symptom**: High jitter or packet loss

```bash
# Check RTPEngine status
docker exec brivas-rtpengine rtpengine-ctl list sessions

# View QoS metrics
curl http://localhost:8095/api/v1/analytics/qos/{carrier_id}
```

**Solutions:**
1. Check network path to carrier
2. Verify codec compatibility
3. Consider switching to different carrier
4. Check for congestion on our side

### 6.2 Carrier Authentication Failures

```bash
# Check Kamailio logs
docker logs brivas-kamailio-sbc --tail 100 | grep "401\|403"

# SIP trace with Homer
# Access: http://localhost:9080
```

**Common causes:**
- Wrong credentials
- IP not whitelisted at carrier
- Clock skew (check NTP)
- Digest realm mismatch

### 6.3 High PDD (Post-Dial Delay)

**Diagnosis:**
```sql
-- Find carriers with high PDD
SELECT carrier_name, avg(pdd_ms) as avg_pdd
FROM cdr
WHERE timestamp > dateadd('h', -1, now())
GROUP BY carrier_name
HAVING avg_pdd > 3000
ORDER BY avg_pdd DESC;
```

**Actions:**
1. Switch to backup carrier temporarily
2. Contact carrier support
3. Adjust carrier priority in LCR

---

## 7. Capacity Planning

### 7.1 Current Capacity Limits

| Resource | Limit | Warning |
|----------|-------|---------|
| Concurrent calls | 10,000 | 8,000 |
| CPS | 1,000 | 800 |
| RTP sessions | 5,000 | 4,000 |

### 7.2 Scaling Request Process

1. Create capacity request in Jira
2. Include: current usage, projected growth, timeline
3. DevOps will provision additional resources
4. Verify new capacity in staging

---

## 8. Emergency Procedures

### 8.1 Carrier Failover

```bash
# Immediately disable problematic carrier
curl -X PATCH http://localhost:8095/api/v1/carriers/{id} \
  -d '{"status": "inactive"}'

# Traffic automatically routes to next LCR option
```

### 8.2 Platform Rollback

```bash
# Contact DevOps for rollback
# They will execute:
kubectl rollout undo deployment/voice-switch -n brivas-core
```

### 8.3 Escalation Matrix

| Severity | Response Time | Contact |
|----------|---------------|---------|
| Critical | 15 min | On-call engineer (PagerDuty) |
| High | 1 hour | Team lead |
| Medium | 4 hours | Ticket queue |
| Low | Next business day | Ticket queue |

---

## 9. Quick Reference Commands

```bash
# Service health
docker-compose ps
curl localhost:8080/health

# Carrier operations  
curl localhost:8095/api/v1/carriers          # List all
curl localhost:8095/api/v1/carriers/{id}     # Get one
curl -X POST localhost:8095/api/v1/carriers  # Create
curl -X PATCH localhost:8095/api/v1/carriers/{id}  # Update

# Analytics
curl localhost:8095/api/v1/analytics/traffic
curl localhost:8095/api/v1/analytics/carriers
curl localhost:8095/api/v1/analytics/cps

# Logs
docker logs brivas-voice-switch --tail 100
docker logs brivas-kamailio-sbc --tail 100
```

---

## 10. Certification Checklist

Complete these tasks to be certified:

- [ ] Add a test carrier
- [ ] Create a route for prefix 234
- [ ] Generate a traffic report from QuestDB
- [ ] Set up a Grafana alert
- [ ] Perform a carrier failover drill
- [ ] Document a troubleshooting scenario
