//! # fibcalc-core
//!
//! Core library for the FibCalc-rs high-performance Fibonacci calculator.
//! Implements Fast Doubling, Matrix Exponentiation, and FFT-based algorithms.

pub mod arena;
pub mod calculator;
pub mod common;
pub mod constants;
pub mod doubling_framework;
pub mod dynamic_threshold;
pub mod fastdoubling;
pub mod fft_based;
pub mod fft_wrappers;
pub mod generator;
pub mod generator_iterative;
pub mod matrix;
pub mod matrix_framework;
pub mod matrix_ops;
pub mod matrix_types;
pub mod memory_budget;
pub mod modular;
pub mod observer;
pub mod observers;
pub mod options;
pub mod progress;
pub mod registry;
pub mod strategy;
pub mod threshold_types;

#[cfg(feature = "gmp")]
pub mod calculator_gmp;

// Re-exports
pub use calculator::{Calculator, CoreCalculator, FibCalculator};
pub use constants::*;
pub use observer::{ProgressObserver, ProgressSubject};
pub use options::Options;
pub use progress::ProgressUpdate;
pub use registry::{CalculatorFactory, DefaultFactory};
pub use strategy::{DoublingStepExecutor, Multiplier};
