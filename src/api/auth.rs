//! Authentication middleware for Chetna API
//!
//! Provides optional API key authentication for securing the service.

use axum::{
    extract::Request,
    http::header::AUTHORIZATION,
    http::StatusCode,
    middleware::Next,
    response::Response,
};

pub async fn api_key_auth(req: Request, next: Next) -> Result<Response, StatusCode> {
    let uri = req.uri();
    
    // Skip auth for health check, root, and static content
    let public_paths = ["/health", "/", "/memories", "/skills", "/sessions", "/settings"];
    if public_paths.iter().any(|p| uri.path() == *p) {
        return Ok(next.run(req).await);
    }
    
    // Check for API key in header
    let api_key = req
        .headers()
        .get(AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .map(|s| s.to_string());

    // Get configured API key from extensions
    let configured_key = req
        .extensions()
        .get::<String>()
        .cloned();

    // If no API key is configured, allow all requests (development mode)
    if configured_key.is_none() {
        return Ok(next.run(req).await);
    }

    // Validate API key
    if let Some(key) = api_key {
        if key == configured_key.unwrap() {
            return Ok(next.run(req).await);
        }
    }

    // Invalid or missing API key
    Err(StatusCode::UNAUTHORIZED)
}

pub fn check_api_key(request_key: Option<&str>, configured_key: Option<&str>) -> bool {
    match (request_key, configured_key) {
        (Some(req), Some(cfg)) => req == cfg,
        (None, None) => true, // No auth configured
        _ => false,
    }
}
