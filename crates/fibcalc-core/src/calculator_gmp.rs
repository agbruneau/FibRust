//! GMP-based calculator using the `rug` crate.
//!
//! Only available when the `gmp` feature is enabled.
//! Implements Fast Doubling using `rug::Integer` for hardware-accelerated
//! big-integer arithmetic, converting to `BigUint` at the API boundary.

#[cfg(feature = "gmp")]
mod inner {
    use num_bigint::BigUint;
    use rug::Integer;

    use crate::calculator::{CoreCalculator, FibError};
    use crate::observer::ProgressObserver;
    use crate::options::Options;
    use crate::progress::{CancellationToken, ProgressUpdate};

    /// GMP-backed Fibonacci calculator using `rug::Integer`.
    ///
    /// Uses the Fast Doubling algorithm with GMP's optimized arithmetic.
    /// Results are converted to `BigUint` at the API boundary via string
    /// conversion, keeping rug types internal.
    pub struct GmpCalculator;

    impl GmpCalculator {
        /// Create a new `GmpCalculator`.
        #[must_use]
        pub fn new() -> Self {
            Self
        }

        /// Convert a `rug::Integer` to `num_bigint::BigUint`.
        ///
        /// # Panics
        ///
        /// Panics if the integer is negative (should never happen for Fibonacci).
        fn to_biguint(value: &Integer) -> BigUint {
            value
                .to_string()
                .parse::<BigUint>()
                .expect("rug::Integer should produce a valid decimal string")
        }

        /// Execute the Fast Doubling loop using `rug::Integer`.
        #[allow(clippy::cast_possible_truncation)]
        fn execute_doubling_loop(
            &self,
            n: u64,
            cancel: &CancellationToken,
            observer: &dyn ProgressObserver,
            calc_index: usize,
        ) -> Result<Integer, FibError> {
            let num_bits = 64 - n.leading_zeros();

            let mut fk = Integer::from(0);
            let mut fk1 = Integer::from(1);
            let mut t = Integer::new();
            let mut f2k = Integer::new();
            let mut f2k1 = Integer::new();

            let frozen = observer.freeze();

            for i in (0..num_bits).rev() {
                if cancel.is_cancelled() {
                    return Err(FibError::Cancelled);
                }

                // t = 2*fk1 - fk
                t.assign(&fk1 * 2);
                t -= &fk;

                // f2k = fk * t
                f2k.assign(&fk * &t);

                // f2k1 = fk^2 + fk1^2
                f2k1.assign(fk.square_ref());
                f2k1 += Integer::from(fk1.square_ref());

                fk.assign(&f2k);
                fk1.assign(&f2k1);

                // Conditional addition step
                if (n >> i) & 1 == 1 {
                    std::mem::swap(&mut fk, &mut fk1);
                    fk1 += &fk;
                }

                // Progress reporting
                let progress = 1.0 - (f64::from(i) / f64::from(num_bits));
                if frozen.should_report(progress) {
                    frozen.update(progress);
                    observer.on_progress(&ProgressUpdate::new(
                        calc_index,
                        "GMP",
                        progress,
                        u64::from(num_bits - i),
                        u64::from(num_bits),
                    ));
                }
            }

            Ok(fk)
        }
    }

    impl Default for GmpCalculator {
        fn default() -> Self {
            Self::new()
        }
    }

    impl CoreCalculator for GmpCalculator {
        fn calculate_core(
            &self,
            cancel: &CancellationToken,
            observer: &dyn ProgressObserver,
            calc_index: usize,
            n: u64,
            _opts: &Options,
        ) -> Result<BigUint, FibError> {
            let result = self.execute_doubling_loop(n, cancel, observer, calc_index)?;

            observer.on_progress(&ProgressUpdate::done(calc_index, "GMP"));

            Ok(Self::to_biguint(&result))
        }

        fn name(&self) -> &'static str {
            "GMP"
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use crate::observers::NoOpObserver;

        fn compute_fib(n: u64) -> BigUint {
            let calc = GmpCalculator::new();
            let cancel = CancellationToken::new();
            let observer = NoOpObserver::new();
            let opts = Options::default();
            calc.calculate_core(&cancel, &observer, 0, n, &opts)
                .unwrap()
        }

        #[test]
        fn gmp_f94() {
            assert_eq!(
                compute_fib(94),
                BigUint::parse_bytes(b"19740274219868223167", 10).unwrap()
            );
        }

        #[test]
        fn gmp_f100() {
            assert_eq!(
                compute_fib(100),
                BigUint::parse_bytes(b"354224848179261915075", 10).unwrap()
            );
        }

        #[test]
        fn gmp_f1000_digit_count() {
            let f1000 = compute_fib(1000);
            let s = f1000.to_string();
            assert!(s.starts_with("43466557686937456435688527675040625802564"));
            assert_eq!(s.len(), 209);
        }

        #[test]
        fn gmp_agrees_with_fast_doubling() {
            use crate::fastdoubling::OptimizedFastDoubling;

            let gmp = GmpCalculator::new();
            let fd = OptimizedFastDoubling::new();
            let cancel = CancellationToken::new();
            let observer = NoOpObserver::new();
            let opts = Options::default();

            for n in [94, 100, 200, 500, 1000] {
                let gmp_result = gmp
                    .calculate_core(&cancel, &observer, 0, n, &opts)
                    .unwrap();
                let fd_result = fd
                    .calculate_core(&cancel, &observer, 0, n, &opts)
                    .unwrap();
                assert_eq!(gmp_result, fd_result, "mismatch at F({n})");
            }
        }

        #[test]
        fn gmp_cancellation() {
            let calc = GmpCalculator::new();
            let cancel = CancellationToken::new();
            cancel.cancel();
            let observer = NoOpObserver::new();
            let opts = Options::default();
            let result = calc.calculate_core(&cancel, &observer, 0, 10000, &opts);
            assert!(matches!(result, Err(FibError::Cancelled)));
        }
    }
}

#[cfg(feature = "gmp")]
pub use inner::GmpCalculator;
