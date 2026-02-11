//! Calculator traits and the `FibCalculator` decorator.
//!
//! `Calculator` is the public trait consumed by orchestration.
//! `CoreCalculator` is the internal trait implemented by algorithms.
//! `FibCalculator` is a decorator that adds the fast path (n <= 93) and progress reporting.

use std::sync::Arc;

use num_bigint::BigUint;

use crate::constants::{FIB_TABLE, MAX_FIB_U64};
use crate::observer::ProgressObserver;
use crate::options::Options;
use crate::progress::{CancellationToken, ProgressUpdate};

/// Error type for Fibonacci calculations.
#[derive(Debug, thiserror::Error)]
pub enum FibError {
    /// A calculation error occurred.
    #[error("calculation error: {0}")]
    Calculation(String),

    /// Configuration error.
    #[error("configuration error: {0}")]
    Config(String),

    /// Calculation was cancelled.
    #[error("calculation cancelled")]
    Cancelled,

    /// Calculation timed out.
    #[error("calculation timed out after {0}")]
    Timeout(String),

    /// Results from different algorithms don't match.
    #[error("result mismatch between algorithms")]
    Mismatch,
}

/// Public trait for Fibonacci calculators, consumed by orchestration.
pub trait Calculator: Send + Sync {
    /// Calculate F(n) with the given options.
    fn calculate(
        &self,
        cancel: &CancellationToken,
        observer: &dyn ProgressObserver,
        calc_index: usize,
        n: u64,
        opts: &Options,
    ) -> Result<BigUint, FibError>;

    /// Get the name of this calculator.
    fn name(&self) -> &str;
}

/// Internal trait for algorithm implementations.
/// Wrapped by `FibCalculator` which adds fast path and progress reporting.
pub trait CoreCalculator: Send + Sync {
    /// Perform the core calculation for large n.
    fn calculate_core(
        &self,
        cancel: &CancellationToken,
        observer: &dyn ProgressObserver,
        calc_index: usize,
        n: u64,
        opts: &Options,
    ) -> Result<BigUint, FibError>;

    /// Get the name of this algorithm.
    fn name(&self) -> &str;
}

/// Decorator that wraps a `CoreCalculator` with fast path and progress reporting.
pub struct FibCalculator {
    inner: Arc<dyn CoreCalculator>,
}

impl FibCalculator {
    /// Create a new `FibCalculator` wrapping the given core calculator.
    #[must_use]
    pub fn new(inner: Arc<dyn CoreCalculator>) -> Self {
        Self { inner }
    }

    /// Fast path for small n (n <= 93) using precomputed table.
    fn calculate_small(n: u64) -> BigUint {
        BigUint::from(FIB_TABLE[n as usize])
    }
}

impl Calculator for FibCalculator {
    fn calculate(
        &self,
        cancel: &CancellationToken,
        observer: &dyn ProgressObserver,
        calc_index: usize,
        n: u64,
        opts: &Options,
    ) -> Result<BigUint, FibError> {
        // Fast path for small n
        if n <= MAX_FIB_U64 {
            observer.on_progress(&ProgressUpdate::done(calc_index, self.inner.name()));
            return Ok(Self::calculate_small(n));
        }

        // Check cancellation before starting
        if cancel.is_cancelled() {
            return Err(FibError::Cancelled);
        }

        // Delegate to core algorithm
        self.inner
            .calculate_core(cancel, observer, calc_index, n, opts)
    }

    fn name(&self) -> &str {
        self.inner.name()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn calculate_small_values() {
        assert_eq!(FibCalculator::calculate_small(0), BigUint::from(0u64));
        assert_eq!(FibCalculator::calculate_small(1), BigUint::from(1u64));
        assert_eq!(FibCalculator::calculate_small(10), BigUint::from(55u64));
        assert_eq!(FibCalculator::calculate_small(20), BigUint::from(6765u64));
    }

    #[test]
    fn calculate_small_max() {
        assert_eq!(
            FibCalculator::calculate_small(93),
            BigUint::from(12_200_160_415_121_876_738u64)
        );
    }

    #[test]
    fn fib_error_display() {
        let err = FibError::Calculation("test".into());
        assert_eq!(err.to_string(), "calculation error: test");

        let err = FibError::Cancelled;
        assert_eq!(err.to_string(), "calculation cancelled");
    }
}
