//! ZeroMQTT - High-performance bidirectional bridge between ZeroMQ and MQTT
//!
//! This library provides a web management interface with RESTful API
//! for runtime configuration and status monitoring.

pub mod config;
pub mod models;
pub mod mock;
pub mod auth;
pub mod api;
pub mod error;
pub mod db;
pub mod mqtt;
pub mod zeromq;
pub mod bridge;
pub mod state;
pub mod telemetry;
