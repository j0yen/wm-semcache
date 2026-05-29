//! AC2: lookup of an unrelated utterance (cosine < threshold) returns None
//! (no false hit).
//!
//! Read-only: the edit-agent must NOT modify acceptance tests.

mod common;
use common::MappedEmbed;
use wm_semcache::{SemCache, SemCacheConfig, Ttl};

/// A completely unrelated utterance (orthogonal vector) must never produce
/// a cache hit.
#[test]
fn unrelated_utterance_returns_none() {
    let stored = "What did I have for breakfast?";
    let unrelated = "How do I bake bread?";

    // Orthogonal vectors → cosine = 0.0
    let vec_breakfast: Vec<f32> = vec![1.0, 0.0, 0.0];
    let vec_bread: Vec<f32> = vec![0.0, 1.0, 0.0];

    let embedder = MappedEmbed {
        map: vec![
            (stored, vec_breakfast.clone()),
            (unrelated, vec_bread.clone()),
        ],
        default: vec![0.0, 0.0, 1.0],
    };

    let config = SemCacheConfig {
        similarity_threshold: 0.85,
        ..SemCacheConfig::default()
    };
    let mut cache = SemCache::new(config, Box::new(embedder));

    cache
        .store(stored, "You had oatmeal.", true, Ttl::Secs(3600))
        .expect("store should succeed");

    let result = cache.lookup(unrelated);
    assert!(
        result.is_none(),
        "unrelated utterance should return None, not {:?}",
        result.map(|h| h.response)
    );
}
