# Finance & Billing Training Manual

> **Role**: Finance Team  
> **Platform**: Unified Brivas Platform  
> **Version**: 1.0.0 | January 2026

---

## 1. Introduction

### 1.1 Role Overview
As a Finance team member, you interact with:
- **CDR (Call Detail Records)** - Usage data for billing
- **Rating & Billing** - Cost/revenue calculations
- **Carrier Settlements** - Vendor payments
- **Customer Invoicing** - Revenue collection
- **Revenue Analytics** - Financial reporting

### 1.2 Key Systems

| System | Purpose | Access |
|--------|---------|--------|
| QuestDB | CDR Analytics | `http://localhost:9000` |
| ClickHouse | OLAP Reports | `http://localhost:8123` |
| Billing Service | Invoicing | API via gateway |
| Grafana | Dashboards | `http://localhost:3000` |

---

## 2. Understanding CDRs

### 2.1 CDR Fields

| Field | Description | Example |
|-------|-------------|---------|
| `call_id` | Unique identifier | UUID |
| `timestamp` | Call start time | 2026-01-10T12:00:00Z |
| `source_number` | Calling party | +2348012345678 |
| `destination_number` | Called party | +14155551234 |
| `duration_secs` | Call length | 180 |
| `billable_seconds` | Billed duration | 180 |
| `rate` | Cost per minute | 0.025 |
| `cost` | Total cost | 0.075 |
| `revenue` | Billed amount | 0.15 |
| `carrier_id` | Terminating carrier | UUID |
| `disposition` | Call outcome | answered, busy, failed |

### 2.2 Accessing CDR Data

**QuestDB Web Console**: `http://localhost:9000`

```sql
-- Today's revenue summary
SELECT 
    count(*) as total_calls,
    sum(duration_secs)/60 as total_minutes,
    sum(cost) as total_cost,
    sum(revenue) as total_revenue,
    sum(revenue) - sum(cost) as gross_margin
FROM cdr
WHERE timestamp > dateadd('d', -1, now())
  AND disposition = 'answered';
```

---

## 3. Revenue Reports

### 3.1 Daily Revenue Report

```sql
SELECT 
    date_trunc('day', timestamp) as date,
    count(*) as calls,
    sum(duration_secs)/60.0 as minutes,
    sum(revenue) as revenue,
    sum(cost) as cost,
    sum(revenue) - sum(cost) as margin,
    (sum(revenue) - sum(cost)) / sum(revenue) * 100 as margin_pct
FROM cdr
WHERE timestamp > dateadd('d', -30, now())
  AND disposition = 'answered'
GROUP BY date_trunc('day', timestamp)
ORDER BY date DESC;
```

### 3.2 Revenue by Destination

```sql
SELECT 
    substring(destination_number, 1, 3) as country_code,
    count(*) as calls,
    sum(duration_secs)/60.0 as minutes,
    sum(revenue) as revenue,
    avg(rate) as avg_rate
FROM cdr
WHERE timestamp > dateadd('d', -7, now())
GROUP BY country_code
ORDER BY revenue DESC
LIMIT 20;
```

### 3.3 Revenue by Customer

```sql
SELECT 
    customer_id,
    customer_name,
    count(*) as calls,
    sum(duration_secs)/60.0 as minutes,
    sum(revenue) as revenue
FROM cdr
JOIN customers ON cdr.customer_id = customers.id
WHERE timestamp > dateadd('month', -1, now())
GROUP BY customer_id, customer_name
ORDER BY revenue DESC;
```

---

## 4. Carrier Settlements

### 4.1 Carrier Cost Report

```sql
SELECT 
    carrier_id,
    carrier_name,
    count(*) as calls,
    sum(duration_secs)/60.0 as minutes,
    sum(cost) as total_cost,
    avg(rate) as avg_rate
FROM cdr
WHERE timestamp >= '2026-01-01'
  AND timestamp < '2026-02-01'
  AND disposition = 'answered'
GROUP BY carrier_id, carrier_name
ORDER BY total_cost DESC;
```

### 4.2 Settlement Reconciliation

```sql
-- Compare our CDRs vs carrier invoice
SELECT 
    date_trunc('day', timestamp) as date,
    carrier_name,
    sum(duration_secs) as our_seconds,
    sum(cost) as our_cost
FROM cdr
WHERE carrier_id = '{carrier_uuid}'
  AND timestamp >= '2026-01-01'
  AND timestamp < '2026-02-01'
GROUP BY date_trunc('day', timestamp), carrier_name
ORDER BY date;
```

---

## 5. Customer Billing

### 5.1 Generate Invoice Data

```sql
SELECT 
    customer_id,
    destination_number,
    count(*) as calls,
    sum(duration_secs) as total_seconds,
    sum(billable_seconds) as billed_seconds,
    rate,
    sum(revenue) as amount
FROM cdr
WHERE customer_id = '{customer_uuid}'
  AND timestamp >= '2026-01-01'
  AND timestamp < '2026-02-01'
  AND disposition = 'answered'
GROUP BY customer_id, destination_number, rate
ORDER BY amount DESC;
```

### 5.2 Billing API

```bash
# Generate invoice
curl -X POST http://localhost:8080/api/v1/billing/invoices \
  -H "Content-Type: application/json" \
  -d '{
    "customer_id": "uuid",
    "period_start": "2026-01-01",
    "period_end": "2026-01-31"
  }'

# View invoice
curl http://localhost:8080/api/v1/billing/invoices/{invoice_id}
```

---

## 6. Key Metrics

### 6.1 Margin Analysis

| Metric | Formula | Target |
|--------|---------|--------|
| Gross Margin | (Revenue - Cost) / Revenue | >30% |
| Revenue/Minute | Total Revenue / Total Minutes | Varies by route |
| Cost/Minute | Total Cost / Total Minutes | <Revenue/Minute |

### 6.2 Quick Query Templates

```sql
-- Margin by route prefix
SELECT 
    substring(destination_number, 1, 5) as prefix,
    sum(revenue) as revenue,
    sum(cost) as cost,
    (sum(revenue) - sum(cost)) / sum(revenue) * 100 as margin_pct
FROM cdr
WHERE timestamp > dateadd('d', -7, now())
GROUP BY prefix
HAVING sum(revenue) > 100
ORDER BY margin_pct ASC
LIMIT 20;

-- Low-margin alert
SELECT * FROM (
    SELECT 
        carrier_name,
        (sum(revenue) - sum(cost)) / sum(revenue) * 100 as margin
    FROM cdr
    WHERE timestamp > dateadd('d', -1, now())
    GROUP BY carrier_name
) WHERE margin < 10;
```

---

## 7. Fraud Detection for Finance

### 7.1 Anomaly Detection

```sql
-- Unusual traffic patterns
SELECT 
    customer_id,
    date_trunc('hour', timestamp) as hour,
    count(*) as calls,
    sum(duration_secs)/60 as minutes
FROM cdr
WHERE timestamp > dateadd('d', -1, now())
GROUP BY customer_id, hour
HAVING calls > 1000  -- Alert threshold
ORDER BY calls DESC;

-- High-cost destinations spike
SELECT 
    destination_number,
    count(*) as calls,
    sum(cost) as total_cost
FROM cdr
WHERE timestamp > dateadd('h', -1, now())
  AND rate > 0.5  -- Premium rate
GROUP BY destination_number
HAVING total_cost > 100
ORDER BY total_cost DESC;
```

### 7.2 Viewing Fraud Alerts

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
    description
FROM fraud_alerts
WHERE timestamp > dateadd('d', -1, now())
ORDER BY timestamp DESC
LIMIT 50;
```

---

## 8. Grafana Dashboards

### 8.1 Finance Dashboard Access

**URL**: `http://localhost:3000/dashboards`

**Key Dashboards:**
1. **Revenue Overview** - Daily/monthly revenue trends
2. **Carrier Costs** - Per-carrier cost breakdown
3. **Margin Analysis** - Route profitability
4. **Customer Billing** - Top customers by revenue

### 8.2 Creating Custom Panels

1. Click "Add Panel"
2. Select "QuestDB" as datasource
3. Paste SQL query
4. Choose visualization (Table, Graph, etc.)
5. Save dashboard

---

## 9. Export & Reporting

### 9.1 CSV Export

```sql
-- In QuestDB Console, run query then click "Export CSV"
SELECT * FROM cdr
WHERE timestamp >= '2026-01-01'
  AND timestamp < '2026-02-01'
ORDER BY timestamp;
```

### 9.2 Scheduled Reports

Contact DevOps to set up scheduled email reports via Grafana.

---

## 10. Quick Reference

### Common Queries

```sql
-- Today's summary
SELECT sum(revenue), sum(cost), sum(revenue)-sum(cost) as margin
FROM cdr WHERE timestamp > dateadd('d', -1, now());

-- This month's summary
SELECT sum(revenue), sum(cost), count(*) as calls
FROM cdr WHERE timestamp >= date_trunc('month', now());

-- Top 10 customers this week
SELECT customer_id, sum(revenue) as revenue
FROM cdr WHERE timestamp > dateadd('d', -7, now())
GROUP BY customer_id ORDER BY revenue DESC LIMIT 10;

-- Low ASR carriers (potential quality/cost issue)
SELECT carrier_name, 
       count(*) as attempts,
       sum(case when disposition='answered' then 1 else 0 end) as answered,
       sum(case when disposition='answered' then 1.0 else 0.0 end)/count(*)*100 as asr
FROM cdr WHERE timestamp > dateadd('d', -1, now())
GROUP BY carrier_name HAVING asr < 40
ORDER BY attempts DESC;
```
