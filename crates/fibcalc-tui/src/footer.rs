//! TUI footer panel.

use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

/// Render the footer panel with keyboard shortcuts.
pub fn render_footer(frame: &mut Frame, area: Rect) {
    let text = vec![Line::from(vec![
        Span::styled("q", Style::default().fg(Color::Yellow)),
        Span::raw(": quit | "),
        Span::styled("p", Style::default().fg(Color::Yellow)),
        Span::raw(": pause | "),
        Span::styled("r", Style::default().fg(Color::Yellow)),
        Span::raw(": resume | "),
        Span::styled("d", Style::default().fg(Color::Yellow)),
        Span::raw(": details | "),
        Span::styled("l", Style::default().fg(Color::Yellow)),
        Span::raw(": logs | "),
        Span::styled("c", Style::default().fg(Color::Yellow)),
        Span::raw(": cancel"),
    ])];

    let block = Block::default().borders(Borders::TOP);
    let paragraph = Paragraph::new(text).block(block);
    frame.render_widget(paragraph, area);
}
