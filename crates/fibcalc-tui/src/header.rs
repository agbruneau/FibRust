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

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;

    #[test]
    fn render_header_does_not_panic() {
        let backend = TestBackend::new(80, 3);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|frame| {
                let area = frame.area();
                render_header(frame, area, 1000, "FastDoubling");
            })
            .unwrap();
    }

    #[test]
    fn render_header_contains_n_value() {
        let backend = TestBackend::new(80, 3);
        let mut terminal = Terminal::new(backend).unwrap();
        let buf = terminal
            .draw(|frame| {
                let area = frame.area();
                render_header(frame, area, 42, "Matrix");
            })
            .unwrap();

        // Collect all rows into a single string to find "N=42"
        let mut all_content = String::new();
        for y in 0..buf.area.height {
            for x in 0..buf.area.width {
                all_content.push_str(buf.buffer[(x, y)].symbol());
            }
        }
        assert!(
            all_content.contains("N=42"),
            "Buffer did not contain N=42: {all_content}"
        );
    }

    #[test]
    fn render_header_contains_algorithm() {
        let backend = TestBackend::new(80, 3);
        let mut terminal = Terminal::new(backend).unwrap();
        let buf = terminal
            .draw(|frame| {
                let area = frame.area();
                render_header(frame, area, 100, "FFT-Based");
            })
            .unwrap();

        let mut all_content = String::new();
        for y in 0..buf.area.height {
            for x in 0..buf.area.width {
                all_content.push_str(buf.buffer[(x, y)].symbol());
            }
        }
        assert!(
            all_content.contains("FFT-Based"),
            "Buffer did not contain FFT-Based: {all_content}"
        );
    }

    #[test]
    fn render_header_small_area() {
        let backend = TestBackend::new(20, 2);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|frame| {
                let area = frame.area();
                render_header(frame, area, 999_999, "FastDoubling");
            })
            .unwrap();
    }

    #[test]
    fn render_header_zero_n() {
        let backend = TestBackend::new(80, 3);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|frame| {
                let area = frame.area();
                render_header(frame, area, 0, "None");
            })
            .unwrap();
    }
}
