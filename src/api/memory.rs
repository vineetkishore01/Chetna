use axum::{
    extract::{Query, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use crate::{Brain, config_file::UserConfig, db::brain::{RecallWeights, CATEGORIES}};
use tracing::error;

const MAX_CONTENT_LENGTH: usize = 50_000;
const MAX_TAGS: usize = 50;
const MAX_TAG_LENGTH: usize = 100;

fn map_err<T>(err: T) -> StatusCode 
where T: std::fmt::Debug {
    error!("Database error: {:?}", err);
    StatusCode::INTERNAL_SERVER_ERROR
}

pub fn router(_brain: Arc<Brain>) -> Router<(Arc<Brain>, Arc<tokio::sync::RwLock<UserConfig>>)> {
    Router::new()
        .route("/", get(list_memories).post(create_memory))
        .route("/batch", post(create_memories_batch))
        .route("/:id", get(get_memory).delete(delete_memory).patch(update_memory))
        .route("/search", get(search_memories).post(search_memories_post))
        .route("/search/semantic", get(semantic_search))
        .route("/search/explain", get(search_explain).post(search_explain_post))
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
    pub namespace: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BatchMemoryRequest {
    pub memories: Vec<CreateMemoryRequest>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BatchMemoryResponse {
    pub created: Vec<MemoryResponse>,
    pub failed: Vec<BatchMemoryError>,
    pub total: usize,
    pub success_count: usize,
    pub failure_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchMemoryError {
    pub index: usize,
    pub error: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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
    pub namespace: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ContextRequest {
    pub query: String,
    pub max_tokens: Option<i64>,
    pub include_importance: Option<f32>,
    pub namespace: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ContextResponse {
    pub memories: Vec<MemoryResponse>,
    pub total_tokens: i64,
    pub context: String,
}

async fn list_memories(State((brain, _)): State<(Arc<Brain>, Arc<tokio::sync::RwLock<UserConfig>>)>, Query(params): Query<ListParams>) -> Result<Json<Vec<MemoryResponse>>, StatusCode> {
    let limit = params.limit.unwrap_or(100);
    let category = params.category;
    let namespace = params.namespace.as_deref();

    let memories = if let Some(cat) = category {
        brain.list_memories_by_category(&cat, limit, namespace).await.map_err(map_err)?
    } else {
        brain.list_memories(limit, namespace).await.map_err(map_err)?
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
    namespace: Option<String>,
}

async fn get_memory(State((brain, _)): State<(Arc<Brain>, Arc<tokio::sync::RwLock<UserConfig>>)>, axum::extract::Path(id): axum::extract::Path<String>) -> Result<Json<MemoryResponse>, String> {
    let memory = brain.get_memory(&id).await.map_err(|e| e.to_string())?;
    
    // Explicit access by ID is a strong indicator of relevance - increment count
    let _ = brain.increment_access_count(&id).await;

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
        is_pinned: memory.is_pinned,
        memory_category: memory.memory_category,
        last_ranked: memory.last_ranked,
        rank_source: memory.rank_source,
    }))
}

async fn create_memory(
    State((brain, _)): State<(Arc<Brain>, Arc<tokio::sync::RwLock<UserConfig>>)>,
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

    if let Some(ref session_id) = req.session_id {
        if !session_id.is_empty() && brain.get_session(session_id).await.is_err() {
            return Err(format!("Session not found: {}", session_id));
        }
    }

    if let Some(ref category) = req.category {
        if !category.is_empty() && !CATEGORIES.contains(&category.as_str()) {
            return Err(format!("Invalid category '{}'. Valid options: {:?}", category, CATEGORIES));
        }
    }

    let auto_score = req.auto_score.unwrap_or(false);

    let (importance, valence, arousal) = if auto_score && brain.has_embedder().await {
        let scores = brain.auto_score_importance(&req.content).await.unwrap_or((0.5, 0.0, 0.0));
        (scores.0.clamp(0.0, 1.0), scores.1.clamp(-1.0, 1.0), scores.2.clamp(0.0, 1.0))
    } else {
        (
            req.importance.unwrap_or(0.5).clamp(0.0, 1.0),
            req.valence.unwrap_or(0.0).clamp(-1.0, 1.0),
            req.arousal.unwrap_or(0.0).clamp(0.0, 1.0)
        )
    };

    let memory = brain.create_memory(
        &req.content,
        importance,
        valence,
        arousal,
        req.tags.as_deref().unwrap_or(&[]),
        req.memory_type.as_deref().unwrap_or("fact"),
        req.category.as_deref().unwrap_or("fact"),
        req.session_id.as_deref(),
        req.namespace.as_deref(),
    ).await.map_err(|e| {
        tracing::error!("Failed to create memory: {}", e);
        e.to_string()
    })?;

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
        is_pinned: memory.is_pinned,
        memory_category: memory.memory_category,
        last_ranked: memory.last_ranked,
        rank_source: memory.rank_source,
    }))
}

async fn create_memories_batch(
    State((brain, _)): State<(Arc<Brain>, Arc<tokio::sync::RwLock<UserConfig>>)>,
    Json(req): Json<BatchMemoryRequest>,
) -> Result<Json<BatchMemoryResponse>, StatusCode> {
    let mut results = Vec::new();
    let mut failures = Vec::new();

    for (index, mem_req) in req.memories.iter().enumerate() {
        // Input validation
        if mem_req.content.trim().is_empty() {
            failures.push(BatchMemoryError {
                index,
                error: "Memory content cannot be empty".to_string(),
            });
            continue;
        }
        if mem_req.content.len() > MAX_CONTENT_LENGTH {
            failures.push(BatchMemoryError {
                index,
                error: format!("Memory content exceeds maximum length of {} characters", MAX_CONTENT_LENGTH),
            });
            continue;
        }
        if let Some(ref tags) = mem_req.tags {
            if tags.len() > MAX_TAGS {
                failures.push(BatchMemoryError {
                    index,
                    error: format!("Too many tags (max {})", MAX_TAGS),
                });
                continue;
            }
            let mut has_tag_error = false;
            for tag in tags {
                if tag.len() > MAX_TAG_LENGTH {
                    failures.push(BatchMemoryError {
                        index,
                        error: format!("Tag '{}' exceeds maximum length of {} characters", tag, MAX_TAG_LENGTH),
                    });
                    has_tag_error = true;
                }
            }
            if has_tag_error {
                continue;
            }
        }

        // Clamp importance, valence, arousal to valid ranges
        let importance = mem_req.importance.unwrap_or(0.5).clamp(0.0, 1.0);
        let valence = mem_req.valence.unwrap_or(0.0).clamp(-1.0, 1.0);
        let arousal = mem_req.arousal.unwrap_or(0.0).clamp(0.0, 1.0);

        match brain.create_memory(
            &mem_req.content,
            importance,
            valence,
            arousal,
            mem_req.tags.as_deref().unwrap_or(&[]),
            mem_req.memory_type.as_deref().unwrap_or("fact"),
            mem_req.category.as_deref().unwrap_or("fact"),
            mem_req.session_id.as_deref(),
            mem_req.namespace.as_deref(),
        ).await {
            Ok(memory) => {
                if true {
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
                                        is_pinned: memory.is_pinned,
                        memory_category: memory.memory_category,
                        last_ranked: memory.last_ranked,
                        rank_source: memory.rank_source,
                    });
                }
            }
            Err(e) => {
                failures.push(BatchMemoryError {
                    index,
                    error: e.to_string(),
                });
            }
        }
    }

    let success_count = results.len();
    let failure_count = failures.len();

    Ok(Json(BatchMemoryResponse {
        created: results,
        failed: failures,
        total: req.memories.len(),
        success_count,
        failure_count,
    }))
}

async fn delete_memory(
    State((brain, _)): State<(Arc<Brain>, Arc<tokio::sync::RwLock<UserConfig>>)>,
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
    State((brain, _)): State<(Arc<Brain>, Arc<tokio::sync::RwLock<UserConfig>>)>,
    Query(params): Query<SearchRequest>,
) -> Result<Json<Vec<MemoryResponse>>, StatusCode> {
    let limit = params.limit.unwrap_or(20);
    let memories = brain.search_memories(&params.query, limit, params.namespace.as_deref()).await.map_err(map_err)?;

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
        is_pinned: m.is_pinned,
        memory_category: m.memory_category,
        last_ranked: m.last_ranked,
        rank_source: m.rank_source,
    }).collect()))
}

async fn search_memories_post(
    State((brain, _)): State<(Arc<Brain>, Arc<tokio::sync::RwLock<UserConfig>>)>,
    Json(req): Json<SearchRequest>,
) -> Result<Json<Vec<MemoryResponse>>, StatusCode> {
    let limit = req.limit.unwrap_or(20);
    let memories = brain.search_memories(&req.query, limit, req.namespace.as_deref()).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

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
        is_pinned: m.is_pinned,
        memory_category: m.memory_category,
        last_ranked: m.last_ranked,
        rank_source: m.rank_source,
    }).collect()))
}

async fn semantic_search(
    State((brain, _)): State<(Arc<Brain>, Arc<tokio::sync::RwLock<UserConfig>>)>,
    Query(params): Query<SearchRequest>,
) -> Result<Json<Vec<MemoryResponse>>, StatusCode> {
    let limit = params.limit.unwrap_or(20);
    let min_sim = params.min_similarity.unwrap_or(0.1);

    let memories = brain.semantic_search(&params.query, limit, min_sim, params.namespace.as_deref()).await.map_err(map_err)?;

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
        is_pinned: m.is_pinned,
        memory_category: m.memory_category,
        last_ranked: m.last_ranked,
        rank_source: m.rank_source,
    }).collect()))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchExplainRequest {
    pub query: String,
    pub limit: Option<i64>,
    pub min_similarity: Option<f32>,
    pub weights: Option<RecallWeights>,
    pub namespace: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchExplainResponse {
    pub memories: Vec<MemoryResponse>,
    pub explanation: Vec<RecallExplanation>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RecallExplanation {
    pub memory_id: String,
    pub total_score: f32,
    pub breakdown: ScoreBreakdownResponse,
    pub factors: FactorsResponse,
    pub weights_used: WeightsResponse,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ScoreBreakdownResponse {
    pub similarity: f32,
    pub importance: f32,
    pub recency: f32,
    pub access_frequency: f32,
    pub emotional: f32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FactorsResponse {
    pub is_pinned: bool,
    pub hours_old: i64,
    pub access_count: i64,
    pub memory_type: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WeightsResponse {
    pub similarity: f32,
    pub importance: f32,
    pub recency: f32,
    pub access_frequency: f32,
    pub emotional: f32,
}

async fn search_explain(
    State((brain, _)): State<(Arc<Brain>, Arc<tokio::sync::RwLock<UserConfig>>)>,
    Query(params): Query<SearchExplainRequest>,
) -> Result<Json<SearchExplainResponse>, StatusCode> {
    let limit = params.limit.unwrap_or(20);
    let min_sim = params.min_similarity.unwrap_or(0.1);  // Lowered from 0.3 - qwen3-embedding produces lower similarity scores

    let results = brain.semantic_search_with_explanation(&params.query, limit, min_sim, params.weights, params.namespace.as_deref()).await.map_err(map_err)?;

    let memories: Vec<MemoryResponse> = results.iter().map(|r| MemoryResponse {
        id: r.memory.id.clone(),
        content: r.memory.content.clone(),
        importance: r.memory.importance,
        emotional_tone: r.memory.emotional_tone,
        arousal: r.memory.arousal,
        tags: r.memory.tags.clone(),
        memory_type: r.memory.memory_type.clone(),
        category: r.memory.category.clone(),
        embedding_model: r.memory.embedding_model.clone(),
        access_count: r.memory.access_count,
        created_at: r.memory.created_at.clone(),
        updated_at: r.memory.updated_at.clone(),
        is_pinned: r.memory.is_pinned,
        memory_category: r.memory.memory_category.clone(),
        last_ranked: r.memory.last_ranked.clone(),
        rank_source: r.memory.rank_source.clone(),
    }).collect();

    let explanation: Vec<RecallExplanation> = results.iter().map(|r| RecallExplanation {
        memory_id: r.memory.id.clone(),
        total_score: r.total_score,
        breakdown: ScoreBreakdownResponse {
            similarity: r.breakdown.similarity,
            importance: r.breakdown.importance,
            recency: r.breakdown.recency,
            access_frequency: r.breakdown.access_frequency,
            emotional: r.breakdown.emotional,
        },
        factors: FactorsResponse {
            is_pinned: r.factors.is_pinned,
            hours_old: r.factors.hours_old,
            access_count: r.factors.access_count,
            memory_type: r.factors.memory_type.clone(),
        },
        weights_used: WeightsResponse {
            similarity: r.weights_used.similarity,
            importance: r.weights_used.importance,
            recency: r.weights_used.recency,
            access_frequency: r.weights_used.access_frequency,
            emotional: r.weights_used.emotional,
        },
    }).collect();

    Ok(Json(SearchExplainResponse { memories, explanation }))
}

async fn search_explain_post(
    State((brain, _)): State<(Arc<Brain>, Arc<tokio::sync::RwLock<UserConfig>>)>,
    Json(req): Json<SearchExplainRequest>,
) -> Result<Json<SearchExplainResponse>, StatusCode> {
    let limit = req.limit.unwrap_or(20);
    let min_sim = req.min_similarity.unwrap_or(0.1);  // Lowered from 0.3 - qwen3-embedding produces lower similarity scores

    let results = brain.semantic_search_with_explanation(&req.query, limit, min_sim, req.weights, req.namespace.as_deref()).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let memories: Vec<MemoryResponse> = results.iter().map(|r| MemoryResponse {
        id: r.memory.id.clone(),
        content: r.memory.content.clone(),
        importance: r.memory.importance,
        emotional_tone: r.memory.emotional_tone,
        arousal: r.memory.arousal,
        tags: r.memory.tags.clone(),
        memory_type: r.memory.memory_type.clone(),
        category: r.memory.category.clone(),
        embedding_model: r.memory.embedding_model.clone(),
        access_count: r.memory.access_count,
        created_at: r.memory.created_at.clone(),
        updated_at: r.memory.updated_at.clone(),
        is_pinned: r.memory.is_pinned,
        memory_category: r.memory.memory_category.clone(),
        last_ranked: r.memory.last_ranked.clone(),
        rank_source: r.memory.rank_source.clone(),
    }).collect();

    let explanation: Vec<RecallExplanation> = results.iter().map(|r| RecallExplanation {
        memory_id: r.memory.id.clone(),
        total_score: r.total_score,
        breakdown: ScoreBreakdownResponse {
            similarity: r.breakdown.similarity,
            importance: r.breakdown.importance,
            recency: r.breakdown.recency,
            access_frequency: r.breakdown.access_frequency,
            emotional: r.breakdown.emotional,
        },
        factors: FactorsResponse {
            is_pinned: r.factors.is_pinned,
            hours_old: r.factors.hours_old,
            access_count: r.factors.access_count,
            memory_type: r.factors.memory_type.clone(),
        },
        weights_used: WeightsResponse {
            similarity: r.weights_used.similarity,
            importance: r.weights_used.importance,
            recency: r.weights_used.recency,
            access_frequency: r.weights_used.access_frequency,
            emotional: r.weights_used.emotional,
        },
    }).collect();

    Ok(Json(SearchExplainResponse { memories, explanation }))
}

async fn get_related_memories(
    State((brain, _)): State<(Arc<Brain>, Arc<tokio::sync::RwLock<UserConfig>>)>,
    axum::extract::Path(id): axum::extract::Path<String>,
    Query(params): Query<ListParams>,
) -> Result<Json<Vec<MemoryResponse>>, StatusCode> {
    let limit = params.limit.unwrap_or(10);
    let namespace = params.namespace.as_deref();

    let memories = brain.find_related_memories(&id, limit, namespace).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

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
        is_pinned: m.is_pinned,
        memory_category: m.memory_category,
        last_ranked: m.last_ranked,
        rank_source: m.rank_source,
    }).collect()))
}

async fn prune_memories(State((brain, _)): State<(Arc<Brain>, Arc<tokio::sync::RwLock<UserConfig>>)>) -> Result<Json<serde_json::Value>, StatusCode> {
    let count = brain.prune_memories(30, 0.1).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(serde_json::json!({
        "pruned": count,
        "status": "ok"
    })))
}

async fn build_context(
    State((brain, _)): State<(Arc<Brain>, Arc<tokio::sync::RwLock<UserConfig>>)>,
    Json(req): Json<ContextRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let max_tokens = req.max_tokens.unwrap_or(4000);
    let include_importance = req.include_importance.unwrap_or(0.3);

    let result = brain.build_context(&req.query, max_tokens, include_importance, req.namespace.as_deref()).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(result))
}

async fn embed_existing_memories(State((brain, _)): State<(Arc<Brain>, Arc<tokio::sync::RwLock<UserConfig>>)>) -> Result<Json<serde_json::Value>, StatusCode> {
    let count = brain.embed_existing_memories().await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(serde_json::json!({
        "embedded": count,
        "status": "ok"
    })))
}

async fn pin_memory(
    State((brain, _)): State<(Arc<Brain>, Arc<tokio::sync::RwLock<UserConfig>>)>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Json<serde_json::Value> {
    match brain.set_memory_pinned(&id, true).await {
        Ok(_) => Json(serde_json::json!({"success": true, "message": "Memory pinned"})),
        Err(e) => Json(serde_json::json!({"success": false, "message": e.to_string()})),
    }
}

async fn unpin_memory(
    State((brain, _)): State<(Arc<Brain>, Arc<tokio::sync::RwLock<UserConfig>>)>,
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
    State((brain, _)): State<(Arc<Brain>, Arc<tokio::sync::RwLock<UserConfig>>)>,
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
    pub content: Option<String>,
    pub importance: Option<f32>,
    pub memory_type: Option<String>,
    pub category: Option<String>,
    pub memory_category: Option<String>,
    pub tags: Option<Vec<String>>,
}

async fn update_memory(
    State((brain, _)): State<(Arc<Brain>, Arc<tokio::sync::RwLock<UserConfig>>)>,
    axum::extract::Path(id): axum::extract::Path<String>,
    Json(req): Json<UpdateMemoryRequest>,
) -> Json<serde_json::Value> {
    if let Some(ref mem_cat) = req.memory_category {
        if let Err(e) = brain.set_memory_category(&id, mem_cat).await {
            return Json(serde_json::json!({"success": false, "message": format!("Failed to update memory category: {}", e)}));
        }
    }

    match brain.update_memory(
        &id,
        req.content.as_deref(),
        req.importance,
        req.memory_type.as_deref(),
        req.category.as_deref(),
        req.tags
    ).await {
        Ok(_) => Json(serde_json::json!({"success": true, "message": "Memory updated"})),
        Err(e) => Json(serde_json::json!({"success": false, "message": e.to_string()})),
    }
}

async fn restore_memory(
    State((brain, _)): State<(Arc<Brain>, Arc<tokio::sync::RwLock<UserConfig>>)>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Result<Json<serde_json::Value>, String> {
    brain.restore_memory(&id).await.map_err(|e| e.to_string())?;
    Ok(Json(serde_json::json!({"success": true, "message": "Memory restored"})))
}

async fn list_deleted_memories(
    State((brain, _)): State<(Arc<Brain>, Arc<tokio::sync::RwLock<UserConfig>>)>,
    Query(params): Query<ListParams>,
) -> Result<Json<Vec<MemoryResponse>>, StatusCode> {
    let limit = params.limit.unwrap_or(100);
    let memories = brain.list_deleted_memories(limit, params.namespace.as_deref()).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

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
        is_pinned: m.is_pinned,
        memory_category: m.memory_category,
        last_ranked: m.last_ranked,
        rank_source: m.rank_source,
    }).collect()))
}
