//! Scrollable log panel with navigation.

use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, List, ListItem};
use ratatui::Frame;

/// Scroll state for the log panel.
#[derive(Debug, Clone)]
pub struct LogScrollState {
    /// Current scroll offset (first visible line index).
    pub offset: usize,
    /// Whether auto-scroll to bottom is enabled.
    pub auto_scroll: bool,
}

impl LogScrollState {
    /// Create a new scroll state.
    #[must_use]
    pub fn new() -> Self {
        Self {
            offset: 0,
            auto_scroll: true,
        }
    }

    /// Handle a new log message (auto-scroll if enabled).
    pub fn on_new_message(&mut self, total_logs: usize) {
        if self.auto_scroll {
            self.offset = total_logs.saturating_sub(1);
        }
    }

    /// Scroll up by one line.
    pub fn scroll_up(&mut self) {
        self.auto_scroll = false;
        self.offset = self.offset.saturating_sub(1);
    }

    /// Scroll down by one line.
    pub fn scroll_down(&mut self, total_logs: usize) {
        self.offset = (self.offset + 1).min(total_logs.saturating_sub(1));
        if self.offset >= total_logs.saturating_sub(1) {
            self.auto_scroll = true;
        }
    }

    /// Page up.
    pub fn page_up(&mut self, page_size: usize) {
        self.auto_scroll = false;
        self.offset = self.offset.saturating_sub(page_size);
    }

    /// Page down.
    pub fn page_down(&mut self, page_size: usize, total_logs: usize) {
        self.offset = (self.offset + page_size).min(total_logs.saturating_sub(1));
        if self.offset >= total_logs.saturating_sub(1) {
            self.auto_scroll = true;
        }
    }

    /// Jump to top.
    pub fn home(&mut self) {
        self.auto_scroll = false;
        self.offset = 0;
    }

    /// Jump to bottom.
    pub fn end(&mut self, total_logs: usize) {
        self.auto_scroll = true;
        self.offset = total_logs.saturating_sub(1);
    }
}

impl Default for LogScrollState {
    fn default() -> Self {
        Self::new()
    }
}

/// Render the scrollable log panel.
pub fn render_logs(frame: &mut Frame, area: Rect, logs: &[String], scroll_offset: usize) {
    let visible_height = area.height.saturating_sub(2) as usize; // account for borders
    let total = logs.len();

    // Clamp offset so we never skip past the point where
    // the last log sits at the bottom of the visible area.
    let max_offset = total.saturating_sub(visible_height);
    let effective_offset = scroll_offset.min(max_offset);

    let items: Vec<ListItem> = logs
        .iter()
        .skip(effective_offset)
        .take(visible_height)
        .map(|log| {
            let style = if log.starts_with("[ERROR]") {
                Style::default().fg(Color::Red)
            } else if log.starts_with("[WARN]") {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default()
            };
            ListItem::new(Line::raw(log.as_str())).style(style)
        })
        .collect();

    let scroll_indicator = if total > visible_height {
        let pct = if max_offset == 0 {
            100
        } else {
            (effective_offset * 100) / max_offset
        };
        format!(" Logs ({pct}%) ")
    } else {
        " Logs ".to_string()
    };

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title(scroll_indicator)
            .border_style(Style::default().fg(Color::DarkGray)),
    );

    frame.render_widget(list, area);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scroll_state_initial() {
        let state = LogScrollState::new();
        assert_eq!(state.offset, 0);
        assert!(state.auto_scroll);
    }

    #[test]
    fn scroll_state_on_new_message() {
        let mut state = LogScrollState::new();
        state.on_new_message(10);
        assert_eq!(state.offset, 9);
    }

    #[test]
    fn scroll_up() {
        let mut state = LogScrollState::new();
        state.offset = 5;
        state.scroll_up();
        assert_eq!(state.offset, 4);
        assert!(!state.auto_scroll);
    }

    #[test]
    fn scroll_up_at_zero() {
        let mut state = LogScrollState::new();
        state.scroll_up();
        assert_eq!(state.offset, 0);
    }

    #[test]
    fn scroll_down() {
        let mut state = LogScrollState::new();
        state.auto_scroll = false;
        state.scroll_down(10);
        assert_eq!(state.offset, 1);
    }

    #[test]
    fn scroll_down_to_bottom_enables_auto_scroll() {
        let mut state = LogScrollState::new();
        state.auto_scroll = false;
        state.offset = 8;
        state.scroll_down(10); // offset becomes 9 = total-1
        assert!(state.auto_scroll);
    }

    #[test]
    fn page_up() {
        let mut state = LogScrollState::new();
        state.offset = 15;
        state.page_up(10);
        assert_eq!(state.offset, 5);
        assert!(!state.auto_scroll);
    }

    #[test]
    fn page_up_clamps_to_zero() {
        let mut state = LogScrollState::new();
        state.offset = 3;
        state.page_up(10);
        assert_eq!(state.offset, 0);
    }

    #[test]
    fn page_down() {
        let mut state = LogScrollState::new();
        state.auto_scroll = false;
        state.offset = 5;
        state.page_down(10, 50);
        assert_eq!(state.offset, 15);
    }

    #[test]
    fn page_down_to_end() {
        let mut state = LogScrollState::new();
        state.auto_scroll = false;
        state.page_down(100, 20);
        assert_eq!(state.offset, 19);
        assert!(state.auto_scroll);
    }

    #[test]
    fn home() {
        let mut state = LogScrollState::new();
        state.offset = 50;
        state.home();
        assert_eq!(state.offset, 0);
        assert!(!state.auto_scroll);
    }

    #[test]
    fn end() {
        let mut state = LogScrollState::new();
        state.auto_scroll = false;
        state.end(30);
        assert_eq!(state.offset, 29);
        assert!(state.auto_scroll);
    }

    #[test]
    fn default_equals_new() {
        let state = LogScrollState::default();
        assert_eq!(state.offset, 0);
        assert!(state.auto_scroll);
    }

    #[test]
    fn on_new_message_no_auto_scroll() {
        let mut state = LogScrollState::new();
        state.auto_scroll = false;
        state.offset = 5;
        state.on_new_message(20);
        // Should not change offset when auto_scroll is off
        assert_eq!(state.offset, 5);
    }

    #[test]
    fn scroll_down_already_at_end() {
        let mut state = LogScrollState::new();
        state.auto_scroll = false;
        state.offset = 9;
        state.scroll_down(10);
        assert_eq!(state.offset, 9);
        assert!(state.auto_scroll);
    }

    #[test]
    fn end_with_zero_logs() {
        let mut state = LogScrollState::new();
        state.end(0);
        assert_eq!(state.offset, 0);
        assert!(state.auto_scroll);
    }

    #[test]
    fn page_down_with_zero_logs() {
        let mut state = LogScrollState::new();
        state.page_down(10, 0);
        assert_eq!(state.offset, 0);
        assert!(state.auto_scroll);
    }

    // --- render_logs tests ---

    use ratatui::backend::TestBackend;
    use ratatui::Terminal;

    fn render_logs_in_test_terminal(logs: &[String], scroll_offset: usize) {
        let backend = TestBackend::new(60, 15);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|frame| {
                let area = frame.area();
                render_logs(frame, area, logs, scroll_offset);
            })
            .unwrap();
    }

    #[test]
    fn render_logs_empty() {
        render_logs_in_test_terminal(&[], 0);
    }

    #[test]
    fn render_logs_single_entry() {
        let logs = vec!["Hello world".to_string()];
        render_logs_in_test_terminal(&logs, 0);
    }

    #[test]
    fn render_logs_multiple_entries() {
        let logs: Vec<String> = (0..10).map(|i| format!("Log entry {i}")).collect();
        render_logs_in_test_terminal(&logs, 0);
    }

    #[test]
    fn render_logs_with_scroll_offset() {
        let logs: Vec<String> = (0..20).map(|i| format!("Log entry {i}")).collect();
        render_logs_in_test_terminal(&logs, 5);
    }

    #[test]
    fn render_logs_scroll_offset_clamped() {
        let logs: Vec<String> = (0..5).map(|i| format!("Log entry {i}")).collect();
        // Offset larger than logs should be clamped
        render_logs_in_test_terminal(&logs, 100);
    }

    #[test]
    fn render_logs_with_error_styling() {
        let logs = vec![
            "[ERROR] something went wrong".to_string(),
            "Normal log".to_string(),
            "[WARN] watch out".to_string(),
        ];
        render_logs_in_test_terminal(&logs, 0);
    }

    #[test]
    fn render_logs_shows_percentage_when_scrollable() {
        let logs: Vec<String> = (0..50).map(|i| format!("Log entry {i}")).collect();
        let backend = TestBackend::new(60, 10);
        let mut terminal = Terminal::new(backend).unwrap();
        let buf = terminal
            .draw(|frame| {
                let area = frame.area();
                render_logs(frame, area, &logs, 0);
            })
            .unwrap();

        // Check border/title row for "Logs (0%)"
        let title_row: String = (0..buf.area.width)
            .map(|x| buf.buffer[(x, 0)].symbol().to_string())
            .collect();
        assert!(title_row.contains("Logs"));
        assert!(title_row.contains("%"));
    }

    #[test]
    fn render_logs_no_percentage_when_not_scrollable() {
        // Few logs that fit in the view
        let logs = vec!["Short log".to_string()];
        let backend = TestBackend::new(60, 10);
        let mut terminal = Terminal::new(backend).unwrap();
        let buf = terminal
            .draw(|frame| {
                let area = frame.area();
                render_logs(frame, area, &logs, 0);
            })
            .unwrap();

        let title_row: String = (0..buf.area.width)
            .map(|x| buf.buffer[(x, 0)].symbol().to_string())
            .collect();
        assert!(title_row.contains("Logs"));
        assert!(!title_row.contains("%"));
    }
}
