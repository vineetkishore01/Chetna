pub mod memory;
pub mod session;
pub mod skill;
pub mod procedure;
pub mod stats;
pub mod config_api;
pub mod auth;

use axum::{extract::State, Router, routing::{get, post}, Json};
use std::sync::Arc;
use crate::{Brain, cache::SessionCache, config, config_file::UserConfig};

pub fn create_router(
    brain: Arc<Brain>,
    session_cache: Arc<SessionCache>,
    user_config: Arc<tokio::sync::RwLock<UserConfig>>,
) -> Router<(Arc<Brain>, Arc<SessionCache>, Arc<tokio::sync::RwLock<UserConfig>>)> {
    let state = (brain.clone(), session_cache.clone(), user_config.clone());
    
    Router::new()
        .route("/health", get(health))
        .nest("/api/memory", memory::router(brain.clone()))
        .nest("/api/session", session::router(brain.clone()))
        .nest("/api/skill", skill::router(brain.clone()))
        .nest("/api/procedure", procedure::router(brain.clone()))
        .nest("/api/stats", stats::router(brain.clone()))
        .route("/api/config/models", get(list_models))
        .route("/api/config/cache", get(cache_stats).post(clear_cache))
        .route("/api/config/embedding/set", post(set_embedding_handler))
        .route("/api/config/embedding/test", post(test_embedding_handler))
        .route("/api/config/llm/test", post(test_llm_connection))
        .route("/api/config/llm/set", post(set_llm_handler))
        .route("/api/config/consolidation/set", post(set_consolidation_settings))
        .route("/api/config/consolidation/status", get(consolidation_status))
        .route("/api/memory/consolidate", post(manual_consolidate))
        .route("/api/memory/decay", post(run_decay))
        .route("/api/memory/flush", post(flush_low_importance))
        .route("/api/status/connections", get(connection_status))
        .route("/api/config/user", get(get_user_config).post(set_user_config))
        .route("/mcp", get(mcp_list_tools).post(mcp_handle))
        .with_state(state)
}

async fn health(State(_): State<(Arc<Brain>, Arc<SessionCache>, Arc<tokio::sync::RwLock<UserConfig>>)> ) -> &'static str {
    "OK"
}

async fn list_models(State(_): State<(Arc<Brain>, Arc<SessionCache>, Arc<tokio::sync::RwLock<UserConfig>>)> ) -> axum::Json<serde_json::Value> {
    let embedding_models = config::available_embedding_models();
    let llm_models = config::available_llm_models();

    axum::Json(serde_json::json!({
        "embedding_models": embedding_models,
        "llm_models": llm_models,
    }))
}

async fn cache_stats(
    State((_brain, cache, _)): State<(Arc<Brain>, Arc<SessionCache>, Arc<tokio::sync::RwLock<UserConfig>>)>,
) -> axum::Json<serde_json::Value> {
    let stats = cache.stats().await;
    axum::Json(serde_json::json!({
        "hits": stats.hits,
        "misses": stats.misses,
        "hit_rate": stats.hit_rate,
        "entries": stats.entries,
        "capacity": stats.capacity,
    }))
}

async fn clear_cache(
    State((_brain, cache, _)): State<(Arc<Brain>, Arc<SessionCache>, Arc<tokio::sync::RwLock<UserConfig>>)>,
) -> axum::Json<serde_json::Value> {
    cache.clear().await;
    axum::Json(serde_json::json!({
        "status": "cleared"
    }))
}

async fn mcp_list_tools(
    State((brain, _cache, _)): State<(Arc<Brain>, Arc<SessionCache>, Arc<tokio::sync::RwLock<UserConfig>>)>,
) -> axum::Json<serde_json::Value> {
    let mcp = crate::mcp::McpServer::new_with_brain(brain);
    axum::Json(serde_json::json!({
        "tools": mcp.list_tools()
    }))
}

async fn mcp_handle(
    State((brain, _cache, _)): State<(Arc<Brain>, Arc<SessionCache>, Arc<tokio::sync::RwLock<UserConfig>>)>,
    axum::Json(req): axum::Json<crate::mcp::McpRequest>,
) -> axum::Json<crate::mcp::McpResponse> {
    let mcp = crate::mcp::McpServer::new_with_brain(brain);
    let response = mcp.handle_request(req).await;
    axum::Json(response)
}

async fn connection_status(
    State((brain, _cache, user_config)): State<(Arc<Brain>, Arc<SessionCache>, Arc<tokio::sync::RwLock<UserConfig>>)>,
) -> axum::Json<serde_json::Value> {
    let has_embedder = brain.has_embedder();
    let config_guard = user_config.read().await;

    // Try to get models from Ollama if embedder is configured
    let mut embedding_connected = false;
    let mut embedding_model_name = String::from("Not configured");
    let mut available_models: Vec<String> = Vec::new();
    let mut model_installed = false;

    // Use user_config for base_url and model
    let base_url = config_guard.embedding_base_url.clone().unwrap_or_else(|| "http://localhost:11434".to_string());
    let model = config_guard.embedding_model.clone().unwrap_or_else(|| "nomic-embed-text".to_string());

    if has_embedder {
        if let Ok(resp) = reqwest::get(format!("{}/api/tags", base_url)).await {
            if resp.status().is_success() {
                if let Ok(json) = resp.json::<serde_json::Value>().await {
                    embedding_connected = true;

                    if let Some(models) = json.get("models").and_then(|m| m.as_array()) {
                        for model_entry in models {
                            if let Some(name) = model_entry.get("name").and_then(|n| n.as_str()) {
                                available_models.push(name.to_string());

                                if name == model {
                                    model_installed = true;
                                }
                            }
                        }
                    }

                    embedding_model_name = if model_installed {
                        model.clone()
                    } else {
                        format!("{} (not installed)", model)
                    };
                }
            }
        }
    }

    
    let mut llm_connected = false;
    let mut llm_model_name = String::from("Not configured");

    let llm_base_url = config_guard.llm_base_url.clone().unwrap_or_else(|| "http://localhost:11434".to_string());
    let llm_model = config_guard.llm_model.clone().unwrap_or_else(|| "llama3.2".to_string());

    let llm_configured = config_guard.llm_model.is_some();
    if llm_configured {
        if let Ok(resp) = reqwest::get(format!("{}/api/tags", llm_base_url)).await {
            if resp.status().is_success() {
                llm_connected = true;
                llm_model_name = llm_model.clone();
            }
        }
    }

    drop(config_guard);

    axum::Json(serde_json::json!({
        "embedding": {
            "configured": has_embedder,
            "connected": embedding_connected,
            "model_installed": model_installed,
            "model": embedding_model_name,
            "available_models": available_models,
            "base_url": base_url,
        },
        "llm": {
            "configured": llm_configured,
            "connected": llm_connected,
            "model": llm_model_name,
            "base_url": llm_base_url,
        }
    }))
}

async fn get_user_config(
    State((_brain, _cache, user_config)): State<(Arc<Brain>, Arc<SessionCache>, Arc<tokio::sync::RwLock<UserConfig>>)>,
) -> axum::Json<serde_json::Value> {
    let config = user_config.read().await;
    axum::Json(serde_json::json!({
        "embedding_base_url": config.embedding_base_url,
        "embedding_model": config.embedding_model,
        "llm_base_url": config.llm_base_url,
        "llm_model": config.llm_model,
        "consolidation_interval_hours": config.consolidation_interval_hours,
        "auto_decay_enabled": config.auto_decay_enabled,
        "auto_flush_enabled": config.auto_flush_enabled,
        "min_importance_threshold": config.min_importance_threshold,
    }))
}

async fn set_user_config(
    State((_brain, _cache, user_config)): State<(Arc<Brain>, Arc<SessionCache>, Arc<tokio::sync::RwLock<UserConfig>>)>,
    Json(req): Json<serde_json::Value>,
) -> axum::Json<serde_json::Value> {
    let mut config = user_config.write().await;

    if let Some(url) = req.get("embedding_base_url").and_then(|v| v.as_str()) {
        config.embedding_base_url = Some(url.to_string());
    }
    if let Some(model) = req.get("embedding_model").and_then(|v| v.as_str()) {
        config.embedding_model = Some(model.to_string());
    }
    if let Some(url) = req.get("llm_base_url").and_then(|v| v.as_str()) {
        config.llm_base_url = Some(url.to_string());
    }
    if let Some(model) = req.get("llm_model").and_then(|v| v.as_str()) {
        config.llm_model = Some(model.to_string());
    }
    if let Some(interval) = req.get("consolidation_interval_hours").and_then(|v| v.as_i64()) {
        config.consolidation_interval_hours = Some(interval as i32);
    }
    if let Some(enabled) = req.get("auto_decay_enabled").and_then(|v| v.as_bool()) {
        config.auto_decay_enabled = Some(enabled);
    }
    if let Some(enabled) = req.get("auto_flush_enabled").and_then(|v| v.as_bool()) {
        config.auto_flush_enabled = Some(enabled);
    }
    if let Some(threshold) = req.get("min_importance_threshold").and_then(|v| v.as_f64()) {
        config.min_importance_threshold = Some(threshold);
    }

    if let Err(e) = config.save() {
        return axum::Json(serde_json::json!({
            "success": false,
            "message": format!("Failed to save config: {}", e)
        }));
    }

    // Also sync to .env for backward compatibility
    if let Err(e) = crate::config_file::sync_to_env(&config) {
        tracing::warn!("Failed to sync config to .env: {}", e);
    }

    axum::Json(serde_json::json!({
        "success": true,
        "message": "Settings saved successfully. Some changes may require restart."
    }))
}

async fn set_embedding_handler(
    State((_brain, _cache, user_config)): State<(Arc<Brain>, Arc<SessionCache>, Arc<tokio::sync::RwLock<UserConfig>>)>,
    Json(req): Json<serde_json::Value>,
) -> axum::Json<serde_json::Value> {
    let model = req.get("model").and_then(|m| m.as_str()).unwrap_or("");
    let base_url = req.get("base_url").and_then(|b| b.as_str()).unwrap_or("http://localhost:11434");

    if model.is_empty() {
        return axum::Json(serde_json::json!({
            "success": false,
            "message": "Model name is required"
        }));
    }

    // Update user config
    let mut config = user_config.write().await;
    config.embedding_model = Some(model.to_string());
    config.embedding_base_url = Some(base_url.to_string());

    if let Err(e) = config.save() {
        return axum::Json(serde_json::json!({
            "success": false,
            "message": format!("Failed to save configuration: {}", e)
        }));
    }

    // Sync to .env
    if let Err(e) = crate::config_file::sync_to_env(&config) {
        tracing::warn!("Failed to sync to .env: {}", e);
    }

    axum::Json(serde_json::json!({
        "success": true,
        "message": format!("Model set to {}. Restart Chetna to apply changes.", model),
        "model": model,
        "base_url": base_url
    }))
}

async fn test_embedding_handler(
    Json(req): Json<serde_json::Value>,
) -> axum::Json<serde_json::Value> {
    let base_url = req.get("base_url").and_then(|b| b.as_str()).unwrap_or("http://localhost:11434");
    let model = req.get("model").and_then(|m| m.as_str()).unwrap_or("");
    
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .unwrap_or_default();
    
    let models_result = client
        .get(format!("{}/api/tags", base_url))
        .send()
        .await;
    
    match models_result {
        Ok(resp) if resp.status().is_success() => {
            let json = match resp.json::<serde_json::Value>().await {
                Ok(j) => j,
                Err(e) => {
                    return axum::Json(serde_json::json!({
                        "success": false,
                        "connected": false,
                        "message": format!("Failed to parse Ollama response: {}", e)
                    }));
                }
            };
            
            let mut available_models: Vec<String> = Vec::new();
            let mut model_installed = false;
            
            if let Some(models) = json.get("models").and_then(|m| m.as_array()) {
                for m in models {
                    if let Some(name) = m.get("name").and_then(|n| n.as_str()) {
                        available_models.push(name.to_string());
                        if !model.is_empty() && name == model {
                            model_installed = true;
                        }
                    }
                }
            }
                
                // If no model specified, just return available models
                if model.is_empty() {
                    return axum::Json(serde_json::json!({
                        "success": true,
                        "connected": true,
                        "available_models": available_models,
                        "model_installed": false,
                        "embedding_works": false,
                        "message": if !available_models.is_empty() {
                            format!("Found {} models. Select one to test.", available_models.len())
                        } else {
                            "Connected but no models found".to_string()
                        }
                    }));
                }
                
                // If model specified, test embedding
                let mut embedding_works = false;
                if model_installed {
                    let embed_resp = client
                        .post(format!("{}/api/embeddings", base_url))
                        .json(&serde_json::json!({
                            "model": model,
                            "prompt": "test"
                        }))
                        .send()
                        .await;
                    
                    if let Ok(er) = embed_resp {
                        if er.status().is_success() {
                            embedding_works = true;
                        }
                    }
                }
                
                axum::Json(serde_json::json!({
                    "success": true,
                    "connected": true,
                    "available_models": available_models,
                    "model_installed": model_installed,
                    "embedding_works": embedding_works,
                    "message": if embedding_works {
                        format!("✓ Model '{}' is installed and embedding works!", model)
                    } else if model_installed {
                        format!("Model '{}' is installed but embedding test failed", model)
                    } else if !model.is_empty() {
                        format!("Model '{}' is not installed. Select from available models.", model)
                    } else {
                        "Connected to Ollama. Select a model to test.".to_string()
                    }
                }))
        }
        Ok(resp) => {
            axum::Json(serde_json::json!({
                "success": false,
                "connected": false,
                "message": format!("Ollama returned error: {}", resp.status())
            }))
        }
        Err(e) => {
            axum::Json(serde_json::json!({
                "success": false,
                "connected": false,
                "message": format!("Cannot connect to Ollama at {}: {}", base_url, e)
            }))
        }
    }
}

async fn test_llm_connection(
    Json(req): Json<serde_json::Value>,
) -> axum::Json<serde_json::Value> {
    let base_url = req.get("base_url").and_then(|b| b.as_str()).unwrap_or("http://localhost:11434");
    let model = req.get("model").and_then(|m| m.as_str()).unwrap_or("");
    
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .unwrap_or_default();
    
    let models_result = client
        .get(format!("{}/api/tags", base_url))
        .send()
        .await;
    
    match models_result {
        Ok(resp) if resp.status().is_success() => {
            if let Ok(json) = resp.json::<serde_json::Value>().await {
                let mut available_models: Vec<String> = Vec::new();
                let mut model_installed = false;
                
                if let Some(models) = json.get("models").and_then(|m| m.as_array()) {
                    for m in models {
                        if let Some(name) = m.get("name").and_then(|n| n.as_str()) {
                            available_models.push(name.to_string());
                            if !model.is_empty() && name == model {
                                model_installed = true;
                            }
                        }
                    }
                }
                
                // If no model specified, just return available models
                if model.is_empty() {
                    return axum::Json(serde_json::json!({
                        "success": true,
                        "connected": true,
                        "available_models": available_models,
                        "model_installed": false,
                        "llm_works": false,
                        "message": if !available_models.is_empty() {
                            format!("Found {} models. Select one to test.", available_models.len())
                        } else {
                            "Connected but no models found".to_string()
                        }
                    }));
                }
                
                let mut llm_works = false;
                if model_installed {
                    let chat_resp = client
                        .post(format!("{}/api/chat", base_url))
                        .json(&serde_json::json!({
                            "model": model,
                            "messages": [{"role": "user", "content": "Hi"}],
                            "stream": false
                        }))
                        .send()
                        .await;
                    
                    if let Ok(cr) = chat_resp {
                        if cr.status().is_success() {
                            llm_works = true;
                        }
                    }
                }
                
                axum::Json(serde_json::json!({
                    "success": true,
                    "connected": true,
                    "available_models": available_models,
                    "model_installed": model_installed,
                    "llm_works": llm_works,
                    "message": if llm_works {
                        format!("✓ Model '{}' is installed and LLM works!", model)
                    } else if model_installed {
                        format!("Model '{}' is installed but chat test failed", model)
                    } else if !model.is_empty() {
                        format!("Model '{}' is not installed", model)
                    } else {
                        "Connected to Ollama. Select a model to test.".to_string()
                    }
                }))
            } else {
                axum::Json(serde_json::json!({
                    "success": false,
                    "connected": false,
                    "message": "Failed to parse Ollama response"
                }))
            }
        }
        Ok(resp) => {
            axum::Json(serde_json::json!({
                "success": false,
                "connected": false,
                "message": format!("Ollama returned error: {}", resp.status())
            }))
        }
        Err(e) => {
            axum::Json(serde_json::json!({
                "success": false,
                "connected": false,
                "message": format!("Cannot connect to Ollama at {}: {}", base_url, e)
            }))
        }
    }
}

async fn set_llm_handler(
    State((_brain, _cache, user_config)): State<(Arc<Brain>, Arc<SessionCache>, Arc<tokio::sync::RwLock<UserConfig>>)>,
    Json(req): Json<serde_json::Value>,
) -> axum::Json<serde_json::Value> {
    let model = req.get("model").and_then(|m| m.as_str()).unwrap_or("");
    let base_url = req.get("base_url").and_then(|b| b.as_str()).unwrap_or("http://localhost:11434");

    if model.is_empty() {
        return axum::Json(serde_json::json!({
            "success": false,
            "message": "Model name is required"
        }));
    }

    // Update user config
    let mut config = user_config.write().await;
    config.llm_model = Some(model.to_string());
    config.llm_base_url = Some(base_url.to_string());

    if let Err(e) = config.save() {
        return axum::Json(serde_json::json!({
            "success": false,
            "message": format!("Failed to save configuration: {}", e)
        }));
    }

    // Sync to .env
    if let Err(e) = crate::config_file::sync_to_env(&config) {
        tracing::warn!("Failed to sync to .env: {}", e);
    }

    axum::Json(serde_json::json!({
        "success": true,
        "message": format!("Model set to {}. Restart Chetna to apply changes.", model),
        "model": model,
        "base_url": base_url
    }))
}

async fn set_consolidation_settings(
    State((_brain, _cache, user_config)): State<(Arc<Brain>, Arc<SessionCache>, Arc<tokio::sync::RwLock<UserConfig>>)>,
    Json(req): Json<serde_json::Value>,
) -> axum::Json<serde_json::Value> {
    let interval = req.get("interval_hours").and_then(|i| i.as_i64()).unwrap_or(6);
    let auto_decay = req.get("auto_decay_enabled").and_then(|b| b.as_bool()).unwrap_or(true);
    let auto_flush = req.get("auto_flush_enabled").and_then(|b| b.as_bool()).unwrap_or(true);
    let threshold = req.get("min_importance_threshold").and_then(|t| t.as_f64()).unwrap_or(0.1);

    // Update user config
    let mut config = user_config.write().await;
    config.consolidation_interval_hours = Some(interval as i32);
    config.auto_decay_enabled = Some(auto_decay);
    config.auto_flush_enabled = Some(auto_flush);
    config.min_importance_threshold = Some(threshold);

    if let Err(e) = config.save() {
        return axum::Json(serde_json::json!({
            "success": false,
            "message": format!("Failed to save configuration: {}", e)
        }));
    }

    // Sync to .env
    if let Err(e) = crate::config_file::sync_to_env(&config) {
        tracing::warn!("Failed to sync to .env: {}", e);
    }

    axum::Json(serde_json::json!({
        "success": true,
        "message": "Consolidation settings saved. Restart to apply changes.",
        "settings": {
            "consolidation_interval_hours": interval,
            "auto_decay_enabled": auto_decay,
            "auto_flush_enabled": auto_flush,
            "min_importance_threshold": threshold
        }
    }))
}

async fn consolidation_status(
    State((_brain, _cache, user_config)): State<(Arc<Brain>, Arc<SessionCache>, Arc<tokio::sync::RwLock<UserConfig>>)>,
) -> axum::Json<serde_json::Value> {
    let config = user_config.read().await;
    axum::Json(serde_json::json!({
        "consolidation_interval_hours": config.consolidation_interval_hours.unwrap_or(6),
        "auto_decay_enabled": config.auto_decay_enabled.unwrap_or(true),
        "auto_flush_enabled": config.auto_flush_enabled.unwrap_or(true),
        "min_importance_threshold": config.min_importance_threshold.unwrap_or(0.1),
        "llm_configured": config.llm_model.is_some(),
        "llm_model": config.llm_model.clone().unwrap_or_else(|| "Not configured".to_string()),
        "llm_base_url": config.llm_base_url.clone().unwrap_or_else(|| "http://localhost:11434".to_string()),
    }))
}

async fn manual_consolidate(
    State((brain, _cache, _)): State<(Arc<Brain>, Arc<SessionCache>, Arc<tokio::sync::RwLock<UserConfig>>)>,
    Json(req): Json<serde_json::Value>,
) -> axum::Json<serde_json::Value> {
    let limit = req.get("limit").and_then(|l| l.as_i64()).unwrap_or(100);

    match brain.consolidate_memories_llm(limit).await {
        Ok(result) => axum::Json(serde_json::json!({
            "success": true,
            "memories_processed": result.0,
            "memories_updated": result.1,
            "message": format!("Processed {} memories, updated importance for {} using LLM", result.0, result.1)
        })),
        Err(e) => axum::Json(serde_json::json!({
            "success": false,
            "message": format!("Error: {}", e)
        }))
    }
}

async fn run_decay(
    State((brain, _cache, _)): State<(Arc<Brain>, Arc<SessionCache>, Arc<tokio::sync::RwLock<UserConfig>>)>,
) -> axum::Json<serde_json::Value> {
    match brain.apply_decay_formula().await {
        Ok(count) => axum::Json(serde_json::json!({
            "success": true,
            "memories_updated": count,
            "message": format!("Applied Ebbinghaus decay formula to {} memories", count)
        })),
        Err(e) => axum::Json(serde_json::json!({
            "success": false,
            "message": format!("Error: {}", e)
        }))
    }
}

async fn flush_low_importance(
    State((brain, _cache, _)): State<(Arc<Brain>, Arc<SessionCache>, Arc<tokio::sync::RwLock<UserConfig>>)>,
    Json(req): Json<serde_json::Value>,
) -> axum::Json<serde_json::Value> {
    let threshold = req.get("threshold").and_then(|t| t.as_f64()).unwrap_or(0.1);

    match brain.flush_low_importance(threshold).await {
        Ok(count) => axum::Json(serde_json::json!({
            "success": true,
            "memories_deleted": count,
            "message": format!("Flushed {} memories below importance {}", count, threshold)
        })),
        Err(e) => axum::Json(serde_json::json!({
            "success": false,
            "message": format!("Error: {}", e)
        }))
    }
}
