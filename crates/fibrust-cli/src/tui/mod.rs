//! TUI module for FibRust - HTOP-inspired terminal interface.

pub mod app;
pub mod event;
pub mod state;
pub mod style;
pub mod terminal;
pub mod ui;
pub mod widgets;

pub use terminal::run_tui;
