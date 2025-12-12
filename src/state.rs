//! Application state shared across routes

use crate::bridge::BridgeCore;
use crate::config::AppConfig;
use crate::db::Repository;
use std::sync::Arc;

/// Shared application state
#[derive(Clone)]
pub struct AppState {
    pub config: Arc<AppConfig>,
    pub repo: Repository,
    pub bridge: Arc<BridgeCore>,
}

impl AppState {
    pub fn new(config: AppConfig, repo: Repository, bridge: BridgeCore) -> Self {
        Self {
            config: Arc::new(config),
            repo,
            bridge: Arc::new(bridge),
        }
    }
}
