# Audit & Compliance Training Manual

> **Role**: Audit & Compliance  
> **Platform**: Unified Brivas Platform  
> **Version**: 1.0.0 | January 2026

---

## 1. Compliance Framework

### 1.1 Regulatory Requirements

| Regulation | Scope | Status |
|------------|-------|--------|
| **NCC** (Nigeria) | Telecom licensing | Compliant |
| **ICASA** (South Africa) | Communications Act | Compliant |
| **FCC** (USA) | STIR/SHAKEN | Implemented |
| **GDPR** (EU) | Data protection | Compliant |
| **PCI-DSS** | Payment data | Level 1 |
| **ISO 27001** | Information security | Certified |

### 1.2 Key Controls

| Control Area | Implementation |
|--------------|----------------|
| Access Control | RBAC + MFA |
| Data Encryption | AES-256 at rest, TLS 1.3 in transit |
| Audit Logging | Immutable logs in QuestDB |
| Fraud Prevention | ML-powered real-time detection |
| Call Authentication | STIR/SHAKEN |

---

## 2. Audit Trail Access

### 2.1 What Gets Logged

| Event Type | Retention | Access |
|------------|-----------|--------|
| CDR (Call Detail Records) | 7 years | QuestDB |
| API Access Logs | 2 years | Elasticsearch |
| Admin Actions | 7 years | Audit DB |
| Config Changes | Indefinite | Git |
| Login Events | 2 years | Auth DB |

### 2.2 Querying Audit Logs

**CDR Audit Query:**
```sql
SELECT 
    call_id,
    timestamp,
    source_number,
    destination_number,
    carrier_name,
    duration_secs,
    disposition,
    cost,
    revenue
FROM cdr
WHERE timestamp BETWEEN '2026-01-01' AND '2026-01-31'
ORDER BY timestamp;
```

**Admin Action Log:**
```sql
SELECT 
    timestamp,
    user_id,
    user_email,
    action,
    resource_type,
    resource_id,
    old_value,
    new_value,
    ip_address
FROM audit_log
WHERE action IN ('CREATE', 'UPDATE', 'DELETE')
ORDER BY timestamp DESC
LIMIT 1000;
```

---

## 3. Fraud Detection Audit

### 3.1 Fraud Alert Review

```sql
SELECT 
    id,
    timestamp,
    alert_type,
    severity,
    source_number,
    destination_number,
    risk_score,
    blocked,
    reviewed_by,
    reviewed_at,
    disposition
FROM fraud_alerts
WHERE timestamp > dateadd('d', -30, now())
ORDER BY timestamp DESC;
```

### 3.2 Alert Types

| Type | Description | Severity |
|------|-------------|----------|
| `irsf` | International Revenue Share Fraud | Critical |
| `wangiri` | One-ring callback scam | High |
| `cli_spoofing` | Caller ID manipulation | High |
| `prs` | Premium Rate Services abuse | Medium |
| `call_pumping` | Traffic pumping | Medium |

---

## 4. Data Access Controls

### 4.1 Role-Based Access

| Role | CDR Access | Config Access | Admin |
|------|------------|---------------|-------|
| Operator | Read own | None | No |
| Support | Read all | Read | No |
| Finance | Read all | None | No |
| Admin | Full | Full | Yes |
| Audit | Read all | Read | View only |

### 4.2 Generating Access Reports

```sql
-- Who accessed what
SELECT 
    user_email,
    resource_type,
    action,
    count(*) as access_count
FROM access_log
WHERE timestamp > dateadd('d', -30, now())
GROUP BY user_email, resource_type, action
ORDER BY access_count DESC;
```

---

## 5. STIR/SHAKEN Compliance

### 5.1 Attestation Levels

| Level | Meaning | Our Usage |
|-------|---------|-----------|
| **A (Full)** | We authenticated the caller | Direct customers |
| **B (Partial)** | Caller authenticated, not identity | Wholesale transit |
| **C (Gateway)** | Originated from gateway | International inbound |

### 5.2 Verification Report

```sql
SELECT 
    date_trunc('day', timestamp) as date,
    attestation_level,
    verification_result,
    count(*) as calls
FROM stir_shaken_log
WHERE timestamp > dateadd('d', -30, now())
GROUP BY date_trunc('day', timestamp), attestation_level, verification_result
ORDER BY date DESC, attestation_level;
```

---

## 6. Compliance Reports

### 6.1 Monthly Compliance Dashboard

**Metrics to Review:**
- [ ] Fraud block rate
- [ ] STIR/SHAKEN attestation distribution
- [ ] Failed authentication attempts
- [ ] Privilege escalations
- [ ] Configuration changes

### 6.2 Report Generation

```bash
# Generate compliance report
curl http://localhost:8080/api/v1/reports/compliance \
  -H "Authorization: Bearer $AUDIT_TOKEN" \
  -d '{"period": "2026-01", "format": "pdf"}' \
  > compliance_report_2026_01.pdf
```

---

## 7. Incident Response

### 7.1 Classification

| Severity | Response Time | Escalation |
|----------|---------------|------------|
| Critical | 15 min | CEO + Legal |
| High | 1 hour | CTO + Compliance |
| Medium | 4 hours | Team Lead |
| Low | Next day | Regular queue |

### 7.2 Investigation Queries

```sql
-- Find all activity for suspicious number
SELECT * FROM cdr
WHERE source_number = '+12345678901'
   OR destination_number = '+12345678901'
ORDER BY timestamp DESC
LIMIT 100;

-- User session analysis
SELECT * FROM session_log
WHERE user_id = 'suspicious_user_id'
ORDER BY timestamp DESC;
```

---

## 8. Data Retention

| Data Type | Retention | Deletion |
|-----------|-----------|----------|
| CDRs | 7 years | Automated |
| Recordings | 2 years | On request |
| Logs | 2 years | Automated |
| Backups | 90 days | Automated |
| PII | Per GDPR | On request |

### 8.1 GDPR Data Subject Requests

```bash
# Export user data
curl -X POST http://localhost:8080/api/v1/gdpr/export \
  -d '{"email": "user@example.com"}'

# Delete user data
curl -X POST http://localhost:8080/api/v1/gdpr/delete \
  -d '{"email": "user@example.com", "confirmation": true}'
```

---

## 9. Checklist

### External Audit Preparation

- [ ] CDR samples exported
- [ ] Access control matrix documented
- [ ] STIR/SHAKEN certificates valid
- [ ] Fraud detection rules reviewed
- [ ] Encryption keys rotated
- [ ] Backup restoration tested
- [ ] Incident log reviewed
