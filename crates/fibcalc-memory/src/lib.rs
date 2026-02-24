//! # fibcalc-memory
//!
//! Unified memory management for the `FibCalc` workspace.
//!
//! Provides `BigUint` pooling with size classes, bump arenas for FFT temporaries,
//! generic thread-local object pools, and pool warming strategies.
#![warn(missing_docs)]

pub mod arena;
pub mod pool;
pub mod stats;
pub mod thread_local;
pub mod warming;
