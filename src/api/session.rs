use axum::{
    extract::{State, Query},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use crate::{Brain, config_file::UserConfig};

pub fn router(_brain: Arc<Brain>) -> Router<(Arc<Brain>, Arc<tokio::sync::RwLock<UserConfig>>)> {
    Router::new()
        .route("/", get(list_sessions).post(create_session))
        .route("/:id", get(get_session).delete(delete_session))
        .route("/:id/end", post(end_session))

}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateSessionRequest {
    pub name: String,
    pub agent_id: Option<String>,
    pub project: Option<String>,
    pub directory: Option<String>,
    pub namespace: Option<String>,
}

#[derive(Deserialize)]
pub struct SessionListParams {
    pub limit: Option<i64>,
    pub namespace: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SessionResponse {
    pub id: String,
    pub namespace: String,
    pub name: String,
    pub agent_id: Option<String>,
    pub project: Option<String>,
    pub directory: Option<String>,
    pub started_at: String,
    pub ended_at: Option<String>,
}

async fn list_sessions(
    State((brain, _)): State<(Arc<Brain>, Arc<tokio::sync::RwLock<UserConfig>>)>,
    Query(params): Query<SessionListParams>,
) -> Result<Json<Vec<SessionResponse>>, StatusCode> {
    let limit = params.limit.unwrap_or(50);
    let namespace = params.namespace.as_deref();
    
    let sessions = brain.list_sessions(limit, namespace).await.map_err(|e| {
        tracing::error!("Failed to list sessions: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    Ok(Json(sessions.into_iter().map(|s| SessionResponse {
        id: s.id,
        namespace: s.namespace,
        name: s.name,
        agent_id: s.agent_id,
        project: s.project,
        directory: s.directory,
        started_at: s.started_at,
        ended_at: s.ended_at,
    }).collect()))
}

async fn get_session(
    State((brain, _)): State<(Arc<Brain>, Arc<tokio::sync::RwLock<UserConfig>>)>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Result<Json<SessionResponse>, String> {
    let session = brain.get_session(&id).await.map_err(|e| e.to_string())?;
    Ok(Json(SessionResponse {
        id: session.id,
        namespace: session.namespace,
        name: session.name,
        agent_id: session.agent_id,
        project: session.project,
        directory: session.directory,
        started_at: session.started_at,
        ended_at: session.ended_at,
    }))
}

async fn create_session(
    State((brain, _)): State<(Arc<Brain>, Arc<tokio::sync::RwLock<UserConfig>>)>,
    Json(req): Json<CreateSessionRequest>,
) -> Result<Json<SessionResponse>, String> {
    let name = req.name.trim();
    if name.is_empty() {
        return Err("Session name cannot be empty".to_string());
    }
    if name.len() > 255 {
        return Err("Session name exceeds maximum length of 255 characters".to_string());
    }
    
    let id = brain.create_session(
        name, 
        req.agent_id.as_deref(), 
        req.project.as_deref(), 
        req.directory.as_deref(), 
        req.namespace.as_deref()
    )
    .await
    .map_err(|e| e.to_string())?;
    
    let session = brain.get_session(&id).await.map_err(|e| e.to_string())?;
    Ok(Json(SessionResponse {
        id: session.id,
        namespace: session.namespace,
        name: session.name,
        agent_id: session.agent_id,
        project: session.project,
        directory: session.directory,
        started_at: session.started_at,
        ended_at: session.ended_at,
    }))
}

async fn end_session(
    State((brain, _)): State<(Arc<Brain>, Arc<tokio::sync::RwLock<UserConfig>>)>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Result<Json<serde_json::Value>, String> {
    brain.end_session(&id).await.map_err(|e| e.to_string())?;
    Ok(Json(serde_json::json!({
        "success": true,
        "message": "Session ended",
        "session_id": id
    })))
}

async fn delete_session(
    State((brain, _)): State<(Arc<Brain>, Arc<tokio::sync::RwLock<UserConfig>>)>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Result<Json<serde_json::Value>, String> {
    brain.delete_session(&id).await.map_err(|e| e.to_string())?;
    Ok(Json(serde_json::json!({
        "success": true,
        "message": "Session deleted",
        "session_id": id
    })))
}
