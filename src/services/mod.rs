use crate::models::{EventPayload, EventRow, GeoInfo, IpApiResponse, ReverseGeoResponse, UserEvent};
use crate::repositories;
use clickhouse::Client;

/// Resolve location using the best available method:
/// 1. If GPS lat/lon provided → reverse geocode via BigDataCloud (exact)
/// 2. Else → fall back to IP-based lookup via ip-api.com (approximate)
pub async fn resolve_geo(ip: &str, latitude: Option<f64>, longitude: Option<f64>) -> GeoInfo {
    // Strategy 1: GPS reverse geocoding (exact location)
    if let (Some(lat), Some(lon)) = (latitude, longitude) {
        tracing::info!("GPS coordinates available: {}, {} — using reverse geocoding", lat, lon);
        match reverse_geocode(lat, lon).await {
            Some(geo) => {
                tracing::info!(
                    "GPS geo resolved: {} / {} / {}",
                    geo.country, geo.region, geo.city
                );
                return geo;
            }
            None => {
                tracing::warn!("Reverse geocoding failed, falling back to IP lookup");
            }
        }
    }

    // Strategy 2: IP-based geolocation (fallback)
    tracing::info!("No GPS data — using IP-based geolocation for {}", ip);
    resolve_geo_by_ip(ip).await
}

/// Reverse geocode GPS coordinates using BigDataCloud (free, no API key)
/// Returns exact country/region/city (e.g., India / Tamil Nadu / Chennai)
async fn reverse_geocode(lat: f64, lon: f64) -> Option<GeoInfo> {
    let url = format!(
        "https://api.bigdatacloud.net/data/reverse-geocode-client?latitude={}&longitude={}&localityLanguage=en",
        lat, lon
    );

    match reqwest::get(&url).await {
        Ok(resp) => match resp.json::<ReverseGeoResponse>().await {
            Ok(data) => {
                let country = data.country_name.unwrap_or_else(|| "Unknown".to_string());
                let region = data.principal_subdivision.unwrap_or_else(|| "Unknown".to_string());
                // BigDataCloud: city can be empty for some areas, use locality as fallback
                let city = data
                    .city
                    .filter(|c| !c.is_empty())
                    .or(data.locality)
                    .unwrap_or_else(|| "Unknown".to_string());

                Some(GeoInfo { country, region, city })
            }
            Err(e) => {
                tracing::warn!("Failed to parse reverse geocode response: {}", e);
                None
            }
        },
        Err(e) => {
            tracing::warn!("Reverse geocode HTTP request failed: {}", e);
            None
        }
    }
}

/// Resolve location by IP address using ip-api.com (fallback method)
async fn resolve_geo_by_ip(ip: &str) -> GeoInfo {
    // For localhost/private IPs, call without IP to get server's public IP location
    let url = if is_private_ip(ip) {
        tracing::info!("Private/local IP detected ({}), using public IP lookup", ip);
        "http://ip-api.com/json".to_string()
    } else {
        format!("http://ip-api.com/json/{}", ip)
    };

    match reqwest::get(&url).await {
        Ok(resp) => match resp.json::<IpApiResponse>().await {
            Ok(data) if data.status == "success" => {
                let geo = GeoInfo {
                    country: data.country.unwrap_or_else(|| "Unknown".to_string()),
                    region: data.region_name.unwrap_or_else(|| "Unknown".to_string()),
                    city: data.city.unwrap_or_else(|| "Unknown".to_string()),
                };
                tracing::info!(
                    "IP geo resolved: {} / {} / {}",
                    geo.country, geo.region, geo.city
                );
                geo
            }
            Ok(_) => {
                tracing::warn!("IP geo lookup returned non-success for: {}", ip);
                GeoInfo::default()
            }
            Err(e) => {
                tracing::warn!("Failed to parse IP geo response for {}: {}", ip, e);
                GeoInfo::default()
            }
        },
        Err(e) => {
            tracing::warn!("IP geo HTTP request failed for {}: {}", ip, e);
            GeoInfo::default()
        }
    }
}

/// Check if an IP address is private/local (127.x, 10.x, 192.168.x, ::1, etc.)
fn is_private_ip(ip: &str) -> bool {
    ip == "127.0.0.1"
        || ip == "::1"
        || ip == "0.0.0.0"
        || ip.starts_with("10.")
        || ip.starts_with("172.16.")
        || ip.starts_with("172.17.")
        || ip.starts_with("172.18.")
        || ip.starts_with("172.19.")
        || ip.starts_with("172.2")
        || ip.starts_with("172.30.")
        || ip.starts_with("172.31.")
        || ip.starts_with("192.168.")
        || ip.starts_with("fe80:")
}
pub async fn get_events(
    client: &Client,
) -> Result<Vec<UserEvent>, clickhouse::error::Error> {
    repositories::get_events(client).await
}

/// Enrich raw event payloads with geo-location and unique IDs
pub fn enrich_events(events: Vec<EventPayload>, _ip: &str, geo: &GeoInfo) -> Vec<EventRow> {
    events
        .into_iter()
        .map(|e| {
            EventRow {
                user_id: e.user_id,
                session_id: e.session_id,
                device: e.device,
                os: e.os,
                browser: e.browser,
                country: geo.country.clone(),
                region: geo.region.clone(),
                city: geo.city.clone(),
                latitude: e.latitude.unwrap_or(0.0),
                longitude: e.longitude.unwrap_or(0.0),
                click_count: e.click_count,
                user_journey: e.user_journey,
                screen_width: e.screen_width,
                screen_height: e.screen_height,
            }
        })
        .collect()
}
