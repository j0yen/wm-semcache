//! Lightweight LRU-order tracker using a `Vec` as an ordered list.
//!
//! `u64` entry IDs are appended on insert and moved to the back on access.
//! The front of the list is the least-recently-used entry.
//!
//! This is O(n) on touch; acceptable for the expected cache sizes (≤10 000
//! entries). A `VecDeque`-based implementation with a hash-index could
//! provide O(1) touch but would require a second crate dep.

/// Ordered list of entry IDs for LRU tracking.
///
/// Invariant: each ID appears at most once.
#[derive(Debug, Default)]
pub(crate) struct LruOrder {
    /// Front = LRU, back = MRU.
    order: Vec<u64>,
}

impl LruOrder {
    /// Create a new, empty order tracker.
    #[must_use]
    pub(crate) fn new() -> Self {
        Self::default()
    }

    /// Push a newly inserted entry to the MRU position.
    pub(crate) fn push(&mut self, id: u64) {
        // Remove any stale entry for the same ID (shouldn't happen, but be safe).
        self.order.retain(|&x| x != id);
        self.order.push(id);
    }

    /// Promote `id` to the MRU position (call on a cache hit).
    pub(crate) fn touch(&mut self, id: u64) {
        self.order.retain(|&x| x != id);
        self.order.push(id);
    }

    /// Remove and return the least-recently-used entry ID, or `None` if empty.
    pub(crate) fn pop_lru(&mut self) -> Option<u64> {
        if self.order.is_empty() {
            None
        } else {
            Some(self.order.remove(0))
        }
    }

    /// Remove IDs that no longer exist in the entry map.
    pub(crate) fn retain<F>(&mut self, mut keep: F)
    where
        F: FnMut(&u64) -> bool,
    {
        self.order.retain(|id| keep(id));
    }

    /// Number of IDs tracked.
    #[must_use]
    #[allow(dead_code)]
    pub(crate) fn len(&self) -> usize {
        self.order.len()
    }

    /// Returns `true` if there are no IDs tracked.
    #[must_use]
    #[allow(dead_code)]
    pub(crate) fn is_empty(&self) -> bool {
        self.order.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn push_pop_order() {
        let mut lru = LruOrder::new();
        lru.push(1);
        lru.push(2);
        lru.push(3);
        assert_eq!(lru.pop_lru(), Some(1));
        assert_eq!(lru.pop_lru(), Some(2));
        assert_eq!(lru.pop_lru(), Some(3));
        assert_eq!(lru.pop_lru(), None);
    }

    #[test]
    fn touch_promotes_to_mru() {
        let mut lru = LruOrder::new();
        lru.push(1);
        lru.push(2);
        lru.push(3);
        lru.touch(1); // 1 was LRU; after touch it should be MRU
        // Now LRU order: 2, 3, 1
        assert_eq!(lru.pop_lru(), Some(2));
        assert_eq!(lru.pop_lru(), Some(3));
        assert_eq!(lru.pop_lru(), Some(1));
    }
}
