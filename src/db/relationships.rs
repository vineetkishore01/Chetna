//! Memory relationships - Track connections between memories
//!
//! This enables:
//! - Finding related memories
//! - Building memory graphs
//! - Understanding context

use anyhow::Result;
use rusqlite::params;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;
use chrono::Utc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryRelationship {
    pub id: String,
    pub source_id: String,
    pub target_id: String,
    pub relationship_type: RelationshipType,
    pub strength: f64,
    pub created_at: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum RelationshipType {
    Related,
    Contradicts,
    Supports,
    Extends,
    Similar,
    Cause,
    Effect,
    PartOf,
    Before,
    After,
}

impl RelationshipType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Related => "related",
            Self::Contradicts => "contradicts",
            Self::Supports => "supports",
            Self::Extends => "extends",
            Self::Similar => "similar",
            Self::Cause => "cause",
            Self::Effect => "effect",
            Self::PartOf => "part_of",
            Self::Before => "before",
            Self::After => "after",
        }
    }

    pub fn from_str_value(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "contradicts" => Self::Contradicts,
            "supports" => Self::Supports,
            "extends" => Self::Extends,
            "similar" => Self::Similar,
            "cause" => Self::Cause,
            "effect" => Self::Effect,
            "part_of" | "partof" => Self::PartOf,
            "before" => Self::Before,
            "after" => Self::After,
            _ => Self::Related,
        }
    }
}

pub struct RelationshipManager {
    conn: Arc<Mutex<rusqlite::Connection>>,
}

impl RelationshipManager {
    pub fn new(conn: Arc<Mutex<rusqlite::Connection>>) -> Self {
        Self { conn }
    }

    /// Create a relationship between two memories
    pub async fn create_relationship(
        &self,
        source_id: &str,
        target_id: &str,
        relationship_type: RelationshipType,
        strength: f64,
    ) -> Result<MemoryRelationship> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();

        let conn = self.conn.lock().await;
        conn.execute(
            "INSERT INTO memory_relationships (id, source_id, target_id, relationship_type, strength, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![id, source_id, target_id, relationship_type.as_str(), strength, now],
        )?;

        Ok(MemoryRelationship {
            id,
            source_id: source_id.to_string(),
            target_id: target_id.to_string(),
            relationship_type,
            strength,
            created_at: now,
        })
    }

    /// Get all relationships for a memory
    pub async fn get_relationships(&self, memory_id: &str) -> Result<Vec<MemoryRelationship>> {
        let conn = self.conn.lock().await;
        let mut stmt = conn.prepare(
            "SELECT id, source_id, target_id, relationship_type, strength, created_at FROM memory_relationships WHERE source_id = ?1 OR target_id = ?1"
        )?;

        let rows = stmt.query_map(params![memory_id], |row| {
            let rel_type: String = row.get(3)?;
            Ok(MemoryRelationship {
                id: row.get(0)?,
                source_id: row.get(1)?,
                target_id: row.get(2)?,
                relationship_type: RelationshipType::from_str_value(&rel_type),
                strength: row.get(4)?,
                created_at: row.get(5)?,
            })
        })?;

        let mut relationships = Vec::new();
        for row in rows {
            relationships.push(row?);
        }

        Ok(relationships)
    }

    /// Get related memories (just the IDs)
    pub async fn get_related_ids(&self, memory_id: &str) -> Result<Vec<(String, RelationshipType, f64)>> {
        let conn = self.conn.lock().await;
        let mut stmt = conn.prepare(
            "SELECT CASE WHEN source_id = ?1 THEN target_id ELSE source_id END as related_id, relationship_type, strength FROM memory_relationships WHERE source_id = ?1 OR target_id = ?1"
        )?;

        let rows = stmt.query_map(params![memory_id], |row| {
            Ok((
                row.get::<_, String>(0)?,
                RelationshipType::from_str_value(&row.get::<_, String>(1)?),
                row.get(2)?,
            ))
        })?;

        let mut related = Vec::new();
        for row in rows {
            related.push(row?);
        }

        Ok(related)
    }

    /// Delete a relationship
    pub async fn delete_relationship(&self, id: &str) -> Result<()> {
        let conn = self.conn.lock().await;
        conn.execute(
            "DELETE FROM memory_relationships WHERE id = ?1",
            params![id],
        )?;
        Ok(())
    }

    /// Auto-create relationships based on semantic similarity
    /// Uses a single transaction to batch all relationship creations for better performance
    pub async fn auto_link_memories(&self, memories: &[(String, Vec<f32>)], threshold: f64) -> Result<i64> {
        let mut relationships_to_create = Vec::new();

        // First pass: calculate all similarities (no locking)
        for i in 0..memories.len() {
            for j in (i + 1)..memories.len() {
                let similarity = cosine_similarity(&memories[i].1, &memories[j].1);

                if similarity >= threshold {
                    let rel_type = if similarity > 0.9 {
                        RelationshipType::Similar
                    } else {
                        RelationshipType::Related
                    };

                    relationships_to_create.push((
                        memories[i].0.clone(),
                        memories[j].0.clone(),
                        rel_type,
                        similarity,
                    ));
                }
            }
        }

        // Second pass: batch insert all relationships in a single transaction (single lock)
        if relationships_to_create.is_empty() {
            return Ok(0);
        }

        let conn = self.conn.lock().await;

        // Begin transaction
        conn.execute("BEGIN TRANSACTION", [])?;

        let mut count = 0;
        for (source_id, target_id, rel_type, strength) in relationships_to_create {
            let id = Uuid::new_v4().to_string();
            let now = Utc::now().to_rfc3339();

            conn.execute(
                "INSERT INTO memory_relationships (id, source_id, target_id, relationship_type, strength, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![id, source_id, target_id, rel_type.as_str(), strength, now],
            )?;
            count += 1;
        }

        // Commit transaction
        conn.execute("COMMIT", [])?;

        Ok(count)
    }

    /// Get relationship statistics
    pub async fn get_stats(&self) -> Result<RelationshipStats> {
        let conn = self.conn.lock().await;
        let total: i64 = conn.query_row(
            "SELECT COUNT(*) FROM memory_relationships",
            [],
            |row| row.get(0),
        )?;

        let mut stmt = conn.prepare(
            "SELECT relationship_type, COUNT(*) FROM memory_relationships GROUP BY relationship_type"
        )?;

        let rows = stmt.query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
        })?;

        let mut by_type = std::collections::HashMap::new();
        for row in rows.flatten() {
            by_type.insert(row.0, row.1);
        }

        Ok(RelationshipStats {
            total_relationships: total,
            by_type,
        })
    }
}

fn cosine_similarity(a: &[f32], b: &[f32]) -> f64 {
    if a.len() != b.len() || a.is_empty() {
        return 0.0;
    }

    let dot: f64 = a.iter().zip(b.iter()).map(|(x, y)| (*x as f64) * (*y as f64)).sum();
    let mag_a: f64 = a.iter().map(|x| (*x as f64).powi(2)).sum::<f64>().sqrt();
    let mag_b: f64 = b.iter().map(|x| (*x as f64).powi(2)).sum::<f64>().sqrt();

    if mag_a == 0.0 || mag_b == 0.0 {
        return 0.0;
    }

    dot / (mag_a * mag_b)
}

#[derive(Debug, Serialize)]
pub struct RelationshipStats {
    pub total_relationships: i64,
    pub by_type: std::collections::HashMap<String, i64>,
}
