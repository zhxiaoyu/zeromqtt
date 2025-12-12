//! Status API handlers

use crate::mock::get_mock_store;
use crate::models::{BridgeStatus, ChartData, MessageStats};
use axum::{routing::get, Json, Router};

/// Get bridge status
async fn get_status() -> Json<BridgeStatus> {
    let store = get_mock_store();
    Json(store.get_status())
}

/// Get message statistics
async fn get_stats() -> Json<MessageStats> {
    let store = get_mock_store();
    Json(store.get_stats())
}

/// Get throughput chart data
async fn get_chart_data() -> Json<Vec<ChartData>> {
    let store = get_mock_store();
    Json(store.get_throughput_chart())
}

/// Create status routes
pub fn status_routes() -> Router {
    Router::new()
        .route("/", get(get_status))
        .route("/stats", get(get_stats))
        .route("/chart", get(get_chart_data))
}
