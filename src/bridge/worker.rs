//! Bridge worker - handles message forwarding with XPUB/XSUB proxy and multi-broker support

use crate::db::Repository;
use crate::models::{MqttConfig, ZmqConfig, TopicMapping, ZmqSocketType, EndpointType};
use crate::telemetry::metrics;
use std::sync::Arc;
use std::time::Instant;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::{self, JoinHandle};
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};

/// Message to be forwarded
#[derive(Debug, Clone)]
pub struct ForwardMessage {
    pub source: MessageSource,
    pub source_id: u32,
    pub topic: String,
    pub payload: Vec<u8>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MessageSource {
    Mqtt,
    Zmq,
}

/// Bridge worker that runs MQTT and ZMQ clients in dedicated threads
pub struct BridgeWorker {
    running: Arc<AtomicBool>,
    mqtt_threads: Vec<JoinHandle<()>>,
    zmq_threads: Vec<JoinHandle<()>>,
    forward_tx: Option<mpsc::Sender<ForwardMessage>>,
    /// MQTT command channels for dynamic subscription updates
    mqtt_cmd_txs: std::collections::HashMap<u32, std::sync::mpsc::Sender<MqttCommand>>,
}

impl BridgeWorker {
    pub fn new() -> Self {
        Self {
            running: Arc::new(AtomicBool::new(false)),
            mqtt_threads: vec![],
            zmq_threads: vec![],
            forward_tx: None,
            mqtt_cmd_txs: std::collections::HashMap::new(),
        }
    }

    /// Start the bridge worker with extended multi-config support
    pub fn start_extended(
        &mut self,
        mqtt_configs: Vec<MqttConfig>,
        zmq_configs: Vec<ZmqConfig>,
        mappings_cache: Arc<tokio::sync::RwLock<Vec<TopicMapping>>>,
        repo: Repository,
    ) -> Result<(), anyhow::Error> {
        if self.running.load(Ordering::SeqCst) {
            return Ok(());
        }

        self.running.store(true, Ordering::SeqCst);

        // Create channels for message forwarding
        let (forward_tx, mut forward_rx) = mpsc::channel::<ForwardMessage>(1000);
        
        // Command channels for each endpoint
        let mut mqtt_cmd_txs: std::collections::HashMap<u32, std::sync::mpsc::Sender<MqttCommand>> = std::collections::HashMap::new();
        let mut zmq_cmd_txs: std::collections::HashMap<u32, std::sync::mpsc::Sender<ZmqCommand>> = std::collections::HashMap::new();

        self.forward_tx = Some(forward_tx.clone());

        // Start MQTT threads for each enabled broker
        for config in mqtt_configs.iter().filter(|c| c.enabled) {
            let (mqtt_cmd_tx, mqtt_cmd_rx) = std::sync::mpsc::channel::<MqttCommand>();
            let config_id = config.id.unwrap_or(0);
            mqtt_cmd_txs.insert(config_id, mqtt_cmd_tx);
            
            // Get initial topics from mappings cache
            // New topics can be subscribed dynamically via MqttCommand::Subscribe
            let subscribe_topics: Vec<String> = {
                if let Ok(guard) = mappings_cache.try_read() {
                    guard.iter()
                        .filter(|m| m.enabled && m.source_endpoint_type == EndpointType::Mqtt && m.source_endpoint_id == config_id)
                        .map(|m| m.source_topic.clone())
                        .collect()
                } else {
                    vec![]
                }
            };

            let running_mqtt = self.running.clone();
            let forward_tx_mqtt = forward_tx.clone();
            let config_clone = config.clone();

            let mqtt_thread = thread::spawn(move || {
                run_mqtt_worker(
                    running_mqtt,
                    config_clone,
                    subscribe_topics,
                    forward_tx_mqtt,
                    mqtt_cmd_rx,
                );
            });

            self.mqtt_threads.push(mqtt_thread);
        }

        // Start ZMQ threads for each enabled config (XPUB/XSUB pattern)
        for config in zmq_configs.iter().filter(|c| c.enabled) {
            let (zmq_cmd_tx, zmq_cmd_rx) = std::sync::mpsc::channel::<ZmqCommand>();
            let config_id = config.id.unwrap_or(0);
            zmq_cmd_txs.insert(config_id, zmq_cmd_tx);

            let running_zmq = self.running.clone();
            let forward_tx_zmq = forward_tx.clone();
            let config_clone = config.clone();

            let zmq_thread = thread::spawn(move || {
                run_zmq_worker(
                    running_zmq,
                    config_clone,
                    forward_tx_zmq,
                    zmq_cmd_rx,
                );
            });

            self.zmq_threads.push(zmq_thread);
        }

        // Store MQTT command channels for dynamic subscription updates
        self.mqtt_cmd_txs = mqtt_cmd_txs.clone();

        // Start forwarding task
        let running_fwd = self.running.clone();
        let repo_fwd = repo.clone();
        let mappings_cache_fwd = mappings_cache.clone();

        tokio::spawn(async move {
            while running_fwd.load(Ordering::SeqCst) {
                tokio::select! {
                    Some(msg) = forward_rx.recv() => {
                        let forward_start = Instant::now();
                        info!("Received message from {:?} id={}: topic={}", msg.source, msg.source_id, msg.topic);
                        
                        // Track received stats (both DB and telemetry)
                        match msg.source {
                            MessageSource::Mqtt => {
                                metrics().record_mqtt_received();
                                let _ = repo_fwd.increment_stats(1, 0, 0, 0, 0).await;
                            }
                            MessageSource::Zmq => {
                                metrics().record_zmq_received();
                                let _ = repo_fwd.increment_stats(0, 0, 1, 0, 0).await;
                            }
                        }
                        
                        // Read mappings from shared cache (fast, in-memory)
                        let mappings = mappings_cache_fwd.read().await;
                        
                        let mut matched = false;
                        // Find matching mappings
                        for mapping in mappings.iter().filter(|m| m.enabled) {
                            // Check if source matches
                            let source_matches = match msg.source {
                                MessageSource::Mqtt => {
                                    mapping.source_endpoint_type == EndpointType::Mqtt
                                        && mapping.source_endpoint_id == msg.source_id
                                        && matches_topic_pattern(&mapping.source_topic, &msg.topic)
                                }
                                MessageSource::Zmq => {
                                    mapping.source_endpoint_type == EndpointType::Zmq
                                        && mapping.source_endpoint_id == msg.source_id
                                        && matches_topic_pattern(&mapping.source_topic, &msg.topic)
                                }
                            };

                            if source_matches {
                                matched = true;
                                let target_topic = apply_mapping(&mapping.source_topic, &mapping.target_topic, &msg.topic);
                                
                                match mapping.target_endpoint_type {
                                    EndpointType::Mqtt => {
                                        if let Some(tx) = mqtt_cmd_txs.get(&mapping.target_endpoint_id) {
                                            info!("Forwarding to MQTT endpoint {}: {}", mapping.target_endpoint_id, target_topic);
                                            let _ = tx.send(MqttCommand::Publish(target_topic, msg.payload.clone()));
                                            metrics().record_mqtt_sent();
                                            let _ = repo_fwd.increment_stats(0, 1, 0, 0, 0).await;
                                        } else {
                                            metrics().record_error();
                                            warn!("MQTT endpoint {} not found!", mapping.target_endpoint_id);
                                        }
                                    }
                                    EndpointType::Zmq => {
                                        if let Some(tx) = zmq_cmd_txs.get(&mapping.target_endpoint_id) {
                                            info!("Forwarding to ZMQ endpoint {}: {}", mapping.target_endpoint_id, target_topic);
                                            let _ = tx.send(ZmqCommand::Publish(target_topic, msg.payload.clone()));
                                            metrics().record_zmq_sent();
                                            let _ = repo_fwd.increment_stats(0, 0, 0, 1, 0).await;
                                        } else {
                                            metrics().record_error();
                                            warn!("ZMQ endpoint {} not found!", mapping.target_endpoint_id);
                                        }
                                    }
                                }
                            }
                        }
                        
                        if !matched {
                            debug!("No matching mapping found for topic: {}", msg.topic);
                        } else {
                            // Record forwarding latency
                            let latency_ms = forward_start.elapsed().as_secs_f64() * 1000.0;
                            metrics().record_latency(latency_ms);
                        }
                    }
                    else => {
                        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                    }
                }
            }
        });

        info!("Bridge worker started with {} MQTT brokers and {} ZMQ endpoints", 
              mqtt_configs.iter().filter(|c| c.enabled).count(),
              zmq_configs.iter().filter(|c| c.enabled).count());
        Ok(())
    }

    /// Update MQTT subscriptions dynamically based on new mappings
    pub fn update_subscriptions(&self, mappings: &[TopicMapping]) {
        for (config_id, tx) in &self.mqtt_cmd_txs {
            // Get topics for this MQTT broker from the mappings
            let topics: Vec<String> = mappings
                .iter()
                .filter(|m| m.enabled && m.source_endpoint_type == EndpointType::Mqtt && m.source_endpoint_id == *config_id)
                .map(|m| m.source_topic.clone())
                .collect();
            
            if !topics.is_empty() {
                if let Err(e) = tx.send(MqttCommand::Subscribe(topics.clone())) {
                    error!("Failed to send subscribe command: {}", e);
                } else {
                    info!("Sent subscribe command for topics: {:?}", topics);
                }
            }
        }
    }

    /// Stop the bridge worker
    pub fn stop(&mut self) {
        self.running.store(false, Ordering::SeqCst);
        
        // Wait for threads to finish
        for handle in self.mqtt_threads.drain(..) {
            let _ = handle.join();
        }
        for handle in self.zmq_threads.drain(..) {
            let _ = handle.join();
        }
        
        self.forward_tx = None;
        info!("Bridge worker stopped");
    }

    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }
}

impl Default for BridgeWorker {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for BridgeWorker {
    fn drop(&mut self) {
        self.stop();
    }
}

// Commands for MQTT thread
enum MqttCommand {
    Publish(String, Vec<u8>),
    Subscribe(Vec<String>),
}

// Commands for ZMQ thread
enum ZmqCommand {
    Publish(String, Vec<u8>),
}

fn run_mqtt_worker(
    running: Arc<AtomicBool>,
    config: MqttConfig,
    subscribe_topics: Vec<String>,
    forward_tx: mpsc::Sender<ForwardMessage>,
    cmd_rx: std::sync::mpsc::Receiver<MqttCommand>,
) {
    use paho_mqtt::{AsyncClient, ConnectOptionsBuilder, CreateOptionsBuilder, Message};
    use std::time::Duration;

    let config_id = config.id.unwrap_or(0);
    let server_uri = if config.use_tls {
        format!("ssl://{}:{}", config.broker_url, config.port)
    } else {
        format!("tcp://{}:{}", config.broker_url, config.port)
    };

    let create_opts = CreateOptionsBuilder::new()
        .server_uri(&server_uri)
        .client_id(&config.client_id)
        .finalize();

    let mut client = match AsyncClient::new(create_opts) {
        Ok(c) => c,
        Err(e) => {
            error!("[MQTT:{}] Failed to create client: {}", config.name, e);
            return;
        }
    };

    let rt = match tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build() {
        Ok(rt) => rt,
        Err(e) => {
            error!("[MQTT:{}] Failed to create tokio runtime: {}", config.name, e);
            return;
        }
    };

    rt.block_on(async {
        let mut conn_opts = ConnectOptionsBuilder::new();
        conn_opts
            .keep_alive_interval(Duration::from_secs(config.keep_alive_seconds as u64))
            .clean_session(config.clean_session)
            .automatic_reconnect(Duration::from_secs(1), Duration::from_secs(30));

        if let Some(ref username) = config.username {
            conn_opts.user_name(username);
        }
        if let Some(ref password) = config.password {
            conn_opts.password(password);
        }

        let conn_opts = conn_opts.finalize();

        if let Err(e) = client.connect(conn_opts).await {
            error!("[MQTT:{}] Failed to connect: {}", config.name, e);
            return;
        }

        info!("[MQTT:{}] Connected to {}:{}", config.name, config.broker_url, config.port);

        // Subscribe to topics
        if !subscribe_topics.is_empty() {
            let qos: Vec<i32> = subscribe_topics.iter().map(|_| 1).collect();
            let topics_ref: Vec<&str> = subscribe_topics.iter().map(|s| s.as_str()).collect();
            if let Err(e) = client.subscribe_many(&topics_ref, &qos).await {
                error!("[MQTT:{}] Failed to subscribe: {}", config.name, e);
            } else {
                info!("[MQTT:{}] Subscribed to {:?}", config.name, subscribe_topics);
            }
        }

        let stream = client.get_stream(100);

        while running.load(Ordering::SeqCst) {
            tokio::select! {
                msg_opt = async { stream.recv().await.ok().flatten() } => {
                    if let Some(msg) = msg_opt {
                        let fwd_msg = ForwardMessage {
                            source: MessageSource::Mqtt,
                            source_id: config_id,
                            topic: msg.topic().to_string(),
                            payload: msg.payload().to_vec(),
                        };
                        if let Err(e) = forward_tx.send(fwd_msg).await {
                            error!("[MQTT:{}] Failed to forward: {}", config.name, e);
                        }
                    }
                }
                _ = tokio::time::sleep(Duration::from_millis(10)) => {
                    while let Ok(cmd) = cmd_rx.try_recv() {
                        match cmd {
                            MqttCommand::Publish(topic, payload) => {
                                let msg = Message::new(&topic, payload, 1);
                                if let Err(e) = client.publish(msg).await {
                                    error!("[MQTT:{}] Failed to publish: {}", config.name, e);
                                }
                            }
                            MqttCommand::Subscribe(topics) => {
                                if !topics.is_empty() {
                                    let qos: Vec<i32> = topics.iter().map(|_| 1).collect();
                                    let topics_ref: Vec<&str> = topics.iter().map(|s| s.as_str()).collect();
                                    if let Err(e) = client.subscribe_many(&topics_ref, &qos).await {
                                        error!("[MQTT:{}] Failed to subscribe: {}", config.name, e);
                                    } else {
                                        info!("[MQTT:{}] Dynamically subscribed to {:?}", config.name, topics);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        let _ = client.disconnect(None).await;
        info!("[MQTT:{}] Disconnected", config.name);
    });
}

fn run_zmq_worker(
    running: Arc<AtomicBool>,
    config: ZmqConfig,
    forward_tx: mpsc::Sender<ForwardMessage>,
    cmd_rx: std::sync::mpsc::Receiver<ZmqCommand>,
) {
    use zmq::{Context, SocketType};

    let config_id = config.id.unwrap_or(0);
    let context = Context::new();

    // Create socket based on type
    let socket_type = match config.socket_type {
        ZmqSocketType::XPub => SocketType::XPUB,
        ZmqSocketType::XSub => SocketType::XSUB,
        ZmqSocketType::Pub => SocketType::PUB,
        ZmqSocketType::Sub => SocketType::SUB,
    };

    let socket = match context.socket(socket_type) {
        Ok(s) => s,
        Err(e) => {
            error!("[ZMQ:{}] Failed to create socket: {}", config.name, e);
            return;
        }
    };

    let _ = socket.set_sndhwm(config.high_water_mark as i32);
    let _ = socket.set_rcvhwm(config.high_water_mark as i32);

    // Bind or connect based on socket type
    match config.socket_type {
        ZmqSocketType::XPub | ZmqSocketType::XSub => {
            // Bind for proxy sockets
            if let Some(ref endpoint) = config.bind_endpoint {
                if let Err(e) = socket.bind(endpoint) {
                    error!("[ZMQ:{}] Failed to bind: {}", config.name, e);
                    return;
                }
                info!("[ZMQ:{}] Bound to {}", config.name, endpoint);
            }
            
            // XSUB needs to subscribe to all
            if config.socket_type == ZmqSocketType::XSub {
                let _ = socket.set_subscribe(b"");
                
                // Also connect to external publishers
                for endpoint in &config.connect_endpoints {
                    if let Err(e) = socket.connect(endpoint) {
                        warn!("[ZMQ:{}] Failed to connect to {}: {}", config.name, endpoint, e);
                    } else {
                        info!("[ZMQ:{}] Connected to {}", config.name, endpoint);
                    }
                }
            }
        }
        ZmqSocketType::Pub => {
            // Bind for publishing
            if let Some(ref endpoint) = config.bind_endpoint {
                if let Err(e) = socket.bind(endpoint) {
                    error!("[ZMQ:{}] Failed to bind: {}", config.name, e);
                    return;
                }
                info!("[ZMQ:{}] PUB bound to {}", config.name, endpoint);
            }
        }
        ZmqSocketType::Sub => {
            // Connect to publishers
            let _ = socket.set_subscribe(b"");
            for endpoint in &config.connect_endpoints {
                if let Err(e) = socket.connect(endpoint) {
                    warn!("[ZMQ:{}] Failed to connect to {}: {}", config.name, endpoint, e);
                } else {
                    info!("[ZMQ:{}] SUB connected to {}", config.name, endpoint);
                }
            }
        }
    }

    let _ = socket.set_rcvtimeo(100); // 100ms timeout

    let rt = match tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build() {
        Ok(rt) => rt,
        Err(e) => {
            error!("[ZMQ:{}] Failed to create tokio runtime: {}", config.name, e);
            return;
        }
    };

    while running.load(Ordering::SeqCst) {
        // Receive from socket (for XSUB, SUB types)
        if matches!(config.socket_type, ZmqSocketType::XSub | ZmqSocketType::Sub) {
            match socket.recv_bytes(0) {
                Ok(data) => {
                    info!("[ZMQ:{}] Received {} bytes", config.name, data.len());
                    
                    // Parse topic and payload (format: "topic payload")
                    if let Some(sep_pos) = data.iter().position(|&b| b == b' ') {
                        let topic = String::from_utf8_lossy(&data[..sep_pos]).to_string();
                        let payload = data[sep_pos + 1..].to_vec();

                        info!("[ZMQ:{}] Parsed message: topic={}, payload_len={}", config.name, topic, payload.len());

                        let fwd_msg = ForwardMessage {
                            source: MessageSource::Zmq,
                            source_id: config_id,
                            topic,
                            payload,
                        };

                        rt.block_on(async {
                            if let Err(e) = forward_tx.send(fwd_msg).await {
                                error!("[ZMQ:{}] Failed to forward: {}", config.name, e);
                            }
                        });
                    } else {
                        // No space separator - treat entire message as topic or use alternative parsing
                        warn!("[ZMQ:{}] Message has no space separator, raw: {:?}", config.name, String::from_utf8_lossy(&data));
                    }
                }
                Err(zmq::Error::EAGAIN) => {
                    // Timeout, no message
                }
                Err(e) => {
                    if running.load(Ordering::SeqCst) {
                        warn!("[ZMQ:{}] Receive error: {}", config.name, e);
                    }
                }
            }
        } else {
            // For XPUB/PUB sockets, just sleep a bit to prevent busy loop
            std::thread::sleep(std::time::Duration::from_millis(10));
        }

        // Check for commands (for all socket types that can publish: XPUB, PUB)
        if matches!(config.socket_type, ZmqSocketType::XPub | ZmqSocketType::Pub) {
            while let Ok(cmd) = cmd_rx.try_recv() {
                match cmd {
                    ZmqCommand::Publish(topic, payload) => {
                        let mut message = topic.as_bytes().to_vec();
                        message.push(b' ');
                        message.extend_from_slice(&payload);
                        
                        info!("[ZMQ:{}] Publishing to topic: {} ({} bytes)", config.name, topic, payload.len());
                        
                        match socket.send(&message, 0) {
                            Ok(_) => debug!("[ZMQ:{}] Message sent successfully", config.name),
                            Err(e) => error!("[ZMQ:{}] Failed to send: {}", config.name, e),
                        }
                    }
                }
            }
        }
    }

    info!("[ZMQ:{}] Worker stopped", config.name);
}

/// Check if topic matches pattern with MQTT wildcards
fn matches_topic_pattern(pattern: &str, topic: &str) -> bool {
    let pattern_parts: Vec<&str> = pattern.split('/').collect();
    let topic_parts: Vec<&str> = topic.split('/').collect();

    let mut p_idx = 0;
    let mut t_idx = 0;

    while p_idx < pattern_parts.len() && t_idx < topic_parts.len() {
        let p = pattern_parts[p_idx];

        if p == "#" {
            return true;
        } else if p == "+" || p == topic_parts[t_idx] {
            p_idx += 1;
            t_idx += 1;
        } else {
            return false;
        }
    }

    p_idx == pattern_parts.len() && t_idx == topic_parts.len()
        || (p_idx < pattern_parts.len() && pattern_parts[p_idx] == "#")
}

/// Apply topic mapping
fn apply_mapping(pattern: &str, target: &str, source: &str) -> String {
    if !pattern.contains('+') && !pattern.contains('#') {
        return target.to_string();
    }

    let source_parts: Vec<&str> = source.split('/').collect();
    let target_parts: Vec<&str> = target.split('/').collect();
    
    let mut result = Vec::new();
    let mut src_idx = 0;

    for part in target_parts {
        if part == "+" && src_idx < source_parts.len() {
            result.push(source_parts[src_idx].to_string());
            src_idx += 1;
        } else if part == "#" {
            while src_idx < source_parts.len() {
                result.push(source_parts[src_idx].to_string());
                src_idx += 1;
            }
        } else {
            result.push(part.to_string());
        }
    }

    if result.is_empty() {
        target.to_string()
    } else {
        result.join("/")
    }
}
