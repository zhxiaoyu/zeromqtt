//! Authentication middleware for Axum

use crate::auth::jwt::decode_token;
use crate::config::AppConfig;
use crate::error::AppError;
use crate::models::User;
use axum::{
    extract::FromRequestParts,
    http::{header::AUTHORIZATION, request::Parts},
};
use std::sync::Arc;

/// Authenticated user extractor
#[derive(Debug, Clone)]
pub struct AuthUser(pub User);

impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        // Get authorization header
        let auth_header = parts
            .headers
            .get(AUTHORIZATION)
            .and_then(|value| value.to_str().ok())
            .ok_or_else(|| AppError::AuthError("Missing authorization header".to_string()))?;

        // Extract bearer token
        let token = auth_header
            .strip_prefix("Bearer ")
            .ok_or_else(|| AppError::AuthError("Invalid authorization header format".to_string()))?;

        // Get config from extensions
        let config = parts
            .extensions
            .get::<Arc<AppConfig>>()
            .ok_or_else(|| AppError::Internal("Config not found in request".to_string()))?;

        // Decode and validate token
        let claims = decode_token(token, config)?;

        Ok(AuthUser(User {
            username: claims.sub,
        }))
    }
}
