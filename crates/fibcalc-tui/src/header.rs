//! TUI header panel.

use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

/// Render the header panel.
pub fn render_header(frame: &mut Frame, area: Rect, n: u64, algo: &str) {
    let text = vec![Line::from(vec![
        Span::styled("FibCalc-rs", Style::default().fg(Color::Cyan)),
        Span::raw(format!(" | N={n} | Algorithm: {algo}")),
    ])];

    let block = Block::default()
        .borders(Borders::BOTTOM)
        .title(" FibCalc-rs ");

    let paragraph = Paragraph::new(text).block(block);
    frame.render_widget(paragraph, area);
}
