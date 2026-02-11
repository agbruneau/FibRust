//! Matrix Exponentiation algorithm for Fibonacci computation.
//!
//! Computes F(n) via Q^n where Q = [[1,1],[1,0]].
//! Uses binary exponentiation (square-and-multiply).
//! Includes thread-local pooling of `MatrixState` objects.

use std::cell::RefCell;

use num_bigint::BigUint;
use parking_lot::Mutex;

use crate::calculator::{CoreCalculator, FibError};
use crate::matrix_ops::{matrix_multiply, matrix_square};
use crate::matrix_types::MatrixState;
use crate::observer::ProgressObserver;
use crate::options::Options;
use crate::progress::{CancellationToken, ProgressUpdate};

/// Thread-safe pool of `MatrixState` objects.
pub struct MatrixStatePool {
    pool: Mutex<Vec<MatrixState>>,
    max_size: usize,
}

impl MatrixStatePool {
    /// Create a new pool with the given maximum size.
    #[must_use]
    pub fn new(max_size: usize) -> Self {
        Self {
            pool: Mutex::new(Vec::with_capacity(max_size)),
            max_size,
        }
    }

    /// Acquire a `MatrixState` from the pool, or create a new one.
    /// The returned state is always reset and ready for use.
    pub fn acquire(&self) -> MatrixState {
        let mut pool = self.pool.lock();
        match pool.pop() {
            Some(mut state) => {
                state.reset();
                state
            }
            None => MatrixState::new(),
        }
    }

    /// Return a `MatrixState` to the pool for reuse.
    pub fn release(&self, state: MatrixState) {
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

impl Default for MatrixStatePool {
    fn default() -> Self {
        Self::new(4)
    }
}

thread_local! {
    static MATRIX_STATE_POOL: RefCell<Vec<MatrixState>> = const { RefCell::new(Vec::new()) };
}

const THREAD_LOCAL_POOL_MAX: usize = 4;

/// Acquire a `MatrixState` from the thread-local pool.
fn tl_acquire_state() -> MatrixState {
    MATRIX_STATE_POOL.with(|pool| {
        let mut pool = pool.borrow_mut();
        match pool.pop() {
            Some(mut state) => {
                state.reset();
                state
            }
            None => MatrixState::new(),
        }
    })
}

/// Return a `MatrixState` to the thread-local pool.
fn tl_release_state(state: MatrixState) {
    MATRIX_STATE_POOL.with(|pool| {
        let mut pool = pool.borrow_mut();
        if pool.len() < THREAD_LOCAL_POOL_MAX {
            pool.push(state);
        }
    });
}

/// Matrix Exponentiation calculator.
pub struct MatrixExponentiation;

impl MatrixExponentiation {
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    /// Execute the matrix exponentiation loop.
    fn execute_matrix_loop(
        &self,
        n: u64,
        cancel: &CancellationToken,
        observer: &dyn ProgressObserver,
        calc_index: usize,
    ) -> Result<BigUint, FibError> {
        let num_bits = 64 - n.leading_zeros();
        let mut state = tl_acquire_state();

        let frozen = observer.freeze();

        let result = (|| {
            // Binary exponentiation: square-and-multiply
            for i in (0..num_bits).rev() {
                if cancel.is_cancelled() {
                    return Err(FibError::Cancelled);
                }

                // Square the result
                state.result = matrix_square(&state.result);

                // Multiply by base if bit is set
                if (n >> i) & 1 == 1 {
                    state.result = matrix_multiply(&state.result, &state.base);
                }

                // Progress reporting
                let progress = 1.0 - (f64::from(i) / f64::from(num_bits));
                if frozen.should_report(progress) {
                    frozen.update(progress);
                    observer.on_progress(&ProgressUpdate::new(
                        calc_index,
                        "MatrixExponentiation",
                        progress,
                        u64::from(num_bits - i),
                        u64::from(num_bits),
                    ));
                }
            }

            // Extract F(n) = Q^n[0][1] (or [1][0])
            Ok(std::mem::take(&mut state.result.b))
        })();

        // Return state to pool regardless of success/failure
        tl_release_state(state);

        result
    }
}

impl Default for MatrixExponentiation {
    fn default() -> Self {
        Self::new()
    }
}

impl CoreCalculator for MatrixExponentiation {
    fn calculate_core(
        &self,
        cancel: &CancellationToken,
        observer: &dyn ProgressObserver,
        calc_index: usize,
        n: u64,
        _opts: &Options,
    ) -> Result<BigUint, FibError> {
        let result = self.execute_matrix_loop(n, cancel, observer, calc_index)?;
        observer.on_progress(&ProgressUpdate::done(calc_index, "MatrixExponentiation"));
        Ok(result)
    }

    fn name(&self) -> &'static str {
        "MatrixExponentiation"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::observers::NoOpObserver;

    fn compute_fib(n: u64) -> BigUint {
        let calc = MatrixExponentiation::new();
        let cancel = CancellationToken::new();
        let observer = NoOpObserver::new();
        let opts = Options::default();
        calc.calculate_core(&cancel, &observer, 0, n, &opts)
            .unwrap()
    }

    #[test]
    fn matrix_small_values() {
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
    fn matrix_f200() {
        let f200 = compute_fib(200);
        let expected =
            BigUint::parse_bytes(b"280571172992510140037611932413038677189525", 10).unwrap();
        assert_eq!(f200, expected);
    }

    #[test]
    fn matrix_f1000() {
        let f1000 = compute_fib(1000);
        let s = f1000.to_string();
        assert!(s.starts_with("43466557686937456435688527675040625802564"));
        assert_eq!(s.len(), 209);
    }

    #[test]
    fn matrix_cancellation() {
        let calc = MatrixExponentiation::new();
        let cancel = CancellationToken::new();
        cancel.cancel();
        let observer = NoOpObserver::new();
        let opts = Options::default();
        let result = calc.calculate_core(&cancel, &observer, 0, 10000, &opts);
        assert!(matches!(result, Err(FibError::Cancelled)));
    }

    #[test]
    fn matrix_state_pool_acquire_release() {
        let pool = MatrixStatePool::new(2);
        assert_eq!(pool.available(), 0);

        let state = pool.acquire();
        assert!(state.result.is_identity());

        pool.release(state);
        assert_eq!(pool.available(), 1);

        // Acquire returns the pooled state (reset)
        let state2 = pool.acquire();
        assert!(state2.result.is_identity());
        assert_eq!(pool.available(), 0);
    }

    #[test]
    fn matrix_state_pool_max_size() {
        let pool = MatrixStatePool::new(1);
        let s1 = MatrixState::new();
        let s2 = MatrixState::new();
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
        assert!(state.result.is_identity());
        tl_release_state(state);

        // Should get it back from pool
        let state2 = tl_acquire_state();
        assert!(state2.result.is_identity());
        tl_release_state(state2);
    }
}
