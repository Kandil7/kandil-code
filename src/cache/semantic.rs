//! Semantic cache for Kandil Code
//!
//! Implements semantic similarity-based caching for AI responses.

use dashmap::DashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tiktoken_rs::ChatCompletionRequestMessage;

// We'll use a simple embedding approach for now - in production,
// we'd want to use a proper embedding model
use tiktoken_rs::num_tokens_from_messages;

pub struct SemanticCache {
    // Storage for cache entries - using a simpler approach without HNSW
    store: Arc<DashMap<usize, CacheEntry>>,
    // Configuration for the cache
    config: CacheConfig,
    // Simple hash map for exact matches (faster lookup)
    exact_store: Arc<DashMap<u64, CacheEntry>>,
}

#[derive(Debug)]
struct CacheEntry {
    response: String,
    tokens: usize,
    hit_count: AtomicU64,
    last_access: AtomicU64,
    created_at: std::time::SystemTime,
}

impl Clone for CacheEntry {
    fn clone(&self) -> Self {
        CacheEntry {
            response: self.response.clone(),
            tokens: self.tokens,
            hit_count: AtomicU64::new(self.hit_count.load(Ordering::Relaxed)),
            last_access: AtomicU64::new(self.last_access.load(Ordering::Relaxed)),
            created_at: self.created_at,
        }
    }
}

#[derive(Debug, Clone)]
struct CacheConfig {
    max_size: usize,
    similarity_threshold: f32, // 0.95 = 95% similarity
    ttl_seconds: u64,          // Time to live for cache entries
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            max_size: 10000,
            similarity_threshold: 0.90, // 90% similarity threshold
            ttl_seconds: 3600,          // 1 hour TTL
        }
    }
}

impl SemanticCache {
    pub fn new(config: CacheConfig) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            store: Arc::new(DashMap::new()),
            exact_store: Arc::new(DashMap::new()),
            config,
        })
    }

    pub async fn get(&self, prompt: &str) -> Option<String> {
        // First check for exact match (fastest)
        let exact_hash = self.calculate_hash(prompt);
        if let Some(entry) = self.exact_store.get(&exact_hash) {
            // Update access stats
            entry.hit_count.fetch_add(1, Ordering::Relaxed);
            entry.last_access.store(
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
                Ordering::Relaxed,
            );

            // Check if entry is expired
            if self.is_expired(&entry.created_at) {
                self.exact_store.remove(&exact_hash);
                return None;
            }

            return Some(entry.response.clone());
        }

        // Then check for semantic similarity
        // In a real implementation we would generate proper embeddings
        // For now, we'll use a simplified approach based on token analysis
        let similarity_result = self.find_semantic_match(prompt).await;

        if let Some(response) = similarity_result {
            Some(response)
        } else {
            None
        }
    }

    async fn find_semantic_match(&self, prompt: &str) -> Option<String> {
        // This is a simplified implementation
        // In a real system we would:
        // 1. Generate embeddings for the prompt
        // 2. Search the HNSW index for similar embeddings
        // 3. Return the most similar cached response

        // For now, we'll do a simple keyword-based approach
        let prompt_lower = prompt.to_lowercase();
        let prompt_tokens: Vec<&str> = prompt_lower.split_whitespace().collect();

        // Check store for cache entries with significant overlap in keywords
        for entry in self.store.iter() {
            if self.is_expired(&entry.created_at) {
                continue;
            }

            let cached_response_lower = entry.response.to_lowercase();
            let cached_tokens: Vec<&str> = cached_response_lower.split_whitespace().collect();

            // Calculate keyword overlap
            let overlap: usize = prompt_tokens
                .iter()
                .filter(|token| cached_tokens.contains(token))
                .count();

            let total_tokens = prompt_tokens.len().max(cached_tokens.len());
            let similarity = if total_tokens > 0 {
                overlap as f32 / total_tokens as f32
            } else {
                0.0
            };

            if similarity >= self.config.similarity_threshold {
                // Update access stats
                entry.hit_count.fetch_add(1, Ordering::Relaxed);
                entry.last_access.store(
                    std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_secs(),
                    Ordering::Relaxed,
                );

                return Some(entry.response.clone());
            }
        }

        None
    }

    pub async fn put(&self, prompt: String, response: String) {
        // Prune if we're over capacity
        if self.store.len() >= self.config.max_size {
            self.evict_lru().await;
        }

        let tokens = self.count_tokens(&response);
        let created_at = std::time::SystemTime::now();

        // Calculate prompt hash for exact match lookup
        let prompt_hash = self.calculate_hash(&prompt);

        // Add to exact match store
        let entry = CacheEntry {
            response: response.clone(),
            tokens,
            hit_count: AtomicU64::new(1),
            last_access: AtomicU64::new(
                created_at
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
            ),
            created_at,
        };

        self.exact_store.insert(prompt_hash, entry.clone());

        // Add to semantic store with a simple numeric index
        let idx = self.store.len();
        self.store.insert(idx, entry);
    }

    async fn evict_lru(&self) {
        // Remove least recently used entry
        let lru_entry = self
            .store
            .iter()
            .filter(|entry| !self.is_expired(&entry.created_at))
            .min_by_key(|entry| entry.last_access.load(Ordering::Relaxed));

        if let Some(entry) = lru_entry {
            self.store.remove(&entry.key());
        }
    }

    fn calculate_hash(&self, s: &str) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        s.hash(&mut hasher);
        hasher.finish()
    }

    fn count_tokens(&self, text: &str) -> usize {
        // Use tiktoken_rs to count tokens accurately
        match num_tokens_from_messages(
            "gpt-3.5-turbo",
            &[ChatCompletionRequestMessage {
                role: "user".to_string(),
                content: Some(text.to_string()),
                name: None,
                function_call: None,
            }],
        ) {
            Ok(count) => count,
            Err(_) => {
                // Fallback to a rough character-based estimate if token counting fails
                text.chars().count() / 4 // Rough estimate: 1 token ~ 4 characters
            }
        }
    }
    fn is_expired(&self, created_at: &std::time::SystemTime) -> bool {
        if self.config.ttl_seconds == 0 {
            return false; // No TTL
        }

        match created_at.elapsed() {
            Ok(duration) => duration.as_secs() > self.config.ttl_seconds,
            Err(_) => true, // If there's an error with time, consider it expired
        }
    }
}

impl Default for SemanticCache {
    fn default() -> Self {
        Self::new(CacheConfig::default()).expect("Failed to create default semantic cache")
    }
}

// Add the cache module to the cache mod.rs file
