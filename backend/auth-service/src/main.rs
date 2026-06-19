use std::sync::Arc;

use axum::extract::{Request, State};
use axum::http::Method;
use axum::middleware::{self, Next};
use axum::response::Response;
use axum::routing::{get, post};
use axum::{Json, Router};
use backend_shared::{AppError, JwtClaims, RegisterRequest, RegisterResponse, UserPublic};
use serde_json::json;
use sqlx::PgPool;
use tower_http::cors::{Any, CorsLayer};
use tracing_subscriber::EnvFilter;

mod jwt;

#[cfg(test)]
mod tests;

#[derive(Clone)]
struct AppState {
    db: PgPool,
    jwt_secret: String,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| "auth_service=info".into()),
        )
        .init();

    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://remitsave:remitsave_dev@localhost:5432/remitsave".into());
    let jwt_secret = std::env::var("JWT_SECRET")
        .unwrap_or_else(|_| "remitsave-dev-secret-do-not-use-in-prod".into());

    let db = backend_shared::init_pool(&database_url).await;

    let state = Arc::new(AppState {
        db,
        jwt_secret: jwt_secret.clone(),
    });

    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_origin(Any)
        .allow_headers(Any);

    let auth_routes =
        Router::new()
            .route("/auth/me", get(me))
            .layer(middleware::from_fn_with_state(
                state.clone(),
                auth_middleware,
            ));

    let app = Router::new()
        .route("/health", get(health))
        .route("/auth/register", post(register))
        .merge(auth_routes)
        .layer(cors)
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3001").await.unwrap();
    tracing::info!(
        "auth-service listening on {}",
        listener.local_addr().unwrap()
    );
    axum::serve(listener, app).await.unwrap();
}

async fn health() -> Json<serde_json::Value> {
    Json(json!({ "status": "ok" }))
}

async fn register(
    State(state): State<Arc<AppState>>,
    Json(req): Json<RegisterRequest>,
) -> Result<Json<RegisterResponse>, AppError> {
    if req.email.is_empty() || req.password.is_empty() {
        return Err(AppError::BadRequest(
            "Email and password are required".into(),
        ));
    }
    if req.password.len() < 8 {
        return Err(AppError::BadRequest(
            "Password must be at least 8 characters".into(),
        ));
    }
    let valid_countries = ["NG", "KE", "GH", "UG", "RW", "ZA"];
    if !valid_countries.contains(&req.country.as_str()) {
        return Err(AppError::BadRequest(format!(
            "Invalid country. Must be one of: {:?}",
            valid_countries
        )));
    }

    let existing = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM users WHERE email = $1")
        .bind(&req.email)
        .fetch_one(&state.db)
        .await?;

    if existing > 0 {
        return Err(AppError::Conflict(
            "A user with this email already exists".into(),
        ));
    }

    let password_hash = bcrypt::hash(&req.password, bcrypt::DEFAULT_COST)
        .map_err(|e| AppError::Internal(format!("Failed to hash password: {e}")))?;

    let user_id = uuid::Uuid::new_v4();
    let now = chrono::Utc::now();

    sqlx::query(
        r#"
        INSERT INTO users (id, email, phone, country, password_hash, kyc_level, created_at, last_active)
        VALUES ($1, $2, $3, $4, $5, 0, $6, $6)
        "#,
    )
    .bind(user_id)
    .bind(&req.email)
    .bind(&req.phone)
    .bind(&req.country)
    .bind(&password_hash)
    .bind(now)
    .execute(&state.db)
    .await?;

    let token = jwt::issue_token(user_id, &state.jwt_secret)?;

    let user = UserPublic {
        id: user_id,
        email: req.email,
        phone: req.phone,
        country: req.country,
        kyc_level: 0,
        stellar_address: None,
        created_at: now,
        last_active: now,
    };

    Ok(Json(RegisterResponse { user, token }))
}

async fn me(
    State(state): State<Arc<AppState>>,
    req: Request,
) -> Result<Json<UserPublic>, AppError> {
    let claims = req
        .extensions()
        .get::<JwtClaims>()
        .cloned()
        .ok_or(AppError::Unauthorized("No valid JWT claims found".into()))?;
    let user_id: uuid::Uuid = claims
        .sub
        .parse()
        .map_err(|_| AppError::Unauthorized("Invalid token claims".into()))?;

    let user = sqlx::query_as::<_, backend_shared::User>(
        r#"
        SELECT id, email, phone, country, password_hash, kyc_level,
               stellar_address, created_at, last_active
        FROM users WHERE id = $1
        "#,
    )
    .bind(user_id)
    .fetch_optional(&state.db)
    .await?
    .ok_or(AppError::NotFound("User not found".into()))?;

    Ok(Json(user.into()))
}

async fn auth_middleware(
    State(state): State<Arc<AppState>>,
    mut req: Request,
    next: Next,
) -> Result<Response, AppError> {
    let auth_header = req
        .headers()
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .ok_or(AppError::Unauthorized(
            "Missing Authorization header".into(),
        ))?;

    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or(AppError::Unauthorized(
            "Invalid Authorization header format. Expected: Bearer <token>".into(),
        ))?;

    let claims = jwt::validate_token(token, &state.jwt_secret)?;
    req.extensions_mut().insert(claims);
    Ok(next.run(req).await)
}
