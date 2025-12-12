//! Repository implementations for database access

use crate::models::{
    BridgeState, BridgeStatus, ConnectionStatus, CreateMappingRequest, MappingDirection,
    MessageStats, MqttConfig, TopicMapping, ZmqConfig,
};
use sqlx::sqlite::SqlitePool;
use sqlx::FromRow;

// ============ Row Types for SQLite ============

#[derive(FromRow)]
struct MqttConfigRow {
    id: i64,
    broker_url: String,
    port: i64,
    client_id: String,
    username: Option<String>,
    password: Option<String>,
    use_tls: i64,
    keep_alive_seconds: i64,
    clean_session: i64,
}

impl From<MqttConfigRow> for MqttConfig {
    fn from(row: MqttConfigRow) -> Self {
        MqttConfig {
            id: Some(row.id as u32),
            broker_url: row.broker_url,
            port: row.port as u16,
            client_id: row.client_id,
            username: row.username,
            password: row.password,
            use_tls: row.use_tls != 0,
            keep_alive_seconds: row.keep_alive_seconds as u16,
            clean_session: row.clean_session != 0,
        }
    }
}

#[derive(FromRow)]
struct ZmqConfigRow {
    id: i64,
    pub_endpoint: String,
    sub_endpoint: String,
    high_water_mark: i64,
    reconnect_interval_ms: i64,
}

impl From<ZmqConfigRow> for ZmqConfig {
    fn from(row: ZmqConfigRow) -> Self {
        ZmqConfig {
            id: Some(row.id as u32),
            pub_endpoint: row.pub_endpoint,
            sub_endpoint: row.sub_endpoint,
            high_water_mark: row.high_water_mark as u32,
            reconnect_interval_ms: row.reconnect_interval_ms as u32,
        }
    }
}

#[derive(FromRow)]
struct TopicMappingRow {
    id: i64,
    source_topic: String,
    target_topic: String,
    direction: String,
    enabled: i64,
    description: Option<String>,
}

impl From<TopicMappingRow> for TopicMapping {
    fn from(row: TopicMappingRow) -> Self {
        let direction = match row.direction.as_str() {
            "zmq_to_mqtt" => MappingDirection::ZmqToMqtt,
            "bidirectional" => MappingDirection::Bidirectional,
            _ => MappingDirection::MqttToZmq,
        };
        TopicMapping {
            id: row.id as u32,
            source_topic: row.source_topic,
            target_topic: row.target_topic,
            direction,
            enabled: row.enabled != 0,
            description: row.description,
        }
    }
}

#[derive(FromRow)]
struct MessageStatsRow {
    mqtt_received: i64,
    mqtt_sent: i64,
    zmq_received: i64,
    zmq_sent: i64,
    error_count: i64,
    start_time: i64,
}

// ============ Repository ============

/// Database repository for all data access
#[derive(Clone)]
pub struct Repository {
    pool: SqlitePool,
}

impl Repository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    // ============ MQTT Config ============

    pub async fn get_mqtt_config(&self) -> Result<MqttConfig, sqlx::Error> {
        let row: MqttConfigRow = sqlx::query_as("SELECT * FROM mqtt_config WHERE id = 1")
            .fetch_one(&self.pool)
            .await?;
        Ok(row.into())
    }

    pub async fn update_mqtt_config(&self, config: &MqttConfig) -> Result<MqttConfig, sqlx::Error> {
        sqlx::query(
            r#"
            UPDATE mqtt_config SET
                broker_url = ?,
                port = ?,
                client_id = ?,
                username = ?,
                password = ?,
                use_tls = ?,
                keep_alive_seconds = ?,
                clean_session = ?
            WHERE id = 1
            "#,
        )
        .bind(&config.broker_url)
        .bind(config.port as i64)
        .bind(&config.client_id)
        .bind(&config.username)
        .bind(&config.password)
        .bind(if config.use_tls { 1i64 } else { 0i64 })
        .bind(config.keep_alive_seconds as i64)
        .bind(if config.clean_session { 1i64 } else { 0i64 })
        .execute(&self.pool)
        .await?;

        self.get_mqtt_config().await
    }

    // ============ ZMQ Config ============

    pub async fn get_zmq_config(&self) -> Result<ZmqConfig, sqlx::Error> {
        let row: ZmqConfigRow = sqlx::query_as("SELECT * FROM zmq_config WHERE id = 1")
            .fetch_one(&self.pool)
            .await?;
        Ok(row.into())
    }

    pub async fn update_zmq_config(&self, config: &ZmqConfig) -> Result<ZmqConfig, sqlx::Error> {
        sqlx::query(
            r#"
            UPDATE zmq_config SET
                pub_endpoint = ?,
                sub_endpoint = ?,
                high_water_mark = ?,
                reconnect_interval_ms = ?
            WHERE id = 1
            "#,
        )
        .bind(&config.pub_endpoint)
        .bind(&config.sub_endpoint)
        .bind(config.high_water_mark as i64)
        .bind(config.reconnect_interval_ms as i64)
        .execute(&self.pool)
        .await?;

        self.get_zmq_config().await
    }

    // ============ Topic Mappings ============

    pub async fn get_mappings(&self) -> Result<Vec<TopicMapping>, sqlx::Error> {
        let rows: Vec<TopicMappingRow> =
            sqlx::query_as("SELECT * FROM topic_mappings ORDER BY id")
                .fetch_all(&self.pool)
                .await?;
        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn add_mapping(&self, req: &CreateMappingRequest) -> Result<TopicMapping, sqlx::Error> {
        let direction = match req.direction {
            MappingDirection::MqttToZmq => "mqtt_to_zmq",
            MappingDirection::ZmqToMqtt => "zmq_to_mqtt",
            MappingDirection::Bidirectional => "bidirectional",
        };

        let result = sqlx::query(
            r#"
            INSERT INTO topic_mappings (source_topic, target_topic, direction, enabled, description)
            VALUES (?, ?, ?, ?, ?)
            "#,
        )
        .bind(&req.source_topic)
        .bind(&req.target_topic)
        .bind(direction)
        .bind(if req.enabled { 1i64 } else { 0i64 })
        .bind(&req.description)
        .execute(&self.pool)
        .await?;

        let id = result.last_insert_rowid() as u32;
        Ok(TopicMapping {
            id,
            source_topic: req.source_topic.clone(),
            target_topic: req.target_topic.clone(),
            direction: req.direction.clone(),
            enabled: req.enabled,
            description: req.description.clone(),
        })
    }

    pub async fn update_mapping(
        &self,
        id: u32,
        req: &CreateMappingRequest,
    ) -> Result<Option<TopicMapping>, sqlx::Error> {
        let direction = match req.direction {
            MappingDirection::MqttToZmq => "mqtt_to_zmq",
            MappingDirection::ZmqToMqtt => "zmq_to_mqtt",
            MappingDirection::Bidirectional => "bidirectional",
        };

        let result = sqlx::query(
            r#"
            UPDATE topic_mappings SET
                source_topic = ?,
                target_topic = ?,
                direction = ?,
                enabled = ?,
                description = ?
            WHERE id = ?
            "#,
        )
        .bind(&req.source_topic)
        .bind(&req.target_topic)
        .bind(direction)
        .bind(if req.enabled { 1i64 } else { 0i64 })
        .bind(&req.description)
        .bind(id as i64)
        .execute(&self.pool)
        .await?;

        if result.rows_affected() > 0 {
            Ok(Some(TopicMapping {
                id,
                source_topic: req.source_topic.clone(),
                target_topic: req.target_topic.clone(),
                direction: req.direction.clone(),
                enabled: req.enabled,
                description: req.description.clone(),
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn delete_mapping(&self, id: u32) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM topic_mappings WHERE id = ?")
            .bind(id as i64)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    // ============ Message Stats ============

    pub async fn get_stats(&self) -> Result<MessageStats, sqlx::Error> {
        let row: MessageStatsRow = sqlx::query_as("SELECT * FROM message_stats WHERE id = 1")
            .fetch_one(&self.pool)
            .await?;

        Ok(MessageStats {
            mqtt_received: row.mqtt_received as u64,
            mqtt_sent: row.mqtt_sent as u64,
            zmq_received: row.zmq_received as u64,
            zmq_sent: row.zmq_sent as u64,
            messages_per_second: 0.0, // Calculated at runtime
            avg_latency_ms: 0.0,      // Calculated at runtime
            error_count: row.error_count as u64,
            queue_depth: 0, // Runtime value
        })
    }

    pub async fn increment_stats(
        &self,
        mqtt_received: i64,
        mqtt_sent: i64,
        zmq_received: i64,
        zmq_sent: i64,
        errors: i64,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            UPDATE message_stats SET
                mqtt_received = mqtt_received + ?,
                mqtt_sent = mqtt_sent + ?,
                zmq_received = zmq_received + ?,
                zmq_sent = zmq_sent + ?,
                error_count = error_count + ?
            WHERE id = 1
            "#,
        )
        .bind(mqtt_received)
        .bind(mqtt_sent)
        .bind(zmq_received)
        .bind(zmq_sent)
        .bind(errors)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn get_start_time(&self) -> Result<i64, sqlx::Error> {
        let row: (i64,) = sqlx::query_as("SELECT start_time FROM message_stats WHERE id = 1")
            .fetch_one(&self.pool)
            .await?;
        Ok(row.0)
    }

    pub async fn reset_stats(&self) -> Result<(), sqlx::Error> {
        let now = chrono::Utc::now().timestamp();
        sqlx::query(
            r#"
            UPDATE message_stats SET
                mqtt_received = 0,
                mqtt_sent = 0,
                zmq_received = 0,
                zmq_sent = 0,
                error_count = 0,
                start_time = ?
            WHERE id = 1
            "#,
        )
        .bind(now)
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}
