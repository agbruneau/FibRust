//! # fibcalc-tui
//!
//! Interactive TUI dashboard using ratatui with Elm architecture.

pub mod bridge;
pub mod chart;
pub mod footer;
pub mod header;
pub mod keymap;
pub mod logs;
pub mod messages;
pub mod metrics;
pub mod model;
pub mod sparkline;
pub mod styles;

pub use bridge::{TuiBridgeObserver, TUIProgressReporter, TUIResultPresenter};
pub use logs::LogScrollState;
pub use messages::{SystemMetrics, TuiMessage};
pub use metrics::MetricsCollector;
pub use model::TuiApp;
pub use sparkline::SparklineBuffer;
