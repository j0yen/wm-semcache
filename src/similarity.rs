//! Cosine similarity between two f32 vectors.

/// Compute the cosine similarity between `a` and `b`.
///
/// Both vectors are L2-normalised internally before computing the dot product,
/// so the result is always in `[-1.0, 1.0]`.
///
/// Returns `0.0` if either vector has zero magnitude (zero vector has no
/// direction → no similarity).
#[must_use]
pub(crate) fn cosine(a: &[f32], b: &[f32]) -> f32 {
    if a.is_empty() || b.is_empty() || a.len() != b.len() {
        return 0.0_f32;
    }

    let norm_a = l2_norm(a);
    let norm_b = l2_norm(b);

    if norm_a < f32::EPSILON || norm_b < f32::EPSILON {
        return 0.0_f32;
    }

    #[allow(clippy::float_arithmetic)]
    let dot: f32 = a.iter().zip(b.iter()).map(|(&x, &y)| x * y).sum();
    #[allow(clippy::float_arithmetic)]
    let result = (dot / (norm_a * norm_b)).clamp(-1.0_f32, 1.0_f32);
    result
}

fn l2_norm(v: &[f32]) -> f32 {
    #[allow(clippy::float_arithmetic)]
    v.iter().map(|&x| x * x).sum::<f32>().sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identical_vectors_score_one() {
        let v = vec![0.6, 0.8];
        let s = cosine(&v, &v);
        assert!((s - 1.0_f32).abs() < 1e-5_f32);
    }

    #[test]
    fn orthogonal_vectors_score_zero() {
        let a = vec![1.0, 0.0];
        let b = vec![0.0, 1.0];
        let s = cosine(&a, &b);
        assert!(s.abs() < 1e-5_f32);
    }

    #[test]
    fn zero_vector_returns_zero() {
        let a = vec![0.0, 0.0];
        let b = vec![1.0, 0.0];
        assert_eq!(cosine(&a, &b), 0.0_f32);
    }

    #[test]
    fn different_lengths_return_zero() {
        let a = vec![1.0, 0.0];
        let b = vec![1.0];
        assert_eq!(cosine(&a, &b), 0.0_f32);
    }

    #[test]
    fn high_similarity_near_paraphrase() {
        // Two nearly-identical vectors should score close to 1.
        let a = vec![0.9, 0.1, 0.05];
        let b = vec![0.88, 0.12, 0.04];
        assert!(cosine(&a, &b) > 0.99_f32);
    }
}
