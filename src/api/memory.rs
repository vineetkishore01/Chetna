use axum::{
    extract::{Query, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use crate::{Brain, cache::SessionCache, config_file::UserConfig};

const MAX_CONTENT_LENGTH: usize = 50_000;
const MAX_TAGS: usize = 50;
const MAX_TAG_LENGTH: usize = 100;

pub fn router(_brain: Arc<Brain>) -> Router<(Arc<Brain>, Arc<SessionCache>, Arc<tokio::sync::RwLock<UserConfig>>)> {
    Router::new()
        .route("/", get(list_memories).post(create_memory))
        .route("/batch", post(create_memories_batch))
        .route("/:id", get(get_memory).delete(delete_memory).patch(update_memory))
        .route("/search", get(search_memories).post(search_memories_post))
        .route("/search/semantic", get(semantic_search))
        .route("/related/:id", get(get_related_memories))
        .route("/prune", post(prune_memories))
        .route("/context", post(build_context))
        .route("/embed-batch", post(embed_existing_memories))
        .route("/pin/:id", post(pin_memory).delete(unpin_memory))
        .route("/category/:id", post(set_memory_category))
        .route("/restore/:id", post(restore_memory))
        .route("/deleted", get(list_deleted_memories))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateMemoryRequest {
    pub content: String,
    pub importance: Option<f32>,
    pub valence: Option<f32>,
    pub arousal: Option<f32>,
    pub tags: Option<Vec<String>>,
    pub memory_type: Option<String>,
    pub category: Option<String>,
    pub auto_score: Option<bool>,
    pub session_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BatchMemoryRequest {
    pub memories: Vec<CreateMemoryRequest>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MemoryResponse {
    pub id: String,
    pub content: String,
    pub importance: f64,
    pub emotional_tone: f64,
    pub arousal: f64,
    pub tags: Vec<String>,
    pub memory_type: String,
    pub category: String,
    pub embedding_model: Option<String>,
    pub access_count: i64,
    pub created_at: String,
    pub updated_at: String,
    pub consolidated: bool,
    pub is_pinned: bool,
    pub memory_category: String,
    pub last_ranked: Option<String>,
    pub rank_source: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchRequest {
    pub query: String,
    pub limit: Option<i64>,
    pub min_similarity: Option<f32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ContextRequest {
    pub query: String,
    pub max_tokens: Option<i64>,
    pub include_importance: Option<f32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ContextResponse {
    pub memories: Vec<MemoryResponse>,
    pub total_tokens: i64,
    pub context: String,
}

async fn list_memories(State((brain, _, _)): State<(Arc<Brain>, Arc<SessionCache>, Arc<tokio::sync::RwLock<UserConfig>>)>, Query(params): Query<ListParams>) -> Result<Json<Vec<MemoryResponse>>, StatusCode> {
    let limit = params.limit.unwrap_or(100);
    let category = params.category;

    let memories = if let Some(cat) = category {
        brain.list_memories_by_category(&cat, limit).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    } else {
        brain.list_memories(limit).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    };

    Ok(Json(memories.into_iter().map(|m| MemoryResponse {
        id: m.id,
        content: m.content,
        importance: m.importance,
        emotional_tone: m.emotional_tone,
        arousal: m.arousal,
        tags: m.tags,
        memory_type: m.memory_type,
        category: m.category,
        embedding_model: m.embedding_model,
        access_count: m.access_count,
        created_at: m.created_at,
        updated_at: m.updated_at,
        consolidated: m.consolidated,
        is_pinned: m.is_pinned,
        memory_category: m.memory_category,
        last_ranked: m.last_ranked,
        rank_source: m.rank_source,
    }).collect()))
}

#[derive(Deserialize)]
struct ListParams {
    limit: Option<i64>,
    category: Option<String>,
}

async fn get_memory(State((brain, _, _)): State<(Arc<Brain>, Arc<SessionCache>, Arc<tokio::sync::RwLock<UserConfig>>)>, axum::extract::Path(id): axum::extract::Path<String>) -> Result<Json<MemoryResponse>, String> {
    let memory = brain.get_memory(&id).await.map_err(|e| e.to_string())?;
    Ok(Json(MemoryResponse {
        id: memory.id,
        content: memory.content,
        importance: memory.importance,
        emotional_tone: memory.emotional_tone,
        arousal: memory.arousal,
        tags: memory.tags,
        memory_type: memory.memory_type,
        category: memory.category,
        embedding_model: memory.embedding_model,
        access_count: memory.access_count,
        created_at: memory.created_at,
        updated_at: memory.updated_at,
        consolidated: memory.consolidated,
        is_pinned: memory.is_pinned,
        memory_category: memory.memory_category,
        last_ranked: memory.last_ranked,
        rank_source: memory.rank_source,
    }))
}

async fn create_memory(
    State((brain, _, _)): State<(Arc<Brain>, Arc<SessionCache>, Arc<tokio::sync::RwLock<UserConfig>>)>,
    Json(req): Json<CreateMemoryRequest>,
) -> Result<Json<MemoryResponse>, String> {
    // Input validation
    if req.content.trim().is_empty() {
        return Err("Memory content cannot be empty".to_string());
    }
    if req.content.len() > MAX_CONTENT_LENGTH {
        return Err(format!("Memory content exceeds maximum length of {} characters", MAX_CONTENT_LENGTH));
    }
    if let Some(ref tags) = req.tags {
        if tags.len() > MAX_TAGS {
            return Err(format!("Too many tags (max {})", MAX_TAGS));
        }
        for tag in tags {
            if tag.len() > MAX_TAG_LENGTH {
                return Err(format!("Tag '{}' exceeds maximum length of {} characters", tag, MAX_TAG_LENGTH));
            }
        }
    }

    let auto_score = req.auto_score.unwrap_or(false);

    let (importance, valence, arousal) = if auto_score && brain.has_embedder() {
        let scores = brain.auto_score_importance(&req.content).await.unwrap_or((0.5, 0.0, 0.0));
        (scores.0, scores.1, scores.2)
    } else {
        (req.importance.unwrap_or(0.5), req.valence.unwrap_or(0.0), req.arousal.unwrap_or(0.0))
    };

    let id = brain.create_memory(
        &req.content,
        importance,
        valence,
        arousal,
        req.tags.as_deref().unwrap_or(&[]),
        req.memory_type.as_deref().unwrap_or("fact"),
        req.category.as_deref().unwrap_or("fact"),
        req.session_id.as_deref(),
    ).await.map_err(|e| {
        tracing::error!("Failed to create memory: {}", e);
        e.to_string()
    })?;

    let memory = brain.get_memory(&id).await.map_err(|e| e.to_string())?;
    Ok(Json(MemoryResponse {
        id: memory.id,
        content: memory.content,
        importance: memory.importance,
        emotional_tone: memory.emotional_tone,
        arousal: memory.arousal,
        tags: memory.tags,
        memory_type: memory.memory_type,
        category: memory.category,
        embedding_model: memory.embedding_model,
        access_count: memory.access_count,
        created_at: memory.created_at,
        updated_at: memory.updated_at,
        consolidated: memory.consolidated,
        is_pinned: memory.is_pinned,
        memory_category: memory.memory_category,
        last_ranked: memory.last_ranked,
        rank_source: memory.rank_source,
    }))
}

async fn create_memories_batch(
    State((brain, _, _)): State<(Arc<Brain>, Arc<SessionCache>, Arc<tokio::sync::RwLock<UserConfig>>)>,
    Json(req): Json<BatchMemoryRequest>,
) -> Result<Json<Vec<MemoryResponse>>, StatusCode> {
    let mut results = Vec::new();

    for mem_req in req.memories {
        // Input validation
        if mem_req.content.trim().is_empty() {
            continue;
        }
        if mem_req.content.len() > MAX_CONTENT_LENGTH {
            continue;
        }
        if let Some(ref tags) = mem_req.tags {
            if tags.len() > MAX_TAGS {
                continue;
            }
        }

        let importance = mem_req.importance.unwrap_or(0.5);
        let valence = mem_req.valence.unwrap_or(0.0);
        let arousal = mem_req.arousal.unwrap_or(0.0);

        let id = brain.create_memory(
            &mem_req.content,
            importance,
            valence,
            arousal,
            mem_req.tags.as_deref().unwrap_or(&[]),
            mem_req.memory_type.as_deref().unwrap_or("fact"),
            mem_req.category.as_deref().unwrap_or("fact"),
            mem_req.session_id.as_deref(),
        ).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        if let Ok(memory) = brain.get_memory(&id).await {
            results.push(MemoryResponse {
                id: memory.id,
                content: memory.content,
                importance: memory.importance,
                emotional_tone: memory.emotional_tone,
                arousal: memory.arousal,
                tags: memory.tags,
                memory_type: memory.memory_type,
                category: memory.category,
                embedding_model: memory.embedding_model,
                access_count: memory.access_count,
                created_at: memory.created_at,
                updated_at: memory.updated_at,
                consolidated: memory.consolidated,
                is_pinned: memory.is_pinned,
                memory_category: memory.memory_category,
                last_ranked: memory.last_ranked,
                rank_source: memory.rank_source,
            });
        }
    }

    Ok(Json(results))
}

async fn delete_memory(
    State((brain, _, _)): State<(Arc<Brain>, Arc<SessionCache>, Arc<tokio::sync::RwLock<UserConfig>>)>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Result<Json<serde_json::Value>, String> {
    brain.soft_delete_memory(&id).await.map_err(|e| e.to_string())?;
    Ok(Json(serde_json::json!({
        "success": true,
        "message": "Memory deleted",
        "memory_id": id
    })))
}

async fn search_memories(
    State((brain, _, _)): State<(Arc<Brain>, Arc<SessionCache>, Arc<tokio::sync::RwLock<UserConfig>>)>,
    Query(params): Query<SearchRequest>,
) -> Result<Json<Vec<MemoryResponse>>, StatusCode> {
    let limit = params.limit.unwrap_or(20);
    let memories = brain.search_memories(&params.query, limit).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(memories.into_iter().map(|m| MemoryResponse {
        id: m.id,
        content: m.content,
        importance: m.importance,
        emotional_tone: m.emotional_tone,
        arousal: m.arousal,
        tags: m.tags,
        memory_type: m.memory_type,
        category: m.category,
        embedding_model: m.embedding_model,
        access_count: m.access_count,
        created_at: m.created_at,
        updated_at: m.updated_at,
        consolidated: m.consolidated,
        is_pinned: m.is_pinned,
        memory_category: m.memory_category,
        last_ranked: m.last_ranked,
        rank_source: m.rank_source,
    }).collect()))
}

async fn search_memories_post(
    State((brain, _, _)): State<(Arc<Brain>, Arc<SessionCache>, Arc<tokio::sync::RwLock<UserConfig>>)>,
    Json(req): Json<SearchRequest>,
) -> Result<Json<Vec<MemoryResponse>>, StatusCode> {
    let limit = req.limit.unwrap_or(20);
    let memories = brain.search_memories(&req.query, limit).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(memories.into_iter().map(|m| MemoryResponse {
        id: m.id,
        content: m.content,
        importance: m.importance,
        emotional_tone: m.emotional_tone,
        arousal: m.arousal,
        tags: m.tags,
        memory_type: m.memory_type,
        category: m.category,
        embedding_model: m.embedding_model,
        access_count: m.access_count,
        created_at: m.created_at,
        updated_at: m.updated_at,
        consolidated: m.consolidated,
        is_pinned: m.is_pinned,
        memory_category: m.memory_category,
        last_ranked: m.last_ranked,
        rank_source: m.rank_source,
    }).collect()))
}

async fn semantic_search(
    State((brain, _, _)): State<(Arc<Brain>, Arc<SessionCache>, Arc<tokio::sync::RwLock<UserConfig>>)>,
    Query(params): Query<SearchRequest>,
) -> Result<Json<Vec<MemoryResponse>>, StatusCode> {
    let limit = params.limit.unwrap_or(20);
    let min_sim = params.min_similarity.unwrap_or(0.7);

    let memories = brain.semantic_search(&params.query, limit, min_sim).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(memories.into_iter().map(|m| MemoryResponse {
        id: m.id,
        content: m.content,
        importance: m.importance,
        emotional_tone: m.emotional_tone,
        arousal: m.arousal,
        tags: m.tags,
        memory_type: m.memory_type,
        category: m.category,
        embedding_model: m.embedding_model,
        access_count: m.access_count,
        created_at: m.created_at,
        updated_at: m.updated_at,
        consolidated: m.consolidated,
        is_pinned: m.is_pinned,
        memory_category: m.memory_category,
        last_ranked: m.last_ranked,
        rank_source: m.rank_source,
    }).collect()))
}

async fn get_related_memories(
    State((brain, _, _)): State<(Arc<Brain>, Arc<SessionCache>, Arc<tokio::sync::RwLock<UserConfig>>)>,
    axum::extract::Path(id): axum::extract::Path<String>,
    Query(params): Query<ListParams>,
) -> Result<Json<Vec<MemoryResponse>>, StatusCode> {
    let limit = params.limit.unwrap_or(10);

    let memories = brain.find_related_memories(&id, limit).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(memories.into_iter().map(|m| MemoryResponse {
        id: m.id,
        content: m.content,
        importance: m.importance,
        emotional_tone: m.emotional_tone,
        arousal: m.arousal,
        tags: m.tags,
        memory_type: m.memory_type,
        category: m.category,
        embedding_model: m.embedding_model,
        access_count: m.access_count,
        created_at: m.created_at,
        updated_at: m.updated_at,
        consolidated: m.consolidated,
        is_pinned: m.is_pinned,
        memory_category: m.memory_category,
        last_ranked: m.last_ranked,
        rank_source: m.rank_source,
    }).collect()))
}

async fn prune_memories(State((brain, _, _)): State<(Arc<Brain>, Arc<SessionCache>, Arc<tokio::sync::RwLock<UserConfig>>)>) -> Result<Json<serde_json::Value>, StatusCode> {
    let count = brain.prune_memories(30, 0.1).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(serde_json::json!({
        "pruned": count,
        "status": "ok"
    })))
}

async fn build_context(
    State((brain, _, _)): State<(Arc<Brain>, Arc<SessionCache>, Arc<tokio::sync::RwLock<UserConfig>>)>,
    Json(req): Json<ContextRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let max_tokens = req.max_tokens.unwrap_or(4000);
    let include_importance = req.include_importance.unwrap_or(0.3);

    let result = brain.build_context(&req.query, max_tokens, include_importance).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(result))
}

async fn embed_existing_memories(State((brain, _, _)): State<(Arc<Brain>, Arc<SessionCache>, Arc<tokio::sync::RwLock<UserConfig>>)>) -> Result<Json<serde_json::Value>, StatusCode> {
    let count = brain.embed_existing_memories().await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(serde_json::json!({
        "embedded": count,
        "status": "ok"
    })))
}

async fn pin_memory(
    State((brain, _, _)): State<(Arc<Brain>, Arc<SessionCache>, Arc<tokio::sync::RwLock<UserConfig>>)>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Json<serde_json::Value> {
    match brain.set_memory_pinned(&id, true).await {
        Ok(_) => Json(serde_json::json!({"success": true, "message": "Memory pinned"})),
        Err(e) => Json(serde_json::json!({"success": false, "message": e.to_string()})),
    }
}

async fn unpin_memory(
    State((brain, _, _)): State<(Arc<Brain>, Arc<SessionCache>, Arc<tokio::sync::RwLock<UserConfig>>)>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Json<serde_json::Value> {
    match brain.set_memory_pinned(&id, false).await {
        Ok(_) => Json(serde_json::json!({"success": true, "message": "Memory unpinned"})),
        Err(e) => Json(serde_json::json!({"success": false, "message": e.to_string()})),
    }
}

#[derive(Deserialize)]
pub struct SetCategoryRequest {
    pub category: String,
}

async fn set_memory_category(
    State((brain, _, _)): State<(Arc<Brain>, Arc<SessionCache>, Arc<tokio::sync::RwLock<UserConfig>>)>,
    axum::extract::Path(id): axum::extract::Path<String>,
    Json(req): Json<SetCategoryRequest>,
) -> Json<serde_json::Value> {
    match brain.set_memory_category(&id, &req.category).await {
        Ok(_) => Json(serde_json::json!({"success": true, "message": "Category updated"})),
        Err(e) => Json(serde_json::json!({"success": false, "message": e.to_string()})),
    }
}

#[derive(Deserialize)]
pub struct UpdateMemoryRequest {
    pub importance: Option<f64>,
    pub memory_category: Option<String>,
}

async fn update_memory(
    State((brain, _, _)): State<(Arc<Brain>, Arc<SessionCache>, Arc<tokio::sync::RwLock<UserConfig>>)>,
    axum::extract::Path(id): axum::extract::Path<String>,
    Json(req): Json<UpdateMemoryRequest>,
) -> Json<serde_json::Value> {
    if let Some(importance) = req.importance {
        if let Err(e) = brain.update_memory_importance(&id, importance, "manual").await {
            return Json(serde_json::json!({"success": false, "message": e.to_string()}));
        }
    }
    if let Some(category) = req.memory_category {
        if let Err(e) = brain.set_memory_category(&id, &category).await {
            return Json(serde_json::json!({"success": false, "message": e.to_string()}));
        }
    }
    Json(serde_json::json!({"success": true, "message": "Memory updated"}))
}

async fn restore_memory(
    State((brain, _, _)): State<(Arc<Brain>, Arc<SessionCache>, Arc<tokio::sync::RwLock<UserConfig>>)>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Result<Json<serde_json::Value>, String> {
    brain.restore_memory(&id).await.map_err(|e| e.to_string())?;
    Ok(Json(serde_json::json!({"success": true, "message": "Memory restored"})))
}

async fn list_deleted_memories(
    State((brain, _, _)): State<(Arc<Brain>, Arc<SessionCache>, Arc<tokio::sync::RwLock<UserConfig>>)>,
    Query(params): Query<ListParams>,
) -> Result<Json<Vec<MemoryResponse>>, StatusCode> {
    let limit = params.limit.unwrap_or(100);
    let memories = brain.list_deleted_memories(limit).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(memories.into_iter().map(|m| MemoryResponse {
        id: m.id,
        content: m.content,
        importance: m.importance,
        emotional_tone: m.emotional_tone,
        arousal: m.arousal,
        tags: m.tags,
        memory_type: m.memory_type,
        category: m.category,
        embedding_model: m.embedding_model,
        access_count: m.access_count,
        created_at: m.created_at,
        updated_at: m.updated_at,
        consolidated: m.consolidated,
        is_pinned: m.is_pinned,
        memory_category: m.memory_category,
        last_ranked: m.last_ranked,
        rank_source: m.rank_source,
    }).collect()))
}
