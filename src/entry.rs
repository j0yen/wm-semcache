//! A single cache entry.

/// A stored utterance–response pair with its embedding vector and TTL.
#[derive(Debug, Clone)]
pub struct Entry {
    /// L2-normalised embedding vector of `utterance`.
    pub vec: Vec<f32>,
    /// The original utterance that was stored.
    pub utterance: String,
    /// The response associated with `utterance`.
    pub response: String,
    /// Unix timestamp (seconds) at which this entry was stored.
    pub stored_at: u64,
    /// Time-to-live in seconds. `u64::MAX` means never expires.
    pub ttl_secs: u64,
}

impl Entry {
    /// Returns `true` if this entry has expired relative to `now_secs`.
    #[must_use]
    pub const fn is_expired(&self, now_secs: u64) -> bool {
        if self.ttl_secs == u64::MAX {
            return false;
        }
        now_secs.saturating_sub(self.stored_at) >= self.ttl_secs
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_entry(stored_at: u64, ttl_secs: u64) -> Entry {
        Entry {
            vec: vec![1.0],
            utterance: "test".into(),
            response: "resp".into(),
            stored_at,
            ttl_secs,
        }
    }

    #[test]
    fn not_expired_within_ttl() {
        let e = make_entry(1000, 300);
        assert!(!e.is_expired(1200));
    }

    #[test]
    fn expired_past_ttl() {
        let e = make_entry(1000, 300);
        assert!(e.is_expired(1300));
    }

    #[test]
    fn never_expires() {
        let e = make_entry(0, u64::MAX);
        assert!(!e.is_expired(u64::MAX));
    }
}
