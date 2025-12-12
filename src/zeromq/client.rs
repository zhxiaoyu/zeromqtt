//! ZeroMQ client wrapper - supports XPUB/XSUB proxy pattern

use crate::models::{ZmqConfig, ZmqSocketType};
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

/// ZeroMQ client wrapper with XPUB/XSUB support
pub struct ZmqClient {
    context: Context,
    config: ZmqConfig,
    socket: Option<Socket>,
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
            socket: None,
            message_tx,
            running: Arc::new(parking_lot::RwLock::new(false)),
        })
    }

    /// Initialize the socket based on config type
    pub fn init_socket(&mut self) -> Result<(), zmq::Error> {
        let socket_type = match self.config.socket_type {
            ZmqSocketType::XPub => SocketType::XPUB,
            ZmqSocketType::XSub => SocketType::XSUB,
            ZmqSocketType::Pub => SocketType::PUB,
            ZmqSocketType::Sub => SocketType::SUB,
        };

        let socket = self.context.socket(socket_type)?;
        socket.set_sndhwm(self.config.high_water_mark as i32)?;
        socket.set_rcvhwm(self.config.high_water_mark as i32)?;

        // Bind or connect based on socket type
        if let Some(ref endpoint) = self.config.bind_endpoint {
            socket.bind(endpoint)?;
            info!("[ZMQ:{}] Socket bound to: {}", self.config.name, endpoint);
        }

        for endpoint in &self.config.connect_endpoints {
            socket.connect(endpoint)?;
            info!("[ZMQ:{}] Socket connected to: {}", self.config.name, endpoint);
        }

        // SUB/XSUB needs to subscribe
        if matches!(self.config.socket_type, ZmqSocketType::Sub | ZmqSocketType::XSub) {
            socket.set_subscribe(b"")?;
        }

        self.socket = Some(socket);
        Ok(())
    }

    /// Publish a message
    pub fn publish(&self, topic: &str, payload: &[u8]) -> Result<(), zmq::Error> {
        if let Some(ref socket) = self.socket {
            let mut message = topic.as_bytes().to_vec();
            message.push(b' '); // Separator
            message.extend_from_slice(payload);
            
            socket.send(&message, 0)?;
            debug!("[ZMQ:{}] Published to topic: {}", self.config.name, topic);
        } else {
            warn!("[ZMQ:{}] Socket not initialized", self.config.name);
        }
        Ok(())
    }

    /// Start the receiver in a background thread
    pub fn start_receiver(&self) -> Result<(), zmq::Error> {
        // Only start receiver for SUB/XSUB socket types
        if !matches!(self.config.socket_type, ZmqSocketType::Sub | ZmqSocketType::XSub) {
            return Ok(());
        }

        let context = self.context.clone();
        let config = self.config.clone();
        let tx = self.message_tx.clone();
        let running = self.running.clone();

        *running.write() = true;

        thread::spawn(move || {
            let socket_type = match config.socket_type {
                ZmqSocketType::XSub => SocketType::XSUB,
                _ => SocketType::SUB,
            };

            let socket = match context.socket(socket_type) {
                Ok(s) => s,
                Err(e) => {
                    error!("[ZMQ:{}] Failed to create socket: {}", config.name, e);
                    return;
                }
            };

            // Bind or connect
            if let Some(ref endpoint) = config.bind_endpoint
                && socket.bind(endpoint).is_err()
            {
                error!("[ZMQ:{}] Failed to bind", config.name);
                return;
            }

            for endpoint in &config.connect_endpoints {
                if let Err(e) = socket.connect(endpoint) {
                    warn!("[ZMQ:{}] Failed to connect to {}: {}", config.name, endpoint, e);
                }
            }

            let _ = socket.set_subscribe(b"");
            let _ = socket.set_rcvtimeo(1000);

            info!("[ZMQ:{}] Receiver started", config.name);

            while *running.read() {
                match socket.recv_bytes(0) {
                    Ok(data) => {
                        if let Some(sep_pos) = data.iter().position(|&b| b == b' ') {
                            let topic = String::from_utf8_lossy(&data[..sep_pos]).to_string();
                            let payload = data[sep_pos + 1..].to_vec();

                            let msg = ZmqMessage { topic, payload };
                            
                            let tx_clone = tx.clone();
                            let _ = tokio::runtime::Handle::try_current()
                                .map(|h| h.block_on(tx_clone.send(msg)));
                        }
                    }
                    Err(zmq::Error::EAGAIN) => {
                        continue;
                    }
                    Err(e) => {
                        if *running.read() {
                            error!("[ZMQ:{}] Receive error: {}", config.name, e);
                        }
                        break;
                    }
                }
            }

            info!("[ZMQ:{}] Receiver stopped", config.name);
        });

        Ok(())
    }

    /// Stop the receiver
    pub fn stop(&self) {
        *self.running.write() = false;
    }

    /// Check if receiver is running
    pub fn is_running(&self) -> bool {
        *self.running.read()
    }
}

impl Drop for ZmqClient {
    fn drop(&mut self) {
        self.stop();
    }
}
