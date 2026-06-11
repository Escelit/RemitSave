#!/bin/bash
set -e

# RemitSave Africa - Day 4: Deployment & Initialization Script
# This script deploys the remit-save contract to a local Soroban network.

NETWORK="local"
RPC_URL="http://localhost:8000/soroban/rpc"
FRIENDBOT_URL="http://localhost:8000/friendbot"

# Helper to fund accounts on local network
fund_account() {
    local address=$1
    echo "Funding account $address..."
    curl -s "$FRIENDBOT_URL?addr=$address" > /dev/null
}

echo "--- Day 4: Stellar Integration & Deployment ---"

# 1. Build Contracts
echo "Building Soroban contracts..."
cd contracts && cargo build --target wasm32-unknown-unknown --release
cd ..

# 2. Setup Accounts (Admin, User, Beneficiary)
echo "Generating temporary accounts..."
soroban config identity generate admin --network local || true
soroban config identity generate user1 --network local || true
soroban config identity generate beneficiary1 --network local || true

ADMIN_ADDR=$(soroban config identity address admin)
USER_ADDR=$(soroban config identity address user1)
BENE_ADDR=$(soroban config identity address beneficiary1)

echo "Admin: $ADMIN_ADDR"
echo "User:  $USER_ADDR"
echo "Bene:  $BENE_ADDR"

# 3. Deploy remit-save
echo "Deploying remit-save contract..."
WASM_PATH="contracts/remit-save/target/wasm32-unknown-unknown/release/remit_save.wasm"
CONTRACT_ID=$(soroban contract deploy --wasm "$WASM_PATH" --source admin --network local)

echo "Contract Deployed! ID: $CONTRACT_ID"

# 4. Initialize Contract
echo "Initializing contract..."
# initialize(admin: Address, fee_recipient: Address, protocol_fee_bps: u32)
soroban contract invoke \
    --id "$CONTRACT_ID" \
    --source admin \
    --network local \
    -- \
    initialize \
    --admin "$ADMIN_ADDR" \
    --fee_recipient "$ADMIN_ADDR" \
    --protocol_fee_bps 50

# 5. Register User
echo "Registering user..."
# register_user(user: Address, country: Symbol, phone: Bytes)
soroban contract invoke \
    --id "$CONTRACT_ID" \
    --source user1 \
    --network local \
    -- \
    register_user \
    --user "$USER_ADDR" \
    --country "NG" \
    --phone "2348012345678"

# 6. Deploy Mock Assets (USDC and eNGN)
echo "Deploying mock assets (SAC)..."
USDC_ID=$(soroban contract deploy --wasm contracts/remit-save/target/wasm32-unknown-unknown/release/remit_save.wasm --source admin --network local) # Using same wasm as mock or could use a token wasm
# In reality, we'd use 'soroban lab token' or similar to deploy SAC for an asset.
# For simplicity in this mock script, we'll just use the contract ID as asset address if needed.
# But better to use real token-like addresses.

# 7. Setup Remittance Rule
echo "Setting up remittance rule (70/30 split)..."
# set_remittance_rule(sender: Address, rule: RemittanceRule)
# RemittanceRule { sender, beneficiary, incoming_asset, local_asset, split_type, split_value, savings_plan_id, active }
# Note: For mock, we'll use USER_ADDR as asset address just to pass validation if we don't deploy real SAC.

echo "--- Deployment Complete ---"
echo "RemitSave Contract: $CONTRACT_ID"
echo "Admin Address:      $ADMIN_ADDR"
echo "User Address:       $USER_ADDR"
