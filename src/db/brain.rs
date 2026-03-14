//! Brain - Core memory operations

use crate::db::init_db;
use crate::db::migrate_db;
use crate::db::embedding::{Embedder, EmbedderConfig};
use anyhow::{anyhow, Result};
use chrono::Utc;
use rusqlite::{params, Connection, Row};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;
use tracing::info;

pub const MAX_CONTENT_LENGTH: usize = 50_000;
pub const MAX_TAGS: usize = 50;
pub const MAX_TAG_LENGTH: usize = 100;
pub const MAX_SEMANTIC_SEARCH_RESULTS: i64 = 1000;

pub const CATEGORIES: &[&str] = &["fact", "preference", "rule", "experience", "skill_learned"];
pub const SCOPES: &[&str] = &["global", "session", "project"];
pub const SOURCES: &[&str] = &["agent", "user", "system"];

fn row_to_memory(row: &Row) -> rusqlite::Result<Memory> {
    let embedding_blob: Option<Vec<u8>> = row.get(8)?;
    let embedding = embedding_blob.map(|blob| {
        let float_count = blob.len() / 4;
        let mut vec = vec![0.0; float_count];
        for i in 0..float_count {
            let bytes: [u8; 4] = blob[i * 4..(i + 1) * 4].try_into().unwrap_or([0, 0, 0, 0]);
            vec[i] = f32::from_le_bytes(bytes);
        }
        vec
    });
    let tags_str: String = row.get(10)?;
    let tags: Vec<String> = serde_json::from_str(&tags_str).unwrap_or_default();

    Ok(Memory {
        id: row.get(0)?,
        session_id: row.get(1)?,
        category: row.get(2)?,
        key: row.get(3)?,
        content: row.get(4)?,
        importance: row.get(5)?,
        emotional_tone: row.get(6)?,
        arousal: row.get(7)?,
        embedding,
        embedding_model: row.get(9)?,
        tags,
        memory_type: row.get(11)?,
        access_count: row.get(12)?,
        last_accessed: row.get(13)?,
        created_at: row.get(14)?,
        updated_at: row.get(15)?,
        consolidated: row.get::<_, i64>(16)? != 0,
        last_consolidated: row.get(17)?,
        source: row.get(18)?,
        source_tool: row.get(19)?,
        scope: row.get(20)?,
        deleted_at: row.get(21)?,
        is_pinned: row.get::<_, i64>(22)? != 0,
        memory_category: row.get(23)?,
        last_ranked: row.get(24)?,
        rank_source: row.get(25)?,
    })
}

// ==================== INPUT TYPES ====================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryInput {
    pub session_id: Option<String>,
    pub category: Option<String>,
    pub key: Option<String>,
    pub content: String,
    pub importance: Option<f32>,
    pub emotional_tone: Option<f32>,
    pub arousal: Option<f32>,
    pub tags: Option<Vec<String>>,
    pub memory_type: Option<String>,
    pub source: Option<String>,
    pub source_tool: Option<String>,
    pub scope: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionInput {
    pub project: String,
    pub directory: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillInput {
    pub name: String,
    pub description: String,
    pub code: String,
    pub trigger_keywords: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcedureInput {
    pub name: String,
    pub description: String,
    pub steps: Vec<String>,
    pub trigger_keywords: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchOptions {
    pub query: String,
    pub limit: Option<i64>,
    pub category: Option<String>,
    pub scope: Option<String>,
    pub session_id: Option<String>,
    pub include_deleted: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Stats {
    pub total_memories: i64,
    pub total_sessions: i64,
    pub total_skills: i64,
    pub total_procedures: i64,
    pub categories: Vec<CategoryCount>,
    pub avg_importance: f64,
    pub sessions_active: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryCount {
    pub category: String,
    pub count: i64,
}

// ==================== DATA STRUCTURES ====================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Memory {
    pub id: String,
    pub session_id: Option<String>,
    pub category: String,
    pub key: Option<String>,
    pub content: String,
    pub importance: f64,
    pub emotional_tone: f64,
    pub arousal: f64,
    pub embedding: Option<Vec<f32>>,
    pub embedding_model: Option<String>,
    pub tags: Vec<String>,
    pub memory_type: String,
    pub access_count: i64,
    pub last_accessed: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub consolidated: bool,
    pub last_consolidated: Option<String>,
    pub source: String,
    pub source_tool: Option<String>,
    pub scope: String,
    pub deleted_at: Option<String>,
    pub is_pinned: bool,
    pub memory_category: String,
    pub last_ranked: Option<String>,
    pub rank_source: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: String,
    pub name: String,
    pub agent_id: Option<String>,
    pub started_at: String,
    pub ended_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Skill {
    pub id: String,
    pub name: String,
    pub description: String,
    pub code: String,
    pub language: String,
    pub trigger_keywords: Vec<String>,
    pub enabled: bool,
    pub eligible: bool,
    pub eligible_reason: Option<String>,
    pub success_count: i64,
    pub fail_count: i64,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Procedure {
    pub id: String,
    pub name: String,
    pub description: String,
    pub steps: Vec<String>,
    pub trigger_keywords: Vec<String>,
    pub success_count: i64,
    pub fail_count: i64,
    pub last_used: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcedureStep {
    pub tool: String,
    pub args: serde_json::Value,
}

pub struct Brain {
    conn: Arc<Mutex<Connection>>,
    embedder: Option<Embedder>,
    config: std::sync::Mutex<Option<crate::config::Config>>,
}

impl Brain {
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
        init_db(&conn)?;
        migrate_db(&conn)?;

        let conn = Arc::new(Mutex::new(conn));

        let embedder = embedder_config.map(|config| {
            let api_key = config.api_key.clone();
            let model = config.model.clone();
            Embedder::new(
                config.provider(),
                model,
                api_key,
                config.base_url(),
                conn.clone(),
            )
        });

        // Try to load config, but don't fail if not available
        let config = crate::Config::from_env().ok();

        info!("Brain initialized at {}", db_path);

        if embedder.is_some() {
            info!("Embeddings enabled");
        }

        Ok(Self { conn, embedder, config: std::sync::Mutex::new(config) })
    }

    /// Check if embedder is available
    pub fn has_embedder(&self) -> bool {
        self.embedder.is_some()
    }

    /// Helper function to convert a vector of f32 to a blob (Vec<u8>)
    fn vec_to_blob(vec: &[f32]) -> Vec<u8> {
        let mut blob = Vec::with_capacity(vec.len() * 4);
        for &val in vec {
            blob.extend_from_slice(&val.to_le_bytes());
        }
        blob
    }

    /// Create embedding for text
    pub async fn create_embedding(&self, text: &str) -> Result<Option<crate::db::embedding::Embedding>> {
        match &self.embedder {
            Some(emb) => Ok(Some(emb.embed(text).await?)),
            None => Ok(None),
        }
    }

    // ==================== MEMORY OPERATIONS ====================

    pub async fn list_memories(&self, limit: i64) -> Result<Vec<Memory>> {
        let conn = self.conn.clone();
        let limit = limit.min(1000);
        let result = tokio::task::spawn_blocking(move || {
            let conn = conn.blocking_lock();
            let mut stmt = conn.prepare(
                "SELECT id, session_id, category, key, content, importance, emotional_tone, arousal, embedding, embedding_model, tags, memory_type, access_count, last_accessed, created_at, updated_at, consolidated, last_consolidated, source, source_tool, scope, deleted_at, is_pinned, memory_category, last_ranked, rank_source FROM memories WHERE deleted_at IS NULL ORDER BY created_at DESC LIMIT ?1"
            )?;
            let rows = stmt.query_map(params![limit], row_to_memory)?;
            rows.collect::<Result<Vec<_>, rusqlite::Error>>()
        }).await?;
        Ok(result?)
    }

    pub async fn soft_delete_memory(&self, id: &str) -> Result<()> {
        let conn = self.conn.clone();
        let id = id.to_string();
        Ok(tokio::task::spawn_blocking(move || {
            let conn = conn.blocking_lock();
            let now = Utc::now().to_rfc3339();
            conn.execute(
                "UPDATE memories SET deleted_at = ?1 WHERE id = ?2",
                params![now, id],
            )?;
            Ok::<_, rusqlite::Error>(())
        }).await??)
    }

    pub async fn restore_memory(&self, id: &str) -> Result<()> {
        let conn = self.conn.clone();
        let id = id.to_string();
        Ok(tokio::task::spawn_blocking(move || {
            let conn = conn.blocking_lock();
            conn.execute(
                "UPDATE memories SET deleted_at = NULL WHERE id = ?1",
                params![id],
            )?;
            Ok::<_, rusqlite::Error>(())
        }).await??)
    }

    pub async fn list_deleted_memories(&self, limit: i64) -> Result<Vec<Memory>> {
        let conn = self.conn.clone();
        let limit = limit.min(1000);
        Ok(tokio::task::spawn_blocking(move || {
            let conn = conn.blocking_lock();
            let mut stmt = conn.prepare(
                "SELECT id, session_id, category, key, content, importance, emotional_tone, arousal, embedding, embedding_model, tags, memory_type, access_count, last_accessed, created_at, updated_at, consolidated, last_consolidated, source, source_tool, scope, deleted_at, is_pinned, memory_category, last_ranked, rank_source FROM memories WHERE deleted_at IS NOT NULL ORDER BY deleted_at DESC LIMIT ?1"
            )?;
            let rows = stmt.query_map(params![limit], row_to_memory)?;
            rows.collect::<Result<Vec<_>, _>>()
        }).await??)
    }

    pub async fn search_memories(&self, query: &str, limit: i64) -> Result<Vec<Memory>> {
        let limit = limit.min(1000);

        // Try semantic search first if embedder is available and query is not empty
        if let Some(ref embedder) = self.embedder {
            if !query.trim().is_empty() {
                tracing::info!("🔍 Semantic search for: {}", query);
                match embedder.embed(query).await {
                    Ok(_) => {
                        tracing::debug!("Embedding created successfully, running semantic search");
                        return Box::pin(self.semantic_search(query, limit, 0.3)).await;
                    },
                    Err(e) => {
                        tracing::warn!("Embedder failed, falling back to keyword search: {}", e);
                    }
                }
            }
        }

        // Fall back to keyword search
        tracing::debug!("Keyword search for: {}", query);
        self.keyword_search(query, limit).await
    }

    /// Keyword-based search (fallback when embeddings unavailable)
    async fn keyword_search(&self, query: &str, limit: i64) -> Result<Vec<Memory>> {
        let conn = self.conn.clone();
        let query_str = query.to_string();
        Ok(tokio::task::spawn_blocking(move || {
            let conn = conn.blocking_lock();
            let mut stmt = conn.prepare(
                "SELECT id, session_id, category, key, content, importance, emotional_tone, arousal, embedding, embedding_model, tags, memory_type, access_count, last_accessed, created_at, updated_at, consolidated, last_consolidated, source, source_tool, scope, deleted_at, is_pinned, memory_category, last_ranked, rank_source FROM memories WHERE deleted_at IS NULL AND content LIKE ?1 ORDER BY importance DESC LIMIT ?2"
            )?;
            let pattern = format!("%{}%", query_str);
            let rows = stmt.query_map(params![pattern, limit], row_to_memory)?;
            rows.collect::<Result<Vec<_>, _>>()
        }).await??)
    }

    /// Semantic search using embeddings - finds memories with similar meaning
    pub async fn semantic_search(&self, query: &str, limit: i64, min_similarity: f32) -> Result<Vec<Memory>> {
        let limit = limit.min(MAX_SEMANTIC_SEARCH_RESULTS);
        
        // Create embedding for the query
        let query_embedding = match &self.embedder {
            Some(emb) => emb.embed(query).await?,
            None => return self.keyword_search(query, limit).await,
        };

        let query_vector = query_embedding.vector;

        // Fetch memories with embeddings (paginated with limit to prevent OOM)
        let memories = self.list_memories_with_embeddings_paginated(limit * 2).await?;

        // Calculate similarity for each memory
        let mut scored_memories: Vec<(Memory, f32)> = memories
            .into_iter()
            .filter_map(|mem| {
                if let Some(ref emb) = mem.embedding {
                    let similarity = Embedder::cosine_similarity(&query_vector, emb);
                    if similarity >= min_similarity {
                        return Some((mem, similarity));
                    }
                }
                None
            })
            .collect();

        // Sort by similarity
        scored_memories.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        // Return top results
        Ok(scored_memories.into_iter().take(limit as usize).map(|(m, _)| m).collect())
    }

    /// Find related memories - memories that are semantically similar to a given memory
    pub async fn find_related_memories(&self, memory_id: &str, limit: i64) -> Result<Vec<Memory>> {
        let limit = limit.min(100);
        
        // Get the source memory first
        let source_memory = self.get_memory(memory_id).await?;
        
        let source_embedding = source_memory.embedding.clone();

        if let Some(source_emb) = source_embedding {
            // Fetch memories with embeddings for comparison
            let memories = self.list_memories_with_embeddings_paginated(limit * 3).await?;

            // Calculate similarity with all other memories (excluding deleted)
            let mut scored: Vec<(Memory, f32)> = memories
                .into_iter()
                .filter(|m| m.id != memory_id && m.deleted_at.is_none())
                .filter_map(|mem| {
                    if let Some(ref emb) = mem.embedding {
                        let similarity = Embedder::cosine_similarity(&source_emb, emb);
                        if similarity > 0.5 {
                            return Some((mem, similarity));
                        }
                    }
                    None
                })
                .collect();

            scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
            return Ok(scored.into_iter().take(limit as usize).map(|(m, _)| m).collect());
        }

        Ok(vec![])
    }

    /// List memories with embeddings (paginated) - used for semantic search
    /// Now properly filters out deleted memories
    async fn list_memories_with_embeddings_paginated(&self, limit: i64) -> Result<Vec<Memory>> {
        let conn = self.conn.clone();
        let limit = limit.min(MAX_SEMANTIC_SEARCH_RESULTS);
        Ok(tokio::task::spawn_blocking(move || {
            let conn = conn.blocking_lock();
            let mut stmt = conn.prepare(
                "SELECT id, session_id, category, key, content, importance, emotional_tone, arousal, embedding, embedding_model, tags, memory_type, access_count, last_accessed, created_at, updated_at, consolidated, last_consolidated, source, source_tool, scope, deleted_at, is_pinned, memory_category, last_ranked, rank_source FROM memories WHERE deleted_at IS NULL AND embedding IS NOT NULL ORDER BY created_at DESC LIMIT ?1"
            )?;
            let rows = stmt.query_map(params![limit], row_to_memory)?;
            rows.collect::<Result<Vec<_>, _>>()
        }).await??)
    }

    pub async fn consolidate_memories(&self) -> Result<i64> {
        let conn = self.conn.clone();
        Ok(tokio::task::spawn_blocking(move || {
            let conn = conn.blocking_lock();
            let now = Utc::now().to_rfc3339();
            let count = conn.execute(
                "UPDATE memories SET consolidated = 1, last_consolidated = ?1 WHERE consolidated = 0 AND importance > 0.7",
                params![now],
            )?;
            Ok::<_, rusqlite::Error>(count as i64)
        }).await??)
    }

    pub async fn list_memories_by_category(&self, category: &str, limit: i64) -> Result<Vec<Memory>> {
        let conn = self.conn.clone();
        let category = category.to_string();
        let limit = limit.min(1000);
        Ok(tokio::task::spawn_blocking(move || {
            let conn = conn.blocking_lock();
            let mut stmt = conn.prepare(
                "SELECT id, session_id, category, key, content, importance, emotional_tone, arousal, embedding, embedding_model, tags, memory_type, access_count, last_accessed, created_at, updated_at, consolidated, last_consolidated, source, source_tool, scope, deleted_at, is_pinned, memory_category, last_ranked, rank_source FROM memories WHERE deleted_at IS NULL AND category = ?1 ORDER BY importance DESC, created_at DESC LIMIT ?2"
            )?;
            let rows = stmt.query_map(params![category, limit], row_to_memory)?;
            rows.collect::<Result<Vec<_>, _>>()
        }).await??)
    }

    /// Create a new memory with the given content and metadata
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
    ) -> Result<String> {
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
        let content = content.to_string();
        let tags_json = serde_json::to_string(tags)?;
        let memory_type = memory_type.to_string();
        let category = category.to_string();
        let session_id = session_id.map(|s| s.to_string());

        // Create embedding if embedder is available
        let embedding_blob = if let Some(ref embedder) = self.embedder {
            match embedder.embed(content.as_str()).await {
                Ok(emb) => {
                    let blob = Self::vec_to_blob(&emb.vector);
                    Some((blob, embedder.model.clone()))
                }
                Err(e) => {
                    tracing::warn!("Failed to create embedding: {}", e);
                    None
                }
            }
        } else {
            None
        };

        // Extract embedding data before passing to blocking task
        let emb_blob = embedding_blob;
        let importance_f64 = importance as f64;
        let valence_f64 = valence as f64;
        let arousal_f64 = arousal as f64;
        let id_for_task = id.clone();
        let session_id_for_task = session_id.clone();

        let conn = self.conn.clone();
        tokio::task::spawn_blocking(move || {
            let conn = conn.blocking_lock();
            let now = Utc::now().to_rfc3339();

            if let Some((blob, model)) = emb_blob {
                conn.execute(
                    "INSERT INTO memories (id, session_id, category, content, importance, emotional_tone, arousal, embedding, embedding_model, embedding_created_at, tags, memory_type, access_count, created_at, updated_at, consolidated, source, scope) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, 0, ?13, ?14, 0, 'agent', 'global')",
                    rusqlite::params![id_for_task, session_id_for_task, &category, &content, importance_f64, valence_f64, arousal_f64, &blob, &model, &now, &tags_json, &memory_type, &now, &now],
                ).map_err(|e| {
                    tracing::error!("SQLite INSERT error (with embedding): {}", e);
                    e
                })?;
            } else {
                conn.execute(
                    "INSERT INTO memories (id, session_id, category, content, importance, emotional_tone, arousal, tags, memory_type, access_count, created_at, updated_at, consolidated, source, scope) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, 0, ?10, ?11, 0, 'agent', 'global')",
                    rusqlite::params![id_for_task, session_id_for_task, &category, &content, importance_f64, valence_f64, arousal_f64, &tags_json, &memory_type, &now, &now],
                ).map_err(|e| {
                    tracing::error!("SQLite INSERT error (no embedding): {}", e);
                    e
                })?;
            }
            Ok::<_, rusqlite::Error>(())
        }).await??;

        Ok(id)
    }

    pub async fn get_memory(&self, id: &str) -> Result<Memory> {
        let conn = self.conn.clone();
        let id = id.to_string();
        let result = tokio::task::spawn_blocking(move || {
            let conn = conn.blocking_lock();
            conn.query_row(
                "SELECT id, session_id, category, key, content, importance, emotional_tone, arousal, embedding, embedding_model, tags, memory_type, access_count, last_accessed, created_at, updated_at, consolidated, last_consolidated, source, source_tool, scope, deleted_at, is_pinned, memory_category, last_ranked, rank_source FROM memories WHERE id = ?1 AND deleted_at IS NULL",
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
                "SELECT id, session_id, category, key, content, importance, emotional_tone, arousal, embedding, embedding_model, tags, memory_type, access_count, last_accessed, created_at, updated_at, consolidated, last_consolidated, source, source_tool, scope, deleted_at, is_pinned, memory_category, last_ranked, rank_source FROM memories WHERE id = ?1 AND deleted_at IS NULL"
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
        // In production, this would use an LLM to score
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

            let count = conn.execute(
                "UPDATE memories SET deleted_at = ?1 WHERE deleted_at IS NULL AND importance < ?2 AND created_at < ?3",
                params![cutoff_str, min_importance, cutoff_str],
            )?;
            Ok::<_, rusqlite::Error>(count as i64)
        }).await??)
    }

    pub async fn embed_existing_memories(&self) -> Result<i64> {
        if self.embedder.is_none() {
            return Ok(0);
        }

        let embedder = self.embedder.clone();

        // Get memories without embeddings
        let memories_without_emb = {
            let conn = self.conn.clone();
            tokio::task::spawn_blocking(move || {
                let conn = conn.blocking_lock();
                let mut stmt = conn.prepare(
                    "SELECT id, content FROM memories WHERE embedding IS NULL AND deleted_at IS NULL"
                )?;
                let rows = stmt.query_map([], |row| {
                    Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
                })?;
                rows.collect::<Result<Vec<_>, _>>()
            }).await??
        };

        let mut count = 0;
        for (id, content) in memories_without_emb {
            if let Some(ref emb) = embedder {
                match emb.embed(&content).await {
                    Ok(embedding) => {
                        let blob = Self::vec_to_blob(&embedding.vector);
                        let model = embedding.model.clone();
                        let id_clone = id.clone();
                        let conn = self.conn.clone();
                        tokio::task::spawn_blocking(move || {
                            let conn = conn.blocking_lock();
                            let now = chrono::Utc::now().to_rfc3339();
                            conn.execute(
                                "UPDATE memories SET embedding = ?1, embedding_model = ?2, embedding_created_at = ?3 WHERE id = ?4",
                                params![blob, model, now, id_clone],
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

    pub async fn build_context(&self, query: &str, max_tokens: i64, _min_importance: f32) -> Result<serde_json::Value> {
        // Try semantic search first
        let semantic_result = self.semantic_search(query, 50, 0.3).await;
        
        // If semantic search returns poor results, fall back to keyword search
        let memories = match semantic_result {
            Ok(semantic_memories) if !semantic_memories.is_empty() => Ok(semantic_memories),
            _ => {
                // Keyword fallback
                let conn = self.conn.clone();
                let query_str = query.to_string();
                tokio::task::spawn_blocking(move || {
                    let conn = conn.blocking_lock();
                    let pattern = format!("%{}%", query_str.to_lowercase());
                    let mut stmt = conn.prepare(
                        "SELECT id, session_id, category, key, content, importance, emotional_tone, arousal, embedding, embedding_model, tags, memory_type, access_count, last_accessed, created_at, updated_at, consolidated, last_consolidated, source, source_tool, scope, deleted_at, is_pinned, memory_category, last_ranked, rank_source FROM memories WHERE deleted_at IS NULL AND LOWER(content) LIKE ?1 ORDER BY importance DESC LIMIT 50"
                    )?;
                    let rows = stmt.query_map(params![pattern], |row| {
                        Ok(Memory {
                            id: row.get(0)?,
                            session_id: row.get(1)?,
                            category: row.get(2)?,
                            key: row.get(3)?,
                            content: row.get(4)?,
                            importance: row.get(5)?,
                            emotional_tone: row.get(6)?,
                            arousal: row.get(7)?,
                            embedding: None,
                            embedding_model: row.get(9)?,
                            tags: serde_json::from_str(&row.get::<_, String>(10)?).unwrap_or_default(),
                            memory_type: row.get(11)?,
                            access_count: row.get(12)?,
                            last_accessed: row.get(13)?,
                            created_at: row.get(14)?,
                            updated_at: row.get(15)?,
                            consolidated: row.get::<_, i64>(16)? != 0,
                            last_consolidated: row.get(17)?,
                            source: row.get(18)?,
                            source_tool: row.get(19)?,
                            scope: row.get(20)?,
                            deleted_at: row.get(21)?,
                            is_pinned: row.get::<_, i64>(22).unwrap_or(0) != 0,
                            memory_category: row.get::<_, String>(23).unwrap_or_else(|_| "general".to_string()),
                            last_ranked: row.get(24).ok(),
                            rank_source: row.get(25).ok(),
                        })
                    })?;
                    rows.collect::<Result<Vec<_>, _>>()
                }).await?
            }
        }?;

        let mut context_parts = Vec::new();
        let mut total_tokens: i64 = 0;
        let tokens_per_char: usize = 4;

        for memory in &memories {
            let mem_tokens = (memory.content.len() / tokens_per_char) as i64;
            if total_tokens + mem_tokens > max_tokens {
                break;
            }

            context_parts.push(format!(
                "[{}] {} (importance: {:.2})",
                memory.category, memory.content, memory.importance
            ));
            total_tokens += mem_tokens;
        }

        let context = context_parts.join("\n\n");

        Ok(serde_json::json!({
            "memories": memories.iter().take(20).map(|m| serde_json::json!({
                "id": m.id,
                "content": m.content,
                "importance": m.importance,
                "emotional_tone": m.emotional_tone,
                "arousal": m.arousal,
                "tags": m.tags,
                "memory_type": m.memory_type,
                "category": m.category,
                "embedding_model": m.embedding_model,
                "access_count": m.access_count,
                "created_at": m.created_at,
                "updated_at": m.updated_at,
                "consolidated": m.consolidated,
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

    pub async fn list_sessions(&self, limit: i64) -> Result<Vec<Session>> {
        let conn = self.conn.clone();
        Ok(tokio::task::spawn_blocking(move || {
            let conn = conn.blocking_lock();
            let mut stmt = conn.prepare(
                "SELECT id, name, agent_id, started_at, ended_at FROM sessions ORDER BY started_at DESC LIMIT ?1"
            )?;
            let rows = stmt.query_map(params![limit], |row| {
                Ok(Session {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    agent_id: row.get(2)?,
                    started_at: row.get(3)?,
                    ended_at: row.get(4)?,
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
                "SELECT id, name, agent_id, started_at, ended_at FROM sessions WHERE id = ?1",
                params![id],
                |row| {
                    Ok(Session {
                        id: row.get(0)?,
                        name: row.get(1)?,
                        agent_id: row.get(2)?,
                        started_at: row.get(3)?,
                        ended_at: row.get(4)?,
                    })
                },
            )
        }).await??)
    }

    pub async fn create_session(&self, name: &str, agent_id: Option<&str>) -> Result<String> {
        let conn = self.conn.clone();
        let name = name.to_string();
        let agent_id = agent_id.map(|s| s.to_string());
        Ok(tokio::task::spawn_blocking(move || {
            let conn = conn.blocking_lock();
            let id = Uuid::new_v4().to_string();
            let now = Utc::now().to_rfc3339();
            conn.execute(
                "INSERT INTO sessions (id, name, agent_id, started_at) VALUES (?1, ?2, ?3, ?4)",
                params![id, name, agent_id, now],
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

    // ==================== SKILL OPERATIONS ====================

    pub async fn list_skills(&self) -> Result<Vec<Skill>> {
        let conn = self.conn.clone();
        Ok(tokio::task::spawn_blocking(move || {
            let conn = conn.blocking_lock();
            let mut stmt = conn.prepare(
                "SELECT id, name, description, code, language, trigger_keywords, enabled, eligible, eligible_reason, success_count, fail_count, created_at, updated_at FROM skills ORDER BY created_at DESC"
            )?;
            let rows = stmt.query_map([], |row| {
                let keywords_str: String = row.get(5)?;
                let keywords: Vec<String> = serde_json::from_str(&keywords_str).unwrap_or_default();
                Ok(Skill {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    description: row.get(2)?,
                    code: row.get(3)?,
                    language: row.get(4)?,
                    trigger_keywords: keywords,
                    enabled: row.get::<_, i64>(6)? != 0,
                    eligible: row.get::<_, i64>(7)? != 0,
                    eligible_reason: row.get(8)?,
                    success_count: row.get(9)?,
                    fail_count: row.get(10)?,
                    created_at: row.get(11)?,
                    updated_at: row.get(12)?,
                })
            })?;
            rows.collect::<Result<Vec<_>, _>>()
        }).await??)
    }

    pub async fn get_skill(&self, id: &str) -> Result<Skill> {
        let conn = self.conn.clone();
        let id = id.to_string();
        Ok(tokio::task::spawn_blocking(move || {
            let conn = conn.blocking_lock();
            conn.query_row(
                "SELECT id, name, description, code, language, trigger_keywords, enabled, eligible, eligible_reason, success_count, fail_count, created_at, updated_at FROM skills WHERE id = ?1",
                params![id],
                |row| {
                    let keywords_str: String = row.get(5)?;
                    let keywords: Vec<String> = serde_json::from_str(&keywords_str).unwrap_or_default();
                    Ok(Skill {
                        id: row.get(0)?,
                        name: row.get(1)?,
                        description: row.get(2)?,
                        code: row.get(3)?,
                        language: row.get(4)?,
                        trigger_keywords: keywords,
                        enabled: row.get::<_, i64>(6)? != 0,
                        eligible: row.get::<_, i64>(7)? != 0,
                        eligible_reason: row.get(8)?,
                        success_count: row.get(9)?,
                        fail_count: row.get(10)?,
                        created_at: row.get(11)?,
                        updated_at: row.get(12)?,
                    })
                },
            )
        }).await??)
    }

    pub async fn create_skill(&self, name: &str, description: &str, code: &str, language: &str) -> Result<String> {
        let conn = self.conn.clone();
        let name = name.to_string();
        let description = description.to_string();
        let code = code.to_string();
        let language = language.to_string();
        Ok(tokio::task::spawn_blocking(move || {
            let conn = conn.blocking_lock();
            let id = Uuid::new_v4().to_string();
            let now = Utc::now().to_rfc3339();
            conn.execute(
                "INSERT INTO skills (id, name, description, code, language, enabled, eligible, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, 1, 1, ?6, ?7)",
                params![id, name, description, code, language, now, now],
            )?;
            Ok::<_, rusqlite::Error>(id)
        }).await??)
    }

    pub async fn delete_skill(&self, id: &str) -> Result<()> {
        let conn = self.conn.clone();
        let id = id.to_string();
        let _ = tokio::task::spawn_blocking(move || {
            let conn = conn.blocking_lock();
            conn.execute("DELETE FROM skills WHERE id = ?1", params![id])?;
            Ok::<_, rusqlite::Error>(())
        }).await?;
        Ok(())
    }

    // ==================== PROCEDURE OPERATIONS ====================

    pub async fn list_procedures(&self) -> Result<Vec<Procedure>> {
        let conn = self.conn.clone();
        Ok(tokio::task::spawn_blocking(move || {
            let conn = conn.blocking_lock();
            let mut stmt = conn.prepare(
                "SELECT id, name, description, steps, trigger_keywords, success_count, fail_count, last_used, created_at FROM procedures ORDER BY created_at DESC"
            )?;
            let rows = stmt.query_map([], |row| {
                let steps_str: String = row.get(3)?;
                let steps: Vec<String> = serde_json::from_str(&steps_str).unwrap_or_default();
                let keywords_str: String = row.get(4)?;
                let keywords: Vec<String> = serde_json::from_str(&keywords_str).unwrap_or_default();
                Ok(Procedure {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    description: row.get(2)?,
                    steps,
                    trigger_keywords: keywords,
                    success_count: row.get(5)?,
                    fail_count: row.get(6)?,
                    last_used: row.get(7)?,
                    created_at: row.get(8)?,
                })
            })?;
            rows.collect::<Result<Vec<_>, _>>()
        }).await??)
    }

    pub async fn get_procedure(&self, id: &str) -> Result<Procedure> {
        let conn = self.conn.clone();
        let id = id.to_string();
        Ok(tokio::task::spawn_blocking(move || {
            let conn = conn.blocking_lock();
            conn.query_row(
                "SELECT id, name, description, steps, trigger_keywords, success_count, fail_count, last_used, created_at FROM procedures WHERE id = ?1",
                params![id],
                |row| {
                    let steps_str: String = row.get(3)?;
                    let steps: Vec<String> = serde_json::from_str(&steps_str).unwrap_or_default();
                    let keywords_str: String = row.get(4)?;
                    let keywords: Vec<String> = serde_json::from_str(&keywords_str).unwrap_or_default();
                    Ok(Procedure {
                        id: row.get(0)?,
                        name: row.get(1)?,
                        description: row.get(2)?,
                        steps,
                        trigger_keywords: keywords,
                        success_count: row.get(5)?,
                        fail_count: row.get(6)?,
                        last_used: row.get(7)?,
                        created_at: row.get(8)?,
                    })
                },
            )
        }).await??)
    }

    pub async fn create_procedure(&self, name: &str, description: &str, steps: &[String]) -> Result<String> {
        let conn = self.conn.clone();
        let name = name.to_string();
        let description = description.to_string();
        let steps_json = serde_json::to_string(steps)?;
        Ok(tokio::task::spawn_blocking(move || {
            let conn = conn.blocking_lock();
            let id = Uuid::new_v4().to_string();
            let now = Utc::now().to_rfc3339();
            conn.execute(
                "INSERT INTO procedures (id, name, description, steps, created_at) VALUES (?1, ?2, ?3, ?4, ?5)",
                params![id, name, description, steps_json, now],
            )?;
            Ok::<_, rusqlite::Error>(id)
        }).await??)
    }

    pub async fn delete_procedure(&self, id: &str) -> Result<()> {
        let conn = self.conn.clone();
        let id = id.to_string();
        let _ = tokio::task::spawn_blocking(move || {
            let conn = conn.blocking_lock();
            conn.execute("DELETE FROM procedures WHERE id = ?1", params![id])?;
            Ok::<_, rusqlite::Error>(())
        }).await?;
        Ok(())
    }

    pub async fn execute_procedure(&self, id: &str, _parameters: serde_json::Value) -> Result<serde_json::Value> {
        let _proc = self.get_procedure(id).await?;
        Ok(serde_json::json!({"status": "executed", "procedure_id": id}))
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
                "SELECT COUNT(*) FROM memories WHERE deleted_at IS NULL AND consolidated = 0",
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

            let total_skills: i64 = conn.query_row(
                "SELECT COUNT(*) FROM skills",
                [],
                |row| row.get(0),
            ).unwrap_or(0);

            let total_procedures: i64 = conn.query_row(
                "SELECT COUNT(*) FROM procedures",
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
                total_skills,
                total_procedures,
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
            conn.execute(
                "UPDATE memories SET is_pinned = ?1 WHERE id = ?2",
                params![if pinned { 1 } else { 0 }, id],
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
            conn.execute(
                "UPDATE memories SET memory_category = ?1 WHERE id = ?2",
                params![category, id],
            )?;
            Ok::<_, rusqlite::Error>(())
        }).await??)
    }

    pub async fn update_memory_importance(&self, id: &str, importance: f64, rank_source: &str) -> Result<()> {
        let conn = self.conn.clone();
        let id = id.to_string();
        let rank_source = rank_source.to_string();
        let now = chrono::Utc::now().to_rfc3339();
        Ok(tokio::task::spawn_blocking(move || {
            let conn = conn.blocking_lock();
            conn.execute(
                "UPDATE memories SET importance = ?1, last_ranked = ?2, rank_source = ?3 WHERE id = ?4",
                params![importance, now, rank_source, id],
            )?;
            Ok::<_, rusqlite::Error>(())
        }).await??)
    }

    pub async fn consolidate_memories_llm(&self, limit: i64) -> Result<(i64, i64)> {
        let config = match self.config.lock() {
            Ok(c) => c.clone(),
            Err(_) => return Ok((0, 0)),
        };

        let config = match config {
            Some(c) => c,
            None => return Ok((0, 0)),
        };

        if !config.has_llm() {
            tracing::warn!("LLM not configured, skipping LLM consolidation");
            return Ok((0, 0));
        }

        tracing::info!("🔄 Starting LLM consolidation (limit: {})", limit);
        tracing::info!("   Using LLM model: {} at {}", config.llm_model, config.llm_base_url.as_ref().unwrap_or(&"unknown".to_string()));

        let memories = self.list_memories(limit).await?;
        tracing::info!("   Found {} memories to process", memories.len());

        let base_url = config.llm_base_url.unwrap_or_else(|| "http://localhost:11434".to_string());
        let model = config.llm_model.clone();

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .unwrap_or_default();

        let mut processed = 0i64;
        let mut updated = 0i64;

        for memory in memories.iter().take(50) {
            if memory.is_pinned {
                tracing::debug!("Skipping pinned memory: {}", memory.id);
                continue;
            }

            tracing::debug!("Processing memory {}/{}: {}", processed + 1, memories.len().min(50), memory.content.chars().take(50).collect::<String>());

            let prompt = format!(
                "Analyze this memory and rate its importance from 0.0 to 1.0. Consider: factual knowledge (higher), user preferences (higher), emotional content (higher), transient info (lower).\n\nMemory: {}\n\nRespond with only a number between 0.0 and 1.0.",
                memory.content
            );

            let request_body = serde_json::json!({
                "model": model,
                "messages": [{"role": "user", "content": prompt}],
                "stream": false
            });

            if let Ok(resp) = client.post(format!("{}/api/chat", base_url)).json(&request_body).send().await {
                if resp.status().is_success() {
                    if let Ok(json) = resp.json::<serde_json::Value>().await {
                        if let Some(content) = json.get("message").and_then(|m| m.get("content")) {
                            if let Some(text) = content.as_str() {
                                if let Ok(importance) = text.trim().parse::<f64>() {
                                    let importance = importance.clamp(0.0, 1.0);
                                    let old_importance = memory.importance;
                                    if self.update_memory_importance(&memory.id, importance, "llm").await.is_ok() {
                                        tracing::debug!("Updated importance: {} -> {:.2}", memory.content.chars().take(30).collect::<String>(), importance);
                                        updated += 1;
                                    }
                                }
                            }
                        }
                    }
                }
            }
            processed += 1;
        }

        tracing::info!("✅ LLM consolidation complete: processed={}, updated={}", processed, updated);

        Ok((processed, updated))
    }

    fn get_stability_for_category(category: &str) -> f64 {
        match category {
            "system" => 10000.0,
            "preference" => 720.0,
            "fact" => 168.0,
            "event" => 24.0,
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

        let memories = self.list_memories(5000).await?;
        let now = chrono::Utc::now();
        let mut updated = 0i64;

        for memory in memories {
            if memory.is_pinned {
                continue;
            }

            let created = chrono::DateTime::parse_from_rfc3339(&memory.created_at)
                .map(|dt| dt.with_timezone(&chrono::Utc))
                .unwrap_or(now);
            
            let hours_since_creation = (now - created).num_hours() as f64;
            let stability = Self::get_stability_for_category(&memory.memory_category);
            
            let base_decay = (-hours_since_creation / stability).exp();
            let access_boost = (memory.access_count as f64 * 0.02).min(0.5);

            let new_importance = (memory.importance * base_decay + access_boost).clamp(0.01, 1.0);

            if (new_importance - memory.importance).abs() > 0.01
                && self.update_memory_importance(&memory.id, new_importance, "decay").await.is_ok() {
                updated += 1;
            }
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
                "UPDATE memories SET deleted_at = ?1 WHERE deleted_at IS NULL AND importance < ?2 AND is_pinned = 0",
                params![now, effective_threshold],
            )?;
            Ok::<_, rusqlite::Error>(count as i64)
        }).await??)
    }
}
