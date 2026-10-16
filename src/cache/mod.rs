pub mod memory;

pub use memory::MemoryCache;

use crate::error::Result;
use crate::format::StructuredValue;
use async_trait::async_trait;
use std::time::Duration;

/// Pluggable read-through cache provider trait.
///
/// Enables seamless caching of orchestrated results across any storage medium
/// (In-memory, Redis, PostgreSQL, Disk) without coupling Orcrustrator to any database driver.
#[async_trait]
pub trait CacheProvider: Send + Sync {
    /// Retrieve a cached StructuredValue by key
    async fn get(&self, key: &str) -> Result<Option<StructuredValue>>;

    /// Store a StructuredValue in the cache with an optional time-to-live (TTL)
    async fn set(
        &self,
        key: &str,
        value: &StructuredValue,
        ttl: Option<Duration>,
    ) -> Result<()>;

    /// Invalidate a specific cache entry
    async fn invalidate(&self, key: &str) -> Result<()>;

    /// Clear all cached entries
    async fn clear(&self) -> Result<()>;
}
