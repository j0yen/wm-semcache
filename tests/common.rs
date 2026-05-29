//! Shared test helpers: deterministic stub embedder.

use wm_semcache::{EmbedError, Embedder};

/// A stub embedder that returns a fixed vector regardless of utterance.
pub struct FixedEmbed(pub Vec<f32>);

impl Embedder for FixedEmbed {
    fn embed(&self, _utterance: &str) -> Result<Vec<f32>, EmbedError> {
        Ok(self.0.clone())
    }
}

/// A stub embedder that maps utterances to specific vectors.
///
/// Unknown utterances fall back to the `default` vector.
pub struct MappedEmbed {
    pub map: Vec<(&'static str, Vec<f32>)>,
    pub default: Vec<f32>,
}

impl Embedder for MappedEmbed {
    fn embed(&self, utterance: &str) -> Result<Vec<f32>, EmbedError> {
        for (key, vec) in &self.map {
            if utterance == *key {
                return Ok(vec.clone());
            }
        }
        Ok(self.default.clone())
    }
}

/// A stub embedder that always returns an error (simulates unreachable socket).
pub struct FailingEmbed;

impl Embedder for FailingEmbed {
    fn embed(&self, _utterance: &str) -> Result<Vec<f32>, EmbedError> {
        Err(EmbedError::Unreachable("test socket closed".into()))
    }
}
