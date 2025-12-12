//! Mock bridge store for development and testing

use crate::models::{
    BridgeState, BridgeStatus, ConnectionStatus, CreateMappingRequest,
    EndpointType, MappingDirection, MessageStats, MqttConfig, TopicMapping, ZmqConfig,
};
use chrono::Utc;
use parking_lot::RwLock;
use rand::Rng;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::OnceLock;
use std::sync::Arc;

// Uptime tracking
static UPTIME_START: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
static NEXT_MAPPING_ID: AtomicU32 = AtomicU32::new(4);

/// Mock data store for the bridge
pub struct MockBridgeStore {
    mqtt_config: RwLock<MqttConfig>,
    zmq_config: RwLock<ZmqConfig>,
    mappings: RwLock<Vec<TopicMapping>>,
    message_stats: RwLock<MessageStats>,
}

impl MockBridgeStore {
    pub fn new() -> Self {
        let now = Utc::now().timestamp() as u64;
        UPTIME_START.store(now, Ordering::SeqCst);

        Self {
            mqtt_config: RwLock::new(MqttConfig::default()),
            zmq_config: RwLock::new(ZmqConfig::default()),
            mappings: RwLock::new(Self::default_mappings()),
            message_stats: RwLock::new(MessageStats::default()),
        }
    }

    fn default_mappings() -> Vec<TopicMapping> {
        vec![
            TopicMapping {
                id: 1,
                source_endpoint_type: EndpointType::Mqtt,
                source_endpoint_id: 1,
                target_endpoint_type: EndpointType::Zmq,
                target_endpoint_id: 1,
                source_topic: "sensors/#".to_string(),
                target_topic: "zmq.sensors".to_string(),
                direction: MappingDirection::MqttToZmq,
                enabled: true,
                description: Some("Forward all sensor data to ZeroMQ".to_string()),
            },
            TopicMapping {
                id: 2,
                source_endpoint_type: EndpointType::Zmq,
                source_endpoint_id: 1,
                target_endpoint_type: EndpointType::Mqtt,
                target_endpoint_id: 1,
                source_topic: "commands".to_string(),
                target_topic: "mqtt/commands".to_string(),
                direction: MappingDirection::ZmqToMqtt,
                enabled: true,
                description: Some("Forward commands from ZeroMQ to MQTT".to_string()),
            },
            TopicMapping {
                id: 3,
                source_endpoint_type: EndpointType::Mqtt,
                source_endpoint_id: 1,
                target_endpoint_type: EndpointType::Zmq,
                target_endpoint_id: 1,
                source_topic: "telemetry/+/status".to_string(),
                target_topic: "telemetry.status".to_string(),
                direction: MappingDirection::Bidirectional,
                enabled: false,
                description: Some("Bidirectional telemetry sync".to_string()),
            },
        ]
    }

    /// Get current bridge status
    pub fn get_status(&self) -> BridgeStatus {
        let uptime_start = UPTIME_START.load(Ordering::SeqCst);
        let now = Utc::now().timestamp() as u64;
        let uptime = now.saturating_sub(uptime_start);

        BridgeStatus {
            state: BridgeState::Running,
            uptime_seconds: uptime,
            mqtt_status: ConnectionStatus::Connected,
            zmq_status: ConnectionStatus::Connected,
            version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }

    /// Get message statistics with simulated real-time data
    pub fn get_stats(&self) -> MessageStats {
        let mut rng = rand::thread_rng();
        let mut stats = self.message_stats.write();

        // Simulate some activity
        stats.mqtt_received += rng.gen_range(0..5);
        stats.mqtt_sent += rng.gen_range(0..3);
        stats.zmq_received += rng.gen_range(0..4);
        stats.zmq_sent += rng.gen_range(0..3);
        stats.messages_per_second = rng.gen_range(10.0..50.0);
        stats.avg_latency_ms = rng.gen_range(1.0..5.0);
        stats.queue_depth = rng.gen_range(0..100);

        stats.clone()
    }

    /// Get MQTT configuration
    pub fn get_mqtt_config(&self) -> MqttConfig {
        self.mqtt_config.read().clone()
    }

    /// Update MQTT configuration
    pub fn update_mqtt_config(&self, config: MqttConfig) -> MqttConfig {
        *self.mqtt_config.write() = config.clone();
        config
    }

    /// Get ZeroMQ configuration
    pub fn get_zmq_config(&self) -> ZmqConfig {
        self.zmq_config.read().clone()
    }

    /// Update ZeroMQ configuration
    pub fn update_zmq_config(&self, config: ZmqConfig) -> ZmqConfig {
        *self.zmq_config.write() = config.clone();
        config
    }

    /// Get all topic mappings
    pub fn get_mappings(&self) -> Vec<TopicMapping> {
        self.mappings.read().clone()
    }

    /// Add a new topic mapping
    pub fn add_mapping(&self, req: CreateMappingRequest) -> TopicMapping {
        let id = NEXT_MAPPING_ID.fetch_add(1, Ordering::SeqCst);
        let mapping = TopicMapping {
            id,
            source_endpoint_type: req.source_endpoint_type,
            source_endpoint_id: req.source_endpoint_id,
            target_endpoint_type: req.target_endpoint_type,
            target_endpoint_id: req.target_endpoint_id,
            source_topic: req.source_topic,
            target_topic: req.target_topic,
            direction: req.direction,
            enabled: req.enabled,
            description: req.description,
        };

        self.mappings.write().push(mapping.clone());
        mapping
    }

    /// Delete a topic mapping
    pub fn delete_mapping(&self, id: u32) -> bool {
        let mut mappings = self.mappings.write();
        if let Some(pos) = mappings.iter().position(|m| m.id == id) {
            mappings.remove(pos);
            true
        } else {
            false
        }
    }

    /// Update a topic mapping
    pub fn update_mapping(&self, id: u32, req: CreateMappingRequest) -> Option<TopicMapping> {
        let mut mappings = self.mappings.write();
        if let Some(mapping) = mappings.iter_mut().find(|m| m.id == id) {
            mapping.source_endpoint_type = req.source_endpoint_type;
            mapping.source_endpoint_id = req.source_endpoint_id;
            mapping.target_endpoint_type = req.target_endpoint_type;
            mapping.target_endpoint_id = req.target_endpoint_id;
            mapping.source_topic = req.source_topic;
            mapping.target_topic = req.target_topic;
            mapping.direction = req.direction;
            mapping.enabled = req.enabled;
            mapping.description = req.description;
            Some(mapping.clone())
        } else {
            None
        }
    }
}

impl Default for MockBridgeStore {
    fn default() -> Self {
        Self::new()
    }
}

/// Global mock store instance
static MOCK_STORE: OnceLock<Arc<MockBridgeStore>> = OnceLock::new();

/// Get the global mock store
pub fn get_mock_store() -> Arc<MockBridgeStore> {
    MOCK_STORE.get_or_init(|| Arc::new(MockBridgeStore::new())).clone()
}
