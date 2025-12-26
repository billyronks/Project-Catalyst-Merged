-- Unified Brivas Platform - LumaDB Schema Migration
-- This schema consolidates all data models from brivas-api, Project-Catalyst, and admin backends
-- LumaDB provides PostgreSQL wire protocol compatibility - use standard SQL

-- ============================================================================
-- CORE ACCOUNTS & AUTHENTICATION
-- ============================================================================

CREATE TABLE IF NOT EXISTS accounts (
    id VARCHAR(15) PRIMARY KEY DEFAULT 'BV' || substr(md5(random()::text), 1, 9),
    is_verified BOOLEAN DEFAULT FALSE,
    type VARCHAR(45),
    first_name VARCHAR(30),
    last_name VARCHAR(30),
    username VARCHAR(45),
    business JSONB DEFAULT '{"name": "", "email": "", "type": "", "address": ""}',
    email VARCHAR(60) NOT NULL UNIQUE,
    country_code VARCHAR(5),
    phone_number VARCHAR(20),
    is_phone_verified BOOLEAN DEFAULT FALSE,
    test_secret_key VARCHAR(50) NOT NULL UNIQUE,
    live_secret_key VARCHAR(50) NOT NULL UNIQUE,
    reg_time TIMESTAMP NOT NULL DEFAULT NOW(),
    password VARCHAR(255),
    sdp_password VARCHAR(45),
    sdp_sid VARCHAR(50),
    sdp_token VARCHAR(50),
    sid VARCHAR(50) DEFAULT '',
    sdp_friendly_name VARCHAR(255) DEFAULT '',
    is_blacklist BOOLEAN DEFAULT FALSE,
    rates JSONB DEFAULT '{"flashcall": 1, "otp": 3, "transactional": 3, "corporate": 3, "promotional": 2.5}',
    balance DOUBLE PRECISION DEFAULT 0,
    balance_threshold DOUBLE PRECISION DEFAULT 100,
    created_at TIMESTAMP DEFAULT NOW(),
    updated_at TIMESTAMP DEFAULT NOW()
);

CREATE INDEX idx_accounts_reg_time ON accounts(reg_time);
CREATE INDEX idx_accounts_email ON accounts(email);

-- ============================================================================
-- USER BUCKETS (Rate Limits & Balances)
-- ============================================================================

CREATE TABLE IF NOT EXISTS user_buckets (
    id SERIAL PRIMARY KEY,
    account_id VARCHAR(15) NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    transactional_sms_rate JSONB,
    promotional_sms_rate JSONB,
    call_rates JSONB,
    sms_transactional_unit_balance DOUBLE PRECISION DEFAULT 0.0,
    sms_promotional_unit_balance DOUBLE PRECISION DEFAULT 0.0,
    call_unit DOUBLE PRECISION DEFAULT 0.0,
    balance DOUBLE PRECISION DEFAULT 0.0,
    is_active BOOLEAN DEFAULT TRUE,
    created_at TIMESTAMP DEFAULT NOW(),
    updated_at TIMESTAMP DEFAULT NOW()
);

CREATE INDEX idx_user_buckets_account_id ON user_buckets(account_id);

-- ============================================================================
-- SMS HISTORY & MESSAGING
-- ============================================================================

CREATE TABLE IF NOT EXISTS sms_history (
    id SERIAL PRIMARY KEY,
    account_id VARCHAR(100),
    sid VARCHAR(100),
    rid VARCHAR(100),
    u_aid INTEGER,
    service_id VARCHAR(100),
    sender VARCHAR(100),
    recipient VARCHAR(45) NOT NULL,
    status VARCHAR(20) DEFAULT 'pending',
    type VARCHAR(20),
    message TEXT,
    is_live BOOLEAN DEFAULT TRUE,
    rate_per_sms DOUBLE PRECISION,
    sent_date DATE DEFAULT CURRENT_DATE,
    sent_time TIME DEFAULT CURRENT_TIME,
    origin_bucket_id INTEGER,
    sms_type VARCHAR(30) DEFAULT 'transactional',
    created_at TIMESTAMP DEFAULT NOW()
);

CREATE INDEX idx_sms_history_account_id ON sms_history(account_id);
CREATE INDEX idx_sms_history_u_aid ON sms_history(u_aid);
CREATE INDEX idx_sms_history_rid ON sms_history(rid);
CREATE INDEX idx_sms_history_sid ON sms_history(sid);
CREATE INDEX idx_sms_history_sent_date ON sms_history(sent_date);

-- ============================================================================
-- FLASH CALL & VOICE OTP
-- ============================================================================

CREATE TABLE IF NOT EXISTS flash_call_history (
    id SERIAL PRIMARY KEY,
    account_id VARCHAR(45) NOT NULL,
    u_aid INTEGER NOT NULL DEFAULT 1,
    from_number VARCHAR(45) NOT NULL,
    to_number VARCHAR(45) NOT NULL,
    retry_count INTEGER DEFAULT 0,
    is_verified BOOLEAN DEFAULT FALSE,
    sid VARCHAR(100),
    call_date DATE DEFAULT CURRENT_DATE,
    call_time TIME DEFAULT CURRENT_TIME,
    status VARCHAR(50) DEFAULT 'pending',
    is_live BOOLEAN DEFAULT TRUE,
    rate_per_call DOUBLE PRECISION,
    created_at TIMESTAMP DEFAULT NOW()
);

CREATE INDEX idx_flash_call_account_id ON flash_call_history(account_id);
CREATE INDEX idx_flash_call_u_aid ON flash_call_history(u_aid);

-- ============================================================================
-- SENDER IDS
-- ============================================================================

CREATE TABLE IF NOT EXISTS sender_ids (
    id SERIAL PRIMARY KEY,
    account_id VARCHAR(45) NOT NULL,
    sender VARCHAR(255) NOT NULL,
    status VARCHAR(100) DEFAULT 'pending',
    slug VARCHAR(45),
    docs JSONB,
    approved BOOLEAN DEFAULT FALSE,
    mno_approved BOOLEAN DEFAULT FALSE,
    type VARCHAR(100) DEFAULT 'promotional',
    is_public BOOLEAN DEFAULT FALSE,
    is_general BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMP DEFAULT NOW(),
    UNIQUE(sender, type)
);

CREATE INDEX idx_sender_ids_account_id ON sender_ids(account_id);

-- ============================================================================
-- USER APPS & WEBHOOKS
-- ============================================================================

CREATE TABLE IF NOT EXISTS user_apps (
    id SERIAL PRIMARY KEY,
    account_id VARCHAR(45) NOT NULL,
    meta JSONB,
    slug VARCHAR(45) NOT NULL,
    webhook VARCHAR(255),
    rate_limit INTEGER DEFAULT 4,
    is_enabled BOOLEAN DEFAULT TRUE,
    created_at TIMESTAMP DEFAULT NOW()
);

CREATE INDEX idx_user_apps_account_id ON user_apps(account_id);
CREATE INDEX idx_user_apps_slug ON user_apps(slug);

-- ============================================================================
-- APPLICATIONS REGISTRY
-- ============================================================================

CREATE TABLE IF NOT EXISTS apps (
    id SERIAL PRIMARY KEY,
    uid VARCHAR(8) NOT NULL UNIQUE,
    type VARCHAR(10) NOT NULL,
    name VARCHAR(45) NOT NULL,
    slug VARCHAR(45),
    create_mode VARCHAR(5) NOT NULL,
    description TEXT NOT NULL,
    icon TEXT NOT NULL,
    features TEXT,
    required_docs VARCHAR(255) NOT NULL
);

-- ============================================================================
-- SERVICES CONFIGURATION
-- ============================================================================

CREATE TABLE IF NOT EXISTS services (
    id SERIAL PRIMARY KEY,
    service_name VARCHAR(45) NOT NULL,
    service_id VARCHAR(100) UNIQUE,
    provider VARCHAR(50),
    is_active BOOLEAN DEFAULT TRUE,
    config JSONB,
    created_at TIMESTAMP DEFAULT NOW()
);

-- ============================================================================
-- USSD MENUS & SESSIONS
-- ============================================================================

CREATE TABLE IF NOT EXISTS ussd_menus (
    id SERIAL PRIMARY KEY,
    code VARCHAR(10) NOT NULL UNIQUE,
    str_menu JSONB NOT NULL
);

CREATE TABLE IF NOT EXISTS ussd_sessions (
    id SERIAL PRIMARY KEY,
    code VARCHAR(10) NOT NULL,
    session_id VARCHAR(255) NOT NULL,
    subscriber VARCHAR(20) NOT NULL,
    is_active BOOLEAN DEFAULT TRUE,
    created_at TIMESTAMP DEFAULT NOW(),
    updated_at TIMESTAMP DEFAULT NOW(),
    UNIQUE(subscriber, session_id, code)
);

-- ============================================================================
-- NUMBER POOL (Flash Call Numbers)
-- ============================================================================

CREATE TABLE IF NOT EXISTS number_pool (
    id SERIAL PRIMARY KEY,
    number VARCHAR(15) NOT NULL UNIQUE,
    is_reserved INTEGER DEFAULT 0,
    is_assigned INTEGER DEFAULT 0,
    is_manual INTEGER DEFAULT 0,
    is_tok9ja INTEGER DEFAULT 0,
    is_tok9ja_plus INTEGER DEFAULT 0,
    is_agent INTEGER DEFAULT 0,
    keep_alive_retrial_count INTEGER DEFAULT 0,
    did_ring_today INTEGER DEFAULT 0,
    last_time_used TIMESTAMP,
    is_mtn_ring_today BOOLEAN DEFAULT FALSE,
    is_airtel_ring_today BOOLEAN DEFAULT FALSE,
    is_airtel BOOLEAN DEFAULT FALSE,
    is_mtn BOOLEAN DEFAULT FALSE
);

-- ============================================================================
-- CONTACTS
-- ============================================================================

CREATE TABLE IF NOT EXISTS contacts (
    id SERIAL PRIMARY KEY,
    account_id VARCHAR(45) NOT NULL DEFAULT '',
    numbers JSONB,
    uid VARCHAR(100) NOT NULL UNIQUE,
    name VARCHAR(100)
);

CREATE INDEX idx_contacts_account_id ON contacts(account_id);

-- ============================================================================
-- DEFAULT SMS RATES (Volume-based pricing)
-- ============================================================================

CREATE TABLE IF NOT EXISTS default_sms_rates (
    id SERIAL PRIMARY KEY,
    lower_bound DOUBLE PRECISION DEFAULT 0.0,
    upper_bound DOUBLE PRECISION DEFAULT 0.0,
    type VARCHAR(15), -- promotional or transactional
    mno VARCHAR(10),  -- MTN, AIRTEL, 9MOBILE
    unit_rate DOUBLE PRECISION,
    wallet_rate DOUBLE PRECISION
);

-- ============================================================================
-- BILLING (From Project-Catalyst)
-- ============================================================================

CREATE TABLE IF NOT EXISTS tenant_billing_config (
    id SERIAL PRIMARY KEY,
    tenant_id VARCHAR(50) NOT NULL UNIQUE,
    currency VARCHAR(3) DEFAULT 'NGN',
    billing_cycle VARCHAR(20) DEFAULT 'daily',
    payment_terms VARCHAR(20) DEFAULT 'net30',
    credit_limit DOUBLE PRECISION DEFAULT 0,
    current_balance DOUBLE PRECISION DEFAULT 0,
    low_balance_alert DOUBLE PRECISION DEFAULT 100,
    suspension_point DOUBLE PRECISION DEFAULT 0,
    volume_discounts JSONB DEFAULT '[]',
    tax_rate DOUBLE PRECISION DEFAULT 7.5,
    timezone VARCHAR(50) DEFAULT 'Africa/Lagos',
    invoice_email VARCHAR(255),
    custom_rate_card_id VARCHAR(50),
    last_balance_update TIMESTAMP DEFAULT NOW(),
    created_at TIMESTAMP DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS billing_transactions (
    id SERIAL PRIMARY KEY,
    transaction_id VARCHAR(100) NOT NULL,
    tenant_id VARCHAR(50) NOT NULL,
    invoice_id VARCHAR(50),
    event_id VARCHAR(100),
    channel VARCHAR(20),
    base_amount DOUBLE PRECISION,
    discount_percent DOUBLE PRECISION DEFAULT 0,
    discount_amount DOUBLE PRECISION DEFAULT 0,
    sub_total DOUBLE PRECISION,
    tax_percent DOUBLE PRECISION DEFAULT 0,
    tax_amount DOUBLE PRECISION DEFAULT 0,
    final_amount DOUBLE PRECISION,
    currency VARCHAR(3) DEFAULT 'NGN',
    exchange_rate DOUBLE PRECISION DEFAULT 1.0,
    status VARCHAR(20) DEFAULT 'pending',
    idempotency_key VARCHAR(100) UNIQUE,
    created_at TIMESTAMP DEFAULT NOW(),
    processed_at TIMESTAMP
);

CREATE INDEX idx_billing_tx_tenant_id ON billing_transactions(tenant_id);
CREATE INDEX idx_billing_tx_created_at ON billing_transactions(created_at);

CREATE TABLE IF NOT EXISTS invoices (
    id SERIAL PRIMARY KEY,
    invoice_id VARCHAR(50) NOT NULL UNIQUE,
    tenant_id VARCHAR(50) NOT NULL,
    invoice_date DATE NOT NULL,
    period_start TIMESTAMP NOT NULL,
    period_end TIMESTAMP NOT NULL,
    sub_total DOUBLE PRECISION DEFAULT 0,
    total_discount DOUBLE PRECISION DEFAULT 0,
    total_tax DOUBLE PRECISION DEFAULT 0,
    total DOUBLE PRECISION DEFAULT 0,
    currency VARCHAR(3) DEFAULT 'NGN',
    payment_status VARCHAR(20) DEFAULT 'unpaid',
    payment_date TIMESTAMP,
    pdf_url TEXT,
    line_items JSONB DEFAULT '[]',
    created_at TIMESTAMP DEFAULT NOW(),
    sent_at TIMESTAMP
);

CREATE INDEX idx_invoices_tenant_id ON invoices(tenant_id);

-- ============================================================================
-- RATE CARDS
-- ============================================================================

CREATE TABLE IF NOT EXISTS rate_cards (
    id SERIAL PRIMARY KEY,
    rate_card_id VARCHAR(50) NOT NULL UNIQUE,
    name VARCHAR(100) NOT NULL,
    version INTEGER DEFAULT 1,
    effective_date TIMESTAMP NOT NULL,
    expiry_date TIMESTAMP,
    rates JSONB NOT NULL, -- channel -> ChannelRates
    is_active BOOLEAN DEFAULT TRUE,
    created_at TIMESTAMP DEFAULT NOW()
);

-- ============================================================================
-- SMS TEMPLATES (Consolidated from bulk-sms-template-[1-10])
-- ============================================================================

CREATE TABLE IF NOT EXISTS sms_templates (
    id SERIAL PRIMARY KEY,
    template_id VARCHAR(50) NOT NULL UNIQUE,
    account_id VARCHAR(45),
    name VARCHAR(100) NOT NULL,
    content TEXT NOT NULL,
    variables JSONB DEFAULT '[]', -- [{name: "firstName", type: "string"}]
    category VARCHAR(50), -- marketing, transactional, otp, notification
    language VARCHAR(10) DEFAULT 'en',
    version INTEGER DEFAULT 1,
    is_active BOOLEAN DEFAULT TRUE,
    ab_test_group VARCHAR(10), -- A, B, control
    performance_metrics JSONB DEFAULT '{"sent": 0, "delivered": 0, "clicked": 0}',
    created_at TIMESTAMP DEFAULT NOW(),
    updated_at TIMESTAMP DEFAULT NOW()
);

CREATE INDEX idx_sms_templates_account_id ON sms_templates(account_id);
CREATE INDEX idx_sms_templates_category ON sms_templates(category);

-- ============================================================================
-- CAMPAIGNS
-- ============================================================================

CREATE TABLE IF NOT EXISTS campaigns (
    id SERIAL PRIMARY KEY,
    campaign_id VARCHAR(50) NOT NULL UNIQUE,
    account_id VARCHAR(45) NOT NULL,
    name VARCHAR(100) NOT NULL,
    template_id VARCHAR(50) REFERENCES sms_templates(template_id),
    sender_id VARCHAR(255),
    status VARCHAR(20) DEFAULT 'draft', -- draft, scheduled, running, paused, completed, failed
    scheduled_at TIMESTAMP,
    started_at TIMESTAMP,
    completed_at TIMESTAMP,
    total_recipients INTEGER DEFAULT 0,
    sent_count INTEGER DEFAULT 0,
    delivered_count INTEGER DEFAULT 0,
    failed_count INTEGER DEFAULT 0,
    config JSONB, -- rate limits, retry policy, etc.
    created_at TIMESTAMP DEFAULT NOW(),
    updated_at TIMESTAMP DEFAULT NOW()
);

CREATE INDEX idx_campaigns_account_id ON campaigns(account_id);
CREATE INDEX idx_campaigns_status ON campaigns(status);

-- ============================================================================
-- SERVICE ERRORS (Debugging & Monitoring)
-- ============================================================================

CREATE TABLE IF NOT EXISTS service_errors (
    id SERIAL PRIMARY KEY,
    error_message TEXT,
    msisdn VARCHAR(20),
    service_id VARCHAR(100),
    product_id VARCHAR(100),
    network VARCHAR(10),
    account_id VARCHAR(25),
    created_at TIMESTAMP DEFAULT NOW()
);

-- ============================================================================
-- RESELLERS (From brivas-resellers-temp_backend)
-- ============================================================================

CREATE TABLE IF NOT EXISTS resellers (
    id SERIAL PRIMARY KEY,
    reseller_id VARCHAR(50) NOT NULL UNIQUE,
    parent_account_id VARCHAR(45) REFERENCES accounts(id),
    business_name VARCHAR(100) NOT NULL,
    contact_email VARCHAR(100) NOT NULL,
    contact_phone VARCHAR(20),
    commission_rate DOUBLE PRECISION DEFAULT 0,
    status VARCHAR(20) DEFAULT 'pending', -- pending, active, suspended
    credit_limit DOUBLE PRECISION DEFAULT 0,
    current_balance DOUBLE PRECISION DEFAULT 0,
    config JSONB,
    created_at TIMESTAMP DEFAULT NOW(),
    updated_at TIMESTAMP DEFAULT NOW()
);

CREATE INDEX idx_resellers_parent_account ON resellers(parent_account_id);
