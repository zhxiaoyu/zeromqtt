//! Repository implementations for database access

use crate::models::{
    CreateMappingRequest, CreateMqttConfigRequest, CreateZmqConfigRequest,
    EndpointType, MappingDirection, MessageStats, MqttConfig, TopicMapping,
    ZmqConfig, ZmqSocketType,
};
use sqlx::sqlite::SqlitePool;
use sqlx::FromRow;

// ============ Row Types for SQLite ============

#[derive(FromRow)]
#[allow(dead_code)]
struct MqttConfigRow {
    id: i64,
    name: String,
    enabled: i64,
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
            name: row.name,
            enabled: row.enabled != 0,
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
#[allow(dead_code)]
struct ZmqConfigRow {
    id: i64,
    name: String,
    enabled: i64,
    socket_type: String,
    bind_endpoint: Option<String>,
    connect_endpoints: Option<String>,
    high_water_mark: i64,
    reconnect_interval_ms: i64,
}

impl From<ZmqConfigRow> for ZmqConfig {
    fn from(row: ZmqConfigRow) -> Self {
        let socket_type = match row.socket_type.as_str() {
            "xpub" => ZmqSocketType::XPub,
            "xsub" => ZmqSocketType::XSub,
            "pub" => ZmqSocketType::Pub,
            "sub" => ZmqSocketType::Sub,
            _ => ZmqSocketType::XPub,
        };
        
        let connect_endpoints: Vec<String> = row.connect_endpoints
            .map(|s| s.split(',').filter(|s| !s.is_empty()).map(|s| s.to_string()).collect())
            .unwrap_or_default();

        ZmqConfig {
            id: Some(row.id as u32),
            name: row.name,
            enabled: row.enabled != 0,
            socket_type,
            bind_endpoint: row.bind_endpoint,
            connect_endpoints,
            high_water_mark: row.high_water_mark as u32,
            reconnect_interval_ms: row.reconnect_interval_ms as u32,
        }
    }
}

#[derive(FromRow)]
#[allow(dead_code)]
struct TopicMappingRow {
    id: i64,
    source_endpoint_type: String,
    source_endpoint_id: i64,
    target_endpoint_type: String,
    target_endpoint_id: i64,
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
            "mqtt_to_mqtt" => MappingDirection::MqttToMqtt,
            "zmq_to_zmq" => MappingDirection::ZmqToZmq,
            "bidirectional" => MappingDirection::Bidirectional,
            _ => MappingDirection::MqttToZmq,
        };
        
        let source_endpoint_type = match row.source_endpoint_type.as_str() {
            "zmq" => EndpointType::Zmq,
            _ => EndpointType::Mqtt,
        };
        
        let target_endpoint_type = match row.target_endpoint_type.as_str() {
            "zmq" => EndpointType::Zmq,
            _ => EndpointType::Mqtt,
        };

        TopicMapping {
            id: row.id as u32,
            source_endpoint_type,
            source_endpoint_id: row.source_endpoint_id as u32,
            target_endpoint_type,
            target_endpoint_id: row.target_endpoint_id as u32,
            source_topic: row.source_topic,
            target_topic: row.target_topic,
            direction,
            enabled: row.enabled != 0,
            description: row.description,
        }
    }
}

#[derive(FromRow)]
#[allow(dead_code)]
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

    // ============ MQTT Configs (Multiple Brokers) ============

    pub async fn get_mqtt_configs(&self) -> Result<Vec<MqttConfig>, sqlx::Error> {
        let rows: Vec<MqttConfigRow> = sqlx::query_as("SELECT * FROM mqtt_configs ORDER BY id")
            .fetch_all(&self.pool)
            .await?;
        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn get_mqtt_config(&self, id: u32) -> Result<Option<MqttConfig>, sqlx::Error> {
        let row: Option<MqttConfigRow> = sqlx::query_as("SELECT * FROM mqtt_configs WHERE id = ?")
            .bind(id as i64)
            .fetch_optional(&self.pool)
            .await?;
        Ok(row.map(|r| r.into()))
    }

    pub async fn add_mqtt_config(&self, req: &CreateMqttConfigRequest) -> Result<MqttConfig, sqlx::Error> {
        let result = sqlx::query(
            r#"
            INSERT INTO mqtt_configs (name, enabled, broker_url, port, client_id, username, password, use_tls, keep_alive_seconds, clean_session)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&req.name)
        .bind(if req.enabled { 1i64 } else { 0i64 })
        .bind(&req.broker_url)
        .bind(req.port as i64)
        .bind(&req.client_id)
        .bind(&req.username)
        .bind(&req.password)
        .bind(if req.use_tls { 1i64 } else { 0i64 })
        .bind(req.keep_alive_seconds as i64)
        .bind(if req.clean_session { 1i64 } else { 0i64 })
        .execute(&self.pool)
        .await?;

        let id = result.last_insert_rowid() as u32;
        Ok(MqttConfig {
            id: Some(id),
            name: req.name.clone(),
            enabled: req.enabled,
            broker_url: req.broker_url.clone(),
            port: req.port,
            client_id: req.client_id.clone(),
            username: req.username.clone(),
            password: req.password.clone(),
            use_tls: req.use_tls,
            keep_alive_seconds: req.keep_alive_seconds,
            clean_session: req.clean_session,
        })
    }

    pub async fn update_mqtt_config(&self, id: u32, req: &CreateMqttConfigRequest) -> Result<Option<MqttConfig>, sqlx::Error> {
        let result = sqlx::query(
            r#"
            UPDATE mqtt_configs SET
                name = ?, enabled = ?, broker_url = ?, port = ?, client_id = ?,
                username = ?, password = ?, use_tls = ?, keep_alive_seconds = ?, clean_session = ?
            WHERE id = ?
            "#,
        )
        .bind(&req.name)
        .bind(if req.enabled { 1i64 } else { 0i64 })
        .bind(&req.broker_url)
        .bind(req.port as i64)
        .bind(&req.client_id)
        .bind(&req.username)
        .bind(&req.password)
        .bind(if req.use_tls { 1i64 } else { 0i64 })
        .bind(req.keep_alive_seconds as i64)
        .bind(if req.clean_session { 1i64 } else { 0i64 })
        .bind(id as i64)
        .execute(&self.pool)
        .await?;

        if result.rows_affected() > 0 {
            self.get_mqtt_config(id).await
        } else {
            Ok(None)
        }
    }

    pub async fn delete_mqtt_config(&self, id: u32) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM mqtt_configs WHERE id = ?")
            .bind(id as i64)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    // ============ ZMQ Configs (XPUB/XSUB) ============

    pub async fn get_zmq_configs(&self) -> Result<Vec<ZmqConfig>, sqlx::Error> {
        let rows: Vec<ZmqConfigRow> = sqlx::query_as("SELECT * FROM zmq_configs ORDER BY id")
            .fetch_all(&self.pool)
            .await?;
        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn get_zmq_config(&self, id: u32) -> Result<Option<ZmqConfig>, sqlx::Error> {
        let row: Option<ZmqConfigRow> = sqlx::query_as("SELECT * FROM zmq_configs WHERE id = ?")
            .bind(id as i64)
            .fetch_optional(&self.pool)
            .await?;
        Ok(row.map(|r| r.into()))
    }

    pub async fn add_zmq_config(&self, req: &CreateZmqConfigRequest) -> Result<ZmqConfig, sqlx::Error> {
        let socket_type = match req.socket_type {
            ZmqSocketType::XPub => "xpub",
            ZmqSocketType::XSub => "xsub",
            ZmqSocketType::Pub => "pub",
            ZmqSocketType::Sub => "sub",
        };
        
        let connect_endpoints = req.connect_endpoints.join(",");

        let result = sqlx::query(
            r#"
            INSERT INTO zmq_configs (name, enabled, socket_type, bind_endpoint, connect_endpoints, high_water_mark, reconnect_interval_ms)
            VALUES (?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&req.name)
        .bind(if req.enabled { 1i64 } else { 0i64 })
        .bind(socket_type)
        .bind(&req.bind_endpoint)
        .bind(&connect_endpoints)
        .bind(req.high_water_mark as i64)
        .bind(req.reconnect_interval_ms as i64)
        .execute(&self.pool)
        .await?;

        let id = result.last_insert_rowid() as u32;
        Ok(ZmqConfig {
            id: Some(id),
            name: req.name.clone(),
            enabled: req.enabled,
            socket_type: req.socket_type.clone(),
            bind_endpoint: req.bind_endpoint.clone(),
            connect_endpoints: req.connect_endpoints.clone(),
            high_water_mark: req.high_water_mark,
            reconnect_interval_ms: req.reconnect_interval_ms,
        })
    }

    pub async fn update_zmq_config(&self, id: u32, req: &CreateZmqConfigRequest) -> Result<Option<ZmqConfig>, sqlx::Error> {
        let socket_type = match req.socket_type {
            ZmqSocketType::XPub => "xpub",
            ZmqSocketType::XSub => "xsub",
            ZmqSocketType::Pub => "pub",
            ZmqSocketType::Sub => "sub",
        };
        
        let connect_endpoints = req.connect_endpoints.join(",");

        let result = sqlx::query(
            r#"
            UPDATE zmq_configs SET
                name = ?, enabled = ?, socket_type = ?, bind_endpoint = ?,
                connect_endpoints = ?, high_water_mark = ?, reconnect_interval_ms = ?
            WHERE id = ?
            "#,
        )
        .bind(&req.name)
        .bind(if req.enabled { 1i64 } else { 0i64 })
        .bind(socket_type)
        .bind(&req.bind_endpoint)
        .bind(&connect_endpoints)
        .bind(req.high_water_mark as i64)
        .bind(req.reconnect_interval_ms as i64)
        .bind(id as i64)
        .execute(&self.pool)
        .await?;

        if result.rows_affected() > 0 {
            self.get_zmq_config(id).await
        } else {
            Ok(None)
        }
    }

    pub async fn delete_zmq_config(&self, id: u32) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM zmq_configs WHERE id = ?")
            .bind(id as i64)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
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
            MappingDirection::MqttToMqtt => "mqtt_to_mqtt",
            MappingDirection::ZmqToZmq => "zmq_to_zmq",
            MappingDirection::Bidirectional => "bidirectional",
        };
        
        let source_type = match req.source_endpoint_type {
            EndpointType::Mqtt => "mqtt",
            EndpointType::Zmq => "zmq",
        };
        
        let target_type = match req.target_endpoint_type {
            EndpointType::Mqtt => "mqtt",
            EndpointType::Zmq => "zmq",
        };

        let result = sqlx::query(
            r#"
            INSERT INTO topic_mappings (source_endpoint_type, source_endpoint_id, target_endpoint_type, target_endpoint_id, source_topic, target_topic, direction, enabled, description)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(source_type)
        .bind(req.source_endpoint_id as i64)
        .bind(target_type)
        .bind(req.target_endpoint_id as i64)
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
            source_endpoint_type: req.source_endpoint_type.clone(),
            source_endpoint_id: req.source_endpoint_id,
            target_endpoint_type: req.target_endpoint_type.clone(),
            target_endpoint_id: req.target_endpoint_id,
            source_topic: req.source_topic.clone(),
            target_topic: req.target_topic.clone(),
            direction: req.direction.clone(),
            enabled: req.enabled,
            description: req.description.clone(),
        })
    }

    pub async fn update_mapping(&self, id: u32, req: &CreateMappingRequest) -> Result<Option<TopicMapping>, sqlx::Error> {
        let direction = match req.direction {
            MappingDirection::MqttToZmq => "mqtt_to_zmq",
            MappingDirection::ZmqToMqtt => "zmq_to_mqtt",
            MappingDirection::MqttToMqtt => "mqtt_to_mqtt",
            MappingDirection::ZmqToZmq => "zmq_to_zmq",
            MappingDirection::Bidirectional => "bidirectional",
        };
        
        let source_type = match req.source_endpoint_type {
            EndpointType::Mqtt => "mqtt",
            EndpointType::Zmq => "zmq",
        };
        
        let target_type = match req.target_endpoint_type {
            EndpointType::Mqtt => "mqtt",
            EndpointType::Zmq => "zmq",
        };

        let result = sqlx::query(
            r#"
            UPDATE topic_mappings SET
                source_endpoint_type = ?, source_endpoint_id = ?,
                target_endpoint_type = ?, target_endpoint_id = ?,
                source_topic = ?, target_topic = ?, direction = ?,
                enabled = ?, description = ?
            WHERE id = ?
            "#,
        )
        .bind(source_type)
        .bind(req.source_endpoint_id as i64)
        .bind(target_type)
        .bind(req.target_endpoint_id as i64)
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
                source_endpoint_type: req.source_endpoint_type.clone(),
                source_endpoint_id: req.source_endpoint_id,
                target_endpoint_type: req.target_endpoint_type.clone(),
                target_endpoint_id: req.target_endpoint_id,
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
            messages_per_second: 0.0,
            avg_latency_ms: 0.0,
            error_count: row.error_count as u64,
            queue_depth: 0,
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
