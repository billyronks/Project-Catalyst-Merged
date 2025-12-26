-- API Gateway Schema for LumaDB (PostgreSQL Protocol)
-- File: microservices/api-gateway/schema/gateway.sql

-- API Keys Table
CREATE TABLE IF NOT EXISTS api_keys (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL,
    user_id UUID,
    name VARCHAR(255) NOT NULL,
    key_hash VARCHAR(128) NOT NULL UNIQUE,
    key_prefix VARCHAR(12) NOT NULL,
    scopes TEXT[] NOT NULL DEFAULT '{}',
    rate_limit_rps INTEGER DEFAULT 1000,
    rate_limit_burst INTEGER DEFAULT 5000,
    allowed_ips INET[],
    allowed_origins TEXT[],
    is_active BOOLEAN NOT NULL DEFAULT true,
    expires_at TIMESTAMPTZ,
    last_used_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    metadata JSONB DEFAULT '{}'
);

CREATE INDEX IF NOT EXISTS idx_api_keys_tenant ON api_keys(tenant_id);
CREATE INDEX IF NOT EXISTS idx_api_keys_user ON api_keys(user_id);
CREATE INDEX IF NOT EXISTS idx_api_keys_active ON api_keys(is_active) WHERE is_active = true;

-- Route Configuration Table
CREATE TABLE IF NOT EXISTS routes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    path_pattern VARCHAR(500) NOT NULL,
    methods TEXT[] NOT NULL DEFAULT '{GET}',
    backend_service VARCHAR(255) NOT NULL,
    backend_url VARCHAR(500) NOT NULL,
    protocol VARCHAR(50) NOT NULL DEFAULT 'http',
    timeout_ms INTEGER NOT NULL DEFAULT 30000,
    retries INTEGER NOT NULL DEFAULT 3,
    required_scopes TEXT[] DEFAULT '{}',
    rate_limit_override JSONB,
    transform_request JSONB,
    transform_response JSONB,
    cache_config JSONB,
    is_public BOOLEAN NOT NULL DEFAULT false,
    is_active BOOLEAN NOT NULL DEFAULT true,
    priority INTEGER NOT NULL DEFAULT 100,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_routes_active ON routes(is_active, priority);

-- Rate Limit Overrides Table
CREATE TABLE IF NOT EXISTS rate_limit_overrides (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID,
    api_key_id UUID,
    endpoint_pattern VARCHAR(500),
    requests_per_second INTEGER NOT NULL,
    burst_size INTEGER NOT NULL,
    valid_from TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    valid_until TIMESTAMPTZ,
    reason TEXT,
    created_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Request Logs (partitioned by time for analytics)
CREATE TABLE IF NOT EXISTS request_logs (
    id UUID NOT NULL DEFAULT gen_random_uuid(),
    timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    request_id UUID NOT NULL,
    tenant_id UUID,
    user_id UUID,
    method VARCHAR(10) NOT NULL,
    path VARCHAR(2000) NOT NULL,
    query_params JSONB,
    headers JSONB,
    client_ip INET,
    user_agent VARCHAR(500),
    backend_service VARCHAR(255),
    response_status INTEGER,
    response_time_ms INTEGER,
    response_size_bytes BIGINT,
    error_code VARCHAR(100),
    error_message TEXT,
    trace_id VARCHAR(64),
    span_id VARCHAR(32),
    pop_region VARCHAR(50) NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_request_logs_tenant_time ON request_logs(tenant_id, timestamp);
CREATE INDEX IF NOT EXISTS idx_request_logs_trace ON request_logs(trace_id);
