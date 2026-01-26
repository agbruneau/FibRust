//! Event types for the TUI.

use crossterm::event::KeyEvent;
use ibig::UBig;
use std::time::Duration;

/// Events that can occur in the TUI.
#[derive(Debug)]
pub enum Event {
    /// Keyboard input event.
    Key(KeyEvent),
    /// Terminal resize event.
    Resize(u16, u16),
    /// Periodic tick for UI updates.
    Tick,
    /// Progress update from calculation thread.
    Progress(f64),
    /// An algorithm completed.
    AlgorithmComplete(AlgorithmCompleteEvent),
    /// All calculations finished.
    CalculationComplete,
}

/// Event data when an algorithm completes.
#[derive(Debug, Clone)]
pub struct AlgorithmCompleteEvent {
    pub name: String,
    pub duration: Duration,
    pub result: UBig,
}
