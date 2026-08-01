use clickhouse::Row;
use serde::{Deserialize, Serialize};
/// =======================================================
/// Incoming Events From tracker.js
/// =======================================================

#[derive(Debug, Deserialize, Clone)]
pub struct EventPayload {
    #[serde(default)]
    pub user_id: String,

    #[serde(default)]
    pub session_id: String,

    #[serde(default)]
    pub device: String,

    #[serde(default)]
    pub browser: String,
    #[serde(default)]

    pub os: String,

    #[serde(default)]
    pub country: String,

    #[serde(default)]
    pub region: String,

    #[serde(default)]
    pub city: String,

    #[serde(default)]
    pub latitude: Option<f64>,

    #[serde(default)]
    pub longitude: Option<f64>,

    #[serde(default)]
    pub click_count: u32,

    #[serde(default)]
    pub user_journey: String,

    #[serde(default)]
    pub screen_width: u32,

    #[serde(default)]
    pub screen_height: u32,
}

#[derive(Debug, Deserialize)]
pub struct EventBatch {
    pub events: Vec<EventPayload>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Row)]
pub struct EventRow {
    pub user_id: String,
    pub session_id: String,
    pub device: String,
    pub browser: String,
    pub os: String,
    pub country: String,
    pub region: String,
    pub city: String,
    pub latitude: f64,
    pub longitude: f64,
    pub click_count: u32,
    pub user_journey: String,
    pub screen_width: u32,
    pub screen_height: u32,
}

pub type UserEvent = EventRow;
/// =======================================================
/// Geo Information
/// =======================================================

#[derive(Debug, Clone, Deserialize)]
pub struct GeoInfo {
    pub country: String,
    pub region: String,
    pub city: String,
}

impl Default for GeoInfo {
    fn default() -> Self {
        Self {
            country: "Unknown".into(),
            region: "Unknown".into(),
            city: "Unknown".into(),
        }
    }
}

/// =======================================================
/// IP API Response
/// =======================================================

#[derive(Debug, Clone, Deserialize)]
pub struct IpApiResponse {
    pub status: String,
    pub country: Option<String>,

    #[serde(rename = "regionName")]
    pub region_name: Option<String>,

    pub city: Option<String>,
}

/// =======================================================
/// Reverse Geo Response
/// =======================================================

#[derive(Debug, Clone, Deserialize)]
pub struct ReverseGeoResponse {
    #[serde(rename = "countryName")]
    pub country_name: Option<String>,

    #[serde(rename = "principalSubdivision")]
    pub principal_subdivision: Option<String>,

    pub city: Option<String>,

    pub locality: Option<String>,
}

/// =======================================================
/// API Response
/// =======================================================

#[derive(Debug, Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub message: String,
    pub data: Option<T>,
}