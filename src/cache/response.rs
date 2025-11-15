//! Response cache with TTL for Kandil Code
//!
//! Implements a simple in-memory cache with time-to-live for AI responses.

use std::sync::Arc;
use std::time::{SystemTime, Duration, UNIX_EPOCH};
use dashmap::DashMap;

pub struct ResponseCache {
    cache: Arc<DashMap<String, CachedResponse>>,
    ttl: Duration,
}

struct CachedResponse {
    response: String,
    created_at: u64, // Unix timestamp
    prompt_hash: u64,
}

impl ResponseCache {
    pub fn new(ttl_minutes: u64) -> Self {
        Self {
            cache: Arc::new(DashMap::new()),
            ttl: Duration::from_secs(ttl_minutes * 60),
        }
    }

    pub async fn get(&self, prompt: &str) -> Option<String> {
        let hash = self.calculate_hash(prompt);
        let key = hash.to_string();
        
        if let Some(entry) = self.cache.get(&key) {
            // Check if entry is expired
            let current_time = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();
                
            if current_time - entry.created_at < self.ttl.as_secs() {
                // Entry is still valid
                Some(entry.response.clone())
            } else {
                // Entry is expired, remove it
                self.cache.remove(&key);
                None
            }
        } else {
            None
        }
    }

    pub async fn insert(&self, prompt: &str, response: String) {
        let hash = self.calculate_hash(prompt);
        let key = hash.to_string();
        
        let created_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
            
        self.cache.insert(
            key, 
            CachedResponse {
                response,
                created_at,
                prompt_hash: hash,
            }
        );
    }

    pub async fn remove(&self, prompt: &str) -> Option<String> {
        let hash = self.calculate_hash(prompt);
        let key = hash.to_string();
        
        self.cache.remove(&key).map(|(_, entry)| entry.response)
    }

    pub async fn clear_expired(&self) {
        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
            
        self.cache.retain(|_, entry| {
            current_time - entry.created_at < self.ttl.as_secs()
        });
    }

    pub async fn size(&self) -> usize {
        self.cache.len()
    }

    pub async fn clear(&self) {
        self.cache.clear();
    }

    fn calculate_hash(&self, s: &str) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        s.hash(&mut hasher);
        hasher.finish()
    }
}

impl Default for ResponseCache {
    fn default() -> Self {
        Self::new(30) // Default TTL: 30 minutes
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio;

    #[tokio::test]
    async fn test_cache_insert_and_get() {
        let cache = ResponseCache::new(60); // 60 minute TTL
        
        let prompt = "test prompt";
        let response = "test response".to_string();
        
        cache.insert(prompt, response.clone()).await;
        let retrieved = cache.get(prompt).await;
        
        assert_eq!(retrieved, Some(response));
    }

    #[tokio::test]
    async fn test_cache_expiration() {
        let cache = ResponseCache::new(0); // 0 second TTL (immediate expiration)
        
        let prompt = "test prompt";
        let response = "test response".to_string();
        
        cache.insert(prompt, response).await;
        let retrieved = cache.get(prompt).await;
        
        assert_eq!(retrieved, None);
    }

    #[tokio::test]
    async fn test_cache_clear_expired() {
        let cache = ResponseCache::new(0); // 0 second TTL
        
        let prompt = "test prompt";
        let response = "test response".to_string();
        
        cache.insert(prompt, response).await;
        cache.clear_expired().await;
        
        assert_eq!(cache.size().await, 0);
    }
}