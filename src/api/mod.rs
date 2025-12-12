//! API routes module

pub mod auth;
pub mod bridge;
pub mod config;
pub mod status;
pub mod metrics;

use crate::state::AppState;
use axum::Router;

pub use auth::auth_routes;
pub use bridge::bridge_routes;
pub use config::config_routes;
pub use status::status_routes;
pub use metrics::metrics_routes;

/// Create all API routes
pub fn api_routes() -> Router<AppState> {
    Router::new()
        .nest("/auth", auth_routes())
        .nest("/status", status_routes())
        .nest("/config", config_routes())
        .nest("/bridge", bridge_routes())
        .nest("/metrics", metrics_routes())
}
