//! # fibcalc-cli
//!
//! CLI output, progress display, and shell completion.

pub mod completion;
pub mod output;
pub mod presenter;
pub mod progress_eta;
pub mod ui;

pub use presenter::{CLIProgressReporter, CLIResultPresenter};
