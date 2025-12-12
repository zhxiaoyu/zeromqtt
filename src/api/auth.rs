//! Authentication API handlers

use crate::auth::{encode_token, validate_credentials, AuthUser};
use crate::config::AppConfig;
use crate::error::{AppError, AppResult};
use crate::models::{LoginRequest, LoginResponse, MeResponse};
use axum::{extract::State, routing::{get, post}, Json, Router};
use std::sync::Arc;

/// Login handler
async fn login(
    State(config): State<Arc<AppConfig>>,
    Json(req): Json<LoginRequest>,
) -> AppResult<Json<LoginResponse>> {
    if !validate_credentials(&req.username, &req.password, &config) {
        return Err(AppError::AuthError("Invalid username or password".to_string()));
    }

    let token = encode_token(&req.username, &config)?;

    Ok(Json(LoginResponse {
        token,
        token_type: "Bearer".to_string(),
        expires_in: config.jwt.expiration_hours * 3600,
    }))
}

/// Get current user info
async fn me(AuthUser(user): AuthUser) -> Json<MeResponse> {
    Json(MeResponse {
        username: user.username,
    })
}

/// Create authentication routes
pub fn auth_routes(config: Arc<AppConfig>) -> Router {
    Router::new()
        .route("/login", post(login))
        .route("/me", get(me))
        .with_state(config)
}
