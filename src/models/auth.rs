//! Authentication related models

use serde::{Deserialize, Serialize};

/// Login request payload
#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

/// Login response with JWT token
#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub token: String,
    pub token_type: String,
    pub expires_in: i64,
}

/// JWT Claims structure
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    /// Subject (username)
    pub sub: String,
    /// Expiration timestamp
    pub exp: i64,
    /// Issued at timestamp
    pub iat: i64,
}

/// User information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub username: String,
}

/// Current user response
#[derive(Debug, Serialize)]
pub struct MeResponse {
    pub username: String,
}

// ============ User Management Types ============

/// User record from database
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserRecord {
    pub id: u32,
    pub username: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub is_default: bool,
    pub created_at: i64,
    pub updated_at: i64,
}

/// User response for API (without password)
#[derive(Debug, Clone, Serialize)]
pub struct UserResponse {
    pub id: u32,
    pub username: String,
    pub is_default: bool,
    pub created_at: i64,
    pub updated_at: i64,
}

impl From<UserRecord> for UserResponse {
    fn from(user: UserRecord) -> Self {
        UserResponse {
            id: user.id,
            username: user.username,
            is_default: user.is_default,
            created_at: user.created_at,
            updated_at: user.updated_at,
        }
    }
}

/// Create user request
#[derive(Debug, Deserialize)]
pub struct CreateUserRequest {
    pub username: String,
    pub password: String,
}

/// Update user request (username only)
#[derive(Debug, Deserialize)]
pub struct UpdateUserRequest {
    pub username: String,
}

/// Change password request
#[derive(Debug, Deserialize)]
pub struct ChangePasswordRequest {
    pub current_password: Option<String>,
    pub new_password: String,
}
