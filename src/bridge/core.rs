//! Bridge core - orchestrates MQTT and ZeroMQ message forwarding

use crate::db::Repository;
use crate::models::{BridgeState, BridgeStatus, ConnectionStatus};
use crate::bridge::{TopicMapper, BridgeWorker};
use std::sync::Arc;
use tokio::sync::RwLock;
use parking_lot::Mutex;
use tracing::info;

/// Bridge state container
#[derive(Clone)]
pub struct BridgeCore {
    state: Arc<RwLock<BridgeState>>,
    repo: Repository,
    topic_mapper: Arc<RwLock<TopicMapper>>,
    worker: Arc<Mutex<BridgeWorker>>,
}

impl BridgeCore {
    /// Create a new bridge core
    pub fn new(repo: Repository) -> Self {
        Self {
            state: Arc::new(RwLock::new(BridgeState::Stopped)),
            repo,
            topic_mapper: Arc::new(RwLock::new(TopicMapper::new(vec![]))),
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

        // Load configurations
        let mqtt_config = self.repo.get_mqtt_config().await?;
        let zmq_config = self.repo.get_zmq_config().await?;
        let mappings = self.repo.get_mappings().await?;

        // Update topic mapper
        *self.topic_mapper.write().await = TopicMapper::new(mappings.clone());

        // Reset stats and record start time
        let _ = self.repo.reset_stats().await;

        // Start the worker
        {
            let mut worker = self.worker.lock();
            worker.start(mqtt_config, zmq_config, mappings, self.repo.clone())?;
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

    /// Reload topic mappings
    pub async fn reload_mappings(&self) -> Result<(), anyhow::Error> {
        let mappings = self.repo.get_mappings().await?;
        *self.topic_mapper.write().await = TopicMapper::new(mappings);
        info!("Topic mappings reloaded");
        Ok(())
    }
}
