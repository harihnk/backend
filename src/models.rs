use serde::{Deserialize, Serialize};
use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;

#[derive(Debug, Deserialize)]
pub struct IncomingEvent {
    pub user_id: String,
    pub session_id: String,
    pub event_type: String,

    #[serde(default)]
    pub seq: u32,

    #[serde(default)]
    pub page_url: String,

    #[serde(default)]
    pub timestamp: Option<String>,

    #[serde(default)]
    pub browser: Option<String>,

    #[serde(default)]
    pub os: Option<String>,

    #[serde(default)]
    pub device_type: Option<String>,

    #[serde(default)]
    pub viewport_width: Option<u16>,

    #[serde(default)]
    pub viewport_height: Option<u16>,

    #[serde(default)]
    pub ip: Option<String>,

    #[serde(default)]
    pub country: Option<String>,

    #[serde(default)]
    pub region: Option<String>,

    #[serde(default)]
    pub city: Option<String>,

    #[serde(default)]
    pub previous_page: Option<String>,

    #[serde(default)]
    pub current_page: Option<String>,

    #[serde(default)]
    pub external_referrer: Option<String>,

    #[serde(default)]
    pub hash: Option<String>,

    #[serde(default)]
    pub section: Option<String>,

    #[serde(default)]
    pub duration_seconds: Option<u32>,

    #[serde(default)]
    pub funnel_step_name: Option<String>,

    #[serde(default)]
    pub funnel_step_number: Option<u16>,

    #[serde(default)]
    pub funnel_description: Option<String>,

    #[serde(default)]
    pub element_tag: Option<String>,

    #[serde(default)]
    pub element_id: Option<String>,

    #[serde(default)]
    pub element_text: Option<String>,

    #[serde(default)]
    pub click_count: Option<u32>,

    #[serde(default)]
    pub x_pos: Option<u16>,

    #[serde(default)]
    pub y_pos: Option<u16>,

    #[serde(default)]
    pub x_percent: Option<f32>,

    #[serde(default)]
    pub y_percent: Option<f32>,

    #[serde(default)]
    pub scroll_depth: Option<u8>,
}

#[derive(Debug, Deserialize)]
pub struct EventBatch {
    pub events: Vec<IncomingEvent>,
}

#[derive(Debug, Serialize, clickhouse::Row)]
pub struct ClickEventRow {
    pub user_id: String,
    pub session_id: String,
    pub seq: u32,
    pub event_type: String,
    #[serde(with = "clickhouse::serde::time::datetime64::millis")]
    pub timestamp: OffsetDateTime,
    pub page_url: String,

    pub browser: String,
    pub os: String,
    pub device_type: String,
    pub viewport_width: u16,
    pub viewport_height: u16,

    pub ip: String,
    pub country: String,
    pub region: String,
    pub city: String,

    pub previous_page: Option<String>,
    pub current_page: Option<String>,
    pub external_referrer: Option<String>,

    pub hash: Option<String>,

    pub section: Option<String>,
    pub duration_seconds: Option<u32>,

    pub funnel_step_name: Option<String>,
    pub funnel_step_number: Option<u16>,
    pub funnel_description: Option<String>,

    pub element_tag: Option<String>,
    pub element_id: Option<String>,
    pub element_text: Option<String>,
    pub click_count: Option<u32>,

    pub x_pos: Option<u16>,
    pub y_pos: Option<u16>,
    pub x_percent: Option<f32>,
    pub y_percent: Option<f32>,

    pub scroll_depth: Option<u8>,
}

impl From<IncomingEvent> for ClickEventRow {
    fn from(e: IncomingEvent) -> Self {
        let timestamp = e
            .timestamp
            .as_deref()
            .and_then(|s| OffsetDateTime::parse(s, &Rfc3339).ok())
            .unwrap_or_else(OffsetDateTime::now_utc);

        Self {
            user_id: e.user_id,
            session_id: e.session_id,
            seq: e.seq,
            event_type: e.event_type,
            timestamp,
            page_url: e.page_url,

            browser: e.browser.unwrap_or_default(),
            os: e.os.unwrap_or_default(),
            device_type: e.device_type.unwrap_or_default(),
            viewport_width: e.viewport_width.unwrap_or_default(),
            viewport_height: e.viewport_height.unwrap_or_default(),

            ip: e.ip.unwrap_or_default(),
            country: e.country.unwrap_or_default(),
            region: e.region.unwrap_or_default(),
            city: e.city.unwrap_or_default(),

            previous_page: e.previous_page,
            current_page: e.current_page,
            external_referrer: e.external_referrer,

            hash: e.hash,

            section: e.section,
            duration_seconds: e.duration_seconds,

            funnel_step_name: e.funnel_step_name,
            funnel_step_number: e.funnel_step_number,
            funnel_description: e.funnel_description,

            element_tag: e.element_tag,
            element_id: e.element_id,
            element_text: e.element_text,
            click_count: e.click_count,

            x_pos: e.x_pos,
            y_pos: e.y_pos,
            x_percent: e.x_percent,
            y_percent: e.y_percent,

            scroll_depth: e.scroll_depth,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, clickhouse::Row)]
pub struct HeatmapPoint {
    pub x_percent: f32,
    pub y_percent: f32,
    pub hits: u64,
}

#[derive(Debug, Serialize, Deserialize, clickhouse::Row)]
pub struct NavigationStep {
    pub session_id: String,
    pub seq: u32,
    pub previous_page: Option<String>,
    pub current_page: Option<String>,
    pub external_referrer: Option<String>,
    #[serde(with = "clickhouse::serde::time::datetime64::millis")]
    pub timestamp: OffsetDateTime,
}

#[derive(Debug, Serialize, Deserialize, clickhouse::Row)]
pub struct NavigationStats {
    pub previous_page: Option<String>,
    pub current_page: Option<String>,
    pub transitions: u64,
}

#[derive(Debug, Serialize, Deserialize, clickhouse::Row)]
pub struct ClickLog {
    pub page_url: String,
    pub element_id: Option<String>,
    pub element_text: Option<String>,
    pub clicks: u64,
}

#[derive(Debug, Serialize, Deserialize, clickhouse::Row)]
pub struct ClickCountResult {
    pub user_id: String,
    pub session_id: String,
    pub total_clicks: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FunnelStepResult {
    pub step_name: String,
    pub sessions_reached: u64,
}

#[derive(Debug, Serialize, Deserialize, clickhouse::Row)]
pub struct ScrollDepthResult {
    pub page_url: String,
    pub avg_depth: f64,
}

#[derive(Debug, Serialize, Deserialize, clickhouse::Row)]
pub struct LocationStats {
    pub location: String,
    pub visitors: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserProfileReport {
    pub user_id: String,
    pub country: String,
    pub region: String,
    pub city: String,
    pub browser: String,
    pub os: String,
    pub device_type: String,
    pub navigation_flow: Vec<NavigationFlowStep>,
    pub clicks: Vec<UserClickDetail>,
    pub exit_way: Option<ExitWayDetail>,
}

#[derive(Debug, Serialize, Deserialize, clickhouse::Row)]
pub struct UserMetadataRow {
    pub country: String,
    pub region: String,
    pub city: String,
    pub browser: String,
    pub os: String,
    pub device_type: String,
}

#[derive(Debug, Serialize, Deserialize, clickhouse::Row)]
pub struct NavigationFlowStep {
    pub session_id: String,
    pub seq: u32,
    pub event_type: String,
    pub previous_page: Option<String>,
    pub current_page: Option<String>,
    pub hash: Option<String>,
    pub section: Option<String>,
    #[serde(with = "clickhouse::serde::time::datetime64::millis")]
    pub timestamp: OffsetDateTime,
}

#[derive(Debug, Serialize, Deserialize, clickhouse::Row)]
pub struct UserClickDetail {
    pub previous_page: Option<String>,
    pub current_page: Option<String>,
    pub viewport_width: u16,
    pub viewport_height: u16,
    pub element_tag: Option<String>,
    pub element_id: Option<String>,
    pub element_text: Option<String>,
    pub x_pos: Option<u16>,
    pub y_pos: Option<u16>,
    pub x_percent: Option<f32>,
    pub y_percent: Option<f32>,
    #[serde(with = "clickhouse::serde::time::datetime64::millis")]
    pub timestamp: OffsetDateTime,
}

#[derive(Debug, Serialize, Deserialize, clickhouse::Row)]
pub struct ExitWayDetail {
    pub exit_type: Option<String>,
    pub target_url: Option<String>,
    #[serde(with = "clickhouse::serde::time::datetime64::millis")]
    pub timestamp: OffsetDateTime,
}