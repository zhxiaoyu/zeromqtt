//! Authentication API handlers

use crate::auth::{encode_token, AuthUser};
use crate::error::{AppError, AppResult};
use crate::models::{LoginRequest, LoginResponse, MeResponse};
use crate::state::AppState;
use axum::{
    extract::State,
    routing::{get, post},
    Json, Router,
};

/// Login handler - validates credentials against database
async fn login(
    State(state): State<AppState>,
    Json(req): Json<LoginRequest>,
) -> AppResult<Json<LoginResponse>> {
    // Validate credentials using database
    let user = state.repo.verify_credentials(&req.username, &req.password).await
        .map_err(|e| AppError::DbError(format!("Database error: {}", e)))?;

    if user.is_none() {
        return Err(AppError::AuthError(
            "Invalid username or password".to_string(),
        ));
    }

    let token = encode_token(&req.username, &state.config)?;

    Ok(Json(LoginResponse {
        token,
        token_type: "Bearer".to_string(),
        expires_in: state.config.jwt.expiration_hours * 3600,
    }))
}

/// Get current user info
async fn me(AuthUser(user): AuthUser) -> Json<MeResponse> {
    Json(MeResponse {
        username: user.username,
    })
}

/// Create authentication routes
pub fn auth_routes() -> Router<AppState> {
    Router::new()
        .route("/login", post(login))
        .route("/me", get(me))
}

