//! Algorithm comparison table widget.

use ratatui::{
    layout::{Constraint, Rect},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table},
    Frame,
};

use crate::tui::app::App;
use crate::tui::state::AlgorithmStatus;
use crate::tui::style::styles;

/// Render the algorithm comparison table.
pub fn render_comparison(app: &App, frame: &mut Frame, area: Rect) {
    let block = Block::default()
        .title(Span::styled(" ALGORITHM COMPARISON ", styles::header()))
        .borders(Borders::ALL)
        .border_style(styles::border());

    if app.results.is_empty() {
        // Show placeholder when no results
        let placeholder = Paragraph::new(Line::from(Span::styled(
            "Run a calculation to see comparison results",
            styles::text_dim(),
        )))
        .block(block);
        frame.render_widget(placeholder, area);
        return;
    }

    // Build table rows
    let header = Row::new(vec![
        Cell::from("Algorithm").style(styles::table_header()),
        Cell::from("Duration").style(styles::table_header()),
        Cell::from("Status").style(styles::table_header()),
        Cell::from("Speedup").style(styles::table_header()),
    ])
    .height(1);

    let fastest_duration = app
        .results
        .iter()
        .filter(|r| r.status == AlgorithmStatus::Done)
        .map(|r| r.duration)
        .min()
        .unwrap_or_default();

    let rows: Vec<Row> = app
        .results
        .iter()
        .map(|result| {
            let status = match result.status {
                AlgorithmStatus::Pending => Span::styled("Pending", styles::text_dim()),
                AlgorithmStatus::Running => Span::styled("Running", styles::running()),
                AlgorithmStatus::Done => Span::styled("Done", styles::success()),
                AlgorithmStatus::Error => Span::styled("Error", styles::error()),
            };

            let speedup = if result.status == AlgorithmStatus::Done && !fastest_duration.is_zero() {
                let ratio = result.duration.as_secs_f64() / fastest_duration.as_secs_f64();
                if ratio < 1.01 {
                    Span::styled("1.0x (fastest)", styles::success())
                } else {
                    Span::styled(format!("{:.2}x", ratio), styles::value())
                }
            } else {
                Span::styled("-", styles::text_dim())
            };

            Row::new(vec![
                Cell::from(result.name.clone()),
                Cell::from(format_duration(result.duration)),
                Cell::from(status),
                Cell::from(speedup),
            ])
        })
        .collect();

    let widths = [
        Constraint::Percentage(40),
        Constraint::Percentage(20),
        Constraint::Percentage(20),
        Constraint::Percentage(20),
    ];

    let table = Table::new(rows, widths)
        .header(header)
        .block(block)
        .row_highlight_style(styles::table_selected());

    frame.render_widget(table, area);
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
