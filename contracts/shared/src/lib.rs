#![no_std]
use soroban_sdk::{contracttype, Address, Bytes, Symbol};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UserProfile {
    pub stellar_address: Address,
    pub country: Symbol,                // "NG", "KE", "GH", "UG", "RW", "ZA"
    pub phone: Bytes,                   // E.164 format
    pub kyc_level: u32,                 // 0 = unverified, 1 = basic, 2 = enhanced
    pub created_at: u64,
    pub last_active: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PlanStatus { 
    Active, 
    Completed, 
    Withdrawn, 
    Closed 
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SavingsPlan {
    pub owner: Address,
    pub plan_id: u32,
    pub goal_name: Symbol,              // "School Fees", "Emergency Fund", "Retirement"
    pub target_amount: i128,            // target in LOCAL stablecoin units (e.g., eNGN)
    pub balance: i128,                  // always in local stablecoin (eNGN, eKES, etc.)
    pub local_asset: Address,           // local stablecoin (e.g., Cowrie's eNGN, eKES)
    pub auto_save_pct: u32,             // 0–10000 in basis points (e.g., 3000 = 30%)
    pub lock_until: Option<u64>,        // epoch seconds, None = no timelock
    pub status: PlanStatus,
    pub created_at: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SplitType { 
    Percentage, 
    Fixed 
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RemittanceRule {
    pub sender: Address,
    pub beneficiary: Address,           // address of the receiver on Stellar
    pub incoming_asset: Address,        // what the sender sends (e.g., USDC)
    pub local_asset: Address,           // what savings land in (e.g., eNGN, eKES)
    pub split_type: SplitType,          // Percentage or Fixed
    pub split_value: u32,               // basis points if Percentage, absolute amount if Fixed
    pub savings_plan_id: Option<u32>,   // plan denominated in local_asset
    pub active: bool,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum YieldSource { 
    TBILL, 
    MoneyMarketFund, 
    LendingPool, 
    Staking 
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VaultPool {
    pub pool_id: u32,
    pub local_asset: Address,           // local stablecoin (eNGN, eKES, etc.)
    pub name: Symbol,                   // "Nigerian T-Bill Pool", "KSh Money Market"
    pub total_deposits: i128,
    pub total_shares: i128,
    pub share_price: i128,              // in basis points (1e7 = 1.0)
    pub yield_source: YieldSource,
    pub apy: u32,                       // basis points (500 = 5.00%)
    pub admin: Address,
    pub min_lockup: u64,                // seconds
    pub created_at: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataKey {
    User(Address),
    Plan(Address, u32),
    Rule(Address, u32),
    Pool(u32),
    GlobalPlanCount(Address),
    GlobalRuleCount(Address),
    GlobalPoolCount,
}
