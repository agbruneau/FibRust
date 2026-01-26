//! System information widget.

use ratatui::{
    layout::Rect,
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::tui::app::App;
use crate::tui::style::styles;

/// Render the system info panel.
pub fn render_system_info(app: &App, frame: &mut Frame, area: Rect) {
    let info = &app.system_info;

    let lines = vec![
        Line::from(vec![
            Span::styled("CPU Cores:    ", styles::label()),
            Span::styled(format!("{} logical processors", info.cpu_count), styles::value()),
        ]),
        Line::from(vec![
            Span::styled("Parallelism:  ", styles::label()),
            Span::styled(format!("{} threshold", info.parallelism_threshold), styles::value()),
        ]),
        Line::from(vec![
            Span::styled("FFT Switch:   ", styles::label()),
            Span::styled(info.fft_threshold.to_string(), styles::value()),
        ]),
        Line::from(vec![
            Span::styled("Memory Est:   ", styles::label()),
            Span::styled(estimate_memory(&app.input), styles::value()),
        ]),
    ];

    let block = Block::default()
        .title(Span::styled(" SYSTEM INFO ", styles::header()))
        .borders(Borders::ALL)
        .border_style(styles::border());

    let paragraph = Paragraph::new(lines).block(block);

    frame.render_widget(paragraph, area);
}

fn estimate_memory(input: &crate::tui::state::InputState) -> String {
    if let Some(n) = input.n() {
        // Rough estimate: F(n) has about n * 0.694 bits
        let bits = (n as f64 * 0.694) as u64;
        let bytes = bits / 8;
        if bytes < 1024 {
            format!("~{} bytes for F(n)", bytes)
        } else if bytes < 1024 * 1024 {
            format!("~{:.1} KB for F(n)", bytes as f64 / 1024.0)
        } else {
            format!("~{:.1} MB for F(n)", bytes as f64 / (1024.0 * 1024.0))
        }
    } else {
        "Invalid n".to_string()
    }
}
