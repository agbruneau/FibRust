//! # fibcalc-calibration
//!
//! Auto-tuning, adaptive benchmarks, and calibration profiles.

pub mod adaptive;
pub mod calibration;
pub mod io;
pub mod microbench;
pub mod profile;
pub mod runner;

pub use calibration::{CalibrationEngine, CalibrationMode};
pub use profile::CalibrationProfile;
