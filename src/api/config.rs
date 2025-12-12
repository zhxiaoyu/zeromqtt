//! Configuration API handlers

use crate::error::{AppError, AppResult};
use crate::mock::get_mock_store;
use crate::models::{CreateMappingRequest, MqttConfig, TopicMapping, ZmqConfig};
use axum::{
    extract::Path,
    routing::{get, put},
    Json, Router,
};

// ============ MQTT Config ============

/// Get MQTT configuration
async fn get_mqtt_config() -> Json<MqttConfig> {
    let store = get_mock_store();
    Json(store.get_mqtt_config())
}

/// Update MQTT configuration
async fn update_mqtt_config(Json(config): Json<MqttConfig>) -> Json<MqttConfig> {
    let store = get_mock_store();
    Json(store.update_mqtt_config(config))
}

// ============ ZeroMQ Config ============

/// Get ZeroMQ configuration
async fn get_zmq_config() -> Json<ZmqConfig> {
    let store = get_mock_store();
    Json(store.get_zmq_config())
}

/// Update ZeroMQ configuration
async fn update_zmq_config(Json(config): Json<ZmqConfig>) -> Json<ZmqConfig> {
    let store = get_mock_store();
    Json(store.update_zmq_config(config))
}

// ============ Topic Mappings ============

/// Get all topic mappings
async fn get_mappings() -> Json<Vec<TopicMapping>> {
    let store = get_mock_store();
    Json(store.get_mappings())
}

/// Add a new topic mapping
async fn add_mapping(Json(req): Json<CreateMappingRequest>) -> Json<TopicMapping> {
    let store = get_mock_store();
    Json(store.add_mapping(req))
}

/// Update an existing topic mapping
async fn update_mapping(
    Path(id): Path<u32>,
    Json(req): Json<CreateMappingRequest>,
) -> AppResult<Json<TopicMapping>> {
    let store = get_mock_store();
    store
        .update_mapping(id, req)
        .map(Json)
        .ok_or_else(|| AppError::NotFound(format!("Mapping with id {} not found", id)))
}

/// Delete a topic mapping
async fn delete_mapping(Path(id): Path<u32>) -> AppResult<Json<serde_json::Value>> {
    let store = get_mock_store();
    if store.delete_mapping(id) {
        Ok(Json(serde_json::json!({"deleted": true, "id": id})))
    } else {
        Err(AppError::NotFound(format!(
            "Mapping with id {} not found",
            id
        )))
    }
}

/// Create configuration routes
pub fn config_routes() -> Router {
    Router::new()
        // MQTT config
        .route("/mqtt", get(get_mqtt_config).put(update_mqtt_config))
        // ZeroMQ config
        .route("/zmq", get(get_zmq_config).put(update_zmq_config))
        // Topic mappings
        .route("/mappings", get(get_mappings).post(add_mapping))
        .route(
            "/mappings/{id}",
            put(update_mapping).delete(delete_mapping),
        )
}
