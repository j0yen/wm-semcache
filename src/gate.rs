//! Cache-unsafe intent gate.
//!
//! Any utterance that matches one of the unsafe intent keywords is classified
//! as cache-unsafe. The classification is conservative: a false positive
//! (routing a safe answer through the skill tier) costs one API call; a false
//! negative (serving a stale time/weather answer as truth) harms the user.

/// Keywords whose presence anywhere in a lowercased utterance marks it
/// cache-unsafe.
///
/// This list is the v1 hardcoded unsafe-intent set. It is intentionally
/// narrow — adding keywords here only reduces cache hit-rate, never
/// introduces safety risk.
pub const UNSAFE_INTENTS: &[&str] = &[
    // Time / date
    "what time",
    "what's the time",
    "whats the time",
    "what is the time",
    "current time",
    "time is it",
    "time now",
    "what day",
    "today's date",
    "todays date",
    "what date",
    "current date",
    "date today",
    "day today",
    "day is it",
    "date is it",
    // Weather
    "weather",
    "forecast",
    "temperature outside",
    "temp outside",
    "raining",
    "snowing",
    // Calendar / reminders
    "calendar",
    "appointments",
    "appointment today",
    "schedule today",
    "reminder",
    "reminders",
    "what do i have today",
    "what's on today",
    "whats on today",
];

/// Returns `true` if `utterance` should never be served from the cache.
///
/// Case-insensitive prefix/substring match against [`UNSAFE_INTENTS`].
#[must_use]
pub(crate) fn is_cache_unsafe(utterance: &str) -> bool {
    let lower = utterance.to_lowercase();
    UNSAFE_INTENTS.iter().any(|kw| lower.contains(kw))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn time_question_is_unsafe() {
        assert!(is_cache_unsafe("What time is it?"));
        assert!(is_cache_unsafe("Do you know what time it is?"));
        assert!(is_cache_unsafe("WHAT TIME IS IT"));
    }

    #[test]
    fn weather_is_unsafe() {
        assert!(is_cache_unsafe("What's the weather like?"));
        assert!(is_cache_unsafe("Tell me the forecast"));
    }

    #[test]
    fn calendar_is_unsafe() {
        assert!(is_cache_unsafe("Show my calendar"));
        assert!(is_cache_unsafe("What are my reminders?"));
    }

    #[test]
    fn factual_question_is_safe() {
        assert!(!is_cache_unsafe("Who wrote Hamlet?"));
        assert!(!is_cache_unsafe("What is the capital of France?"));
        assert!(!is_cache_unsafe("How do I make tea?"));
    }
}
