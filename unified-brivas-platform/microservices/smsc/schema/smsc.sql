-- SMSC Schema for LumaDB (PostgreSQL Protocol)
-- File: microservices/smsc/schema/smsc.sql

-- Carriers
CREATE TABLE IF NOT EXISTS carriers (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    code VARCHAR(50) NOT NULL UNIQUE,
    country VARCHAR(3),
    carrier_type VARCHAR(50),
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    metadata JSONB DEFAULT '{}'
);

-- Carrier Routes
CREATE TABLE IF NOT EXISTS carrier_routes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    carrier_id UUID NOT NULL REFERENCES carriers(id),
    mcc_mnc VARCHAR(10) NOT NULL,
    priority INTEGER NOT NULL DEFAULT 100,
    cost_per_message DECIMAL(10,6) NOT NULL DEFAULT 0,
    protocol JSONB NOT NULL,
    endpoint VARCHAR(500),
    credentials JSONB,
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    CONSTRAINT unique_carrier_mcc_mnc UNIQUE (carrier_id, mcc_mnc)
);

CREATE INDEX IF NOT EXISTS idx_carrier_routes_mcc_mnc ON carrier_routes(mcc_mnc);

-- SMPP Credentials
CREATE TABLE IF NOT EXISTS smpp_credentials (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL,
    system_id VARCHAR(16) NOT NULL UNIQUE,
    password_hash VARCHAR(255) NOT NULL,
    allowed_ips INET[],
    max_connections INTEGER DEFAULT 5,
    window_size INTEGER DEFAULT 10,
    throughput_limit INTEGER DEFAULT 100,
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- SMS Messages
CREATE TABLE IF NOT EXISTS sms_messages (
    id UUID NOT NULL,
    tenant_id UUID NOT NULL,
    source VARCHAR(20) NOT NULL,
    destination VARCHAR(20) NOT NULL,
    message TEXT,
    encoded_message BYTEA,
    encoding VARCHAR(20),
    parts INTEGER NOT NULL DEFAULT 1,
    priority INTEGER NOT NULL DEFAULT 5,
    validity_period INTEGER,
    delivery_report_url VARCHAR(500),
    registered_delivery INTEGER DEFAULT 0,
    status VARCHAR(50) NOT NULL DEFAULT 'pending',
    error_code VARCHAR(20),
    error_message TEXT,
    carrier_id UUID,
    carrier_message_id VARCHAR(100),
    smpp_session_id VARCHAR(100),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    sent_at TIMESTAMPTZ,
    delivered_at TIMESTAMPTZ,
    failed_at TIMESTAMPTZ,
    metadata JSONB DEFAULT '{}',
    pop_region VARCHAR(50) NOT NULL DEFAULT 'default',
    
    PRIMARY KEY (id)
);

CREATE INDEX IF NOT EXISTS idx_sms_messages_tenant ON sms_messages(tenant_id, created_at);
CREATE INDEX IF NOT EXISTS idx_sms_messages_status ON sms_messages(status, created_at);
CREATE INDEX IF NOT EXISTS idx_sms_messages_destination ON sms_messages(destination, created_at);

-- Delivery Reports
CREATE TABLE IF NOT EXISTS delivery_reports (
    id UUID NOT NULL DEFAULT gen_random_uuid(),
    message_id UUID NOT NULL,
    status VARCHAR(50) NOT NULL,
    status_code VARCHAR(20),
    error_code VARCHAR(20),
    carrier_status VARCHAR(100),
    received_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    metadata JSONB DEFAULT '{}',
    pop_region VARCHAR(50) NOT NULL DEFAULT 'default',
    
    PRIMARY KEY (id)
);

CREATE INDEX IF NOT EXISTS idx_dlr_message ON delivery_reports(message_id);

-- Phone Prefix Rules (for routing)
CREATE TABLE IF NOT EXISTS phone_prefix_rules (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    prefix VARCHAR(20) NOT NULL,
    mcc_mnc VARCHAR(10) NOT NULL,
    country VARCHAR(100),
    carrier VARCHAR(255),
    
    CONSTRAINT unique_prefix UNIQUE (prefix)
);

CREATE INDEX IF NOT EXISTS idx_prefix_rules ON phone_prefix_rules(prefix);

-- Insert default Nigerian carriers
INSERT INTO carriers (id, name, code, country, carrier_type) VALUES
    (gen_random_uuid(), 'MTN Nigeria', 'mtn-ng', 'NGA', 'mobile'),
    (gen_random_uuid(), 'Airtel Nigeria', 'airtel-ng', 'NGA', 'mobile'),
    (gen_random_uuid(), 'Glo Nigeria', 'glo-ng', 'NGA', 'mobile'),
    (gen_random_uuid(), '9mobile Nigeria', '9mobile-ng', 'NGA', 'mobile')
ON CONFLICT DO NOTHING;

-- Insert Nigerian prefix rules
INSERT INTO phone_prefix_rules (prefix, mcc_mnc, country, carrier) VALUES
    ('234803', '62130', 'Nigeria', 'MTN'),
    ('234806', '62130', 'Nigeria', 'MTN'),
    ('234813', '62130', 'Nigeria', 'MTN'),
    ('234814', '62130', 'Nigeria', 'MTN'),
    ('234816', '62130', 'Nigeria', 'MTN'),
    ('234903', '62130', 'Nigeria', 'MTN'),
    ('234906', '62130', 'Nigeria', 'MTN'),
    ('234802', '62120', 'Nigeria', 'Airtel'),
    ('234808', '62120', 'Nigeria', 'Airtel'),
    ('234812', '62120', 'Nigeria', 'Airtel'),
    ('234901', '62120', 'Nigeria', 'Airtel'),
    ('234902', '62120', 'Nigeria', 'Airtel'),
    ('234805', '62150', 'Nigeria', 'Glo'),
    ('234807', '62150', 'Nigeria', 'Glo'),
    ('234815', '62150', 'Nigeria', 'Glo'),
    ('234905', '62150', 'Nigeria', 'Glo'),
    ('234809', '62160', 'Nigeria', '9mobile'),
    ('234817', '62160', 'Nigeria', '9mobile'),
    ('234818', '62160', 'Nigeria', '9mobile'),
    ('234908', '62160', 'Nigeria', '9mobile'),
    ('234909', '62160', 'Nigeria', '9mobile')
ON CONFLICT DO NOTHING;
