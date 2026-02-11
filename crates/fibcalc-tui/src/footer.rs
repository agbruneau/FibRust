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

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;

    #[test]
    fn render_footer_does_not_panic() {
        let backend = TestBackend::new(80, 3);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|frame| {
                let area = frame.area();
                render_footer(frame, area);
            })
            .unwrap();
    }

    #[test]
    fn render_footer_contains_quit_key() {
        let backend = TestBackend::new(80, 3);
        let mut terminal = Terminal::new(backend).unwrap();
        let buf = terminal
            .draw(|frame| {
                let area = frame.area();
                render_footer(frame, area);
            })
            .unwrap();

        // Row 1 (after the border) should contain the key hints
        let content: String = (0..buf.area.width)
            .map(|x| buf.buffer[(x, 1)].symbol().to_string())
            .collect();
        assert!(content.contains("quit"));
    }

    #[test]
    fn render_footer_contains_all_shortcuts() {
        let backend = TestBackend::new(100, 3);
        let mut terminal = Terminal::new(backend).unwrap();
        let buf = terminal
            .draw(|frame| {
                let area = frame.area();
                render_footer(frame, area);
            })
            .unwrap();

        let content: String = (0..buf.area.width)
            .map(|x| buf.buffer[(x, 1)].symbol().to_string())
            .collect();
        assert!(content.contains("quit"));
        assert!(content.contains("pause"));
        assert!(content.contains("resume"));
        assert!(content.contains("details"));
        assert!(content.contains("logs"));
        assert!(content.contains("cancel"));
    }

    #[test]
    fn render_footer_small_area() {
        let backend = TestBackend::new(20, 2);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|frame| {
                let area = frame.area();
                render_footer(frame, area);
            })
            .unwrap();
    }
}
