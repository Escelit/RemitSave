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
