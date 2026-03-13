//! Session-level LRU cache for hot memories
//! 
//! This provides fast in-memory caching for frequently accessed memories
//! within a session, reducing database calls significantly.

use lru::LruCache;
use std::num::NonZeroUsize;
use std::sync::Arc;
use tokio::sync::Mutex;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedMemory {
    pub id: String,
    pub content: String,
    pub importance: f64,
    pub category: String,
    pub tags: Vec<String>,
    pub embedding: Option<Vec<f32>>,
}

pub struct SessionCache {
    cache: Arc<Mutex<LruCache<String, CachedMemory>>>,
    hits: Arc<Mutex<u64>>,
    misses: Arc<Mutex<u64>>,
}

impl SessionCache {
    pub fn new(capacity: usize) -> Self {
        Self {
            cache: Arc::new(Mutex::new(LruCache::new(
                NonZeroUsize::new(capacity).unwrap_or(NonZeroUsize::new(100).unwrap())
            ))),
            hits: Arc::new(Mutex::new(0)),
            misses: Arc::new(Mutex::new(0)),
        }
    }

    pub async fn get(&self, key: &str) -> Option<CachedMemory> {
        let mut cache = self.cache.lock().await;
        if let Some(memory) = cache.get(key) {
            let mut hits = self.hits.lock().await;
            *hits += 1;
            return Some(memory.clone());
        }
        let mut misses = self.misses.lock().await;
        *misses += 1;
        None
    }

    pub async fn put(&self, key: String, value: CachedMemory) {
        let mut cache = self.cache.lock().await;
        cache.put(key, value);
    }

    pub async fn invalidate(&self, key: &str) {
        let mut cache = self.cache.lock().await;
        cache.pop(key);
    }

    pub async fn clear(&self) {
        let mut cache = self.cache.lock().await;
        cache.clear();
    }

    pub async fn stats(&self) -> CacheStats {
        let hits = *self.hits.lock().await;
        let misses = *self.misses.lock().await;
        let total = hits + misses;
        let hit_rate = if total > 0 { hits as f64 / total as f64 } else { 0.0 };
        
        let cache = self.cache.lock().await;
        let len = cache.len();
        let capacity = cache.cap().get();
        
        CacheStats {
            hits,
            misses,
            hit_rate,
            entries: len,
            capacity,
        }
    }
}

impl Default for SessionCache {
    fn default() -> Self {
        Self::new(100)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheStats {
    pub hits: u64,
    pub misses: u64,
    pub hit_rate: f64,
    pub entries: usize,
    pub capacity: usize,
}

