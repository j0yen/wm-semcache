//! AC5: Max-entry bound enforced with LRU eviction; exceeding the bound evicts
//! the least-recently-hit entry.
//!
//! Read-only: the edit-agent must NOT modify acceptance tests.

mod common;
use common::MappedEmbed;
use wm_semcache::{SemCache, SemCacheConfig, Ttl};

/// Fill the cache to capacity + 1 and verify the LRU (first inserted, never
/// accessed again) entry is evicted, while the MRU entries remain.
#[test]
fn lru_eviction_on_capacity_overflow() {
    // Build a mapped embedder with 3 orthogonal utterances.
    let utterances: &[(&str, Vec<f32>)] = &[
        ("first", vec![1.0, 0.0, 0.0]),
        ("second", vec![0.0, 1.0, 0.0]),
        ("third", vec![0.0, 0.0, 1.0]),
    ];
    let embedder = MappedEmbed {
        map: utterances
            .iter()
            .map(|(k, v)| (*k, v.clone()))
            .collect(),
        default: vec![0.5, 0.5, 0.0],
    };

    // Capacity = 2 so inserting 3 entries forces an eviction.
    let config = SemCacheConfig {
        similarity_threshold: 0.85,
        capacity: 2,
        ..SemCacheConfig::default()
    };
    let mut cache = SemCache::new(config, Box::new(embedder));

    // Insert "first" — now at LRU position.
    cache
        .store("first", "response-first", true, Ttl::Never)
        .expect("store first");

    // Insert "second".
    cache
        .store("second", "response-second", true, Ttl::Never)
        .expect("store second");

    // Touch "first" to promote it to MRU, making "second" the new LRU.
    let _ = cache.lookup("first");

    // Insert "third" — capacity exceeded; "second" (LRU) should be evicted.
    cache
        .store("third", "response-third", true, Ttl::Never)
        .expect("store third");

    // "first" and "third" must still be present.
    assert!(
        cache.lookup("first").is_some(),
        "first (MRU) should survive eviction"
    );
    assert!(
        cache.lookup("third").is_some(),
        "third (just inserted) should survive eviction"
    );

    // "second" (LRU at the time of the third insert) must have been evicted.
    assert!(
        cache.lookup("second").is_none(),
        "second (LRU) should have been evicted"
    );
}

/// The cache never holds more than `capacity` live entries.
#[test]
fn cache_len_never_exceeds_capacity() {
    let mut map_entries: Vec<(&'static str, Vec<f32>)> = Vec::new();
    // Use unique, orthogonal-ish vectors for 20 distinct utterances.
    let labels: &[&str] = &[
        "a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k", "l", "m", "n", "o", "p", "q",
        "r", "s", "t",
    ];
    for (i, &label) in labels.iter().enumerate() {
        #[allow(clippy::cast_precision_loss, clippy::as_conversions)]
        let mut v = vec![0.0_f32; 20];
        v[i] = 1.0;
        map_entries.push((label, v));
    }
    let embedder = MappedEmbed {
        map: map_entries.clone(),
        default: vec![0.0; 20],
    };

    let cap = 5_usize;
    let config = SemCacheConfig {
        similarity_threshold: 0.85,
        capacity: cap,
        ..SemCacheConfig::default()
    };
    let mut cache = SemCache::new(config, Box::new(embedder));

    for (label, _) in &map_entries {
        cache
            .store(label, "response", true, Ttl::Never)
            .expect("store should succeed");
    }

    assert!(
        cache.len() <= cap,
        "cache len {} exceeds capacity {}",
        cache.len(),
        cap
    );
}
