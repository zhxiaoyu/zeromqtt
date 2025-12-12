//! Bridge control API handlers

use crate::state::AppState;
use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::post,
    Json, Router,
};
use serde::Serialize;

#[derive(Serialize)]
struct BridgeActionResponse {
    success: bool,
    message: String,
}

/// Start the bridge
async fn start_bridge(State(state): State<AppState>) -> impl IntoResponse {
    match state.bridge.start().await {
        Ok(_) => (
            StatusCode::OK,
            Json(BridgeActionResponse {
                success: true,
                message: "Bridge started successfully".to_string(),
            }),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(BridgeActionResponse {
                success: false,
                message: e.to_string(),
            }),
        ),
    }
}

/// Stop the bridge
async fn stop_bridge(State(state): State<AppState>) -> impl IntoResponse {
    match state.bridge.stop().await {
        Ok(_) => (
            StatusCode::OK,
            Json(BridgeActionResponse {
                success: true,
                message: "Bridge stopped successfully".to_string(),
            }),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(BridgeActionResponse {
                success: false,
                message: e.to_string(),
            }),
        ),
    }
}

/// Restart the bridge
async fn restart_bridge(State(state): State<AppState>) -> impl IntoResponse {
    match state.bridge.restart().await {
        Ok(_) => (
            StatusCode::OK,
            Json(BridgeActionResponse {
                success: true,
                message: "Bridge restarted successfully".to_string(),
            }),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(BridgeActionResponse {
                success: false,
                message: e.to_string(),
            }),
        ),
    }
}

/// Create bridge control routes
pub fn bridge_routes() -> Router<AppState> {
    Router::new()
        .route("/start", post(start_bridge))
        .route("/stop", post(stop_bridge))
        .route("/restart", post(restart_bridge))
}
