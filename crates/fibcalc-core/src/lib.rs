//! # fibcalc-core
//!
//! Core library for the FibCalc-rs high-performance Fibonacci calculator.
//! Implements Fast Doubling, Matrix Exponentiation, and FFT-based algorithms.

pub(crate) mod arena;
pub mod calculator;
pub(crate) mod common;
pub mod constants;
pub(crate) mod doubling_framework;
pub mod dynamic_threshold;
pub mod fastdoubling;
pub mod fft_based;
pub(crate) mod fft_wrappers;
pub(crate) mod generator;
pub(crate) mod generator_iterative;
pub mod iterator;
pub mod matrix;
pub(crate) mod matrix_framework;
pub(crate) mod matrix_ops;
pub(crate) mod matrix_types;
pub mod memory_budget;
pub mod modular;
pub mod observer;
pub mod observers;
pub mod options;
pub mod progress;
pub mod registry;
pub mod strategy;
pub(crate) mod threshold_types;

#[cfg(feature = "gmp")]
pub mod calculator_gmp;

// Re-exports
pub use calculator::{Calculator, CoreCalculator, FibCalculator};
pub use constants::{
    exit_codes, DEFAULT_FFT_THRESHOLD, DEFAULT_PARALLEL_THRESHOLD, DEFAULT_STRASSEN_THRESHOLD,
    FIB_TABLE, MAX_FIB_U64, PROGRESS_REPORT_THRESHOLD,
};
pub use observer::{ProgressObserver, ProgressSubject};
pub use options::Options;
pub use progress::ProgressUpdate;
pub use registry::{CalculatorFactory, DefaultFactory};
pub use strategy::{DoublingStepExecutor, Multiplier};

use num_bigint::BigUint;

/// Compute F(n) using the fast doubling algorithm.
///
/// This is a convenience function for simple use cases. For advanced
/// configuration (progress, cancellation, memory limits), use the
/// `Calculator` trait directly.
///
/// # Example
/// ```
/// assert_eq!(fibcalc_core::fibonacci(10).to_string(), "55");
/// assert_eq!(fibcalc_core::fibonacci(0).to_string(), "0");
/// ```
#[must_use]
pub fn fibonacci(n: u64) -> BigUint {
    use calculator::Calculator;
    use fastdoubling::OptimizedFastDoubling;
    use observers::NoOpObserver;
    use progress::CancellationToken;

    let calc = FibCalculator::new(std::sync::Arc::new(OptimizedFastDoubling::new()));
    let cancel = CancellationToken::new();
    let observer = NoOpObserver::new();
    let opts = Options::default();
    calc.calculate(&cancel, &observer, 0, n, &opts)
        .expect("fast doubling should not fail for valid input")
}
