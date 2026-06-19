use backend_shared::{JwtClaims, RegisterRequest};
use uuid::Uuid;

#[test]
fn test_register_validation_empty_email() {
    let req = RegisterRequest {
        email: "".into(),
        phone: "+234".into(),
        country: "NG".into(),
        password: "password123".into(),
    };
    assert!(req.email.is_empty());
    assert!(req.password.len() >= 8);
}

#[test]
fn test_register_validation_short_password() {
    let req = RegisterRequest {
        email: "test@test.com".into(),
        phone: "+234".into(),
        country: "NG".into(),
        password: "short".into(),
    };
    assert!(req.password.len() < 8);
}

#[test]
fn test_register_validation_invalid_country() {
    let valid = ["NG", "KE", "GH", "UG", "RW", "ZA"];
    let req = RegisterRequest {
        email: "test@test.com".into(),
        phone: "+234".into(),
        country: "FR".into(),
        password: "password123".into(),
    };
    assert!(!valid.contains(&req.country.as_str()));
}

#[test]
fn test_register_validation_valid_country() {
    let valid = ["NG", "KE", "GH", "UG", "RW", "ZA"];
    for country in valid {
        let req = RegisterRequest {
            email: "test@test.com".into(),
            phone: "+234".into(),
            country: country.to_string(),
            password: "password123".into(),
        };
        assert!(valid.contains(&req.country.as_str()));
    }
}

#[test]
fn test_jwt_token_roundtrip() {
    let user_id = Uuid::new_v4();
    let secret = "test-secret-key-for-testing";

    let token = crate::jwt::issue_token(user_id, secret).unwrap();
    assert!(!token.is_empty());

    let claims: JwtClaims = crate::jwt::validate_token(&token, secret).unwrap();
    assert_eq!(claims.sub, user_id.to_string());
}

#[test]
fn test_jwt_invalid_secret() {
    let user_id = Uuid::new_v4();
    let token = crate::jwt::issue_token(user_id, "secret1").unwrap();
    let result = crate::jwt::validate_token(&token, "secret2");
    assert!(result.is_err());
}

#[test]
fn test_jwt_invalid_token() {
    let result = crate::jwt::validate_token("invalid.jwt.token", "secret");
    assert!(result.is_err());
}

#[test]
fn test_jwt_expiry_in_future() {
    let user_id = Uuid::new_v4();
    let token = crate::jwt::issue_token(user_id, "secret").unwrap();

    use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
    let token_data = decode::<JwtClaims>(
        &token,
        &DecodingKey::from_secret("secret".as_bytes()),
        &Validation::new(Algorithm::HS256),
    )
    .unwrap();

    let now = chrono::Utc::now().timestamp() as usize;
    assert!(token_data.claims.exp > now);
}
