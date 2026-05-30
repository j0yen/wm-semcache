//! AC7: Deflection metric (hits / total_lookups) is exposed so the cache's
//! cost saving is measurable.
//!
//! Read-only: the edit-agent must NOT modify acceptance tests.

mod common;
use common::FixedEmbed;
use wm_semcache::{SemCache, SemCacheConfig, Ttl};

/// After a mix of hits and misses, the metric counters reflect reality.
#[test]
fn metrics_track_hits_and_lookups() {
    let embedder = FixedEmbed(vec![1.0, 0.0]);
    let mut cache = SemCache::new(SemCacheConfig::default(), Box::new(embedder));

    // Start: no lookups, hit-rate = 0.
    assert_eq!(cache.metrics().total_lookups, 0);
    assert_eq!(cache.metrics().hits, 0);
    assert!((cache.metrics().hit_rate() - 0.0_f64).abs() < f64::EPSILON);

    // Store one entry.
    cache
        .store("Who wrote Hamlet?", "William Shakespeare.", true, Ttl::Never)
        .expect("store should succeed");

    // Hit lookup.
    let hit = cache.lookup("Who wrote Hamlet?");
    assert!(hit.is_some());
    assert_eq!(cache.metrics().total_lookups, 1);
    assert_eq!(cache.metrics().hits, 1);
    assert!((cache.metrics().hit_rate() - 1.0_f64).abs() < f64::EPSILON);

    // Miss lookup (cache-unsafe utterance counts as a lookup but not a hit).
    let _ = cache.lookup("What time is it?");
    assert_eq!(cache.metrics().total_lookups, 2);
    assert_eq!(cache.metrics().hits, 1);

    // hit_rate should now be 0.5.
    let rate = cache.metrics().hit_rate();
    assert!(
        (rate - 0.5_f64).abs() < 1e-9_f64,
        "hit_rate should be 0.5, got {rate}"
    );
}

/// The hit rate is 0.0 when there are zero lookups (no divide-by-zero).
#[test]
fn hit_rate_is_zero_with_no_lookups() {
    let embedder = FixedEmbed(vec![1.0, 0.0]);
    let cache = SemCache::new(SemCacheConfig::default(), Box::new(embedder));
    assert!((cache.metrics().hit_rate() - 0.0_f64).abs() < f64::EPSILON);
}
