//! Configuration management for Chetna
//!
//! Handles environment variables, embedding providers, and LLM configuration.

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

    // LLM for importance scoring
    pub llm_provider: String,
    pub llm_model: String,
    pub llm_api_key: Option<String>,
    pub llm_base_url: Option<String>,

    // Vector DB
    pub use_lancedb: bool,
    pub lancedb_path: Option<String>,

    // Cache
    pub session_cache_size: usize,

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

/// LLM model metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMModel {
    pub id: String,
    pub name: String,
    pub provider: String,
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

/// Returns list of available LLM models for auto-scoring
pub fn available_llm_models() -> Vec<LLMModel> {
    vec![
        // Ollama models
        LLMModel {
            id: "llama3.2".to_string(),
            name: "Llama 3.2".to_string(),
            provider: "ollama".to_string(),
            description: "Meta's latest Llama model".to_string(),
        },
        LLMModel {
            id: "gemma3".to_string(),
            name: "Gemma 3".to_string(),
            provider: "ollama".to_string(),
            description: "Google's efficient model".to_string(),
        },
        LLMModel {
            id: "qwen2.5".to_string(),
            name: "Qwen 2.5".to_string(),
            provider: "ollama".to_string(),
            description: "Alibaba's capable model".to_string(),
        },
        // OpenAI models
        LLMModel {
            id: "gpt-4o-mini".to_string(),
            name: "GPT-4o Mini".to_string(),
            provider: "openai".to_string(),
            description: "OpenAI's fast, cost-effective model".to_string(),
        },
    ]
}

impl Config {
    /// Load configuration from environment variables
    /// Looks for .env in ./ChetnaData/ first, then falls back to current directory
    pub fn from_env() -> anyhow::Result<Self> {
        // Try loading from ChetnaData/.env first
        let _ = dotenv::from_filename("ChetnaData/.env").ok();
        // Fall back to .env in current directory if not found
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

        // LLM configuration
        let llm_provider = env::var("LLM_PROVIDER").unwrap_or_else(|_| "ollama".to_string());
        let llm_model = env::var("LLM_MODEL").unwrap_or_else(|_| "llama3.2".to_string());
        let llm_api_key = env::var("LLM_API_KEY").ok();
        let llm_base_url = env::var("LLM_BASE_URL").ok();

        // Vector DB
        let use_lancedb = env::var("USE_LANCEDB")
            .unwrap_or_else(|_| "false".to_string())
            .parse()
            .unwrap_or(false);
        let lancedb_path = env::var("LANCEDB_PATH").ok();

        // Cache
        let session_cache_size: usize = env::var("SESSION_CACHE_SIZE")
            .unwrap_or_else(|_| "100".to_string())
            .parse()
            .unwrap_or(100);

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
        let db_path = PathBuf::from(&db_path);
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        Ok(Self {
            host,
            port,
            db_path: db_path.to_string_lossy().to_string(),
            log_level,
            embedding_provider,
            embedding_model,
            embedding_api_key,
            embedding_base_url,
            embedding_dimensions,
            llm_provider,
            llm_model,
            llm_api_key,
            llm_base_url,
            use_lancedb,
            lancedb_path,
            session_cache_size,
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

    /// Check if LLM is configured and available
    pub fn has_llm(&self) -> bool {
        match self.llm_provider.as_str() {
            "ollama" | "google" => true,
            "openai" | "openrouter" => self.llm_api_key.is_some(),
            _ => false,
        }
    }

    /// Get API key for authentication (returns None if not configured)
    pub fn get_api_key(&self) -> Option<&str> {
        self.embedding_api_key
            .as_deref()
            .or(self.llm_api_key.as_deref())
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
            llm_provider: "ollama".to_string(),
            llm_model: "llama3.2".to_string(),
            llm_api_key: None,
            llm_base_url: None,
            use_lancedb: false,
            lancedb_path: None,
            session_cache_size: 100,
            consolidation_interval_hours: 6,
            auto_decay_enabled: true,
            auto_flush_enabled: true,
            min_importance_threshold: 0.1,
            api_key: None,
        }
    }
}
