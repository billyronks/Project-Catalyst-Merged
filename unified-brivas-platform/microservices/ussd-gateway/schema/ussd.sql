-- USSD Gateway Schema for LumaDB (PostgreSQL Protocol)
-- File: microservices/ussd-gateway/schema/ussd.sql

-- USSD Applications
CREATE TABLE IF NOT EXISTS ussd_applications (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL,
    ussd_code VARCHAR(20) NOT NULL,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    callback_url VARCHAR(500) NOT NULL,
    menu_definition JSONB NOT NULL,
    default_language VARCHAR(10) DEFAULT 'en',
    supported_languages TEXT[] DEFAULT ARRAY['en'],
    billing_enabled BOOLEAN DEFAULT false,
    billing_rate DECIMAL(10,4),
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    CONSTRAINT unique_ussd_code UNIQUE (ussd_code)
);

CREATE INDEX IF NOT EXISTS idx_ussd_app_tenant ON ussd_applications(tenant_id);
CREATE INDEX IF NOT EXISTS idx_ussd_app_code ON ussd_applications(ussd_code);

-- USSD Carrier Registration
CREATE TABLE IF NOT EXISTS ussd_carrier_registrations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    application_id UUID NOT NULL REFERENCES ussd_applications(id),
    carrier_id VARCHAR(50) NOT NULL,
    carrier_code_id VARCHAR(100),
    registration_status VARCHAR(50) DEFAULT 'pending',
    registered_at TIMESTAMPTZ,
    metadata JSONB DEFAULT '{}',
    
    CONSTRAINT unique_app_carrier UNIQUE (application_id, carrier_id)
);

-- USSD Sessions
CREATE TABLE IF NOT EXISTS ussd_sessions (
    id UUID PRIMARY KEY,
    msisdn VARCHAR(20) NOT NULL,
    ussd_code VARCHAR(20) NOT NULL,
    application_id UUID NOT NULL REFERENCES ussd_applications(id),
    carrier_id VARCHAR(50) NOT NULL,
    state JSONB NOT NULL,
    session_data JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_activity_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ NOT NULL,
    ended_at TIMESTAMPTZ,
    end_reason VARCHAR(50),
    pop_region VARCHAR(50) NOT NULL DEFAULT 'default'
);

CREATE INDEX IF NOT EXISTS idx_ussd_sessions_msisdn ON ussd_sessions(msisdn);
CREATE INDEX IF NOT EXISTS idx_ussd_sessions_app ON ussd_sessions(application_id);
CREATE INDEX IF NOT EXISTS idx_ussd_sessions_active ON ussd_sessions(ended_at) WHERE ended_at IS NULL;

-- USSD Request Log
CREATE TABLE IF NOT EXISTS ussd_request_log (
    id UUID NOT NULL DEFAULT gen_random_uuid(),
    timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    session_id UUID NOT NULL,
    request_type VARCHAR(20) NOT NULL,
    user_input VARCHAR(200),
    response_type VARCHAR(20) NOT NULL,
    response_message TEXT,
    latency_ms INTEGER,
    carrier_id VARCHAR(50),
    pop_region VARCHAR(50) NOT NULL DEFAULT 'default'
);

CREATE INDEX IF NOT EXISTS idx_ussd_log_session ON ussd_request_log(session_id);
CREATE INDEX IF NOT EXISTS idx_ussd_log_timestamp ON ussd_request_log(timestamp);
