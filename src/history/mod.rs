//! History logging module for tracking memory operations
//!
//! Logs memory creation, queries, and context building events with detailed scoring information.
//! Uses async background logging with bounded queue for minimal performance impact.

use anyhow::Result;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::mpsc;
use uuid::Uuid;

/// Event types for history logging
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum EventType {
    MemoryCreated,
    QuerySearched,
    ContextBuilt,
}

/// History event with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEvent {
    pub id: String,
    pub event_type: EventType,
    pub timestamp: String,
    pub namespace: String,
    pub session_id: Option<String>,
    pub metadata: serde_json::Value,
}

/// Query result with scores
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryResult {
    pub memory_id: String,
    pub rank: i32,
    pub similarity_score: Option<f32>,
    pub recall_score: Option<f32>,
}

/// Filters for history queries
#[derive(Debug, Clone, Default)]
pub struct HistoryFilters {
    pub event_type: Option<EventType>,
    pub namespace: Option<String>,
    pub session_id: Option<String>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

/// Pagination parameters
#[derive(Debug, Clone, Default)]
pub struct Pagination {
    pub limit: i64,
    pub offset: i64,
}

/// Analytics data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Analytics {
    pub total_events: i64,
    pub events_by_type: HashMap<String, i64>,
    pub most_common_queries: Vec<QueryStats>,
    pub most_accessed_memories: Vec<MemoryStats>,
    pub average_query_duration_ms: f64,
    pub query_success_rate: f64,
    pub time_range: TimeRange,
}

/// Query statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryStats {
    pub query: String,
    pub count: i64,
    pub average_similarity: f64,
}

/// Memory statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryStats {
    pub memory_id: String,
    pub content: String,
    pub access_count: i64,
    pub last_accessed: String,
}

/// Time range for analytics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeRange {
    pub start: String,
    pub end: String,
}

/// Event details with full information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventDetails {
    pub event: HistoryEvent,
    pub query_results: Option<Vec<QueryResult>>,
}

/// History logger with async background processing
pub struct HistoryLogger {
    conn: Arc<tokio::sync::Mutex<rusqlite::Connection>>,
    event_sender: mpsc::Sender<HistoryEvent>,
    queue_size: usize,
}

impl HistoryLogger {
    /// Create a new history logger with async background processing
    pub fn new(conn: Arc<tokio::sync::Mutex<rusqlite::Connection>>, queue_size: usize) -> Result<Self> {
        let (event_sender, mut event_receiver) = mpsc::channel::<HistoryEvent>(queue_size);

        let conn_clone = conn.clone();
        tokio::spawn(async move {
            while let Some(event) = event_receiver.recv().await {
                if let Err(e) = Self::persist_event(&conn_clone, event).await {
                    tracing::error!("Failed to persist history event: {}", e);
                }
            }
        });

        tracing::info!("History logger initialized with queue_size={}", queue_size);

        Ok(Self {
            conn,
            event_sender,
            queue_size,
        })
    }

    /// Log a history event (async, non-blocking)
    pub fn log_event(&self, event: HistoryEvent) -> Result<()> {
        if let Err(e) = self.event_sender.try_send(event) {
            tracing::warn!("Failed to queue history event: {}", e);
            return Err(anyhow::anyhow!("Failed to queue history event: {}", e));
        }
        Ok(())
    }

    /// Log query results for a search event
    pub async fn log_query_results(&self, event_id: &str, results: &[QueryResult]) -> Result<()> {
        let conn = self.conn.lock().await;
        
        for result in results {
            let id = Uuid::new_v4().to_string();
            conn.execute(
                "INSERT INTO query_results (id, event_id, memory_id, rank, similarity_score, recall_score) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                rusqlite::params![
                    id,
                    event_id,
                    result.memory_id,
                    result.rank,
                    result.similarity_score,
                    result.recall_score
                ],
            )?;
        }

        Ok(())
    }

    /// Get history events with filters
    pub async fn get_history(&self, filters: HistoryFilters) -> Result<Vec<HistoryEvent>> {
        let conn = self.conn.lock().await;

        let mut query = String::from(
            "SELECT id, event_type, timestamp, namespace, session_id, metadata FROM history_events WHERE 1=1"
        );
        let mut params = Vec::new();

        if let Some(event_type) = &filters.event_type {
            query.push_str(" AND event_type = ?");
            params.push(serde_json::to_string(event_type)?);
        }

        if let Some(namespace) = &filters.namespace {
            query.push_str(" AND namespace = ?");
            params.push(namespace.clone());
        }

        if let Some(session_id) = &filters.session_id {
            query.push_str(" AND session_id = ?");
            params.push(session_id.clone());
        }

        if let Some(start_date) = &filters.start_date {
            query.push_str(" AND timestamp >= ?");
            params.push(start_date.clone());
        }

        if let Some(end_date) = &filters.end_date {
            query.push_str(" AND timestamp <= ?");
            params.push(end_date.clone());
        }

        query.push_str(" ORDER BY timestamp DESC");

        let limit = filters.limit.unwrap_or(50);
        let offset = filters.offset.unwrap_or(0);
        query.push_str(&format!(" LIMIT {} OFFSET {}", limit, offset));

        let mut stmt = conn.prepare(&query)?;
        let mut events = Vec::new();

        let param_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p as &dyn rusqlite::ToSql).collect();

        let rows = stmt.query_map(param_refs.as_slice(), |row| {
            let event_type_str: String = row.get(1)?;
            let metadata_str: String = row.get(5)?;
            Ok(HistoryEvent {
                id: row.get(0)?,
                event_type: serde_json::from_str(&event_type_str).map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?,
                timestamp: row.get(2)?,
                namespace: row.get(3)?,
                session_id: row.get(4)?,
                metadata: serde_json::from_str(&metadata_str).map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?,
            })
        })?;

        for row in rows {
            events.push(row?);
        }

        Ok(events)
    }

    /// Get event details with query results
    pub async fn get_event_details(&self, event_id: &str) -> Result<EventDetails> {
        let conn = self.conn.lock().await;

        let event = conn.query_row(
            "SELECT id, event_type, timestamp, namespace, session_id, metadata FROM history_events WHERE id = ?1",
            [event_id],
            |row| {
                let event_type_str: String = row.get(1)?;
                let metadata_str: String = row.get(5)?;
                Ok(HistoryEvent {
                    id: row.get(0)?,
                    event_type: serde_json::from_str(&event_type_str).map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?,
                    timestamp: row.get(2)?,
                    namespace: row.get(3)?,
                    session_id: row.get(4)?,
                    metadata: serde_json::from_str(&metadata_str).map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?,
                })
            },
        )?;

        let query_results = if event.event_type == EventType::QuerySearched {
            let mut stmt = conn.prepare(
                "SELECT memory_id, rank, similarity_score, recall_score FROM query_results WHERE event_id = ?1 ORDER BY rank"
            )?;
            let rows = stmt.query_map([event_id], |row| {
                Ok(QueryResult {
                    memory_id: row.get(0)?,
                    rank: row.get(1)?,
                    similarity_score: row.get(2)?,
                    recall_score: row.get(3)?,
                })
            })?;
            let mut results = Vec::new();
            for row in rows {
                results.push(row?);
            }
            Some(results)
        } else {
            None
        };

        Ok(EventDetails {
            event,
            query_results,
        })
    }

    /// Get analytics for a time range
    pub async fn get_analytics(&self, time_range: TimeRange) -> Result<Analytics> {
        let conn = self.conn.lock().await;

        // Total events
        let total_events: i64 = conn.query_row(
            "SELECT COUNT(*) FROM history_events WHERE timestamp >= ?1 AND timestamp <= ?2",
            [&time_range.start, &time_range.end],
            |row| row.get(0),
        )?;

        // Events by type
        let mut events_by_type = HashMap::new();
        let mut stmt = conn.prepare(
            "SELECT event_type, COUNT(*) FROM history_events WHERE timestamp >= ?1 AND timestamp <= ?2 GROUP BY event_type"
        )?;
        let rows = stmt.query_map([&time_range.start, &time_range.end], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
        })?;
        for row in rows {
            let (event_type, count) = row?;
            events_by_type.insert(event_type, count);
        }

        // Most common queries
        let mut most_common_queries = Vec::new();
        let mut stmt = conn.prepare(
            "SELECT metadata, COUNT(*) as count FROM history_events WHERE event_type = 'query_searched' AND timestamp >= ?1 AND timestamp <= ?2 GROUP BY metadata ORDER BY count DESC LIMIT 10"
        )?;
        let rows = stmt.query_map([&time_range.start, &time_range.end], |row| {
            let metadata_str: String = row.get(0)?;
            let metadata: serde_json::Value = serde_json::from_str(&metadata_str).map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;
            let count: i64 = row.get(1)?;
            let query = metadata.get("query").and_then(|q| q.as_str()).unwrap_or("").to_string();
            Ok((query, count))
        })?;
        for row in rows {
            let (query, count) = row?;
            most_common_queries.push(QueryStats {
                query,
                count,
                average_similarity: 0.0, // TODO: Calculate from query_results
            });
        }

        // Most accessed memories
        let mut most_accessed_memories = Vec::new();
        let mut stmt = conn.prepare(
            "SELECT qr.memory_id, m.content, COUNT(*) as access_count, MAX(h.timestamp) as last_accessed FROM query_results qr JOIN history_events h ON qr.event_id = h.id JOIN memories m ON qr.memory_id = m.id WHERE h.timestamp >= ?1 AND h.timestamp <= ?2 GROUP BY qr.memory_id ORDER BY access_count DESC LIMIT 10"
        )?;
        let rows = stmt.query_map([&time_range.start, &time_range.end], |row| {
            Ok(MemoryStats {
                memory_id: row.get(0)?,
                content: row.get(1)?,
                access_count: row.get(2)?,
                last_accessed: row.get(3)?,
            })
        })?;
        for row in rows {
            most_accessed_memories.push(row?);
        }

        // Average query duration
        let average_query_duration_ms: f64 = conn.query_row(
            "SELECT AVG(CAST(json_extract(metadata, '$.duration_ms') AS REAL)) FROM history_events WHERE event_type = 'query_searched' AND timestamp >= ?1 AND timestamp <= ?2",
            [&time_range.start, &time_range.end],
            |row| row.get::<_, Option<f64>>(0).map(|v| v.unwrap_or(0.0)),
        ).unwrap_or(0.0);

        // Query success rate (queries that returned results)
        let total_queries: i64 = conn.query_row(
            "SELECT COUNT(*) FROM history_events WHERE event_type = 'query_searched' AND timestamp >= ?1 AND timestamp <= ?2",
            [&time_range.start, &time_range.end],
            |row| row.get(0),
        )?;
        let successful_queries: i64 = conn.query_row(
            "SELECT COUNT(DISTINCT h.id) FROM history_events h JOIN query_results qr ON h.id = qr.event_id WHERE h.event_type = 'query_searched' AND h.timestamp >= ?1 AND h.timestamp <= ?2",
            [&time_range.start, &time_range.end],
            |row| row.get(0),
        )?;
        let query_success_rate = if total_queries > 0 {
            successful_queries as f64 / total_queries as f64
        } else {
            0.0
        };

        Ok(Analytics {
            total_events,
            events_by_type,
            most_common_queries,
            most_accessed_memories,
            average_query_duration_ms,
            query_success_rate,
            time_range,
        })
    }

    /// Cleanup old events (30-day retention)
    pub async fn cleanup_old_events(&self, days: i32) -> Result<i64> {
        let cutoff_timestamp = Utc::now().timestamp() - (days as i64 * 86400);
        let cutoff_date = chrono::DateTime::from_timestamp(cutoff_timestamp, 0)
            .unwrap()
            .format("%Y-%m-%d %H:%M:%S")
            .to_string();

        let conn = self.conn.lock().await;

        // Delete query results first (foreign key)
        let deleted_results = conn.execute(
            "DELETE FROM query_results WHERE event_id IN (SELECT id FROM history_events WHERE timestamp < ?1)",
            [&cutoff_date],
        )?;

        // Delete events
        let deleted_events = conn.execute(
            "DELETE FROM history_events WHERE timestamp < ?1",
            [&cutoff_date],
        )?;

        tracing::info!("Cleaned up {} events and {} query results older than {} days", deleted_events, deleted_results, days);

        Ok(deleted_events as i64)
    }

    /// Persist an event to the database
    async fn persist_event(conn: &Arc<tokio::sync::Mutex<rusqlite::Connection>>, event: HistoryEvent) -> Result<()> {
        let conn = conn.lock().await;
        
        conn.execute(
            "INSERT INTO history_events (id, event_type, timestamp, namespace, session_id, metadata, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            rusqlite::params![
                event.id,
                serde_json::to_string(&event.event_type)?,
                event.timestamp,
                event.namespace,
                event.session_id,
                serde_json::to_string(&event.metadata)?,
                Utc::now().to_rfc3339()
            ],
        )?;

        Ok(())
    }

    /// Get queue statistics
    pub fn queue_stats(&self) -> QueueStats {
        QueueStats {
            queue_size: self.queue_size,
            current_size: self.queue_size, // We can't get the actual current size from mpsc::Sender
        }
    }
}

/// Queue statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueStats {
    pub queue_size: usize,
    pub current_size: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_type_serialization() {
        let event_type = EventType::QuerySearched;
        let serialized = serde_json::to_string(&event_type).unwrap();
        assert_eq!(serialized, "\"query_searched\"");

        let deserialized: EventType = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized, EventType::QuerySearched);
    }
}