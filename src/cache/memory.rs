use crate::cache::CacheProvider;
use crate::error::Result;
use crate::format::StructuredValue;
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

#[derive(Clone)]
struct CacheEntry {
    value: StructuredValue,
    expires_at: Option<Instant>,
}

/// High-performance, thread-safe in-memory cache provider with TTL expiration.
#[derive(Clone, Default)]
pub struct MemoryCache {
    store: Arc<RwLock<HashMap<String, CacheEntry>>>,
    default_ttl: Option<Duration>,
}

impl MemoryCache {
    pub fn new() -> Self {
        Self {
            store: Arc::new(RwLock::new(HashMap::new())),
            default_ttl: None,
        }
    }

    pub fn with_default_ttl(ttl: Duration) -> Self {
        Self {
            store: Arc::new(RwLock::new(HashMap::new())),
            default_ttl: Some(ttl),
        }
    }

    /// Return the number of active (non-expired) entries in the cache
    pub async fn len(&self) -> usize {
        let store = self.store.read().await;
        let now = Instant::now();
        store
            .values()
            .filter(|e| e.expires_at.map(|exp| exp > now).unwrap_or(true))
            .count()
    }

    pub async fn is_empty(&self) -> bool {
        self.len().await == 0
    }
}

#[async_trait]
impl CacheProvider for MemoryCache {
    async fn get(&self, key: &str) -> Result<Option<StructuredValue>> {
        let store = self.store.read().await;
        if let Some(entry) = store.get(key) {
            if let Some(exp) = entry.expires_at {
                if Instant::now() > exp {
                    return Ok(None);
                }
            }
            return Ok(Some(entry.value.clone()));
        }
        Ok(None)
    }

    async fn set(
        &self,
        key: &str,
        value: &StructuredValue,
        ttl: Option<Duration>,
    ) -> Result<()> {
        let effective_ttl = ttl.or(self.default_ttl);
        let expires_at = effective_ttl.map(|d| Instant::now() + d);

        let mut store = self.store.write().await;
        store.insert(
            key.to_string(),
            CacheEntry {
                value: value.clone(),
                expires_at,
            },
        );
        Ok(())
    }

    async fn invalidate(&self, key: &str) -> Result<()> {
        let mut store = self.store.write().await;
        store.remove(key);
        Ok(())
    }

    async fn clear(&self) -> Result<()> {
        let mut store = self.store.write().await;
        store.clear();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_memory_cache_hit_and_miss() {
        let cache = MemoryCache::new();
        let key = "user:42";
        let val = StructuredValue::String("John Doe".to_string());

        assert!(cache.get(key).await.unwrap().is_none());

        cache.set(key, &val, None).await.unwrap();
        assert_eq!(cache.get(key).await.unwrap(), Some(val.clone()));

        cache.invalidate(key).await.unwrap();
        assert!(cache.get(key).await.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_memory_cache_ttl_expiration() {
        let cache = MemoryCache::new();
        let key = "temp:1";
        let val = StructuredValue::String("Ephemeral".to_string());

        // Expire in 10 milliseconds
        cache
            .set(key, &val, Some(Duration::from_millis(10)))
            .await
            .unwrap();
        assert!(cache.get(key).await.unwrap().is_some());

        tokio::time::sleep(Duration::from_millis(20)).await;
        assert!(cache.get(key).await.unwrap().is_none());
    }
}
