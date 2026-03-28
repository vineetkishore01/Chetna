//! Brain - Core memory operations

use crate::db::init_db;
use crate::db::migrate_db;
use crate::db::embedding::{Embedder, EmbedderConfig};
use crate::cache::QueryCache;
use crate::history::{HistoryLogger, HistoryEvent, EventType, QueryResult};
use anyhow::{anyhow, Result};
use chrono::Utc;
use regex::Regex;
use rusqlite::{params, Connection, Row};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};
use uuid::Uuid;
use tracing::info;
use crate::shared::{blob_to_vec, vec_to_blob, cosine_similarity};

pub const MAX_CONTENT_LENGTH: usize = 50_000;
pub const MAX_TAGS: usize = 50;
pub const MAX_TAG_LENGTH: usize = 100;
pub const MAX_SEMANTIC_SEARCH_RESULTS: i64 = 1000;

pub const CATEGORIES: &[&str] = &["fact", "preference", "rule", "experience"];
pub const SCOPES: &[&str] = &["global", "session", "project"];
pub const SOURCES: &[&str] = &["agent", "user", "system"];

/// Escape special characters in FTS5 query strings
/// Prevents FTS5 query injection by escaping quotes and other special characters
fn escape_fts5_query(query: &str) -> String {
    query.replace('"', "\"\"")
}

/// Escape special characters in LIKE patterns
/// Prevents SQL injection by escaping wildcards and special characters
fn escape_like_pattern(pattern: &str) -> String {
    pattern.replace('\\', "\\\\")
           .replace('%', "\\%")
           .replace('_', "\\_")
}

fn row_to_memory(row: &Row) -> rusqlite::Result<Memory> {
    let embedding_blob: Option<Vec<u8>> = row.get(9)?;
    let embedding = embedding_blob.and_then(|blob| {
        blob_to_vec(&blob).ok()
    });

    let tags_str: String = row.get(12)?;
    let tags: Vec<String> = serde_json::from_str(&tags_str).unwrap_or_default();

    Ok(Memory {
        id: row.get(0)?,
        session_id: row.get(1)?,
        namespace: row.get(2)?,
        category: row.get(3)?,
        content: row.get(4)?,
        entities: row.get(5)?,
        importance: row.get(6)?,
        emotional_tone: row.get(7)?,
        arousal: row.get(8)?,
        embedding,
        embedding_model: row.get(10)?,
        embedding_created_at: row.get(11)?,
        tags,
        memory_type: row.get(13)?,
        access_count: row.get(14)?,
        last_accessed: row.get(15)?,
        created_at: row.get(16)?,
        updated_at: row.get(17)?,
        source: row.get(18)?,
        scope: row.get(19)?,
        is_pinned: row.get::<_, i64>(20)? != 0,
        memory_category: row.get(21)?,
        last_ranked: row.get(22)?,
        rank_source: row.get(23)?,
        deleted_at: row.get(24)?,
    })
}

// ==================== INPUT TYPES ====================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryInput {
    pub session_id: Option<String>,
    pub namespace: Option<String>,
    pub category: Option<String>,
    pub content: String,
    pub importance: Option<f32>,
    pub emotional_tone: Option<f32>,
    pub arousal: Option<f32>,
    pub tags: Option<Vec<String>>,
    pub memory_type: Option<String>,
    pub source: Option<String>,
    pub scope: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionInput {
    pub project: String,
    pub directory: Option<String>,
    pub namespace: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Stats {
    pub total_memories: i64,
    pub total_sessions: i64,
    pub categories: Vec<CategoryCount>,
    pub avg_importance: f64,
    pub sessions_active: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryCount {
    pub category: String,
    pub count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecallWeights {
    pub similarity: f32,
    pub importance: f32,
    pub recency: f32,
    pub access_frequency: f32,
    pub emotional: f32,
    pub working_memory: f32,
}

impl Default for RecallWeights {
    fn default() -> Self {
        Self {
            similarity: 0.30,
            importance: 0.20,
            recency: 0.10,
            access_frequency: 0.10,
            emotional: 0.15,
            working_memory: 0.15,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoreBreakdown {
    pub similarity: f32,
    pub importance: f32,
    pub recency: f32,
    pub access_frequency: f32,
    pub emotional: f32,
    pub working_memory: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecallFactors {
    pub is_pinned: bool,
    pub hours_old: i64,
    pub access_count: i64,
    pub memory_type: String,
    pub is_current_session: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecallScoreResult {
    pub memory: Memory,
    pub total_score: f32,
    pub breakdown: ScoreBreakdown,
    pub factors: RecallFactors,
    pub weights_used: RecallWeights,
}

// ==================== DATA STRUCTURES ====================
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Memory {
    pub id: String,
    pub session_id: Option<String>,
    pub namespace: String,
    pub category: String,
    pub content: String,
    pub entities: String,
    pub importance: f64,
    pub emotional_tone: f64,
    pub arousal: f64,
    pub embedding: Option<Vec<f32>>,
    pub embedding_model: Option<String>,
    pub embedding_created_at: Option<String>,
    pub tags: Vec<String>,
    pub memory_type: String,
    pub access_count: i64,
    pub last_accessed: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub source: String,
    pub scope: String,
    pub is_pinned: bool,
    pub memory_category: String,
    pub last_ranked: Option<String>,
    pub rank_source: Option<String>,
    pub deleted_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: String,
    pub namespace: String,
    pub name: String,
    pub agent_id: Option<String>,
    pub project: Option<String>,
    pub directory: Option<String>,
    pub started_at: String,
    pub ended_at: Option<String>,
    pub summary: Option<String>,
}

#[derive(Clone)]
pub struct Brain {
    conn: Arc<Mutex<Connection>>,
    embedder: Arc<RwLock<Option<Embedder>>>,
    config: Arc<std::sync::Mutex<Option<crate::config::Config>>>,
    connection_state: std::sync::Arc<tokio::sync::RwLock<ConnectionState>>,
    query_cache: Arc<QueryCache>,
    history_logger: Arc<HistoryLogger>,
}

#[derive(Debug, Clone, Default)]
pub struct ConnectionState {
    pub embedding_connected: bool,
    pub last_check: Option<chrono::DateTime<chrono::Utc>>,
    pub consecutive_failures: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultimodalMemory {
    pub id: String,
    pub memory_id: Option<String>,
    pub namespace: String,
    pub content_type: String,
    pub content: String,
    pub mime_type: Option<String>,
    pub file_size: Option<i64>,
    pub metadata: Option<String>,
    pub created_at: String,
}

impl Brain {
    /// Create a multimodal attachment for a memory
    #[allow(clippy::too_many_arguments)]
    pub async fn create_multimodal_memory(
        &self,
        memory_id: Option<&str>,
        content_type: &str,
        content: &str,
        mime_type: Option<&str>,
        file_size: Option<i64>,
        metadata: Option<&str>,
        namespace: Option<&str>,
    ) -> Result<String> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();
        let conn = self.conn.clone();
        
        let id_clone = id.clone();
        let mid = memory_id.map(|s| s.to_string());
        let namespace = namespace.unwrap_or("default").to_string();
        let ctype = content_type.to_string();
        let cont = content.to_string();
        let mtype = mime_type.map(|s| s.to_string());
        let meta = metadata.map(|s| s.to_string());
        
        tokio::task::spawn_blocking(move || {
            let conn = conn.blocking_lock();
            conn.execute(
                "INSERT INTO multimodal_memories (id, memory_id, namespace, content_type, content, mime_type, file_size, metadata, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                params![id_clone, mid, namespace, ctype, cont, mtype, file_size, meta, now],
            )
        }).await??;
        
        // If it's an image, we could trigger OCR here in the background
        if content_type == "image" {
            let brain_clone = self.clone();
            let id_for_ocr = id.clone();
            tokio::spawn(async move {
                if let Err(e) = brain_clone.run_ocr_on_multimodal(&id_for_ocr).await {
                    tracing::error!("OCR failed for {}: {}", id_for_ocr, e);
                }
            });
        }
        
        Ok(id)
    }

    /// OCR implementation placeholder for future expansion
    async fn run_ocr_on_multimodal(&self, multimodal_id: &str) -> Result<()> {
        tracing::info!("Running OCR on multimodal memory {}", multimodal_id);
        Ok(())
    }

    pub async fn get_multimodal_memories(&self, memory_id: &str) -> Result<Vec<MultimodalMemory>> {
        let conn = self.conn.clone();
        let mid = memory_id.to_string();
        let result = tokio::task::spawn_blocking(move || {
            let conn = conn.blocking_lock();
            let mut stmt = conn.prepare(
                "SELECT id, memory_id, namespace, content_type, content, mime_type, file_size, metadata, created_at FROM multimodal_memories WHERE memory_id = ?1"
            )?;
            let rows = stmt.query_map(params![mid], |row| {
                Ok(MultimodalMemory {
                    id: row.get(0)?,
                    memory_id: row.get(1)?,
                    namespace: row.get(2)?,
                    content_type: row.get(3)?,
                    content: row.get(4)?,
                    mime_type: row.get(5)?,
                    file_size: row.get(6)?,
                    metadata: row.get(7)?,
                    created_at: row.get(8)?,
                })
            })?;
            rows.collect::<Result<Vec<_>, _>>()
        }).await?;
        Ok(result?)
    }
    /// Create a new brain with optional embedder
    pub fn new(db_path: &str) -> Result<Self> {
        Self::new_with_embedder(db_path, None)
    }

    /// Create a new brain with embedder
    pub fn new_with_embedder(db_path: &str, embedder_config: Option<EmbedderConfig>) -> Result<Self> {
        // Ensure directory exists
        if let Some(parent) = Path::new(db_path).parent() {
            std::fs::create_dir_all(parent)?;
        }

        let conn = Connection::open(db_path)?;
        migrate_db(&conn)?;
        init_db(&conn)?;

        let conn = Arc::new(Mutex::new(conn));

        let embedder_val = embedder_config.and_then(|config| {
            let api_key = config.api_key.clone();
            let model = config.model.clone();
            match Embedder::new(
                config.provider(),
                model,
                api_key,
                config.base_url(),
                conn.clone(),
            ) {
                Ok(e) => Some(e),
                Err(e) => {
                    tracing::error!("Failed to create embedder: {}", e);
                    None
                }
            }
        });

        // Try to load config, but don't fail if not available
        let config = match crate::Config::from_env() {
            Ok(c) => Some(c),
            Err(e) => {
                tracing::warn!("Failed to load config from environment: {}. Using defaults.", e);
                None
            }
        };

        info!("Brain initialized at {}", db_path);

        if embedder_val.is_some() {
            info!("Embeddings enabled");
        }

        // Initialize query cache
        let query_cache = Arc::new(QueryCache::new(1000, 3600));
        tracing::info!("Query cache initialized with max_size=1000, ttl=3600s");

        // Initialize history logger
        let history_logger = Arc::new(HistoryLogger::new(conn.clone(), 1000)?);
        tracing::info!("History logger initialized with queue_size=1000");

        Ok(Self {
            conn,
            embedder: Arc::new(RwLock::new(embedder_val)),
            config: Arc::new(std::sync::Mutex::new(config)),
            connection_state: std::sync::Arc::new(tokio::sync::RwLock::new(ConnectionState::default())),
            query_cache,
            history_logger,
        })
    }

    /// Reload the embedder with new configuration without restarting the brain
    pub async fn reload_embedder(&self, config: EmbedderConfig) -> Result<()> {
        let conn = self.conn.clone();
        let api_key = config.api_key.clone();
        let model = config.model.clone();
        
        let new_embedder = Embedder::new(
            config.provider(),
            model,
            api_key,
            config.base_url(),
            conn,
        ).map_err(|e| anyhow!(e.to_string()))?;
        
        {
            let mut embedder_lock = self.embedder.write().await;
            *embedder_lock = Some(new_embedder);
        }
        info!("Embedder reloaded successfully");
        Ok(())
    }

    /// Check if an embedder is configured
    pub async fn has_embedder(&self) -> bool {
        self.embedder.read().await.is_some()
    }

    /// Check embedder health - returns (is_connected, error_message)
    pub async fn check_embedder_health(&self) -> (bool, Option<String>) {
        let embedder_lock = self.embedder.read().await;
        match &*embedder_lock {
            Some(emb) => {
                match emb.check_connection().await {
                    Ok(true) => (true, None),
                    Ok(false) => (false, Some("Embedder not reachable".to_string())),
                    Err(e) => (false, Some(format!("Embedder connection error: {}", e))),
                }
            }
            None => (false, Some("No embedder configured".to_string())),
        }
    }

    /// Update connection state with health check result
    pub async fn update_connection_state(&self, is_connected: bool) {
        let mut state = self.connection_state.write().await;
        state.last_check = Some(chrono::Utc::now());
        
        if is_connected {
            state.consecutive_failures = 0;
            state.embedding_connected = true;
        } else {
            state.consecutive_failures += 1;
            if state.consecutive_failures >= 3 {
                state.embedding_connected = false;
            }
        }
    }

    /// Get current connection state
    pub async fn get_connection_state(&self) -> ConnectionState {
        self.connection_state.read().await.clone()
    }

    /// Check if circuit breaker is open (too many failures)
    pub async fn is_circuit_breaker_open(&self) -> bool {
        let state = self.connection_state.read().await;
        state.consecutive_failures >= 3
    }

    /// Cleanup old history events (30-day retention)
    pub async fn cleanup_old_history(&self, days: i32) -> Result<i64> {
        self.history_logger.cleanup_old_events(days).await
    }

    /// Get history events with filters
    pub async fn get_history(&self, filters: crate::history::HistoryFilters) -> Result<Vec<crate::history::HistoryEvent>> {
        self.history_logger.get_history(filters).await
    }

    /// Get event details with query results
    pub async fn get_event_details(&self, event_id: &str) -> Result<crate::history::EventDetails> {
        self.history_logger.get_event_details(event_id).await
    }

    /// Get analytics for a time range
    pub async fn get_analytics(&self, time_range: crate::history::TimeRange) -> Result<crate::history::Analytics> {
        self.history_logger.get_analytics(time_range).await
    }

    /// Extract technical entities from text (IPs, Hashes, Paths, UUIDs)
    fn extract_entities(&self, text: &str) -> String {
        let mut entities = Vec::new();
        
        // IP Addresses (IPv4)
        let ip_regex = Regex::new(r"\b(?:\d{1,3}\.){3}\d{1,3}\b")
            .map_err(|e| anyhow!("Invalid IP regex: {}", e))
            .unwrap();
        for mat in ip_regex.find_iter(text) {
            entities.push(mat.as_str().to_string());
        }
        
        // Git Hashes / Hex strings (40 chars or 7-8 chars)
        let hex_regex = Regex::new(r"\b[0-9a-fA-F]{40}\b|\b[0-9a-fA-F]{7,8}\b")
            .map_err(|e| anyhow!("Invalid hex regex: {}", e))
            .unwrap();
        for mat in hex_regex.find_iter(text) {
            entities.push(mat.as_str().to_string());
        }
        
        // File Paths (simple heuristic)
        let path_regex = Regex::new(r"(?:/|(?:\./|(?:\.\./)+))[\w\-./]+\.[\w]+")
            .map_err(|e| anyhow!("Invalid path regex: {}", e))
            .unwrap();
        for mat in path_regex.find_iter(text) {
            entities.push(mat.as_str().to_string());
        }
        
        // UUIDs
        let uuid_regex = Regex::new(r"\b[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}\b")
            .map_err(|e| anyhow!("Invalid UUID regex: {}", e))
            .unwrap();
        for mat in uuid_regex.find_iter(text) {
            entities.push(mat.as_str().to_string());
        }
        
        entities.sort();
        entities.dedup();
        entities.join(" ")
    }

    /// Create embedding for text
    pub async fn create_embedding(&self, text: &str) -> Result<Option<crate::db::embedding::Embedding>> {
        let embedder_lock = self.embedder.read().await;
        match &*embedder_lock {
            Some(emb) => Ok(Some(emb.embed(text).await?)),
            None => Ok(None),
        }
    }

    // ==================== MEMORY OPERATIONS ====================

    pub async fn list_memories(&self, limit: i64, namespace: Option<&str>) -> Result<Vec<Memory>> {
        let conn = self.conn.clone();
        let namespace = namespace.unwrap_or("default").to_string();
        let result = tokio::task::spawn_blocking(move || {
            let conn = conn.blocking_lock();
            let mut stmt = conn.prepare(
                "SELECT id, session_id, namespace, category, content, entities, importance, emotional_tone, arousal, embedding, embedding_model, embedding_created_at, tags, memory_type, access_count, last_accessed, created_at, updated_at, source, scope, is_pinned, memory_category, last_ranked, rank_source, deleted_at FROM memories WHERE deleted_at IS NULL AND namespace = ?1 ORDER BY created_at DESC LIMIT ?2"
            )?;
            let rows = stmt.query_map(params![namespace, limit], row_to_memory)?;
            rows.collect::<Result<Vec<_>, rusqlite::Error>>()
        }).await?;
        Ok(result?)
    }

    pub async fn search_memories(&self, query: &str, limit: i64, min_similarity: f32, namespace: Option<&str>, current_session_id: Option<&str>) -> Result<Vec<Memory>> {
        if query.trim().is_empty() {
            return Ok(vec![]);
        }
        self.hybrid_search(query, limit, min_similarity, namespace, current_session_id).await
    }

    /// Hybrid Search using Reciprocal Rank Fusion (RRF)
    pub async fn hybrid_search(&self, query: &str, limit: i64, min_similarity: f32, namespace: Option<&str>, current_session_id: Option<&str>) -> Result<Vec<Memory>> {
        let k = 60.0; // RRF constant
        let limit_multiplied = limit * 2;
        let namespace = namespace.unwrap_or("default");
        let start_time = std::time::Instant::now();

        // 1. Get Keyword Results
        let keyword_results = self.keyword_search(query, limit_multiplied, Some(namespace)).await.unwrap_or_default();

        // 2. Get Semantic Results
        let semantic_results = {
            let embedder_lock = self.embedder.read().await;
            if let Some(ref embedder) = *embedder_lock {
                match embedder.embed(query).await {
                    Ok(embedding) => {
                        self.semantic_search_by_vector(&embedding.vector, limit_multiplied, min_similarity, Some(namespace), current_session_id).await.unwrap_or_default()
                    },
                    Err(e) => {
                        tracing::warn!("Embedding failed for hybrid search: {}", e);
                        vec![]
                    }
                }
            } else {
                vec![]
            }
        };

        if semantic_results.is_empty() {
            let results = Ok(keyword_results.into_iter().take(limit as usize).collect());

            // Log query event
            let duration_ms = start_time.elapsed().as_millis() as f64;
            let event_id = Uuid::new_v4().to_string();
            let event = HistoryEvent {
                id: event_id.clone(),
                event_type: EventType::QuerySearched,
                timestamp: Utc::now().to_rfc3339(),
                namespace: namespace.to_string(),
                session_id: current_session_id.map(|s| s.to_string()),
                metadata: serde_json::json!({
                    "query": query,
                    "limit": limit,
                    "min_similarity": min_similarity,
                    "results_count": results.as_ref().map(|r: &Vec<Memory>| r.len()).unwrap_or(0),
                    "duration_ms": duration_ms,
                    "search_type": "keyword_only",
                }),
            };
            self.history_logger.log_event(event)?;

            return results;
        }
        if keyword_results.is_empty() {
            let results = Ok(semantic_results.into_iter().take(limit as usize).collect());

            // Log query event
            let duration_ms = start_time.elapsed().as_millis() as f64;
            let event_id = Uuid::new_v4().to_string();
            let event = HistoryEvent {
                id: event_id.clone(),
                event_type: EventType::QuerySearched,
                timestamp: Utc::now().to_rfc3339(),
                namespace: namespace.to_string(),
                session_id: current_session_id.map(|s| s.to_string()),
                metadata: serde_json::json!({
                    "query": query,
                    "limit": limit,
                    "min_similarity": min_similarity,
                    "results_count": results.as_ref().map(|r: &Vec<Memory>| r.len()).unwrap_or(0),
                    "duration_ms": duration_ms,
                    "search_type": "semantic_only",
                }),
            };
            self.history_logger.log_event(event)?;

            return results;
        }

        let mut rrf_scores: HashMap<String, f32> = HashMap::new();
        let mut memories: HashMap<String, Memory> = HashMap::new();

        for (rank, memory) in keyword_results.into_iter().enumerate() {
            let score = 1.0 / (k + (rank + 1) as f32);
            rrf_scores.insert(memory.id.clone(), score);
            memories.insert(memory.id.clone(), memory);
        }

        for (rank, memory) in semantic_results.into_iter().enumerate() {
            let score = 1.0 / (k + (rank + 1) as f32);
            *rrf_scores.entry(memory.id.clone()).or_insert(0.0) += score;
            memories.insert(memory.id.clone(), memory);
        }

        let mut scored_memories: Vec<(String, f32)> = rrf_scores.into_iter().collect();
        scored_memories.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        let results: Vec<Memory> = scored_memories
            .into_iter()
            .take(limit as usize)
            .filter_map(|(id, _)| memories.remove(&id))
            .collect();

        // Log query event
        let duration_ms = start_time.elapsed().as_millis() as f64;
        let event_id = Uuid::new_v4().to_string();
        let event = HistoryEvent {
            id: event_id.clone(),
            event_type: EventType::QuerySearched,
            timestamp: Utc::now().to_rfc3339(),
            namespace: namespace.to_string(),
            session_id: current_session_id.map(|s| s.to_string()),
            metadata: serde_json::json!({
                "query": query,
                "limit": limit,
                "min_similarity": min_similarity,
                "results_count": results.len(),
                "duration_ms": duration_ms,
                "search_type": "hybrid",
            }),
        };
        self.history_logger.log_event(event)?;

        // Log query results
        let query_results: Vec<QueryResult> = results.iter().enumerate().map(|(idx, mem)| {
            QueryResult {
                memory_id: mem.id.clone(),
                rank: (idx + 1) as i32,
                similarity_score: None, // Hybrid search doesn't have a single similarity score
                recall_score: None,
            }
        }).collect();

        if !query_results.is_empty() {
            let _ = self.history_logger.log_query_results(&event_id, &query_results).await;
        }

        Ok(results)
    }

    async fn keyword_search(&self, query: &str, limit: i64, namespace: Option<&str>) -> Result<Vec<Memory>> {
        let conn = self.conn.clone();
        let query_str = query.to_string();
        let namespace = namespace.unwrap_or("default").to_string();
        
        if query_str.trim().is_empty() {
            return Ok(vec![]);
        }

        let entities = self.extract_entities(query);

        Ok(tokio::task::spawn_blocking(move || {
            let conn = conn.blocking_lock();
            let mut stmt = conn.prepare(
                "SELECT m.id, m.session_id, m.namespace, m.category, m.content, m.entities, m.importance, m.emotional_tone, m.arousal, m.embedding, m.embedding_model, m.embedding_created_at, m.tags, m.memory_type, m.access_count, m.last_accessed, m.created_at, m.updated_at, m.source, m.scope, m.is_pinned, m.memory_category, m.last_ranked, m.rank_source, m.deleted_at 
                 FROM memories m
                 JOIN memories_fts f ON m.id = f.memory_id
                 WHERE m.deleted_at IS NULL AND m.namespace = ?1 AND memories_fts MATCH ?2 
                 ORDER BY bm25(f) 
                 LIMIT ?3"
            )?;
            
            let safe_query = escape_fts5_query(&query_str);
            let fts_query = if !entities.is_empty() {
                let entity_terms: Vec<String> = entities.split_whitespace()
                    .map(|e| format!("entities:\"{}\"", escape_fts5_query(e)))
                    .collect();
                format!("({}) OR \"{}\"", entity_terms.join(" OR "), safe_query)
            } else {
                format!("\"{}\"", safe_query)
            };
            
            let result = stmt.query_map(params![namespace, fts_query, limit], row_to_memory);
            
            match result {
                Ok(rows) => rows.collect::<Result<Vec<_>, _>>(),
                Err(e) => {
                    tracing::warn!("FTS5 search failed, falling back to LIKE: {}", e);
                    let mut stmt = conn.prepare(
                        "SELECT id, session_id, namespace, category, content, entities, importance, emotional_tone, arousal, embedding, embedding_model, embedding_created_at, tags, memory_type, access_count, last_accessed, created_at, updated_at, source, scope, is_pinned, memory_category, last_ranked, rank_source, deleted_at FROM memories WHERE deleted_at IS NULL AND namespace = ?1 AND (content LIKE ?2 OR entities LIKE ?2) ORDER BY importance DESC LIMIT ?3"
                    )?;
                    let pattern = format!("%{}%", escape_like_pattern(&query_str));
                    let rows = stmt.query_map(params![namespace, pattern, limit], row_to_memory)?;
                    rows.collect::<Result<Vec<_>, _>>()
                }
            }
        }).await??)
    }

    pub async fn semantic_search(&self, query: &str, limit: i64, min_similarity: f32, namespace: Option<&str>, current_session_id: Option<&str>) -> Result<Vec<Memory>> {
        let limit = limit.min(MAX_SEMANTIC_SEARCH_RESULTS);
        let namespace = namespace.unwrap_or("default");
        let start_time = std::time::Instant::now();

        let query_embedding_opt = {
            let embedder_lock = self.embedder.read().await;
            if let Some(ref emb) = *embedder_lock {
                // Use query cache to avoid repeated embedding generation
                let query_string = query.to_string();
                let embedder_clone = emb.clone();
                let cached_vector = self.query_cache.get_or_create(&query_string, move |q| {
                    let query_owned = q.to_string();
                    let embedder = embedder_clone.clone();
                    async move { embedder.embed(&query_owned).await.map(|e| e.vector) }
                }).await?;
                Some(cached_vector)
            } else {
                None
            }
        };

        let results = match query_embedding_opt {
            Some(ref embedding) => self.semantic_search_by_vector(embedding, limit, min_similarity, Some(namespace), current_session_id).await,
            None => self.keyword_search(query, limit, Some(namespace)).await,
        };

        // Log query event
        let duration_ms = start_time.elapsed().as_millis() as f64;
        let event_id = Uuid::new_v4().to_string();
        let event = HistoryEvent {
            id: event_id.clone(),
            event_type: EventType::QuerySearched,
            timestamp: Utc::now().to_rfc3339(),
            namespace: namespace.to_string(),
            session_id: current_session_id.map(|s| s.to_string()),
            metadata: serde_json::json!({
                "query": query,
                "limit": limit,
                "min_similarity": min_similarity,
                "results_count": results.as_ref().map(|r: &Vec<Memory>| r.len()).unwrap_or(0),
                "duration_ms": duration_ms,
            }),
        };
        self.history_logger.log_event(event)?;

        // Log query results if we have results
        if let Ok(ref memories) = results {
            let query_results: Vec<QueryResult> = memories.iter().enumerate().map(|(idx, mem)| {
                QueryResult {
                    memory_id: mem.id.clone(),
                    rank: (idx + 1) as i32,
                    similarity_score: mem.embedding.as_ref().map(|emb| {
                        // Calculate similarity if we have the query embedding
                        if let Some(ref embedding) = query_embedding_opt {
                            cosine_similarity(embedding, emb)
                        } else {
                            0.0
                        }
                    }),
                    recall_score: None, // TODO: Calculate recall score
                }
            }).collect();

            if !query_results.is_empty() {
                let _ = self.history_logger.log_query_results(&event_id, &query_results).await;
            }
        }

        results
    }

    async fn semantic_search_by_vector(&self, query_vector: &[f32], limit: i64, min_similarity: f32, namespace: Option<&str>, current_session_id: Option<&str>) -> Result<Vec<Memory>> {
        let now = chrono::Utc::now();
        let namespace = namespace.unwrap_or("default");

        // Use linear search
        let candidates = self.linear_search_by_vector(query_vector, limit, min_similarity, namespace, current_session_id).await?;

        // Calculate recall scores and sort
        let mut scored: Vec<(Memory, f32)> = candidates
            .into_iter()
            .map(|(memory, similarity)| {
                let recall_result = Self::calculate_recall_score(&memory, similarity, &now, current_session_id);
                (recall_result.memory, recall_result.total_score)
            })
            .collect();

        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        Ok(scored.into_iter().take(limit as usize).map(|(m, _)| m).collect())
    }

    async fn linear_search_by_vector(&self, query_vector: &[f32], limit: i64, min_similarity: f32, namespace: &str, current_session_id: Option<&str>) -> Result<Vec<(Memory, f32)>> {
        let now = chrono::Utc::now();
        const BATCH_SIZE: i64 = 1000;
        const MAX_TOTAL: i64 = 5000;
        const PARALLEL_BATCHES: usize = 3;

        let mut all_scored: Vec<(Memory, f32)> = Vec::new();
        let mut offset = 0;

        while offset < MAX_TOTAL && all_scored.len() < (limit as usize) * 3 {
            // Fetch multiple batches in parallel
            let mut batch_futures = Vec::new();
            for i in 0..PARALLEL_BATCHES {
                let batch_offset = offset + (i as i64 * BATCH_SIZE);
                if batch_offset >= MAX_TOTAL {
                    break;
                }
                batch_futures.push(self.list_memories_with_embeddings_paginated_offset(BATCH_SIZE, batch_offset, Some(namespace)));
            }

            // Wait for all batches to complete
            let batch_results = futures::future::join_all(batch_futures).await;

            // Process batches in parallel
            let mut processing_futures = Vec::new();
            for batch_result in batch_results {
                let batch = batch_result?;
                if batch.is_empty() {
                    continue;
                }

                let query_vec = query_vector.to_vec();
                let now_clone = now.clone();
                let session_id = current_session_id.map(|s| s.to_string());

                processing_futures.push(tokio::task::spawn_blocking(move || {
                    batch
                        .into_iter()
                        .filter_map(|mem| {
                            if let Some(ref emb) = mem.embedding {
                                let similarity = cosine_similarity(&query_vec, emb);
                                if similarity < min_similarity {
                                    return None;
                                }
                                let recall_result = Self::calculate_recall_score(&mem, similarity, &now_clone, session_id.as_deref());
                                return Some((recall_result.memory, recall_result.total_score));
                            }
                            None
                        })
                        .collect::<Vec<(Memory, f32)>>()
                }));
            }

            // Collect all processed results
            for future in processing_futures {
                let scored_batch = future.await?;
                all_scored.extend(scored_batch);
            }

            // Sort all results by score (descending)
            all_scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

            // Early termination if we have enough good results
            if all_scored.len() >= limit as usize * 3 {
                break;
            }

            offset += (PARALLEL_BATCHES as i64) * BATCH_SIZE;
        }

        Ok(all_scored)
    }

    async fn list_memories_with_embeddings_paginated_offset(&self, limit: i64, offset: i64, namespace: Option<&str>) -> Result<Vec<Memory>> {
        let conn = self.conn.clone();
        let limit = limit.min(MAX_SEMANTIC_SEARCH_RESULTS);
        let namespace = namespace.unwrap_or("default").to_string();
        Ok(tokio::task::spawn_blocking(move || {
            let conn = conn.blocking_lock();
            let mut stmt = conn.prepare(
                "SELECT id, session_id, namespace, category, content, entities, importance, emotional_tone, arousal, embedding, embedding_model, embedding_created_at, tags, memory_type, access_count, last_accessed, created_at, updated_at, source, scope, is_pinned, memory_category, last_ranked, rank_source, deleted_at FROM memories WHERE deleted_at IS NULL AND namespace = ?1 AND embedding IS NOT NULL ORDER BY created_at DESC LIMIT ?2 OFFSET ?3"
            )?;
            let rows = stmt.query_map(params![namespace, limit, offset], row_to_memory)?;
            rows.collect::<Result<Vec<_>, _>>()
        }).await??)
    }

    pub async fn soft_delete_memory(&self, id: &str) -> Result<()> {
        let conn = self.conn.clone();
        let id = id.to_string();
        Ok(tokio::task::spawn_blocking(move || {
            let conn = conn.blocking_lock();
            let now = Utc::now().to_rfc3339();
            conn.execute(
                "UPDATE memories SET deleted_at = ?1, updated_at = ?2 WHERE id = ?3",
                params![now, now, id],
            )?;
            Ok::<_, rusqlite::Error>(())
        }).await??)
    }

    fn calculate_recall_score(memory: &Memory, similarity: f32, now: &chrono::DateTime<chrono::Utc>, current_session_id: Option<&str>) -> RecallScoreResult {
        let weights = RecallWeights::default();
        Self::calculate_recall_score_with_weights(memory, similarity, now, &weights, current_session_id)
    }

    fn calculate_recall_score_with_weights(memory: &Memory, similarity: f32, now: &chrono::DateTime<chrono::Utc>, weights: &RecallWeights, current_session_id: Option<&str>) -> RecallScoreResult {
        let similarity_contribution = similarity * weights.similarity;
        
        let importance_raw = if memory.is_pinned { 1.0 } else { memory.importance as f32 };
        let importance_contribution = importance_raw * weights.importance;
        
        let created = chrono::DateTime::parse_from_rfc3339(&memory.created_at)
            .map(|dt| dt.with_timezone(&chrono::Utc))
            .unwrap_or(*now);
        let hours_since_creation = (*now - created).num_hours() as f32;
        let recency_raw = (-hours_since_creation / 720.0).exp().max(0.1);
        let recency_contribution = recency_raw * weights.recency;
        
        let access_raw = (memory.access_count as f32 / 10.0).sqrt().min(1.0);
        let access_contribution = access_raw * weights.access_frequency;
        
        // Enhanced Emotional Salience: Combine valence intensity and arousal
        let emotional_raw = ((memory.emotional_tone.abs() as f32) + (memory.arousal as f32)) / 2.0;
        let emotional_contribution = emotional_raw * weights.emotional;

        // Working Memory Priming: Significant boost if memory belongs to current session
        let is_current_session = current_session_id.is_some() && memory.session_id.as_deref() == current_session_id;
        let working_memory_contribution = if is_current_session { weights.working_memory } else { 0.0 };

        let total_score = similarity_contribution + importance_contribution + recency_contribution + access_contribution + emotional_contribution + working_memory_contribution;
        
        RecallScoreResult {
            memory: memory.clone(),
            total_score,
            breakdown: ScoreBreakdown {
                similarity: similarity_contribution,
                importance: importance_contribution,
                recency: recency_contribution,
                access_frequency: access_contribution,
                emotional: emotional_contribution,
                working_memory: working_memory_contribution,
            },
            factors: RecallFactors {
                is_pinned: memory.is_pinned,
                hours_old: hours_since_creation as i64,
                access_count: memory.access_count,
                memory_type: memory.memory_type.clone(),
                is_current_session,
            },
            weights_used: weights.clone(),
        }
    }

    pub async fn semantic_search_with_explanation(&self, query: &str, limit: i64, min_similarity: f32, custom_weights: Option<RecallWeights>, namespace: Option<&str>, current_session_id: Option<&str>) -> Result<Vec<RecallScoreResult>> {
        let limit = limit.min(MAX_SEMANTIC_SEARCH_RESULTS);
        let weights = custom_weights.unwrap_or_default();
        let namespace = namespace.unwrap_or("default");
        
        let query_embedding_opt = {
            let embedder_lock = self.embedder.read().await;
            if let Some(ref emb) = *embedder_lock {
                Some(emb.embed(query).await?)
            } else {
                None
            }
        };

        let query_embedding = match query_embedding_opt {
            Some(emb) => emb,
            None => {
                let memories = self.keyword_search(query, limit, Some(namespace)).await?;
                let now = chrono::Utc::now();
                return Ok(memories.into_iter().map(|m| Self::calculate_recall_score_with_weights(&m, 0.5, &now, &weights, current_session_id)).take(limit as usize).collect());
            }
        };
        
        let query_vector = query_embedding.vector;
        let now = chrono::Utc::now();
        const BATCH_SIZE: i64 = 1000;
        const MAX_TOTAL: i64 = 10000;
        let mut all_scored: Vec<RecallScoreResult> = Vec::new();
        let mut offset = 0;
        while offset < MAX_TOTAL {
            let batch = self.list_memories_with_embeddings_paginated_offset(BATCH_SIZE, offset, Some(namespace)).await?;
            if batch.is_empty() { break; }
            let scored_batch: Vec<RecallScoreResult> = batch.into_iter().filter_map(|mem| {
                if let Some(ref emb) = mem.embedding {
                    let similarity = cosine_similarity(&query_vector, emb);
                    if similarity < min_similarity { return None; }
                    Some(Self::calculate_recall_score_with_weights(&mem, similarity, &now, &weights, current_session_id))
                } else { None }
            }).collect();
            all_scored.extend(scored_batch);
            if all_scored.len() >= (limit as usize) * 2 { break; }
            offset += BATCH_SIZE;
        }
        all_scored.sort_by(|a, b| b.total_score.partial_cmp(&a.total_score).unwrap_or(std::cmp::Ordering::Equal));
        Ok(all_scored.into_iter().take(limit as usize).collect())
    }

    pub async fn find_related_memories(&self, memory_id: &str, limit: i64, namespace: Option<&str>) -> Result<Vec<Memory>> {
        let limit = limit.min(100);
        let source_memory = self.get_memory(memory_id).await?;
        let source_embedding = source_memory.embedding.clone();
        if let Some(source_emb) = source_embedding {
            let memories = self.list_memories_with_embeddings_paginated(limit * 3, namespace).await?;
            let mut scored: Vec<(Memory, f32)> = memories.into_iter().filter(|m| m.id != memory_id && m.deleted_at.is_none()).filter_map(|mem| {
                if let Some(ref emb) = mem.embedding {
                    let similarity = cosine_similarity(&source_emb, emb);
                    if similarity > 0.2 { return Some((mem, similarity)); }
                }
                None
            }).collect();
            scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
            return Ok(scored.into_iter().take(limit as usize).map(|(m, _)| m).collect());
        }
        Ok(vec![])
    }

    async fn list_memories_with_embeddings_paginated(&self, limit: i64, namespace: Option<&str>) -> Result<Vec<Memory>> {
        self.list_memories_with_embeddings_paginated_offset(limit, 0, namespace).await
    }

    pub async fn list_memories_by_category(&self, category: &str, limit: i64, namespace: Option<&str>) -> Result<Vec<Memory>> {
        let conn = self.conn.clone();
        let category = category.to_string();
        let namespace = namespace.unwrap_or("default").to_string();
        Ok(tokio::task::spawn_blocking(move || {
            let conn = conn.blocking_lock();
            let mut stmt = conn.prepare(
                "SELECT id, session_id, namespace, category, content, entities, importance, emotional_tone, arousal, embedding, embedding_model, embedding_created_at, tags, memory_type, access_count, last_accessed, created_at, updated_at, source, scope, is_pinned, memory_category, last_ranked, rank_source, deleted_at FROM memories WHERE deleted_at IS NULL AND namespace = ?1 AND category = ?2 ORDER BY importance DESC, created_at DESC LIMIT ?3"
            )?;
            let rows = stmt.query_map(params![namespace, category, limit], row_to_memory)?;
            rows.collect::<Result<Vec<_>, _>>()
        }).await??)
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn create_memory(
        &self,
        content: &str,
        importance: f32,
        valence: f32,
        arousal: f32,
        tags: &[String],
        memory_type: &str,
        category: &str,
        session_id: Option<&str>,
        namespace: Option<&str>,
    ) -> Result<Memory> {
        // Input validation
        if content.is_empty() {
            return Err(anyhow!("Memory content cannot be empty"));
        }
        if content.len() > MAX_CONTENT_LENGTH {
            return Err(anyhow!("Memory content exceeds maximum length of {} characters", MAX_CONTENT_LENGTH));
        }
        if tags.len() > MAX_TAGS {
            return Err(anyhow!("Too many tags (max {})", MAX_TAGS));
        }
        for tag in tags {
            if tag.len() > MAX_TAG_LENGTH {
                return Err(anyhow!("Tag '{}' exceeds maximum length of {} characters", tag, MAX_TAG_LENGTH));
            }
        }

        // Generate ID once and use it throughout
        let id = Uuid::new_v4().to_string();
        let entities = self.extract_entities(content);
        let content_str = content.to_string();
        let tags_json = serde_json::to_string(tags)?;
        let memory_type = memory_type.to_string();
        let category = category.to_string();
        let session_id = session_id.map(|s| s.to_string());
        let namespace = namespace.unwrap_or("default").to_string();

        // Create embedding if embedder is available
        let embedding_blob = {
            let embedder_lock = self.embedder.read().await;
            if let Some(ref embedder) = *embedder_lock {
                match embedder.embed(content_str.as_str()).await {
                    Ok(emb) => {
                        let blob = vec_to_blob(&emb.vector);
                        Some((blob, embedder.model.clone()))
                    }
                    Err(e) => {
                        tracing::warn!("Failed to create embedding: {}", e);
                        None
                    }
                }
            } else {
                None
            }
        };

        // Extract embedding data before passing to blocking task
        let emb_blob = embedding_blob;
        let importance_f64 = importance as f64;
        let valence_f64 = valence as f64;
        let arousal_f64 = arousal as f64;
        let id_for_task = id.clone();
        let session_id_for_task = session_id.clone();
        let namespace_for_task = namespace.clone();
        let entities_for_task = entities.clone();
        let category_for_task = category.clone();
        let content_for_task = content_str.clone();
        let tags_for_task = tags_json.clone();
        let memory_type_for_task = memory_type.clone();

        let conn = self.conn.clone();
        tokio::task::spawn_blocking(move || {
            let conn = conn.blocking_lock();
            let now = Utc::now().to_rfc3339();

            if let Some((blob, model)) = emb_blob {
                conn.execute(
                    "INSERT INTO memories (id, session_id, namespace, category, content, entities, importance, emotional_tone, arousal, embedding, embedding_model, embedding_created_at, tags, memory_type, access_count, created_at, updated_at, source, scope) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, 0, ?15, ?16, 'agent', 'global')",
                    rusqlite::params![id_for_task, session_id_for_task, namespace_for_task, &category_for_task, &content_for_task, &entities_for_task, importance_f64, valence_f64, arousal_f64, &blob, &model, &now, &tags_for_task, &memory_type_for_task, &now, &now],
                ).map_err(|e| {
                    tracing::error!("SQLite INSERT error (with embedding): {}", e);
                    e
                })?;
            } else {
                conn.execute(
                    "INSERT INTO memories (id, session_id, namespace, category, content, entities, importance, emotional_tone, arousal, tags, memory_type, access_count, created_at, updated_at, source, scope) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, 0, ?12, ?13, 'agent', 'global')",
                    rusqlite::params![id_for_task, session_id_for_task, namespace_for_task, &category_for_task, &content_for_task, &entities_for_task, importance_f64, valence_f64, arousal_f64, &tags_for_task, &memory_type_for_task, &now, &now],
                ).map_err(|e| {
                    tracing::error!("SQLite INSERT error (no embedding): {}", e);
                    e
                })?;
            }
            Ok::<_, rusqlite::Error>(())
        }).await??;

        // Log memory creation event
        let memory = self.get_memory(&id).await?;
        let event = HistoryEvent {
            id: Uuid::new_v4().to_string(),
            event_type: EventType::MemoryCreated,
            timestamp: Utc::now().to_rfc3339(),
            namespace: namespace.clone(),
            session_id: session_id.clone(),
            metadata: serde_json::json!({
                "memory_id": id,
                "content": content_str,
                "category": category,
                "importance": importance,
                "tags": tags,
                "memory_type": memory_type,
            }),
        };
        self.history_logger.log_event(event)?;

        Ok(memory)
    }

    /// Create memory with automatic chunking for large content
    #[allow(clippy::too_many_arguments)]
    pub async fn create_memory_with_chunking(
        &self,
        content: &str,
        importance: f32,
        valence: f32,
        arousal: f32,
        tags: &[String],
        memory_type: &str,
        category: &str,
        session_id: Option<&str>,
        namespace: Option<&str>,
    ) -> Result<Vec<Memory>> {
        let chunk_size = 3000; // Roughly 750-1000 tokens
        let overlap = 300;
        let namespace = namespace.unwrap_or("default");

        if content.len() <= chunk_size {
            return Ok(vec![self.create_memory(content, importance, valence, arousal, tags, memory_type, category, session_id, Some(namespace)).await?]);
        }

        tracing::info!("Chunking large memory of {} characters", content.len());

        // Create Parent (as a summary/container)
        let summary = if content.len() > 200 {
            format!("Large document ({} chars): {}...", content.len(), &content[..200])
        } else {
            content.to_string()
        };
        
        let parent = self.create_memory(
            &summary, 
            importance, 
            valence, 
            arousal, 
            tags, 
            "document_parent", 
            category, 
            session_id,
            Some(namespace)
        ).await?;
        
        let mut memories = vec![parent.clone()];
        let rel_manager = crate::db::relationships::RelationshipManager::new(self.conn.clone());
        
        // Chunking
        let mut start = 0;
        while start < content.len() {
            let mut end = (start + chunk_size).min(content.len());
            
            // Try to find a good break point (newline or period)
            if end < content.len() {
                if let Some(newline) = content[start + chunk_size / 2 .. end].rfind('\n') {
                    end = start + chunk_size / 2 + newline + 1;
                } else if let Some(period) = content[start + chunk_size / 2 .. end].rfind('.') {
                    end = start + chunk_size / 2 + period + 1;
                }
            }
            
            let chunk_content = &content[start..end];
            if chunk_content.trim().is_empty() {
                start = end;
                continue;
            }
            
            let chunk = self.create_memory(
                chunk_content,
                importance,
                valence,
                arousal,
                tags,
                "document_chunk",
                category,
                session_id,
                Some(namespace),
            ).await?;
            
            // Link to parent
            rel_manager.create_relationship(&chunk.id, &parent.id, crate::db::relationships::RelationshipType::PartOf, 1.0).await?;
            
            memories.push(chunk);
            
            if end == content.len() {
                break;
            }
            start = end - overlap;
            if start >= content.len() {
                break;
            }
        }

        Ok(memories)
    }

    pub async fn increment_access_count(&self, id: &str) -> Result<()> {
        let conn = self.conn.clone();
        let id = id.to_string();
        let now = chrono::Utc::now().to_rfc3339();
        
        tokio::task::spawn_blocking(move || {
            let conn = conn.blocking_lock();
            conn.execute(
                "UPDATE memories SET access_count = access_count + 1, last_accessed = ?1, updated_at = ?2 WHERE id = ?3",
                params![now, now, id],
            )?;
            Ok::<_, rusqlite::Error>(())
        }).await??;
        
        Ok(())
    }

    pub async fn get_memory(&self, id: &str) -> Result<Memory> {
        let conn = self.conn.clone();
        let id = id.to_string();
        let result = tokio::task::spawn_blocking(move || {
            let conn = conn.blocking_lock();
            conn.query_row(
                "SELECT id, session_id, namespace, category, content, entities, importance, emotional_tone, arousal, embedding, embedding_model, embedding_created_at, tags, memory_type, access_count, last_accessed, created_at, updated_at, source, scope, is_pinned, memory_category, last_ranked, rank_source, deleted_at FROM memories WHERE id = ?1 AND deleted_at IS NULL",
                params![id],
                row_to_memory,
            )
        }).await?;
        Ok(result?)
    }

    pub async fn get_memory_by_id(&self, id: &str) -> Result<Option<Memory>> {
        let conn = self.conn.clone();
        let id = id.to_string();
        let result = tokio::task::spawn_blocking(move || {
            let conn = conn.blocking_lock();
            let mut stmt = conn.prepare(
                "SELECT id, session_id, namespace, category, content, entities, importance, emotional_tone, arousal, embedding, embedding_model, embedding_created_at, tags, memory_type, access_count, last_accessed, created_at, updated_at, source, scope, is_pinned, memory_category, last_ranked, rank_source, deleted_at FROM memories WHERE id = ?1 AND deleted_at IS NULL"
            )?;
            let result = stmt.query_row(params![id], row_to_memory);
            match result {
                Ok(m) => Ok(Some(m)),
                Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
                Err(e) => Err(e),
            }
        }).await?;
        Ok(result?)
    }

    pub async fn auto_score_importance(&self, content: &str) -> Result<(f32, f32, f32)> {
        // Simple heuristic-based scoring
        let content_lower = content.to_lowercase();

        let mut importance: f32 = 0.5;
        let mut valence: f32 = 0.0;
        let mut arousal: f32 = 0.0;

        // High importance indicators
        let important_keywords = ["important", "critical", "remember", "never forget", "preference", "always", "never"];
        for kw in important_keywords {
            if content_lower.contains(kw) {
                importance = (importance + 0.2).min(1.0);
            }
        }

        // Low importance indicators
        let low_importance = ["maybe", "possibly", "might", "probably"];
        for kw in low_importance {
            if content_lower.contains(kw) {
                importance = (importance - 0.15).max(0.1);
            }
        }

        // Emotional valence indicators
        let positive = ["love", "great", "awesome", "happy", "good", "best", "wonderful"];
        let negative = ["hate", "bad", "terrible", "awful", "worst", "sad", "angry"];

        for kw in positive {
            if content_lower.contains(kw) {
                valence = (valence + 0.3).min(1.0);
            }
        }
        for kw in negative {
            if content_lower.contains(kw) {
                valence = (valence - 0.3).max(-1.0);
            }
        }

        // Arousal indicators
        let high_arousal = ["urgent", "emergency", "asap", "important", "critical", "excited"];
        let low_arousal = ["calm", "slowly", "eventually", "sometime"];

        for kw in high_arousal {
            if content_lower.contains(kw) {
                arousal = (arousal + 0.3).min(1.0);
            }
        }
        for kw in low_arousal {
            if content_lower.contains(kw) {
                arousal = (arousal - 0.2).max(0.0);
            }
        }

        Ok((importance, valence, arousal))
    }

    pub async fn prune_memories(&self, days_threshold: i64, min_importance: f32) -> Result<i64> {
        let conn = self.conn.clone();
        Ok(tokio::task::spawn_blocking(move || {
            let conn = conn.blocking_lock();
            let cutoff = chrono::Utc::now() - chrono::Duration::days(days_threshold);
            let cutoff_str = cutoff.to_rfc3339();
            let now = Utc::now().to_rfc3339();

            let count = conn.execute(
                "UPDATE memories SET deleted_at = ?1, updated_at = ?2 WHERE deleted_at IS NULL AND importance < ?3 AND created_at < ?4",
                params![cutoff_str, now, min_importance, cutoff_str],
            )?;
            Ok::<_, rusqlite::Error>(count as i64)
        }).await??)
    }

    pub async fn embed_existing_memories(&self) -> Result<i64> {
        let embedder_model = {
            let embedder_lock = self.embedder.read().await;
            embedder_lock.as_ref().map(|e| e.model.clone())
        };

        if embedder_model.is_none() {
            return Ok(0);
        }
        let current_model = embedder_model.unwrap();

        // Get memories without embeddings or from different model
        let memories_to_embed = {
            let conn = self.conn.clone();
            let model_name = current_model.clone();
            tokio::task::spawn_blocking(move || {
                let conn = conn.blocking_lock();
                let mut stmt = conn.prepare(
                    "SELECT id, content FROM memories WHERE (embedding IS NULL OR embedding_model != ?1) AND deleted_at IS NULL"
                )?;
                let rows = stmt.query_map([model_name], |row| {
                    Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
                })?;
                rows.collect::<Result<Vec<_>, _>>()
            }).await??
        };

        let mut count = 0;
        for (id, content) in memories_to_embed {
            let embedder_opt = self.embedder.read().await.clone();
            if let Some(emb) = embedder_opt {
                match emb.embed(&content).await {
                    Ok(embedding) => {
                        let blob = vec_to_blob(&embedding.vector);
                        let model = embedding.model.clone();
                        let id_clone = id.clone();
                        let conn = self.conn.clone();
                        tokio::task::spawn_blocking(move || {
                            let conn = conn.blocking_lock();
                            let now = chrono::Utc::now().to_rfc3339();
                            conn.execute(
                                "UPDATE memories SET embedding = ?1, embedding_model = ?2, embedding_created_at = ?3, updated_at = ?4 WHERE id = ?5",
                                params![blob, model, now, now, id_clone],
                            )?;
                            Ok::<_, rusqlite::Error>(())
                        }).await??;
                        count += 1;
                    }
                    Err(e) => {
                        tracing::warn!("Failed to embed memory {}: {}", id, e);
                    }
                }
            }
        }

        Ok(count)
    }
pub async fn build_context(&self, query: &str, max_tokens: i64, min_importance: f32, min_similarity: f32, namespace: Option<&str>, current_session_id: Option<&str>) -> Result<serde_json::Value> {
    let namespace = namespace.unwrap_or("default");

    // Use hybrid search (RRF semantic + keyword), asking for more to allow filtering
    let raw_memories = self.hybrid_search(query, 100, min_similarity, Some(namespace), current_session_id).await?;

        // 1. Strict Importance Filter
        let filtered_memories: Vec<Memory> = raw_memories.into_iter()
            .filter(|m| m.importance as f32 >= min_importance)
            .collect();

        // 2. Deduplication (exact content)
        let mut unique_memories = Vec::new();
        let mut seen_contents = std::collections::HashSet::new();
        for memory in filtered_memories {
            if seen_contents.insert(memory.content.clone()) {
                unique_memories.push(memory);
            }
        }

        let memories = if !unique_memories.is_empty() {
            unique_memories
        } else {
            // Keyword fallback - returns by importance (semantic search already has combined scoring)
            let conn = self.conn.clone();
            let query_str = query.to_string();
            let namespace_str = namespace.to_string();
            tokio::task::spawn_blocking(move || {
                let conn = conn.blocking_lock();

                // Improved keyword search: split query into tokens and match any
                let tokens: Vec<String> = query_str.to_lowercase()
                    .split_whitespace()
                    .filter(|w| w.len() > 2)
                    .map(|w| format!("%{}%", w))
                    .collect();

                if tokens.is_empty() {
                    let mut stmt = conn.prepare(
                        "SELECT id, session_id, namespace, category, content, entities, importance, emotional_tone, arousal, embedding, embedding_model, embedding_created_at, tags, memory_type, access_count, last_accessed, created_at, updated_at, source, scope, is_pinned, memory_category, last_ranked, rank_source, deleted_at FROM memories WHERE deleted_at IS NULL AND namespace = ?1 AND importance >= ?2 ORDER BY importance DESC LIMIT 50"
                    )?;
                    let rows = stmt.query_map(params![namespace_str, min_importance], row_to_memory)?;
                    return rows.collect::<Result<Vec<_>, _>>();
                }

                // Build dynamic query for multiple tokens
                let mut query_base = "SELECT id, session_id, namespace, category, content, entities, importance, emotional_tone, arousal, embedding, embedding_model, embedding_created_at, tags, memory_type, access_count, last_accessed, created_at, updated_at, source, scope, is_pinned, memory_category, last_ranked, rank_source, deleted_at FROM memories WHERE deleted_at IS NULL AND namespace = ?1 AND importance >= ?2 AND (".to_string();
                let mut conditions = Vec::new();
                for i in 0..tokens.len() {
                    conditions.push(format!("LOWER(content) LIKE ?{}", i + 3));
                }
                query_base.push_str(&conditions.join(" OR "));
                query_base.push_str(") ORDER BY importance DESC LIMIT 50");

                let mut stmt = conn.prepare(&query_base)?;

                // Manual binding because of dynamic params - escape tokens to prevent SQL injection
                let mut params_vec: Vec<rusqlite::types::Value> = vec![rusqlite::types::Value::Text(namespace_str), rusqlite::types::Value::Real(min_importance as f64)];
                for t in tokens {
                    // Escape special characters in LIKE pattern
                    let escaped_token = format!("%{}%", escape_like_pattern(&t.to_lowercase()));
                    params_vec.push(rusqlite::types::Value::Text(escaped_token));
                }

                let rows = stmt.query_map(rusqlite::params_from_iter(params_vec), row_to_memory)?;
                rows.collect::<Result<Vec<_>, _>>()
            }).await??
        };

        // We DO NOT manually sort by importance or category here. 
        // We preserve the Hybrid RRF ranking (which mathematically blends similarity, recency, access count, keyword matches, and importance).

        let mut context_parts = Vec::new();
        let mut total_tokens: i64 = 0;
        let mut final_memories = Vec::new();

        for memory in memories {
            let mem_tokens = (memory.content.split_whitespace().count() as f64 * 1.3) as i64;
            if total_tokens + mem_tokens > max_tokens {
                break;
            }

            context_parts.push(format!(
                "[{}] {} (importance: {:.2})",
                memory.category, memory.content, memory.importance
            ));
            total_tokens += mem_tokens;
            final_memories.push(memory);
        }

        let context = context_parts.join("\n\n");

        Ok(serde_json::json!({
            "memories": final_memories.iter().take(20).map(|m| serde_json::json!({
                "id": m.id,
                "content": m.content,
                "importance": m.importance,
                "emotional_tone": m.emotional_tone,
                "arousal": m.arousal,
                "tags": m.tags,
                "memory_type": m.memory_type,
                "category": m.category,
                "embedding_model": m.embedding_model,
                "embedding_created_at": m.embedding_created_at,
                "access_count": m.access_count,
                "created_at": m.created_at,
                "updated_at": m.updated_at,
                "is_pinned": m.is_pinned,
                "memory_category": m.memory_category,
                "last_ranked": m.last_ranked,
                "rank_source": m.rank_source,
            })).collect::<Vec<_>>(),
            "total_tokens": total_tokens,
            "context": context,
        }))
    }
            
    // ==================== SESSION OPERATIONS ====================

    pub async fn list_sessions(&self, limit: i64, namespace: Option<&str>) -> Result<Vec<Session>> {
        let conn = self.conn.clone();
        let namespace = namespace.unwrap_or("default").to_string();
        Ok(tokio::task::spawn_blocking(move || {
            let conn = conn.blocking_lock();
            let mut stmt = conn.prepare(
                "SELECT id, namespace, name, agent_id, project, directory, started_at, ended_at, summary FROM sessions WHERE namespace = ?1 ORDER BY started_at DESC LIMIT ?2"
            )?;
            let rows = stmt.query_map(params![namespace, limit], |row| {
                Ok(Session {
                    id: row.get(0)?,
                    namespace: row.get(1)?,
                    name: row.get(2)?,
                    agent_id: row.get(3)?,
                    project: row.get(4)?,
                    directory: row.get(5)?,
                    started_at: row.get(6)?,
                    ended_at: row.get(7)?,
                    summary: row.get(8)?,
                })
            })?;
            rows.collect::<Result<Vec<_>, _>>()
        }).await??)
    }

    pub async fn get_session(&self, id: &str) -> Result<Session> {
        let conn = self.conn.clone();
        let id = id.to_string();
        Ok(tokio::task::spawn_blocking(move || {
            let conn = conn.blocking_lock();
            conn.query_row(
                "SELECT id, namespace, name, agent_id, project, directory, started_at, ended_at, summary FROM sessions WHERE id = ?1",
                params![id],
                |row| {
                    Ok(Session {
                        id: row.get(0)?,
                        namespace: row.get(1)?,
                        name: row.get(2)?,
                        agent_id: row.get(3)?,
                        project: row.get(4)?,
                        directory: row.get(5)?,
                        started_at: row.get(6)?,
                        ended_at: row.get(7)?,
                        summary: row.get(8)?,
                    })
                },
            )
        }).await??)
    }

    pub async fn create_session(&self, name: &str, agent_id: Option<&str>, project: Option<&str>, directory: Option<&str>, namespace: Option<&str>) -> Result<String> {
        let conn = self.conn.clone();
        let name = name.to_string();
        let agent_id = agent_id.map(|s| s.to_string());
        let project = project.map(|s| s.to_string());
        let directory = directory.map(|s| s.to_string());
        let namespace = namespace.unwrap_or("default").to_string();
        Ok(tokio::task::spawn_blocking(move || {
            let conn = conn.blocking_lock();
            let id = Uuid::new_v4().to_string();
            let now = Utc::now().to_rfc3339();
            conn.execute(
                "INSERT INTO sessions (id, namespace, name, agent_id, project, directory, started_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                params![id, namespace, name, agent_id, project, directory, now],
            )?;
            Ok::<_, rusqlite::Error>(id)
        }).await??)
    }

    pub async fn end_session(&self, id: &str) -> Result<()> {
        let conn = self.conn.clone();
        let id = id.to_string();
        let _ = tokio::task::spawn_blocking(move || {
            let conn = conn.blocking_lock();
            let now = Utc::now().to_rfc3339();
            conn.execute(
                "UPDATE sessions SET ended_at = ?1 WHERE id = ?2",
                params![now, id],
            )?;
            Ok::<_, rusqlite::Error>(())
        }).await?;
        Ok(())
    }

    pub async fn delete_session(&self, id: &str) -> Result<()> {
        let conn = self.conn.clone();
        let id = id.to_string();
        let _ = tokio::task::spawn_blocking(move || {
            let conn = conn.blocking_lock();
            conn.execute(
                "DELETE FROM sessions WHERE id = ?1",
                params![id],
            )?;
            Ok::<_, rusqlite::Error>(())
        }).await?;
        Ok(())
    }

    // ==================== STATISTICS ====================

    pub async fn get_stats(&self) -> Result<crate::api::stats::StatsResponse> {
        let conn = self.conn.clone();
        Ok(tokio::task::spawn_blocking(move || {
            let conn = conn.blocking_lock();
            let total_memories: i64 = conn.query_row(
                "SELECT COUNT(*) FROM memories WHERE deleted_at IS NULL",
                [],
                |row| row.get(0),
            ).unwrap_or(0);

            let active_memories: i64 = conn.query_row(
                "SELECT COUNT(*) FROM memories WHERE deleted_at IS NULL AND embedding IS NOT NULL",
                [],
                |row| row.get(0),
            ).unwrap_or(0);

            let deleted_memories: i64 = conn.query_row(
                "SELECT COUNT(*) FROM memories WHERE deleted_at IS NOT NULL",
                [],
                |row| row.get(0),
            ).unwrap_or(0);

            let total_sessions: i64 = conn.query_row(
                "SELECT COUNT(*) FROM sessions",
                [],
                |row| row.get(0),
            ).unwrap_or(0);

            let active_sessions: i64 = conn.query_row(
                "SELECT COUNT(*) FROM sessions WHERE ended_at IS NULL",
                [],
                |row| row.get(0),
            ).unwrap_or(0);

            let avg_importance: f32 = conn.query_row(
                "SELECT AVG(importance) FROM memories WHERE deleted_at IS NULL",
                [],
                |row| row.get::<_, Option<f64>>(0),
            ).unwrap_or(None).unwrap_or(0.5) as f32;

            let mut memory_types = HashMap::new();
            let mut stmt = conn.prepare(
                "SELECT category, COUNT(*) as cnt FROM memories WHERE deleted_at IS NULL GROUP BY category"
            )?;
            let rows = stmt.query_map([], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
            })?;
            for row in rows.flatten() {
                memory_types.insert(row.0, row.1);
            }

            Ok::<_, rusqlite::Error>(crate::api::stats::StatsResponse {
                total_memories,
                active_memories,
                deleted_memories,
                total_sessions,
                active_sessions,
                avg_importance,
                memory_types,
            })
        }).await??)
    }

    pub async fn set_memory_pinned(&self, id: &str, pinned: bool) -> Result<()> {
        let conn = self.conn.clone();
        let id = id.to_string();
        Ok(tokio::task::spawn_blocking(move || {
            let conn = conn.blocking_lock();
            let now = Utc::now().to_rfc3339();
            conn.execute(
                "UPDATE memories SET is_pinned = ?1, updated_at = ?2 WHERE id = ?3",
                params![if pinned { 1 } else { 0 }, now, id],
            )?;
            Ok::<_, rusqlite::Error>(())
        }).await??)
    }

    pub async fn set_memory_category(&self, id: &str, category: &str) -> Result<()> {
        let conn = self.conn.clone();
        let id = id.to_string();
        let category = category.to_string();
        Ok(tokio::task::spawn_blocking(move || {
            let conn = conn.blocking_lock();
            let now = Utc::now().to_rfc3339();
            conn.execute(
                "UPDATE memories SET memory_category = ?1, updated_at = ?2 WHERE id = ?3",
                params![category, now, id],
            )?;
            Ok::<_, rusqlite::Error>(())
        }).await??)
    }

    pub async fn update_memory(&self, id: &str, content: Option<&str>, importance: Option<f32>, memory_type: Option<&str>, category: Option<&str>, tags: Option<Vec<String>>) -> Result<()> {
        let conn = self.conn.clone();
        let id = id.to_string();
        let now = chrono::Utc::now().to_rfc3339();

        let content_owned = content.map(|s| s.to_string());
        let memory_type_owned = memory_type.map(|s| s.to_string());
        let category_owned = category.map(|s| s.to_string());
        let tags_owned = tags.map(|t| serde_json::to_string(&t).unwrap_or_else(|_| "[]".to_string()));

        Ok(tokio::task::spawn_blocking(move || {
            let conn = conn.blocking_lock();

            let mut updates = vec!["updated_at = ?1".to_string()];
            let mut param_idx = 2;

            if content_owned.is_some() {
                updates.push(format!("content = ?{}", param_idx));
                param_idx += 1;
            }
            if importance.is_some() {
                updates.push(format!("importance = ?{}", param_idx));
                param_idx += 1;
            }
            if memory_type_owned.is_some() {
                updates.push(format!("memory_type = ?{}", param_idx));
                param_idx += 1;
            }
            if category_owned.is_some() {
                updates.push(format!("category = ?{}", param_idx));
                param_idx += 1;
            }
            if tags_owned.is_some() {
                updates.push(format!("tags = ?{}", param_idx));
            }

            let sql = format!("UPDATE memories SET {} WHERE id = ?{}", updates.join(", "), param_idx);

            let mut params: Vec<Box<dyn rusqlite::ToSql>> = vec![Box::new(now)];

            if let Some(c) = content_owned {
                params.push(Box::new(c));
            }
            if let Some(i) = importance {
                params.push(Box::new(i));
            }
            if let Some(t) = memory_type_owned {
                params.push(Box::new(t));
            }
            if let Some(c) = category_owned {
                params.push(Box::new(c));
            }
            if let Some(t) = tags_owned {
                params.push(Box::new(t));
            }
            params.push(Box::new(id));

            let params_ref: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p.as_ref()).collect();
            conn.execute(&sql, params_ref.as_slice())?;
            Ok::<_, rusqlite::Error>(())
        }).await??)
    }

    fn get_stability_for_category(category: &str) -> f64 {
        match category {
            "system" => 10000.0,
            "preference" => 720.0,
            "fact" => 168.0,
            "experience" => 24.0,
            "rule" => 240.0,
            _ => 72.0,
        }
    }

    pub async fn apply_decay_formula(&self) -> Result<i64> {
        let config = match self.config.lock() {
            Ok(c) => c.clone(),
            Err(_) => return Ok(0),
        };

        let config = match config {
            Some(c) => c,
            None => return Ok(0),
        };

        if !config.auto_decay_enabled {
            return Ok(0);
        }

        let memories = self.list_memories(5000, None).await?;
        let now = chrono::Utc::now();
        let mut updated = 0i64;
        let mut updates_to_apply = Vec::new();

        for memory in memories {
            if memory.is_pinned {
                continue;
            }

            let created = chrono::DateTime::parse_from_rfc3339(&memory.created_at)
                .map(|dt| dt.with_timezone(&chrono::Utc))
                .unwrap_or(now);
            
             let _hours_since_creation = (now - created).num_hours() as f64;
            
            // Ebbinghaus Forgetting Curve: R = exp(-t / S)
            // S (Stability) is increased by Active Recall (access_count)
            let stability_base = Self::get_stability_for_category(&memory.memory_type);

            // Active Recall: each access significantly increases stability (logarithmic boost)
            // reset the "t" to be relative to last_accessed if it exists
            let last_active = memory.last_accessed.as_ref()
                .and_then(|la| chrono::DateTime::parse_from_rfc3339(la).ok())
                .map(|dt| dt.with_timezone(&chrono::Utc))
                .unwrap_or(created);

            let t_hours = (now - last_active).num_hours() as f64;

            let active_recall_boost = if memory.access_count > 0 {
                1.0 + (memory.access_count as f64).ln()
            } else {
                1.0
            };

            let stability = stability_base * active_recall_boost;

            // Calculate new importance using the exponential decay formula from last active point
            let decay_factor = (-t_hours / stability).exp();

            // Ensure importance doesn't drop too fast for highly important memories
            let new_importance = (memory.importance * decay_factor).clamp(0.01, 1.0);

            if (new_importance - memory.importance).abs() > 0.001 {
                updates_to_apply.push((memory.id.clone(), new_importance));
                updated += 1;
            }
            }

            if !updates_to_apply.is_empty() {
            let conn = self.conn.clone();
            let now_str = now.to_rfc3339();
            tokio::task::spawn_blocking(move || {
                let mut conn = conn.blocking_lock();
                let tx = conn.transaction()?;

                // Use a prepared statement for the batch update
                {
                    let mut stmt = tx.prepare_cached(
                        "UPDATE memories SET importance = ?1, last_ranked = ?2, rank_source = 'ebbinghaus_v3', updated_at = ?3 WHERE id = ?4"
                    )?;

                    for (id, new_imp) in updates_to_apply {
                        stmt.execute(rusqlite::params![new_imp, now_str, now_str, id])?;
                    }
                }

                tx.commit()?;
                Ok::<_, rusqlite::Error>(())
            }).await??;
            }
        Ok(updated)
    }

    pub async fn flush_low_importance(&self, threshold: f64) -> Result<i64> {
        let default_threshold = match self.config.lock() {
            Ok(c) => c.as_ref().map(|cfg| cfg.min_importance_threshold).unwrap_or(0.1),
            Err(_) => 0.1,
        };

        let effective_threshold = if threshold > 0.0 { threshold } else { default_threshold };

        let conn = self.conn.clone();
        let now = chrono::Utc::now().to_rfc3339();
        
        Ok(tokio::task::spawn_blocking(move || {
            let conn = conn.blocking_lock();
            let count = conn.execute(
                "UPDATE memories SET deleted_at = ?1, updated_at = ?2 WHERE deleted_at IS NULL AND importance < ?3 AND is_pinned = 0",
                params![now, now, effective_threshold],
            )?;
            Ok::<_, rusqlite::Error>(count as i64)
        }).await??)
    }
}
