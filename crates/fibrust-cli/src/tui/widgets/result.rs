//! Result analysis widget.

use ratatui::{
    layout::Rect,
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

use crate::tui::app::App;
use crate::tui::state::CalculationStatus;
use crate::tui::style::styles;

/// Render the result analysis panel.
pub fn render_result(app: &App, frame: &mut Frame, area: Rect) {
    let block = Block::default()
        .title(Span::styled(" RESULT ANALYSIS ", styles::header()))
        .borders(Borders::ALL)
        .border_style(styles::border());

    if app.calculation.status != CalculationStatus::Complete {
        let placeholder = Paragraph::new(Line::from(Span::styled(
            "Results will appear here after calculation completes",
            styles::text_dim(),
        )))
        .block(block);
        frame.render_widget(placeholder, area);
        return;
    }

    let analysis = &app.analysis;

    let consistency_text = if app.results.len() > 1 {
        if analysis.consistent {
            Span::styled("All results match", styles::success())
        } else {
            Span::styled("WARNING: Results differ!", styles::error())
        }
    } else {
        Span::styled("N/A (single algorithm)", styles::text_dim())
    };

    let lines = vec![
        Line::from(vec![
            Span::styled("Binary Size:  ", styles::label()),
            Span::styled(format_number(analysis.binary_bits), styles::value()),
            Span::styled(" bits", styles::text_dim()),
            Span::styled("    |    ", styles::text_dim()),
            Span::styled("Digits: ", styles::label()),
            Span::styled(format_number(analysis.digit_count), styles::value()),
        ]),
        Line::from(vec![
            Span::styled("Scientific:   ", styles::label()),
            Span::styled(&analysis.scientific, styles::value()),
        ]),
        Line::from(vec![
            Span::styled("Consistency:  ", styles::label()),
            consistency_text,
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Preview: ", styles::label()),
            Span::styled(&analysis.preview, styles::text_dim()),
        ]),
    ];

    let paragraph = Paragraph::new(lines)
        .block(block)
        .wrap(Wrap { trim: true });

    frame.render_widget(paragraph, area);
}

fn format_number(n: usize) -> String {
    let s = n.to_string();
    let mut result = String::new();
    for (i, c) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            result.push(',');
        }
        result.push(c);
    }
    result.chars().rev().collect()
}
