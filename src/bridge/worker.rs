//! Bridge worker - handles message forwarding in a dedicated thread

use crate::db::Repository;
use crate::models::{MqttConfig, ZmqConfig, TopicMapping, MappingDirection};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::{self, JoinHandle};
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};

/// Message to be forwarded
#[derive(Debug, Clone)]
pub struct ForwardMessage {
    pub source: MessageSource,
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
    mqtt_thread: Option<JoinHandle<()>>,
    zmq_thread: Option<JoinHandle<()>>,
    forward_tx: Option<mpsc::Sender<ForwardMessage>>,
}

impl BridgeWorker {
    pub fn new() -> Self {
        Self {
            running: Arc::new(AtomicBool::new(false)),
            mqtt_thread: None,
            zmq_thread: None,
            forward_tx: None,
        }
    }

    /// Start the bridge worker with given configurations
    pub fn start(
        &mut self,
        mqtt_config: MqttConfig,
        zmq_config: ZmqConfig,
        mappings: Vec<TopicMapping>,
        repo: Repository,
    ) -> Result<(), anyhow::Error> {
        if self.running.load(Ordering::SeqCst) {
            return Ok(());
        }

        self.running.store(true, Ordering::SeqCst);

        // Create channels for message forwarding
        let (forward_tx, mut forward_rx) = mpsc::channel::<ForwardMessage>(1000);
        let (mqtt_cmd_tx, mqtt_cmd_rx) = std::sync::mpsc::channel::<MqttCommand>();
        let (zmq_cmd_tx, zmq_cmd_rx) = std::sync::mpsc::channel::<ZmqCommand>();

        self.forward_tx = Some(forward_tx.clone());

        // Clone mappings for each direction
        let mqtt_to_zmq_mappings: Vec<_> = mappings
            .iter()
            .filter(|m| m.enabled && (m.direction == MappingDirection::MqttToZmq || m.direction == MappingDirection::Bidirectional))
            .cloned()
            .collect();

        let zmq_to_mqtt_mappings: Vec<_> = mappings
            .iter()
            .filter(|m| m.enabled && (m.direction == MappingDirection::ZmqToMqtt || m.direction == MappingDirection::Bidirectional))
            .cloned()
            .collect();

        // MQTT subscribe topics
        let mqtt_subscribe_topics: Vec<String> = mqtt_to_zmq_mappings
            .iter()
            .map(|m| m.source_topic.clone())
            .collect();

        // Start MQTT thread
        let running_mqtt = self.running.clone();
        let forward_tx_mqtt = forward_tx.clone();
        let mqtt_config_clone = mqtt_config.clone();

        let mqtt_thread = thread::spawn(move || {
            run_mqtt_worker(
                running_mqtt,
                mqtt_config_clone,
                mqtt_subscribe_topics,
                forward_tx_mqtt,
                mqtt_cmd_rx,
            );
        });

        // Start ZMQ thread
        let running_zmq = self.running.clone();
        let zmq_config_clone = zmq_config.clone();

        let zmq_thread = thread::spawn(move || {
            run_zmq_worker(
                running_zmq,
                zmq_config_clone,
                forward_tx.clone(),
                zmq_cmd_rx,
            );
        });

        self.mqtt_thread = Some(mqtt_thread);
        self.zmq_thread = Some(zmq_thread);

        // Start forwarding task
        let running_fwd = self.running.clone();
        let repo_fwd = repo.clone();

        tokio::spawn(async move {
            while running_fwd.load(Ordering::SeqCst) {
                tokio::select! {
                    Some(msg) = forward_rx.recv() => {
                        match msg.source {
                            MessageSource::Mqtt => {
                                // Forward MQTT -> ZMQ
                                if let Some(target) = find_target_topic(&msg.topic, &mqtt_to_zmq_mappings) {
                                    debug!("Forwarding MQTT->ZMQ: {} -> {}", msg.topic, target);
                                    let _ = zmq_cmd_tx.send(ZmqCommand::Publish(target, msg.payload));
                                    let _ = repo_fwd.increment_stats(1, 0, 0, 1, 0).await;
                                }
                            }
                            MessageSource::Zmq => {
                                // Forward ZMQ -> MQTT
                                if let Some(target) = find_target_topic(&msg.topic, &zmq_to_mqtt_mappings) {
                                    debug!("Forwarding ZMQ->MQTT: {} -> {}", msg.topic, target);
                                    let _ = mqtt_cmd_tx.send(MqttCommand::Publish(target, msg.payload));
                                    let _ = repo_fwd.increment_stats(0, 1, 1, 0, 0).await;
                                }
                            }
                        }
                    }
                    else => {
                        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                    }
                }
            }
        });

        info!("Bridge worker started");
        Ok(())
    }

    /// Stop the bridge worker
    pub fn stop(&mut self) {
        self.running.store(false, Ordering::SeqCst);
        
        // Wait for threads to finish
        if let Some(handle) = self.mqtt_thread.take() {
            let _ = handle.join();
        }
        if let Some(handle) = self.zmq_thread.take() {
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
            error!("Failed to create MQTT client: {}", e);
            return;
        }
    };

    // Create a runtime for this thread
    let rt = match tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build() {
        Ok(rt) => rt,
        Err(e) => {
            error!("Failed to create tokio runtime: {}", e);
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
            error!("Failed to connect to MQTT broker: {}", e);
            return;
        }

        info!("Connected to MQTT broker: {}:{}", config.broker_url, config.port);

        // Subscribe to topics
        if !subscribe_topics.is_empty() {
            let qos: Vec<i32> = subscribe_topics.iter().map(|_| 1).collect();
            let topics_ref: Vec<&str> = subscribe_topics.iter().map(|s| s.as_str()).collect();
            if let Err(e) = client.subscribe_many(&topics_ref, &qos).await {
                error!("Failed to subscribe to MQTT topics: {}", e);
            } else {
                info!("Subscribed to MQTT topics: {:?}", subscribe_topics);
            }
        }

        // Get message stream
        let stream = client.get_stream(100);

        while running.load(Ordering::SeqCst) {
            // Check for incoming messages
            tokio::select! {
                msg_opt = async { stream.recv().await.ok().flatten() } => {
                    if let Some(msg) = msg_opt {
                        let fwd_msg = ForwardMessage {
                            source: MessageSource::Mqtt,
                            topic: msg.topic().to_string(),
                            payload: msg.payload().to_vec(),
                        };
                        if let Err(e) = forward_tx.send(fwd_msg).await {
                            error!("Failed to forward MQTT message: {}", e);
                        }
                    }
                }
                _ = tokio::time::sleep(Duration::from_millis(10)) => {
                    // Check for commands
                    while let Ok(cmd) = cmd_rx.try_recv() {
                        match cmd {
                            MqttCommand::Publish(topic, payload) => {
                                let msg = Message::new(&topic, payload, 1);
                                if let Err(e) = client.publish(msg).await {
                                    error!("Failed to publish MQTT message: {}", e);
                                }
                            }
                        }
                    }
                }
            }
        }

        let _ = client.disconnect(None).await;
        info!("MQTT worker stopped");
    });
}

fn run_zmq_worker(
    running: Arc<AtomicBool>,
    config: ZmqConfig,
    forward_tx: mpsc::Sender<ForwardMessage>,
    cmd_rx: std::sync::mpsc::Receiver<ZmqCommand>,
) {
    use zmq::{Context, SocketType};

    let context = Context::new();

    // Create PUB socket
    let pub_socket = match context.socket(SocketType::PUB) {
        Ok(s) => s,
        Err(e) => {
            error!("Failed to create ZMQ PUB socket: {}", e);
            return;
        }
    };

    if let Err(e) = pub_socket.bind(&config.pub_endpoint) {
        error!("Failed to bind ZMQ PUB socket: {}", e);
        return;
    }
    let _ = pub_socket.set_sndhwm(config.high_water_mark as i32);
    info!("ZMQ PUB socket bound to: {}", config.pub_endpoint);

    // Create SUB socket
    let sub_socket = match context.socket(SocketType::SUB) {
        Ok(s) => s,
        Err(e) => {
            error!("Failed to create ZMQ SUB socket: {}", e);
            return;
        }
    };

    if let Err(e) = sub_socket.connect(&config.sub_endpoint) {
        error!("Failed to connect ZMQ SUB socket: {}", e);
        return;
    }
    let _ = sub_socket.set_subscribe(b"");
    let _ = sub_socket.set_rcvhwm(config.high_water_mark as i32);
    let _ = sub_socket.set_rcvtimeo(100); // 100ms timeout
    info!("ZMQ SUB socket connected to: {}", config.sub_endpoint);

    // Create a runtime for sending to async channel
    let rt = match tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build() {
        Ok(rt) => rt,
        Err(e) => {
            error!("Failed to create tokio runtime: {}", e);
            return;
        }
    };

    while running.load(Ordering::SeqCst) {
        // Receive from SUB socket
        match sub_socket.recv_bytes(0) {
            Ok(data) => {
                // Parse topic and payload (space-separated)
                if let Some(sep_pos) = data.iter().position(|&b| b == b' ') {
                    let topic = String::from_utf8_lossy(&data[..sep_pos]).to_string();
                    let payload = data[sep_pos + 1..].to_vec();

                    let fwd_msg = ForwardMessage {
                        source: MessageSource::Zmq,
                        topic,
                        payload,
                    };

                    rt.block_on(async {
                        if let Err(e) = forward_tx.send(fwd_msg).await {
                            error!("Failed to forward ZMQ message: {}", e);
                        }
                    });
                }
            }
            Err(zmq::Error::EAGAIN) => {
                // Timeout, no message
            }
            Err(e) => {
                if running.load(Ordering::SeqCst) {
                    warn!("ZMQ receive error: {}", e);
                }
            }
        }

        // Check for commands
        while let Ok(cmd) = cmd_rx.try_recv() {
            match cmd {
                ZmqCommand::Publish(topic, payload) => {
                    let mut message = topic.as_bytes().to_vec();
                    message.push(b' ');
                    message.extend_from_slice(&payload);
                    if let Err(e) = pub_socket.send(&message, 0) {
                        error!("Failed to send ZMQ message: {}", e);
                    }
                }
            }
        }
    }

    info!("ZMQ worker stopped");
}

/// Find target topic based on mappings
fn find_target_topic(source: &str, mappings: &[TopicMapping]) -> Option<String> {
    for mapping in mappings {
        if matches_topic_pattern(&mapping.source_topic, source) {
            return Some(apply_mapping(&mapping.source_topic, &mapping.target_topic, source));
        }
    }
    None
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

    // Simple replacement for now
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
