//! TUI message types (Elm Messages).

use std::time::Duration;

/// System metrics snapshot.
#[derive(Debug, Clone)]
pub struct SystemMetrics {
    /// CPU usage as a percentage (0.0 - 100.0).
    pub cpu_percent: f64,
    /// Memory usage in megabytes.
    pub memory_mb: f64,
    /// Throughput in bits per second.
    pub throughput_bits_per_sec: f64,
}

/// Messages that drive the TUI update cycle.
#[derive(Debug, Clone)]
pub enum TuiMessage {
    /// Progress update from a calculator.
    Progress {
        index: usize,
        progress: f64,
        algorithm: &'static str,
    },
    /// Log message.
    Log(String),
    /// Sparkline data point.
    SparklineData(f64),
    /// Calculation started.
    Started,
    /// Calculation complete.
    Complete {
        algorithm: String,
        duration: Duration,
    },
    /// Quit the application.
    Quit,
    /// Tick event for periodic updates.
    Tick,
    /// Terminal resize event.
    Resize { width: u16, height: u16 },
    /// Key press event forwarded from the event loop.
    KeyPress(crate::keymap::KeyAction),
    /// Error message.
    Error(String),
    /// System metrics update (CPU, memory, throughput).
    SystemMetrics(SystemMetrics),
    /// All calculations finished — freezes the elapsed timer.
    Finished,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn system_metrics_creation() {
        let metrics = SystemMetrics {
            cpu_percent: 42.5,
            memory_mb: 1024.0,
            throughput_bits_per_sec: 1_000_000.0,
        };
        assert!((metrics.cpu_percent - 42.5).abs() < f64::EPSILON);
        assert!((metrics.memory_mb - 1024.0).abs() < f64::EPSILON);
    }

    #[test]
    fn message_variants() {
        let msg = TuiMessage::Progress {
            index: 0,
            progress: 0.5,
            algorithm: "FastDoubling",
        };
        assert!(matches!(msg, TuiMessage::Progress { .. }));

        let msg = TuiMessage::Tick;
        assert!(matches!(msg, TuiMessage::Tick));

        let msg = TuiMessage::Resize {
            width: 80,
            height: 24,
        };
        assert!(matches!(msg, TuiMessage::Resize { .. }));

        let msg = TuiMessage::Error("test error".to_string());
        assert!(matches!(msg, TuiMessage::Error(_)));

        let msg = TuiMessage::SystemMetrics(SystemMetrics {
            cpu_percent: 50.0,
            memory_mb: 512.0,
            throughput_bits_per_sec: 0.0,
        });
        assert!(matches!(msg, TuiMessage::SystemMetrics(_)));
    }
}
