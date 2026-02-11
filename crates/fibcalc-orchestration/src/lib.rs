//! # fibcalc-orchestration
//!
//! Parallel execution, calculator selection, and result analysis.

pub mod calculator_selection;
pub mod interfaces;
pub mod orchestrator;

pub use interfaces::{ProgressReporter, ResultPresenter};
pub use orchestrator::{analyze_comparison_results, execute_calculations};
