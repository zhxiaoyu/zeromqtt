//! Configuration API handlers

use crate::error::{AppError, AppResult};
use crate::models::{CreateMappingRequest, MqttConfig, TopicMapping, ZmqConfig};
use crate::state::AppState;
use axum::{
    extract::{Path, State},
    routing::{get, put},
    Json, Router,
};

// ============ MQTT Config ============

/// Get MQTT configuration
async fn get_mqtt_config(State(state): State<AppState>) -> AppResult<Json<MqttConfig>> {
    let config = state
        .repo
        .get_mqtt_config()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    Ok(Json(config))
}

/// Update MQTT configuration
async fn update_mqtt_config(
    State(state): State<AppState>,
    Json(config): Json<MqttConfig>,
) -> AppResult<Json<MqttConfig>> {
    let updated = state
        .repo
        .update_mqtt_config(&config)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    Ok(Json(updated))
}

// ============ ZeroMQ Config ============

/// Get ZeroMQ configuration
async fn get_zmq_config(State(state): State<AppState>) -> AppResult<Json<ZmqConfig>> {
    let config = state
        .repo
        .get_zmq_config()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    Ok(Json(config))
}

/// Update ZeroMQ configuration
async fn update_zmq_config(
    State(state): State<AppState>,
    Json(config): Json<ZmqConfig>,
) -> AppResult<Json<ZmqConfig>> {
    let updated = state
        .repo
        .update_zmq_config(&config)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    Ok(Json(updated))
}

// ============ Topic Mappings ============

/// Get all topic mappings
async fn get_mappings(State(state): State<AppState>) -> AppResult<Json<Vec<TopicMapping>>> {
    let mappings = state
        .repo
        .get_mappings()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    Ok(Json(mappings))
}

/// Add a new topic mapping
async fn add_mapping(
    State(state): State<AppState>,
    Json(req): Json<CreateMappingRequest>,
) -> AppResult<Json<TopicMapping>> {
    let mapping = state
        .repo
        .add_mapping(&req)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    
    // Reload mappings in bridge
    let _ = state.bridge.reload_mappings().await;
    
    Ok(Json(mapping))
}

/// Update an existing topic mapping
async fn update_mapping(
    State(state): State<AppState>,
    Path(id): Path<u32>,
    Json(req): Json<CreateMappingRequest>,
) -> AppResult<Json<TopicMapping>> {
    let mapping = state
        .repo
        .update_mapping(id, &req)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or_else(|| AppError::NotFound(format!("Mapping with id {} not found", id)))?;
    
    // Reload mappings in bridge
    let _ = state.bridge.reload_mappings().await;
    
    Ok(Json(mapping))
}

/// Delete a topic mapping
async fn delete_mapping(
    State(state): State<AppState>,
    Path(id): Path<u32>,
) -> AppResult<Json<serde_json::Value>> {
    let deleted = state
        .repo
        .delete_mapping(id)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    if deleted {
        // Reload mappings in bridge
        let _ = state.bridge.reload_mappings().await;
        Ok(Json(serde_json::json!({"deleted": true, "id": id})))
    } else {
        Err(AppError::NotFound(format!(
            "Mapping with id {} not found",
            id
        )))
    }
}

/// Create configuration routes
pub fn config_routes() -> Router<AppState> {
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
