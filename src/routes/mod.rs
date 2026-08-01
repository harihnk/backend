use axum::{
    routing::{get, post},
    Router,
};
use tower_http::cors::{Any, CorsLayer};
use crate::controllers;
use crate::AppState;

/// Build the application router with all routes and CORS middleware
pub fn create_router(state: AppState) -> Router {
    // CORS — allow Vercel frontend + localhost
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        .route("/events", post(controllers::receive_events).get(controllers::get_events))
        .route("/health", get(controllers::health_check))
        .layer(cors)
        .with_state(state)
}
