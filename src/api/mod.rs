pub mod memory;
pub mod session;
pub mod stats;
pub mod auth;
pub mod error;
pub mod config;
pub mod history;

use axum::{extract::State, Router, routing::get, response::Sse};
use futures::stream::Stream;
use std::convert::Infallible;
use std::sync::Arc;
use std::time::Duration;
use crate::{Brain, config_file::UserConfig, START_TIME};
use crate::api::auth::api_key_auth;

fn create_status_stream(
    brain: Arc<Brain>,
    model: String,
) -> Sse<impl Stream<Item = Result<axum::response::sse::Event, Infallible>>> {
    // Use a simple stream that yields one initial event
    // The client will poll /api/status/connections for continuous updates
    // This keeps SSE simple while dashboard already polls every 30s
    let stream = futures::stream::once(async move {
        let (embedding_connected, _) = brain.check_embedder_health().await;
        let embedding_model_name = model.clone();
        
        let state = brain.get_connection_state().await;
        
        let event_data = format!(
            r#"{{"embedding_connected":{},"embedding_model":"{}","circuit_breaker_open":{},"timestamp":"{}"}}"#,
            embedding_connected,
            embedding_model_name.replace('"', "'"),
            state.consecutive_failures >= 3,
            chrono::Utc::now().to_rfc3339()
        );
        
        Ok::<_, Infallible>(axum::response::sse::Event::default().data(event_data))
    });
    
    Sse::new(stream)
}

pub fn create_router(
    brain: Arc<Brain>,
    user_config: Arc<tokio::sync::RwLock<UserConfig>>,
) -> Router<(Arc<Brain>, Arc<tokio::sync::RwLock<UserConfig>>)> {
    let state = (brain.clone(), user_config.clone());

    Router::new()
        .route("/health", get(health))
        .nest("/api/memory", memory::router(brain.clone()))
        .nest("/api/session", session::router(brain.clone()))
        .nest("/api/stats", stats::router(brain.clone()))
        .nest("/api/config", config::router())
        .nest("/api/history", history::router())
        .route("/api/status/connections", get(connection_status))
        .route("/api/status/stream", get(connection_status_stream))
        .route("/api/capabilities", get(capabilities))
        .route("/mcp", get(mcp_list_tools).post(mcp_handle))
        .layer(axum::middleware::from_fn_with_state(user_config.clone(), api_key_auth))
        .with_state(state)
}

async fn health(
    State((brain, user_config)): State<(Arc<Brain>, Arc<tokio::sync::RwLock<UserConfig>>)>,
) -> axum::Json<serde_json::Value> {
    let uptime_seconds = START_TIME
        .get()
        .map(|t| t.elapsed().as_secs() as i64)
        .unwrap_or(0);

    let config_guard = user_config.read().await;
    let has_embedder = brain.has_embedder().await;
    drop(config_guard);

    let (embedding_connected, _) = brain.check_embedder_health().await;

    let status = if embedding_connected {
        "healthy"
    } else if has_embedder {
        "degraded"
    } else {
        "healthy"  // No embedder configured is still healthy (keyword-only mode)
    };

    axum::Json(serde_json::json!({
        "status": status,
        "version": "0.1.0",
        "database": "connected",
        "embedding": if embedding_connected { "connected" } else { "disconnected" },
        "uptime_seconds": uptime_seconds,
        "message": if embedding_connected {
            "All systems operational"
        } else if has_embedder {
            "Semantic search unavailable. Keyword search works."
        } else {
            "Running in keyword-only mode"
        }
    }))
}

async fn mcp_list_tools(
    State((brain, _)): State<(Arc<Brain>, Arc<tokio::sync::RwLock<UserConfig>>)>,
) -> axum::Json<serde_json::Value> {
    let mcp = crate::mcp::McpServer::new_with_brain(brain);
    axum::Json(serde_json::json!({
        "tools": mcp.list_tools()
    }))
}

async fn mcp_handle(
    State((brain, _)): State<(Arc<Brain>, Arc<tokio::sync::RwLock<UserConfig>>)>,
    axum::Json(req): axum::Json<crate::mcp::McpRequest>,
) -> axum::Json<crate::mcp::McpResponse> {
    let mcp = crate::mcp::McpServer::new_with_brain(brain);
    let response = mcp.handle_request(req).await;
    axum::Json(response)
}

async fn connection_status(
    State((brain, user_config)): State<(Arc<Brain>, Arc<tokio::sync::RwLock<UserConfig>>)>,
) -> axum::Json<serde_json::Value> {
    let has_embedder = brain.has_embedder().await;
    let config_guard = user_config.read().await;

    let mut embedding_model_name = String::from("Not configured");
    let mut available_models: Vec<String> = Vec::new();
    let mut model_installed = false;

    let provider = config_guard.embedding_provider.clone().unwrap_or_else(|| "ollama".to_string());
    let base_url = config_guard.embedding_base_url.clone().unwrap_or_else(|| "http://localhost:11434".to_string());
    let model = config_guard.embedding_model.clone().unwrap_or_else(|| "nomic-embed-text".to_string());

    let (embedding_connected, _) = brain.check_embedder_health().await;

    if embedding_connected && provider == "ollama" {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(5))
            .build()
            .unwrap_or_default();
        if let Ok(resp) = client.get(format!("{}/api/tags", base_url)).send().await {
            if resp.status().is_success() {
                if let Ok(json) = resp.json::<serde_json::Value>().await {
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
                }
            }
        }
    } else if embedding_connected {
        // Non-ollama providers don't have local installation checks
        model_installed = true;
    }

    if !model_installed && provider == "ollama" {
        embedding_model_name = format!("{} (not installed)", model);
    } else if embedding_connected {
        embedding_model_name = model.clone();
    }

    axum::Json(serde_json::json!({
        "embedding": {
            "configured": has_embedder,
            "connected": embedding_connected,
            "model_installed": model_installed,
            "model": embedding_model_name,
            "available_models": available_models,
            "base_url": base_url,
        }
    }))
}

async fn connection_status_stream(
    State((brain, user_config)): State<(Arc<Brain>, Arc<tokio::sync::RwLock<UserConfig>>)>,
) -> Sse<impl Stream<Item = Result<axum::response::sse::Event, Infallible>>> {
    let model = {
        let config_guard = user_config.read().await;
        config_guard.embedding_model.clone().unwrap_or_else(|| "nomic-embed-text".to_string())
    };

    create_status_stream(brain, model)
}

async fn capabilities(
    State((brain, user_config)): State<(Arc<Brain>, Arc<tokio::sync::RwLock<UserConfig>>)>,
) -> axum::Json<serde_json::Value> {
    let has_embedder = brain.has_embedder().await;
    let config_guard = user_config.read().await;
    
    let model = config_guard.embedding_model.clone().unwrap_or_else(|| "nomic-embed-text".to_string());
    drop(config_guard);
    
    let (embedding_connected, _) = brain.check_embedder_health().await;
    
    axum::Json(serde_json::json!({
        "version": "0.1.0",
        "features": {
            "semantic_search": embedding_connected,
            "context_building": embedding_connected,
            "mcp_tools": true,
            "batch_operations": true,
            "pinning": true,
            "categories": true,
            "emotional_memory": true,
        },
        "models": {
            "embedding": {
                "configured": has_embedder,
                "connected": embedding_connected,
                "model": model,
            }
        },
        "endpoints": {
            "health": "/health",
            "memories": "/api/memory",
            "search": "/api/memory/search",
            "context": "/api/memory/context",
            "stats": "/api/stats",
            "mcp": "/mcp",
        }
    }))
}
