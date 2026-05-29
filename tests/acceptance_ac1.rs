//! AC1: store then lookup of a paraphrase returns the cached response when
//! cosine >= threshold, using a deterministic stubbed embedder.
//!
//! Read-only: the edit-agent must NOT modify acceptance tests.

mod common;
use common::MappedEmbed;
use wm_semcache::{SemCache, SemCacheConfig, Ttl};

/// Two utterances that "mean the same thing" are given nearly-identical
/// vectors by the stub embedder. The second lookup must hit the entry
/// stored by the first.
#[test]
fn paraphrase_hit_returns_cached_response() {
    // "What did I have for breakfast?" and its paraphrase share a very
    // similar embedding vector (cosine ≈ 0.9999).
    let original = "What did I have for breakfast?";
    let paraphrase = "What was my breakfast this morning?";

    let vec_a: Vec<f32> = vec![0.6, 0.8, 0.0];
    let vec_b: Vec<f32> = vec![0.601, 0.799, 0.001]; // cosine ≈ 0.9999

    let embedder = MappedEmbed {
        map: vec![
            (original, vec_a.clone()),
            (paraphrase, vec_b.clone()),
        ],
        default: vec![0.0, 0.0, 1.0], // far from everything else
    };

    let config = SemCacheConfig {
        similarity_threshold: 0.95,
        ..SemCacheConfig::default()
    };
    let mut cache = SemCache::new(config, Box::new(embedder));

    // Store the canonical answer under the original utterance.
    cache
        .store(original, "You had oatmeal.", true, Ttl::Secs(3600))
        .expect("store should succeed");

    // Lookup via the paraphrase — must hit.
    let hit = cache.lookup(paraphrase).expect("paraphrase should hit the cache");
    assert_eq!(
        hit.response, "You had oatmeal.",
        "hit response should match stored response"
    );
    assert!(
        hit.similarity >= 0.95,
        "similarity {:.4} should be >= threshold 0.95",
        hit.similarity
    );
}
