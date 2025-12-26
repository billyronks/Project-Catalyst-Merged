-- Billing Service Schema for LumaDB (PostgreSQL Protocol)
-- File: microservices/billing/schema/billing.sql

-- Rate Plans
CREATE TABLE IF NOT EXISTS rate_plans (
    plan_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    plan_type INTEGER NOT NULL,
    billing_cycle INTEGER NOT NULL,
    rates JSONB NOT NULL DEFAULT '[]',
    tiers JSONB DEFAULT '[]',
    discounts JSONB DEFAULT '[]',
    active BOOLEAN DEFAULT true,
    effective_from TIMESTAMPTZ DEFAULT NOW(),
    effective_to TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_rate_plans_tenant ON rate_plans(tenant_id);
CREATE INDEX IF NOT EXISTS idx_rate_plans_active ON rate_plans(active, effective_from);

-- CDRs (Call Detail Records)
CREATE TABLE IF NOT EXISTS cdrs (
    cdr_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL,
    event_type INTEGER NOT NULL,
    source VARCHAR(50),
    destination VARCHAR(50),
    quantity BIGINT NOT NULL,
    unit VARCHAR(20),
    duration_seconds INTEGER DEFAULT 0,
    rated_cost_minor BIGINT,
    rated_currency VARCHAR(3) DEFAULT 'USD',
    rate_plan_id UUID REFERENCES rate_plans(plan_id),
    rate_id VARCHAR(50),
    message_id VARCHAR(50),
    session_id VARCHAR(50),
    carrier VARCHAR(50),
    processing_pop VARCHAR(20) DEFAULT 'default',
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_cdrs_tenant ON cdrs(tenant_id);
CREATE INDEX IF NOT EXISTS idx_cdrs_event_type ON cdrs(event_type);
CREATE INDEX IF NOT EXISTS idx_cdrs_created ON cdrs(created_at DESC);

-- Invoices
CREATE TABLE IF NOT EXISTS invoices (
    invoice_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    invoice_number VARCHAR(50) UNIQUE NOT NULL,
    tenant_id UUID NOT NULL,
    status INTEGER DEFAULT 1,
    period_start TIMESTAMPTZ NOT NULL,
    period_end TIMESTAMPTZ NOT NULL,
    issue_date TIMESTAMPTZ DEFAULT NOW(),
    due_date TIMESTAMPTZ NOT NULL,
    subtotal_minor BIGINT NOT NULL,
    tax_minor BIGINT DEFAULT 0,
    discount_minor BIGINT DEFAULT 0,
    total_minor BIGINT NOT NULL,
    amount_paid_minor BIGINT DEFAULT 0,
    amount_due_minor BIGINT NOT NULL,
    currency VARCHAR(3) DEFAULT 'USD',
    line_items JSONB NOT NULL DEFAULT '[]',
    tax_items JSONB DEFAULT '[]',
    pdf_url TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_invoices_tenant ON invoices(tenant_id);
CREATE INDEX IF NOT EXISTS idx_invoices_status ON invoices(status);

-- Wallets (Prepaid)
CREATE TABLE IF NOT EXISTS wallets (
    wallet_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID UNIQUE NOT NULL,
    balance_minor BIGINT DEFAULT 0,
    reserved_minor BIGINT DEFAULT 0,
    credit_limit_minor BIGINT DEFAULT 0,
    currency VARCHAR(3) DEFAULT 'USD',
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Wallet Transactions
CREATE TABLE IF NOT EXISTS wallet_transactions (
    transaction_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    wallet_id UUID NOT NULL REFERENCES wallets(wallet_id),
    type VARCHAR(20) NOT NULL,
    amount_minor BIGINT NOT NULL,
    balance_after_minor BIGINT NOT NULL,
    reference VARCHAR(255),
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_wallet_txns_wallet ON wallet_transactions(wallet_id, created_at DESC);

-- Balance Reservations
CREATE TABLE IF NOT EXISTS balance_reservations (
    reservation_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    wallet_id UUID NOT NULL REFERENCES wallets(wallet_id),
    amount_minor BIGINT NOT NULL,
    reference VARCHAR(255),
    expires_at TIMESTAMPTZ NOT NULL,
    committed BOOLEAN DEFAULT false,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_reservations_wallet ON balance_reservations(wallet_id);
CREATE INDEX IF NOT EXISTS idx_reservations_expires ON balance_reservations(expires_at) WHERE NOT committed;

-- Insert default rate plan
INSERT INTO rate_plans (name, description, plan_type, billing_cycle, rates) VALUES
    ('Standard', 'Pay-as-you-go pricing', 1, 1, '[
        {"event_type": 1, "unit_price_minor": 50, "currency": "USD", "unit": "message"},
        {"event_type": 2, "unit_price_minor": 25, "currency": "USD", "unit": "message"},
        {"event_type": 3, "unit_price_minor": 100, "currency": "USD", "unit": "session"},
        {"event_type": 4, "unit_price_minor": 10, "currency": "USD", "unit": "second"},
        {"event_type": 5, "unit_price_minor": 30, "currency": "USD", "unit": "message"}
    ]')
ON CONFLICT DO NOTHING;
