#!/bin/bash
set -euo pipefail

# RemitSave Africa - Day 4: Deployment & Initialization Script
# This script deploys the remit-save contract to a local Soroban network.

NETWORK="${NETWORK:-local}"
RPC_URL="${RPC_URL:-http://localhost:8000/soroban/rpc}"
FRIENDBOT_URL="${FRIENDBOT_URL:-http://localhost:8000/friendbot}"

fund_account() {
    local address=$1
    echo "Funding account $address..."
    curl -s -X POST "$FRIENDBOT_URL?addr=$address" > /dev/null
}

echo "--- Day 4: Stellar Integration & Deployment ---"

# 1. Build Contracts
echo "Building Soroban contracts (release)..."
(cd contracts && cargo build --target wasm32-unknown-unknown --release)

# 2. Setup Accounts
echo "Setting up identities..."
soroban config identity generate admin --network "$NETWORK" 2>/dev/null || true
soroban config identity generate user1 --network "$NETWORK" 2>/dev/null || true
soroban config identity generate beneficiary1 --network "$NETWORK" 2>/dev/null || true

ADMIN_ADDR=$(soroban config identity address admin)
USER_ADDR=$(soroban config identity address user1)
BENE_ADDR=$(soroban config identity address beneficiary1)

echo "Admin:       $ADMIN_ADDR"
echo "User:        $USER_ADDR"
echo "Beneficiary: $BENE_ADDR"

# Fund accounts on local network
if [ "$NETWORK" = "local" ]; then
    fund_account "$ADMIN_ADDR"
    fund_account "$USER_ADDR"
    fund_account "$BENE_ADDR"
fi

# 3. Deploy remit-save contract
echo "Deploying remit-save contract..."
WASM_PATH="contracts/remit-save/target/wasm32-unknown-unknown/release/remit_save.wasm"
CONTRACT_ID=$(soroban contract deploy \
    --wasm "$WASM_PATH" \
    --source admin \
    --network "$NETWORK")

echo "Contract Deployed! ID: $CONTRACT_ID"

# 4. Initialize Contract
echo "Initializing contract..."
# initialize(admin: Address, fee_recipient: Address, protocol_fee_bps: u32)
soroban contract invoke \
    --id "$CONTRACT_ID" \
    --source admin \
    --network "$NETWORK" \
    -- \
    initialize \
    --admin "$ADMIN_ADDR" \
    --fee_recipient "$ADMIN_ADDR" \
    --protocol_fee_bps 50

echo "Contract initialized (fee: 0.5%)."

# 5. Register User
echo "Registering user..."
soroban contract invoke \
    --id "$CONTRACT_ID" \
    --source user1 \
    --network "$NETWORK" \
    -- \
    register_user \
    --user "$USER_ADDR" \
    --country 'NG' \
    --phone '2348012345678'

echo "User registered."

# 6. Deploy Mock SAC Tokens (USDC and a local stablecoin)
echo "Deploying mock SAC tokens..."

# Check if soroban lab token is available; fallback to manual
if command -v soroban &> /dev/null && soroban lab token --help &> /dev/null 2>&1; then
    # Use soroban lab token to wrap a mock USDC asset
    USDC_ID=$(soroban lab token wrap \
        --asset "USDC:$ADMIN_ADDR" \
        --source admin \
        --network "$NETWORK" 2>/dev/null || true)
    
    if [ -z "$USDC_ID" ]; then
        echo "  soroban lab token wrap not available, deploying mock token manually."
        USDC_ID=""
    else
        echo "  USDC token deployed: $USDC_ID"
    fi
fi

# If token wrap didn't work, use a placeholder
if [ -z "${USDC_ID:-}" ]; then
    echo "  Note: For full SAC token deployment, run 'soroban lab token wrap' on a Stellar network."
    echo "  Using admin address as token placeholder for local testing."
    USDC_ID="$ADMIN_ADDR"
fi

# 7. Create a savings plan
echo "Creating savings plan..."
PLAN_ID=$(soroban contract invoke \
    --id "$CONTRACT_ID" \
    --source user1 \
    --network "$NETWORK" \
    -- \
    create_savings_plan \
    --owner "$USER_ADDR" \
    --goal_name 'School Fees' \
    --target_amount 10000 \
    --local_asset "$USDC_ID" \
    --lock_until 'null' | tr -d '"')

echo "Savings plan created! ID: $PLAN_ID"

# 8. Setup Remittance Rule (70% payout / 30% savings)
echo "Setting up remittance rule (70/30 split)..."
soroban contract invoke \
    --id "$CONTRACT_ID" \
    --source user1 \
    --network "$NETWORK" \
    -- \
    set_remittance_rule \
    --sender "$USER_ADDR" \
    --rule "{
        \"sender\": \"$USER_ADDR\",
        \"beneficiary\": \"$BENE_ADDR\",
        \"incoming_asset\": \"$USDC_ID\",
        \"local_asset\": \"$USDC_ID\",
        \"split_type\": \"Percentage\",
        \"split_value\": 3000,
        \"savings_plan_id\": $PLAN_ID,
        \"active\": true
    }"

echo "Remittance rule set."

# 9. Summary
echo ""
echo "--- Deployment Complete ---"
echo "RemitSave Contract: $CONTRACT_ID"
echo "Admin:              $ADMIN_ADDR"
echo "User:               $USER_ADDR"
echo "Beneficiary:        $BENE_ADDR"
echo "Savings Plan ID:    $PLAN_ID"

# Save to .env for other scripts
cat > .env.deployment <<EOF
CONTRACT_ID=$CONTRACT_ID
ADMIN_ADDR=$ADMIN_ADDR
USER_ADDR=$USER_ADDR
BENE_ADDR=$BENE_ADDR
USDC_ID=$USDC_ID
PLAN_ID=$PLAN_ID
NETWORK=$NETWORK
EOF

echo ""
echo "Deployment info saved to .env.deployment"
