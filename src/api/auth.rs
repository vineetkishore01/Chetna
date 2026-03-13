//! Authentication middleware for Chetna API

use axum::{
    extract::Request,
    http::header::AUTHORIZATION,
    http::StatusCode,
    middleware::Next,
    response::Response,
};

pub async fn api_key_auth(mut req: Request, next: Next) -> Result<Response, StatusCode> {
    // Check for API key in header
    let api_key = req
        .headers()
        .get(AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .map(|s| s.to_string());

    // If no API key provided, check query param as fallback
    if api_key.is_none() {
        if let Some(query_key) = axum::extract::Query::<std::collections::HashMap<String, String>>::from_request(req.extensions().get::<axum::extract::State<()>>().cloned().unwrap_or_default(), &mut req).await.ok() {
            if let Some(key) = query_key.get("api_key") {
                api_key.clone_from(&Some(key.clone()));
            }
        }
    }

    // For now, we allow requests without API key in development
    // In production, you would validate against configured API keys
    if let Some(key) = api_key {
        // Store the API key in extensions for logging/auditing
        req.extensions_mut().insert(key);
    }

    Ok(next.run(req).await)
}

pub fn check_api_key(request_key: Option<&str>, configured_key: Option<&str>) -> bool {
    match (request_key, configured_key) {
        (Some(req), Some(cfg)) => req == cfg,
        (None, None) => true, // No auth configured
        _ => false,
    }
}
