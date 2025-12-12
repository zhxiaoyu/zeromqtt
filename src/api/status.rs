//! Status API handlers

use crate::error::{AppError, AppResult};
use crate::models::{BridgeStatus, ChartData, MessageStats, TimeSeriesPoint};
use crate::state::AppState;
use axum::{extract::State, routing::get, Json, Router};
use rand::Rng;

/// Get bridge status
async fn get_status(State(state): State<AppState>) -> Json<BridgeStatus> {
    let status = state.bridge.get_status().await;
    Json(status)
}

/// Get message statistics
async fn get_stats(State(state): State<AppState>) -> AppResult<Json<MessageStats>> {
    let mut stats = state
        .repo
        .get_stats()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    // Calculate runtime values
    let start_time = state
        .repo
        .get_start_time()
        .await
        .unwrap_or(chrono::Utc::now().timestamp());
    let elapsed = (chrono::Utc::now().timestamp() - start_time) as f64;
    if elapsed > 0.0 {
        let total_messages =
            stats.mqtt_received + stats.mqtt_sent + stats.zmq_received + stats.zmq_sent;
        stats.messages_per_second = total_messages as f64 / elapsed;
    }

    // Simulate latency (would be calculated from actual measurements in production)
    stats.avg_latency_ms = rand::thread_rng().gen_range(0.5..2.0);

    Ok(Json(stats))
}

/// Get throughput chart data (simulated for now - would track real data in production)
async fn get_chart_data(State(state): State<AppState>) -> AppResult<Json<Vec<ChartData>>> {
    let stats = state
        .repo
        .get_stats()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    // Generate simulated time series based on current counts
    let now = chrono::Utc::now().timestamp();
    let mut rng = rand::thread_rng();

    let base_mqtt = (stats.mqtt_received + stats.mqtt_sent) as f64 / 60.0;
    let base_zmq = (stats.zmq_received + stats.zmq_sent) as f64 / 60.0;

    let mqtt_data: Vec<TimeSeriesPoint> = (0..60)
        .map(|i| TimeSeriesPoint {
            timestamp: now - (60 - i) * 1000,
            value: (base_mqtt * rng.gen_range(0.8..1.2)).max(0.0),
        })
        .collect();

    let zmq_data: Vec<TimeSeriesPoint> = (0..60)
        .map(|i| TimeSeriesPoint {
            timestamp: now - (60 - i) * 1000,
            value: (base_zmq * rng.gen_range(0.8..1.2)).max(0.0),
        })
        .collect();

    Ok(Json(vec![
        ChartData {
            label: "MQTT".to_string(),
            data: mqtt_data,
        },
        ChartData {
            label: "ZeroMQ".to_string(),
            data: zmq_data,
        },
    ]))
}

/// Create status routes
pub fn status_routes() -> Router<AppState> {
    Router::new()
        .route("/", get(get_status))
        .route("/stats", get(get_stats))
        .route("/chart", get(get_chart_data))
}
