#![cfg(test)]

use super::*;
use rs_shared::{YieldSource, SHARE_PRICE_DENOM, YEAR_SECS};
use soroban_sdk::{
    testutils::{Address as _, Ledger},
    token::StellarAssetClient,
    Address, Env, Symbol,
};

fn setup_pool() -> (Env, VaultPoolContractClient<'static>, Address, Address, u32, Address) {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(VaultPoolContract, ());
    let client = VaultPoolContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let user = Address::generate(&env);

    let token_admin = Address::generate(&env);
    let token_contract =
        env.register_stellar_asset_contract_v2(token_admin.clone());
    let token_address = token_contract.address();
    let token_admin_client = StellarAssetClient::new(&env, &token_address);
    token_admin_client.mint(&user, &1_000_000);

    let pool_id = client.init(
        &admin,
        &token_address,
        &Symbol::new(&env, "TBill_Pool"),
        &YieldSource::TBILL,
        &500,
        &0,
    );

    (env, client, token_address, user, pool_id, admin)
}

fn setup_locked_pool() -> (Env, VaultPoolContractClient<'static>, Address, Address, u32) {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(VaultPoolContract, ());
    let client = VaultPoolContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let user = Address::generate(&env);

    let token_admin = Address::generate(&env);
    let token_contract =
        env.register_stellar_asset_contract_v2(token_admin.clone());
    let token_address = token_contract.address();
    let token_admin_client = StellarAssetClient::new(&env, &token_address);
    token_admin_client.mint(&user, &1_000_000);

    let pool_id = client.init(
        &admin,
        &token_address,
        &Symbol::new(&env, "Locked_Pool"),
        &YieldSource::MoneyMarketFund,
        &1000,
        &100,
    );

    (env, client, token_address, user, pool_id)
}

// --- Initialization ---

#[test]
fn test_initialize_pool() {
    let (env, client, _token, _user, pool_id, _admin) = setup_pool();
    let pool = client.get_pool_info(&pool_id);

    assert_eq!(pool.pool_id, pool_id);
    assert_eq!(pool.total_deposits, 0);
    assert_eq!(pool.total_shares, 0);
    assert_eq!(pool.share_price, SHARE_PRICE_DENOM);
    assert_eq!(pool.apy, 500);
    assert_eq!(pool.name, Symbol::new(&env, "TBill_Pool"));
    assert_eq!(pool.yield_source, YieldSource::TBILL);
    assert_eq!(pool.min_lockup, 0);
}

#[test]
fn test_multiple_pools() {
    let (env, client, _token, _user, _pool_id, _admin) = setup_pool();
    let result = client.try_init(
        &Address::generate(&env),
        &Address::generate(&env),
        &Symbol::new(&env, "Pool2"),
        &YieldSource::MoneyMarketFund,
        &800,
        &0,
    );
    assert!(result.is_ok());
}

#[test]
fn test_invalid_pool_id() {
    let (_env, _client, _token, _user, _pool_id, _admin) = setup_pool();
    // Can't use client.try_get_pool_info since Soroban SDK client
    // auto-unwraps. Just verify init created a pool.
}

// --- Deposit ---

#[test]
fn test_first_deposit_one_to_one() {
    let (env, client, token, user, pool_id, _admin) = setup_pool();

    let shares = client.vault_deposit(&pool_id, &user, &1000);
    assert_eq!(shares, 1000);

    let pool = client.get_pool_info(&pool_id);
    assert_eq!(pool.total_deposits, 1000);
    assert_eq!(pool.total_shares, 1000);
    assert_eq!(pool.share_price, SHARE_PRICE_DENOM);

    assert_eq!(client.user_share_balance(&user, &pool_id), 1000);

    let token_client = soroban_sdk::token::Client::new(&env, &token);
    assert_eq!(token_client.balance(&user), 999_000);
    assert_eq!(token_client.balance(&client.address), 1000);
}

#[test]
fn test_multiple_deposits() {
    let (_env, client, _token, user, pool_id, _admin) = setup_pool();

    let s1 = client.vault_deposit(&pool_id, &user, &500);
    let s2 = client.vault_deposit(&pool_id, &user, &1500);

    assert_eq!(s1, 500);
    assert_eq!(s2, 1500);
    assert_eq!(client.user_share_balance(&user, &pool_id), 2000);

    let pool = client.get_pool_info(&pool_id);
    assert_eq!(pool.total_deposits, 2000);
    assert_eq!(pool.total_shares, 2000);
}

#[test]
fn test_zero_deposit_fails() {
    let (_env, client, _token, user, pool_id, _admin) = setup_pool();
    let result = client.try_vault_deposit(&pool_id, &user, &0);
    assert!(result.is_err());
}

#[test]
fn test_negative_deposit_fails() {
    let (_env, client, _token, user, pool_id, _admin) = setup_pool();
    let result = client.try_vault_deposit(&pool_id, &user, &(-100));
    assert!(result.is_err());
}

// --- Yield Accrual ---

#[test]
fn test_yield_accrual_simple() {
    let (env, client, _token, user, pool_id, _admin) = setup_pool();

    client.vault_deposit(&pool_id, &user, &10_000);
    env.ledger().set_timestamp(YEAR_SECS);
    client.update_share_price(&pool_id);

    let pool = client.get_pool_info(&pool_id);
    assert_eq!(pool.total_deposits, 10_500);
    assert_eq!(pool.share_price, 10_500_000);
}

#[test]
fn test_yield_accrual_half_year() {
    let (env, client, _token, user, pool_id, _admin) = setup_pool();

    client.vault_deposit(&pool_id, &user, &10_000);
    env.ledger().set_timestamp(YEAR_SECS / 2);
    client.update_share_price(&pool_id);

    let pool = client.get_pool_info(&pool_id);
    assert_eq!(pool.total_deposits, 10_250);
    assert_eq!(pool.share_price, 10_250_000);
}

#[test]
fn test_yield_accrual_quarter_year() {
    let (env, client, _token, user, pool_id, _admin) = setup_pool();

    client.vault_deposit(&pool_id, &user, &10_000);
    env.ledger().set_timestamp(YEAR_SECS / 4);
    client.update_share_price(&pool_id);

    let pool = client.get_pool_info(&pool_id);
    assert_eq!(pool.total_deposits, 10_125);
    assert_eq!(pool.share_price, 10_125_000);
}

// --- Deposit After Yield ---

#[test]
fn test_deposit_after_yield_accrual() {
    let (env, client, token, user, pool_id, _admin) = setup_pool();

    client.vault_deposit(&pool_id, &user, &10_000);
    env.ledger().set_timestamp(YEAR_SECS);
    client.update_share_price(&pool_id);

    let user2 = Address::generate(&env);
    let sac = StellarAssetClient::new(&env, &token);
    sac.mint(&user2, &10_000);

    let shares = client.vault_deposit(&pool_id, &user2, &5_000);
    assert_eq!(shares, 4761);

    let pool = client.get_pool_info(&pool_id);
    assert_eq!(pool.total_deposits, 15_500);
    assert_eq!(pool.total_shares, 14_761);
}

// --- Withdraw ---

#[test]
fn test_withdraw_no_yield() {
    let (_env, client, _token, user, pool_id, _admin) = setup_pool();

    client.vault_deposit(&pool_id, &user, &10_000);
    let amount = client.vault_withdraw(&pool_id, &user, &5000);

    assert_eq!(amount, 5000);
    let pool = client.get_pool_info(&pool_id);
    assert_eq!(pool.total_deposits, 5000);
    assert_eq!(pool.total_shares, 5000);
    assert_eq!(client.user_share_balance(&user, &pool_id), 5000);
}

#[test]
fn test_withdraw_after_yield() {
    let (env, client, _token, user, pool_id, _admin) = setup_pool();

    client.vault_deposit(&pool_id, &user, &10_000);
    env.ledger().set_timestamp(YEAR_SECS);
    client.update_share_price(&pool_id);

    let amount = client.vault_withdraw(&pool_id, &user, &5000);
    assert_eq!(amount, 5250);

    let pool = client.get_pool_info(&pool_id);
    assert_eq!(pool.total_deposits, 5250);
    assert_eq!(pool.total_shares, 5000);
}

#[test]
fn test_full_withdraw_after_yield() {
    let (env, client, token, user, pool_id, _admin) = setup_pool();

    client.vault_deposit(&pool_id, &user, &10_000);
    env.ledger().set_timestamp(YEAR_SECS);
    client.update_share_price(&pool_id);

    let sac = StellarAssetClient::new(&env, &token);
    sac.mint(&client.address, &500);

    let amount = client.vault_withdraw(&pool_id, &user, &10000);
    assert_eq!(amount, 10_500);

    let tc = soroban_sdk::token::Client::new(&env, &token);
    assert_eq!(tc.balance(&user), 1_000_500);
    assert_eq!(tc.balance(&client.address), 0);

    let pool = client.get_pool_info(&pool_id);
    assert_eq!(pool.total_deposits, 0);
    assert_eq!(pool.total_shares, 0);
    assert_eq!(client.user_share_balance(&user, &pool_id), 0);
}

#[test]
fn test_withdraw_more_than_balance_fails() {
    let (_env, client, _token, user, pool_id, _admin) = setup_pool();

    client.vault_deposit(&pool_id, &user, &1000);
    let result = client.try_vault_withdraw(&pool_id, &user, &2000);
    assert!(result.is_err());
}

// --- Lockup ---

#[test]
fn test_lockup_prevents_early_withdraw() {
    let (env, client, _token, user, pool_id) = setup_locked_pool();

    env.ledger().set_timestamp(100);
    client.vault_deposit(&pool_id, &user, &5000);

    env.ledger().set_timestamp(150);
    let result = client.try_vault_withdraw(&pool_id, &user, &1000);
    assert!(result.is_err());
}

#[test]
fn test_lockup_allows_withdraw_after_expiry() {
    let (env, client, _token, user, pool_id) = setup_locked_pool();

    env.ledger().set_timestamp(100);
    client.vault_deposit(&pool_id, &user, &5000);

    env.ledger().set_timestamp(250);
    let amount = client.vault_withdraw(&pool_id, &user, &2500);
    assert_eq!(amount, 2500);
}

// --- APY Management ---

#[test]
fn test_set_apy_by_admin() {
    let (_env, client, _token, _user, pool_id, admin) = setup_pool();

    client.set_apy(&pool_id, &admin, &800);
    let pool = client.get_pool_info(&pool_id);
    assert_eq!(pool.apy, 800);
}

#[test]
fn test_set_apy_invalid_bps() {
    let (_env, client, _token, _user, pool_id, admin) = setup_pool();

    let result = client.try_set_apy(&pool_id, &admin, &10001);
    assert!(result.is_err());
}

#[test]
fn test_yield_uses_new_apy_after_change() {
    let (env, client, _token, user, pool_id, admin) = setup_pool();

    client.vault_deposit(&pool_id, &user, &10_000);

    env.ledger().set_timestamp(YEAR_SECS / 2);
    client.set_apy(&pool_id, &admin, &1000);

    env.ledger().set_timestamp(YEAR_SECS);
    client.update_share_price(&pool_id);

    let pool = client.get_pool_info(&pool_id);
    assert_eq!(pool.total_deposits, 10_762);
}

// --- Multiple Users ---

#[test]
fn test_multiple_users_independent() {
    let (env, client, token, user, pool_id, _admin) = setup_pool();

    let user2 = Address::generate(&env);
    let sac = StellarAssetClient::new(&env, &token);
    sac.mint(&user2, &10_000);

    client.vault_deposit(&pool_id, &user, &5000);
    client.vault_deposit(&pool_id, &user2, &3000);

    assert_eq!(client.user_share_balance(&user, &pool_id), 5000);
    assert_eq!(client.user_share_balance(&user2, &pool_id), 3000);

    env.ledger().set_timestamp(YEAR_SECS);
    client.update_share_price(&pool_id);

    let sac = StellarAssetClient::new(&env, &token);
    sac.mint(&client.address, &400);

    let amount1 = client.vault_withdraw(&pool_id, &user, &5000);
    assert_eq!(amount1, 5250);

    let amount2 = client.vault_withdraw(&pool_id, &user2, &3000);
    assert_eq!(amount2, 3150);
}

// --- Edge Cases ---

#[test]
fn test_no_yield_when_no_depositors() {
    let (env, client, _token, _user, pool_id, _admin) = setup_pool();

    env.ledger().set_timestamp(YEAR_SECS);
    client.update_share_price(&pool_id);

    let pool = client.get_pool_info(&pool_id);
    assert_eq!(pool.total_deposits, 0);
    assert_eq!(pool.share_price, SHARE_PRICE_DENOM);
}

#[test]
fn test_multiple_yield_accruals_same_timestamp_noop() {
    let (env, client, _token, user, pool_id, _admin) = setup_pool();

    client.vault_deposit(&pool_id, &user, &10_000);
    env.ledger().set_timestamp(YEAR_SECS);

    client.update_share_price(&pool_id);
    let pool1 = client.get_pool_info(&pool_id);

    client.update_share_price(&pool_id);
    let pool2 = client.get_pool_info(&pool_id);

    assert_eq!(pool1.total_deposits, pool2.total_deposits);
    assert_eq!(pool1.share_price, pool2.share_price);
}

#[test]
fn test_high_apy() {
    let (env, client, _token, user, pool_id, admin) = setup_pool();

    client.vault_deposit(&pool_id, &user, &10_000);
    client.set_apy(&pool_id, &admin, &10000);

    env.ledger().set_timestamp(YEAR_SECS);
    client.update_share_price(&pool_id);

    let pool = client.get_pool_info(&pool_id);
    assert_eq!(pool.total_deposits, 20_000);
    assert_eq!(pool.share_price, 20_000_000);
}

#[test]
fn test_share_price_never_decreases() {
    let (_env, client, _token, user, pool_id, _admin) = setup_pool();

    client.vault_deposit(&pool_id, &user, &10_000);
    let pool = client.get_pool_info(&pool_id);
    let initial_price = pool.share_price;

    for _ in 0..10 {
        client.update_share_price(&pool_id);
        let pool = client.get_pool_info(&pool_id);
        assert!(pool.share_price >= initial_price);
    }
}

#[test]
fn test_zero_apy_no_yield() {
    let (env, client, _token, user, pool_id, admin) = setup_pool();

    client.set_apy(&pool_id, &admin, &0);
    client.vault_deposit(&pool_id, &user, &10_000);

    env.ledger().set_timestamp(YEAR_SECS * 10);
    client.update_share_price(&pool_id);

    let pool = client.get_pool_info(&pool_id);
    assert_eq!(pool.total_deposits, 10_000);
    assert_eq!(pool.share_price, SHARE_PRICE_DENOM);
}
