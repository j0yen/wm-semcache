//! `wm-semcache` — embedding-keyed semantic response cache.
// `pub(crate)` in private modules is redundant per `redundant_pub_crate`, but
// we keep it to satisfy the `unreachable_pub` lint which requires non-exported
// items to not be bare `pub`. Suppress the contradiction here.
#![allow(clippy::redundant_pub_crate)]
//!
//! Caches responses to utterances keyed by their embedding vector.
//! A cache hit is declared when the cosine similarity between a new
//! utterance's embedding and a stored entry's embedding meets the
//! configured threshold.
//!
//! ## Safety property
//!
//! Utterances classified as cache-unsafe (time, weather, calendar-today, etc.)
//! are **never stored and never served** from the cache. The `cache_safe` flag
//! is set by the caller at [`SemCache::store`] time; wm-semcache enforces the
//! contract but does not classify intents.
//!
//! ## Usage
//!
//! ```rust
//! use wm_semcache::{SemCache, SemCacheConfig, Ttl};
//!
//! // A deterministic stub embedder for doctests.
//! struct FixedEmbed(Vec<f32>);
//! impl wm_semcache::Embedder for FixedEmbed {
//!     fn embed(&self, _utterance: &str) -> Result<Vec<f32>, wm_semcache::EmbedError> {
//!         Ok(self.0.clone())
//!     }
//! }
//!
//! let embedder = FixedEmbed(vec![1.0, 0.0]);
//! let mut cache = SemCache::new(SemCacheConfig::default(), Box::new(embedder));
//! cache.store("hello world", "Hi!", true, Ttl::Secs(300)).unwrap();
//! let hit = cache.lookup("hello world").unwrap();
//! assert_eq!(hit.response, "Hi!");
//! ```

mod clock;
mod entry;
mod error;
mod gate;
mod lru;
mod metrics;
mod similarity;

pub use clock::ClockFn;
pub use entry::Entry;
pub use error::{EmbedError, SemCacheError};
pub use gate::UNSAFE_INTENTS;
pub use metrics::CacheMetrics;

use std::collections::HashMap;

/// A cached response returned on a successful lookup.
#[derive(Debug, Clone)]
pub struct CachedHit {
    /// The original utterance that was stored.
    pub utterance: String,
    /// The response associated with the stored utterance.
    pub response: String,
    /// Cosine similarity score that triggered this hit.
    pub similarity: f32,
}

/// Time-to-live for a cache entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ttl {
    /// Entry lives for `n` seconds.
    Secs(u64),
    /// Entry never expires (use with caution — only for truly static facts).
    Never,
}

/// Trait for embedding utterances into f32 vectors.
///
/// Implementors must produce L2-normalised vectors so that dot-product ==
/// cosine similarity. The built-in [`crate::similarity::cosine`] function
/// re-normalises internally so un-normalised vectors still work, but
/// normalising in the embedder is more efficient.
pub trait Embedder: Send + Sync {
    /// Embed `utterance` into an f32 vector.
    ///
    /// Returns [`EmbedError`] if the embedding service is unreachable or
    /// returns an invalid response. The caller treats any error as a cache
    /// miss.
    ///
    /// # Errors
    ///
    /// Returns [`EmbedError::Unreachable`] if the embedding service is
    /// unavailable, or [`EmbedError::InvalidResponse`] if the service
    /// returns an unexpected or empty vector.
    fn embed(&self, utterance: &str) -> Result<Vec<f32>, EmbedError>;
}

/// Configuration for [`SemCache`].
#[derive(Debug, Clone)]
pub struct SemCacheConfig {
    /// Cosine similarity threshold (0.0 – 1.0). Entries with similarity ≥
    /// this value are considered a hit. Default: 0.85.
    pub similarity_threshold: f32,
    /// Maximum number of live (non-expired) entries. When the bound is
    /// exceeded the least-recently-hit entry is evicted. Default: 1000.
    pub capacity: usize,
    /// Default TTL when the caller passes [`Ttl::Secs`] but provides no
    /// override. Not used directly; callers always pass an explicit [`Ttl`].
    pub default_ttl_secs: u64,
    /// Clock function used for TTL expiry. Defaults to [`std::time::SystemTime`].
    /// Replace with a frozen clock in tests.
    pub clock: ClockFn,
}

impl Default for SemCacheConfig {
    fn default() -> Self {
        Self {
            similarity_threshold: 0.85,
            capacity: 1000,
            default_ttl_secs: 300,
            clock: ClockFn::wall(),
        }
    }
}

/// Embedding-keyed semantic response cache.
///
/// Thread-safety: `SemCache` is NOT `Sync`. Wrap in `Mutex` if shared across
/// threads. This is intentional: the access pattern (local device, single
/// owner) does not need concurrent reads.
pub struct SemCache {
    config: SemCacheConfig,
    embedder: Box<dyn Embedder>,
    /// Ordered list of entry keys for LRU tracking.
    lru: lru::LruOrder,
    /// Map from a stable entry ID to the entry.
    entries: HashMap<u64, Entry>,
    /// Monotonically increasing counter for assigning entry IDs.
    next_id: u64,
    /// Deflection / diagnostic metrics.
    metrics: CacheMetrics,
}

impl SemCache {
    /// Create a new cache with `config` and `embedder`.
    #[must_use]
    pub fn new(config: SemCacheConfig, embedder: Box<dyn Embedder>) -> Self {
        Self {
            config,
            embedder,
            lru: lru::LruOrder::new(),
            entries: HashMap::new(),
            next_id: 0,
            metrics: CacheMetrics::default(),
        }
    }

    /// Look up a response for `utterance`.
    ///
    /// Returns `None` when:
    /// - The utterance is classified cache-unsafe (by the unsafe-intent gate).
    /// - The embedder is unreachable.
    /// - No entry meets the cosine similarity threshold.
    /// - All candidate entries have expired.
    ///
    /// Never panics.
    pub fn lookup(&mut self, utterance: &str) -> Option<CachedHit> {
        self.metrics.total_lookups += 1;

        // Safety gate: cache-unsafe utterances are never served.
        if gate::is_cache_unsafe(utterance) {
            return None;
        }

        // Embed the utterance; any error → cache miss (never panic).
        let Ok(query_vec) = self.embedder.embed(utterance) else {
            return None;
        };

        let now_secs = (self.config.clock.now_secs)();
        let threshold = self.config.similarity_threshold;

        let mut best_sim: f32 = -1.0_f32;
        let mut best_id: Option<u64> = None;

        for (&id, entry) in &self.entries {
            if entry.is_expired(now_secs) {
                continue;
            }
            let sim = similarity::cosine(&query_vec, &entry.vec);
            if sim >= threshold && sim > best_sim {
                best_sim = sim;
                best_id = Some(id);
            }
        }

        if let Some(id) = best_id {
            // Promote to MRU position.
            self.lru.touch(id);
            self.metrics.hits += 1;

            let entry = self.entries.get(&id)?;
            Some(CachedHit {
                utterance: entry.utterance.clone(),
                response: entry.response.clone(),
                similarity: best_sim,
            })
        } else {
            None
        }
    }

    /// Store a response for `utterance` in the cache.
    ///
    /// `cache_safe` must be `true`; if `false`, or if the utterance passes
    /// the unsafe-intent gate, the call returns
    /// [`SemCacheError::CacheUnsafe`] and nothing is stored.
    ///
    /// # Errors
    ///
    /// Returns [`SemCacheError::CacheUnsafe`] when the entry is not safe to
    /// cache. Returns [`SemCacheError::EmbedFailed`] when the embedder
    /// cannot produce a vector.
    pub fn store(
        &mut self,
        utterance: &str,
        response: &str,
        cache_safe: bool,
        ttl: Ttl,
    ) -> Result<(), SemCacheError> {
        // Dual gate: caller flag AND intent classification.
        if !cache_safe || gate::is_cache_unsafe(utterance) {
            return Err(SemCacheError::CacheUnsafe);
        }

        let vec = self
            .embedder
            .embed(utterance)
            .map_err(SemCacheError::EmbedFailed)?;

        let now_secs = (self.config.clock.now_secs)();
        let ttl_secs = match ttl {
            Ttl::Secs(s) => s,
            Ttl::Never => u64::MAX,
        };

        let id = self.next_id;
        self.next_id = self.next_id.saturating_add(1);

        let entry = Entry {
            vec,
            utterance: utterance.to_owned(),
            response: response.to_owned(),
            stored_at: now_secs,
            ttl_secs,
        };

        self.entries.insert(id, entry);
        self.lru.push(id);

        // Enforce capacity bound: evict LRU entries until within limit.
        self.evict_to_capacity();

        Ok(())
    }

    /// Return a snapshot of deflection metrics (hits + total lookups).
    #[must_use]
    pub const fn metrics(&self) -> &CacheMetrics {
        &self.metrics
    }

    /// Number of live (non-expired) entries currently in the cache.
    #[must_use]
    pub fn len(&self) -> usize {
        let now_secs = (self.config.clock.now_secs)();
        self.entries
            .values()
            .filter(|e| !e.is_expired(now_secs))
            .count()
    }

    /// Returns `true` if the cache contains no live entries.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Evict expired entries and enforce the capacity bound via LRU.
    fn evict_to_capacity(&mut self) {
        let now_secs = (self.config.clock.now_secs)();

        // First pass: remove expired entries from storage (LRU list cleaned lazily).
        self.entries.retain(|_, e| !e.is_expired(now_secs));

        // Second pass: LRU eviction while over capacity.
        while self.entries.len() > self.config.capacity {
            if let Some(evict_id) = self.lru.pop_lru() {
                self.entries.remove(&evict_id);
            } else {
                // LRU list exhausted (shouldn't happen, but be safe).
                break;
            }
        }

        // Synchronise: remove IDs from LRU that were evicted above.
        self.lru.retain(|id| self.entries.contains_key(id));
    }
}
