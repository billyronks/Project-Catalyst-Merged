-- Payment Service Schema for LumaDB (PostgreSQL Protocol)
-- File: microservices/payment-service/schema/payments.sql

-- Payments
CREATE TABLE IF NOT EXISTS payments (
    payment_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL,
    customer_id VARCHAR(255) NOT NULL,
    amount_minor BIGINT NOT NULL,
    currency VARCHAR(3) NOT NULL,
    fee_minor BIGINT DEFAULT 0,
    net_amount_minor BIGINT NOT NULL,
    method_type INTEGER NOT NULL,
    payment_method_id UUID,
    gateway INTEGER NOT NULL,
    status INTEGER DEFAULT 1,
    failure_code VARCHAR(50),
    failure_message TEXT,
    gateway_payment_id VARCHAR(255),
    gateway_reference VARCHAR(255),
    three_d_secure JSONB,
    description TEXT,
    invoice_id UUID,
    order_id VARCHAR(255),
    metadata JSONB DEFAULT '{}',
    processing_pop VARCHAR(20) DEFAULT 'default',
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    completed_at TIMESTAMPTZ
);

CREATE INDEX IF NOT EXISTS idx_payments_tenant ON payments(tenant_id);
CREATE INDEX IF NOT EXISTS idx_payments_customer ON payments(customer_id);
CREATE INDEX IF NOT EXISTS idx_payments_status ON payments(status);
CREATE INDEX IF NOT EXISTS idx_payments_gateway ON payments(gateway_payment_id);

-- Payment Methods
CREATE TABLE IF NOT EXISTS payment_methods (
    payment_method_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    customer_id VARCHAR(255) NOT NULL,
    type INTEGER NOT NULL,
    is_default BOOLEAN DEFAULT false,
    details JSONB NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_payment_methods_customer ON payment_methods(customer_id);

-- Refunds
CREATE TABLE IF NOT EXISTS refunds (
    refund_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    payment_id UUID NOT NULL REFERENCES payments(payment_id),
    amount_minor BIGINT NOT NULL,
    currency VARCHAR(3) NOT NULL,
    status INTEGER DEFAULT 1,
    reason TEXT,
    gateway_refund_id VARCHAR(255),
    created_at TIMESTAMPTZ DEFAULT NOW(),
    completed_at TIMESTAMPTZ
);

CREATE INDEX IF NOT EXISTS idx_refunds_payment ON refunds(payment_id);

-- Gateway Configurations
CREATE TABLE IF NOT EXISTS gateway_configs (
    config_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID,
    gateway VARCHAR(50) NOT NULL,
    environment VARCHAR(20) NOT NULL DEFAULT 'sandbox',
    credentials JSONB NOT NULL,
    settings JSONB DEFAULT '{}',
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    
    CONSTRAINT unique_gateway_config UNIQUE (tenant_id, gateway, environment)
);

-- Payment gateway enum values
COMMENT ON TABLE payments IS 'Gateway values: 1=Stripe, 2=Paystack, 3=Flutterwave, 4=MPesa, 5=PIX, 6=PayNow, 7=ACH, 8=SEPA, 9=Crypto';
COMMENT ON TABLE payments IS 'Status values: 1=Pending, 2=RequiresAction, 3=Processing, 4=Succeeded, 5=Failed, 6=Cancelled';
COMMENT ON TABLE payment_methods IS 'Type values: 1=Card, 2=BankTransfer, 3=MobileMoney, 4=Wallet, 5=Crypto';
