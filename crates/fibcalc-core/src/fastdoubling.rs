//! Optimized Fast Doubling algorithm for Fibonacci computation.
//!
//! Uses the doubling identities:
//!   F(2k)   = 2*F(k)*F(k+1) - F(k)^2
//!   F(2k+1) = F(k+1)^2 + F(k)^2
//!
//! Iterates from MSB to LSB with zero-copy pointer rotation.
//! Includes thread-local pooling of `CalculationState` objects.

use std::cell::RefCell;

use fibcalc_bigfft::{mul, sqr};
use num_bigint::BigUint;
use num_traits::{One, Zero};

use crate::calculator::{CoreCalculator, FibError};
use crate::observer::ProgressObserver;
use crate::options::Options;
use crate::pool;
use crate::progress::{CancellationToken, ProgressUpdate};

/// State for the Fast Doubling computation, enabling pool reuse.
pub struct CalculationState {
    /// Current F(k).
    pub fk: BigUint,
    /// Current F(k+1).
    pub fk1: BigUint,
    /// Temporary register 1.
    pub t1: BigUint,
    /// Temporary register 2.
    pub t2: BigUint,
    /// Temporary register 3.
    pub t3: BigUint,
}

impl CalculationState {
    /// Create a new calculation state initialized for F(0)=0, F(1)=1.
    #[must_use]
    pub fn new() -> Self {
        Self {
            fk: BigUint::ZERO,
            fk1: BigUint::from(1u32),
            t1: BigUint::ZERO,
            t2: BigUint::ZERO,
            t3: BigUint::ZERO,
        }
    }

    /// Reset state for reuse.
    pub fn reset(&mut self) {
        self.fk.set_zero();
        self.fk1.set_one();
        self.t1.set_zero();
        self.t2.set_zero();
        self.t3.set_zero();
    }
}

impl Default for CalculationState {
    fn default() -> Self {
        Self::new()
    }
}

thread_local! {
    static CALC_STATE_POOL: RefCell<Vec<CalculationState>> = const { RefCell::new(Vec::new()) };
}

const THREAD_LOCAL_POOL_MAX: usize = 4;

/// Acquire a `CalculationState` from the thread-local pool.
fn tl_acquire_state() -> CalculationState {
    CALC_STATE_POOL.with(|p| {
        pool::tl_acquire(
            p,
            CalculationState::new,
            CalculationState::reset,
        )
    })
}

/// Return a `CalculationState` to the thread-local pool.
fn tl_release_state(state: CalculationState) {
    CALC_STATE_POOL.with(|p| pool::tl_release(p, THREAD_LOCAL_POOL_MAX, state));
}

/// Optimized Fast Doubling calculator.
///
/// # Example
/// ```
/// use fibcalc_core::fastdoubling::OptimizedFastDoubling;
/// use fibcalc_core::calculator::CoreCalculator;
/// use fibcalc_core::observers::NoOpObserver;
/// use fibcalc_core::options::Options;
/// use fibcalc_core::progress::CancellationToken;
///
/// let calc = OptimizedFastDoubling::new();
/// let cancel = CancellationToken::new();
/// let observer = NoOpObserver::new();
/// let opts = Options::default();
/// let result = calc.calculate_core(&cancel, &observer, 0, 100, &opts).unwrap();
/// assert_eq!(result.to_string(), "354224848179261915075");
/// ```
pub struct OptimizedFastDoubling;

impl OptimizedFastDoubling {
    /// Create a new `OptimizedFastDoubling` calculator.
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    /// Execute the doubling loop.
    #[allow(clippy::cast_possible_truncation, clippy::unused_self)]
    fn execute_doubling_loop(
        &self,
        n: u64,
        cancel: &CancellationToken,
        observer: &dyn ProgressObserver,
        calc_index: usize,
        opts: &Options,
    ) -> Result<BigUint, FibError> {
        let num_bits = 64 - n.leading_zeros();
        let mut state = tl_acquire_state();

        let frozen = observer.freeze();
        // Inline strategy logic to reuse CalculationState buffers

        let result = (|| {
            for i in (0..num_bits).rev() {
                // Cancellation checkpoint
                if cancel.is_cancelled() {
                    return Err(FibError::Cancelled);
                }

                // Doubling step: compute F(2k) and F(2k+1)
                // t = (fk1 << 1) - fk
                // Reuse state.t1 for t to avoid allocation
                state.t1.clone_from(&state.fk1);
                state.t1 <<= 1;
                state.t1 -= &state.fk;

                let max_bits = state.fk.bits().max(state.fk1.bits()) as usize;
                let use_fft = max_bits >= opts.fft_threshold;

                let (f2k, f2k1) = if max_bits >= opts.parallel_threshold {
                    // Parallel: multiply and 2 squarings concurrently
                    let ((fk_sq, fk1_sq), f2k) = rayon::join(
                        || {
                            rayon::join(
                                || {
                                    if use_fft {
                                        sqr(&state.fk)
                                    } else {
                                        &state.fk * &state.fk
                                    }
                                },
                                || {
                                    if use_fft {
                                        sqr(&state.fk1)
                                    } else {
                                        &state.fk1 * &state.fk1
                                    }
                                },
                            )
                        },
                        || {
                            if use_fft {
                                mul(&state.fk, &state.t1)
                            } else {
                                &state.fk * &state.t1
                            }
                        },
                    );
                    (f2k, fk_sq + fk1_sq)
                } else {
                    // Sequential for small operands
                    let f2k = if use_fft {
                        mul(&state.fk, &state.t1)
                    } else {
                        &state.fk * &state.t1
                    };
                    let fk_sq = if use_fft {
                        sqr(&state.fk)
                    } else {
                        &state.fk * &state.fk
                    };
                    let fk1_sq = if use_fft {
                        sqr(&state.fk1)
                    } else {
                        &state.fk1 * &state.fk1
                    };
                    (f2k, fk_sq + fk1_sq)
                };

                state.fk = f2k;
                state.fk1 = f2k1;

                // Conditional addition step
                if (n >> i) & 1 == 1 {
                    // F(2k+1) = F(2k) + F(2k+1) -> become new F(2k+2)
                    // F(2k) -> become F(2k+1)

                    // Swap ensures fk holds old_fk1 (new F(2k+1))
                    std::mem::swap(&mut state.fk, &mut state.fk1);
                    // Add ensures fk1 holds old_fk + old_fk1 (new F(2k+2))
                    state.fk1 += &state.fk;
                }

                // Progress reporting
                let progress = 1.0 - (f64::from(i) / f64::from(num_bits));
                if frozen.should_report(progress) {
                    frozen.update(progress);
                    observer.on_progress(&ProgressUpdate::new(
                        calc_index,
                        "FastDoubling",
                        progress,
                        u64::from(num_bits - i),
                        u64::from(num_bits),
                    ));
                }
            }

            // Zero-copy result extraction
            Ok(std::mem::take(&mut state.fk))
        })();

        // Return state to pool regardless of success/failure
        tl_release_state(state);

        result
    }
}

impl Default for OptimizedFastDoubling {
    fn default() -> Self {
        Self::new()
    }
}

impl CoreCalculator for OptimizedFastDoubling {
    fn calculate_core(
        &self,
        cancel: &CancellationToken,
        observer: &dyn ProgressObserver,
        calc_index: usize,
        n: u64,
        opts: &Options,
    ) -> Result<BigUint, FibError> {
        let result = self.execute_doubling_loop(n, cancel, observer, calc_index, opts)?;

        // Send completion
        observer.on_progress(&ProgressUpdate::done(calc_index, "FastDoubling"));

        Ok(result)
    }

    fn name(&self) -> &'static str {
        "FastDoubling"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::observers::NoOpObserver;

    fn compute_fib(n: u64) -> BigUint {
        let calc = OptimizedFastDoubling::new();
        let cancel = CancellationToken::new();
        let observer = NoOpObserver::new();
        let opts = Options::default();
        calc.calculate_core(&cancel, &observer, 0, n, &opts)
            .unwrap()
    }

    #[test]
    fn fast_doubling_small_values() {
        // These go through the core calculator (not the fast path)
        assert_eq!(
            compute_fib(94),
            BigUint::parse_bytes(b"19740274219868223167", 10).unwrap()
        );
        assert_eq!(
            compute_fib(100),
            BigUint::parse_bytes(b"354224848179261915075", 10).unwrap()
        );
    }

    #[test]
    fn fast_doubling_known_values() {
        // F(200) = 280571172992510140037611932413038677189525
        let f200 = compute_fib(200);
        let expected =
            BigUint::parse_bytes(b"280571172992510140037611932413038677189525", 10).unwrap();
        assert_eq!(f200, expected);
    }

    #[test]
    fn fast_doubling_f1000() {
        let f1000 = compute_fib(1000);
        // F(1000) starts with "43466557686937456435688527675040625802564"
        let s = f1000.to_string();
        assert!(s.starts_with("43466557686937456435688527675040625802564"));
        assert_eq!(s.len(), 209); // F(1000) has 209 digits
    }

    #[test]
    fn fast_doubling_cancellation() {
        let calc = OptimizedFastDoubling::new();
        let cancel = CancellationToken::new();
        cancel.cancel();
        let observer = NoOpObserver::new();
        let opts = Options::default();
        let result = calc.calculate_core(&cancel, &observer, 0, 10000, &opts);
        assert!(matches!(result, Err(FibError::Cancelled)));
    }

    #[test]
    fn calculation_state_reset() {
        let mut state = CalculationState::new();
        state.fk = BigUint::from(42u32);
        state.reset();
        assert_eq!(state.fk, BigUint::ZERO);
        assert_eq!(state.fk1, BigUint::from(1u32));
    }

    #[test]
    fn thread_local_pool_reuse() {
        // First computation populates the thread-local pool
        let f100a = compute_fib(100);
        // Second computation reuses from pool
        let f100b = compute_fib(100);
        assert_eq!(f100a, f100b);
    }

    #[test]
    fn thread_local_pool_acquire_release() {
        let state = tl_acquire_state();
        assert_eq!(state.fk, BigUint::ZERO);
        tl_release_state(state);

        // Should get it back from pool
        let state2 = tl_acquire_state();
        assert_eq!(state2.fk, BigUint::ZERO);
        assert_eq!(state2.fk1, BigUint::from(1u32));
        tl_release_state(state2);
    }
}
