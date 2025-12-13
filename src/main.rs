//! ZeroMQTT - Web Management Interface
//!
//! This is the main entry point for the ZeroMQTT bridge with web management.

use axum::Router;
use tokio::net::TcpListener;
use tower_http::cors::{Any, CorsLayer};
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use vite_rs_axum_0_8::ViteServe;

use zeromqtt::api::api_routes;
use zeromqtt::bridge::BridgeCore;
use zeromqtt::config::AppConfig;
use zeromqtt::db::{init_db, Repository};
use zeromqtt::state::AppState;

#[derive(vite_rs::Embed)]
#[root = "./dashboard"]
struct Assets;

#[tokio::main]
async fn main() {
    // Initialize logging
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "zeromqtt=info,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!("===================================");
    info!("    ZeroMQTT Bridge v{}    ", env!("CARGO_PKG_VERSION"));
    info!("===================================");

    // Initialize configuration
    let config = AppConfig::new();
    info!("Configuration loaded");

    // Initialize database
    let pool = match init_db().await {
        Ok(pool) => {
            info!("Database initialized successfully");
            pool
        }
        Err(e) => {
            tracing::error!("Failed to initialize database: {}", e);
            std::process::exit(1);
        }
    };

    // Create repository
    let repo = Repository::new(pool);

    // Create bridge core
    let bridge = BridgeCore::new(repo.clone());
    info!("Bridge core created");

    // Auto-start the bridge
    match bridge.start().await {
        Ok(()) => info!("🔗 Bridge started successfully"),
        Err(e) => tracing::warn!("Failed to auto-start bridge: {} (can be started manually)", e),
    }

    // Create application state
    let state = AppState::new(config.clone(), repo, bridge);

    // Start Vite dev server in development mode
    #[cfg(debug_assertions)]
    let _guard = Assets::start_dev_server(true);

    // Configure CORS
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // Build API routes with state
    let api = api_routes();

    // Build main router
    let app = Router::new()
        // API routes
        .nest("/api", api)
        // Static assets (Vite)
        .route_service("/", ViteServe::new(Assets::boxed()))
        .route_service("/{*path}", ViteServe::new(Assets::boxed()))
        // Add CORS middleware
        .layer(cors)
        // Add config to request extensions for auth middleware
        .layer(axum::Extension(state.config.clone()))
        // Add application state
        .with_state(state);

    let addr = format!("{}:{}", config.server.host, config.server.port);
    info!("🚀 ZeroMQTT Web Server starting on http://{}", addr);
    info!("📊 Dashboard: http://localhost:{}", config.server.port);
    info!("🔌 API: http://localhost:{}/api", config.server.port);
    info!("📁 Database: ~/.zeromqtt/data.db");

    let listener = TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}