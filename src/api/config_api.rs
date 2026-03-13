use axum::{
    extract::State,
    routing::{get, post},
    Json, Router,
};
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use crate::{Brain, cache::SessionCache};

pub fn router(_brain: Arc<Brain>) -> Router<(Arc<Brain>, Arc<SessionCache>)> {
    Router::new()
        .route("/", get(get_config))
        .route("/embedding/set", post(set_embedding_model))
        .route("/llm/set", post(set_llm_model))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConfigResponse {
    pub embedding_provider: String,
    pub embedding_model: String,
    pub embedding_dimensions: Option<usize>,
    pub llm_provider: String,
    pub llm_model: String,
    pub use_lancedb: bool,
    pub session_cache_size: usize,
}

async fn get_config() -> Json<ConfigResponse> {
    // In production, get from actual config
    Json(ConfigResponse {
        embedding_provider: "ollama".to_string(),
        embedding_model: "nomic-embed-text".to_string(),
        embedding_dimensions: Some(768),
        llm_provider: "ollama".to_string(),
        llm_model: "llama3.2".to_string(),
        use_lancedb: false,
        session_cache_size: 100,
    })
}

#[derive(Debug, Deserialize)]
pub struct SetModelRequest {
    pub provider: String,
    pub model: String,
    pub api_key: Option<String>,
    pub base_url: Option<String>,
}

async fn set_embedding_model(
    State((_brain, _)): State<(Arc<Brain>, Arc<SessionCache>)>,
    Json(req): Json<SetModelRequest>,
) -> Json<serde_json::Value> {
    // Save to config/database
    // Restart embedder with new model
    Json(serde_json::json!({
        "status": "set",
        "provider": req.provider,
        "model": req.model,
        "message": "Restart required for changes to take effect"
    }))
}

async fn set_llm_model(
    State((_brain, _)): State<(Arc<Brain>, Arc<SessionCache>)>,
    Json(req): Json<SetModelRequest>,
) -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "set",
        "provider": req.provider,
        "model": req.model,
        "message": "Restart required for changes to take effect"
    }))
}
