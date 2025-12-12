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

/// Run database migrations
async fn run_migrations(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    // Create mqtt_config table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS mqtt_config (
            id INTEGER PRIMARY KEY DEFAULT 1,
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

    // Create zmq_config table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS zmq_config (
            id INTEGER PRIMARY KEY DEFAULT 1,
            pub_endpoint TEXT NOT NULL DEFAULT 'tcp://*:5555',
            sub_endpoint TEXT NOT NULL DEFAULT 'tcp://*:5556',
            high_water_mark INTEGER NOT NULL DEFAULT 1000,
            reconnect_interval_ms INTEGER NOT NULL DEFAULT 1000
        )
        "#,
    )
    .execute(pool)
    .await?;

    // Create topic_mappings table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS topic_mappings (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
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

    Ok(())
}

/// Initialize default data if tables are empty
async fn init_default_data(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    // Check if mqtt_config exists
    let mqtt_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM mqtt_config")
        .fetch_one(pool)
        .await?;
    
    if mqtt_count.0 == 0 {
        sqlx::query(
            r#"
            INSERT INTO mqtt_config (id, broker_url, port, client_id, use_tls, keep_alive_seconds, clean_session)
            VALUES (1, 'localhost', 1883, 'zeromqtt-bridge', 0, 60, 1)
            "#,
        )
        .execute(pool)
        .await?;
    }

    // Check if zmq_config exists
    let zmq_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM zmq_config")
        .fetch_one(pool)
        .await?;
    
    if zmq_count.0 == 0 {
        sqlx::query(
            r#"
            INSERT INTO zmq_config (id, pub_endpoint, sub_endpoint, high_water_mark, reconnect_interval_ms)
            VALUES (1, 'tcp://*:5555', 'tcp://*:5556', 1000, 1000)
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
