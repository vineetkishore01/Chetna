//! Configuration management for Chetna
//!
//! Handles environment variables and embedding providers.

use serde::{Deserialize, Serialize};
use std::env;
use std::path::PathBuf;

/// Main configuration structure for Chetna
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub host: String,
    pub port: u16,
    pub db_path: String,
    pub log_level: String,

    // Embedding settings
    pub embedding_provider: String,
    pub embedding_model: String,
    pub embedding_api_key: Option<String>,
    pub embedding_base_url: Option<String>,
    pub embedding_dimensions: Option<usize>,

    // Vector DB
    pub use_lancedb: bool,
    pub lancedb_path: Option<String>,

    // Cache

    // Consolidation & Memory Management
    pub consolidation_interval_hours: i32, // 0 = disabled
    pub auto_decay_enabled: bool,
    pub auto_flush_enabled: bool,
    pub min_importance_threshold: f64, // below this gets deleted

    // Authentication
    pub api_key: Option<String>, // API key for securing the service
}

/// Embedding model metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingModel {
    pub id: String,
    pub name: String,
    pub provider: String,
    pub dimensions: usize,
    pub description: String,
}

/// Returns list of available embedding models across all providers
pub fn available_embedding_models() -> Vec<EmbeddingModel> {
    vec![
        // Ollama models
        EmbeddingModel {
            id: "nomic-embed-text".to_string(),
            name: "Nomic Embed Text".to_string(),
            provider: "ollama".to_string(),
            dimensions: 768,
            description: "High-quality open source embeddings".to_string(),
        },
        EmbeddingModel {
            id: "mxbai-embed-large".to_string(),
            name: "MXBAI Embed Large".to_string(),
            provider: "ollama".to_string(),
            dimensions: 768,
            description: "Fast and efficient embeddings".to_string(),
        },
        EmbeddingModel {
            id: "gemma3-embed-e2b".to_string(),
            name: "Gemma 3 Embed E2B".to_string(),
            provider: "ollama".to_string(),
            dimensions: 256,
            description: "Google's efficient on-device embeddings".to_string(),
        },
        EmbeddingModel {
            id: "bge-m3".to_string(),
            name: "BGE M3".to_string(),
            provider: "ollama".to_string(),
            dimensions: 1024,
            description: "Multilingual BGE embeddings".to_string(),
        },
        // OpenAI models
        EmbeddingModel {
            id: "text-embedding-3-small".to_string(),
            name: "OpenAI Text Embedding 3 Small".to_string(),
            provider: "openai".to_string(),
            dimensions: 1536,
            description: "OpenAI's small embedding model".to_string(),
        },
        EmbeddingModel {
            id: "text-embedding-3-large".to_string(),
            name: "OpenAI Text Embedding 3 Large".to_string(),
            provider: "openai".to_string(),
            dimensions: 3072,
            description: "OpenAI's large embedding model".to_string(),
        },
        // Google models
        EmbeddingModel {
            id: "gemini-embedding-001".to_string(),
            name: "Google Gemini Embedding".to_string(),
            provider: "google".to_string(),
            dimensions: 768,
            description: "Google's Gemini text embeddings".to_string(),
        },
        EmbeddingModel {
            id: "gemini-embedding-2".to_string(),
            name: "Google Gemini Embedding 2".to_string(),
            provider: "google".to_string(),
            dimensions: 3072,
            description: "Google's latest multimodal embeddings".to_string(),
        },
    ]
}

impl Config {
    /// Load configuration from environment variables
    /// Looks for .env in ./ChetnaData/ first, then falls back to current directory
    pub fn from_env() -> anyhow::Result<Self> {
        // Load .env files - both attempts are fine, dotenv handles duplicates
        // First: ChetnaData/.env (preferred for data directory deployment)
        let _ = dotenv::from_filename("ChetnaData/.env").ok();
        // Second: .env in current directory (fallback)
        let _ = dotenv::dotenv().ok();

        let host = env::var("CHETNA_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
        let port: u16 = env::var("CHETNA_PORT")
            .unwrap_or_else(|_| "1987".to_string())
            .parse()?;
        let db_path =
            env::var("CHETNA_DB_PATH").unwrap_or_else(|_| "./ChetnaData/chetna.db".to_string());
        let log_level = env::var("CHETNA_LOG_LEVEL").unwrap_or_else(|_| "info".to_string());

        // Embedding configuration
        let embedding_provider =
            env::var("EMBEDDING_PROVIDER").unwrap_or_else(|_| "ollama".to_string());
        let embedding_model =
            env::var("EMBEDDING_MODEL").unwrap_or_else(|_| "nomic-embed-text".to_string());
        let embedding_api_key = env::var("EMBEDDING_API_KEY").ok();
        let embedding_base_url = env::var("EMBEDDING_BASE_URL").ok();
        let embedding_dimensions = env::var("EMBEDDING_DIMENSIONS")
            .ok()
            .and_then(|s| s.parse().ok());

        // Vector DB
        let use_lancedb = env::var("USE_LANCEDB")
            .unwrap_or_else(|_| "false".to_string())
            .parse()
            .unwrap_or(false);
        let lancedb_path = env::var("LANCEDB_PATH").ok();

        // Consolidation settings
        let consolidation_interval_hours: i32 = env::var("CONSOLIDATION_INTERVAL")
            .unwrap_or_else(|_| "6".to_string())
            .parse()
            .unwrap_or(6);
        let auto_decay_enabled = env::var("AUTO_DECAY_ENABLED")
            .unwrap_or_else(|_| "true".to_string())
            .parse()
            .unwrap_or(true);
        let auto_flush_enabled = env::var("AUTO_FLUSH_ENABLED")
            .unwrap_or_else(|_| "true".to_string())
            .parse()
            .unwrap_or(true);
        let min_importance_threshold: f64 = env::var("MIN_IMPORTANCE_THRESHOLD")
            .unwrap_or_else(|_| "0.1".to_string())
            .parse()
            .unwrap_or(0.1);

        // API key for authentication
        let api_key = env::var("CHETNA_API_KEY").ok();

        // Ensure data directory exists
        let db_path_buf = PathBuf::from(&db_path);
        if let Some(parent) = db_path_buf.parent() {
            std::fs::create_dir_all(parent)?;
        }

        Ok(Self {
            host,
            port,
            db_path,
            log_level,
            embedding_provider,
            embedding_model,
            embedding_api_key,
            embedding_base_url,
            embedding_dimensions,
            use_lancedb,
            lancedb_path,
            consolidation_interval_hours,
            auto_decay_enabled,
            auto_flush_enabled,
            min_importance_threshold,
            api_key,
        })
    }

    /// Check if embedding is configured and available
    pub fn has_embedding(&self) -> bool {
        match self.embedding_provider.as_str() {
            "ollama" => true,
            "openai" | "openrouter" | "google" => self.embedding_api_key.is_some(),
            _ => false,
        }
    }

    /// Get embedding provider API key
    pub fn get_embedding_api_key(&self) -> Option<&str> {
        self.embedding_api_key.as_deref()
    }

    /// Get user authentication API key (for authenticating to Chetna itself)
    pub fn get_user_auth_api_key(&self) -> Option<&str> {
        self.api_key.as_deref()
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 1987,
            db_path: "./data/chetna.db".to_string(),
            log_level: "info".to_string(),
            embedding_provider: "ollama".to_string(),
            embedding_model: "nomic-embed-text".to_string(),
            embedding_api_key: None,
            embedding_base_url: None,
            embedding_dimensions: None,
            use_lancedb: false,
            lancedb_path: None,
            consolidation_interval_hours: 6,
            auto_decay_enabled: true,
            auto_flush_enabled: true,
            min_importance_threshold: 0.1,
            api_key: None,
        }
    }
}
