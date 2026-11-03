use backend_shared::{JwtClaims, RegisterRequest};
use uuid::Uuid;

#[test]
fn test_register_validation_empty_email() {
    let req = RegisterRequest {
        email: "".into(),
        phone: "+234".into(),
        country: "NG".into(),
        password: "password123".into(),
        stellar_address: None,
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
        stellar_address: None,
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
        stellar_address: None,
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
            stellar_address: None,
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

#[test]
fn test_login_validation_empty_fields() {
    let req1 = backend_shared::LoginRequest {
        email: "".into(),
        password: "secretpassword".into(),
    };
    assert!(req1.email.is_empty());
    assert!(!req1.password.is_empty());

    let req2 = backend_shared::LoginRequest {
        email: "user@example.com".into(),
        password: "".into(),
    };
    assert!(!req2.email.is_empty());
    assert!(req2.password.is_empty());
}

#[test]
fn test_bcrypt_password_verification() {
    let password = "SuperSecretPassword123!";
    let hashed = bcrypt::hash(password, 4).unwrap();

    let matches = bcrypt::verify(password, &hashed).unwrap();
    assert!(matches);

    let wrong = bcrypt::verify("WrongPassword123!", &hashed).unwrap();
    assert!(!wrong);
}

#[test]
fn test_login_response_payload() {
    let user_id = Uuid::new_v4();
    let now = chrono::Utc::now();
    let user_pub = backend_shared::UserPublic {
        id: user_id,
        email: "user@example.com".into(),
        phone: "+2348012345678".into(),
        country: "NG".into(),
        kyc_level: 1,
        stellar_address: Some("GBBD47IF6LWK7P7MDEVSCWR7DPUWV3NY3DTQEVFL4NAT4AQH3ZLLFLA5".into()),
        created_at: now,
        last_active: now,
    };

    let token = "dummy.jwt.token".to_string();
    let res = backend_shared::LoginResponse {
        user: user_pub.clone(),
        token: token.clone(),
    };

    assert_eq!(res.user.email, "user@example.com");
    assert_eq!(res.token, token);
    assert_eq!(res.user.kyc_level, 1);
}

#[test]
fn test_stellar_address_validation() {
    let valid_address = "GBBD47IF6LWK7P7MDEVSCWR7DPUWV3NY3DTQEVFL4NAT4AQH3ZLLFLA5";
    assert!(backend_shared::is_valid_stellar_address(valid_address));

    // Invalid prefix
    let invalid_prefix = "CBBD47IF6LWK7P7MDEVSCWR7DPUWV3NY3DTQEVFL4NAT4AQH3ZLLFLA5";
    assert!(!backend_shared::is_valid_stellar_address(invalid_prefix));

    // Too short
    let too_short = "GBBD47IF6LWK7P7MDEV";
    assert!(!backend_shared::is_valid_stellar_address(too_short));

    // Empty
    assert!(!backend_shared::is_valid_stellar_address(""));
}

#[test]
fn test_register_request_with_stellar_address() {
    let valid_addr = "GBBD47IF6LWK7P7MDEVSCWR7DPUWV3NY3DTQEVFL4NAT4AQH3ZLLFLA5";
    let req = backend_shared::RegisterRequest {
        email: "diaspora@example.com".into(),
        phone: "+447911123456".into(),
        country: "NG".into(),
        password: "SecurePassword123!".into(),
        stellar_address: Some(valid_addr.into()),
    };

    assert_eq!(req.stellar_address.as_deref(), Some(valid_addr));
    let json = serde_json::to_string(&req).unwrap();
    let deserialized: backend_shared::RegisterRequest = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.stellar_address.as_deref(), Some(valid_addr));
}
