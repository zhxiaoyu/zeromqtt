//! Bridge core - orchestrates MQTT and ZeroMQ message forwarding
//! Now supports multiple MQTT brokers and XPUB/XSUB proxy pattern

use crate::db::Repository;
use crate::models::{BridgeState, BridgeStatus, ConnectionStatus, TopicMapping};
use crate::bridge::BridgeWorker;
use std::sync::Arc;
use tokio::sync::RwLock;
use parking_lot::Mutex;
use tracing::info;

/// Bridge state container
#[derive(Clone)]
pub struct BridgeCore {
    state: Arc<RwLock<BridgeState>>,
    repo: Repository,
    /// Shared mappings cache - updated on add/update/delete, used by worker
    mappings_cache: Arc<RwLock<Vec<TopicMapping>>>,
    worker: Arc<Mutex<BridgeWorker>>,
}

impl BridgeCore {
    /// Create a new bridge core
    pub fn new(repo: Repository) -> Self {
        Self {
            state: Arc::new(RwLock::new(BridgeState::Stopped)),
            repo,
            mappings_cache: Arc::new(RwLock::new(vec![])),
            worker: Arc::new(Mutex::new(BridgeWorker::new())),
        }
    }

    /// Get current bridge status
    pub async fn get_status(&self) -> BridgeStatus {
        let state = self.state.read().await.clone();
        let start_time = self.repo.get_start_time().await.unwrap_or(0);
        let now = chrono::Utc::now().timestamp();
        let uptime = if start_time > 0 && state == BridgeState::Running {
            (now - start_time) as u64
        } else {
            0
        };

        // Determine connection statuses based on state
        let (mqtt_status, zmq_status) = match state {
            BridgeState::Running => (ConnectionStatus::Connected, ConnectionStatus::Connected),
            BridgeState::Connecting => (ConnectionStatus::Connecting, ConnectionStatus::Connecting),
            BridgeState::Error => (ConnectionStatus::Error, ConnectionStatus::Error),
            BridgeState::Stopped => (ConnectionStatus::Disconnected, ConnectionStatus::Disconnected),
        };

        BridgeStatus {
            state,
            uptime_seconds: uptime,
            mqtt_status,
            zmq_status,
            version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }

    /// Start the bridge
    pub async fn start(&self) -> Result<(), anyhow::Error> {
        {
            let current_state = self.state.read().await;
            if *current_state == BridgeState::Running {
                info!("Bridge is already running");
                return Ok(());
            }
        }

        info!("Starting bridge...");
        *self.state.write().await = BridgeState::Connecting;

        // Load configurations - now supporting multiple configs
        let mqtt_configs = self.repo.get_mqtt_configs().await?;
        let zmq_configs = self.repo.get_zmq_configs().await?;
        let mappings = self.repo.get_mappings().await?;

        // Initialize mappings cache
        *self.mappings_cache.write().await = mappings;

        // Reset stats and record start time
        let _ = self.repo.reset_stats().await;

        // Start the worker with shared mappings cache
        {
            let mut worker = self.worker.lock();
            worker.start_extended(
                mqtt_configs, 
                zmq_configs, 
                self.mappings_cache.clone(), 
                self.repo.clone()
            )?;
        }

        *self.state.write().await = BridgeState::Running;
        info!("Bridge started successfully");

        Ok(())
    }

    /// Stop the bridge
    pub async fn stop(&self) -> Result<(), anyhow::Error> {
        info!("Stopping bridge...");

        {
            let mut worker = self.worker.lock();
            worker.stop();
        }

        *self.state.write().await = BridgeState::Stopped;
        info!("Bridge stopped");
        Ok(())
    }

    /// Restart the bridge
    pub async fn restart(&self) -> Result<(), anyhow::Error> {
        self.stop().await?;
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        self.start().await
    }

    /// Reload topic mappings from database into cache and update subscriptions
    pub async fn reload_mappings(&self) -> Result<(), anyhow::Error> {
        let mappings = self.repo.get_mappings().await?;
        *self.mappings_cache.write().await = mappings.clone();
        
        // Update MQTT subscriptions dynamically
        {
            let worker = self.worker.lock();
            worker.update_subscriptions(&mappings);
        }
        
        info!("Topic mappings reloaded into cache");
        Ok(())
    }
}
