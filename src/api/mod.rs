//! API routes module

pub mod auth;
pub mod config;
pub mod status;

use axum::Router;
use std::sync::Arc;
use crate::config::AppConfig;

pub use auth::auth_routes;
pub use config::config_routes;
pub use status::status_routes;

/// Create all API routes
pub fn api_routes(config: Arc<AppConfig>) -> Router {
    Router::new()
        .nest("/auth", auth_routes(config.clone()))
        .nest("/status", status_routes())
        .nest("/config", config_routes())
}
