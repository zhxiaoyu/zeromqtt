//! JWT token handling

use crate::config::AppConfig;
use crate::error::{AppError, AppResult};
use crate::models::Claims;
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};

/// Encode a JWT token for the given username
pub fn encode_token(username: &str, config: &AppConfig) -> AppResult<String> {
    let now = Utc::now();
    let expiration = now + Duration::hours(config.jwt.expiration_hours);

    let claims = Claims {
        sub: username.to_string(),
        iat: now.timestamp(),
        exp: expiration.timestamp(),
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(config.jwt.secret.as_bytes()),
    )
    .map_err(|e| AppError::TokenError(format!("Failed to encode token: {}", e)))
}

/// Decode and validate a JWT token
pub fn decode_token(token: &str, config: &AppConfig) -> AppResult<Claims> {
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(config.jwt.secret.as_bytes()),
        &Validation::default(),
    )
    .map(|data| data.claims)
    .map_err(|e| AppError::TokenError(format!("Invalid token: {}", e)))
}

/// Validate user credentials against default config
pub fn validate_credentials(username: &str, password: &str, config: &AppConfig) -> bool {
    username == config.credentials.username && password == config.credentials.password
}
