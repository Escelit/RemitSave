CREATE TABLE IF NOT EXISTS savings_plans (
    id              UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id         UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    onchain_plan_id INTEGER,
    goal_name       VARCHAR(100) NOT NULL,
    target_amount   NUMERIC(30,0) NOT NULL,
    current_balance NUMERIC(30,0) NOT NULL DEFAULT 0,
    local_asset     VARCHAR(255) NOT NULL,
    lock_until      TIMESTAMPTZ,
    status          VARCHAR(20) NOT NULL DEFAULT 'Active',
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_savings_plans_user_id ON savings_plans (user_id);
