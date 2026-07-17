-- ALLBRIGHT Wallet Management Engine (WME) Schema
-- PostgreSQL initialization script for docker-compose
-- Part of Allbright DeFi Software Engineering PLC

-- Create extension for UUID generation
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Wallet Management
CREATE TABLE IF NOT EXISTS wallets (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    address VARCHAR(42) NOT NULL UNIQUE,
    chain VARCHAR(20) NOT NULL DEFAULT 'ethereum',
    wallet_type VARCHAR(20) NOT NULL DEFAULT 'hot',
    label VARCHAR(255),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_wallets_address ON wallets(address);
CREATE INDEX idx_wallets_chain ON wallets(chain);

-- Transaction Records
CREATE TABLE IF NOT EXISTS transactions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tx_hash VARCHAR(66) NOT NULL UNIQUE,
    wallet_id UUID REFERENCES wallets(id),
    chain VARCHAR(20) NOT NULL,
    from_address VARCHAR(42) NOT NULL,
    to_address VARCHAR(42),
    value NUMERIC(78, 0) NOT NULL DEFAULT 0,
    gas_price NUMERIC(78, 0),
    gas_used BIGINT,
    status VARCHAR(20) NOT NULL DEFAULT 'pending',
    trade_type VARCHAR(50),
    profit_eth NUMERIC(78, 18) DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    confirmed_at TIMESTAMPTZ
);

CREATE INDEX idx_transactions_tx_hash ON transactions(tx_hash);
CREATE INDEX idx_transactions_wallet_id ON transactions(wallet_id);
CREATE INDEX idx_transactions_status ON transactions(status);
CREATE INDEX idx_transactions_created_at ON transactions(created_at);

-- Nonce Management
CREATE TABLE IF NOT EXISTS nonces (
    wallet_id UUID REFERENCES wallets(id),
    chain VARCHAR(20) NOT NULL,
    nonce BIGINT NOT NULL DEFAULT 0,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (wallet_id, chain)
);

-- Trade Records
CREATE TABLE IF NOT EXISTS trades (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    trade_hash VARCHAR(66) UNIQUE,
    opportunity_id VARCHAR(255),
    strategy VARCHAR(100),
    dex VARCHAR(100),
    pair VARCHAR(50),
    side VARCHAR(10),
    size NUMERIC(78, 18),
    gross_profit_eth NUMERIC(78, 18),
    gas_cost_eth NUMERIC(78, 18),
    net_profit_eth NUMERIC(78, 18),
    slippage_bps INTEGER,
    executed_at TIMESTAMPTZ,
    status VARCHAR(20) DEFAULT 'pending'
);

CREATE INDEX idx_trades_strategy ON trades(strategy);
CREATE INDEX idx_trades_executed_at ON trades(executed_at);

-- KPI Metrics Storage
CREATE TABLE IF NOT EXISTS kpi_metrics (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    pillar VARCHAR(50) NOT NULL,
    kpi_name VARCHAR(100) NOT NULL,
    measured_value NUMERIC(20, 8),
    baseline_value NUMERIC(20, 8),
    recorded_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_kpi_metrics_pillar ON kpi_metrics(pillar);
CREATE INDEX idx_kpi_metrics_recorded_at ON kpi_metrics(recorded_at);

-- Alert History
CREATE TABLE IF NOT EXISTS alerts (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    severity VARCHAR(20) NOT NULL,
    message TEXT NOT NULL,
    source VARCHAR(100),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    acknowledged_at TIMESTAMPTZ,
    resolved_at TIMESTAMPTZ
);

CREATE INDEX idx_alerts_severity ON alerts(severity);
CREATE INDEX idx_alerts_created_at ON alerts(created_at);

-- Audit Log
CREATE TABLE IF NOT EXISTS audit_log (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    event_type VARCHAR(50) NOT NULL,
    actor VARCHAR(100),
    resource VARCHAR(255),
    action TEXT,
    details JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_audit_log_event_type ON audit_log(event_type);
CREATE INDEX idx_audit_log_created_at ON audit_log(created_at);

-- Fleet State
CREATE TABLE IF NOT EXISTS fleet_state (
    runner_id VARCHAR(100) PRIMARY KEY,
    status VARCHAR(50) NOT NULL DEFAULT 'offline',
    last_heartbeat TIMESTAMPTZ,
    metrics JSONB,
    config JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);