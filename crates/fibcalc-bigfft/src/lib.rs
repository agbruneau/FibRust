//! # fibcalc-bigfft
//!
//! FFT-based big number multiplication using Fermat numbers.
//! Port of the Go `internal/bigfft` package.
#![warn(missing_docs)]
// Crate-level #![allow(dead_code)] removed -- individual modules/items annotated instead

pub mod allocator;
pub(crate) mod arith_generic;
pub(crate) mod bump;
pub(crate) mod fermat;
pub(crate) mod fft;
pub(crate) mod fft_cache;
pub(crate) mod fft_core;
pub(crate) mod fft_poly;
pub(crate) mod fft_recursion;
pub(crate) mod memory_est;
pub mod pool;
pub(crate) mod pool_warming;
pub(crate) mod scan;

// Re-exports
pub use fft::{mul, mul_to, sqr, sqr_to};
pub use fft::pool_stats;
pub use memory_est::estimate_fft_memory;
