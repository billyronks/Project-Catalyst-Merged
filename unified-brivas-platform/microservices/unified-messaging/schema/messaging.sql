-- Unified Messaging Hub Schema for LumaDB (PostgreSQL Protocol)
-- File: microservices/unified-messaging/schema/messaging.sql

-- Conversations
CREATE TABLE IF NOT EXISTS conversations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL,
    platform VARCHAR(50) NOT NULL,
    participant_id VARCHAR(255) NOT NULL,
    participant_name VARCHAR(255),
    participant_metadata JSONB DEFAULT '{}',
    status VARCHAR(50) DEFAULT 'open',
    last_message_at TIMESTAMPTZ,
    unread_count INTEGER DEFAULT 0,
    assigned_to UUID,
    tags TEXT[],
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    CONSTRAINT unique_conversation UNIQUE (tenant_id, platform, participant_id)
);

CREATE INDEX IF NOT EXISTS idx_conversations_tenant ON conversations(tenant_id);
CREATE INDEX IF NOT EXISTS idx_conversations_platform ON conversations(platform);
CREATE INDEX IF NOT EXISTS idx_conversations_status ON conversations(status);

-- Messages
CREATE TABLE IF NOT EXISTS messages (
    id UUID NOT NULL,
    conversation_id UUID NOT NULL REFERENCES conversations(id),
    platform VARCHAR(50) NOT NULL,
    direction VARCHAR(20) NOT NULL,
    sender_id VARCHAR(255) NOT NULL,
    recipient_id VARCHAR(255) NOT NULL,
    content JSONB NOT NULL,
    reply_to_message_id VARCHAR(255),
    platform_message_id VARCHAR(255),
    status VARCHAR(50) NOT NULL DEFAULT 'pending',
    error_code VARCHAR(50),
    error_message TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    sent_at TIMESTAMPTZ,
    delivered_at TIMESTAMPTZ,
    read_at TIMESTAMPTZ,
    metadata JSONB DEFAULT '{}',
    pop_region VARCHAR(50) NOT NULL DEFAULT 'default',
    
    PRIMARY KEY (id)
);

CREATE INDEX IF NOT EXISTS idx_messages_conversation ON messages(conversation_id, created_at);
CREATE INDEX IF NOT EXISTS idx_messages_platform_id ON messages(platform_message_id);

-- Webhooks
CREATE TABLE IF NOT EXISTS webhooks (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL,
    url VARCHAR(500) NOT NULL,
    platforms TEXT[] NOT NULL,
    events TEXT[] NOT NULL DEFAULT ARRAY['message.received', 'message.status'],
    secret VARCHAR(255),
    verification_token VARCHAR(255),
    is_active BOOLEAN DEFAULT true,
    failure_count INTEGER DEFAULT 0,
    last_failure_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_webhooks_tenant ON webhooks(tenant_id);

-- Platform Configurations
CREATE TABLE IF NOT EXISTS platform_configurations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL,
    platform VARCHAR(50) NOT NULL,
    credentials JSONB NOT NULL,
    settings JSONB DEFAULT '{}',
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    CONSTRAINT unique_platform_config UNIQUE (tenant_id, platform)
);

-- Message Templates (for WhatsApp, etc.)
CREATE TABLE IF NOT EXISTS message_templates (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL,
    platform VARCHAR(50) NOT NULL,
    name VARCHAR(255) NOT NULL,
    language_code VARCHAR(10) NOT NULL,
    category VARCHAR(50),
    content JSONB NOT NULL,
    variables TEXT[],
    status VARCHAR(50) DEFAULT 'pending',
    platform_template_id VARCHAR(255),
    rejection_reason TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    CONSTRAINT unique_template UNIQUE (tenant_id, platform, name, language_code)
);

CREATE INDEX IF NOT EXISTS idx_templates_tenant ON message_templates(tenant_id);
