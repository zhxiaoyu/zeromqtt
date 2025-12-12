//! ZeroMQ client wrapper

use crate::models::ZmqConfig;
use std::sync::Arc;
use std::thread;
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};
use zmq::{Context, Socket, SocketType};

/// Message received from ZeroMQ
#[derive(Debug, Clone)]
pub struct ZmqMessage {
    pub topic: String,
    pub payload: Vec<u8>,
}

/// ZeroMQ client wrapper
pub struct ZmqClient {
    context: Context,
    config: ZmqConfig,
    pub_socket: Option<Socket>,
    message_tx: mpsc::Sender<ZmqMessage>,
    running: Arc<parking_lot::RwLock<bool>>,
}

impl ZmqClient {
    /// Create a new ZeroMQ client
    pub fn new(config: ZmqConfig, message_tx: mpsc::Sender<ZmqMessage>) -> Result<Self, zmq::Error> {
        let context = Context::new();

        Ok(Self {
            context,
            config,
            pub_socket: None,
            message_tx,
            running: Arc::new(parking_lot::RwLock::new(false)),
        })
    }

    /// Initialize the publisher socket
    pub fn init_publisher(&mut self) -> Result<(), zmq::Error> {
        let socket = self.context.socket(SocketType::PUB)?;
        socket.bind(&self.config.pub_endpoint)?;
        socket.set_sndhwm(self.config.high_water_mark as i32)?;

        info!("ZeroMQ PUB socket bound to: {}", self.config.pub_endpoint);
        self.pub_socket = Some(socket);
        Ok(())
    }

    /// Publish a message
    pub fn publish(&self, topic: &str, payload: &[u8]) -> Result<(), zmq::Error> {
        if let Some(ref socket) = self.pub_socket {
            // ZMQ PUB/SUB uses topic prefix filtering
            let mut message = topic.as_bytes().to_vec();
            message.push(b' '); // Separator
            message.extend_from_slice(payload);
            
            socket.send(&message, 0)?;
            debug!("Published ZMQ message to topic: {}", topic);
        } else {
            warn!("ZMQ publisher not initialized");
        }
        Ok(())
    }

    /// Start the subscriber in a background thread
    pub fn start_subscriber(&self) -> Result<(), zmq::Error> {
        let context = self.context.clone();
        let endpoint = self.config.sub_endpoint.clone();
        let hwm = self.config.high_water_mark as i32;
        let tx = self.message_tx.clone();
        let running = self.running.clone();

        *running.write() = true;

        thread::spawn(move || {
            let socket = match context.socket(SocketType::SUB) {
                Ok(s) => s,
                Err(e) => {
                    error!("Failed to create ZMQ SUB socket: {}", e);
                    return;
                }
            };

            if let Err(e) = socket.connect(&endpoint) {
                error!("Failed to connect ZMQ SUB socket to {}: {}", endpoint, e);
                return;
            }

            // Subscribe to all messages
            if let Err(e) = socket.set_subscribe(b"") {
                error!("Failed to set ZMQ subscription: {}", e);
                return;
            }

            if let Err(e) = socket.set_rcvhwm(hwm) {
                warn!("Failed to set ZMQ receive high water mark: {}", e);
            }

            info!("ZeroMQ SUB socket connected to: {}", endpoint);

            // Set receive timeout for graceful shutdown
            let _ = socket.set_rcvtimeo(1000);

            while *running.read() {
                match socket.recv_bytes(0) {
                    Ok(data) => {
                        // Parse topic and payload (separated by space)
                        if let Some(sep_pos) = data.iter().position(|&b| b == b' ') {
                            let topic = String::from_utf8_lossy(&data[..sep_pos]).to_string();
                            let payload = data[sep_pos + 1..].to_vec();

                            let msg = ZmqMessage { topic, payload };
                            
                            // Use blocking send since we're in a std thread
                            let tx_clone = tx.clone();
                            let _ = tokio::runtime::Handle::try_current()
                                .map(|h| h.block_on(tx_clone.send(msg)));
                        }
                    }
                    Err(zmq::Error::EAGAIN) => {
                        // Timeout, continue loop
                        continue;
                    }
                    Err(e) => {
                        if *running.read() {
                            error!("ZMQ receive error: {}", e);
                        }
                        break;
                    }
                }
            }

            info!("ZeroMQ subscriber stopped");
        });

        Ok(())
    }

    /// Stop the subscriber
    pub fn stop(&self) {
        *self.running.write() = false;
    }

    /// Check if subscriber is running
    pub fn is_running(&self) -> bool {
        *self.running.read()
    }
}

impl Drop for ZmqClient {
    fn drop(&mut self) {
        self.stop();
    }
}
