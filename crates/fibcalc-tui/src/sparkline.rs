//! Sparkline visualization with Braille rendering.

use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, Paragraph, Sparkline as RatatuiSparkline};
use ratatui::Frame;

/// Braille character base offset (Unicode block U+2800).
const BRAILLE_BASE: u32 = 0x2800;

/// Braille dot patterns for rows 0-3 in each column (left, right).
/// Each Braille character has 2 columns x 4 rows = 8 dots.
/// Left column bits:  row0=0x01, row1=0x02, row2=0x04, row3=0x40
/// Right column bits: row0=0x08, row1=0x10, row2=0x20, row3=0x80
const BRAILLE_LEFT: [u32; 4] = [0x40, 0x04, 0x02, 0x01];
const BRAILLE_RIGHT: [u32; 4] = [0x80, 0x20, 0x10, 0x08];

/// Render a sparkline widget using ratatui's built-in sparkline.
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
pub fn render_sparkline(frame: &mut Frame, area: Rect, data: &[f64], title: &str) {
    let scaled: Vec<u64> = data.iter().map(|&v| (v * 100.0) as u64).collect();

    let sparkline = RatatuiSparkline::default()
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!(" {title} ")),
        )
        .data(&scaled)
        .style(Style::default().fg(Color::Yellow));

    frame.render_widget(sparkline, area);
}

/// Render a sparkline using Braille characters for high-resolution display.
///
/// Each character cell encodes a 2-wide x 4-tall grid of dots,
/// giving 2x horizontal and 4x vertical resolution vs block characters.
pub fn render_braille_sparkline(frame: &mut Frame, area: Rect, data: &[f64], title: &str) {
    if area.height < 3 || area.width < 4 {
        // Too small, fall back to standard sparkline
        render_sparkline(frame, area, data, title);
        return;
    }

    let block = Block::default()
        .borders(Borders::ALL)
        .title(format!(" {title} "));
    let inner = block.inner(area);

    if data.is_empty() || inner.width == 0 || inner.height == 0 {
        frame.render_widget(block, area);
        return;
    }

    let lines = braille_lines(data, inner.width as usize, inner.height as usize);
    let paragraph = Paragraph::new(lines).style(Style::default().fg(Color::Cyan));

    frame.render_widget(block, area);
    frame.render_widget(paragraph, inner);
}

/// Generate Braille-encoded lines from data.
#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss
)]
fn braille_lines(data: &[f64], char_width: usize, char_height: usize) -> Vec<Line<'static>> {
    if data.is_empty() || char_width == 0 || char_height == 0 {
        return Vec::new();
    }

    let max_val = data.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let min_val = data.iter().copied().fold(f64::INFINITY, f64::min);
    let range = if (max_val - min_val).abs() < f64::EPSILON {
        1.0
    } else {
        max_val - min_val
    };

    // Each character is 2 data points wide, 4 rows tall
    let dot_rows = char_height * 4;
    let dot_cols = char_width * 2;

    // Resample data to fit dot_cols
    let resampled: Vec<f64> = (0..dot_cols)
        .map(|i| {
            let idx_f = (i as f64) / (dot_cols as f64) * (data.len() as f64);
            let idx = (idx_f as usize).min(data.len().saturating_sub(1));
            data[idx]
        })
        .collect();

    // Normalize to [0, dot_rows-1]
    let normalized: Vec<usize> = resampled
        .iter()
        .map(|&v| {
            let n = ((v - min_val) / range * (dot_rows.saturating_sub(1)) as f64).round() as usize;
            n.min(dot_rows.saturating_sub(1))
        })
        .collect();

    // Build character grid
    let mut lines = Vec::with_capacity(char_height);
    for row in 0..char_height {
        let mut line = String::with_capacity(char_width);
        for col in 0..char_width {
            let mut braille = BRAILLE_BASE;
            // Left column dot (col*2)
            let left_idx = col * 2;
            if left_idx < normalized.len() {
                let val_row = dot_rows.saturating_sub(1) - normalized[left_idx];
                let local_row = val_row.saturating_sub(row * 4);
                if val_row >= row * 4 && local_row < 4 {
                    braille |= BRAILLE_LEFT[local_row];
                }
            }
            // Right column dot (col*2+1)
            let right_idx = col * 2 + 1;
            if right_idx < normalized.len() {
                let val_row = dot_rows.saturating_sub(1) - normalized[right_idx];
                let local_row = val_row.saturating_sub(row * 4);
                if val_row >= row * 4 && local_row < 4 {
                    braille |= BRAILLE_RIGHT[local_row];
                }
            }
            if let Some(ch) = char::from_u32(braille) {
                line.push(ch);
            }
        }
        lines.push(Line::raw(line));
    }

    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn braille_lines_empty_data() {
        let lines = braille_lines(&[], 10, 5);
        assert!(lines.is_empty());
    }

    #[test]
    fn braille_lines_single_value() {
        let lines = braille_lines(&[1.0], 5, 3);
        assert_eq!(lines.len(), 3);
    }

    #[test]
    fn braille_lines_multiple_values() {
        let data: Vec<f64> = (0..20).map(|i| i as f64).collect();
        let lines = braille_lines(&data, 10, 4);
        assert_eq!(lines.len(), 4);
        // Each line should have 10 characters
        for line in &lines {
            let text: String = line.spans.iter().map(|s| s.content.as_ref()).collect();
            assert_eq!(text.chars().count(), 10);
        }
    }

    #[test]
    fn braille_lines_constant_data() {
        let data = vec![5.0; 20];
        let lines = braille_lines(&data, 10, 3);
        assert_eq!(lines.len(), 3);
    }

    #[test]
    fn braille_characters_are_valid_unicode() {
        let data: Vec<f64> = (0..10).map(|i| i as f64).collect();
        let lines = braille_lines(&data, 5, 2);
        for line in &lines {
            let text: String = line.spans.iter().map(|s| s.content.as_ref()).collect();
            for ch in text.chars() {
                let code = ch as u32;
                assert!(
                    (0x2800..=0x28FF).contains(&code),
                    "Expected Braille character, got U+{code:04X}"
                );
            }
        }
    }

    #[test]
    fn braille_lines_zero_width() {
        let lines = braille_lines(&[1.0, 2.0], 0, 5);
        assert!(lines.is_empty());
    }

    #[test]
    fn braille_lines_zero_height() {
        let lines = braille_lines(&[1.0, 2.0], 10, 0);
        assert!(lines.is_empty());
    }

    #[test]
    fn braille_lines_negative_values() {
        let data = vec![-5.0, -3.0, -1.0, 0.0, 1.0, 3.0, 5.0];
        let lines = braille_lines(&data, 4, 3);
        assert_eq!(lines.len(), 3);
    }

    #[test]
    fn braille_lines_large_dataset() {
        let data: Vec<f64> = (0..1000).map(|i| (i as f64).sin() * 100.0).collect();
        let lines = braille_lines(&data, 40, 10);
        assert_eq!(lines.len(), 10);
        for line in &lines {
            let text: String = line.spans.iter().map(|s| s.content.as_ref()).collect();
            assert_eq!(text.chars().count(), 40);
        }
    }

    // --- render function tests ---

    use ratatui::backend::TestBackend;
    use ratatui::Terminal;

    #[test]
    fn render_sparkline_does_not_panic() {
        let backend = TestBackend::new(40, 8);
        let mut terminal = Terminal::new(backend).unwrap();
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        terminal
            .draw(|frame| {
                let area = frame.area();
                render_sparkline(frame, area, &data, "Throughput");
            })
            .unwrap();
    }

    #[test]
    fn render_sparkline_empty_data() {
        let backend = TestBackend::new(40, 8);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|frame| {
                let area = frame.area();
                render_sparkline(frame, area, &[], "Empty");
            })
            .unwrap();
    }

    #[test]
    fn render_braille_sparkline_does_not_panic() {
        let backend = TestBackend::new(40, 8);
        let mut terminal = Terminal::new(backend).unwrap();
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        terminal
            .draw(|frame| {
                let area = frame.area();
                render_braille_sparkline(frame, area, &data, "Braille");
            })
            .unwrap();
    }

    #[test]
    fn render_braille_sparkline_empty_data() {
        let backend = TestBackend::new(40, 8);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|frame| {
                let area = frame.area();
                render_braille_sparkline(frame, area, &[], "Empty");
            })
            .unwrap();
    }

    #[test]
    fn render_braille_sparkline_too_small_falls_back() {
        // Area height < 3, should fall back to standard sparkline
        let backend = TestBackend::new(10, 2);
        let mut terminal = Terminal::new(backend).unwrap();
        let data = vec![1.0, 2.0, 3.0];
        terminal
            .draw(|frame| {
                let area = frame.area();
                render_braille_sparkline(frame, area, &data, "Small");
            })
            .unwrap();
    }

    #[test]
    fn render_braille_sparkline_too_narrow_falls_back() {
        // Area width < 4, should fall back
        let backend = TestBackend::new(3, 8);
        let mut terminal = Terminal::new(backend).unwrap();
        let data = vec![1.0, 2.0, 3.0];
        terminal
            .draw(|frame| {
                let area = frame.area();
                render_braille_sparkline(frame, area, &data, "Narrow");
            })
            .unwrap();
    }

    #[test]
    fn render_braille_sparkline_large_data() {
        let backend = TestBackend::new(60, 12);
        let mut terminal = Terminal::new(backend).unwrap();
        let data: Vec<f64> = (0..100)
            .map(|i| (i as f64 * 0.1).sin() * 50.0 + 50.0)
            .collect();
        terminal
            .draw(|frame| {
                let area = frame.area();
                render_braille_sparkline(frame, area, &data, "Large");
            })
            .unwrap();
    }
}
