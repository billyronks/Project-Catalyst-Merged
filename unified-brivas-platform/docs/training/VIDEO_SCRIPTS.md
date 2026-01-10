# Video Training Scripts

> **Platform**: Unified Brivas Platform  
> **Version**: 1.0.0 | January 2026

---

## Video 1: Platform Overview (8 min)

### Script

**[0:00-0:30] Intro**
"Welcome to the Unified Brivas Platform. I'm [Name], and in the next 8 minutes, I'll walk you through our carrier-grade telecommunications platform that handles over 10 million transactions per second."

**[0:30-2:00] Architecture Overview**
*[Screen: Show ARCHITECTURE.md system diagram]*

"Let's start with the big picture. Our platform has five main layers:
- The Edge Layer with our XDP load balancer handling 100 gigabits of traffic
- The API Gateway providing REST, GraphQL, and WebSocket interfaces
- SIP Signaling with Kamailio and OpenSIPS for voice
- Rust microservices for business logic
- And our analytics layer with QuestDB and ClickHouse"

**[2:00-4:00] Core Products**
*[Screen: Demo dashboard]*

"We offer six core products:
1. Voice Termination - routing calls to 200+ carriers
2. SMS Gateway - A2P messaging at scale
3. USSD Gateway - for mobile money and banking
4. Flash Call - phone verification
5. RCS Messaging - rich business messaging
6. And our upcoming WebRTC platform"

**[4:00-6:00] Key Differentiators**
*[Screen: Performance metrics]*

"What makes us different?
- Sub-millisecond latency - we're 10x faster than competitors
- Real-time analytics - see your data in 2 milliseconds, not hours
- ML-powered fraud detection - blocking scams before they happen
- And 99.99% uptime with automatic failover"

**[6:00-7:30] Quick Demo**
*[Screen: Live API call]*

"Let me show you how easy it is to send an SMS..."
*[Execute API call, show response]*

"...and here's the CDR appearing in real-time in QuestDB."

**[7:30-8:00] Outro**
"That's the Unified Brivas Platform. Check out our role-specific training videos to learn more. Thanks for watching!"

---

## Video 2: Operations - Carrier Management (10 min)

### Script

**[0:00-1:00] Intro**
"In this video, we'll learn how to manage carriers in the Unified Brivas Platform. This is essential for Operations Engineers who need to add, configure, and troubleshoot carrier connections."

**[1:00-3:00] Adding a Carrier**
*[Screen: API call in terminal]*

```bash
curl -X POST http://localhost:8095/api/v1/carriers \
  -d '{"name":"CarrierX","host":"sip.carrierx.com","port":5060}'
```

"Here we're creating a new carrier. The key fields are name, host, port, and authentication credentials."

**[3:00-5:00] Configuring Routes**
*[Screen: Route configuration]*

"Next, let's set up routing. We'll create a route for Nigerian traffic using least-cost routing..."

```bash
curl -X POST http://localhost:8095/api/v1/routes \
  -d '{"prefix":"234","carrier_id":"uuid","rate":0.025}'
```

**[5:00-7:00] Monitoring Carrier Health**
*[Screen: Grafana dashboard]*

"Now let's check our carrier's performance. In Grafana, we can see:
- ASR - currently at 92%, which is excellent
- PDD - 1.2 seconds average
- Active call count
- Error rates by hangup cause"

**[7:00-9:00] Troubleshooting**
*[Screen: QuestDB query]*

"When something goes wrong, here's how to diagnose it..."

```sql
SELECT carrier_name, disposition, count(*)
FROM cdr WHERE timestamp > dateadd('h', -1, now())
GROUP BY carrier_name, disposition;
```

**[9:00-10:00] Summary**
"You've learned how to add carriers, configure routes, monitor health, and troubleshoot issues. Practice these skills in the sandbox environment."

---

## Video 3: DevOps - Deployment & Scaling (12 min)

### Script

**[0:00-1:00] Intro**
"Welcome DevOps engineers. Today we'll deploy the platform locally, then scale it for production."

**[1:00-3:00] Local Setup**
*[Screen: Terminal]*

```bash
git clone https://github.com/billyronks/Project-Catalyst-Merged
cd Project-Catalyst-Merged/unified-brivas-platform
cp .env.example .env
docker-compose up -d
```

"The platform takes about 2 minutes to start. Let's verify..."

**[3:00-5:00] Architecture Review**
*[Screen: docker-compose ps output]*

"We have 12 services running:
- LumaDB for primary data
- QuestDB for analytics
- API Gateway
- Voice Switch
- And supporting services"

**[5:00-8:00] Scaling**
*[Screen: docker-compose scale]*

```bash
docker-compose up -d --scale voice-switch=3
```

"To handle more traffic, we scale horizontally. The load balancer automatically distributes traffic."

**[8:00-10:00] Monitoring Setup**
*[Screen: Grafana setup]*

"Let's add a Prometheus datasource and import our dashboards..."

**[10:00-12:00] Production Considerations**
*[Screen: Kubernetes manifest]*

"For production, use Kubernetes with proper resource limits, health checks, and auto-scaling policies."

---

## Video 4: Finance - Revenue Reporting (8 min)

### Script

**[0:00-1:00] Intro**
"Finance team - let's explore how to generate revenue reports and analyze traffic patterns."

**[1:00-3:00] QuestDB Access**
*[Screen: QuestDB console]*

"Open http://localhost:9000 to access the analytics console. All CDR data is here."

**[3:00-5:00] Daily Revenue Report**
*[Screen: Running SQL query]*

```sql
SELECT date_trunc('day', timestamp) as date,
       sum(revenue) as revenue,
       sum(cost) as cost,
       sum(revenue)-sum(cost) as margin
FROM cdr
WHERE timestamp > dateadd('d', -30, now())
GROUP BY date_trunc('day', timestamp)
ORDER BY date;
```

"This gives us 30 days of revenue, cost, and margin data."

**[5:00-7:00] Carrier Settlements**
*[Screen: Carrier cost query]*

"For carrier settlements, we filter by carrier and compare against invoices..."

**[7:00-8:00] Export Options**
*[Screen: CSV export]*

"Click Export CSV to download for Excel analysis."

---

## Video 5: Business Development - Demo Guide (10 min)

### Script

**[0:00-1:00] Intro**
"BD team - here's your 10-minute guide to running an effective platform demo."

**[1:00-3:00] Pre-Demo Checklist**
- Demo environment running
- Sample customer created
- Rate plan configured
- Dashboards loaded

**[3:00-6:00] Demo Flow**
*[Screen: Live demo]*

"Start with the big picture..."
"Show real-time dashboard..."
"Execute an API call..."
"Highlight performance metrics..."

**[6:00-8:00] Handling Objections**
- "Too expensive" → Show cost comparison
- "We use Twilio" → Highlight local routes, pricing
- "Security concerns" → Explain STIR/SHAKEN, fraud detection

**[8:00-10:00] Closing**
"Always end with:
1. Price estimate based on their volume
2. POC offer
3. Clear next steps"

---

## Video 6: Audit - Compliance Walkthrough (8 min)

### Script

**[0:00-1:00] Intro**
"This video covers compliance requirements and audit processes."

**[1:00-3:00] Regulatory Overview**
*[Screen: Compliance matrix]*

"We're compliant with NCC, ICASA, FCC, GDPR, PCI-DSS, and ISO 27001..."

**[3:00-5:00] Audit Trail**
*[Screen: Audit log query]*

"Every action is logged. Here's how to query admin actions..."

**[5:00-7:00] Fraud Detection**
*[Screen: Fraud alert dashboard]*

"Our ML system detects fraud in real-time. Here are the alert types and severities..."

**[7:00-8:00] Report Generation**
"Generate compliance reports via the API for external auditors."

---

## Video Production Notes

| Video | Duration | Priority | Status |
|-------|----------|----------|--------|
| Platform Overview | 8 min | P0 | Script ready |
| Carrier Management | 10 min | P0 | Script ready |
| Deployment & Scaling | 12 min | P0 | Script ready |
| Revenue Reporting | 8 min | P1 | Script ready |
| Demo Guide | 10 min | P1 | Script ready |
| Compliance Walkthrough | 8 min | P2 | Script ready |

### Recording Requirements
- Screen capture: 1920x1080, 60fps
- Audio: Clear narration, no background noise
- Annotations: Highlight important UI elements
- Captions: Required for accessibility
