CREATE TABLE IF NOT EXISTS vault_pools (
    id              UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    onchain_pool_id INTEGER NOT NULL UNIQUE,
    name            VARCHAR(100) NOT NULL,
    local_asset     VARCHAR(255) NOT NULL,
    apy_bps         INTEGER NOT NULL,
    min_lockup_secs BIGINT NOT NULL DEFAULT 0,
    total_deposits  NUMERIC(30,0) NOT NULL DEFAULT 0,
    total_shares    NUMERIC(30,0) NOT NULL DEFAULT 0,
    share_price     NUMERIC(30,0) NOT NULL DEFAULT 10000000,
    is_paused       BOOLEAN NOT NULL DEFAULT FALSE,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_vault_pools_onchain_id ON vault_pools (onchain_pool_id);

CREATE TABLE IF NOT EXISTS vault_positions (
    id               UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id          UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    pool_id          UUID NOT NULL REFERENCES vault_pools(id) ON DELETE CASCADE,
    shares           NUMERIC(30,0) NOT NULL DEFAULT 0,
    deposited_amount NUMERIC(30,0) NOT NULL DEFAULT 0,
    locked_until     TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_at       TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at       TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT uq_user_pool UNIQUE (user_id, pool_id)
);

CREATE INDEX IF NOT EXISTS idx_vault_positions_user_id ON vault_positions (user_id);
CREATE INDEX IF NOT EXISTS idx_vault_positions_pool_id ON vault_positions (pool_id);

-- Seed initial default vault pools
INSERT INTO vault_pools (onchain_pool_id, name, local_asset, apy_bps, min_lockup_secs, total_deposits, total_shares, share_price)
VALUES
    (0, 'NGN High Yield Vault', 'NGN', 1200, 2592000, 0, 0, 10000000),
    (1, 'KES Growth Pool', 'KES', 950, 604800, 0, 0, 10000000),
    (2, 'USD Flex Yield Pool', 'USDC', 600, 0, 0, 0, 10000000)
ON CONFLICT (onchain_pool_id) DO NOTHING;
