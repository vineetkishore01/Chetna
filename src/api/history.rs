//! History API endpoints for tracking memory operations

use axum::{
    extract::{Path, Query, State},
    routing::get,
    Json, Router,
};
use chrono::{Duration, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use crate::Brain;
use crate::history::{HistoryFilters, TimeRange, Analytics};

/// History list response
#[derive(Debug, Serialize)]
pub struct HistoryListResponse {
    pub events: Vec<crate::history::HistoryEvent>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}

/// History query parameters
#[derive(Debug, Deserialize)]
pub struct HistoryQueryParams {
    pub event_type: Option<String>,
    pub namespace: Option<String>,
    pub session_id: Option<String>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

/// Analytics query parameters
#[derive(Debug, Deserialize)]
pub struct AnalyticsQueryParams {
    pub days: Option<i64>,
}

pub fn router() -> Router<(Arc<Brain>, Arc<tokio::sync::RwLock<crate::config_file::UserConfig>>)> {
    Router::new()
        .route("/", get(list_history))
        .route("/:id", get(get_event_details))
        .route("/analytics", get(get_analytics))
        .route("/cleanup", get(cleanup_history))
}

/// List history events
async fn list_history(
    State((brain, _user_config)): State<(Arc<Brain>, Arc<tokio::sync::RwLock<crate::config_file::UserConfig>>)>,
    Query(params): Query<HistoryQueryParams>,
) -> Result<Json<HistoryListResponse>, axum::http::StatusCode> {
    let event_type = params.event_type.and_then(|t| {
        match t.as_str() {
            "memory_created" => Some(crate::history::EventType::MemoryCreated),
            "query_searched" => Some(crate::history::EventType::QuerySearched),
            "context_built" => Some(crate::history::EventType::ContextBuilt),
            _ => None,
        }
    });

    let filters = HistoryFilters {
        event_type,
        namespace: params.namespace,
        session_id: params.session_id,
        start_date: params.start_date,
        end_date: params.end_date,
        limit: params.limit,
        offset: params.offset,
    };

    match brain.get_history(filters).await {
        Ok(events) => {
            let total = events.len() as i64;
            Ok(Json(HistoryListResponse {
                events,
                total,
                limit: params.limit.unwrap_or(50),
                offset: params.offset.unwrap_or(0),
            }))
        }
        Err(e) => {
            tracing::error!("Failed to get history: {}", e);
            Err(axum::http::StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Get event details
async fn get_event_details(
    State((brain, _user_config)): State<(Arc<Brain>, Arc<tokio::sync::RwLock<crate::config_file::UserConfig>>)>,
    Path(id): Path<String>,
) -> Result<Json<crate::history::EventDetails>, axum::http::StatusCode> {
    match brain.get_event_details(&id).await {
        Ok(details) => Ok(Json(details)),
        Err(e) => {
            tracing::error!("Failed to get event details: {}", e);
            Err(axum::http::StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Get analytics
async fn get_analytics(
    State((brain, _user_config)): State<(Arc<Brain>, Arc<tokio::sync::RwLock<crate::config_file::UserConfig>>)>,
    Query(params): Query<AnalyticsQueryParams>,
) -> Result<Json<Analytics>, axum::http::StatusCode> {
    let days = params.days.unwrap_or(30);
    let end_date = Utc::now();
    let start_date = end_date - Duration::days(days);

    let time_range = TimeRange {
        start: start_date.format("%Y-%m-%d %H:%M:%S").to_string(),
        end: end_date.format("%Y-%m-%d %H:%M:%S").to_string(),
    };

    match brain.get_analytics(time_range).await {
        Ok(analytics) => Ok(Json(analytics)),
        Err(e) => {
            tracing::error!("Failed to get analytics: {}", e);
            Err(axum::http::StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Cleanup old history events
async fn cleanup_history(
    State((brain, _user_config)): State<(Arc<Brain>, Arc<tokio::sync::RwLock<crate::config_file::UserConfig>>)>,
    Query(params): Query<AnalyticsQueryParams>,
) -> Result<Json<serde_json::Value>, axum::http::StatusCode> {
    let days = params.days.unwrap_or(30) as i32;

    match brain.cleanup_old_history(days).await {
        Ok(count) => Ok(Json(serde_json::json!({
            "success": true,
            "deleted_events": count,
            "message": format!("Cleaned up {} events older than {} days", count, days)
        }))),
        Err(e) => {
            tracing::error!("Failed to cleanup history: {}", e);
            Err(axum::http::StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}