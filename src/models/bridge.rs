//! Bridge related models

use serde::{Deserialize, Serialize};

/// Bridge running status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum BridgeState {
    Running,
    Stopped,
    Error,
    Connecting,
}

/// Connection status for MQTT or ZeroMQ
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ConnectionStatus {
    Connected,
    Disconnected,
    Connecting,
    Error,
}

/// Overall bridge status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BridgeStatus {
    pub state: BridgeState,
    pub uptime_seconds: u64,
    pub mqtt_status: ConnectionStatus,
    pub zmq_status: ConnectionStatus,
    pub version: String,
}

/// MQTT connection configuration - supports multiple brokers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MqttConfig {
    pub id: Option<u32>,
    pub name: String,              // Broker name: "Primary", "Backup", etc.
    pub enabled: bool,             // Whether this broker is active
    pub broker_url: String,
    pub port: u16,
    pub client_id: String,
    pub username: Option<String>,
    pub password: Option<String>,
    pub use_tls: bool,
    pub keep_alive_seconds: u16,
    pub clean_session: bool,
}

impl Default for MqttConfig {
    fn default() -> Self {
        Self {
            id: None,
            name: "Default".to_string(),
            enabled: true,
            broker_url: "localhost".to_string(),
            port: 1883,
            client_id: "zeromqtt-bridge".to_string(),
            username: None,
            password: None,
            use_tls: false,
            keep_alive_seconds: 60,
            clean_session: true,
        }
    }
}

/// Request to create/update MQTT config
#[derive(Debug, Clone, Deserialize)]
pub struct CreateMqttConfigRequest {
    pub name: String,
    pub enabled: bool,
    pub broker_url: String,
    pub port: u16,
    pub client_id: String,
    pub username: Option<String>,
    pub password: Option<String>,
    pub use_tls: bool,
    pub keep_alive_seconds: u16,
    pub clean_session: bool,
}

/// ZeroMQ socket type for XPUB/XSUB proxy pattern
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "lowercase")]
pub enum ZmqSocketType {
    /// XPUB socket - binds and receives subscriptions from SUBs
    #[default]
    XPub,
    /// XSUB socket - binds and connects to multiple PUBs
    XSub,
    /// Standard PUB socket - connects to XSUB
    Pub,
    /// Standard SUB socket - connects to XPUB
    Sub,
}

/// ZeroMQ connection configuration - supports XPUB/XSUB proxy pattern
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZmqConfig {
    pub id: Option<u32>,
    pub name: String,                       // Config name: "Proxy", "Publisher", etc.
    pub enabled: bool,
    pub socket_type: ZmqSocketType,
    pub bind_endpoint: Option<String>,      // For XPUB/XSUB: bind address
    pub connect_endpoints: Vec<String>,     // For PUB/SUB: connect addresses
    pub high_water_mark: u32,
    pub reconnect_interval_ms: u32,
}

impl Default for ZmqConfig {
    fn default() -> Self {
        Self {
            id: None,
            name: "Default".to_string(),
            enabled: true,
            socket_type: ZmqSocketType::XPub,
            bind_endpoint: Some("tcp://*:5555".to_string()),
            connect_endpoints: vec![],
            high_water_mark: 1000,
            reconnect_interval_ms: 1000,
        }
    }
}

/// Request to create/update ZMQ config
#[derive(Debug, Clone, Deserialize)]
pub struct CreateZmqConfigRequest {
    pub name: String,
    pub enabled: bool,
    pub socket_type: ZmqSocketType,
    pub bind_endpoint: Option<String>,
    pub connect_endpoints: Vec<String>,
    pub high_water_mark: u32,
    pub reconnect_interval_ms: u32,
}

/// Endpoint type for topic mapping
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum EndpointType {
    Mqtt,
    Zmq,
}

/// Topic mapping direction - now supports intra-protocol forwarding
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum MappingDirection {
    MqttToZmq,
    ZmqToMqtt,
    MqttToMqtt,      // Forward between MQTT brokers
    ZmqToZmq,        // Forward between ZMQ endpoints
    Bidirectional,
}

/// Topic mapping rule - enhanced with endpoint references
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopicMapping {
    pub id: u32,
    pub source_endpoint_type: EndpointType,
    pub source_endpoint_id: u32,           // References mqtt_configs or zmq_configs
    pub target_endpoint_type: EndpointType,
    pub target_endpoint_id: u32,
    pub source_topic: String,
    pub target_topic: String,
    pub direction: MappingDirection,
    pub enabled: bool,
    pub description: Option<String>,
}

/// Request to create a new topic mapping
#[derive(Debug, Deserialize)]
pub struct CreateMappingRequest {
    pub source_endpoint_type: EndpointType,
    pub source_endpoint_id: u32,
    pub target_endpoint_type: EndpointType,
    pub target_endpoint_id: u32,
    pub source_topic: String,
    pub target_topic: String,
    pub direction: MappingDirection,
    pub enabled: bool,
    pub description: Option<String>,
}

/// Message statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageStats {
    /// Total messages received from MQTT
    pub mqtt_received: u64,
    /// Total messages sent to MQTT
    pub mqtt_sent: u64,
    /// Total messages received from ZeroMQ
    pub zmq_received: u64,
    /// Total messages sent to ZeroMQ
    pub zmq_sent: u64,
    /// Messages per second (current rate)
    pub messages_per_second: f64,
    /// Average latency in milliseconds
    pub avg_latency_ms: f64,
    /// Error count
    pub error_count: u64,
    /// Queue depth
    pub queue_depth: u32,
}

impl Default for MessageStats {
    fn default() -> Self {
        Self {
            mqtt_received: 0,
            mqtt_sent: 0,
            zmq_received: 0,
            zmq_sent: 0,
            messages_per_second: 0.0,
            avg_latency_ms: 0.0,
            error_count: 0,
            queue_depth: 0,
        }
    }
}

/// Time series data point for charts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeSeriesPoint {
    pub timestamp: i64,
    pub value: f64,
}

/// Chart data for dashboard
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChartData {
    pub label: String,
    pub data: Vec<TimeSeriesPoint>,
}
