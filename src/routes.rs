use std::sync::Arc;
use axum::{
    routing::{get, post},
    Router,
};
use tower_http::cors::{Any, CorsLayer};
use crate::{controller, AppState};

pub fn build_router(state: Arc<AppState>) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        .route("/health", get(controller::health_check))
        .route("/events", post(controller::track_events))
        .route("/analytics/heatmap", get(controller::get_heatmap))
        .route("/analytics/navigation", get(controller::get_navigation))
        .route("/analytics/navigation-stats", get(controller::get_navigation_stats))
        .route("/analytics/user-journey", get(controller::get_user_journey))
        .route("/analytics/user-report", get(controller::get_user_report))
        .route("/analytics/click-logs", get(controller::get_click_logs))
        .route("/analytics/click-count", get(controller::get_click_count))
        .route("/analytics/funnel", get(controller::get_funnel))
        .route("/analytics/scroll-depth", get(controller::get_scroll_depth))
        .route("/analytics/countries", get(controller::get_countries))
        .route("/analytics/regions", get(controller::get_regions))
        .route("/analytics/cities", get(controller::get_cities))
        .layer(cors)
        .with_state(state)
}