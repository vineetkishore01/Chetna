use axum::{
    extract::State,
    routing::{get, post},
    Json, Router,
};
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use crate::{Brain, config_file::UserConfig, db::embedding::EmbedderConfig};

pub fn router() -> Router<(Arc<Brain>, Arc<tokio::sync::RwLock<UserConfig>>)> {
    Router::new()
        .route("/", get(get_config).post(update_config))
        .route("/health", get(check_health))
        .route("/ping", post(ping_provider))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConfigResponse {
    pub provider: String,
    pub model: String,
    pub base_url: Option<String>,
    pub has_api_key: bool,
    pub auto_decay: bool,
}

#[derive(Debug, Deserialize)]
pub struct UpdateConfigRequest {
    pub provider: String,
    pub model: String,
    pub base_url: Option<String>,
    pub api_key: Option<String>,
    pub auto_decay: Option<bool>,
}

async fn ping_provider(
    Json(req): Json<UpdateConfigRequest>,
) -> Json<serde_json::Value> {
    let embed_config = EmbedderConfig {
        provider: req.provider.clone(),
        model: req.model.clone(),
        api_key: req.api_key.clone(),
        base_url: req.base_url.clone(),
    };
    
    // Create a temporary embedder just to test connection
    // We don't need a DB connection for a ping
    let temp_conn = Arc::new(tokio::sync::Mutex::new(rusqlite::Connection::open_in_memory().unwrap()));
    
    match crate::db::embedding::Embedder::new(
        embed_config.provider(),
        embed_config.model.clone(),
        embed_config.api_key.clone(),
        embed_config.base_url(),
        temp_conn,
    ) {
        Ok(embedder) => {
            let connected = embedder.check_connection().await.unwrap_or(false);
            Json(serde_json::json!({
                "success": connected,
                "message": if connected { "Connection successful" } else { "Could not reach provider" }
            }))
        },
        Err(e) => Json(serde_json::json!({"success": false, "message": e.to_string()}))
    }
}

async fn get_config(
    State((_, user_config)): State<(Arc<Brain>, Arc<tokio::sync::RwLock<UserConfig>>)>,
) -> Json<ConfigResponse> {
    let config = user_config.read().await;
    Json(ConfigResponse {
        provider: config.embedding_provider.clone().unwrap_or_else(|| "ollama".to_string()),
        model: config.embedding_model.clone().unwrap_or_else(|| "nomic-embed-text".to_string()),
        base_url: config.embedding_base_url.clone(),
        has_api_key: config.api_key.is_some(),
        auto_decay: config.auto_decay_enabled.unwrap_or(true),
    })
}

async fn update_config(
    State((brain, user_config)): State<(Arc<Brain>, Arc<tokio::sync::RwLock<UserConfig>>)>,
    Json(req): Json<UpdateConfigRequest>,
) -> Result<Json<serde_json::Value>, String> {
    let mut config = user_config.write().await;
    
    config.embedding_provider = Some(req.provider.clone());
    config.embedding_model = Some(req.model.clone());
    config.embedding_base_url = req.base_url.clone();
    if let Some(key) = req.api_key {
        config.api_key = Some(key);
    }
    if let Some(decay) = req.auto_decay {
        config.auto_decay_enabled = Some(decay);
    }
    
    // Persist to disk
    config.save().map_err(|e| e.to_string())?;
    
    // LIVE RELOAD: Update the brain's embedder immediately
    let embed_config = EmbedderConfig {
        provider: config.embedding_provider.clone().unwrap_or_else(|| "ollama".to_string()),
        model: config.embedding_model.clone().unwrap_or_else(|| "nomic-embed-text".to_string()),
        api_key: config.api_key.clone(),
        base_url: config.embedding_base_url.clone(),
    };
    
    brain.reload_embedder(embed_config).await.map_err(|e| format!("Failed to reload embedder: {}", e))?;
    
    Ok(Json(serde_json::json!({"success": true, "message": "Configuration updated and live-reloaded"})))
}

async fn check_health(
    State((brain, _)): State<(Arc<Brain>, Arc<tokio::sync::RwLock<UserConfig>>)>,
) -> Json<serde_json::Value> {
    let (connected, error) = brain.check_embedder_health().await;
    Json(serde_json::json!({
        "connected": connected,
        "error": error
    }))
}
