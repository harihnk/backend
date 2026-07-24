mod controller;
mod errors;
mod models;
mod repository;
mod routes;
mod service;
mod ip_resolver;
mod geoip;

use std::sync::Arc;
use std::net::SocketAddr;
use clickhouse::Client;
use tracing_subscriber::EnvFilter;

pub struct AppState {
    pub service: service::EventService,
    pub geoip: geoip::GeoLocationService,
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    let ch_url = std::env::var("CLICKHOUSE_URL")
        .unwrap_or_else(|_| "http://localhost:8123".to_string());

    let ch_db = std::env::var("CLICKHOUSE_DB")
        .unwrap_or_else(|_| "default".to_string());

    let ch_user = std::env::var("CLICKHOUSE_USER")
        .unwrap_or_else(|_| "default".to_string());

    let ch_password = std::env::var("CLICKHOUSE_PASSWORD").unwrap_or_default();

    tracing::info!("Connecting to ClickHouse...");
    tracing::info!("CLICKHOUSE_URL={}", ch_url);
    tracing::info!("CLICKHOUSE_DB={}", ch_db);

    let client = Client::default()
        .with_url(ch_url)
        .with_database(ch_db)
        .with_user(ch_user)
        .with_password(ch_password);

    match client.query("SELECT 1").execute().await {
        Ok(_) => {
            tracing::info!("✅ ClickHouse connected successfully");
        }
        Err(err) => {
            tracing::error!("❌ ClickHouse connection failed: {}", err);
            std::process::exit(1);
        }
    }

    let repo = repository::EventRepository::new(client);
    let service = service::EventService::new(repo);
    let geoip = geoip::GeoLocationService::new();
    let state = Arc::new(AppState { service, geoip });

    let app = routes::build_router(state);

    let port = std::env::var("PORT").unwrap_or_else(|_| "3003".to_string());
    let addr = format!("0.0.0.0:{}", port);

    tracing::info!("🚀 Analytics API listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();

    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .with_graceful_shutdown(shutdown_signal())
    .await
    .unwrap();
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    tracing::info!("🛑 Shutdown signal received, draining in-flight requests...");
}