//! Pool warming: pre-allocate `BigInt` pools based on computation size.
//!
//! Predicts the number and sizes of `BigUint` values needed for a Fibonacci
//! computation of a given N, and pre-populates the pool to avoid cold-start
//! allocation overhead.

use crate::pool::BigIntPool;

/// Configuration for pool warming thresholds.
#[derive(Debug, Clone)]
pub struct WarmingConfig {
    /// Minimum N to trigger warming (below this, allocation is cheap).
    pub min_n: u64,
    /// Threshold for medium warming (more pre-allocations).
    pub medium_threshold: u64,
    /// Threshold for aggressive warming.
    pub large_threshold: u64,
}

impl Default for WarmingConfig {
    fn default() -> Self {
        Self {
            min_n: 1_000,
            medium_threshold: 100_000,
            large_threshold: 1_000_000,
        }
    }
}

/// Predicted sizes needed for a computation.
#[derive(Debug, Clone)]
pub struct SizePrediction {
    /// Estimated result bit-size: F(n) has approximately n * log2(phi) bits.
    pub result_bits: usize,
    /// Number of temporaries needed at each size class.
    pub allocations: Vec<(usize, usize)>, // (bits, count)
}

/// Predict the sizes needed for computing F(n).
///
/// F(n) has approximately n * 0.6942 bits (n * log2(phi)).
/// The fast doubling algorithm needs ~6 temporaries at the result size,
/// plus temporaries at intermediate sizes during the squaring loop.
#[must_use]
pub fn predict_sizes(n: u64) -> SizePrediction {
    let result_bits = estimate_result_bits(n);

    let mut allocations = Vec::new();

    if result_bits <= 64 {
        // Small computation, minimal warming needed
        allocations.push((64, 2));
    } else if result_bits <= 10_000 {
        // Medium computation
        allocations.push((result_bits, 6)); // Full-size temporaries
        allocations.push((result_bits / 2, 4)); // Half-size intermediates
    } else {
        // Large computation: warm multiple size classes
        allocations.push((result_bits, 8)); // Full-size
        allocations.push((result_bits / 2, 6)); // Half-size
        allocations.push((result_bits / 4, 4)); // Quarter-size
    }

    SizePrediction {
        result_bits,
        allocations,
    }
}

/// Estimate the number of bits in F(n).
///
/// F(n) ~ phi^n / sqrt(5), so log2(F(n)) ~ n * log2(phi) ~ n * 0.6942.
#[must_use]
#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss
)]
pub fn estimate_result_bits(n: u64) -> usize {
    if n <= 1 {
        return 1;
    }
    // log2(phi) ≈ 0.69424191363
    let bits = (n as f64 * 0.694_241_913_63).ceil() as usize;
    bits.max(1)
}

/// Warm a pool based on the predicted sizes for computing F(n).
///
/// Uses the warming config to decide how aggressively to pre-allocate.
pub fn warm_pool(pool: &BigIntPool, n: u64, config: &WarmingConfig) {
    if n < config.min_n {
        return; // Too small to bother warming
    }

    let prediction = predict_sizes(n);

    // Scale allocations based on thresholds
    let scale = if n >= config.large_threshold {
        2 // Double the predicted allocations for large computations
    } else {
        1 // Use predicted allocations as-is
    };

    for (bits, count) in &prediction.allocations {
        let actual_count = (*count) * scale;
        pool.warm(*bits, actual_count);
    }
}

/// Warm a pool with default configuration.
pub fn warm_pool_default(pool: &BigIntPool, n: u64) {
    warm_pool(pool, n, &WarmingConfig::default());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn estimate_result_bits_small() {
        assert_eq!(estimate_result_bits(0), 1);
        assert_eq!(estimate_result_bits(1), 1);
        // F(10) = 55, which is 6 bits. Estimate: ceil(10 * 0.6942) = 7
        assert!(estimate_result_bits(10) >= 6);
        assert!(estimate_result_bits(10) <= 8);
    }

    #[test]
    fn estimate_result_bits_medium() {
        // F(1000) has 209 digits ≈ 694 bits. Estimate: ceil(1000 * 0.6942) = 695
        let est = estimate_result_bits(1000);
        assert!(est >= 690);
        assert!(est <= 700);
    }

    #[test]
    fn estimate_result_bits_large() {
        // F(1_000_000) should have ~694242 bits
        let est = estimate_result_bits(1_000_000);
        assert!(est >= 694_000);
        assert!(est <= 695_000);
    }

    #[test]
    fn predict_sizes_small() {
        let prediction = predict_sizes(10);
        assert_eq!(prediction.result_bits, estimate_result_bits(10));
        assert!(!prediction.allocations.is_empty());
    }

    #[test]
    fn predict_sizes_medium() {
        let prediction = predict_sizes(10_000);
        assert!(prediction.result_bits > 6000);
        // Should have multiple size classes
        assert!(prediction.allocations.len() >= 2);
    }

    #[test]
    fn predict_sizes_large() {
        let prediction = predict_sizes(1_000_000);
        assert!(prediction.result_bits > 600_000);
        // Should have 3 size classes for large computations
        assert_eq!(prediction.allocations.len(), 3);
    }

    #[test]
    fn warm_pool_below_threshold() {
        let pool = BigIntPool::default();
        warm_pool_default(&pool, 100); // Below min_n
        assert_eq!(pool.total_pooled(), 0);
    }

    #[test]
    fn warm_pool_medium() {
        let pool = BigIntPool::default();
        warm_pool_default(&pool, 10_000);
        assert!(pool.total_pooled() > 0);
    }

    #[test]
    fn warm_pool_large() {
        let pool = BigIntPool::default();
        warm_pool_default(&pool, 1_000_000);
        // Large computation should pre-allocate more
        assert!(pool.total_pooled() > 0);
    }

    #[test]
    fn warm_pool_custom_config() {
        let pool = BigIntPool::default();
        let config = WarmingConfig {
            min_n: 10,
            medium_threshold: 1_000,
            large_threshold: 10_000,
        };
        warm_pool(&pool, 100, &config);
        assert!(pool.total_pooled() > 0);
    }

    #[test]
    fn warming_config_default() {
        let config = WarmingConfig::default();
        assert_eq!(config.min_n, 1_000);
        assert_eq!(config.medium_threshold, 100_000);
        assert_eq!(config.large_threshold, 1_000_000);
    }
}
