use axum::{
    extract::State,
    routing::get,
    Json, Router,
};
use std::sync::Arc;
use serde::Serialize;
use crate::{Brain, config_file::UserConfig};

pub fn router(_brain: Arc<Brain>) -> Router<(Arc<Brain>, Arc<tokio::sync::RwLock<UserConfig>>)> {
    Router::new()
        .route("/", get(get_stats))
        
}

#[derive(Debug, Serialize)]
pub struct StatsResponse {
    pub total_memories: i64,
    pub active_memories: i64,
    pub deleted_memories: i64,
    pub total_sessions: i64,
    pub active_sessions: i64,
    pub avg_importance: f32,
    pub memory_types: std::collections::HashMap<String, i64>,
}

async fn get_stats(State((brain, _)): State<(Arc<Brain>, Arc<tokio::sync::RwLock<UserConfig>>)>) -> Json<StatsResponse> {
    let stats = brain.get_stats().await.unwrap_or(StatsResponse {
        total_memories: 0,
        active_memories: 0,
        deleted_memories: 0,
        total_sessions: 0,
        active_sessions: 0,
        avg_importance: 0.0,
        memory_types: std::collections::HashMap::new(),
    });
    Json(stats)
}
