//! TUI widgets for FibRust.

pub mod comparison;
pub mod header;
pub mod input;
pub mod progress;
pub mod result;
pub mod system_info;

pub use comparison::render_comparison;
pub use header::render_header;
pub use input::render_input;
pub use progress::render_progress;
pub use result::render_result;
pub use system_info::render_system_info;
