//! Multiplication strategy traits and implementations.
//!
//! `Multiplier` is the narrow interface for multiply/square operations.
//! `DoublingStepExecutor` extends it for optimized Fast Doubling steps.
//! Strategies include Karatsuba, FFT, and adaptive selection.

use num_bigint::BigUint;
use rayon;

/// Narrow interface for multiplication operations (ISP).
pub trait Multiplier: Send + Sync {
    /// Multiply two big unsigned integers.
    fn multiply(&self, a: &BigUint, b: &BigUint) -> BigUint;

    /// Square a big unsigned integer (may be optimized over multiply).
    fn square(&self, a: &BigUint) -> BigUint {
        self.multiply(a, a)
    }

    /// Get the name of this multiplication strategy.
    fn name(&self) -> &str;
}

/// Extended interface for optimized Fast Doubling steps.
pub trait DoublingStepExecutor: Multiplier {
    /// Execute a complete doubling step: given F(k) and F(k+1),
    /// compute F(2k) and F(2k+1).
    ///
    /// Returns (F(2k), F(2k+1)).
    fn execute_doubling_step(&self, fk: &BigUint, fk1: &BigUint) -> (BigUint, BigUint);
}

/// Karatsuba multiplication strategy (default for small numbers).
pub struct KaratsubaStrategy;

impl KaratsubaStrategy {
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl Default for KaratsubaStrategy {
    fn default() -> Self {
        Self::new()
    }
}

impl Multiplier for KaratsubaStrategy {
    fn multiply(&self, a: &BigUint, b: &BigUint) -> BigUint {
        a * b
    }

    fn square(&self, a: &BigUint) -> BigUint {
        a * a
    }

    fn name(&self) -> &'static str {
        "Karatsuba"
    }
}

impl DoublingStepExecutor for KaratsubaStrategy {
    fn execute_doubling_step(&self, fk: &BigUint, fk1: &BigUint) -> (BigUint, BigUint) {
        // F(2k) = F(k) * (2*F(k+1) - F(k))  — 1 multiply
        let t = (fk1 << 1u32) - fk;
        let f2k = self.multiply(fk, &t);
        // F(2k+1) = F(k)^2 + F(k+1)^2       — 2 squarings
        let f2k1 = self.square(fk) + self.square(fk1);

        (f2k, f2k1)
    }
}

/// Parallel Karatsuba strategy that uses `rayon::join` to parallelize
/// the three independent multiplications in the doubling step when
/// operand bits exceed the parallel threshold.
pub struct ParallelKaratsubaStrategy {
    parallel_threshold: usize,
}

impl ParallelKaratsubaStrategy {
    #[must_use]
    pub fn new(parallel_threshold: usize) -> Self {
        Self { parallel_threshold }
    }
}

impl Multiplier for ParallelKaratsubaStrategy {
    fn multiply(&self, a: &BigUint, b: &BigUint) -> BigUint {
        a * b
    }

    fn square(&self, a: &BigUint) -> BigUint {
        a * a
    }

    fn name(&self) -> &'static str {
        "ParallelKaratsuba"
    }
}

impl DoublingStepExecutor for ParallelKaratsubaStrategy {
    fn execute_doubling_step(&self, fk: &BigUint, fk1: &BigUint) -> (BigUint, BigUint) {
        // F(2k) = F(k) * (2*F(k+1) - F(k))  — 1 multiply
        // F(2k+1) = F(k)^2 + F(k+1)^2       — 2 squarings
        let t = (fk1 << 1u32) - fk;
        let max_bits = fk.bits().max(fk1.bits()) as usize;

        if max_bits >= self.parallel_threshold {
            // Parallel: multiply and 2 squarings concurrently
            let ((fk_sq, fk1_sq), f2k) = rayon::join(
                || rayon::join(|| fk * fk, || fk1 * fk1),
                || fk * &t,
            );
            let f2k1 = fk_sq + fk1_sq;
            (f2k, f2k1)
        } else {
            // Sequential for small operands
            let f2k = self.multiply(fk, &t);
            let f2k1 = self.square(fk) + self.square(fk1);
            (f2k, f2k1)
        }
    }
}

/// FFT-only multiplication strategy (for very large numbers).
pub struct FFTOnlyStrategy;

impl FFTOnlyStrategy {
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl Default for FFTOnlyStrategy {
    fn default() -> Self {
        Self::new()
    }
}

impl Multiplier for FFTOnlyStrategy {
    fn multiply(&self, a: &BigUint, b: &BigUint) -> BigUint {
        fibcalc_bigfft::mul(a, b)
    }

    fn square(&self, a: &BigUint) -> BigUint {
        fibcalc_bigfft::sqr(a)
    }

    fn name(&self) -> &'static str {
        "FFT"
    }
}

impl DoublingStepExecutor for FFTOnlyStrategy {
    fn execute_doubling_step(&self, fk: &BigUint, fk1: &BigUint) -> (BigUint, BigUint) {
        // F(2k) = F(k) * (2*F(k+1) - F(k))  — 1 multiply
        let t = (fk1 << 1u32) - fk;
        let f2k = self.multiply(fk, &t);
        // F(2k+1) = F(k)^2 + F(k+1)^2       — 2 squarings
        let f2k1 = self.square(fk) + self.square(fk1);

        (f2k, f2k1)
    }
}

/// Adaptive strategy that selects multiplication method based on operand size.
pub struct AdaptiveStrategy {
    fft_threshold: usize,
    _strassen_threshold: usize,
}

impl AdaptiveStrategy {
    #[must_use]
    pub fn new(fft_threshold: usize, strassen_threshold: usize) -> Self {
        Self {
            fft_threshold,
            _strassen_threshold: strassen_threshold,
        }
    }

    /// Get the bit length of a `BigUint`.
    fn bit_len(n: &BigUint) -> usize {
        n.bits() as usize
    }
}

impl Multiplier for AdaptiveStrategy {
    fn multiply(&self, a: &BigUint, b: &BigUint) -> BigUint {
        let max_bits = Self::bit_len(a).max(Self::bit_len(b));
        if max_bits >= self.fft_threshold {
            fibcalc_bigfft::mul(a, b)
        } else {
            a * b
        }
    }

    fn square(&self, a: &BigUint) -> BigUint {
        let bits = Self::bit_len(a);
        if bits >= self.fft_threshold {
            fibcalc_bigfft::sqr(a)
        } else {
            a * a
        }
    }

    fn name(&self) -> &'static str {
        "Adaptive"
    }
}

impl DoublingStepExecutor for AdaptiveStrategy {
    fn execute_doubling_step(&self, fk: &BigUint, fk1: &BigUint) -> (BigUint, BigUint) {
        // F(2k) = F(k) * (2*F(k+1) - F(k))  — 1 multiply
        let t = (fk1 << 1u32) - fk;
        let f2k = self.multiply(fk, &t);
        // F(2k+1) = F(k)^2 + F(k+1)^2       — 2 squarings
        let f2k1 = self.square(fk) + self.square(fk1);

        (f2k, f2k1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn karatsuba_multiply() {
        let strat = KaratsubaStrategy::new();
        let a = BigUint::from(12345u64);
        let b = BigUint::from(67890u64);
        assert_eq!(strat.multiply(&a, &b), BigUint::from(838_102_050u64));
    }

    #[test]
    fn karatsuba_square() {
        let strat = KaratsubaStrategy::new();
        let a = BigUint::from(1000u64);
        assert_eq!(strat.square(&a), BigUint::from(1_000_000u64));
    }

    #[test]
    fn karatsuba_doubling_step() {
        let strat = KaratsubaStrategy::new();
        // F(1) = 1, F(2) = 1 -> F(2) = 1, F(3) = 2
        let fk = BigUint::from(1u64);
        let fk1 = BigUint::from(1u64);
        let (f2k, f2k1) = strat.execute_doubling_step(&fk, &fk1);
        assert_eq!(f2k, BigUint::from(1u64)); // F(2) = 1
        assert_eq!(f2k1, BigUint::from(2u64)); // F(3) = 2
    }

    #[test]
    fn adaptive_strategy_name() {
        let strat = AdaptiveStrategy::new(500_000, 3072);
        assert_eq!(strat.name(), "Adaptive");
    }

    #[test]
    fn parallel_karatsuba_sequential_path() {
        // With a very high threshold, should take the sequential path
        let strat = ParallelKaratsubaStrategy::new(1_000_000);
        let fk = BigUint::from(1u64);
        let fk1 = BigUint::from(1u64);
        let (f2k, f2k1) = strat.execute_doubling_step(&fk, &fk1);
        assert_eq!(f2k, BigUint::from(1u64)); // F(2) = 1
        assert_eq!(f2k1, BigUint::from(2u64)); // F(3) = 2
    }

    #[test]
    fn parallel_karatsuba_parallel_path() {
        // With threshold=0, should always take the parallel path
        let strat = ParallelKaratsubaStrategy::new(0);
        let fk = BigUint::from(1u64);
        let fk1 = BigUint::from(1u64);
        let (f2k, f2k1) = strat.execute_doubling_step(&fk, &fk1);
        assert_eq!(f2k, BigUint::from(1u64)); // F(2) = 1
        assert_eq!(f2k1, BigUint::from(2u64)); // F(3) = 2
    }

    #[test]
    fn parallel_karatsuba_matches_karatsuba() {
        let seq = KaratsubaStrategy::new();
        let par = ParallelKaratsubaStrategy::new(0); // Force parallel path

        // Test with larger values to exercise the parallel code
        let fk = BigUint::parse_bytes(b"354224848179261915075", 10).unwrap();
        let fk1 = BigUint::parse_bytes(b"573147844013817084101", 10).unwrap();

        let (seq_f2k, seq_f2k1) = seq.execute_doubling_step(&fk, &fk1);
        let (par_f2k, par_f2k1) = par.execute_doubling_step(&fk, &fk1);

        assert_eq!(seq_f2k, par_f2k);
        assert_eq!(seq_f2k1, par_f2k1);
    }
}
