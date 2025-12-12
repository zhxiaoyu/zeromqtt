//! Configuration API handlers - Multi-broker and Multi-ZMQ support

use crate::error::{AppError, AppResult};
use crate::models::{
    CreateMappingRequest, CreateMqttConfigRequest, CreateZmqConfigRequest,
    MqttConfig, TopicMapping, ZmqConfig,
};
use crate::state::AppState;
use axum::{
    extract::{Path, State},
    routing::{get, put},
    Json, Router,
};

// ============ MQTT Configs (Multiple Brokers) ============

/// Get all MQTT broker configurations
async fn get_mqtt_configs(State(state): State<AppState>) -> AppResult<Json<Vec<MqttConfig>>> {
    let configs = state
        .repo
        .get_mqtt_configs()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    Ok(Json(configs))
}

/// Get a single MQTT broker configuration by ID
async fn get_mqtt_config_by_id(
    State(state): State<AppState>,
    Path(id): Path<u32>,
) -> AppResult<Json<MqttConfig>> {
    let config = state
        .repo
        .get_mqtt_config(id)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or_else(|| AppError::NotFound(format!("MQTT config {} not found", id)))?;
    Ok(Json(config))
}

/// Add a new MQTT broker configuration
async fn add_mqtt_config(
    State(state): State<AppState>,
    Json(req): Json<CreateMqttConfigRequest>,
) -> AppResult<Json<MqttConfig>> {
    let config = state
        .repo
        .add_mqtt_config(&req)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    Ok(Json(config))
}

/// Update an existing MQTT broker configuration
async fn update_mqtt_config(
    State(state): State<AppState>,
    Path(id): Path<u32>,
    Json(req): Json<CreateMqttConfigRequest>,
) -> AppResult<Json<MqttConfig>> {
    let config = state
        .repo
        .update_mqtt_config(id, &req)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or_else(|| AppError::NotFound(format!("MQTT config {} not found", id)))?;
    Ok(Json(config))
}

/// Delete an MQTT broker configuration
async fn delete_mqtt_config(
    State(state): State<AppState>,
    Path(id): Path<u32>,
) -> AppResult<Json<serde_json::Value>> {
    let deleted = state
        .repo
        .delete_mqtt_config(id)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    if deleted {
        Ok(Json(serde_json::json!({"deleted": true, "id": id})))
    } else {
        Err(AppError::NotFound(format!(
            "MQTT config with id {} not found",
            id
        )))
    }
}

// ============ ZeroMQ Configs (XPUB/XSUB) ============

/// Get all ZeroMQ configurations
async fn get_zmq_configs(State(state): State<AppState>) -> AppResult<Json<Vec<ZmqConfig>>> {
    let configs = state
        .repo
        .get_zmq_configs()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    Ok(Json(configs))
}

/// Get a single ZMQ configuration by ID
async fn get_zmq_config_by_id(
    State(state): State<AppState>,
    Path(id): Path<u32>,
) -> AppResult<Json<ZmqConfig>> {
    let config = state
        .repo
        .get_zmq_config(id)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or_else(|| AppError::NotFound(format!("ZMQ config {} not found", id)))?;
    Ok(Json(config))
}

/// Add a new ZMQ configuration
async fn add_zmq_config(
    State(state): State<AppState>,
    Json(req): Json<CreateZmqConfigRequest>,
) -> AppResult<Json<ZmqConfig>> {
    let config = state
        .repo
        .add_zmq_config(&req)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    Ok(Json(config))
}

/// Update an existing ZMQ configuration
async fn update_zmq_config(
    State(state): State<AppState>,
    Path(id): Path<u32>,
    Json(req): Json<CreateZmqConfigRequest>,
) -> AppResult<Json<ZmqConfig>> {
    let config = state
        .repo
        .update_zmq_config(id, &req)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or_else(|| AppError::NotFound(format!("ZMQ config {} not found", id)))?;
    Ok(Json(config))
}

/// Delete a ZMQ configuration
async fn delete_zmq_config(
    State(state): State<AppState>,
    Path(id): Path<u32>,
) -> AppResult<Json<serde_json::Value>> {
    let deleted = state
        .repo
        .delete_zmq_config(id)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    if deleted {
        Ok(Json(serde_json::json!({"deleted": true, "id": id})))
    } else {
        Err(AppError::NotFound(format!(
            "ZMQ config with id {} not found",
            id
        )))
    }
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
        // MQTT configs (multiple brokers)
        .route("/mqtt", get(get_mqtt_configs).post(add_mqtt_config))
        .route(
            "/mqtt/{id}",
            get(get_mqtt_config_by_id)
                .put(update_mqtt_config)
                .delete(delete_mqtt_config),
        )
        // ZeroMQ configs (XPUB/XSUB)
        .route("/zmq", get(get_zmq_configs).post(add_zmq_config))
        .route(
            "/zmq/{id}",
            get(get_zmq_config_by_id)
                .put(update_zmq_config)
                .delete(delete_zmq_config),
        )
        // Topic mappings
        .route("/mappings", get(get_mappings).post(add_mapping))
        .route(
            "/mappings/{id}",
            put(update_mapping).delete(delete_mapping),
        )
}
