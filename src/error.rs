//! Error types for wm-semcache.

/// Error returned when embedding an utterance fails.
#[derive(Debug, thiserror::Error)]
pub enum EmbedError {
    /// The embedding service (e.g. recall's embed RPC) is unreachable.
    #[error("embedding service unreachable: {0}")]
    Unreachable(String),
    /// The embedder returned an unexpected or empty vector.
    #[error("invalid embedding response: {0}")]
    InvalidResponse(String),
}

/// Error returned by [`crate::SemCache::store`].
#[derive(Debug, thiserror::Error)]
pub enum SemCacheError {
    /// The utterance or caller flag indicates this response must not be cached.
    #[error("utterance is cache-unsafe and cannot be stored")]
    CacheUnsafe,
    /// The embedder failed while trying to store the entry.
    #[error("embed failed during store: {0}")]
    EmbedFailed(#[from] EmbedError),
}
