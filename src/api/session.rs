use axum::{
    extract::State,
    routing::{get, post},
    Json, Router,
};
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use crate::{Brain, cache::SessionCache, config_file::UserConfig};

pub fn router(_brain: Arc<Brain>) -> Router<(Arc<Brain>, Arc<SessionCache>, Arc<tokio::sync::RwLock<UserConfig>>)> {
    Router::new()
        .route("/", get(list_sessions).post(create_session))
        .route("/:id", get(get_session).delete(delete_session))
        .route("/:id/end", post(end_session))

}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateSessionRequest {
    pub name: String,
    pub agent_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SessionResponse {
    pub id: String,
    pub name: String,
    pub agent_id: Option<String>,
    pub started_at: String,
    pub ended_at: Option<String>,
}

async fn list_sessions(State((brain, _, _)): State<(Arc<Brain>, Arc<SessionCache>, Arc<tokio::sync::RwLock<UserConfig>>)>) -> Json<Vec<SessionResponse>> {
    let sessions = brain.list_sessions(50).await.unwrap_or_default();
    Json(sessions.into_iter().map(|s| SessionResponse {
        id: s.id,
        name: s.name,
        agent_id: s.agent_id,
        started_at: s.started_at,
        ended_at: s.ended_at,
    }).collect())
}

async fn get_session(
    State((brain, _, _)): State<(Arc<Brain>, Arc<SessionCache>, Arc<tokio::sync::RwLock<UserConfig>>)>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Result<Json<SessionResponse>, String> {
    let session = brain.get_session(&id).await.map_err(|e| e.to_string())?;
    Ok(Json(SessionResponse {
        id: session.id,
        name: session.name,
        agent_id: session.agent_id,
        started_at: session.started_at,
        ended_at: session.ended_at,
    }))
}

async fn create_session(
    State((brain, _, _)): State<(Arc<Brain>, Arc<SessionCache>, Arc<tokio::sync::RwLock<UserConfig>>)>,
    Json(req): Json<CreateSessionRequest>,
) -> Result<Json<SessionResponse>, String> {
    let id = brain.create_session(&req.name, req.agent_id.as_deref())
        .await
        .map_err(|e| e.to_string())?;
    let session = brain.get_session(&id).await.map_err(|e| e.to_string())?;
    Ok(Json(SessionResponse {
        id: session.id,
        name: session.name,
        agent_id: session.agent_id,
        started_at: session.started_at,
        ended_at: session.ended_at,
    }))
}

async fn end_session(
    State((brain, _, _)): State<(Arc<Brain>, Arc<SessionCache>, Arc<tokio::sync::RwLock<UserConfig>>)>,
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
    State((brain, _, _)): State<(Arc<Brain>, Arc<SessionCache>, Arc<tokio::sync::RwLock<UserConfig>>)>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Result<Json<serde_json::Value>, String> {
    brain.delete_session(&id).await.map_err(|e| e.to_string())?;
    Ok(Json(serde_json::json!({
        "success": true,
        "message": "Session deleted",
        "session_id": id
    })))
}
