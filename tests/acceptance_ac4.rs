//! AC4: TTL expiry — an entry past its ttl_secs is not served.
//! Uses a frozen/injected clock so no wall-clock flakiness.
//!
//! Read-only: the edit-agent must NOT modify acceptance tests.

mod common;
use common::FixedEmbed;
use wm_semcache::{ClockFn, SemCache, SemCacheConfig, Ttl};

/// Store an entry with a 60-second TTL. Advance the frozen clock past the
/// TTL and assert the entry is no longer served.
#[test]
fn expired_entry_not_served() {
    let embedder = FixedEmbed(vec![1.0, 0.0]);

    // Start clock at t=1000.
    let config = SemCacheConfig {
        similarity_threshold: 0.85,
        clock: ClockFn::frozen(1000),
        ..SemCacheConfig::default()
    };
    let mut cache = SemCache::new(config, Box::new(embedder));

    cache
        .store("Who wrote Hamlet?", "William Shakespeare.", true, Ttl::Secs(60))
        .expect("store should succeed");

    // At t=1000 the entry is live — should hit.
    let hit = cache.lookup("Who wrote Hamlet?");
    assert!(hit.is_some(), "entry should be live at t=1000");

    // Rebuild with clock at t=1061 (past TTL of 60s).
    let embedder2 = FixedEmbed(vec![1.0, 0.0]);
    let config2 = SemCacheConfig {
        similarity_threshold: 0.85,
        clock: ClockFn::frozen(1061),
        ..SemCacheConfig::default()
    };
    let mut cache2 = SemCache::new(config2, Box::new(embedder2));
    cache2
        .store("Who wrote Hamlet?", "William Shakespeare.", true, Ttl::Secs(60))
        .expect("store should succeed");

    // The entry was stored at the clock start (1061); we need to test expiry
    // by querying at a time > stored_at + ttl. We rebuild with a different
    // frozen clock that simulates the passage of time differently.
    //
    // Design: store at t=100, query at t=161 with ttl=60 → expired.
    let embedder3 = FixedEmbed(vec![1.0, 0.0]);
    let config3 = SemCacheConfig {
        similarity_threshold: 0.85,
        clock: ClockFn::frozen(100),
        ..SemCacheConfig::default()
    };
    let mut cache3 = SemCache::new(config3, Box::new(embedder3));
    cache3
        .store("Who wrote Hamlet?", "William Shakespeare.", true, Ttl::Secs(60))
        .expect("store should succeed");

    // Now swap the clock to t=161 for the lookup.
    // We do this by reconstructing with the same data but a later clock.
    // The entry was stored in cache3 at t=100; it expires at t=160.
    // To simulate the passage of time, we create a fresh cache with a
    // clock at t=161 and verify the entry is not served (it was never stored
    // in that cache, so miss → correct). Instead, we test via the Entry
    // struct directly through the public is_expired logic, which is
    // fully covered in src/entry.rs.
    //
    // For an integration-level TTL test that uses the full SemCache path,
    // we rely on the ClockFn to be swappable. The cleanest approach:
    // store and lookup happen on the SAME cache instance; the clock
    // function is a closure that we can make time-varying by using a
    // shared counter. We use std::sync::Arc<std::sync::Mutex<u64>>.
    let stored_time = std::sync::Arc::new(std::sync::Mutex::new(100_u64));
    let stored_time_clone = stored_time.clone();

    let embedder4 = FixedEmbed(vec![1.0, 0.0]);
    let config4 = SemCacheConfig {
        similarity_threshold: 0.85,
        clock: ClockFn {
            now_secs: Box::new(move || {
                *stored_time_clone
                    .lock()
                    .unwrap_or_else(|e| e.into_inner())
            }),
        },
        ..SemCacheConfig::default()
    };
    let mut cache4 = SemCache::new(config4, Box::new(embedder4));

    // Store at t=100.
    cache4
        .store("Who wrote Hamlet?", "William Shakespeare.", true, Ttl::Secs(60))
        .expect("store at t=100 should succeed");

    // Confirm it's live at t=100.
    assert!(
        cache4.lookup("Who wrote Hamlet?").is_some(),
        "entry should be live at t=100"
    );

    // Advance clock to t=161 (past the 60s TTL).
    *stored_time.lock().unwrap_or_else(|e| e.into_inner()) = 161;

    let result = cache4.lookup("Who wrote Hamlet?");
    assert!(
        result.is_none(),
        "entry should be expired at t=161 (stored at t=100, ttl=60s)"
    );
}

/// An entry with Ttl::Never must survive even at a far-future clock value.
#[test]
fn never_ttl_entry_never_expires() {
    let stored_time = std::sync::Arc::new(std::sync::Mutex::new(0_u64));
    let stored_time_clone = stored_time.clone();

    let embedder = FixedEmbed(vec![1.0, 0.0]);
    let config = SemCacheConfig {
        similarity_threshold: 0.85,
        clock: ClockFn {
            now_secs: Box::new(move || {
                *stored_time_clone
                    .lock()
                    .unwrap_or_else(|e| e.into_inner())
            }),
        },
        ..SemCacheConfig::default()
    };
    let mut cache = SemCache::new(config, Box::new(embedder));

    cache
        .store("Who wrote Hamlet?", "William Shakespeare.", true, Ttl::Never)
        .expect("store should succeed");

    // Advance clock to an absurdly large value.
    *stored_time.lock().unwrap_or_else(|e| e.into_inner()) = u64::MAX / 2;

    assert!(
        cache.lookup("Who wrote Hamlet?").is_some(),
        "Ttl::Never entry should survive any clock value"
    );
}
