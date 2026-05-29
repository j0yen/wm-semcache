//! AC3 (CARDINAL SAFETY): an utterance classified cache-unsafe returns None
//! from lookup and is rejected by store, even when an identical entry exists.
//!
//! Read-only: the edit-agent must NOT modify acceptance tests.

mod common;
use common::FixedEmbed;
use wm_semcache::{SemCache, SemCacheConfig, SemCacheError, Ttl};

/// Pre-seed a "what time is it" entry via direct manipulation, then prove
/// that lookup and store both refuse it.
#[test]
fn cache_unsafe_utterance_never_served() {
    // Use a fixed embedder so every utterance has the same vector.
    let embedder = FixedEmbed(vec![1.0, 0.0]);
    let config = SemCacheConfig {
        similarity_threshold: 0.85,
        ..SemCacheConfig::default()
    };
    let mut cache = SemCache::new(config, Box::new(embedder));

    // Attempt to store a cache-unsafe utterance — must be rejected.
    let store_result = cache.store(
        "What time is it?",
        "It is 3pm.",
        true, // caller says cache_safe=true, but gate should still reject
        Ttl::Secs(60),
    );
    assert!(
        matches!(store_result, Err(SemCacheError::CacheUnsafe)),
        "store of cache-unsafe utterance should return CacheUnsafe error, got {:?}",
        store_result
    );

    // Also explicitly test with cache_safe=false.
    let store_result2 = cache.store(
        "What time is it?",
        "It is 3pm.",
        false,
        Ttl::Secs(60),
    );
    assert!(
        matches!(store_result2, Err(SemCacheError::CacheUnsafe)),
        "store with cache_safe=false should return CacheUnsafe"
    );

    // Lookup must also return None for cache-unsafe utterances.
    let hit = cache.lookup("What time is it?");
    assert!(
        hit.is_none(),
        "lookup of cache-unsafe utterance must return None, got {:?}",
        hit.map(|h| h.response)
    );

    // Variants of the same intent must also be rejected.
    assert!(
        cache.lookup("Do you know the current time?").is_none(),
        "time variant must also be None"
    );
    assert!(
        cache.lookup("What's the weather today?").is_none(),
        "weather query must be None"
    );
    assert!(
        cache.lookup("Show my calendar").is_none(),
        "calendar query must be None"
    );
}

/// A safe utterance stored normally must still be served correctly, proving
/// the gate is not over-broad.
#[test]
fn safe_utterance_is_served_after_store() {
    let embedder = FixedEmbed(vec![1.0, 0.0]);
    let config = SemCacheConfig {
        similarity_threshold: 0.85,
        ..SemCacheConfig::default()
    };
    let mut cache = SemCache::new(config, Box::new(embedder));

    cache
        .store("Who wrote Hamlet?", "William Shakespeare.", true, Ttl::Secs(3600))
        .expect("safe store should succeed");

    let hit = cache.lookup("Who wrote Hamlet?");
    assert!(hit.is_some(), "safe utterance should hit the cache");
    assert_eq!(hit.unwrap().response, "William Shakespeare.");
}
