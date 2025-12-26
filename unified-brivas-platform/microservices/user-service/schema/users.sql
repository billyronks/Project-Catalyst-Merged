-- User Service Schema for LumaDB (PostgreSQL Protocol)
-- File: microservices/user-service/schema/users.sql

-- Tenants Table (Multi-tenancy support)
CREATE TABLE IF NOT EXISTS tenants (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    slug VARCHAR(100) NOT NULL UNIQUE,
    domain VARCHAR(255),
    logo_url VARCHAR(500),
    settings JSONB DEFAULT '{}',
    tier VARCHAR(50) NOT NULL DEFAULT 'free',
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    metadata JSONB DEFAULT '{}'
);

CREATE INDEX IF NOT EXISTS idx_tenants_slug ON tenants(slug);
CREATE INDEX IF NOT EXISTS idx_tenants_domain ON tenants(domain);

-- Users Table
CREATE TABLE IF NOT EXISTS users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    email VARCHAR(255) NOT NULL,
    password_hash VARCHAR(255),
    first_name VARCHAR(100),
    last_name VARCHAR(100),
    phone VARCHAR(50),
    avatar_url VARCHAR(500),
    email_verified BOOLEAN NOT NULL DEFAULT false,
    phone_verified BOOLEAN NOT NULL DEFAULT false,
    status VARCHAR(50) NOT NULL DEFAULT 'pending_verification',
    mfa_enabled BOOLEAN NOT NULL DEFAULT false,
    mfa_secret VARCHAR(255),
    mfa_backup_codes TEXT[],
    failed_login_attempts INTEGER NOT NULL DEFAULT 0,
    locked_until TIMESTAMPTZ,
    password_changed_at TIMESTAMPTZ,
    last_login_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    metadata JSONB DEFAULT '{}',
    
    CONSTRAINT unique_email_per_tenant UNIQUE (tenant_id, email)
);

CREATE INDEX IF NOT EXISTS idx_users_tenant ON users(tenant_id);
CREATE INDEX IF NOT EXISTS idx_users_email ON users(email);
CREATE INDEX IF NOT EXISTS idx_users_status ON users(status);

-- Roles Table
CREATE TABLE IF NOT EXISTS roles (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID REFERENCES tenants(id) ON DELETE CASCADE,
    name VARCHAR(100) NOT NULL,
    description TEXT,
    is_system BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    CONSTRAINT unique_role_name UNIQUE (tenant_id, name)
);

-- Permissions Table
CREATE TABLE IF NOT EXISTS permissions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    resource VARCHAR(100) NOT NULL,
    action VARCHAR(100) NOT NULL,
    description TEXT,
    conditions JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    CONSTRAINT unique_permission UNIQUE (resource, action)
);

-- Role Permissions Junction
CREATE TABLE IF NOT EXISTS role_permissions (
    role_id UUID NOT NULL REFERENCES roles(id) ON DELETE CASCADE,
    permission_id UUID NOT NULL REFERENCES permissions(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    PRIMARY KEY (role_id, permission_id)
);

-- User Roles Junction
CREATE TABLE IF NOT EXISTS user_roles (
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    role_id UUID NOT NULL REFERENCES roles(id) ON DELETE CASCADE,
    assigned_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    assigned_by UUID REFERENCES users(id),
    
    PRIMARY KEY (user_id, tenant_id, role_id)
);

CREATE INDEX IF NOT EXISTS idx_user_roles_tenant ON user_roles(tenant_id);

-- Sessions Table
CREATE TABLE IF NOT EXISTS sessions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    device_info JSONB DEFAULT '{}',
    ip_address INET,
    user_agent VARCHAR(500),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_active_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT true,
    revoked_at TIMESTAMPTZ,
    revoked_reason VARCHAR(255)
);

CREATE INDEX IF NOT EXISTS idx_sessions_user ON sessions(user_id);
CREATE INDEX IF NOT EXISTS idx_sessions_active ON sessions(user_id, is_active) WHERE is_active = true;
CREATE INDEX IF NOT EXISTS idx_sessions_expires ON sessions(expires_at) WHERE is_active = true;

-- OAuth Connections
CREATE TABLE IF NOT EXISTS oauth_connections (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    provider VARCHAR(50) NOT NULL,
    provider_user_id VARCHAR(255) NOT NULL,
    access_token_encrypted TEXT,
    refresh_token_encrypted TEXT,
    token_expires_at TIMESTAMPTZ,
    profile_data JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    CONSTRAINT unique_oauth UNIQUE (provider, provider_user_id)
);

CREATE INDEX IF NOT EXISTS idx_oauth_user ON oauth_connections(user_id);

-- Audit Log
CREATE TABLE IF NOT EXISTS user_audit_log (
    id UUID NOT NULL DEFAULT gen_random_uuid(),
    timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    user_id UUID,
    tenant_id UUID,
    actor_id UUID,
    action VARCHAR(100) NOT NULL,
    resource_type VARCHAR(100),
    resource_id UUID,
    old_values JSONB,
    new_values JSONB,
    ip_address INET,
    user_agent VARCHAR(500),
    metadata JSONB DEFAULT '{}'
);

CREATE INDEX IF NOT EXISTS idx_audit_user ON user_audit_log(user_id, timestamp);
CREATE INDEX IF NOT EXISTS idx_audit_tenant ON user_audit_log(tenant_id, timestamp);
CREATE INDEX IF NOT EXISTS idx_audit_action ON user_audit_log(action, timestamp);

-- Insert default system roles
INSERT INTO roles (id, name, description, is_system) VALUES
    (gen_random_uuid(), 'super_admin', 'Full system access', true),
    (gen_random_uuid(), 'admin', 'Tenant administration', true),
    (gen_random_uuid(), 'manager', 'Team management', true),
    (gen_random_uuid(), 'user', 'Standard user access', true),
    (gen_random_uuid(), 'viewer', 'Read-only access', true)
ON CONFLICT DO NOTHING;

-- Insert default permissions
INSERT INTO permissions (resource, action, description) VALUES
    ('*', '*', 'Super admin - all permissions'),
    ('users', 'create', 'Create users'),
    ('users', 'read', 'View users'),
    ('users', 'update', 'Update users'),
    ('users', 'delete', 'Delete users'),
    ('roles', 'manage', 'Manage roles and permissions'),
    ('billing', 'manage', 'Manage billing'),
    ('billing', 'view', 'View billing information'),
    ('api_keys', 'manage', 'Manage API keys'),
    ('messaging', 'send', 'Send messages'),
    ('messaging', 'view', 'View message history'),
    ('ussd', 'manage', 'Manage USSD services'),
    ('voice', 'manage', 'Manage voice services'),
    ('reports', 'view', 'View reports'),
    ('reports', 'export', 'Export reports')
ON CONFLICT DO NOTHING;
