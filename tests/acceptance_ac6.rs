//! AC6: Embedder is dim-agnostic (works with any vector length) and degrades
//! safely when recall's embed socket is unreachable — lookup returns None,
//! never panics.
//!
//! Read-only: the edit-agent must NOT modify acceptance tests.

mod common;
use common::{FailingEmbed, FixedEmbed};
use wm_semcache::{SemCache, SemCacheConfig, Ttl};

/// With a 384-dim stub embedder (fastembed dimension), store + lookup works.
#[test]
fn works_with_384_dim_vectors() {
    let vec384 = vec![1.0_f32 / (384.0_f32.sqrt()); 384];
    let embedder = FixedEmbed(vec384);
    let mut cache = SemCache::new(SemCacheConfig::default(), Box::new(embedder));

    cache
        .store("hello", "hi", true, Ttl::Secs(300))
        .expect("store 384-dim should succeed");
    assert!(cache.lookup("hello").is_some(), "384-dim lookup should hit");
}

/// With a 256-dim stub embedder (hash embedder dimension), store + lookup works.
#[test]
fn works_with_256_dim_vectors() {
    let vec256 = vec![1.0_f32 / (256.0_f32.sqrt()); 256];
    let embedder = FixedEmbed(vec256);
    let mut cache = SemCache::new(SemCacheConfig::default(), Box::new(embedder));

    cache
        .store("hello", "hi", true, Ttl::Secs(300))
        .expect("store 256-dim should succeed");
    assert!(cache.lookup("hello").is_some(), "256-dim lookup should hit");
}

/// When the embedder is unreachable, lookup returns None and does NOT panic.
#[test]
fn unreachable_embedder_returns_none_on_lookup() {
    let embedder = FailingEmbed;
    let mut cache = SemCache::new(SemCacheConfig::default(), Box::new(embedder));

    // The cache is empty, but even if it had entries the unreachable embedder
    // must cause a graceful None rather than a panic.
    let result = cache.lookup("What did I have for breakfast?");
    assert!(
        result.is_none(),
        "unreachable embedder must yield None, not panic"
    );
}

/// When the embedder fails during store, the error is surfaced, not swallowed.
#[test]
fn unreachable_embedder_errors_on_store() {
    let embedder = FailingEmbed;
    let mut cache = SemCache::new(SemCacheConfig::default(), Box::new(embedder));

    let result = cache.store("hello", "hi", true, Ttl::Secs(300));
    assert!(
        result.is_err(),
        "store with unreachable embedder must return an error"
    );
}
