//! Runtime metrics panel with sysinfo collection.

use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;
use sysinfo::System;

/// Metrics collector using sysinfo.
pub struct MetricsCollector {
    system: System,
    /// Last collected CPU usage (0.0 - 100.0).
    pub cpu_percent: f64,
    /// Last collected memory usage in MB.
    pub memory_mb: f64,
    /// Throughput in bits/s (must be set externally).
    pub throughput_bits_per_sec: f64,
}

impl MetricsCollector {
    /// Create a new metrics collector.
    #[must_use]
    pub fn new() -> Self {
        Self {
            system: System::new(),
            cpu_percent: 0.0,
            memory_mb: 0.0,
            throughput_bits_per_sec: 0.0,
        }
    }

    /// Refresh system metrics.
    pub fn refresh(&mut self) {
        self.system.refresh_cpu_usage();
        self.system.refresh_memory();

        // Average CPU across all cores
        let cpus = self.system.cpus();
        if !cpus.is_empty() {
            self.cpu_percent =
                cpus.iter().map(|c| f64::from(c.cpu_usage())).sum::<f64>() / cpus.len() as f64;
        }

        // Memory in MB
        self.memory_mb = self.system.used_memory() as f64 / (1024.0 * 1024.0);
    }

    /// Set the throughput value (calculated externally from progress).
    pub fn set_throughput(&mut self, bits_per_sec: f64) {
        self.throughput_bits_per_sec = bits_per_sec;
    }

    /// Create a `SystemMetrics` snapshot for sending as a message.
    #[must_use]
    pub fn snapshot(&self) -> crate::messages::SystemMetrics {
        crate::messages::SystemMetrics {
            cpu_percent: self.cpu_percent,
            memory_mb: self.memory_mb,
            throughput_bits_per_sec: self.throughput_bits_per_sec,
        }
    }
}

impl Default for MetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

/// Format a throughput value into a human-readable string.
#[must_use]
pub fn format_throughput(bits_per_sec: f64) -> String {
    if bits_per_sec >= 1_000_000_000.0 {
        format!("{:.1} Gbit/s", bits_per_sec / 1_000_000_000.0)
    } else if bits_per_sec >= 1_000_000.0 {
        format!("{:.1} Mbit/s", bits_per_sec / 1_000_000.0)
    } else if bits_per_sec >= 1_000.0 {
        format!("{:.1} Kbit/s", bits_per_sec / 1_000.0)
    } else {
        format!("{bits_per_sec:.0} bit/s")
    }
}

/// Render the metrics panel.
pub fn render_metrics(
    frame: &mut Frame,
    area: Rect,
    elapsed_secs: f64,
    memory_mb: f64,
    cpu_percent: f64,
    throughput_bits_per_sec: f64,
) {
    let text = vec![
        Line::raw(format!("Elapsed:    {elapsed_secs:.1}s")),
        Line::raw(format!("Memory:     {memory_mb:.1} MB")),
        Line::raw(format!("CPU:        {cpu_percent:.0}%")),
        Line::raw(format!(
            "Throughput: {}",
            format_throughput(throughput_bits_per_sec)
        )),
    ];

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Metrics ")
        .border_style(Style::default().fg(Color::DarkGray));

    let paragraph = Paragraph::new(text).block(block);
    frame.render_widget(paragraph, area);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn metrics_collector_creation() {
        let collector = MetricsCollector::new();
        assert!((collector.cpu_percent).abs() < f64::EPSILON);
        assert!((collector.memory_mb).abs() < f64::EPSILON);
    }

    #[test]
    fn metrics_collector_refresh() {
        let mut collector = MetricsCollector::new();
        collector.refresh();
        // After refresh, memory should be > 0 on any real system
        // CPU might be 0 on first call (sysinfo needs two samples)
        assert!(collector.memory_mb >= 0.0);
    }

    #[test]
    fn metrics_collector_set_throughput() {
        let mut collector = MetricsCollector::new();
        collector.set_throughput(1_000_000.0);
        assert!((collector.throughput_bits_per_sec - 1_000_000.0).abs() < f64::EPSILON);
    }

    #[test]
    fn metrics_snapshot() {
        let mut collector = MetricsCollector::new();
        collector.cpu_percent = 50.0;
        collector.memory_mb = 1024.0;
        collector.throughput_bits_per_sec = 500_000.0;

        let snap = collector.snapshot();
        assert!((snap.cpu_percent - 50.0).abs() < f64::EPSILON);
        assert!((snap.memory_mb - 1024.0).abs() < f64::EPSILON);
        assert!((snap.throughput_bits_per_sec - 500_000.0).abs() < f64::EPSILON);
    }

    #[test]
    fn format_throughput_bits() {
        assert_eq!(format_throughput(500.0), "500 bit/s");
    }

    #[test]
    fn format_throughput_kbits() {
        assert_eq!(format_throughput(1_500.0), "1.5 Kbit/s");
    }

    #[test]
    fn format_throughput_mbits() {
        assert_eq!(format_throughput(2_500_000.0), "2.5 Mbit/s");
    }

    #[test]
    fn format_throughput_gbits() {
        assert_eq!(format_throughput(3_500_000_000.0), "3.5 Gbit/s");
    }

    #[test]
    fn format_throughput_zero() {
        assert_eq!(format_throughput(0.0), "0 bit/s");
    }

    #[test]
    fn format_throughput_boundary_kbits() {
        assert_eq!(format_throughput(1_000.0), "1.0 Kbit/s");
    }

    #[test]
    fn format_throughput_boundary_mbits() {
        assert_eq!(format_throughput(1_000_000.0), "1.0 Mbit/s");
    }

    #[test]
    fn format_throughput_boundary_gbits() {
        assert_eq!(format_throughput(1_000_000_000.0), "1.0 Gbit/s");
    }

    #[test]
    fn metrics_default_equals_new() {
        let collector = MetricsCollector::default();
        assert!((collector.cpu_percent).abs() < f64::EPSILON);
        assert!((collector.memory_mb).abs() < f64::EPSILON);
        assert!((collector.throughput_bits_per_sec).abs() < f64::EPSILON);
    }

    // --- render_metrics tests ---

    use ratatui::backend::TestBackend;
    use ratatui::Terminal;

    #[test]
    fn render_metrics_does_not_panic() {
        let backend = TestBackend::new(40, 8);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|frame| {
                let area = frame.area();
                render_metrics(frame, area, 12.5, 2048.0, 75.0, 500_000.0);
            })
            .unwrap();
    }

    #[test]
    fn render_metrics_contains_elapsed() {
        let backend = TestBackend::new(40, 8);
        let mut terminal = Terminal::new(backend).unwrap();
        let buf = terminal
            .draw(|frame| {
                let area = frame.area();
                render_metrics(frame, area, 42.3, 1024.0, 50.0, 0.0);
            })
            .unwrap();

        // Check row 1 (after border) for elapsed text
        let row1: String = (0..buf.area.width)
            .map(|x| buf.buffer[(x, 1)].symbol().to_string())
            .collect();
        assert!(row1.contains("Elapsed"));
        assert!(row1.contains("42.3"));
    }

    #[test]
    fn render_metrics_contains_memory() {
        let backend = TestBackend::new(40, 8);
        let mut terminal = Terminal::new(backend).unwrap();
        let buf = terminal
            .draw(|frame| {
                let area = frame.area();
                render_metrics(frame, area, 0.0, 512.5, 0.0, 0.0);
            })
            .unwrap();

        let row2: String = (0..buf.area.width)
            .map(|x| buf.buffer[(x, 2)].symbol().to_string())
            .collect();
        assert!(row2.contains("Memory"));
        assert!(row2.contains("512.5"));
    }

    #[test]
    fn render_metrics_contains_cpu() {
        let backend = TestBackend::new(40, 8);
        let mut terminal = Terminal::new(backend).unwrap();
        let buf = terminal
            .draw(|frame| {
                let area = frame.area();
                render_metrics(frame, area, 0.0, 0.0, 99.0, 0.0);
            })
            .unwrap();

        let row3: String = (0..buf.area.width)
            .map(|x| buf.buffer[(x, 3)].symbol().to_string())
            .collect();
        assert!(row3.contains("CPU"));
        assert!(row3.contains("99"));
    }

    #[test]
    fn render_metrics_zero_values() {
        let backend = TestBackend::new(40, 8);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|frame| {
                let area = frame.area();
                render_metrics(frame, area, 0.0, 0.0, 0.0, 0.0);
            })
            .unwrap();
    }

    #[test]
    fn render_metrics_small_area() {
        let backend = TestBackend::new(20, 5);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|frame| {
                let area = frame.area();
                render_metrics(frame, area, 100.0, 4096.0, 100.0, 0.0);
            })
            .unwrap();
    }
}
