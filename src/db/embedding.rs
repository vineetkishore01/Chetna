//! Embedding module - Multi-provider embeddings for semantic memory
//! 
//! Supports: Ollama (local), OpenAI, OpenRouter
//! Also includes embedding cache to avoid re-embedding same content

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;
use rusqlite::Connection;
use sha2::{Sha256, Digest};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Embedding {
    pub vector: Vec<f32>,
    pub model: String,
    pub dimensions: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingRequest {
    pub model: String,
    pub input: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OllamaResponse {
    #[serde(alias = "embeddings")]
    pub embedding: Vec<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIResponse {
    pub data: Vec<OpenAIEmbedding>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIEmbedding {
    pub embedding: Vec<f32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Default)]
pub enum EmbeddingProvider {
    #[default]
    Ollama,
    OpenAI,
    OpenRouter,
    Google,
}

pub struct Embedder {
    provider: EmbeddingProvider,
    pub model: String,
    api_key: Option<String>,
    base_url: String,
    conn: Arc<Mutex<Connection>>,
    dimensions: usize,
    client: reqwest::Client,
}

impl Clone for Embedder {
    fn clone(&self) -> Self {
        Self {
            provider: self.provider,
            model: self.model.clone(),
            api_key: self.api_key.clone(),
            base_url: self.base_url.clone(),
            conn: self.conn.clone(),
            dimensions: self.dimensions,
            client: self.client.clone(),
        }
    }
}

impl Embedder {
    pub fn new(
        provider: EmbeddingProvider,
        model: String,
        api_key: Option<String>,
        base_url: String,
        conn: Arc<Mutex<Connection>>,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let dimensions = match model.as_str() {
            "nomic-embed-text" | "mxbai-embed-large" => 768,
            "text-embedding-3-small" => 1536,
            "text-embedding-3-large" => 3072,
            "text-embedding-ada-002" => 1536,
            "qwen3-embedding:4b" => 2560,
            _ => 768,
        };

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .connect_timeout(std::time::Duration::from_secs(10))
            .build()?;

        Ok(Self {
            provider,
            model,
            api_key,
            base_url,
            conn,
            dimensions,
            client,
        })
    }

    pub fn dimensions(&self) -> usize {
        self.dimensions
    }

    /// Create embedding for text with caching
    pub async fn embed(&self, text: &str) -> Result<Embedding> {
        let hash = self.hash_content(text);

        // Check cache first
        if let Some(cached) = self.get_cached(&hash).await? {
            tracing::debug!("Embedding cache hit for: {}", text.chars().take(50).collect::<String>());
            return Ok(cached);
        }

        tracing::debug!("Creating embedding for: {}", text.chars().take(50).collect::<String>());

        // Generate new embedding with clear error handling
        let vector = match self.provider {
            EmbeddingProvider::Ollama => {
                tracing::debug!("Using Ollama embedding model: {}", self.model);
                match self.embed_ollama(text).await {
                    Ok(v) => v,
                    Err(e) => {
                        let error_msg = if e.to_string().contains("connection") {
                            "Cannot connect to Ollama. Ensure Ollama is running at the configured URL.".to_string()
                        } else if e.to_string().contains("timeout") {
                            "Ollama request timed out. Check if the server is overloaded.".to_string()
                        } else {
                            format!("Ollama error: {}. Check /api/status for connection details.", e)
                        };
                        return Err(anyhow!(error_msg));
                    }
                }
            },
            EmbeddingProvider::OpenAI | EmbeddingProvider::OpenRouter => {
                tracing::debug!("Using OpenAI embedding model: {}", self.model);
                match self.embed_openai(text).await {
                    Ok(v) => v,
                    Err(e) => {
                        return Err(anyhow!("OpenAI error: {}. Check your API key and quota.", e));
                    }
                }
            }
            EmbeddingProvider::Google => {
                tracing::debug!("Using Google embedding model: {}", self.model);
                match self.embed_google(text).await {
                    Ok(v) => v,
                    Err(e) => {
                        return Err(anyhow!("Google embedding error: {}. Check your API key.", e));
                    }
                }
            },
        };

        // Cache the result
        self.cache(&hash, text, &vector).await?;

        Ok(Embedding {
            vector,
            model: self.model.clone(),
            dimensions: self.dimensions,
        })
    }

    /// Create embeddings for multiple texts
    pub async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Embedding>> {
        use futures::stream::{self, StreamExt};
        
        let results = stream::iter(texts.iter())
            .map(|text| self.embed(text))
            .buffer_unordered(10) // Process up to 10 in parallel
            .collect::<Vec<_>>()
            .await;
        
        Ok(results.into_iter().filter_map(|r| r.ok()).collect())
    }

    async fn embed_ollama(&self, text: &str) -> Result<Vec<f32>> {
        let response = self.client
            .post(format!("{}/api/embeddings", self.base_url))
            .json(&serde_json::json!({
                "model": self.model,
                "prompt": text
            }))
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(anyhow!("Ollama error {}: {}", status, body));
        }

        let result: OllamaResponse = response.json().await?;
        Ok(result.embedding)
    }

    async fn embed_openai(&self, text: &str) -> Result<Vec<f32>> {
        let (url, auth_header) = match self.provider {
            EmbeddingProvider::OpenAI => (
                format!("{}/v1/embeddings", self.base_url.trim_end_matches('/')),
                format!("Bearer {}", self.api_key.as_ref().ok_or(anyhow!("OpenAI API key required"))?),
            ),
            EmbeddingProvider::OpenRouter => (
                format!("{}/api/v1/embeddings", self.base_url.trim_end_matches('/')),
                format!("Bearer {}", self.api_key.as_ref().ok_or(anyhow!("OpenRouter API key required"))?),
            ),
            _ => return Err(anyhow!("Invalid provider for OpenAI")),
        };

        let response = self.client
            .post(url)
            .header("Authorization", auth_header)
            .header("Content-Type", "application/json")
            .json(&serde_json::json!({
                "model": self.model,
                "input": text
            }))
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(anyhow!("OpenAI error {}: {}", status, body));
        }

        let result: OpenAIResponse = response.json().await?;
        
        result
            .data
            .first()
            .map(|e| e.embedding.clone())
            .ok_or_else(|| anyhow!("No embedding returned"))
    }

    async fn embed_google(&self, text: &str) -> Result<Vec<f32>> {
        let api_key = self.api_key.as_ref().ok_or(anyhow!("Google API key required"))?;
        
        // Google Gemini Embedding API
        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:embedContent?key={}",
            self.model, api_key
        );

        #[derive(Serialize)]
        struct GoogleRequest {
            content: Content,
        }
        
        #[derive(Serialize)]
        struct Content {
            parts: Vec<Part>,
        }
        
        #[derive(Serialize)]
        struct Part {
            text: String,
        }

        #[derive(Deserialize)]
        struct GoogleResponse {
            embedding: EmbeddingValue,
        }
        
        #[derive(Deserialize)]
        struct EmbeddingValue {
            values: Vec<f32>,
        }

        let request = GoogleRequest {
            content: Content {
                parts: vec![Part { text: text.to_string() }],
            },
        };

        let response = self.client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(anyhow!("Google error {}: {}", status, body));
        }

        let result: GoogleResponse = response.json().await?;
        Ok(result.embedding.values)
    }

    /// Check if the embedding provider is reachable and responsive
    pub async fn check_connection(&self) -> Result<bool> {
        match self.provider {
            EmbeddingProvider::Ollama => {
                let url = format!("{}/api/tags", self.base_url);
                let resp = self.client.get(url)
                    .timeout(std::time::Duration::from_secs(3))
                    .send()
                    .await;
                match resp {
                    Ok(response) => Ok(response.status().is_success()),
                    Err(_) => Ok(false),
                }
            },
            EmbeddingProvider::OpenAI | EmbeddingProvider::OpenRouter | EmbeddingProvider::Google => {
                let has_api_key = self.api_key.as_ref()
                    .map(|k| !k.is_empty() && k != "none")
                    .unwrap_or(false);
                if !has_api_key {
                    return Ok(false);
                }
                // Simple reachability check
                let resp = self.client.get(&self.base_url)
                    .timeout(std::time::Duration::from_secs(3))
                    .send()
                    .await;
                Ok(resp.is_ok())
            }
        }
    }

    fn hash_content(&self, content: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        hasher.update(self.model.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    async fn get_cached(&self, hash: &str) -> Result<Option<Embedding>> {
        let conn = self.conn.lock().await;
        
        let result: Option<(Vec<u8>, String)> = conn.query_row(
            "SELECT embedding, model FROM embedding_cache WHERE content_hash = ?1",
            [hash],
            |row| Ok((row.get(0)?, row.get(1)?)),
        ).ok();

        if let Some((blob, model)) = result {
            let vector = crate::shared::blob_to_vec(&blob)?;
            return Ok(Some(Embedding {
                vector,
                model,
                dimensions: self.dimensions,
            }));
        }

        Ok(None)
    }

    async fn cache(&self, hash: &str, content: &str, vector: &[f32]) -> Result<()> {
        let conn = self.conn.lock().await;
        let blob = crate::shared::vec_to_blob(vector);
        let now = chrono::Utc::now().to_rfc3339();

        conn.execute(
            "INSERT OR REPLACE INTO embedding_cache (content_hash, content, embedding, model, created_at) VALUES (?1, ?2, ?3, ?4, ?5)",
            rusqlite::params![hash, content, blob, self.model, now],
        )?;

        Ok(())
    }

    /// Calculate cosine similarity between two vectors
    pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
        crate::shared::cosine_similarity(a, b)
    }

    /// Euclidean distance between two vectors
    pub fn euclidean_distance(a: &[f32], b: &[f32]) -> f32 {
        if a.len() != b.len() {
            return f32::MAX;
        }

        a.iter()
            .zip(b.iter())
            .map(|(x, y)| (x - y).powi(2))
            .sum::<f32>()
            .sqrt()
    }
}

/// Configuration for embedding providers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbedderConfig {
    pub provider: String,
    pub model: String,
    pub api_key: Option<String>,
    pub base_url: Option<String>,
}

impl Default for EmbedderConfig {
    fn default() -> Self {
        Self {
            provider: "ollama".to_string(),
            model: "nomic-embed-text".to_string(),
            api_key: None,
            base_url: Some("http://localhost:11434".to_string()),
        }
    }
}

impl EmbedderConfig {
    pub fn from_env() -> Self {
        Self {
            provider: std::env::var("EMBEDDING_PROVIDER")
                .unwrap_or_else(|_| "ollama".to_string()),
            model: std::env::var("EMBEDDING_MODEL")
                .unwrap_or_else(|_| "nomic-embed-text".to_string()),
            api_key: std::env::var("EMBEDDING_API_KEY").ok(),
            base_url: std::env::var("EMBEDDING_BASE_URL").ok(),
        }
    }

    pub fn provider(&self) -> EmbeddingProvider {
        match self.provider.to_lowercase().as_str() {
            "openai" => EmbeddingProvider::OpenAI,
            "openrouter" => EmbeddingProvider::OpenRouter,
            "google" => EmbeddingProvider::Google,
            _ => EmbeddingProvider::Ollama,
        }
    }

    pub fn base_url(&self) -> String {
        self.base_url.clone().unwrap_or_else(|| {
            match self.provider() {
                EmbeddingProvider::Ollama => "http://localhost:11434".to_string(),
                EmbeddingProvider::OpenAI => "https://api.openai.com".to_string(),
                EmbeddingProvider::OpenRouter => "https://openrouter.ai".to_string(),
                EmbeddingProvider::Google => "https://generativelanguage.googleapis.com".to_string(),
            }
        })
    }
}
