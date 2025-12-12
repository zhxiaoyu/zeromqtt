//! Mock bridge data service
//!
//! Provides simulated bridge status, configurations, and statistics
//! for development and testing purposes.

use crate::models::{
    BridgeState, BridgeStatus, ChartData, ConnectionStatus, CreateMappingRequest, MappingDirection,
    MessageStats, MqttConfig, TimeSeriesPoint, TopicMapping, ZmqConfig,
};
use chrono::Utc;
use rand::Rng;
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::sync::{Arc, RwLock};

/// Static counter for uptime simulation
static UPTIME_START: AtomicU64 = AtomicU64::new(0);
static NEXT_MAPPING_ID: AtomicU32 = AtomicU32::new(4);

/// Mock data store
pub struct MockBridgeStore {
    mqtt_config: RwLock<MqttConfig>,
    zmq_config: RwLock<ZmqConfig>,
    mappings: RwLock<Vec<TopicMapping>>,
    message_stats: RwLock<MessageStats>,
}

impl Default for MockBridgeStore {
    fn default() -> Self {
        Self::new()
    }
}

impl MockBridgeStore {
    pub fn new() -> Self {
        // Initialize uptime start time
        UPTIME_START.store(Utc::now().timestamp() as u64, Ordering::SeqCst);

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
                source_topic: "sensors/#".to_string(),
                target_topic: "zmq.sensors".to_string(),
                direction: MappingDirection::MqttToZmq,
                enabled: true,
                description: Some("Forward all sensor data to ZeroMQ".to_string()),
            },
            TopicMapping {
                id: 2,
                source_topic: "commands".to_string(),
                target_topic: "mqtt/commands".to_string(),
                direction: MappingDirection::ZmqToMqtt,
                enabled: true,
                description: Some("Forward commands from ZeroMQ to MQTT".to_string()),
            },
            TopicMapping {
                id: 3,
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
        let mut stats = self.message_stats.write().unwrap();

        // Simulate message activity
        stats.mqtt_received += rng.gen_range(10..50);
        stats.mqtt_sent += rng.gen_range(5..30);
        stats.zmq_received += rng.gen_range(8..40);
        stats.zmq_sent += rng.gen_range(10..45);
        stats.messages_per_second = rng.gen_range(50.0..200.0);
        stats.avg_latency_ms = rng.gen_range(0.5..5.0);
        stats.queue_depth = rng.gen_range(0..100);

        stats.clone()
    }

    /// Get chart data for throughput
    pub fn get_throughput_chart(&self) -> Vec<ChartData> {
        let mut rng = rand::thread_rng();
        let now = Utc::now().timestamp();

        let mqtt_data: Vec<TimeSeriesPoint> = (0..30)
            .map(|i| TimeSeriesPoint {
                timestamp: now - (29 - i) * 60,
                value: rng.gen_range(100.0..500.0),
            })
            .collect();

        let zmq_data: Vec<TimeSeriesPoint> = (0..30)
            .map(|i| TimeSeriesPoint {
                timestamp: now - (29 - i) * 60,
                value: rng.gen_range(80.0..450.0),
            })
            .collect();

        vec![
            ChartData {
                label: "MQTT".to_string(),
                data: mqtt_data,
            },
            ChartData {
                label: "ZeroMQ".to_string(),
                data: zmq_data,
            },
        ]
    }

    /// Get MQTT configuration
    pub fn get_mqtt_config(&self) -> MqttConfig {
        self.mqtt_config.read().unwrap().clone()
    }

    /// Update MQTT configuration
    pub fn update_mqtt_config(&self, config: MqttConfig) -> MqttConfig {
        let mut current = self.mqtt_config.write().unwrap();
        *current = config;
        current.clone()
    }

    /// Get ZeroMQ configuration
    pub fn get_zmq_config(&self) -> ZmqConfig {
        self.zmq_config.read().unwrap().clone()
    }

    /// Update ZeroMQ configuration
    pub fn update_zmq_config(&self, config: ZmqConfig) -> ZmqConfig {
        let mut current = self.zmq_config.write().unwrap();
        *current = config;
        current.clone()
    }

    /// Get all topic mappings
    pub fn get_mappings(&self) -> Vec<TopicMapping> {
        self.mappings.read().unwrap().clone()
    }

    /// Add a new topic mapping
    pub fn add_mapping(&self, req: CreateMappingRequest) -> TopicMapping {
        let id = NEXT_MAPPING_ID.fetch_add(1, Ordering::SeqCst);
        let mapping = TopicMapping {
            id,
            source_topic: req.source_topic,
            target_topic: req.target_topic,
            direction: req.direction,
            enabled: req.enabled,
            description: req.description,
        };

        self.mappings.write().unwrap().push(mapping.clone());
        mapping
    }

    /// Delete a topic mapping by ID
    pub fn delete_mapping(&self, id: u32) -> bool {
        let mut mappings = self.mappings.write().unwrap();
        let original_len = mappings.len();
        mappings.retain(|m| m.id != id);
        mappings.len() < original_len
    }

    /// Update a topic mapping
    pub fn update_mapping(&self, id: u32, req: CreateMappingRequest) -> Option<TopicMapping> {
        let mut mappings = self.mappings.write().unwrap();
        if let Some(mapping) = mappings.iter_mut().find(|m| m.id == id) {
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

use std::sync::OnceLock;

/// Global mock store instance
static MOCK_STORE: OnceLock<Arc<MockBridgeStore>> = OnceLock::new();

/// Get the global mock store
pub fn get_mock_store() -> Arc<MockBridgeStore> {
    MOCK_STORE.get_or_init(|| Arc::new(MockBridgeStore::new())).clone()
}

