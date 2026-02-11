//! Progress chart widget.

use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, Borders, Gauge};
use ratatui::Frame;

/// Render progress gauges for each algorithm.
pub fn render_progress(frame: &mut Frame, area: Rect, algorithms: &[String], progress: &[f64]) {
    if algorithms.is_empty() {
        return;
    }

    let height = area.height as usize;
    let per_gauge = (height / algorithms.len().max(1)).max(1);

    for (i, (algo, &prog)) in algorithms.iter().zip(progress.iter()).enumerate() {
        let y = area.y + (i * per_gauge) as u16;
        if y >= area.y + area.height {
            break;
        }

        let gauge_area = Rect {
            x: area.x,
            y,
            width: area.width,
            height: per_gauge.min((area.y + area.height - y) as usize) as u16,
        };

        let gauge = Gauge::default()
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(format!(" {algo} ")),
            )
            .gauge_style(Style::default().fg(Color::Green))
            .ratio(prog.clamp(0.0, 1.0));

        frame.render_widget(gauge, gauge_area);
    }
}
