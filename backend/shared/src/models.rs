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

#[derive(Debug, Serialize, Deserialize)]
pub struct RegisterRequest {
    pub email: String,
    pub phone: String,
    pub country: String,
    pub password: String,
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

#[derive(Debug, Serialize, Deserialize)]
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

#[derive(Debug, Serialize, Deserialize)]
pub struct JwtClaims {
    pub sub: String,
    pub exp: usize,
    pub iat: usize,
}
