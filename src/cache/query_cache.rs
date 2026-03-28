//! Query embedding cache for fast semantic search
//!
//! Caches query embeddings to avoid repeated calls to embedding services.
//! Uses LRU eviction to manage cache size.

use anyhow::Result;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedEmbedding {
    pub vector: Vec<f32>,
    pub model: String,
    pub created_at: String,
    pub access_count: u64,
}

pub struct QueryCache {
    cache: Arc<RwLock<HashMap<String, CachedEmbedding>>>,
    max_size: usize,
    ttl_seconds: i64,
}

impl QueryCache {
    pub fn new(max_size: usize, ttl_seconds: i64) -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
            max_size,
            ttl_seconds,
        }
    }

    /// Get or create a cached embedding
    pub async fn get_or_create<F, Fut>(
        &self,
        query: &str,
        create_fn: F,
    ) -> Result<Vec<f32>>
    where
        F: FnOnce(&str) -> Fut,
        Fut: std::future::Future<Output = Result<Vec<f32>>>,
    {
        let hash = Self::hash_query(query);

        // Try to get from cache
        {
            let cache = self.cache.read().await;
            if let Some(cached) = cache.get(&hash) {
                // Check if expired
                let created_at = chrono::DateTime::parse_from_rfc3339(&cached.created_at)
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now());
                let age = (Utc::now() - created_at).num_seconds();

                if age < self.ttl_seconds {
                    // Update access count
                    let hash_clone = hash.clone();
                    drop(cache);
                    let mut cache = self.cache.write().await;
                    if let Some(cached) = cache.get_mut(&hash_clone) {
                        cached.access_count += 1;
                        let vector = cached.vector.clone();
                        drop(cache);
                        tracing::debug!("Cache hit for query: {}", query.chars().take(50).collect::<String>());
                        return Ok(vector);
                    }
                }
            }
        }

        // Cache miss - create new embedding
        tracing::debug!("Cache miss for query: {}", query.chars().take(50).collect::<String>());
        let vector = create_fn(query).await?;

        // Store in cache
        let mut cache = self.cache.write().await;
        
        // Evict if cache is full
        if cache.len() >= self.max_size {
            self.evict_lru(&mut cache);
        }

        let cached = CachedEmbedding {
            vector: vector.clone(),
            model: "cached".to_string(),
            created_at: Utc::now().to_rfc3339(),
            access_count: 1,
        };

        cache.insert(hash, cached);

        Ok(vector)
    }

    /// Evict least recently used entry
    fn evict_lru(&self, cache: &mut HashMap<String, CachedEmbedding>) {
        if let Some(lru_key) = cache.iter()
            .min_by_key(|(_, v)| v.access_count)
            .map(|(k, _)| k.clone())
        {
            tracing::debug!("Evicting LRU cache entry");
            cache.remove(&lru_key);
        }
    }

    /// Clear all cached embeddings
    pub async fn clear(&self) -> Result<()> {
        let mut cache = self.cache.write().await;
        cache.clear();
        tracing::info!("Query cache cleared");
        Ok(())
    }

    /// Get cache statistics
    pub async fn stats(&self) -> CacheStats {
        let cache = self.cache.read().await;
        let total_access: u64 = cache.values().map(|c| c.access_count).sum();
        let cache_len = cache.len() as u64;
        let avg_access = if cache.is_empty() {
            0.0
        } else {
            total_access as f64 / cache.len() as f64
        };

        CacheStats {
            size: cache.len(),
            max_size: self.max_size,
            total_access,
            avg_access,
            hit_rate: if total_access > 0 {
                (total_access - cache_len) as f64 / total_access as f64
            } else {
                0.0
            },
        }
    }

    /// Hash query string for cache key
    fn hash_query(query: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(query.as_bytes());
        format!("{:x}", hasher.finalize())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheStats {
    pub size: usize,
    pub max_size: usize,
    pub total_access: u64,
    pub avg_access: f64,
    pub hit_rate: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_query_cache() {
        let cache = QueryCache::new(100, 3600);

        // First call - cache miss
        let result1 = cache.get_or_create("test query", |query| {
            async { Ok(vec![0.1, 0.2, 0.3]) }
        }).await.unwrap();

        // Second call - cache hit
        let result2 = cache.get_or_create("test query", |query| {
            async { Ok(vec![0.4, 0.5, 0.6]) }
        }).await.unwrap();

        assert_eq!(result1, result2);

        let stats = cache.stats().await;
        assert_eq!(stats.size, 1);
        assert_eq!(stats.total_access, 2);
    }
}
