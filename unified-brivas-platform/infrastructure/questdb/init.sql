-- QuestDB Schema for Real-Time Telecom Analytics
-- High-performance CDR and QoS storage

-- CDR (Call Detail Records) - Partitioned by day
CREATE TABLE IF NOT EXISTS cdr (
    call_id UUID,
    timestamp TIMESTAMP,
    source_number STRING,
    destination_number STRING,
    carrier_id UUID,
    carrier_name SYMBOL,
    duration_secs LONG,
    disposition SYMBOL,
    hangup_cause SYMBOL,
    pdd_ms LONG,
    billable_seconds LONG,
    rate DOUBLE,
    cost DOUBLE,
    revenue DOUBLE,
    source_ip STRING,
    user_agent STRING
) TIMESTAMP(timestamp) PARTITION BY DAY;

-- Real-time QoS Metrics
CREATE TABLE IF NOT EXISTS qos_metrics (
    timestamp TIMESTAMP,
    carrier_id UUID,
    carrier_name SYMBOL,
    rtt_ms DOUBLE,
    jitter_ms DOUBLE,
    packet_loss DOUBLE,
    mos DOUBLE,
    r_factor DOUBLE
) TIMESTAMP(timestamp) PARTITION BY HOUR;

-- Traffic Statistics (for dashboard)
CREATE TABLE IF NOT EXISTS traffic_stats (
    timestamp TIMESTAMP,
    calls_per_second DOUBLE,
    active_calls LONG,
    total_calls LONG,
    total_minutes DOUBLE,
    asr DOUBLE,
    acd DOUBLE
) TIMESTAMP(timestamp) PARTITION BY HOUR;

-- Fraud Detection Events
CREATE TABLE IF NOT EXISTS fraud_alerts (
    id UUID,
    timestamp TIMESTAMP,
    alert_type SYMBOL,
    severity SYMBOL,
    source_number STRING,
    destination_number STRING,
    source_ip STRING,
    risk_score DOUBLE,
    blocked BOOLEAN,
    description STRING,
    detected_patterns STRING
) TIMESTAMP(timestamp) PARTITION BY DAY;

-- Carrier Statistics (materialized every minute)
CREATE TABLE IF NOT EXISTS carrier_stats (
    timestamp TIMESTAMP,
    carrier_id UUID,
    carrier_name SYMBOL,
    total_calls LONG,
    successful_calls LONG,
    failed_calls LONG,
    asr DOUBLE,
    acd DOUBLE,
    pdd_avg DOUBLE,
    ner DOUBLE,
    revenue DOUBLE,
    cost DOUBLE
) TIMESTAMP(timestamp) PARTITION BY HOUR;

-- Destination/Prefix Analytics
CREATE TABLE IF NOT EXISTS destination_stats (
    timestamp TIMESTAMP,
    prefix SYMBOL,
    country SYMBOL,
    total_calls LONG,
    asr DOUBLE,
    acd DOUBLE,
    revenue DOUBLE,
    cost DOUBLE
) TIMESTAMP(timestamp) PARTITION BY DAY;

-- Active Calls (real-time tracking)
CREATE TABLE IF NOT EXISTS active_calls (
    call_id UUID,
    timestamp TIMESTAMP,
    source_number STRING,
    destination_number STRING,
    carrier_id UUID,
    carrier_name SYMBOL,
    duration_secs LONG,
    status SYMBOL
) TIMESTAMP(timestamp) PARTITION BY HOUR;
