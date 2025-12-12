//! Application configuration module

use serde::{Deserialize, Serialize};

/// JWT configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JwtConfig {
    /// Secret key for signing tokens
    pub secret: String,
    /// Token expiration time in hours
    pub expiration_hours: i64,
}

impl Default for JwtConfig {
    fn default() -> Self {
        Self {
            secret: "zeromqtt-super-secret-key-change-in-production".to_string(),
            expiration_hours: 24,
        }
    }
}

/// Default user credentials
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefaultCredentials {
    pub username: String,
    pub password: String,
}

impl Default for DefaultCredentials {
    fn default() -> Self {
        Self {
            username: "zeromqtt".to_string(),
            password: "zeromqtt".to_string(),
        }
    }
}

/// Server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: "0.0.0.0".to_string(),
            port: 3000,
        }
    }
}

/// Application configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub jwt: JwtConfig,
    pub credentials: DefaultCredentials,
}

impl AppConfig {
    /// Create a new configuration with defaults
    pub fn new() -> Self {
        Self::default()
    }
}
