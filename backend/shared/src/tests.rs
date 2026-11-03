use crate::models::*;
use uuid::Uuid;

#[test]
fn test_user_into_user_public() {
    let user = User {
        id: Uuid::new_v4(),
        email: "test@example.com".into(),
        phone: "+2348012345678".into(),
        country: "NG".into(),
        password_hash: "hashed".into(),
        kyc_level: 1,
        stellar_address: Some("GABCDEF123".into()),
        created_at: chrono::Utc::now(),
        last_active: chrono::Utc::now(),
    };

    let public: UserPublic = user.clone().into();
    assert_eq!(public.id, user.id);
    assert_eq!(public.email, user.email);
    assert_eq!(public.phone, user.phone);
    assert_eq!(public.country, user.country);
    assert_eq!(public.kyc_level, user.kyc_level);
    assert_eq!(public.stellar_address, user.stellar_address);
}

#[test]
fn test_register_request_validation() {
    let req = RegisterRequest {
        email: "user@example.com".into(),
        phone: "+2348000000000".into(),
        country: "KE".into(),
        password: "securepassword123".into(),
        stellar_address: None,
    };
    assert!(!req.email.is_empty());
    assert!(!req.password.is_empty());
    assert!(req.password.len() >= 8);
}

#[test]
fn test_register_response_serialization() {
    let user = UserPublic {
        id: Uuid::new_v4(),
        email: "a@b.com".into(),
        phone: "+234".into(),
        country: "NG".into(),
        kyc_level: 0,
        stellar_address: None,
        created_at: chrono::Utc::now(),
        last_active: chrono::Utc::now(),
    };
    let resp = RegisterResponse {
        token: "jwt_token".into(),
        user: user.clone(),
    };

    let json = serde_json::to_string(&resp).unwrap();
    let deserialized: RegisterResponse = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.token, resp.token);
    assert_eq!(deserialized.user.id, user.id);
}

#[test]
fn test_jwt_claims_roundtrip() {
    let claims = JwtClaims {
        sub: Uuid::new_v4().to_string(),
        exp: 9999999999,
        iat: 1000000000,
    };

    let json = serde_json::to_string(&claims).unwrap();
    let deserialized: JwtClaims = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.sub, claims.sub);
    assert_eq!(deserialized.exp, claims.exp);
}

#[test]
fn test_kyc_document_default_status() {
    let doc = KycDocument {
        id: Uuid::new_v4(),
        user_id: Uuid::new_v4(),
        document_type: "passport".into(),
        status: "pending".into(),
        created_at: chrono::Utc::now(),
    };
    assert_eq!(doc.status, "pending");
}

#[test]
fn test_session_expiry() {
    let now = chrono::Utc::now();
    let session = Session {
        id: Uuid::new_v4(),
        user_id: Uuid::new_v4(),
        token: "test_token".into(),
        expires_at: now + chrono::Duration::hours(1),
        created_at: now,
    };
    assert!(session.expires_at > session.created_at);
}

#[test]
fn test_vault_pool_conversion_and_apy() {
    let now = chrono::Utc::now();
    let pool = VaultPoolRecord {
        id: Uuid::new_v4(),
        onchain_pool_id: 1,
        name: "NGN Vault".into(),
        local_asset: "NGN".into(),
        apy_bps: 1250,
        min_lockup_secs: 86400,
        total_deposits: 1_000_000,
        total_shares: 100_000,
        share_price: 10_000_000,
        is_paused: false,
        created_at: now,
        updated_at: now,
    };

    let resp: VaultPoolResponse = pool.into();
    assert_eq!(resp.onchain_pool_id, 1);
    assert_eq!(resp.name, "NGN Vault");
    assert_eq!(resp.apy_bps, 1250);
    assert!((resp.apy_percent - 12.5).abs() < f64::EPSILON);
    assert_eq!(resp.share_price, 10_000_000);
    assert!(!resp.is_paused);
}

#[test]
fn test_vault_position_serialization() {
    let now = chrono::Utc::now();
    let pos = VaultPositionResponse {
        id: Uuid::new_v4(),
        user_id: Uuid::new_v4(),
        pool_id: Uuid::new_v4(),
        pool_name: "KES Staking".into(),
        local_asset: "KES".into(),
        shares: 50_000,
        deposited_amount: 500_000,
        estimated_value: 525_000,
        locked_until: now + chrono::Duration::days(7),
        is_locked: true,
        created_at: now,
        updated_at: now,
    };

    let serialized = serde_json::to_string(&pos).unwrap();
    let deserialized: VaultPositionResponse = serde_json::from_str(&serialized).unwrap();
    assert_eq!(deserialized.shares, 50_000);
    assert_eq!(deserialized.estimated_value, 525_000);
    assert!(deserialized.is_locked);
}

#[test]
fn test_app_error_messages() {
    use crate::error::AppError;

    let err = AppError::BadRequest("bad input".into());
    assert!(format!("{err:?}").contains("bad input"));

    let err = AppError::Unauthorized("no access".into());
    assert!(format!("{err:?}").contains("no access"));

    let err = AppError::NotFound("missing".into());
    assert!(format!("{err:?}").contains("missing"));
}
