# RemitSave Africa

[![CI](https://github.com/promise-ogazi/RemitSave/actions/workflows/ci.yml/badge.svg)](https://github.com/promise-ogazi/RemitSave/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-1.78%2B-dea584)](https://rustup.rs)
[![Node](https://img.shields.io/badge/Node-20%2B-339933)](https://nodejs.org)
[![Soroban](https://img.shields.io/badge/Soroban-latest-04b5e5)](https://soroban.stellar.org)
[![PRs Welcome](https://img.shields.io/badge/PRs-welcome-brightgreen)](.github/PULL_REQUEST_TEMPLATE.md)

**Cross-border remittances + automated savings, powered by Stellar & Soroban**

RemitSave Africa is an end-to-end platform that lets the African diaspora send money home while simultaneously building savings — automatically. Every remittance can be split on-chain: a percentage goes directly to the beneficiary's mobile money, and the remainder is deposited into yield-bearing savings vaults (fixed deposits, T-bill pools, or goal-based plans).

---

## Table of Contents

- [Vision](#vision)
- [Why This Matters](#why-this-matters)
- [System Architecture](#system-architecture)
- [Smart Contracts (Soroban)](#smart-contracts-soroban)
  - [Data Models](#data-models)
  - [Contract Interface](#contract-interface)
  - [Key Flows](#key-flows)
- [Frontend](#frontend)
  - [User App (Diaspora Sender)](#user-app-diaspora-sender)
  - [Beneficiary App (Web + USSD)](#beneficiary-app-web--ussd)
  - [Shared Component Library](#shared-component-library)
- [Backend / Off-Chain Services](#backend--off-chain-services)
  - [Anchor Relayer Service](#anchor-relayer-service)
  - [Schedule Keeper](#schedule-keeper)
  - [Webhook & Notification Service](#webhook--notification-service)
  - [Price Oracle Adapter](#price-oracle-adapter)
  - [Analytics & Reporting Service](#analytics--reporting-service)
- [Infrastructure](#infrastructure)
  - [Cloud Architecture](#cloud-architecture)
  - [CI/CD Pipeline](#cicd-pipeline)
  - [Monitoring & Observability](#monitoring--observability)
  - [Disaster Recovery](#disaster-recovery)
- [User Flows](#user-flows)
  - [Diaspora Sender Onboarding](#1-diaspora-sender-onboarding)
  - [Beneficiary Onboarding](#2-beneficiary-onboarding)
  - [Send + Auto-Save](#3-send--auto-save)
  - [Savings Goal Creation](#4-savings-goal-creation)
  - [Beneficiary Withdrawal](#5-beneficiary-withdrawal)
  - [Yield Distribution](#6-yield-distribution)
- [Security](#security)
- [Regulatory & Compliance](#regulatory--compliance)
- [Development Setup](#development-setup)
- [Testing Strategy](#testing-strategy)
- [Roadmap](#roadmap)
- [Contributing](#contributing)
- [License](#license)

---

## Vision

> **Every remittance is also a savings deposit.**

The African diaspora sends over **$100B annually** back home, yet the average remittance recipient has no formal savings account. Meanwhile, diaspora users who want to save or invest at home face a painful round-trip: convert USD → send → convert back to local currency → save → convert again when needed. Each hop bleeds value in FX spreads and fees.

**RemitSave eliminates the round-trip FX cost entirely.** By letting diaspora users save directly into local-currency instruments (T-bills, fixed deposits, mutual funds) in their home country, remittance becomes the *funding mechanism* and savings becomes the *product*. A dollar sent from London can flow straight into a Nigerian T-bill pool without ever leaving the local currency — the FX conversion happens exactly once, on deposit, through Stellar's built-in DEX at market rates.

The platform removes the intention gap: instead of requiring a separate savings decision, every incoming transfer is *automatically split* — part for the family, part for the future. We turn the remittance pipeline into a wealth-building pipeline, directly on Stellar's low-cost, high-speed network, governed by Soroban smart contracts.

---

## Why This Matters

| Problem | How RemitSave Solves It |
|---|---|
| Remittance fees average 5–8% | Stellar transactions cost ~0.00001 XLM (< USD 0.000001) |
| Low savings penetration in Africa | Auto-save embedded into existing remittance habit |
| Informal savings (esusu/tontines) are risky | Non-custodial smart contracts with transparent rules |
| Diaspora wants to invest at home | Vault pools tokenize local assets (T-bills, MMFs) |
| FX double-conversion cost | Direct deposit into local-currency instruments removes the round-trip entirely — FX happens once via Stellar DEX at market rate, not twice at retail spreads |

---

## System Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                        USER LAYER                                │
│                                                                  │
│  ┌──────────────────────┐    ┌──────────────────────────────┐   │
│  │  Diaspora App         │    │  Beneficiary Interface        │   │
│  │  (React Native / Web) │    │  (Web + USSD + SMS)           │   │
│  └──────────┬───────────┘    └─────────────┬────────────────┘   │
└─────────────┼──────────────────────────────┼────────────────────┘
              │                              │
              │     HTTPS / WebSocket        │
              ▼                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                     API GATEWAY LAYER                            │
│                                                                  │
│  ┌────────────────────────────────────────────────────────────┐  │
│  │  Kong / Envoy API Gateway                                  │  │
│  │  • Rate limiting   • Auth (JWT + Stellar signatures)      │  │
│  │  • Request validation   • API versioning                  │  │
│  └──────────────────────────┬─────────────────────────────────┘  │
└─────────────────────────────┼───────────────────────────────────┘
                              │
              ┌───────────────┼───────────────┐
              ▼               ▼               ▼
┌────────────────────┐ ┌──────────────┐ ┌────────────────────┐
│  Backend Services  │ │  Events Bus  │ │  Off-Chain Agents  │
│  (Rust / Go)       │ │  (NATS /     │ │  (Cron Jobs,       │
│                    │ │   Kafka)     │ │   Relayers)        │
│  • Auth Service    │ │              │ │                    │
│  • User Service    │ │              │ │  • Schedule Keeper │
│  • Remit Service   │ │              │ │  • Anchor Relayer  │
│  • Savings Service │ │              │ │  • Oracle Updater  │
│  • Notification    │ │              │ │  • Yield Deployer  │
│  • Analytics       │ │              │ │                    │
└─────────┬──────────┘ └──────┬───────┘ └─────────┬──────────┘
          │                   │                   │
          └───────────────────┼───────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                    STELLAR LAYER                                 │
│                                                                  │
│  ┌────────────────────────────────────────────────────────────┐  │
│  │  Stellar Core (Full Nodes / Horizon RPC)                   │  │
│  │                                                            │  │
│  │  ┌─────────────────────────────────────────────────────┐   │  │
│  │  │  Soroban Smart Contracts                            │   │  │
│  │  │                                                     │   │  │
│  │  │  ┌──────────┐ ┌──────────┐ ┌───────────────────┐   │   │  │
│  │  │  │ Remit-   │ │ Savings  │ │ VaultPool         │   │   │  │
│  │  │  │ Save     │ │ Manager  │ │ (Yield Bearing)   │   │   │  │
│  │  │  │ Contract │ │ Contract │ │                   │   │   │  │
│  │  │  └──────────┘ └──────────┘ └───────────────────┘   │   │  │
│  │  └─────────────────────────────────────────────────────┘   │  │
│  │                                                            │  │
│  │  ┌─────────────────────────────────────────────────────┐   │  │
│  │  │  Anchors & Assets                                    │   │  │
│  │  │  • USDC (Circle)  • eNGN (Cowrie)  • eKES  • cEUR  │   │  │
│  │  │  • Vibrant Africa  • TEMPO Money Transfer           │   │  │
│  │  └─────────────────────────────────────────────────────┘   │  │
│  └────────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────┘
```

### Layer Breakdown

| Layer | Tech | Responsibility |
|---|---|---|
| **User Layer** | React Native (mobile), React (web), USSD gateway | Onboarding, sending, savings dashboard, notifications |
| **API Gateway** | Kong / Envoy | Auth, rate limiting, routing, TLS termination |
| **Backend Services** | Rust (Axum) / Go (Chi) | Business logic: user management, remittance orchestration, savings operations, FX quotes (from Stellar DEX, ensuring single-hop conversion) |
| **Events Bus** | NATS / Kafka | Async communication between services; reliable delivery of Stellar event webhooks |
| **Off-Chain Agents** | Rust / Go binaries (Dockerized) | Scheduled jobs: yield pool deployment, anchor reconciliation, oracle price feeds |
| **Stellar Layer** | Stellar Core + Soroban RPC | Transaction settlement, smart contract execution, asset issuance |
| **Data Layer** | PostgreSQL (primary), Redis (cache), S3 (audit logs) | Persistent storage for off-chain state, session cache, compliance archives |

---

## Smart Contracts (Soroban)

### Data Models

```rust
/// Core user identity — linked to Stellar public key
pub struct UserProfile {
    pub stellar_address: Address,
    pub country: Symbol,                // "NG", "KE", "GH", "UG", "RW", "ZA"
    pub phone: Bytes,                   // E.164 format
    pub kyc_level: u32,                 // 0 = unverified, 1 = basic, 2 = enhanced
    pub created_at: u64,
    pub last_active: u64,
}

/// A goal-based or open-ended savings plan
/// Denominated in a LOCAL stablecoin (e.g., eNGN, eKES) — NOT in USDC.
/// This is the core of the single-FX thesis: once USDC is converted to eNGN at deposit,
/// the savings live in local currency and never need another FX hop.
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

pub enum PlanStatus { Active, Completed, Withdrawn, Closed }

/// Mapping from sender address to their auto-split rule
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

pub enum SplitType { Percentage, Fixed }

/// A yield-bearing pool that aggregates user savings
/// Denominated in the local stablecoin (eNGN, eKES, etc.) — deposits from USDC
/// are converted via Stellar DEX path payment before entry, so the pool
/// only ever holds and yields in local currency.
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

pub enum YieldSource { TBILL, MoneyMarketFund, LendingPool, Staking }
```

### Contract Interface

```rust
// ====================================================================
//  RemitSave Contract (remit_save.wasm)
//  Deployed once per supported country/stablecoin pair
// ====================================================================

#[contractimpl]
impl RemitSave {

    // -- Admin --

    fn initialize(
        env: Env,
        admin: Address,
        fee_recipient: Address,
        protocol_fee_bps: u32,           // e.g., 50 = 0.50%
    );

    fn set_anchor(
        env: Env,
        admin: Address,
        asset: Address,
        anchor_address: Address,
    );

    fn upgrade(env: Env, admin: Address, new_wasm_hash: BytesN<32>);

    // -- User Registration --

    fn register_user(
        env: Env,
        user: Address,
        country: Symbol,
        phone: Bytes,
    ) -> Result<(), RemitError>;

    fn get_user(
        env: Env,
        user: Address,
    ) -> UserProfile;

    // -- Remittance Rules --

    fn set_remittance_rule(
        env: Env,
        sender: Address,
        rule: RemittanceRule,
    ) -> Result<u32, RemitError>;

    fn get_remittance_rule(
        env: Env,
        sender: Address,
        rule_id: u32,
    ) -> RemittanceRule;

    fn list_remittance_rules(
        env: Env,
        sender: Address,
    ) -> Vec<RemittanceRule>;

    fn deactivate_remittance_rule(
        env: Env,
        sender: Address,
        rule_id: u32,
    );

    /// Execute a remittance with auto-split
    /// - `sender` pays `total_amount` in `incoming_asset` (e.g., USDC)
    /// - The rule determines how much goes to the beneficiary vs savings
    /// - The beneficiary portion is sent via path payment → Stellar DEX converts to
    ///   local stablecoin → anchor disburses to mobile money (one FX hop)
    /// - The savings portion is ALSO converted to `local_asset` via Stellar DEX path
    ///   payment before being deposited into the savings plan. This is the key:
    ///   **the savings are denominated in local currency from the moment of deposit**,
    ///   eliminating any future FX round-trip cost.
    fn execute_remittance(
        env: Env,
        sender: Address,
        rule_id: u32,
        total_amount: i128,
        incoming_asset: Address,
    ) -> Result<RemittanceResult, RemitError>;

    // -- Savings Plans --

    /// Create a savings plan denominated in `local_asset` (e.g., eNGN, eKES).
    /// The plan only accepts deposits in its local currency — if auto-save routes
    /// USDC here, the contract converts it via Stellar DEX before crediting.
    fn create_savings_plan(
        env: Env,
        owner: Address,
        goal_name: Symbol,
        target_amount: i128,            // in local_asset units
        local_asset: Address,           // e.g., eNGN, eKES — never USDC
        lock_until: Option<u64>,
    ) -> Result<u32, RemitError>;

    fn get_savings_plan(
        env: Env,
        owner: Address,
        plan_id: u32,
    ) -> SavingsPlan;

    fn list_savings_plans(
        env: Env,
        owner: Address,
    ) -> Vec<SavingsPlan>;

    /// Deposit additional funds into a savings plan manually.
    /// If `asset != plan.local_asset`, the contract converts via DEX path payment
    /// so the plan balance is always in local currency. One FX hop, done.
    fn deposit_to_plan(
        env: Env,
        owner: Address,
        plan_id: u32,
        amount: i128,
        asset: Address,                 // USDC or local_asset — conversion handled automatically
    );

    /// Withdraw from a savings plan (fails if timelocked)
    fn withdraw_from_plan(
        env: Env,
        owner: Address,
        plan_id: u32,
    ) -> Result<i128, RemitError>;

    /// Close a plan (withdraw remaining balance, archive it)
    fn close_savings_plan(
        env: Env,
        owner: Address,
        plan_id: u32,
    ) -> Result<i128, RemitError>;

    // -- Vault Pools (Yield) --

    fn create_vault_pool(
        env: Env,
        admin: Address,
        local_asset: Address,           // e.g., eNGN, eKES — pool only deals in local currency
        name: Symbol,
        yield_source: YieldSource,
        min_lockup: u64,
    ) -> Result<u32, RemitError>;

    /// Deposit into a yield pool.
    /// - If `asset != pool.local_asset`, the contract executes a Stellar DEX path payment
    ///   to convert `amount` of `asset` into `pool.local_asset` before minting shares.
    /// - This is the single FX hop that anchors the thesis: deposits that arrive in
    ///   USDC are converted to local currency (eNGN, eKES, etc.) *once*, and the
    ///   resulting shares are denominated in local currency. No second conversion needed.
    fn vault_deposit(
        env: Env,
        user: Address,
        pool_id: u32,
        amount: i128,
        asset: Address,                 // may be USDC or local_asset
    ) -> Result<i128, RemitError>;      // returns shares minted in local_asset denomination

    /// Withdraw from yield pool.
    /// Always returns `pool.local_asset` — no FX conversion.
    /// The beneficiary can take this directly to an anchor for fiat payout.
    fn vault_withdraw(
        env: Env,
        user: Address,
        pool_id: u32,
        share_amount: i128,
    ) -> Result<i128, RemitError>;      // returns local_asset amount

    fn get_vault_pool(
        env: Env,
        pool_id: u32,
    ) -> VaultPool;

    fn get_user_vault_balance(
        env: Env,
        user: Address,
        pool_id: u32,
    ) -> i128;                       // shares owned

    /// Admin-only: update share price after yield is realized
    fn update_share_price(
        env: Env,
        admin: Address,
        pool_id: u32,
        new_share_price: i128,
    );
}

// ====================================================================
//  Events emitted by the contract
// ====================================================================

/// Fired when a remittance is executed and split
#[contractevent]
pub struct RemittanceExecuted {
    pub remittance_id: BytesN<32>,
    pub sender: Address,
    pub beneficiary: Address,
    pub total_amount: i128,             // in incoming_asset (e.g., USDC)
    pub payout_amount: i128,            // in local_asset (already converted)
    pub savings_amount: i128,           // in local_asset (already converted — no round-trip)
    pub incoming_asset: Address,        // what the sender sent (e.g., USDC)
    pub local_asset: Address,           // what savings & payout are in (e.g., eNGN)
    pub timestamp: u64,
}

/// Fired when a savings plan is created
#[contractevent]
pub struct SavingsPlanCreated {
    pub owner: Address,
    pub plan_id: u32,
    pub goal_name: Symbol,
    pub target_amount: i128,
}

/// Fired when savings yield is distributed
#[contractevent]
pub struct YieldDistributed {
    pub pool_id: u32,
    pub total_yield: i128,
    pub new_share_price: i128,
    pub timestamp: u64,
}
```

### Key Flows

#### Split & Route

```
Sender sends 100 USDC
         │
         ▼
execute_remittance(rule_id=1, total_amount=100_0000000, incoming_asset=USDC)
         │
         ├── Load RemittanceRule(1) → split_value=7000 (70%),
         │     savings_plan_id=2, incoming_asset=USDC, local_asset=eNGN
         │
         ├── Calculate (in incoming_asset):
         │     payout    = 70 USDC
         │     savings   = 30 USDC
         │     fee       =  0.50 USDC (protocol fee, 50 bps)
         │
         ├── Stellar DEX path payment: 70 USDC → eNGN ────────────────┐
         │     (single FX hop at market rate)                         │
         │                                                            ▼
         ├── Transfer 70 USDC worth of eNGN → beneficiary's address
         │     → anchor picks it up → mobile money payout in NGN
         │
         ├── Stellar DEX path payment: 30 USDC → eNGN ────────────────┐
         │     (single FX hop — savings now LIVE IN LOCAL CURRENCY)   │
         │                                                            ▼
         ├── Deposit 30 USDC worth of eNGN → SavingsPlan(2)
         │     (balance updated in eNGN — no future FX needed)
         │
         ├── Transfer fee (0.50 USDC) → fee_recipient
         │
         └── Emit RemittanceExecuted{
                incoming_asset: USDC,
                local_asset: eNGN,
                payout_amount: 70_USDC_value_in_eNGN,
                savings_amount: 30_USDC_value_in_eNGN
             }
```

#### Vault Yield (denominated in local currency)

```
Admin detects yield earned on Nigerian T-bill pool (off-chain, via partner)
         │
         ▼
update_share_price(pool_id=1, new_share_price=1_0250000)  // 2.5% return
         │
         ├── Pool asset: eNGN (Nigeria Naira stablecoin)
         ├── Total deposits: 1,000,000,000 eNGN (~$625,000)
         ├── Total shares:     975,610,000 eNGN shares (at old price 1.025)
         ├── New share price: 1.05 → total value = 1,024,390,000 eNGN
         │
         ├── Note: users originally deposited USDC, but the contract
         │   converted to eNGN at entry. The yield is earned and
         │   reported in eNGN. Withdrawals also return eNGN → anchor
         │   pays out NGN directly. NO USD CONVERSION NEEDED.
         │
         └── Emit YieldDistributed{pool_id=1, total_yield=24_390_000 eNGN, ...}
```

---

## Frontend

### User App (Diaspora Sender)

**Tech stack:** React Native (Expo) + TypeScript

| Screen | Features |
|---|---|
| **Onboarding** | Email/phone auth, Stellar wallet creation (or import via secret key/seed phrase), biometric auth, KYC document upload |
| **Dashboard** | Balance overview (USDC + local stablecoins), recent remittances, savings progress bars, yield earned-to-date, currency conversion rates |
| **Send Money** | Select beneficiary, enter amount, choose payout method (mobile money / bank / cash pickup), set or confirm auto-save % — one tap to send |
| **Savings Goals** | List of all plans with progress rings, target amount, days remaining; tap into detail for transaction history, early withdrawal option (if unlocked) |
| **Create Goal** | Name the goal, set target, choose stablecoin, optionally set auto-save percentage from future remittances, optionally set lock-up period |
| **Vault Pools** | Browse available yield pools (T-Bill, MMF, etc.), APY displayed, deposit/withdraw UI, historical yield chart |
| **Activity / History** | Filterable list of all transactions (sent, received, savings deposits, yield, withdrawals) |
| **Profile / Settings** | Beneficiary management, notification preferences, security settings (2FA, connected wallets), language selection (English, French, Portuguese, Swahili, Hausa, Yoruba, Igbo) |

### Beneficiary App (Web + USSD)

**Tech stack:** React (PWA) + Africa's Talking USSD API

Since many beneficiaries have feature phones or limited data, the primary interface is **USSD** with a progressive web app as the richer option.

| Channel | Features |
|---|---|
| **USSD** (`*347*XX#`) | Check balance, request withdrawal, view last 5 transactions, change preferred payout method (mobile money / bank / cash), get help in local language |
| **PWA** (lightweight) | Same as diaspora app but read-only on savings goals, simpler dashboard with local currency view, transaction history, download receipts |
| **SMS alerts** | Real-time notification: "You received 25,000 NGN from Chisom 🎉 7,500 NGN saved to School Fees goal" |

### Shared Component Library

All frontend apps share a component library (`@remitsave/ui`) built with:

- **Design system**: Custom design tokens (colors, typography, spacing) inspired by African fintech brands — warm, trustworthy, vibrant
- **Component catalog**: Button, Input, Card, ProgressRing, TransactionRow, BeneficiaryPicker, CurrencyInput, PinInput, Modal, Toast, Skeleton, PullToRefresh
- **i18n**: `react-i18next` with ICU MessageFormat; translations managed via Crowdin
- **Offline-first**: Service worker + IndexedDB for transaction history and balance caching; optimistic UI updates with sync queue

---

## Backend / Off-Chain Services

### Architecture

All backend services are written in **Rust** (using `axum` framework) for maximum performance and safety, with a single **Go** service for the USSD gateway (better SMS/telecom libraries).

```
                         ┌──────────────────┐
                         │   NATS / Kafka   │
                         │  (Event Stream)   │
                         └──────┬───────────┘
                                │
        ┌───────────────────────┼───────────────────────┐
        │                       │                       │
        ▼                       ▼                       ▼
┌───────────────┐     ┌─────────────────┐     ┌─────────────────┐
│  Auth Service  │     │  Remit Service  │     │  Savings Service │
│  (Rust/Axum)   │     │  (Rust/Axum)    │     │  (Rust/Axum)     │
└───────┬───────┘     └────────┬────────┘     └────────┬────────┘
        │                      │                       │
        ▼                      ▼                       ▼
┌─────────────────────────────────────────────────────────────┐
│                    PostgreSQL (RDS)                          │
│  users  │  beneficiaries  │  transactions  │  savings_plans  │
│  vault_pools  │  audit_log  │  kyc_documents               │
└─────────────────────────────────────────────────────────────┘
        ▲                      ▲                       ▲
        │                      │                       │
┌───────┴───────┐     ┌────────┴────────┐     ┌───────┴────────┐
│  Anchor        │     │  Schedule       │     │  Notification  │
│  Relayer       │     │  Keeper         │     │  Service       │
│  (Rust)        │     │  (Rust)         │     │  (Rust)        │
└───────────────┘     └─────────────────┘     └────────────────┘
```

### Service Specifications

#### Auth Service
- **Routes**: `POST /auth/register`, `POST /auth/login`, `POST /auth/refresh`, `POST /auth/kyc`, `GET /auth/me`
- **Responsibilities**: JWT issuance and validation, Stellar `verify_signed_payload` authentication, KYC document collection & verification (with integrations to IdentityPass / YouVerify / SmileID), 2FA (TOTP / SMS)
- **Data**: `users` table, `kyc_documents` table, `sessions` table

#### Remit Service
- **Routes**: `POST /remit/rule`, `GET /remit/rules`, `PUT /remit/rule/{id}`, `POST /remit/execute`, `GET /remit/history`
- **Responsibilities**: CRUD for remittance rules, fee calculation, FX quote caching (from Stellar DEX), submission of `execute_remittance` Soroban calls, retry logic for failed submissions
- **Data**: `remittance_rules` table, `remittance_tx` table, `fx_quotes` cache (Redis)

#### Savings Service
- **Routes**: `POST /savings/plan`, `GET /savings/plans`, `POST /savings/plan/{id}/deposit`, `POST /savings/plan/{id}/withdraw`, `GET /vault/pools`, `POST /vault/deposit`, `POST /vault/withdraw`
- **Responsibilities**: Savings plan management, vault pool operations, yield calculation and distribution coordination
- **Data**: `savings_plans` table, `vault_pools` table, `user_vault_balances` table, `yield_events` table

#### Notification Service
- **Tech**: Rust + Firebase Cloud Messaging + Africa's Talking SMS + SendGrid (email)
- **Responsibilities**: Consume events from NATS/Kafka → build localized message → deliver via appropriate channel (push, SMS, email)
- **Templates**: Handlebars-based with ICU message formatting; stored in S3 and cached locally

#### Anchor Relayer Service
- **Tech**: Rust with `stellar-sdk` for Stellar transaction submission
- **Responsibilities**: Monitor incoming Soroban `RemittanceExecuted` events, detect beneficiary payouts, submit Stellar payments to anchor integration accounts, reconcile anchor payout confirmations (via webhook or Stellar account watcher)
- **Anchor integrations**:
  - **Cowrie** (NGN) — SEP-24 deposit/withdrawal
  - **Vibrant Africa** (KES, UGX, RWF) — SEP-6 deposit/withdrawal
  - **TEMPO** (EUR, XOF, XAF) — SEP-24
  - **Flutterwave** (multi-currency, multi-channel payout) — REST API

#### Schedule Keeper
- **Tech**: Rust binary deployed as a Kubernetes CronJob
- **Jobs**:
  - `update_fx_rates` — every 5 minutes, polls Stellar DEX for latest path payment quotes, caches in Redis
  - `rebalance_vault` — every 6 hours, checks vault pool composition, submits `update_share_price` if yield has been realized
  - `expire_lockups` — every hour, checks for savings plans whose `lock_until` has passed and marks them as withdrawable
  - `anchor_reconciliation` — every 24 hours, compares on-chain transfer amounts to anchor payout confirmations, flags discrepancies
  - `daily_summary` — every 24 hours, generates aggregate statistics (volume, active users, savings rate)

---

## Infrastructure

### Cloud Architecture

Deployed on **AWS** with multi-region support (primary: `eu-west-1`, disaster recovery: `af-south-1` or `eu-central-1`).

```
┌─────────────────────────────────────────────────────────────────┐
│                         Route 53                                 │
│                      app.remitsave.africa                        │
└──────────────────────────┬──────────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────────────┐
│                    CloudFront (CDN)                              │
│           /api/* → Application Load Balancer                     │
│           /*     → S3 (static assets)                            │
└─────────────────────────────────────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────────────┐
│                   WAF (Web Application Firewall)                 │
│          Rate limiting, SQL injection, XSS, bot control          │
└─────────────────────────────────────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────────────┐
│              Application Load Balancer (ALB)                     │
│               TLS termination, path-based routing                │
└──────────┬──────────────────┬──────────────────┬────────────────┘
           │                  │                  │
           ▼                  ▼                  ▼
┌──────────────────┐ ┌──────────────────┐ ┌──────────────────┐
│  EKS Cluster      │ │  ElastiCache     │ │  RDS Aurora      │
│  (Fargate)        │ │  (Redis)         │ │  PostgreSQL      │
│                   │ │                  │ │                  │
│  • Auth Pod       │ │  • Session cache │ │  • Primary DB    │
│  • Remit Pod      │ │  • FX quote cache│ │  • Read replicas │
│  • Savings Pod    │ │  • Rate limiter  │ │  • Point-in-time │
│  • Notifications  │ │  • Job queue     │ │    recovery      │
│  • Schedule Jobs  │ │                  │ │                  │
│  • Anchor Relayer │ │                  │ │                  │
└──────────────────┘ └──────────────────┘ └──────────────────┘
```

| Component | Service | Details |
|---|---|---|
| **Compute** | EKS (Fargate) | Auto-scaling Kubernetes, no node management |
| **Database** | RDS Aurora PostgreSQL | Multi-AZ, read replicas, automated backups (30-day retention) |
| **Cache** | ElastiCache Redis | 6+ node cluster, encryption in-transit/at-rest |
| **Storage** | S3 | Static assets, KYC documents (encrypted), audit logs |
| **CDN** | CloudFront | Global edge caching for static assets and API responses |
| **WAF** | AWS WAF | Rate limiting, IP reputation lists, SQLi/XSS rules |
| **DNS** | Route 53 | Latency-based routing, health checks, failover |
| **Secrets** | AWS Secrets Manager | API keys, anchor credentials, database passwords |
| **CI/CD** | GitHub Actions | Build, test, deploy pipeline (see below) |
| **Monitoring** | Datadog / Grafana | APM, metrics, logs, traces, dashboarding |

### CI/CD Pipeline

```
GitHub → main branch
   │
   ▼
┌──────────────────────┐
│  GitHub Actions       │
│                       │
│  Stage: Lint & Test   │
│  ├─ cargo fmt --check │
│  ├─ cargo clippy      │
│  ├─ cargo test        │
│  ├─ npm run lint      │
│  ├─ npm run test      │
│  └─ soroban contract  │
│     build + test      │
│                       │
│  Stage: Build         │
│  ├─ docker build .    │
│  ├─ docker push to    │
│  │  ECR               │
│  └─ Upload frontend   │
│     to S3             │
│                       │
│  Stage: Deploy Staging│
│  ├─ helm upgrade      │
│  │  staging-cluster   │
│  ├─ Run smoke tests   │
│  └─ Run integration   │
│     tests             │
│                       │
│  Stage: Deploy Prod   │
│  ├─ helm upgrade      │
│  │  prod-cluster      │
│  ├─ Canary deploy     │
│  │  (10% traffic)     │
│  ├─ Monitor metrics   │
│  └─ Roll forward to   │
│     100%              │
└──────────────────────┘
```

### Monitoring & Observability

| Tool | Purpose |
|---|---|
| **Datadog APM** | Distributed tracing across all services; P50/P95/P99 latency tracking; error budget alerts |
| **Grafana** | Custom dashboards: daily active users, remittance volume, savings rate, vault TVL, anchor payout success rates |
| **Prometheus** | Metrics collection: Stellar transaction submission success, Soroban execution gas, Horizon RPC latency, queue depths |
| **Loki** | Centralized log aggregation; structured JSON logging (service, level, trace_id, message); 30-day retention |
| **PagerDuty** | On-call rotation; alert routing based on severity (P0: downtime/outage, P1: degraded, P2: warning) |
| **Sentry** | Frontend error tracking; JavaScript stack traces with user context and breadcrumbs |

### Disaster Recovery

| Scenario | RTO | RPO | Strategy |
|---|---|---|---|
| Single pod crash | < 30s | 0 | Kubernetes auto-restart, health check probes |
| AZ outage | < 5 min | 0 | Multi-AZ deployment; Aurora Multi-AZ failover |
| Region outage | < 30 min | < 5 min | Cross-region DB replication (PostgreSQL logical replication); EKS in DR region with warmed infrastructure via Terraform |
| Data corruption | < 4 hr | < 5 min | PITR from RDS automated snapshots (30-day window) |
| Stellar network halt | N/A | N/A | Queue transactions, retry with exponential backoff; manual override to offline-mode (I.O.U. credits) |

---

## User Flows

### 1. Diaspora Sender Onboarding

```
  App                      Backend                       Stellar                    Anchor
   │                         │                              │                        │
   │-- 1. Enter phone/email -->│                             │                        │
   │                         │                              │                        │
   │<-- 2. Send OTP ---------│                             │                        │
   │                         │                              │                        │
   │-- 3. Verify OTP -------->│                             │                        │
   │                         │-- 4. Create wallet ---------->│                       │
   │                         │   (register_user)            │  (via wallet SDK)      │
   │                         │<-- wallet address -----------│                        │
   │                         │                              │                        │
   │<-- 5. Save mnemonic ---│                             │                        │
   │                         │                              │                        │
   │-- 6. Submit KYC --------│                             │                        │
   │   (ID document + selfie)│                             │                        │
   │                         │-- 7. Verify KYC ------------>│                       │
   │                         │   (IdentityPass / SmileID)   │                        │
   │<-- 8. KYC approved -----│                             │                        │
   │                         │                              │                        │
   │-- 9. Fund wallet -------│                             │                        │
   │   (buy USDC with card)  │                             │                        │
   │                         │-- 10. Mint USDC ------------>│                       │
   │                         │   (via Circle API / anchor)  │                        │
   │<-- 11. Balance updated -│                             │                        │
```

### 2. Beneficiary Onboarding

```
 SMS/App                    Backend                      Anchor
   │                         │                              │
   │-- 1. Sender adds --------->│                             │
   │   beneficiary (name,       │                             │
   │   phone, country)          │                             │
   │                         │                             │
   │<-- 2. SMS to beneficiary--│                             │
   │   "Chisom wants to send   │                             │
   │   to you! Click to        │                             │
   │   register: ..."          │                             │
   │                         │                             │
   │-- 3. Follows link / ------>│                             │
   │   dials USSD code          │                             │
   │                         │                             │
   │<-- 4. Choose payout ------│                             │
   │   method (MTN MoMo /       │                             │
   │   Airtel Money / Bank)     │                             │
   │                         │                             │
   │-- 5. Confirm + submit ---->│                             │
   │                         │-- 6. Create anchor -------->│
   │                         │   payout configuration       │
   │                         │<-- anchor reference --------│
   │                         │                             │
   │<-- 7. "You're all set!" -│                             │
   │   "Your first deposit     │                             │
   │   from Chisom will        │                             │
   │   arrive in minutes"      │                             │
```

### 3. Send + Auto-Save (Single FX Hop)

```
 Sender                  RemitSave Contract            Stellar DEX         Anchor          Beneficiary
   │                         │                              │                  │                  │
   │-- 1. Tap "Send" -------->│                             │                  │                  │
   │   100 USDC, rule_id=1    │                              │                  │                  │
   │                         │                              │                  │                  │
   │-- 2. execute_remittance->│                             │                  │                  │
   │   (sender, rule_id,      │                              │                  │                  │
   │    100 USDC)             │                              │                  │                  │
   │                         │                              │                  │                  │
   │                         │-- 3. Load rule: -------------│                 │                  │
   │                         │   split: 70/30               │                  │                  │
   │                         │   local_asset: eNGN          │                  │                  │
   │                         │   savings_plan: 2 (eNGN)     │                  │                  │
   │                         │                              │                  │                  │
   │                         │                              │                  │                  │
   │                         │-- 4. Path payment: ----------│                 │                  │
   │                         │   70 USDC → eNGN via DEX     │                  │                  │
   │                         │   (single FX hop, market     │                  │                  │
   │                         │    rate, no markup)          │                  │                  │
   │                         │                              │                  │                  │
   │                         │<---- 70 USDC worth of eNGN --│                 │                  │
   │                         │                              │                  │                  │
   │                         │-- 5. Transfer eNGN ---------->│                 │                  │
   │                         │   → beneficiary address       │                  │                  │
   │                         │                              │                  │                  │
   │                         │                              │                  │-- 6. Payout ----->│
   │                         │                              │                  │   70,000 NGN     │
   │                         │                              │                  │   to MTN MoMo    │
   │                         │                              │                  │                  │
   │                         │-- 7. Path payment: ----------│                 │                  │
   │                         │   30 USDC → eNGN via DEX     │                  │                  │
   │                         │   (single FX hop — savings   │                  │                  │
   │                         │    NOW IN LOCAL CURRENCY)    │                  │                  │
   │                         │                              │                  │                  │
   │                         │<---- 30 USDC worth of eNGN --│                 │                  │
   │                         │                              │                  │                  │
   │                         │-- 8. Deposit eNGN ----------->│                 │                  │
   │                         │   → SavingsPlan(2)            │                  │                  │
   │                         │   (balance in eNGN — no      │                  │                  │
   │                         │    future FX conversion       │                  │                  │
   │                         │    ever needed)               │                  │                  │
   │                         │                              │                  │                  │
   │                         │-- 9. Transfer 0.5 USDC ----->│                 │                  │
   │                         │   → fee_recipient             │                  │                  │
   │                         │                              │                  │                  │
   │                         │-- 10. Emit event ------------│                 │                  │
   │                         │   RemittanceExecuted{         │                  │                  │
   │                         │    incoming: USDC,            │                  │                  │
   │                         │    local: eNGN,               │                  │                  │
   │                         │    payout: 70_USDC_in_eNGN,   │                  │                  │
   │                         │    savings: 30_USDC_in_eNGN}  │                  │                  │
   │                         │                              │                  │                  │
   │<-- 11. "Sent! ---------│                             │                  │                  │
   │   70 USDC → 70K NGN    │                              │                  │                  │
   │   sent to Mom,          │                              │                  │                  │
   │   30 USDC → ~30K NGN    │                              │                  │                  │
   │   saved in eNGN 🎉"     │                              │                  │                  │
   │                         │                              │                  │                  │
   │                         │                              │                  │<-- 12. SMS -------│
   │                         │                              │                  │   "You received  │
   │                         │                              │                  │    70,000 NGN!   │
   │                         │                              │                  │    30,000 NGN    │
   │                         │                              │                  │    saved to      │
   │                         │                              │                  │    School Fees   │
   │                         │                              │                  │    (eNGN)"       │
```

### 4. Savings Goal Creation

```
 Sender                Savings Service              Soroban Contract
   │                         │                              │
   │-- 1. Open "Create Goal"->│                             │
   │                         │                              │
   │<-- 2. Form -------------│                             │
   │   Goal name: School Fees│                              │
   │   Target: 500,000 NGN   │                              │
   │   Currency: eNGN        │                              │
   │   Lock until: 2027-06-01│                              │
   │   Auto-save: 30%        │                              │
   │                         │                              │
   │-- 3. Submit ------------>│                             │
   │                         │-- 4. create_savings_plan --->│
   │                         │   (owner, "School Fees",     │
   │                         │    500_000_00000, eNGN,      │
   │                         │    lock_until)               │
   │                         │<-- plan_id: 2 ---------------│
   │                         │                              │
    │                         │-- 5. create_remittance_rule->│
    │                         │   (sender, {beneficiary,     │
    │                         │    incoming_asset: USDC,     │
    │                         │    local_asset: eNGN,        │
    │                         │    split: 7000,              │
    │                         │    savings_plan_id: 2})      │
   │                         │<-- rule_id: 1 ---------------│
   │                         │                              │
   │<-- 6. "Goal created! --│                             │
   │   Auto-save activated.  │                              │
   │   You're 0% toward      │                              │
   │   500,000 NGN"          │                              │
```

### 5. Beneficiary Withdrawal

```
 Beneficiary              USSD Gateway              Savings Service          Anchor
   │                         │                              │                  │
   │-- Dial *347*1# --------->│                             │                  │
   │                         │                              │                  │
   │<-- "Select account: ----│                             │                  │
   │    1. School Fees       │                              │                  │
   │       45,000 NGN        │                              │                  │
   │    2. Emergency Fund    │                              │                  │
   │       12,000 NGN"       │                              │                  │
   │                         │                              │                  │
   │-- "1" ------------------>│                             │                  │
   │                         │                              │                  │
   │<-- "School Fees --------│                             │                  │
   │    45,000 NGN           │                              │                  │
   │    Locked until:        │                              │                  │
   │    2027-06-01           │                              │                  │
   │    You can't withdraw   │                              │                  │
   │    yet. Press 0 for     │                              │                  │
   │    main menu"           │                              │                  │
   │                         │                              │                  │
   │-- *347*2# (Emergency)--->│                             │                  │
   │                         │                              │                  │
   │<-- "Emergency Fund -----│                             │                  │
   │    12,000 NGN           │                              │                  │
   │    No lock. Withdraw    │                              │                  │
   │    all? 1. Yes 2. No"   │                              │                  │
   │                         │                              │                  │
   │-- "1" ------------------>│                             │                  │
   │                         │-- withdraw_from_plan ------->│                  │
   │                         │   (beneficiary, plan_id=5)   │                  │
   │                         │<-- 12_000_00000 eNGN -------│                  │
   │                         │                              │                  │
   │                         │-- Transfer 12,000 eNGN ----->│                 │
   │                         │   → anchor payout address    │                  │
   │                         │                              │-- Payout ------->│
   │                         │                              │   12,000 NGN     │
   │                         │                              │   to Mom's MoMo  │
   │                         │                              │                  │
   │<-- "12,000 NGN sent ----│                             │                  │
   │    to your MTN MoMo!    │                              │                  │
   │    Thank you for        │                              │                  │
   │    saving 🙌"           │                              │                  │
```

### 6. Yield Distribution (Local-Currency Denominated)

```
 Schedule Keeper              Vault Contract (eNGN)      Anchor (Nigeria)      User
   │                              │                        │                    │
   │-- Every 6 hours              │                        │                    │
   │                              │                        │                    │
   │-- 1. Check vault pool ------>│                       │                    │
   │   Nigerian T-Bill Pool       │                        │                    │
   │   Pool asset: eNGN           │                        │                    │
   │   TVL: 1,000,000,000 eNGN    │                        │                    │
   │   ≈ $625,000 @ current rate  │                        │                    │
   │   APY target: 12%            │                        │                    │
   │                              │                        │                    │
   │   NOTE: Users deposited USDC │                        │                    │
   │   but the contract converted │                        │                    │
   │   to eNGN at entry. The      │                        │                    │
   │   pool ONLY holds eNGN.      │                        │                    │
   │   No USD exposure.           │                        │                    │
   │                              │                        │                    │
   │                              │-- 2. Query yield ----->│                   │
   │                              │   realized this period  │                    │
   │                              │<-- 24,390,000 eNGN ----│                   │
   │                              │   (2.44% this period)  │                    │
   │                              │                        │                    │
   │-- 3. update_share_price ---->│                       │                    │
   │   (pool_id=1, local_asset,   │                        │                    │
   │    new_share_price=1_0250000)│                        │                    │
   │                              │                        │                    │
   │                              │-- 4. Emit event ------->│                   │
   │                              │   YieldDistributed{     │                    │
   │                              │    pool_id: 1,          │                    │
   │                              │    local_asset: eNGN,   │                    │
   │                              │    total_yield:         │                    │
   │                              │     24,390,000 eNGN,    │                    │
   │                              │    new_share_price:     │                    │
   │                              │     1.025}              │                    │
   │                              │                        │                    │
   │<-- 5. Log + notify ---------│                       │                    │
   │   Push notification:         │                        │                    │
   │   "Your T-Bill pool earned   │                        │                    │
   │    2.44% (= 24,390 eNGN)     │                        │                    │
   │    this period! 🎉"          │                        │                    │
   │                              │                        │                    │
   │   Withdrawals return eNGN    │                        │                    │
   │   → anchor pays NGN.        │                        │                    │
   │   NO USD CONVERSION NEEDED.  │                        │                    │
```

---

## Security

### Smart Contract Security

| Measure | Implementation |
|---|---|
| **Access control** | `admin` address pattern for privileged operations; two-phase ownership transfer (propose + accept) |
| **Reentrancy protection** | Soroban is single-threaded per contract call — no reentrancy by default. Still, follow checks-effects-interactions pattern in `withdraw_from_plan` and `vault_withdraw` |
| **Integer overflow** | Soroban's `i128` has built-in overflow protection (panics on overflow). Always validate before arithmetic. |
| **Flash loan prevention** | Minimum holding period (1 epoch) before `vault_withdraw` is allowed |
| **Pausability** | Emergency pause mechanism: admin can halt deposits/withdrawals in case of vulnerability disclosure |
| **Formal verification** | Use `soroban-spec` + SMT solver to verify invariant properties: share price never decreases (except by admin action), total shares × share_price == total_deposits |
| **Audits** | All contracts audited by **OpenZeppelin** or **Halborn** before mainnet deployment; bug bounty program via Immunefi |

### Platform Security

| Layer | Measures |
|---|---|
| **API** | JWT with short expiry (15 min) + refresh tokens (7 days); Stellar signature verification for sensitive operations; rate limiting per user/IP; request signing |
| **Database** | Encrypted at rest (AES-256); column-level encryption for PII (phone, email, KYC data); IAM-based access (no hardcoded credentials); automated backup encryption |
| **Secrets** | AWS Secrets Manager with automatic rotation; never logged or exposed in environment variables in runtime; audit trail for all secret access |
| **Network** | All traffic over TLS 1.3; private subnets for all services (no public IPs); VPC with security groups; WAF in front of ALB |
| **Frontend** | CSP headers; subresource integrity; no sensitive data in localStorage; secure WebSocket connections; biometric auth for mobile |
| **Stellar** | Multi-sig for admin operations; time-bounded transactions; sequence number management to prevent replays |

### KYC & AML

1. **Tier 1 (send < $500/day):** Phone verification + selfie
2. **Tier 2 (send < $5,000/day):** Government ID + proof of address
3. **Tier 3 (unlimited):** Enhanced due diligence (source of funds, occupation, PEP check)

Integrated with:
- **IdentityPass** (Nigeria, Ghana, Kenya — BVN, NIN, national ID)
- **SmileID** (pan-African document verification + liveness check)
- **Chainalysis** (blockchain transaction monitoring for AML)
- **Trulioo** (global identity verification)

---

## Regulatory & Compliance

### Licensing Strategy (per country)

| Country | License Required | Status |
|---|---|---|
| **Nigeria** | PSB (Payment Service Bank) or CBN Remittance License + SEC (for savings/investment) | Target Q1 2027 |
| **Kenya** | CBR (Community Based Remittance) or PSP License + CMA (Capital Markets) | Target Q2 2027 |
| **Ghana** | Enhanced Payment Service Provider License + SEC license | Target Q2 2027 |
| **South Africa** | FSCA Category II (Remittance) + Category IIA (Savings) | Target Q3 2027 |
| **Uganda** | Remittance Operator License (Bank of Uganda) | Target Q3 2027 |
| **Rwanda** | Payment Service Provider License (BNR) | Target Q4 2027 |

### Data Residency

- User data stored in-region where possible
- AWS `af-south-1` (Cape Town) for Southern Africa
- AWS `eu-west-1` (Ireland) for diaspora users in Europe
- AWS `us-east-1` (N. Virginia) for Americas diaspora
- Cross-region transfers encrypted and logged for GDPR/DPA compliance

### Key Compliance Frameworks

- **GDPR**: User data deletion right, consent tracking, DPIA completed
- **PCI DSS**: For card-funded remittances (handled via Circle/Stripe — no raw card data stored)
- **FATF Travel Rule**: Beneficiary information sharing for transfers > $1,000
- **Local data protection**: NDPR (Nigeria), DPA (Kenya), POPIA (South Africa)

---

## Development Setup

See [CONTRIBUTING.md](CONTRIBUTING.md) for a full guide. Quick start:

```bash
bash scripts/bootstrap.sh          # one-command setup
make test                          # run all tests
```

### Repository Structure

```
remit-save-africa/
├── contracts/                 # Soroban smart contracts
│   ├── remit-save/           # Core remittance + savings contract
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── storage.rs
│   │   │   ├── events.rs
│   │   │   ├── test.rs
│   │   │   └── interfaces/
│   │   │       ├── treaty.rs
│   │   │       └── stellar_asset.rs
│   │   └── Makefile
│   ├── vault-pool/           # Yield-bearing vault pool contract
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── storage.rs
│   │   │   ├── events.rs
│   │   │   ├── math.rs      # Share price calculation
│   │   │   └── test.rs
│   │   └── Makefile
│   └── shared/              # Shared contract types and utilities
│       ├── Cargo.toml
│       └── src/
│           ├── lib.rs
│           ├── types.rs
│           └── auth.rs
│
├── backend/                  # Off-chain services (Rust)
│   ├── auth-service/
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── main.rs
│   │   │   ├── routes/
│   │   │   ├── models/
│   │   │   ├── middleware/
│   │   │   └── providers/   # KYC providers, SMS providers
│   │   ├── migrations/
│   │   └── Dockerfile
│   ├── remit-service/
│   ├── savings-service/
│   ├── notification-service/
│   ├── anchor-relayer/
│   ├── schedule-keeper/
│   └── shared/              # Shared backend crate
│       ├── Cargo.toml
│       └── src/
│           ├── stellar_rpc.rs
│           ├── error.rs
│           ├── config.rs
│           ├── db.rs
│           ├── queue.rs
│           └── utils.rs
│
├── frontend/                 # Frontend apps
│   ├── diaspora-app/        # React Native (Expo) — sender app
│   │   ├── App.tsx
│   │   ├── src/
│   │   │   ├── screens/
│   │   │   ├── components/
│   │   │   ├── hooks/
│   │   │   ├── services/
│   │   │   ├── navigation/
│   │   │   └── i18n/
│   │   ├── app.json
│   │   └── package.json
│   ├── beneficiary-web/     # React PWA — beneficiary app
│   │   ├── src/
│   │   │   ├── pages/
│   │   │   ├── components/
│   │   │   ├── hooks/
│   │   │   └── services/
│   │   ├── public/
│   │   │   └── sw.js       # Service worker
│   │   ├── package.json
│   │   └── vite.config.ts
│   └── ui-lib/             # Shared component library
│       ├── src/
│       │   ├── components/
│       │   ├── tokens/
│       │   └── icons/
│       ├── package.json
│       └── rollup.config.js
│
├── ussd-gateway/            # USSD interface (Go)
│   ├── main.go
│   ├── handlers/
│   ├── models/
│   └── Dockerfile
│
├── infra/                   # Infrastructure as code
│   ├── terraform/
│   │   ├── modules/
│   │   │   ├── eks/
│   │   │   ├── rds/
│   │   │   ├── redis/
│   │   │   ├── s3/
│   │   │   └── cloudfront/
│   │   ├── staging/
│   │   └── prod/
│   ├── helm/
│   │   ├── remit-save/
│   │   │   ├── Chart.yaml
│   │   │   ├── values.yaml
│   │   │   └── templates/
│   │   └── vault-pool/
│   └── docker-compose.yml  # Local development
│
├── scripts/                 # DevOps scripts
│   ├── seed-local.sh        # Seed local testnet with data
│   ├── reset-local.sh       # Reset local environment
│   └── deploy.sh            # Production deployment script
│
├── docs/                    # Documentation
│   ├── architecture.md
│   ├── contracts.md
│   ├── api.md
│   └── regulatory.md
│
├── .github/
│   └── workflows/
│       ├── test.yml
│       ├── build.yml
│       └── deploy.yml
│
├── docker-compose.yml       # Local dev environment
├── Makefile                 # Common commands
├── README.md                # This file
├── LICENSE
└── AGENTS.md                # AI agent guide
```

### Local Development

```bash
# 1. Start local Stellar devnet
docker compose up -d stellar

# 2. Build and deploy contracts
cd contracts/remit-save
make build
make deploy -- --network local
make test

# 3. Start backend services
cd backend
cargo run -p auth-service &
cargo run -p remit-service &
cargo run -p savings-service &

# 4. Start frontend
cd frontend/diaspora-app
npm install
npx expo start

# 5. Run USSD gateway (mock mode)
cd ussd-gateway
go run . --mock

# 6. Open in browser
open http://localhost:8081
```

### Environment Variables

```bash
# Backend /.env
STELLAR_RPC_URL=http://localhost:8000/soroban/rpc
STELLAR_NETWORK_PASSPHRASE=Test SDF Network ; September 2025
DATABASE_URL=postgres://postgres:password@localhost:5432/remitsave
REDIS_URL=redis://localhost:6379
NATS_URL=nats://localhost:4222
JWT_SECRET=local-dev-secret
CONTRACT_REMIT_SAVE=CBQO4S...   # Deployed contract address
ANCHOR_COWRIE_API_KEY=...
ANCHOR_VIBRANT_API_KEY=...
CIRCLE_API_KEY=...
SMTP_HOST=...
SMS_PROVIDER_KEY=...
```

---

## Testing Strategy

| Layer | Tool | Scope |
|---|---|---|
| **Smart Contracts** | `soroban test`, `cargo test`, SMT formal verification | Unit tests for every function, integration tests for full flow, property-based tests for share price math, invariant fuzzing |
| **Backend Services** | `cargo test`, `rust-spec` | Unit tests, integration tests with testcontainers (PostgreSQL + Redis), API contract tests (OpenAPI spec validation) |
| **Frontend** | Jest + React Native Testing Library | Component unit tests, integration tests with MSW (Mock Service Worker) for API calls, E2E with Detox (mobile) and Playwright (web) |
| **E2E** | Playwright + Stellar local devnet | Full user flows: onboard → fund → send → verify beneficiary receives → verify savings balance updated → withdraw |
| **Load** | k6 | 10k concurrent remittances, verify sub-2s latency, no dropped transactions, Stellar Horizon RPC rate limit handling |
| **Fault** | Chaos Mesh | Pod kills, network latency, DB failover — verify system self-heals, no double-spends, queue depth recovers |
| **Security** | Slither (Solidity -> Soroban adaptation), manual audit | Reentrancy, access control, arithmetic, flash loans, front-running, signature replay |

```bash
# Run all tests
make test

# Contract tests only
cd contracts/remit-save && cargo test

# Backend integration tests
cd backend && cargo test -- --include-integration

# Frontend E2E tests
cd frontend && npx playwright test

# Load test
cd scripts && k6 run load-test.js
```

---

## Roadmap

### Phase 1: Core (Q2 2026)
- [x] Soroban contract design
- [x] Architecture & README
- [ ] Contract development & testing
- [ ] Backend MVP (Rust services)
- [ ] Frontend MVP (send + auto-save flow)
- [ ] Local devnet E2E passing
- [ ] Testnet deployment (Stellar Testnet)

### Phase 2: Pilot (Q3 2026)
- [ ] Nigeria anchor integration (Cowrie)
- [ ] Nigeria regulatory application (CBN)
- [ ] KYC integration (IdentityPass)
- [ ] USSD gateway (Africa's Talking)
- [ ] Closed beta (50 diaspora users → 200 beneficiaries)
- [ ] Security audit (OpenZeppelin)

### Phase 3: Launch (Q4 2026)
- [ ] Kenya & Ghana anchor integrations
- [ ] Vault pool MVP (T-bill yield)
- [ ] App store deployment (iOS + Android)
- [ ] Public launch — Nigeria first
- [ ] Monitoring + alerting (Datadog/PagerDuty)
- [ ] Bug bounty program (Immunefi)

### Phase 4: Scale (Q1 2027)
- [ ] Additional corridors (Uganda, Rwanda, South Africa)
- [ ] More yield sources (MMF, lending pools)
- [ ] Savings gamification (streaks, challenges, community savings groups)
- [ ] B2B API for employers (payroll + auto-save for domestic workers)
- [ ] DeFi composability (Blend lending, DUSA liquidity pools)
- [ ] Cross-chain (Bridge from Celo, Polygon — where remittance flows originate)

---

## Contributing

We welcome contributions! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for detailed guidelines.

All contributors must adhere to our [Code of Conduct](CODE_OF_CONDUCT.md). By participating, you agree to uphold this code. Report unacceptable behavior to ogazipromise@gmail.com.

### Quick Start

1. Fork the repository
2. Create a feature branch (`git checkout -b feat/my-feature`)
3. Commit your changes (`git commit -m 'feat: add my feature'`)
4. Push to the branch (`git push origin feat/my-feature`)
5. Open a Pull Request — see the [PR template](.github/PULL_REQUEST_TEMPLATE.md)

### Code Standards

| Requirement | Standard |
|---|---|
| Rust formatting | `cargo fmt` (stable) |
| Rust linting | `cargo clippy` — no warnings |
| TypeScript | ESLint + Prettier (single quotes, 2-space indent) |
| Commit messages | [Conventional Commits](https://www.conventionalcommits.org/) (`feat:`, `fix:`, `docs:`, `chore:`, etc.) |
| Branch naming | `feat/`, `fix/`, `chore/`, `docs/` prefixes |
| PR size | < 400 lines changed unless discussed |

### Security Vulnerabilities

Please report security issues privately to ogazipromise@gmail.com. See [SECURITY.md](SECURITY.md) for details.

---

## License

This project is licensed under the **MIT License** — see [LICENSE](LICENSE) for details.

---

## Built With

| Technology | Purpose |
|---|---|
| [Stellar](https://stellar.org) | Blockchain network for cross-border payments |
| [Soroban](https://soroban.stellar.org) | Smart contract platform |
| [Rust](https://rust-lang.org) | Smart contracts & backend services |
| [Go](https://golang.org) | USSD gateway |
| [React Native](https://reactnative.dev) | Mobile app (diaspora sender) |
| [React](https://react.dev) | Web PWA (beneficiary) |
| [PostgreSQL](https://postgresql.org) | Primary database |
| [Redis](https://redis.io) | Cache & queue |
| [NATS](https://nats.io) | Event streaming |
| [Kubernetes (EKS)](https://aws.amazon.com/eks) | Container orchestration |
| [Terraform](https://terraform.io) | Infrastructure as Code |

---

*Empowering the African diaspora to build wealth at home — one remittance at a time.*
