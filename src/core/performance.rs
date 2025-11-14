//! Performance Optimizations with Caching and Prefetching
//!
//! Implements semantic caching with <500ms response for cached operations
//! and context prefetching for likely next operations

use anyhow::Result;
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::time::{sleep, Duration};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntry {
    pub value: String,
    pub timestamp: std::time::SystemTime,
    pub access_count: u32,
    pub estimated_tokens: usize,
}

pub struct LatencyOptimizer {
    /// LRU cache for model outputs (semantic deduplication)
    cache: Arc<DashMap<String, CacheEntry>>,
    /// Prefetches context for likely next operations
    prefetcher: Arc<ContextPrefetcher>,
    /// Maximum cache size
    max_cache_size: usize,
}

impl LatencyOptimizer {
    pub fn new(max_cache_size: usize) -> Self {
        Self {
            cache: Arc::new(DashMap::new()),
            prefetcher: Arc::new(ContextPrefetcher::new()),
            max_cache_size,
        }
    }

    pub async fn complete_with_cache(&self, prompt: &str) -> Result<String> {
        let hash = self.semantic_hash(prompt);

        // Check if cached
        if let Some(cached) = self.cache.get(&hash) {
            // Update access count
            let mut entry = cached.clone();
            entry.access_count += 1;
            self.cache.insert(hash.clone(), entry);
            
            return Ok(cached.value.clone());
        }

        // Prefetch while generating (in the background)
        let prefetch_handle = self.prefetcher.prefetch_next_context(prompt);
        
        // Simulate model completion (in a real implementation, this would call the AI model)
        let result = self.simulate_model_completion(prompt).await?;
        
        // Store in cache
        let cache_entry = CacheEntry {
            value: result.clone(),
            timestamp: std::time::SystemTime::now(),
            access_count: 1,
            estimated_tokens: prompt.len() / 4, // Rough token estimation
        };
        
        self.cache.insert(hash, cache_entry);
        
        // Wait for prefetch to complete (non-blocking in real usage)
        let _ = prefetch_handle.await;
        
        // Apply cache size limit
        self.maintain_cache_size();
        
        Ok(result)
    }

    fn semantic_hash(&self, prompt: &str) -> String {
        // Create a hash that captures semantic meaning rather than exact text
        // In a real implementation, this would use embeddings or more sophisticated techniques
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        // Normalize the prompt for semantic similarity
        let normalized = prompt
            .to_lowercase()
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ");
        
        normalized.hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }

    async fn simulate_model_completion(&self, prompt: &str) -> Result<String> {
        // Simulate the time it takes to get a response from a model
        // In a real implementation, this would call the actual AI provider
        sleep(Duration::from_millis(100)).await; // Simulated API delay
        Ok(format!("Response to: {}", prompt))
    }

    fn maintain_cache_size(&self) {
        // Remove least recently used items if cache is too large
        if self.cache.len() > self.max_cache_size {
            // Convert to vector and sort by access count and recency
            let mut entries: Vec<_> = self.cache.iter().map(|entry| {
                (entry.key().clone(), entry.value().clone())
            }).collect();
            
            // Sort by access count (ascending) and then by time (oldest first)
            entries.sort_by(|a, b| {
                a.1.access_count.cmp(&b.1.access_count)
                    .then(a.1.timestamp.duration_since(std::time::UNIX_EPOCH)
                          .unwrap_or_default()
                          .cmp(&b.1.timestamp.duration_since(std::time::UNIX_EPOCH)
                               .unwrap_or_default()))
            });
            
            // Remove the oldest/least accessed entries
            let to_remove = entries.len() - self.max_cache_size + 10; // Keep 10% buffer
            for i in 0..to_remove.min(entries.len()) {
                self.cache.remove(&entries[i].0);
            }
        }
    }

    pub fn get_cache_stats(&self) -> (usize, usize) {
        (self.cache.len(), self.max_cache_size)
    }
}

pub struct ContextPrefetcher {
    /// Simulates prefetching for likely next operations
    prefetch_tasks: Arc<DashMap<String, String>>,
}

impl ContextPrefetcher {
    pub fn new() -> Self {
        Self {
            prefetch_tasks: Arc::new(DashMap::new()),
        }
    }

    pub async fn prefetch_next_context(&self, current_prompt: &str) -> tokio::task::JoinHandle<Result<()>> {
        let prefetch_tasks = Arc::clone(&self.prefetch_tasks);
        let current_prompt = current_prompt.to_string();
        
        // Spawn a background task to prefetch likely next context
        tokio::spawn(async move {
            // In a real implementation, this would analyze the current prompt
            // to predict what context will be needed next and prefetch it
            let next_context_key = format!("next_{}", current_prompt);
            
            // Simulate prefetching work
            sleep(Duration::from_millis(50)).await;
            
            // Store the prefetched context
            prefetch_tasks.insert(next_context_key, "prefetched_context".to_string());
            
            Ok(())
        })
    }

    pub async fn get_prefetched_context(&self, key: &str) -> Option<String> {
        self.prefetch_tasks.get(key).map(|v| v.value().clone())
    }
}

pub struct TokenEstimator;

impl TokenEstimator {
    pub fn estimate_tokens(&self, text: &str) -> usize {
        // Rough estimation: 1 token ≈ 4 characters for English text
        // In a real implementation, we'd use actual tokenizers for specific models
        (text.len() / 4).max(1)
    }

    pub fn estimate_tokens_exact(&self, text: &str) -> usize {
        // More precise estimation that counts words, punctuation, etc.
        let mut tokens = 0;
        let mut chars = text.chars().peekable();
        
        while let Some(ch) = chars.next() {
            match ch {
                c if c.is_alphanumeric() => {
                    // Count alphanumeric sequences as tokens
                    while chars.peek().map_or(false, |&c| c.is_alphanumeric()) {
                        chars.next();
                    }
                    tokens += 1;
                }
                c if c.is_whitespace() => {
                    // Skip whitespace
                }
                c if c.is_ascii_punctuation() => {
                    // Punctuation is typically separate tokens
                    tokens += 1;
                }
                _ => {
                    // Other characters
                    tokens += 1;
                }
            }
        }
        
        tokens.max(1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_latency_optimizer_cache() {
        let optimizer = LatencyOptimizer::new(100);
        
        // First call should take longer (simulate model call)
        let result1 = optimizer.complete_with_cache("test prompt").await.unwrap();
        assert!(result1.contains("test prompt"));
        
        // Second call should be faster (cached)
        let result2 = optimizer.complete_with_cache("test prompt").await.unwrap();
        assert_eq!(result1, result2);
        
        // Check cache stats
        let (current_size, max_size) = optimizer.get_cache_stats();
        assert_eq!(current_size, 1);
        assert_eq!(max_size, 100);
    }

    #[test]
    fn test_semantic_hash() {
        let optimizer = LatencyOptimizer::new(100);
        
        // Similar prompts should have same hash (after normalization)
        let hash1 = optimizer.semantic_hash("Hello world");
        let hash2 = optimizer.semantic_hash("Hello  world"); // extra space
        let hash3 = optimizer.semantic_hash("Goodbye world");
        
        assert_eq!(hash1, hash2); // Same semantic meaning
        assert_ne!(hash1, hash3); // Different meaning
    }

    #[test]
    fn test_token_estimator() {
        let estimator = TokenEstimator;
        
        let tokens = estimator.estimate_tokens("Hello world!");
        assert!(tokens > 0);
        
        let exact_tokens = estimator.estimate_tokens_exact("Hello world!");
        assert!(exact_tokens > 0);
        assert!(exact_tokens >= tokens);
    }
}