mod controllers;
mod models;
mod repositories;
mod routes;
mod services;
mod db;

use clickhouse::Client;
use std::net::SocketAddr;
use db::clickhouse::create_clickhouse_client;


/// Shared application state
#[derive(Clone)]
pub struct AppState {
    pub clickhouse: Client,
}

/// Load an environment variable with a fallback default
fn env_or(key: &str, default: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| default.to_string())
}

#[tokio::main]
async fn main() {
    // Load .env file (silently ignore if missing)
    dotenvy::dotenv().ok();

    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info".into()),
        )
        .init();

    tracing::info!("Starting Analytics Backend...");

    // Read ClickHouse configuration from .env
    let ch_url = env_or("CLICKHOUSE_URL", "http://localhost:8123");
    let ch_database = env_or("CLICKHOUSE_DATABASE", "analytics");
    let ch_user = env_or("CLICKHOUSE_USER", "default");
    let ch_password = env_or("CLICKHOUSE_PASSWORD", "");

    tracing::info!("ClickHouse URL: {}", ch_url);
    tracing::info!("ClickHouse Database: {}", ch_database);
    tracing::info!("ClickHouse User: {}", ch_user);

    // Create ClickHouse client for setup (no database set yet)
    let mut setup_client = Client::default()
        .with_url(&ch_url)
        .with_user(&ch_user);
    if !ch_password.is_empty() {
        setup_client = setup_client.with_password(&ch_password);
    }

    // Initialize database schema
    match repositories::init_db(&setup_client).await {
        Ok(_) => tracing::info!("Database initialized successfully"),
        Err(e) => {
            tracing::error!("Failed to initialize database: {}", e);
            tracing::warn!("Make sure ClickHouse is running on {}", ch_url);
        }
    }

    // Create ClickHouse client with database set
    let mut clickhouse = Client::default()
        .with_url(&ch_url)
        .with_database(&ch_database)
        .with_user(&ch_user);
    if !ch_password.is_empty() {
        clickhouse = clickhouse.with_password(&ch_password);
    }
    let client = create_clickhouse_client();
    match client.query("SELECT 1").execute().await {
        Ok(_) => println!("✅ Connected to ClickHouse"),
        Err(e) => println!("❌ {}", e),
    }
    // Build application state
    let state = AppState { clickhouse };

    // Build router
    let app = routes::create_router(state);

    // Read server config from .env
    let host = env_or("SERVER_HOST", "0.0.0.0");
    let port: u16 = env_or("SERVER_PORT", "8080").parse().unwrap_or(8080);
    let addr: SocketAddr = format!("{}:{}", host, port).parse().unwrap();

    tracing::info!("Server listening on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .unwrap();
}
