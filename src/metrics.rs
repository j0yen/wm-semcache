//! Deflection / diagnostic metrics for [`crate::SemCache`].

/// Counters exposed by [`crate::SemCache::metrics`].
///
/// The hit rate is `hits as f64 / total_lookups as f64`.
#[derive(Debug, Clone, Default)]
pub struct CacheMetrics {
    /// Total number of [`crate::SemCache::lookup`] calls, including misses and
    /// cache-unsafe bypass calls.
    pub total_lookups: u64,
    /// Number of lookups that returned a [`crate::CachedHit`].
    pub hits: u64,
}

impl CacheMetrics {
    /// Compute the hit rate as a float in `[0.0, 1.0]`.
    ///
    /// Returns `0.0` when `total_lookups` is zero.
    #[must_use]
    pub fn hit_rate(&self) -> f64 {
        if self.total_lookups == 0 {
            0.0
        } else {
            #[allow(clippy::cast_precision_loss, clippy::as_conversions)]
            let hits = self.hits as f64;
            #[allow(clippy::cast_precision_loss, clippy::as_conversions)]
            let total = self.total_lookups as f64;
            #[allow(clippy::float_arithmetic)]
            {
                hits / total
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hit_rate_zero_on_empty() {
        let m = CacheMetrics::default();
        assert!((m.hit_rate() - 0.0_f64).abs() < f64::EPSILON);
    }

    #[test]
    fn hit_rate_one_when_all_hits() {
        let m = CacheMetrics {
            total_lookups: 5,
            hits: 5,
        };
        assert!((m.hit_rate() - 1.0_f64).abs() < f64::EPSILON);
    }
}
