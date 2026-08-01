use axum::{
    extract::{ConnectInfo, State},
    http::HeaderMap,
    Json,
};
use std::net::SocketAddr;

use crate::models::{ApiResponse, EventBatch, UserEvent};
use crate::repositories;
use crate::services;
use crate::AppState;

/// Extract the real client IP from request headers or connection info.
/// Checks X-Forwarded-For and X-Real-IP headers for proxied requests.
fn extract_client_ip(headers: &HeaderMap, addr: &SocketAddr) -> String {
    // Check X-Forwarded-For header first (reverse proxy / load balancer)
    if let Some(forwarded) = headers.get("x-forwarded-for") {
        if let Ok(value) = forwarded.to_str() {
            if let Some(ip) = value.split(',').next() {
                let trimmed = ip.trim();
                if !trimmed.is_empty() {
                    return trimmed.to_string();
                }
            }
        }
    }

    // Check X-Real-IP header
    if let Some(real_ip) = headers.get("x-real-ip") {
        if let Ok(value) = real_ip.to_str() {
            let trimmed = value.trim();
            if !trimmed.is_empty() {
                return trimmed.to_string();
            }
        }
    }

    // Fallback to direct connection address
    addr.ip().to_string()
}

/// POST /events — Receive a batch of tracking events from tracker.js
pub async fn receive_events(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Json(batch): Json<EventBatch>,
) -> Json<ApiResponse<()>> {
    // Get the real client IP
    let ip = extract_client_ip(&headers, &addr);

    // Extract events from the request body
    let events = batch.events;

    // Log request details
    tracing::info!(
        "Received {} events from IP: {} (session: {})",
        events.len(),
        ip,
        events
            .first()
            .map(|e| e.session_id.as_str())
            .unwrap_or("unknown")
    );
    // Extract GPS coordinates from the first event (all events in a batch share the same location)
    let latitude = events.first().and_then(|e| e.latitude);
    let longitude = events.first().and_then(|e| e.longitude);

    // Resolve geo-location: GPS reverse geocode (exact) → IP fallback (approximate)
    let geo = services::resolve_geo(&ip, latitude, longitude).await;

    // Enrich events with geo data + UUIDs
    let enriched = services::enrich_events(events, &ip, &geo);
    
    // Insert into ClickHouse
    match repositories::insert_events(&state.clickhouse, enriched).await {
        Ok(_) => {
            tracing::info!("Successfully inserted events into ClickHouse");

            Json(ApiResponse {
                success: true,
                message: "Events recorded successfully".to_string(),
                data: None,
            })
        }

        Err(err) => {
            tracing::error!("Failed to insert events: {}", err);

            Json(ApiResponse {
                success: false,
                message: format!("Failed to record events: {}", err),
                data: None,
            })
        }
    }
}

/// GET /events — Retrieve events for testing/debugging purposes
pub async fn get_events(State(state): State<AppState>) -> Json<ApiResponse<Vec<UserEvent>>> {
    match services::get_events(&state.clickhouse).await {
        Ok(events) => {
            tracing::info!("Fetched {} events from ClickHouse", events.len());
            Json(ApiResponse {
                success: true,
                message: format!("Fetched {} events", events.len()),
                data: Some(events),
            })
        }
        Err(e) => {
            tracing::error!("Failed to fetch events from ClickHouse: {}", e);
            Json(ApiResponse {
                success: false,
                message: format!("Failed to fetch events: {}", e),
                data: None,
            })
        }
    }
}

/// GET /health — Health check endpoint
pub async fn health_check() -> Json<ApiResponse<()>> {
    Json(ApiResponse {
        success: true,
        message: "Analytics backend is running".to_string(),
        data: None,
    })
}