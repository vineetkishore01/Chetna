//! Authentication middleware for Chetna API
//!
//! Provides optional API key authentication for securing the service.
//! Supports both environment variable (CHETNA_API_KEY) and user config file.
//! Controlled by CHETNA_AUTH_REQUIRED - if not set or false, auth is skipped (dev mode).

use axum::{
    extract::State,
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use std::sync::Arc;
use tokio::sync::RwLock;
use crate::config_file::UserConfig;

fn timing_safe_eq(a: &str, b: &str) -> bool {
    use std::hint::black_box;
    if a.len() != b.len() {
        return false;
    }
    let a_bytes = a.as_bytes();
    let b_bytes = b.as_bytes();
    let mut result = 0u8;
    for i in 0..a_bytes.len() {
        result |= a_bytes[i] ^ b_bytes[i];
        black_box(result);
    }
    result == 0
}

pub type SharedConfig = Arc<RwLock<UserConfig>>;

static AUTH_REQUIRED: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
static ENV_API_KEY: std::sync::OnceLock<Option<String>> = std::sync::OnceLock::new();

pub async fn api_key_auth(
    State(user_config): State<SharedConfig>,
    req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let uri = req.uri();

    // Public paths that don't require authentication
    let public_paths = [
        "/health",
        "/",
        "/docs",
        "/index.html",
        "/api/status",
        "/api/capabilities",
    ];
    
    // Check if this is a public path
    let is_public = public_paths.iter().any(|p| uri.path() == *p) 
        || uri.path().starts_with("/static")
        || uri.path() == "/api/memory/search";  // Read-only search is public
    
    if is_public {
        return Ok(next.run(req).await);
    }

    // Check if auth is required (opt-in)
    let auth_required = *AUTH_REQUIRED.get_or_init(|| {
        std::env::var("CHETNA_AUTH_REQUIRED")
            .map(|v| v.to_lowercase() == "true")
            .unwrap_or(false)
    });

    // If auth not required, skip authentication (development mode)
    if !auth_required {
        return Ok(next.run(req).await);
    }

    // Auth is required - check for API key
    // Environment variable takes precedence
    let configured_key = ENV_API_KEY.get_or_init(|| {
        std::env::var("CHETNA_API_KEY")
            .ok()
            .filter(|k| !k.is_empty())
    }).clone()
        .or_else(|| {
            match user_config.try_read() {
                Ok(config_guard) => config_guard.api_key.clone().filter(|k| !k.is_empty()),
                Err(_) => None,
            }
        });

    // If auth required but no key configured, reject all requests
    if configured_key.is_none() {
        tracing::error!("CHETNA_AUTH_REQUIRED=true but no API key configured!");
        return Err(StatusCode::UNAUTHORIZED);
    }

    // Check for API key in header
    let api_key = req
        .headers()
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .map(|s| s.to_string());

    // Validate API key
    if let Some(key) = api_key {
        if let Some(ref configured) = configured_key {
            if timing_safe_eq(&key, configured) {
                return Ok(next.run(req).await);
            }
        }
    }

    // Invalid or missing API key
    Err(StatusCode::UNAUTHORIZED)
}

pub fn check_api_key(request_key: Option<&str>, configured_key: Option<&str>) -> bool {
    match (request_key, configured_key) {
        (Some(req), Some(cfg)) => timing_safe_eq(req, cfg),
        (None, None) => true, // No auth configured
        _ => false,
    }
}
