//! User management API handlers

use crate::error::{AppError, AppResult};
use crate::models::{
    ChangePasswordRequest, CreateUserRequest, UpdateUserRequest, UserResponse,
};
use crate::state::AppState;
use axum::{
    extract::{Path, State},
    routing::{get, post},
    Json, Router,
};

/// List all users
async fn list_users(State(state): State<AppState>) -> AppResult<Json<Vec<UserResponse>>> {
    let users = state
        .repo
        .get_users()
        .await
        .map_err(|e| AppError::DbError(format!("Failed to get users: {}", e)))?;

    let responses: Vec<UserResponse> = users.into_iter().map(|u| u.into()).collect();
    Ok(Json(responses))
}

/// Get a single user by ID
async fn get_user(
    State(state): State<AppState>,
    Path(id): Path<u32>,
) -> AppResult<Json<UserResponse>> {
    let user = state
        .repo
        .get_user_by_id(id)
        .await
        .map_err(|e| AppError::DbError(format!("Failed to get user: {}", e)))?;

    match user {
        Some(u) => Ok(Json(u.into())),
        None => Err(AppError::NotFound(format!("User with id {} not found", id))),
    }
}

/// Create a new user
async fn create_user(
    State(state): State<AppState>,
    Json(req): Json<CreateUserRequest>,
) -> AppResult<Json<UserResponse>> {
    // Validate request
    if req.username.trim().is_empty() {
        return Err(AppError::BadRequest("Username cannot be empty".to_string()));
    }
    if req.password.len() < 6 {
        return Err(AppError::BadRequest(
            "Password must be at least 6 characters".to_string(),
        ));
    }

    // Check if username already exists
    let existing = state
        .repo
        .get_user_by_username(&req.username)
        .await
        .map_err(|e| AppError::DbError(format!("Failed to check username: {}", e)))?;

    if existing.is_some() {
        return Err(AppError::BadRequest(format!(
            "Username '{}' already exists",
            req.username
        )));
    }

    let user = state
        .repo
        .create_user(&req)
        .await
        .map_err(|e| AppError::DbError(format!("Failed to create user: {}", e)))?;

    Ok(Json(user.into()))
}

/// Update an existing user
async fn update_user(
    State(state): State<AppState>,
    Path(id): Path<u32>,
    Json(req): Json<UpdateUserRequest>,
) -> AppResult<Json<UserResponse>> {
    // Validate request
    if req.username.trim().is_empty() {
        return Err(AppError::BadRequest("Username cannot be empty".to_string()));
    }

    // Check if username is taken by another user
    let existing = state
        .repo
        .get_user_by_username(&req.username)
        .await
        .map_err(|e| AppError::DbError(format!("Failed to check username: {}", e)))?;

    if let Some(u) = existing
        && u.id != id
    {
        return Err(AppError::BadRequest(format!(
            "Username '{}' already taken",
            req.username
        )));
    }

    let user = state
        .repo
        .update_user(id, &req)
        .await
        .map_err(|e| AppError::DbError(format!("Failed to update user: {}", e)))?;

    match user {
        Some(u) => Ok(Json(u.into())),
        None => Err(AppError::NotFound(format!("User with id {} not found", id))),
    }
}

/// Change user password
async fn change_password(
    State(state): State<AppState>,
    Path(id): Path<u32>,
    Json(req): Json<ChangePasswordRequest>,
) -> AppResult<Json<serde_json::Value>> {
    // Validate new password
    if req.new_password.len() < 6 {
        return Err(AppError::BadRequest(
            "New password must be at least 6 characters".to_string(),
        ));
    }

    let success = state
        .repo
        .change_password(id, &req)
        .await
        .map_err(|e| AppError::DbError(format!("Failed to change password: {}", e)))?;

    if success {
        Ok(Json(serde_json::json!({ "message": "Password changed successfully" })))
    } else {
        Err(AppError::BadRequest(
            "Invalid current password or user not found".to_string(),
        ))
    }
}

/// Delete a user (cannot delete default user)
async fn delete_user(
    State(state): State<AppState>,
    Path(id): Path<u32>,
) -> AppResult<Json<serde_json::Value>> {
    // Check if user is default
    let user = state
        .repo
        .get_user_by_id(id)
        .await
        .map_err(|e| AppError::DbError(format!("Failed to get user: {}", e)))?;

    if let Some(u) = user {
        if u.is_default {
            return Err(AppError::BadRequest(
                "Cannot delete default user".to_string(),
            ));
        }
    } else {
        return Err(AppError::NotFound(format!("User with id {} not found", id)));
    }

    let deleted = state
        .repo
        .delete_user(id)
        .await
        .map_err(|e| AppError::DbError(format!("Failed to delete user: {}", e)))?;

    if deleted {
        Ok(Json(serde_json::json!({ "message": "User deleted successfully" })))
    } else {
        Err(AppError::NotFound(format!("User with id {} not found", id)))
    }
}

/// Create user management routes
pub fn users_routes() -> Router<AppState> {
    Router::new()
        .route("/", get(list_users).post(create_user))
        .route("/{id}", get(get_user).put(update_user).delete(delete_user))
        .route("/{id}/password", post(change_password))
}
