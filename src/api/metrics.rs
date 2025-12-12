//! Metrics API endpoint for Prometheus scraping

use axum::{
    Router,
    response::IntoResponse,
    routing::get,
    http::header::CONTENT_TYPE,
};
use crate::state::AppState;
use crate::telemetry::metrics;

/// Get Prometheus metrics
async fn get_metrics() -> impl IntoResponse {
    let output = metrics().render_prometheus();
    (
        [(CONTENT_TYPE, "text/plain; version=0.0.4; charset=utf-8")],
        output
    )
}

/// Create metrics routes
pub fn metrics_routes() -> Router<AppState> {
    Router::new()
        .route("/", get(get_metrics))
}
