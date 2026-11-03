use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub phone: String,
    pub country: String,
    pub password_hash: String,
    pub kyc_level: i32,
    pub stellar_address: Option<String>,
    pub created_at: DateTime<Utc>,
    pub last_active: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct KycDocument {
    pub id: Uuid,
    pub user_id: Uuid,
    pub document_type: String,
    pub status: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Session {
    pub id: Uuid,
    pub user_id: Uuid,
    pub token: String,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterRequest {
    pub email: String,
    pub phone: String,
    pub country: String,
    pub password: String,
    #[serde(default)]
    pub stellar_address: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkStellarAddressRequest {
    pub stellar_address: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateProfileRequest {
    pub phone: Option<String>,
    pub country: Option<String>,
    pub stellar_address: Option<String>,
}

pub fn is_valid_stellar_address(addr: &str) -> bool {
    addr.len() == 56 && addr.starts_with('G') && addr.chars().all(|c| c.is_ascii_alphanumeric())
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RegisterResponse {
    pub user: UserPublic,
    pub token: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginResponse {
    pub user: UserPublic,
    pub token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPublic {
    pub id: Uuid,
    pub email: String,
    pub phone: String,
    pub country: String,
    pub kyc_level: i32,
    pub stellar_address: Option<String>,
    pub created_at: DateTime<Utc>,
    pub last_active: DateTime<Utc>,
}

impl From<User> for UserPublic {
    fn from(u: User) -> Self {
        UserPublic {
            id: u.id,
            email: u.email,
            phone: u.phone,
            country: u.country,
            kyc_level: u.kyc_level,
            stellar_address: u.stellar_address,
            created_at: u.created_at,
            last_active: u.last_active,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JwtClaims {
    pub sub: String,
    pub exp: usize,
    pub iat: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct VaultPoolRecord {
    pub id: Uuid,
    pub onchain_pool_id: i32,
    pub name: String,
    pub local_asset: String,
    pub apy_bps: i32,
    pub min_lockup_secs: i64,
    pub total_deposits: i64,
    pub total_shares: i64,
    pub share_price: i64,
    pub is_paused: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultPoolResponse {
    pub id: Uuid,
    pub onchain_pool_id: i32,
    pub name: String,
    pub local_asset: String,
    pub apy_bps: i32,
    pub apy_percent: f64,
    pub min_lockup_secs: i64,
    pub total_deposits: i64,
    pub total_shares: i64,
    pub share_price: i64,
    pub is_paused: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<VaultPoolRecord> for VaultPoolResponse {
    fn from(r: VaultPoolRecord) -> Self {
        VaultPoolResponse {
            id: r.id,
            onchain_pool_id: r.onchain_pool_id,
            name: r.name,
            local_asset: r.local_asset,
            apy_bps: r.apy_bps,
            apy_percent: (r.apy_bps as f64) / 100.0,
            min_lockup_secs: r.min_lockup_secs,
            total_deposits: r.total_deposits,
            total_shares: r.total_shares,
            share_price: r.share_price,
            is_paused: r.is_paused,
            created_at: r.created_at,
            updated_at: r.updated_at,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct VaultPositionRecord {
    pub id: Uuid,
    pub user_id: Uuid,
    pub pool_id: Uuid,
    pub shares: i64,
    pub deposited_amount: i64,
    pub locked_until: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultPositionResponse {
    pub id: Uuid,
    pub user_id: Uuid,
    pub pool_id: Uuid,
    pub pool_name: String,
    pub local_asset: String,
    pub shares: i64,
    pub deposited_amount: i64,
    pub estimated_value: i64,
    pub locked_until: DateTime<Utc>,
    pub is_locked: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultDepositRequest {
    pub amount: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultWithdrawRequest {
    pub shares: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultActionResponse {
    pub pool_id: Uuid,
    pub user_id: Uuid,
    pub shares: i64,
    pub deposited_amount: i64,
    pub estimated_value: i64,
    pub locked_until: DateTime<Utc>,
}
