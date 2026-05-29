//! Pluggable clock abstraction for deterministic TTL testing.

/// A function that returns the current time as seconds since the Unix epoch.
///
/// Use [`ClockFn::wall`] for production and [`ClockFn::frozen`] in tests.
pub struct ClockFn {
    /// The clock function.
    pub now_secs: Box<dyn Fn() -> u64 + Send + Sync>,
}

impl std::fmt::Debug for ClockFn {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ClockFn").finish_non_exhaustive()
    }
}

impl Clone for ClockFn {
    fn clone(&self) -> Self {
        // Cloning a ClockFn returns a wall clock; callers that need a frozen
        // clock must construct one explicitly.
        Self::wall()
    }
}

impl ClockFn {
    /// Returns the system wall clock.
    #[must_use]
    pub fn wall() -> Self {
        Self {
            now_secs: Box::new(|| {
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_secs())
                    .unwrap_or(0)
            }),
        }
    }

    /// Returns a clock frozen at `secs` seconds since the Unix epoch.
    ///
    /// Use in tests to make TTL expiry deterministic.
    #[must_use]
    pub fn frozen(secs: u64) -> Self {
        Self {
            now_secs: Box::new(move || secs),
        }
    }
}
