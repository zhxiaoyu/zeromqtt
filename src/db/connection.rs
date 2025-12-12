//! Database connection and initialization

use sqlx::sqlite::{SqliteConnectOptions, SqlitePool, SqlitePoolOptions};
use std::path::PathBuf;
use std::str::FromStr;
use tracing::info;

/// Get the database path: ~/.zeromqtt/data.db
pub fn get_db_path() -> PathBuf {
    let home = dirs::home_dir().expect("Could not find home directory");
    let zeromqtt_dir = home.join(".zeromqtt");
    
    // Create directory if it doesn't exist
    if !zeromqtt_dir.exists() {
        std::fs::create_dir_all(&zeromqtt_dir).expect("Failed to create .zeromqtt directory");
    }
    
    zeromqtt_dir.join("data.db")
}

/// Initialize the database connection pool
pub async fn init_db() -> Result<SqlitePool, sqlx::Error> {
    let db_path = get_db_path();
    let db_url = format!("sqlite:{}?mode=rwc", db_path.display());
    
    info!("Initializing database at: {}", db_path.display());
    
    let options = SqliteConnectOptions::from_str(&db_url)?
        .create_if_missing(true)
        .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)
        .synchronous(sqlx::sqlite::SqliteSynchronous::Normal);
    
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(options)
        .await?;
    
    // Run migrations
    run_migrations(&pool).await?;
    
    // Initialize default data if empty
    init_default_data(&pool).await?;
    
    info!("Database initialized successfully");
    Ok(pool)
}

/// Run database migrations - CREATE NEW SCHEMA
async fn run_migrations(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    // Create mqtt_configs table (plural, supports multiple brokers)
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS mqtt_configs (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL UNIQUE,
            enabled INTEGER NOT NULL DEFAULT 1,
            broker_url TEXT NOT NULL DEFAULT 'localhost',
            port INTEGER NOT NULL DEFAULT 1883,
            client_id TEXT NOT NULL DEFAULT 'zeromqtt-bridge',
            username TEXT,
            password TEXT,
            use_tls INTEGER NOT NULL DEFAULT 0,
            keep_alive_seconds INTEGER NOT NULL DEFAULT 60,
            clean_session INTEGER NOT NULL DEFAULT 1
        )
        "#,
    )
    .execute(pool)
    .await?;

    // Create zmq_configs table (supports XPUB/XSUB with multiple endpoints)
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS zmq_configs (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL UNIQUE,
            enabled INTEGER NOT NULL DEFAULT 1,
            socket_type TEXT NOT NULL DEFAULT 'xpub',
            bind_endpoint TEXT,
            connect_endpoints TEXT,
            high_water_mark INTEGER NOT NULL DEFAULT 1000,
            reconnect_interval_ms INTEGER NOT NULL DEFAULT 1000
        )
        "#,
    )
    .execute(pool)
    .await?;

    // Create NEW topic_mappings table with endpoint references
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS topic_mappings (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            source_endpoint_type TEXT NOT NULL DEFAULT 'mqtt',
            source_endpoint_id INTEGER NOT NULL DEFAULT 1,
            target_endpoint_type TEXT NOT NULL DEFAULT 'zmq',
            target_endpoint_id INTEGER NOT NULL DEFAULT 1,
            source_topic TEXT NOT NULL,
            target_topic TEXT NOT NULL,
            direction TEXT NOT NULL DEFAULT 'mqtt_to_zmq',
            enabled INTEGER NOT NULL DEFAULT 1,
            description TEXT
        )
        "#,
    )
    .execute(pool)
    .await?;

    // Create message_stats table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS message_stats (
            id INTEGER PRIMARY KEY DEFAULT 1,
            mqtt_received INTEGER NOT NULL DEFAULT 0,
            mqtt_sent INTEGER NOT NULL DEFAULT 0,
            zmq_received INTEGER NOT NULL DEFAULT 0,
            zmq_sent INTEGER NOT NULL DEFAULT 0,
            error_count INTEGER NOT NULL DEFAULT 0,
            start_time INTEGER NOT NULL DEFAULT 0
        )
        "#,
    )
    .execute(pool)
    .await?;

    // Migrate old tables if they exist
    migrate_old_tables(pool).await?;

    Ok(())
}

/// Migrate data from old single-config tables to new multi-config tables
async fn migrate_old_tables(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    // Check if old mqtt_config table exists (singular)
    let old_mqtt_exists: Option<(String,)> = sqlx::query_as(
        "SELECT name FROM sqlite_master WHERE type='table' AND name='mqtt_config'"
    )
    .fetch_optional(pool)
    .await?;

    if old_mqtt_exists.is_some() {
        // Check if new table is empty
        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM mqtt_configs")
            .fetch_one(pool)
            .await?;
        
        if count.0 == 0 {
            // Migrate data from old table
            sqlx::query(
                r#"
                INSERT INTO mqtt_configs (name, enabled, broker_url, port, client_id, username, password, use_tls, keep_alive_seconds, clean_session)
                SELECT 'Default', 1, broker_url, port, client_id, username, password, use_tls, keep_alive_seconds, clean_session
                FROM mqtt_config WHERE id = 1
                "#
            )
            .execute(pool)
            .await
            .ok(); // Ignore error if migration fails
        }
        
        // Drop old table
        sqlx::query("DROP TABLE IF EXISTS mqtt_config")
            .execute(pool)
            .await
            .ok();
    }

    // Check if old zmq_config table exists (singular)
    let old_zmq_exists: Option<(String,)> = sqlx::query_as(
        "SELECT name FROM sqlite_master WHERE type='table' AND name='zmq_config'"
    )
    .fetch_optional(pool)
    .await?;

    if old_zmq_exists.is_some() {
        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM zmq_configs")
            .fetch_one(pool)
            .await?;
        
        if count.0 == 0 {
            // Migrate with XSUB as default (for proxy pattern)
            sqlx::query(
                r#"
                INSERT INTO zmq_configs (name, enabled, socket_type, bind_endpoint, connect_endpoints, high_water_mark, reconnect_interval_ms)
                SELECT 'XSUB Proxy', 1, 'xsub', sub_endpoint, '', high_water_mark, reconnect_interval_ms
                FROM zmq_config WHERE id = 1
                "#
            )
            .execute(pool)
            .await
            .ok();

            // Also create XPUB config
            sqlx::query(
                r#"
                INSERT INTO zmq_configs (name, enabled, socket_type, bind_endpoint, connect_endpoints, high_water_mark, reconnect_interval_ms)
                SELECT 'XPUB Proxy', 1, 'xpub', pub_endpoint, '', high_water_mark, reconnect_interval_ms
                FROM zmq_config WHERE id = 1
                "#
            )
            .execute(pool)
            .await
            .ok();
        }
        
        sqlx::query("DROP TABLE IF EXISTS zmq_config")
            .execute(pool)
            .await
            .ok();
    }

    Ok(())
}

/// Initialize default data if tables are empty
async fn init_default_data(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    // Check if mqtt_configs exists
    let mqtt_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM mqtt_configs")
        .fetch_one(pool)
        .await?;
    
    if mqtt_count.0 == 0 {
        sqlx::query(
            r#"
            INSERT INTO mqtt_configs (name, enabled, broker_url, port, client_id, use_tls, keep_alive_seconds, clean_session)
            VALUES ('Default', 1, 'localhost', 1883, 'zeromqtt-bridge', 0, 60, 1)
            "#,
        )
        .execute(pool)
        .await?;
    }

    // Check if zmq_configs exists
    let zmq_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM zmq_configs")
        .fetch_one(pool)
        .await?;
    
    if zmq_count.0 == 0 {
        // Create XSUB socket (receives from publishers)
        sqlx::query(
            r#"
            INSERT INTO zmq_configs (name, enabled, socket_type, bind_endpoint, connect_endpoints, high_water_mark, reconnect_interval_ms)
            VALUES ('XSUB Proxy', 1, 'xsub', 'tcp://*:5556', '', 1000, 1000)
            "#,
        )
        .execute(pool)
        .await?;

        // Create XPUB socket (serves subscribers)
        sqlx::query(
            r#"
            INSERT INTO zmq_configs (name, enabled, socket_type, bind_endpoint, connect_endpoints, high_water_mark, reconnect_interval_ms)
            VALUES ('XPUB Proxy', 1, 'xpub', 'tcp://*:5555', '', 1000, 1000)
            "#,
        )
        .execute(pool)
        .await?;
    }

    // Check if message_stats exists
    let stats_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM message_stats")
        .fetch_one(pool)
        .await?;
    
    if stats_count.0 == 0 {
        let now = chrono::Utc::now().timestamp();
        sqlx::query(
            r#"
            INSERT INTO message_stats (id, mqtt_received, mqtt_sent, zmq_received, zmq_sent, error_count, start_time)
            VALUES (1, 0, 0, 0, 0, 0, ?)
            "#,
        )
        .bind(now)
        .execute(pool)
        .await?;
    }

    Ok(())
}
