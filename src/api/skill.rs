use axum::{
    extract::State,
    routing::get,
    Json, Router,
};
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use crate::{Brain, cache::SessionCache, config_file::UserConfig};

pub fn router(_brain: Arc<Brain>) -> Router<(Arc<Brain>, Arc<SessionCache>, Arc<tokio::sync::RwLock<UserConfig>>)> {
    Router::new()
        .route("/", get(list_skills).post(create_skill))
        .route("/:id", get(get_skill).delete(delete_skill))
        
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateSkillRequest {
    pub name: String,
    pub description: String,
    pub code: String,
    pub language: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SkillResponse {
    pub id: String,
    pub name: String,
    pub description: String,
    pub code: String,
    pub language: String,
    pub created_at: String,
}

async fn list_skills(State((brain, _, _)): State<(Arc<Brain>, Arc<SessionCache>, Arc<tokio::sync::RwLock<UserConfig>>)>) -> Json<Vec<SkillResponse>> {
    let skills = brain.list_skills().await.unwrap_or_default();
    Json(skills.into_iter().map(|s| SkillResponse {
        id: s.id,
        name: s.name,
        description: s.description,
        code: s.code,
        language: s.language,
        created_at: s.created_at,
    }).collect())
}

async fn get_skill(
    State((brain, _, _)): State<(Arc<Brain>, Arc<SessionCache>, Arc<tokio::sync::RwLock<UserConfig>>)>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Result<Json<SkillResponse>, String> {
    let skill = brain.get_skill(&id).await.map_err(|e| e.to_string())?;
    Ok(Json(SkillResponse {
        id: skill.id,
        name: skill.name,
        description: skill.description,
        code: skill.code,
        language: skill.language,
        created_at: skill.created_at,
    }))
}

async fn create_skill(
    State((brain, _, _)): State<(Arc<Brain>, Arc<SessionCache>, Arc<tokio::sync::RwLock<UserConfig>>)>,
    Json(req): Json<CreateSkillRequest>,
) -> Result<Json<SkillResponse>, String> {
    let id = brain.create_skill(&req.name, &req.description, &req.code, &req.language)
        .await
        .map_err(|e| e.to_string())?;
    let skill = brain.get_skill(&id).await.map_err(|e| e.to_string())?;
    Ok(Json(SkillResponse {
        id: skill.id,
        name: skill.name,
        description: skill.description,
        code: skill.code,
        language: skill.language,
        created_at: skill.created_at,
    }))
}

async fn delete_skill(
    State((brain, _, _)): State<(Arc<Brain>, Arc<SessionCache>, Arc<tokio::sync::RwLock<UserConfig>>)>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Result<Json<()>, String> {
    brain.delete_skill(&id).await.map_err(|e| e.to_string())?;
    Ok(Json(()))
}
