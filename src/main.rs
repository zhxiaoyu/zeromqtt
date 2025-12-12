//! ZeroMQTT - Web Management Interface
//!
//! This is the main entry point for the ZeroMQTT bridge with web management.

use axum::Router;
use std::sync::Arc;
use tokio::net::TcpListener;
use tower_http::cors::{Any, CorsLayer};
use vite_rs_axum_0_8::ViteServe;

use zeromqtt::api::api_routes;
use zeromqtt::config::AppConfig;

#[derive(vite_rs::Embed)]
#[root = "./dashboard"]
struct Assets;

#[tokio::main]
async fn main() {
    // Initialize configuration
    let config = Arc::new(AppConfig::new());

    // Start Vite dev server in development mode
    #[cfg(debug_assertions)]
    let _guard = Assets::start_dev_server(true);

    // Configure CORS
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // Build API routes
    let api = api_routes(config.clone());

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
        .layer(axum::Extension(config.clone()));

    let addr = format!("{}:{}", config.server.host, config.server.port);
    println!("🚀 ZeroMQTT Web Server starting on http://{}", addr);
    println!("📊 Dashboard: http://localhost:{}", config.server.port);
    println!("🔌 API: http://localhost:{}/api", config.server.port);

    let listener = TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}