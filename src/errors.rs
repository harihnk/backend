
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};

use serde_json::json;

/// =====================================================
/// Application Error Type
/// =====================================================

#[derive(Debug)]
pub enum AppError {
    ClickHouse(String),

    BadRequest(String),

    Internal(String),
}

/// =====================================================
/// Display
/// =====================================================

impl std::fmt::Display for AppError {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        match self {
            AppError::ClickHouse(msg) => {
                write!(
                    f,
                    "ClickHouse error: {}",
                    msg
                )
            }

            AppError::BadRequest(msg) => {
                write!(
                    f,
                    "Bad request: {}",
                    msg
                )
            }

            AppError::Internal(msg) => {
                write!(
                    f,
                    "Internal error: {}",
                    msg
                )
            }
        }
    }
}

/// =====================================================
/// Error Trait
/// =====================================================

impl std::error::Error for AppError {}

/// =====================================================
/// ClickHouse Error Conversion
/// =====================================================

impl From<clickhouse::error::Error>
    for AppError
{
    fn from(
        e: clickhouse::error::Error,
    ) -> Self {
        AppError::ClickHouse(
            e.to_string(),
        )
    }
}

/// =====================================================
/// Serde JSON Error Conversion
/// =====================================================

impl From<serde_json::Error>
    for AppError
{
    fn from(
        e: serde_json::Error,
    ) -> Self {
        AppError::Internal(
            e.to_string(),
        )
    }
}

/// =====================================================
/// HTTP Response Mapping
/// =====================================================

impl IntoResponse for AppError {
    fn into_response(
        self,
    ) -> Response {

        let (status, message) =
            match self {

                AppError::ClickHouse(msg) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    msg,
                ),

                AppError::BadRequest(msg) => (
                    StatusCode::BAD_REQUEST,
                    msg,
                ),

                AppError::Internal(msg) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    msg,
                ),
            };

        (
            status,
            Json(json!({
                "status": "error",
                "message": message
            })),
        )
            .into_response()
    }
}
