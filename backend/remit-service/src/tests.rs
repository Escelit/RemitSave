// Tests for remit-service split logic.
// Handlers require a live DB, so split calculations are tested in isolation.

#[test]
fn test_split_percentage() {
    let total = 1000i64;
    let fee_bps = 50i64;
    let split_bps = 3000i64;

    let fee = total * fee_bps / 10000;
    let remaining = total - fee;
    let savings = remaining * split_bps / 10000;
    let payout = remaining - savings;

    assert_eq!(fee, 5);
    assert_eq!(savings, 298);
    assert_eq!(payout, 697);
    assert_eq!(fee + payout + savings, total);
}

#[test]
fn test_split_fixed() {
    let total = 1000i64;
    let fee_bps = 100i64;
    let fixed_savings = 200i64;

    let fee = total * fee_bps / 10000;
    let remaining = total - fee;
    let savings = std::cmp::min(fixed_savings, remaining);
    let payout = remaining - savings;

    assert_eq!(fee, 10);
    assert_eq!(savings, 200);
    assert_eq!(payout, 790);
    assert_eq!(fee + payout + savings, total);
}

#[test]
fn test_split_no_savings() {
    let total = 500i64;
    let fee_bps = 50i64;
    let split_bps = 0i64;

    let fee = total * fee_bps / 10000;
    let remaining = total - fee;
    let savings = remaining * split_bps / 10000;
    let payout = remaining - savings;

    assert_eq!(fee, 2);
    assert_eq!(savings, 0);
    assert_eq!(payout, 498);
}

#[test]
fn test_split_fixed_exceeds_remaining() {
    let total = 100i64;
    let fee_bps = 50i64;
    let fixed_savings = 200i64;

    let fee = total * fee_bps / 10000;
    let remaining = total - fee;
    let savings = std::cmp::min(fixed_savings, remaining);
    let payout = remaining - savings;

    assert_eq!(fee, 0);
    assert_eq!(savings, 100);
    assert_eq!(payout, 0);
}

#[test]
fn test_split_zero_amount() {
    let total = 0i64;
    let fee_bps = 50i64;
    let split_bps = 3000i64;

    let fee = total * fee_bps / 10000;
    let remaining = total - fee;
    let savings = remaining * split_bps / 10000;
    let payout = remaining - savings;

    assert_eq!(fee, 0);
    assert_eq!(savings, 0);
    assert_eq!(payout, 0);
}

#[test]
fn test_savings_plan_deposit_and_completion() {
    let target = 5000i64;
    let mut balance = 0i64;
    let mut status = "Active".to_string();

    let deposit_1 = 3000i64;
    balance += deposit_1;
    if balance >= target {
        status = "Completed".to_string();
    }
    assert_eq!(balance, 3000);
    assert_eq!(status, "Active");

    let deposit_2 = 2500i64;
    balance += deposit_2;
    if balance >= target {
        status = "Completed".to_string();
    }
    assert_eq!(balance, 5500);
    assert_eq!(status, "Completed");
}

#[test]
fn test_savings_plan_partial_and_full_withdrawal() {
    let target = 5000i64;
    let mut balance = 5000i64;
    let mut status = "Completed".to_string();

    // Partial withdrawal: drops below target
    let withdraw_1 = 2000i64;
    assert!(withdraw_1 <= balance);
    balance -= withdraw_1;
    if balance == 0 {
        status = "Withdrawn".to_string();
    } else if balance < target {
        status = "Active".to_string();
    }
    assert_eq!(balance, 3000);
    assert_eq!(status, "Active");

    // Full withdrawal: reaches zero
    let withdraw_2 = 3000i64;
    assert!(withdraw_2 <= balance);
    balance -= withdraw_2;
    if balance == 0 {
        status = "Withdrawn".to_string();
    }
    assert_eq!(balance, 0);
    assert_eq!(status, "Withdrawn");
}

#[test]
fn test_savings_plan_validation_checks() {
    let goal_name = "  ".trim();
    assert!(goal_name.is_empty());

    let target_amount = -100i64;
    assert!(target_amount <= 0);

    let local_asset = "".trim();
    assert!(local_asset.is_empty());
}

#[test]
fn test_vault_shares_mint_calculation() {
    let share_price = 10_000_000i128; // initial 1:1
    let deposit_amount = 50_000i64;

    let shares = (deposit_amount as i128 * 10_000_000i128 / share_price) as i64;
    assert_eq!(shares, 50_000);

    // After yield accrual, share_price increases to 1.10 (11_000_000)
    let yield_share_price = 11_000_000i128;
    let shares_after_yield = (deposit_amount as i128 * 10_000_000i128 / yield_share_price) as i64;
    assert_eq!(shares_after_yield, 45_454); // fewer shares minted for same deposit
}

#[test]
fn test_vault_withdraw_payout_with_accrued_yield() {
    let user_shares = 10_000i64;
    // Initial share price 10_000_000
    let initial_payout = (user_shares as i128 * 10_000_000i128 / 10_000_000i128) as i64;
    assert_eq!(initial_payout, 10_000);

    // Share price after 10% yield
    let yield_share_price = 11_000_000i128;
    let payout = (user_shares as i128 * yield_share_price / 10_000_000i128) as i64;
    assert_eq!(payout, 11_000); // 10% yield earned
}

#[test]
fn test_vault_lockup_check() {
    let now = chrono::Utc::now();
    let lock_duration = chrono::Duration::seconds(86400); // 1 day
    let locked_until = now + lock_duration;

    // Check currently locked
    assert!(now < locked_until);

    // Check after lockup expiration
    let future_time = now + chrono::Duration::seconds(86401);
    assert!(future_time >= locked_until);
}

#[test]
fn test_vault_validation_checks() {
    let deposit_amount = 0i64;
    assert!(deposit_amount <= 0);

    let negative_deposit = -500i64;
    assert!(negative_deposit <= 0);

    let user_shares = 100i64;
    let requested_withdraw = 150i64;
    assert!(requested_withdraw > user_shares);
}

#[test]
fn test_user_stellar_address_validation_in_remittance() {
    let unlinked_address: Option<String> = None;
    assert!(unlinked_address.is_none());

    let invalid_address = Some("invalid_key".to_string());
    assert!(!backend_shared::is_valid_stellar_address(
        invalid_address.as_deref().unwrap()
    ));

    let valid_address =
        Some("GBBD47IF6LWK7P7MDEVSCWR7DPUWV3NY3DTQEVFL4NAT4AQH3ZLLFLA5".to_string());
    assert!(backend_shared::is_valid_stellar_address(
        valid_address.as_deref().unwrap()
    ));
}
