//! Optimized Fast Doubling algorithm for Fibonacci computation.
//!
//! Uses the doubling identities:
//!   F(2k)   = 2*F(k)*F(k+1) - F(k)^2
//!   F(2k+1) = F(k+1)^2 + F(k)^2
//!
//! Iterates from MSB to LSB with zero-copy pointer rotation.
//! Includes thread-local pooling of `CalculationState` objects.

use std::cell::RefCell;

use num_bigint::BigUint;
use parking_lot::Mutex;

use crate::calculator::{CoreCalculator, FibError};
use crate::observer::ProgressObserver;
use crate::options::Options;
use crate::progress::{CancellationToken, ProgressUpdate};
use crate::strategy::{DoublingStepExecutor, ParallelKaratsubaStrategy};

/// State for the Fast Doubling computation, enabling pool reuse.
pub struct CalculationState {
    pub fk: BigUint,
    pub fk1: BigUint,
    pub t1: BigUint,
    pub t2: BigUint,
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
        self.fk = BigUint::ZERO;
        self.fk1 = BigUint::from(1u32);
        self.t1 = BigUint::ZERO;
        self.t2 = BigUint::ZERO;
        self.t3 = BigUint::ZERO;
    }
}

impl Default for CalculationState {
    fn default() -> Self {
        Self::new()
    }
}

/// Thread-local pool of `CalculationState` objects.
///
/// Each thread maintains a small stack of pre-allocated states to avoid
/// repeated allocation in hot loops. The pool uses Mutex for thread-safety
/// when accessed from Rayon work-stealing threads.
pub struct CalculationStatePool {
    pool: Mutex<Vec<CalculationState>>,
    max_size: usize,
}

impl CalculationStatePool {
    /// Create a new pool with the given maximum size.
    #[must_use]
    pub fn new(max_size: usize) -> Self {
        Self {
            pool: Mutex::new(Vec::with_capacity(max_size)),
            max_size,
        }
    }

    /// Acquire a `CalculationState` from the pool, or create a new one.
    /// The returned state is always reset and ready for use.
    pub fn acquire(&self) -> CalculationState {
        let mut pool = self.pool.lock();
        match pool.pop() {
            Some(mut state) => {
                state.reset();
                state
            }
            None => CalculationState::new(),
        }
    }

    /// Return a `CalculationState` to the pool for reuse.
    pub fn release(&self, state: CalculationState) {
        let mut pool = self.pool.lock();
        if pool.len() < self.max_size {
            pool.push(state);
        }
    }

    /// Get the number of states currently in the pool.
    #[must_use]
    pub fn available(&self) -> usize {
        self.pool.lock().len()
    }
}

impl Default for CalculationStatePool {
    fn default() -> Self {
        Self::new(4)
    }
}

thread_local! {
    static CALC_STATE_POOL: RefCell<Vec<CalculationState>> = const { RefCell::new(Vec::new()) };
}

const THREAD_LOCAL_POOL_MAX: usize = 4;

/// Acquire a `CalculationState` from the thread-local pool.
fn tl_acquire_state() -> CalculationState {
    CALC_STATE_POOL.with(|pool| {
        let mut pool = pool.borrow_mut();
        match pool.pop() {
            Some(mut state) => {
                state.reset();
                state
            }
            None => CalculationState::new(),
        }
    })
}

/// Return a `CalculationState` to the thread-local pool.
fn tl_release_state(state: CalculationState) {
    CALC_STATE_POOL.with(|pool| {
        let mut pool = pool.borrow_mut();
        if pool.len() < THREAD_LOCAL_POOL_MAX {
            pool.push(state);
        }
    });
}

/// Optimized Fast Doubling calculator.
pub struct OptimizedFastDoubling;

impl OptimizedFastDoubling {
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    /// Execute the doubling loop.
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
        let strategy = ParallelKaratsubaStrategy::new(opts.parallel_threshold);

        let result = (|| {
            for i in (0..num_bits).rev() {
                // Cancellation checkpoint
                if cancel.is_cancelled() {
                    return Err(FibError::Cancelled);
                }

                // Doubling step: compute F(2k) and F(2k+1)
                let (f2k, f2k1) = strategy.execute_doubling_step(&state.fk, &state.fk1);

                state.fk = f2k;
                state.fk1 = f2k1;

                // Conditional addition step
                if (n >> i) & 1 == 1 {
                    let sum = &state.fk + &state.fk1;
                    state.fk = std::mem::replace(&mut state.fk1, sum);
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
    fn calculation_state_pool_acquire_release() {
        let pool = CalculationStatePool::new(2);
        assert_eq!(pool.available(), 0);

        let state = pool.acquire();
        assert_eq!(state.fk, BigUint::ZERO);
        assert_eq!(state.fk1, BigUint::from(1u32));

        pool.release(state);
        assert_eq!(pool.available(), 1);

        // Acquire returns the pooled state (reset)
        let state2 = pool.acquire();
        assert_eq!(state2.fk, BigUint::ZERO);
        assert_eq!(pool.available(), 0);
    }

    #[test]
    fn calculation_state_pool_max_size() {
        let pool = CalculationStatePool::new(1);
        let s1 = CalculationState::new();
        let s2 = CalculationState::new();
        pool.release(s1);
        pool.release(s2); // Should be dropped, pool is full
        assert_eq!(pool.available(), 1);
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
