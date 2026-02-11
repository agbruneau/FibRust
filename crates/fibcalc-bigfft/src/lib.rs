//! # fibcalc-bigfft
//!
//! FFT-based big number multiplication using Fermat numbers.
//! Port of the Go `internal/bigfft` package.

pub mod allocator;
pub mod arith_generic;
pub mod bump;
pub mod fermat;
pub mod fft;
pub mod fft_cache;
pub mod fft_core;
pub mod fft_poly;
pub mod fft_recursion;
pub mod memory_est;
pub mod pool;
pub mod pool_warming;
pub mod scan;

// Re-exports
pub use fft::{mul, mul_to, sqr, sqr_to};
