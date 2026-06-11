use backend_shared::{AppError, JwtClaims};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use uuid::Uuid;

pub fn issue_token(user_id: Uuid, secret: &str) -> Result<String, AppError> {
    let now = chrono::Utc::now();
    let claims = JwtClaims {
        sub: user_id.to_string(),
        iat: now.timestamp() as usize,
        exp: (now + chrono::Duration::hours(24)).timestamp() as usize,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|e| AppError::Internal(format!("Token generation failed: {e}")))
}

pub fn validate_token(token: &str, secret: &str) -> Result<JwtClaims, AppError> {
    let token_data = decode::<JwtClaims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )?;

    Ok(token_data.claims)
}
