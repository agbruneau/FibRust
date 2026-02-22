//! FFT-based Fibonacci calculator.
//!
//! Uses Fast Doubling with FFT multiplication for very large numbers.

use num_bigint::BigUint;

use crate::calculator::{CoreCalculator, FibError};
use crate::constants::DEFAULT_FFT_THRESHOLD;
use crate::observer::ProgressObserver;
use crate::options::Options;
use crate::progress::{CancellationToken, ProgressUpdate};
use crate::strategy::{AdaptiveStrategy, DoublingStepExecutor};

/// FFT-based Fibonacci calculator.
///
/// Uses the Fast Doubling framework but with FFT multiplication
/// for operands exceeding the FFT threshold.
pub struct FFTBasedCalculator;

impl FFTBasedCalculator {
    /// Create a new FFT-based Fibonacci calculator.
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl Default for FFTBasedCalculator {
    fn default() -> Self {
        Self::new()
    }
}

impl CoreCalculator for FFTBasedCalculator {
    fn calculate_core(
        &self,
        cancel: &CancellationToken,
        observer: &dyn ProgressObserver,
        calc_index: usize,
        n: u64,
        _opts: &Options,
    ) -> Result<BigUint, FibError> {
        // Use the same doubling loop structure as FastDoubling,
        // but with FFT multiplication for large operands.
        let num_bits = 64 - n.leading_zeros();
        let mut fk = BigUint::ZERO;
        let mut fk1 = BigUint::from(1u32);

        let frozen = observer.freeze();
        let strategy = AdaptiveStrategy::new(DEFAULT_FFT_THRESHOLD);

        for i in (0..num_bits).rev() {
            if cancel.is_cancelled() {
                return Err(FibError::Cancelled);
            }

            // Doubling step with FFT multiplication for large operands
            let (f2k, f2k1) = strategy.execute_doubling_step(&fk, &fk1);
            fk = f2k;
            fk1 = f2k1;

            // Conditional addition
            if (n >> i) & 1 == 1 {
                let sum = &fk + &fk1;
                fk = std::mem::replace(&mut fk1, sum);
            }

            // Progress reporting
            let progress = 1.0 - (f64::from(i) / f64::from(num_bits));
            if frozen.should_report(progress) {
                frozen.update(progress);
                observer.on_progress(&ProgressUpdate::new(
                    calc_index,
                    "FFTBased",
                    progress,
                    u64::from(num_bits - i),
                    u64::from(num_bits),
                ));
            }
        }

        observer.on_progress(&ProgressUpdate::done(calc_index, "FFTBased"));
        Ok(fk)
    }

    fn name(&self) -> &'static str {
        "FFTBased"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::observers::NoOpObserver;

    fn compute_fib(n: u64) -> BigUint {
        let calc = FFTBasedCalculator::new();
        let cancel = CancellationToken::new();
        let observer = NoOpObserver::new();
        let opts = Options::default();
        calc.calculate_core(&cancel, &observer, 0, n, &opts)
            .unwrap()
    }

    #[test]
    fn fft_small_values() {
        assert_eq!(
            compute_fib(100),
            BigUint::parse_bytes(b"354224848179261915075", 10).unwrap()
        );
    }

    #[test]
    fn fft_f200() {
        let f200 = compute_fib(200);
        let expected =
            BigUint::parse_bytes(b"280571172992510140037611932413038677189525", 10).unwrap();
        assert_eq!(f200, expected);
    }

    #[test]
    fn fft_matches_fast_doubling() {
        use crate::fastdoubling::OptimizedFastDoubling;
        let fd = OptimizedFastDoubling::new();
        let fft = FFTBasedCalculator::new();
        let cancel = CancellationToken::new();
        let observer = NoOpObserver::new();
        let opts = Options::default();

        for n in [100, 200, 500, 1000] {
            let fd_result = fd.calculate_core(&cancel, &observer, 0, n, &opts).unwrap();
            let fft_result = fft.calculate_core(&cancel, &observer, 0, n, &opts).unwrap();
            assert_eq!(fd_result, fft_result, "Mismatch at n={n}");
        }
    }
}
