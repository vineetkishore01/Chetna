//! Chetna - God-Tier Memory System
//!
//! A hyper-fast, standalone memory system written in Rust that combines
//! the best of Wolverine's intelligent memory management and Engram's
//! battle-tested architecture.

pub mod db;
pub mod api;
pub mod mcp;
pub mod web;
pub mod config;
pub mod config_file;
pub mod scheduler;
pub mod shared;

pub use db::brain::{Brain, RecallWeights};
pub use config::Config;
pub use config_file::UserConfig;

use anyhow::Result;
use axum::Router;
use std::sync::Arc;
use std::time::Instant;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

pub static START_TIME: std::sync::OnceLock<Instant> = std::sync::OnceLock::new();

pub struct AppState {
    pub brain: Arc<Brain>,
    pub config: Config,
    pub user_config: Arc<tokio::sync::RwLock<UserConfig>>,
}

impl AppState {
    pub fn new(config: Config) -> Result<Self> {
        // Load user config from file with proper error handling
        let user_config = match UserConfig::load() {
            Ok(cfg) => {
                info!("✅ User config loaded from {}", UserConfig::config_path().display());
                cfg
            }
            Err(e) => {
                tracing::warn!(
                    "Failed to load user config (using defaults): {}",
                    e
                );
                UserConfig::default()
            }
        };

        let embedder_config = if config.has_embedding() || user_config.embedding_provider.is_some() {
            Some(db::embedding::EmbedderConfig {
                provider: user_config.embedding_provider.clone().unwrap_or(config.embedding_provider.clone()),
                model: user_config.embedding_model.clone().unwrap_or(config.embedding_model.clone()),
                api_key: user_config.api_key.clone().or(config.embedding_api_key.clone()),
                base_url: user_config.embedding_base_url.clone().or(config.embedding_base_url.clone()),
            })
        } else {
            None
        };

        let brain = Brain::new_with_embedder(&config.db_path, embedder_config)?;

        Ok(Self {
            brain: Arc::new(brain),
            config: config.clone(),
            user_config: Arc::new(tokio::sync::RwLock::new(user_config)),
        })
    }
}

pub fn create_router(state: AppState) -> Router {
    let brain = state.brain.clone();
    let user_config = state.user_config.clone();
    api::create_router(brain.clone(), user_config.clone())
        .merge(web::routes())
        .with_state((brain, user_config))
}

pub fn init_logging() {
    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| "chetna=info,warn".into());

    tracing_subscriber::registry()
        .with(filter)
        .with(tracing_subscriber::fmt::layer())
        .init();
}
