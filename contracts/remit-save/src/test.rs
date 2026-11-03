#![cfg(test)]

use super::*;
use soroban_sdk::{
    testutils::{Address as _, Ledger},
    token::StellarAssetClient,
    Address, Bytes, Env, Symbol,
};

fn setup_test() -> (
    Env,
    RemitSaveClient<'static>,
    Address,
    Address,
    Address,
    Address,
    Address,
) {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(RemitSave, ());
    let client = RemitSaveClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let fee_recipient = Address::generate(&env);
    let sender = Address::generate(&env);
    let beneficiary = Address::generate(&env);

    // Deploy a mock token (USDC equivalent)
    let token_admin = Address::generate(&env);
    let token_client = env.register_stellar_asset_contract_v2(token_admin.clone());
    let token_address = token_client.address();

    // Mint initial tokens to sender using StellarAssetClient
    let token_admin_client = StellarAssetClient::new(&env, &token_address);
    token_admin_client.mint(&sender, &10000);

    (
        env,
        client,
        admin,
        fee_recipient,
        sender,
        beneficiary,
        token_address,
    )
}

#[test]
fn test_initialize() {
    let (_env, client, admin, fee_recipient, _, _, _) = setup_test();

    // Initialize first time should succeed
    client.initialize(&admin, &fee_recipient, &50);

    // Initialize second time should fail
    let res = client.try_initialize(&admin, &fee_recipient, &50);
    assert!(res.is_err());
}

#[test]
fn test_register_user() {
    let (_env, client, admin, fee_recipient, sender, _, _) = setup_test();
    client.initialize(&admin, &fee_recipient, &50);

    let country = Symbol::new(&_env, "NG");
    let phone = Bytes::from_slice(&_env, b"+2348012345678");

    client.register_user(&sender, &country, &phone);

    let profile = client.get_user(&sender);
    assert_eq!(profile.stellar_address, sender);
    assert_eq!(profile.country, country);
    assert_eq!(profile.phone, phone);
    assert_eq!(profile.kyc_level, 0);
}

#[test]
fn test_percentage_split() {
    let (env, client, admin, fee_recipient, sender, beneficiary, token_address) = setup_test();
    client.initialize(&admin, &fee_recipient, &50); // 0.5% fee

    // Register user
    client.register_user(
        &sender,
        &Symbol::new(&env, "NG"),
        &Bytes::from_slice(&env, b"+234"),
    );

    // Create savings plan
    let plan_id = client.create_savings_plan(
        &sender,
        &Symbol::new(&env, "School"),
        &10000,
        &token_address,
        &None,
    );

    // Setup remittance rule: 30% savings, 70% payout
    let rule = RemittanceRule {
        sender: sender.clone(),
        beneficiary: beneficiary.clone(),
        incoming_asset: token_address.clone(),
        local_asset: token_address.clone(),
        split_type: SplitType::Percentage,
        split_value: 3000, // 30%
        savings_plan_id: Some(plan_id),
        active: true,
    };

    let rule_id = client.set_remittance_rule(&sender, &rule);

    // Execute remittance of 1000 USDC
    // total = 1000
    // fee = 1000 * 50 / 10000 = 5
    // net = 995
    // savings_incoming = 995 * 3000 / 10000 = 298.5 -> integer div -> 298
    // payout_incoming = 995 - 298 = 697
    let result = client.execute_remittance(&sender, &rule_id, &1000, &token_address);

    assert_eq!(result.fee_amount, 5);
    assert_eq!(result.savings_amount, 298);
    assert_eq!(result.payout_amount, 697);

    // Check balances (Incoming Asset)
    let token = soroban_sdk::token::Client::new(&env, &token_address);
    assert_eq!(token.balance(&sender), 9000); // 10000 - 1000
    assert_eq!(token.balance(&beneficiary), 697);
    assert_eq!(token.balance(&fee_recipient), 5);
    assert_eq!(token.balance(&client.address), 298);

    // Verify savings plan balance in contract storage (Local Asset units - 1:1 mock)
    let plan = client.get_savings_plan(&sender, &plan_id);
    assert_eq!(plan.balance, 298);
}

#[test]
fn test_fixed_split() {
    let (env, client, admin, fee_recipient, sender, beneficiary, token_address) = setup_test();
    client.initialize(&admin, &fee_recipient, &100); // 1% fee

    client.register_user(
        &sender,
        &Symbol::new(&env, "NG"),
        &Bytes::from_slice(&env, b"+234"),
    );

    let plan_id = client.create_savings_plan(
        &sender,
        &Symbol::new(&env, "Emergency"),
        &5000,
        &token_address,
        &None,
    );

    // Setup remittance rule: Fixed split of 200 units to savings
    let rule = RemittanceRule {
        sender: sender.clone(),
        beneficiary: beneficiary.clone(),
        incoming_asset: token_address.clone(),
        local_asset: token_address.clone(),
        split_type: SplitType::Fixed,
        split_value: 200,
        savings_plan_id: Some(plan_id),
        active: true,
    };

    let rule_id = client.set_remittance_rule(&sender, &rule);

    // Execute remittance of 1000 USDC
    // total = 1000
    // fee = 1000 * 100 / 10000 = 10
    // net = 990
    // savings_incoming = 200 (fixed)
    // payout_incoming = 990 - 200 = 790
    let result = client.execute_remittance(&sender, &rule_id, &1000, &token_address);

    assert_eq!(result.fee_amount, 10);
    assert_eq!(result.savings_amount, 200);
    assert_eq!(result.payout_amount, 790);

    // Check balances
    let token = soroban_sdk::token::Client::new(&env, &token_address);
    assert_eq!(token.balance(&sender), 9000);
    assert_eq!(token.balance(&beneficiary), 790);
    assert_eq!(token.balance(&fee_recipient), 10);
    assert_eq!(token.balance(&client.address), 200);
}

#[test]
fn test_no_savings_plan() {
    let (env, client, admin, fee_recipient, sender, beneficiary, token_address) = setup_test();
    client.initialize(&admin, &fee_recipient, &50);

    client.register_user(
        &sender,
        &Symbol::new(&env, "NG"),
        &Bytes::from_slice(&env, b"+234"),
    );

    // Setup remittance rule without savings plan
    let rule = RemittanceRule {
        sender: sender.clone(),
        beneficiary: beneficiary.clone(),
        incoming_asset: token_address.clone(),
        local_asset: token_address.clone(),
        split_type: SplitType::Percentage,
        split_value: 3000,
        savings_plan_id: None,
        active: true,
    };

    let rule_id = client.set_remittance_rule(&sender, &rule);

    // Execute remittance of 1000 USDC
    // total = 1000
    // fee = 5
    // net = 995
    // savings = 0
    // payout = 995
    let result = client.execute_remittance(&sender, &rule_id, &1000, &token_address);

    assert_eq!(result.fee_amount, 5);
    assert_eq!(result.savings_amount, 0);
    assert_eq!(result.payout_amount, 995);

    let token = soroban_sdk::token::Client::new(&env, &token_address);
    assert_eq!(token.balance(&beneficiary), 995);
    assert_eq!(token.balance(&client.address), 0);
}

#[test]
fn test_set_anchor() {
    let (env, client, admin, fee_recipient, sender, beneficiary, token_address) = setup_test();
    client.initialize(&admin, &fee_recipient, &0);

    let anchor = Address::generate(&env);

    // Only admin can set anchor
    client.set_anchor(&token_address, &anchor);

    // Register user & create rule
    client.register_user(
        &sender,
        &Symbol::new(&env, "NG"),
        &Bytes::from_slice(&env, b"+234"),
    );
    let rule = RemittanceRule {
        sender: sender.clone(),
        beneficiary: beneficiary.clone(),
        incoming_asset: token_address.clone(),
        local_asset: token_address.clone(),
        split_type: SplitType::Percentage,
        split_value: 0,
        savings_plan_id: None,
        active: true,
    };
    let rule_id = client.set_remittance_rule(&sender, &rule);

    // Execute remittance - funds should go to anchor instead of beneficiary directly
    client.execute_remittance(&sender, &rule_id, &1000, &token_address);

    let token = soroban_sdk::token::Client::new(&env, &token_address);
    assert_eq!(token.balance(&anchor), 1000);
    assert_eq!(token.balance(&beneficiary), 0);
}

#[test]
fn test_withdraw_and_timelock() {
    let (env, client, admin, fee_recipient, sender, beneficiary, token_address) = setup_test();
    client.initialize(&admin, &fee_recipient, &0); // no fee

    client.register_user(
        &sender,
        &Symbol::new(&env, "NG"),
        &Bytes::from_slice(&env, b"+234"),
    );

    // Set lock until timestamp = 100
    let plan_id = client.create_savings_plan(
        &sender,
        &Symbol::new(&env, "Timelocked"),
        &10000,
        &token_address,
        &Some(100),
    );

    let rule = RemittanceRule {
        sender: sender.clone(),
        beneficiary: beneficiary.clone(),
        incoming_asset: token_address.clone(),
        local_asset: token_address.clone(),
        split_type: SplitType::Percentage,
        split_value: 10000, // 100% savings
        savings_plan_id: Some(plan_id),
        active: true,
    };

    let rule_id = client.set_remittance_rule(&sender, &rule);

    client.execute_remittance(&sender, &rule_id, &500, &token_address);

    // Attempt withdrawal before lock_until
    env.ledger().set_timestamp(50);
    let res = client.try_withdraw_from_plan(&sender, &plan_id);
    assert!(res.is_err());

    // Advance ledger timestamp past lock_until
    env.ledger().set_timestamp(101);

    let withdrawn = client.withdraw_from_plan(&sender, &plan_id);
    assert_eq!(withdrawn, 500);

    let token = soroban_sdk::token::Client::new(&env, &token_address);
    assert_eq!(token.balance(&sender), 10000);
    assert_eq!(token.balance(&client.address), 0);
}

#[test]
fn test_two_phase_admin_transfer() {
    let (env, client, admin, fee_recipient, _, _, _) = setup_test();
    client.initialize(&admin, &fee_recipient, &50);

    assert_eq!(client.get_admin(), admin);

    // No pending admin initially
    assert!(client.try_get_proposed_admin().is_err());
    assert!(client.try_accept_admin().is_err());

    // Propose new admin
    let new_admin = Address::generate(&env);
    client.propose_admin(&new_admin);
    assert_eq!(client.get_proposed_admin(), new_admin);

    // Accept admin transfer
    client.accept_admin();
    assert_eq!(client.get_admin(), new_admin);

    // Proposed admin is now cleared
    assert!(client.try_get_proposed_admin().is_err());
}

#[test]
fn test_emergency_pause_mechanism() {
    let (env, client, admin, fee_recipient, sender, beneficiary, token_address) = setup_test();
    client.initialize(&admin, &fee_recipient, &50);

    assert!(!client.is_paused());

    // Register user and create rule
    client.register_user(
        &sender,
        &Symbol::new(&env, "NG"),
        &Bytes::from_slice(&env, b"+234"),
    );
    let plan_id = client.create_savings_plan(
        &sender,
        &Symbol::new(&env, "School"),
        &10000,
        &token_address,
        &None,
    );
    let rule = RemittanceRule {
        sender: sender.clone(),
        beneficiary: beneficiary.clone(),
        incoming_asset: token_address.clone(),
        local_asset: token_address.clone(),
        split_type: SplitType::Percentage,
        split_value: 3000,
        savings_plan_id: Some(plan_id),
        active: true,
    };
    let rule_id = client.set_remittance_rule(&sender, &rule);

    // Pause contract
    client.pause();
    assert!(client.is_paused());

    // Remittance execution fails when paused
    let remit_res = client.try_execute_remittance(&sender, &rule_id, &1000, &token_address);
    assert_eq!(remit_res, Err(Ok(RemitError::ContractPaused)));

    // Deposit to plan fails when paused
    let dep_res = client.try_deposit_to_plan(&sender, &plan_id, &100, &token_address);
    assert_eq!(dep_res, Err(Ok(RemitError::ContractPaused)));

    // Withdraw from plan fails when paused
    let w_res = client.try_withdraw_from_plan(&sender, &plan_id);
    assert_eq!(w_res, Err(Ok(RemitError::ContractPaused)));

    // Unpause contract
    client.unpause();
    assert!(!client.is_paused());

    // Remittance succeeds after unpausing
    let result = client.execute_remittance(&sender, &rule_id, &1000, &token_address);
    assert_eq!(result.fee_amount, 5);
}

#[test]
fn test_partial_plan_withdrawal() {
    let (env, client, admin, fee_recipient, sender, _, token_address) = setup_test();
    client.initialize(&admin, &fee_recipient, &0);

    client.register_user(
        &sender,
        &Symbol::new(&env, "NG"),
        &Bytes::from_slice(&env, b"+234"),
    );

    let plan_id = client.create_savings_plan(
        &sender,
        &Symbol::new(&env, "Emergency"),
        &1000,
        &token_address,
        &None,
    );

    // Deposit 500 units
    client.deposit_to_plan(&sender, &plan_id, &500, &token_address);
    let plan = client.get_savings_plan(&sender, &plan_id);
    assert_eq!(plan.balance, 500);
    assert_eq!(plan.status, PlanStatus::Active);

    // Partial withdrawal of 200
    let withdrawn = client.withdraw_amount_from_plan(&sender, &plan_id, &200);
    assert_eq!(withdrawn, 200);

    let plan_after = client.get_savings_plan(&sender, &plan_id);
    assert_eq!(plan_after.balance, 300);
    assert_eq!(plan_after.status, PlanStatus::Active);

    // Exceeding balance fails
    let err_res = client.try_withdraw_amount_from_plan(&sender, &plan_id, &500);
    assert!(err_res.is_err());

    // Withdrawing remaining 300 sets status to Withdrawn
    let withdrawn_final = client.withdraw_amount_from_plan(&sender, &plan_id, &300);
    assert_eq!(withdrawn_final, 300);

    let plan_withdrawn = client.get_savings_plan(&sender, &plan_id);
    assert_eq!(plan_withdrawn.balance, 0);
    assert_eq!(plan_withdrawn.status, PlanStatus::Withdrawn);
}

#[test]
fn test_goal_completion_transition() {
    let (env, client, admin, fee_recipient, sender, _, token_address) = setup_test();
    client.initialize(&admin, &fee_recipient, &0);

    client.register_user(
        &sender,
        &Symbol::new(&env, "NG"),
        &Bytes::from_slice(&env, b"+234"),
    );

    let plan_id = client.create_savings_plan(
        &sender,
        &Symbol::new(&env, "Laptop"),
        &500,
        &token_address,
        &None,
    );

    // Deposit 300 (< 500 target)
    client.deposit_to_plan(&sender, &plan_id, &300, &token_address);
    let plan = client.get_savings_plan(&sender, &plan_id);
    assert_eq!(plan.status, PlanStatus::Active);

    // Deposit another 200 (hits 500 target)
    client.deposit_to_plan(&sender, &plan_id, &200, &token_address);
    let plan_completed = client.get_savings_plan(&sender, &plan_id);
    assert_eq!(plan_completed.status, PlanStatus::Completed);
    assert_eq!(plan_completed.balance, 500);

    // Partial withdrawal from completed plan
    let withdrawn = client.withdraw_amount_from_plan(&sender, &plan_id, &100);
    assert_eq!(withdrawn, 100);

    // Plan drops below target so becomes Active again
    let plan_active = client.get_savings_plan(&sender, &plan_id);
    assert_eq!(plan_active.status, PlanStatus::Active);
    assert_eq!(plan_active.balance, 400);
}

#[test]
fn test_savings_plan_deposit_to_vault() {
    let (env, client, admin, fee_recipient, sender, _, token_address) = setup_test();
    let vault_contract = Address::generate(&env);
    let token_client = soroban_sdk::token::Client::new(&env, &token_address);

    client.initialize(&admin, &fee_recipient, &0);
    client.set_vault_contract(&vault_contract);
    assert_eq!(client.get_vault_contract(), vault_contract);

    client.register_user(
        &sender,
        &Symbol::new(&env, "NG"),
        &Bytes::from_slice(&env, b"+234"),
    );

    let plan_id = client.create_savings_plan(
        &sender,
        &Symbol::new(&env, "YieldGoal"),
        &1000,
        &token_address,
        &None,
    );

    client.deposit_to_plan(&sender, &plan_id, &500, &token_address);
    let plan = client.get_savings_plan(&sender, &plan_id);
    assert_eq!(plan.balance, 500);

    // Route 300 to vault pool #0
    let deposited = client.deposit_plan_to_vault(&sender, &plan_id, &0, &300);
    assert_eq!(deposited, 300);

    let plan_after = client.get_savings_plan(&sender, &plan_id);
    assert_eq!(plan_after.balance, 200);
    assert_eq!(token_client.balance(&vault_contract), 300);
}
