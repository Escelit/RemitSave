CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE IF NOT EXISTS users (
    id          UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    email       VARCHAR(255) NOT NULL UNIQUE,
    phone       VARCHAR(50) NOT NULL DEFAULT '',
    country     VARCHAR(5) NOT NULL,
    password_hash TEXT NOT NULL,
    kyc_level   INTEGER NOT NULL DEFAULT 0,
    stellar_address TEXT,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_active TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_users_email ON users (email);
CREATE INDEX idx_users_country ON users (country);

CREATE TABLE IF NOT EXISTS kyc_documents (
    id            UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id       UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    document_type VARCHAR(50) NOT NULL,
    status        VARCHAR(20) NOT NULL DEFAULT 'pending',
    created_at    TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_kyc_user_id ON kyc_documents (user_id);

CREATE TABLE IF NOT EXISTS sessions (
    id         UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id    UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token      TEXT NOT NULL,
    expires_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_sessions_user_id ON sessions (user_id);
CREATE INDEX idx_sessions_token ON sessions (token);

CREATE TABLE IF NOT EXISTS remittance_rules (
    id              UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id         UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    beneficiary     VARCHAR(255) NOT NULL,
    incoming_asset  VARCHAR(255) NOT NULL,
    local_asset     VARCHAR(255) NOT NULL,
    split_type      VARCHAR(20) NOT NULL,
    split_value     INTEGER NOT NULL,
    savings_plan_id VARCHAR(255),
    active          BOOLEAN NOT NULL DEFAULT TRUE,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_rules_user_id ON remittance_rules (user_id);

CREATE TABLE IF NOT EXISTS remittance_events (
    id              UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    remittance_id   INTEGER NOT NULL,
    user_id         UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    beneficiary     VARCHAR(255) NOT NULL,
    total_amount    NUMERIC(30,0) NOT NULL,
    payout_amount   NUMERIC(30,0) NOT NULL,
    savings_amount  NUMERIC(30,0) NOT NULL,
    fee_amount      NUMERIC(30,0) NOT NULL,
    incoming_asset  VARCHAR(255) NOT NULL,
    local_asset     VARCHAR(255) NOT NULL,
    status          VARCHAR(20) NOT NULL DEFAULT 'completed',
    tx_hash         TEXT,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_events_user_id ON remittance_events (user_id);
CREATE INDEX idx_events_created_at ON remittance_events (created_at);
