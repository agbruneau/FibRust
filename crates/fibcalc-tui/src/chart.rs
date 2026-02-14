//! Progress chart widget.

use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, Borders, Gauge};
use ratatui::Frame;

/// Render progress gauges for each algorithm.
#[allow(clippy::cast_possible_truncation)]
pub fn render_progress(frame: &mut Frame, area: Rect, algorithms: &[&str], progress: &[f64]) {
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

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;

    fn render_in_test_terminal(width: u16, height: u16, algorithms: &[&str], progress: &[f64]) {
        let backend = TestBackend::new(width, height);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|frame| {
                let area = frame.area();
                render_progress(frame, area, algorithms, progress);
            })
            .unwrap();
    }

    #[test]
    fn render_progress_empty_algorithms() {
        // Should not panic with empty algorithms
        render_in_test_terminal(80, 24, &[], &[]);
    }

    #[test]
    fn render_progress_single_algorithm() {
        let algos = vec!["FastDoubling"];
        let progress = vec![0.5];
        render_in_test_terminal(80, 24, &algos, &progress);
    }

    #[test]
    fn render_progress_multiple_algorithms() {
        let algos = vec!["FastDoubling", "Matrix", "FFT"];
        let progress = vec![0.25, 0.5, 0.75];
        render_in_test_terminal(80, 24, &algos, &progress);
    }

    #[test]
    fn render_progress_zero_progress() {
        let algos = vec!["FastDoubling"];
        let progress = vec![0.0];
        render_in_test_terminal(80, 24, &algos, &progress);
    }

    #[test]
    fn render_progress_full_progress() {
        let algos = vec!["FastDoubling"];
        let progress = vec![1.0];
        render_in_test_terminal(80, 24, &algos, &progress);
    }

    #[test]
    fn render_progress_clamped_above_one() {
        let algos = vec!["FastDoubling"];
        let progress = vec![1.5]; // Should be clamped to 1.0
        render_in_test_terminal(80, 24, &algos, &progress);
    }

    #[test]
    fn render_progress_clamped_below_zero() {
        let algos = vec!["FastDoubling"];
        let progress = vec![-0.5]; // Should be clamped to 0.0
        render_in_test_terminal(80, 24, &algos, &progress);
    }

    #[test]
    fn render_progress_small_area() {
        let algos = vec!["FastDoubling", "Matrix"];
        let progress = vec![0.3, 0.7];
        render_in_test_terminal(20, 5, &algos, &progress);
    }

    #[test]
    fn render_progress_many_algorithms_small_height() {
        // More algorithms than available height - should break early
        let algos: Vec<&str> = vec![
            "Algo0", "Algo1", "Algo2", "Algo3", "Algo4", "Algo5", "Algo6", "Algo7", "Algo8",
            "Algo9",
        ];
        let progress: Vec<f64> = (0..10).map(|i| i as f64 / 10.0).collect();
        render_in_test_terminal(80, 6, &algos, &progress);
    }
}
