//! Progress display widget.

use ratatui::{
    layout::Rect,
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, Paragraph, Sparkline},
    Frame,
};

use crate::tui::app::App;
use crate::tui::state::CalculationStatus;
use crate::tui::style::styles;

/// Render the progress panel.
pub fn render_progress(app: &App, frame: &mut Frame, area: Rect) {
    let block = Block::default()
        .title(Span::styled(" PROGRESS ", styles::header()))
        .borders(Borders::ALL)
        .border_style(styles::border());

    // Calculate inner area for content
    let inner = block.inner(area);
    frame.render_widget(block, area);

    if inner.height < 4 {
        return;
    }

    // Status line
    let status_text = match app.calculation.status {
        CalculationStatus::Idle => "Status: Idle - Press [r] to start",
        CalculationStatus::Running => "Status: Calculating...",
        CalculationStatus::Complete => "Status: Complete!",
        CalculationStatus::Error => "Status: Error",
    };

    let status_style = match app.calculation.status {
        CalculationStatus::Idle => styles::text_dim(),
        CalculationStatus::Running => styles::running(),
        CalculationStatus::Complete => styles::success(),
        CalculationStatus::Error => styles::error(),
    };

    let status_line = Paragraph::new(Line::from(Span::styled(status_text, status_style)));
    let status_area = Rect::new(inner.x, inner.y, inner.width, 1);
    frame.render_widget(status_line, status_area);

    // Progress bar
    let progress_percent = (app.calculation.progress * 100.0) as u16;
    let gauge = Gauge::default()
        .gauge_style(styles::progress_filled())
        .percent(progress_percent)
        .label(format!("{:.1}%", app.calculation.progress * 100.0));

    let gauge_area = Rect::new(inner.x, inner.y + 2, inner.width, 1);
    frame.render_widget(gauge, gauge_area);

    // Time stats line
    if inner.height >= 5 {
        let elapsed = format_duration(app.calculation.elapsed);
        let eta = app
            .calculation
            .eta
            .map(|d| format_duration(d))
            .unwrap_or_else(|| "-".to_string());

        let rate = if app.calculation.elapsed.as_secs_f64() > 0.0 && app.calculation.progress > 0.0 {
            let rate = app.calculation.progress / app.calculation.elapsed.as_secs_f64() * 100.0;
            format!("{:.1}%/s", rate)
        } else {
            "-".to_string()
        };

        let stats = Line::from(vec![
            Span::styled("Elapsed: ", styles::label()),
            Span::styled(format!("{:<8}", elapsed), styles::value()),
            Span::styled(" | ", styles::text_dim()),
            Span::styled("ETA: ", styles::label()),
            Span::styled(format!("{:<8}", eta), styles::value()),
            Span::styled(" | ", styles::text_dim()),
            Span::styled("Rate: ", styles::label()),
            Span::styled(rate, styles::value()),
        ]);

        let stats_area = Rect::new(inner.x, inner.y + 4, inner.width, 1);
        frame.render_widget(Paragraph::new(stats), stats_area);
    }

    // Sparkline (if we have history and enough space)
    if inner.height >= 7 && !app.progress_history.is_empty() {
        let data: Vec<u64> = app
            .progress_history
            .iter()
            .map(|&p| (p * 100.0) as u64)
            .collect();

        let sparkline = Sparkline::default()
            .data(&data)
            .style(styles::progress_filled());

        let sparkline_area = Rect::new(inner.x, inner.y + 6, inner.width, 1);
        frame.render_widget(sparkline, sparkline_area);
    }
}

fn format_duration(d: std::time::Duration) -> String {
    let millis = d.as_millis();
    if millis < 1 {
        let micros = d.as_micros();
        format!("{:.2}ms", micros as f64 / 1000.0)
    } else if millis < 1000 {
        format!("{}ms", millis)
    } else {
        format!("{:.2}s", d.as_secs_f64())
    }
}
