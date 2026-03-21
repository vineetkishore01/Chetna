use axum::{
    extract::State,
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use crate::{Brain, config_file::UserConfig};

pub fn router(_brain: Arc<Brain>) -> Router<(Arc<Brain>, Arc<tokio::sync::RwLock<UserConfig>>)> {
    Router::new()
        .route("/", get(list_procedures).post(create_procedure))
        .route("/:id", get(get_procedure).delete(delete_procedure))
        .route("/:id/execute", post(execute_procedure))
        
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateProcedureRequest {
    pub name: String,
    pub description: String,
    pub steps: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProcedureResponse {
    pub id: String,
    pub name: String,
    pub description: String,
    pub steps: Vec<String>,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExecuteProcedureRequest {
    pub parameters: serde_json::Value,
}

async fn list_procedures(State((brain, _)): State<(Arc<Brain>, Arc<tokio::sync::RwLock<UserConfig>>)>) -> Result<Json<Vec<ProcedureResponse>>, StatusCode> {
    let procedures = brain.list_procedures().await.map_err(|e| {
        tracing::error!("Failed to list procedures: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    Ok(Json(procedures.into_iter().map(|p| ProcedureResponse {
        id: p.id,
        name: p.name,
        description: p.description,
        steps: p.steps,
        created_at: p.created_at,
    }).collect()))
}

async fn get_procedure(
    State((brain, _)): State<(Arc<Brain>, Arc<tokio::sync::RwLock<UserConfig>>)>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Result<Json<ProcedureResponse>, String> {
    let procedure = brain.get_procedure(&id).await.map_err(|e| e.to_string())?;
    Ok(Json(ProcedureResponse {
        id: procedure.id,
        name: procedure.name,
        description: procedure.description,
        steps: procedure.steps,
        created_at: procedure.created_at,
    }))
}

async fn create_procedure(
    State((brain, _)): State<(Arc<Brain>, Arc<tokio::sync::RwLock<UserConfig>>)>,
    Json(req): Json<CreateProcedureRequest>,
) -> Result<Json<ProcedureResponse>, String> {
    let name = req.name.trim();
    if name.is_empty() {
        return Err("Procedure name cannot be empty".to_string());
    }
    if name.len() > 255 {
        return Err("Procedure name exceeds maximum length of 255 characters".to_string());
    }
    if req.steps.is_empty() {
        return Err("Procedure must have at least one step".to_string());
    }
    
    let id = brain.create_procedure(name, &req.description, &req.steps)
        .await
        .map_err(|e| e.to_string())?;
    let procedure = brain.get_procedure(&id).await.map_err(|e| e.to_string())?;
    Ok(Json(ProcedureResponse {
        id: procedure.id,
        name: procedure.name,
        description: procedure.description,
        steps: procedure.steps,
        created_at: procedure.created_at,
    }))
}

async fn delete_procedure(
    State((brain, _)): State<(Arc<Brain>, Arc<tokio::sync::RwLock<UserConfig>>)>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Result<Json<()>, String> {
    brain.delete_procedure(&id).await.map_err(|e| e.to_string())?;
    Ok(Json(()))
}

async fn execute_procedure(
    State((brain, _)): State<(Arc<Brain>, Arc<tokio::sync::RwLock<UserConfig>>)>,
    axum::extract::Path(id): axum::extract::Path<String>,
    Json(req): Json<ExecuteProcedureRequest>,
) -> Result<Json<serde_json::Value>, String> {
    let result = brain.execute_procedure(&id, req.parameters).await.map_err(|e| e.to_string())?;
    Ok(Json(result))
}
