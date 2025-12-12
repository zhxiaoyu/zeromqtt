//! MQTT client wrapper using paho-mqtt

use crate::models::MqttConfig;
use paho_mqtt::{
    AsyncClient, ConnectOptionsBuilder, CreateOptionsBuilder, Message, SslOptionsBuilder,
};
use std::time::Duration;
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};

/// Message received from MQTT
#[derive(Debug, Clone)]
pub struct MqttMessage {
    pub topic: String,
    pub payload: Vec<u8>,
}

/// MQTT client wrapper
pub struct MqttClient {
    client: AsyncClient,
    config: MqttConfig,
    message_tx: mpsc::Sender<MqttMessage>,
}

impl MqttClient {
    /// Create a new MQTT client
    pub fn new(config: MqttConfig, message_tx: mpsc::Sender<MqttMessage>) -> Result<Self, paho_mqtt::Error> {
        let server_uri = if config.use_tls {
            format!("ssl://{}:{}", config.broker_url, config.port)
        } else {
            format!("tcp://{}:{}", config.broker_url, config.port)
        };

        let create_opts = CreateOptionsBuilder::new()
            .server_uri(&server_uri)
            .client_id(&config.client_id)
            .finalize();

        let client = AsyncClient::new(create_opts)?;

        Ok(Self {
            client,
            config,
            message_tx,
        })
    }

    /// Connect to the MQTT broker
    pub async fn connect(&self) -> Result<(), paho_mqtt::Error> {
        let mut conn_opts = ConnectOptionsBuilder::new();
        conn_opts
            .keep_alive_interval(Duration::from_secs(self.config.keep_alive_seconds as u64))
            .clean_session(self.config.clean_session)
            .automatic_reconnect(Duration::from_secs(1), Duration::from_secs(30));

        if let Some(ref username) = self.config.username {
            conn_opts.user_name(username);
        }
        if let Some(ref password) = self.config.password {
            conn_opts.password(password);
        }

        if self.config.use_tls {
            let ssl_opts = SslOptionsBuilder::new().finalize();
            conn_opts.ssl_options(ssl_opts);
        }

        let conn_opts = conn_opts.finalize();

        info!(
            "Connecting to MQTT broker: {}:{}",
            self.config.broker_url, self.config.port
        );

        self.client.connect(conn_opts).await?;
        info!("Connected to MQTT broker");

        Ok(())
    }

    /// Disconnect from the MQTT broker
    pub async fn disconnect(&self) -> Result<(), paho_mqtt::Error> {
        info!("Disconnecting from MQTT broker");
        self.client.disconnect(None).await?;
        Ok(())
    }

    /// Check if connected
    pub fn is_connected(&self) -> bool {
        self.client.is_connected()
    }

    /// Subscribe to topics
    pub async fn subscribe(&self, topics: &[String]) -> Result<(), paho_mqtt::Error> {
        if topics.is_empty() {
            return Ok(());
        }

        let qos: Vec<i32> = topics.iter().map(|_| 1).collect();
        let topics_ref: Vec<&str> = topics.iter().map(|s| s.as_str()).collect();

        info!("Subscribing to MQTT topics: {:?}", topics);
        self.client.subscribe_many(&topics_ref, &qos).await?;
        Ok(())
    }

    /// Unsubscribe from topics
    pub async fn unsubscribe(&self, topics: &[String]) -> Result<(), paho_mqtt::Error> {
        if topics.is_empty() {
            return Ok(());
        }

        let topics_ref: Vec<&str> = topics.iter().map(|s| s.as_str()).collect();
        self.client.unsubscribe_many(&topics_ref).await?;
        Ok(())
    }

    /// Publish a message
    pub async fn publish(&self, topic: &str, payload: &[u8], qos: i32) -> Result<(), paho_mqtt::Error> {
        let msg = Message::new(topic, payload, qos);
        self.client.publish(msg).await?;
        debug!("Published MQTT message to {}", topic);
        Ok(())
    }

    /// Start the message receiving loop
    pub fn start_receiving(&mut self) -> Result<(), paho_mqtt::Error> {
        let stream = self.client.get_stream(100);
        let tx = self.message_tx.clone();

        tokio::spawn(async move {
            let mut stream = stream;
            while let Some(msg_opt) = stream.next().await {
                if let Some(msg) = msg_opt {
                    let mqtt_msg = MqttMessage {
                        topic: msg.topic().to_string(),
                        payload: msg.payload().to_vec(),
                    };
                    if let Err(e) = tx.send(mqtt_msg).await {
                        error!("Failed to send MQTT message to channel: {}", e);
                        break;
                    }
                }
            }
            warn!("MQTT message stream ended");
        });

        Ok(())
    }

    /// Get the underlying client for advanced operations
    pub fn inner(&self) -> &AsyncClient {
        &self.client
    }
}

// Extension trait for the stream
trait StreamExt {
    async fn next(&mut self) -> Option<Option<Message>>;
}

impl StreamExt for paho_mqtt::AsyncReceiver<Option<Message>> {
    async fn next(&mut self) -> Option<Option<Message>> {
        self.recv().await.ok()
    }
}
