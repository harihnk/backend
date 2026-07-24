use std::sync::Arc;
use std::net::SocketAddr;
use axum::{
    body::Bytes,
    extract::{Query, State, ConnectInfo},
    http::HeaderMap,
    Json,
};
use serde::Deserialize;
use serde_json::json;

use crate::errors::AppError;
use crate::models::EventBatch;
use crate::ip_resolver;
use crate::AppState;

pub async fn health_check() -> &'static str {
    "ok"
}

pub async fn track_events(
    State(state): State<Arc<AppState>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<Json<serde_json::Value>, AppError> {
    let mut payload: EventBatch = serde_json::from_slice(&body)
        .map_err(|e| AppError::BadRequest(format!("invalid event payload: {}", e)))?;

    if payload.events.is_empty() {
        return Ok(Json(json!({ "status": "ok", "inserted": 0 })));
    }

    let trust_proxy = std::env::var("TRUST_PROXY_HEADERS")
        .map(|v| v == "true" || v == "1")
        .unwrap_or(false);

    let client_ip = ip_resolver::resolve_client_ip(&headers, Some(addr), trust_proxy);
    tracing::debug!("Client IP resolved: {}", client_ip);

    let geo = if ip_resolver::is_private_or_loopback(&client_ip) {
        tracing::debug!("Private IP {}, skipping geo lookup", client_ip);
        crate::geoip::GeoLocation {
            ip: client_ip.clone(),
            country: "Private".to_string(),
            country_code: "PRV".to_string(),
            region: "Private".to_string(),
            city: "Private".to_string(),
        }
    } else {
        match state.geoip.lookup(&client_ip).await {
            Ok(g) => {
                tracing::debug!("Geo lookup: {} -> {}, {}", client_ip, g.country, g.city);
                g
            }
            Err(e) => {
                tracing::warn!("Geo lookup failed for {}: {}", client_ip, e);
                crate::geoip::GeoLocation {
                    ip: client_ip.clone(),
                    country: "Unknown".to_string(),
                    country_code: "UNK".to_string(),
                    region: "Unknown".to_string(),
                    city: "Unknown".to_string(),
                }
            }
        }
    };

    for event in &mut payload.events {
        let mut final_ip = client_ip.clone();
        let mut final_country = geo.country.clone();
        let mut final_region = geo.region.clone();
        let mut final_city = geo.city.clone();

        // Always prefer client-sent geolocation (from browser GPS / Nominatim)
        // over server-side IP lookup, since GPS is far more accurate.
        if let Some(ref client_country) = event.country {
            if !client_country.is_empty() && client_country != "unknown" && client_country != "Private" {
                final_country = client_country.clone();
            }
        }
        if let Some(ref client_region) = event.region {
            if !client_region.is_empty() && client_region != "unknown" && client_region != "Private" {
                final_region = client_region.clone();
            }
        }
        if let Some(ref client_city) = event.city {
            if !client_city.is_empty() && client_city != "unknown" && client_city != "Private" {
                final_city = client_city.clone();
            }
        }
        if let Some(ref client_sent_ip) = event.ip {
            if !client_sent_ip.is_empty() && client_sent_ip != "unknown" && !ip_resolver::is_private_or_loopback(client_sent_ip) {
                final_ip = client_sent_ip.clone();
            }
        }

        event.ip = Some(final_ip);
        event.country = Some(final_country);
        event.region = Some(final_region);
        event.city = Some(final_city);
    }

    let inserted = state.service.track_events(payload.events).await?;

    Ok(Json(json!({
        "status": "ok",
        "inserted": inserted,
        "ip": client_ip,
        "geo": {
            "country": geo.country,
            "country_code": geo.country_code,
            "region": geo.region,
            "city": geo.city
        }
    })))
}

#[derive(Deserialize)]
pub struct HeatmapQuery {
    pub page_url: String,
    #[serde(default = "default_heatmap_event_type")]
    pub event_type: String,
}

fn default_heatmap_event_type() -> String {
    "click".to_string()
}

pub async fn get_heatmap(
    State(state): State<Arc<AppState>>,
    Query(q): Query<HeatmapQuery>,
) -> Result<Json<serde_json::Value>, AppError> {
    let points = state.service.heatmap(q.page_url, q.event_type).await?;
    Ok(Json(json!({ "points": points })))
}

#[derive(Deserialize)]
pub struct NavigationQuery {
    pub user_id: String,
}

pub async fn get_navigation(
    State(state): State<Arc<AppState>>,
    Query(q): Query<NavigationQuery>,
) -> Result<Json<serde_json::Value>, AppError> {
    let steps = state.service.navigation_flow(q.user_id).await?;
    Ok(Json(json!({ "steps": steps })))
}

pub async fn get_navigation_stats(
    State(state): State<Arc<AppState>>,
) -> Result<Json<serde_json::Value>, AppError> {
    let stats = state.service.navigation_stats().await?;
    Ok(Json(json!({ "navigation": stats })))
}

#[derive(Deserialize)]
pub struct UserJourneyQuery {
    pub user_id: String,
}

pub async fn get_user_journey(
    State(state): State<Arc<AppState>>,
    Query(q): Query<UserJourneyQuery>,
) -> Result<Json<serde_json::Value>, AppError> {
    let journey = state.service.user_journey(q.user_id).await?;
    Ok(Json(json!({ "journey": journey })))
}

#[derive(Deserialize)]
pub struct UserReportQuery {
    pub user_id: String,
}

pub async fn get_user_report(
    State(state): State<Arc<AppState>>,
    Query(q): Query<UserReportQuery>,
) -> Result<Json<serde_json::Value>, AppError> {
    let report = state.service.user_report(q.user_id).await?;
    Ok(Json(json!(report)))
}

pub async fn get_click_logs(
    State(state): State<Arc<AppState>>,
) -> Result<Json<serde_json::Value>, AppError> {
    let logs = state.service.click_logs().await?;
    Ok(Json(json!({ "click_logs": logs })))
}

#[derive(Deserialize)]
pub struct ClickCountQuery {
    pub user_id: String,
}

pub async fn get_click_count(
    State(state): State<Arc<AppState>>,
    Query(q): Query<ClickCountQuery>,
) -> Result<Json<serde_json::Value>, AppError> {
    let counts = state.service.click_count(q.user_id).await?;
    Ok(Json(json!({ "click_counts": counts })))
}

#[derive(Deserialize)]
pub struct FunnelQuery {
    pub steps: String,
}

pub async fn get_funnel(
    State(state): State<Arc<AppState>>,
    Query(q): Query<FunnelQuery>,
) -> Result<Json<serde_json::Value>, AppError> {
    let steps: Vec<String> = q.steps
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    if steps.is_empty() {
        return Err(AppError::BadRequest("steps query param is required".into()));
    }

    let result = state.service.funnel(steps).await?;
    Ok(Json(json!({ "funnel": result })))
}

#[derive(Deserialize)]
pub struct ScrollDepthQuery {
    pub page_url: String,
}

pub async fn get_scroll_depth(
    State(state): State<Arc<AppState>>,
    Query(q): Query<ScrollDepthQuery>,
) -> Result<Json<serde_json::Value>, AppError> {
    let result = state.service.scroll_depth(q.page_url).await?;
    Ok(Json(json!({ "scroll_depth": result })))
}

pub async fn get_countries(
    State(state): State<Arc<AppState>>,
) -> Result<Json<serde_json::Value>, AppError> {
    let countries = state.service.countries().await?;
    Ok(Json(json!({ "countries": countries })))
}

#[derive(Deserialize)]
pub struct RegionsQuery {
    pub country: String,
}

pub async fn get_regions(
    State(state): State<Arc<AppState>>,
    Query(q): Query<RegionsQuery>,
) -> Result<Json<serde_json::Value>, AppError> {
    let regions = state.service.regions(q.country).await?;
    Ok(Json(json!({ "regions": regions })))
}

#[derive(Deserialize)]
pub struct CitiesQuery {
    pub region: String,
}

pub async fn get_cities(
    State(state): State<Arc<AppState>>,
    Query(q): Query<CitiesQuery>,
) -> Result<Json<serde_json::Value>, AppError> {
    let cities = state.service.cities(q.region).await?;
    Ok(Json(json!({ "cities": cities })))
}