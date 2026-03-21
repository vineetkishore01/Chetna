//! Error handling utilities for Chetna API
//!
//! Provides consistent error responses across the API.

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

#[derive(Debug)]
pub struct ApiError {
    pub error: String,
    pub message: String,
    pub status_code: StatusCode,
    pub details: Option<serde_json::Value>,
}

impl ApiError {
    pub fn bad_request(message: &str) -> Self {
        Self {
            error: "invalid_request".to_string(),
            message: message.to_string(),
            status_code: StatusCode::BAD_REQUEST,
            details: None,
        }
    }

    pub fn unauthorized(message: &str) -> Self {
        Self {
            error: "unauthorized".to_string(),
            message: message.to_string(),
            status_code: StatusCode::UNAUTHORIZED,
            details: None,
        }
    }

    pub fn not_found(resource: &str, id: Option<&str>) -> Self {
        Self {
            error: "not_found".to_string(),
            message: format!("{} not found", resource),
            status_code: StatusCode::NOT_FOUND,
            details: id.map(|i| json!({ "id": i })),
        }
    }

    pub fn rate_limited(retry_after: u64) -> Self {
        Self {
            error: "rate_limited".to_string(),
            message: "Too many requests".to_string(),
            status_code: StatusCode::TOO_MANY_REQUESTS,
            details: Some(json!({ "retry_after_seconds": retry_after })),
        }
    }

    pub fn service_unavailable(message: &str, can_retry: bool) -> Self {
        Self {
            error: "service_unavailable".to_string(),
            message: message.to_string(),
            status_code: StatusCode::SERVICE_UNAVAILABLE,
            details: Some(json!({ "can_retry": can_retry })),
        }
    }

    pub fn internal(message: &str) -> Self {
        Self {
            error: "internal_error".to_string(),
            message: message.to_string(),
            status_code: StatusCode::INTERNAL_SERVER_ERROR,
            details: None,
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let body = Json(json!({
            "error": self.error,
            "message": self.message,
            "details": self.details,
        }));

        (self.status_code, body).into_response()
    }
}

// Convenience type for Result handling
pub type ApiResult<T> = Result<T, ApiError>;
