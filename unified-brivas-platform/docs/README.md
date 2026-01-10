# Training & Documentation Index

> **Unified Brivas Platform**  
> **Version**: 1.0.0 | January 2026

---

## Quick Start

| I am a... | Start Here |
|-----------|------------|
| **Operations Engineer** | [Operations Manual](training/OPERATIONS_ENGINEER.md) |
| **DevOps Engineer** | [DevOps Manual](training/DEVOPS_ENGINEER.md) |
| **Software Developer** | [Developer Manual](training/SOFTWARE_DEVELOPER.md) |
| **Back Office Engineer** | [Back Office Manual](training/BACK_OFFICE.md) |
| **Product Engineer** | [Product Manual](training/PRODUCT_ENGINEER.md) |
| **Finance Team** | [Finance Manual](training/FINANCE.md) |
| **Audit/Compliance** | [Audit Manual](training/AUDIT_COMPLIANCE.md) |
| **Business Development** | [BD Manual](training/BUSINESS_DEVELOPMENT.md) |

---

## Architecture & Technical Reference

| Document | Description |
|----------|-------------|
| [**ARCHITECTURE.md**](ARCHITECTURE.md) | Full system architecture with Mermaid diagrams |
| [**API Reference**](api/README.md) | REST/GraphQL API documentation |
| [**Database Schema**](database/SCHEMA.md) | LumaDB and QuestDB schemas |

---

## Training Manuals

### Technical Roles

| Manual | Audience | Topics |
|--------|----------|--------|
| [Operations Engineer](training/OPERATIONS_ENGINEER.md) | NOC, Support Tier 2 | Carrier management, routing, monitoring, troubleshooting |
| [DevOps Engineer](training/DEVOPS_ENGINEER.md) | Infrastructure, SRE | Deployment, CI/CD, scaling, disaster recovery |
| [Software Developer](training/SOFTWARE_DEVELOPER.md) | Backend Engineers | Rust patterns, Temporal workflows, testing |
| [Back Office Engineer](training/BACK_OFFICE.md) | Provisioning, Support | Customer management, provisioning, rate plans |

### Business Roles

| Manual | Audience | Topics |
|--------|----------|--------|
| [Finance](training/FINANCE.md) | Billing, Accounting | CDR analytics, revenue reports, settlements |
| [Product Engineer](training/PRODUCT_ENGINEER.md) | Product Management | Feature lifecycle, metrics, A/B testing |
| [Audit & Compliance](training/AUDIT_COMPLIANCE.md) | Compliance, Legal | Audit trails, fraud detection, regulations |
| [Business Development](training/BUSINESS_DEVELOPMENT.md) | Sales, Partnerships | Products, pricing, demo scripts |

---

## Video Training

| Video | Duration | Audience | Topics |
|-------|----------|----------|--------|
| Platform Overview | 8 min | All | Architecture, products, differentiators |
| Carrier Management | 10 min | Operations | Add/configure carriers, routing |
| Deployment & Scaling | 12 min | DevOps | Docker, Kubernetes, monitoring |
| Revenue Reporting | 8 min | Finance | QuestDB queries, exports |
| Demo Guide | 10 min | BD | Demo flow, objection handling |
| Compliance Walkthrough | 8 min | Audit | Regulations, audit trails |

📺 **Scripts**: [VIDEO_SCRIPTS.md](training/VIDEO_SCRIPTS.md)

---

## Key System URLs

| System | URL | Purpose |
|--------|-----|---------|
| API Gateway | http://localhost:8080 | REST/GraphQL API |
| Voice Switch | http://localhost:8095 | Carrier/LCR management |
| Temporal UI | http://localhost:8088 | Workflow monitoring |
| QuestDB Console | http://localhost:9000 | Analytics queries |
| Grafana | http://localhost:3000 | Dashboards |
| Consul | http://localhost:8500 | Service discovery |

---

## Learning Paths

### Path 1: New Operations Engineer (2 weeks)

| Week | Focus | Resources |
|------|-------|-----------|
| 1 | Fundamentals | Platform Overview video, Architecture doc |
| 1 | Carrier Management | Operations manual Ch. 2-4 |
| 2 | Monitoring | Operations manual Ch. 5-6 |
| 2 | Troubleshooting | Operations manual Ch. 7-8 |

### Path 2: New Developer (3 weeks)

| Week | Focus | Resources |
|------|-------|-----------|
| 1 | Environment Setup | Developer manual Ch. 1-2 |
| 1 | Code Patterns | Developer manual Ch. 3-4 |
| 2 | Temporal Workflows | Developer manual Ch. 5 |
| 2 | Testing | Developer manual Ch. 7 |
| 3 | First Feature | Pair with senior dev |

### Path 3: DevOps Onboarding (2 weeks)

| Week | Focus | Resources |
|------|-------|-----------|
| 1 | Local Setup | DevOps manual Ch. 1-3 |
| 1 | CI/CD | DevOps manual Ch. 4 |
| 2 | Monitoring | DevOps manual Ch. 5 |
| 2 | Production | DevOps manual Ch. 6-9 |

---

## Certification

Complete role-specific certification to demonstrate proficiency:

| Role | Certification Tasks | Evaluator |
|------|---------------------|-----------|
| Operations | Add carrier, create route, troubleshoot issue | Team Lead |
| DevOps | Deploy stack, set up monitoring, perform rollback | Senior SRE |
| Developer | Complete feature, pass code review, write tests | Tech Lead |

---

## Getting Help

| Channel | Purpose | Response |
|---------|---------|----------|
| #platform-help (Slack) | Quick questions | Same day |
| Jira Ticket | Formal requests | 1-3 days |
| Engineering Sync | Complex issues | Weekly meeting |
| Documentation PR | Improvements | 1 week |
