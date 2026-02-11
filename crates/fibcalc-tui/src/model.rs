//! TUI application model (Elm architecture).

use std::io;
use std::time::{Duration, Instant};

use crossbeam_channel::Receiver;
use crossterm::event::{self, Event};
use crossterm::terminal::{self, EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::{event::DisableMouseCapture, event::EnableMouseCapture, execute};
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::Terminal;

use crate::chart::render_progress;
use crate::footer::render_footer;
use crate::header::render_header;
use crate::keymap::{map_key, KeyAction};
use crate::logs::render_logs;
use crate::messages::TuiMessage;
use crate::metrics::render_metrics;
use crate::sparkline::render_sparkline;

/// TUI application state (Elm Model).
pub struct TuiApp {
    /// Whether the app should quit.
    pub should_quit: bool,
    /// Whether the calculation is paused.
    pub paused: bool,
    /// Current calculation progress per algorithm.
    pub progress: Vec<f64>,
    /// Algorithm names.
    pub algorithms: Vec<String>,
    /// Completed algorithms with their durations.
    pub completed: Vec<(String, Duration)>,
    /// Log messages.
    pub logs: Vec<String>,
    /// Sparkline data for throughput.
    pub sparkline_data: Vec<f64>,
    /// Start time of the calculation.
    pub start_time: Option<Instant>,
    /// Message receiver.
    rx: Receiver<TuiMessage>,
    /// Current generation for message filtering.
    generation: u64,
    /// Terminal width.
    pub terminal_width: u16,
    /// Terminal height.
    pub terminal_height: u16,
    /// Log scroll offset.
    pub log_scroll_offset: usize,
    /// Whether to auto-scroll logs.
    pub log_auto_scroll: bool,
    /// Show details toggle.
    pub show_details: bool,
    /// Show logs toggle.
    pub show_logs: bool,
    /// Current CPU usage.
    pub cpu_percent: f64,
    /// Current memory usage in MB.
    pub memory_mb: f64,
    /// Current throughput in bits/s.
    pub throughput_bits_per_sec: f64,
    /// N value being computed.
    pub n_value: u64,
    /// Error messages.
    pub errors: Vec<String>,
}

impl TuiApp {
    /// Create a new TUI app.
    #[must_use]
    pub fn new(rx: Receiver<TuiMessage>) -> Self {
        Self {
            should_quit: false,
            paused: false,
            progress: Vec::new(),
            algorithms: Vec::new(),
            completed: Vec::new(),
            logs: Vec::new(),
            sparkline_data: Vec::new(),
            start_time: None,
            rx,
            generation: 0,
            terminal_width: 80,
            terminal_height: 24,
            log_scroll_offset: 0,
            log_auto_scroll: true,
            show_details: true,
            show_logs: true,
            cpu_percent: 0.0,
            memory_mb: 0.0,
            throughput_bits_per_sec: 0.0,
            n_value: 0,
            errors: Vec::new(),
        }
    }

    /// Set the N value being computed.
    pub fn set_n(&mut self, n: u64) {
        self.n_value = n;
    }

    /// Get the current generation.
    #[must_use]
    pub fn generation(&self) -> u64 {
        self.generation
    }

    /// Update the model with incoming messages (Elm Update).
    pub fn update(&mut self) {
        while let Ok(msg) = self.rx.try_recv() {
            self.handle_message(msg);
        }
    }

    /// Handle a single message.
    pub fn handle_message(&mut self, msg: TuiMessage) {
        match msg {
            TuiMessage::Progress {
                index,
                progress,
                algorithm,
            } => {
                // Ensure vectors are large enough
                while self.progress.len() <= index {
                    self.progress.push(0.0);
                    self.algorithms.push(String::new());
                }
                self.progress[index] = progress;
                self.algorithms[index] = algorithm;
            }
            TuiMessage::Log(msg) => {
                self.logs.push(msg);
                // Keep only last 500 log entries
                if self.logs.len() > 500 {
                    self.logs.remove(0);
                    // Adjust scroll offset when removing items
                    if self.log_scroll_offset > 0 {
                        self.log_scroll_offset -= 1;
                    }
                }
                // Auto-scroll to bottom on new messages
                if self.log_auto_scroll {
                    self.scroll_logs_to_bottom();
                }
            }
            TuiMessage::SparklineData(value) => {
                self.sparkline_data.push(value);
                // Ring buffer: keep last 60 values
                if self.sparkline_data.len() > 60 {
                    self.sparkline_data.remove(0);
                }
            }
            TuiMessage::Started => {
                self.start_time = Some(Instant::now());
                self.generation += 1;
                self.progress.clear();
                self.algorithms.clear();
                self.completed.clear();
                self.errors.clear();
            }
            TuiMessage::Quit => {
                self.should_quit = true;
            }
            TuiMessage::Complete {
                algorithm,
                duration,
            } => {
                self.completed.push((algorithm, duration));
            }
            TuiMessage::Tick => {
                // Tick triggers re-render, nothing to update in model
            }
            TuiMessage::Resize { width, height } => {
                self.terminal_width = width;
                self.terminal_height = height;
            }
            TuiMessage::KeyPress(action) => {
                self.handle_key_action(action);
            }
            TuiMessage::Error(err) => {
                self.errors.push(err.clone());
                self.logs.push(format!("[ERROR] {err}"));
            }
            TuiMessage::SystemMetrics(metrics) => {
                self.cpu_percent = metrics.cpu_percent;
                self.memory_mb = metrics.memory_mb;
                self.throughput_bits_per_sec = metrics.throughput_bits_per_sec;
                // Also push throughput to sparkline
                self.sparkline_data.push(metrics.throughput_bits_per_sec);
                if self.sparkline_data.len() > 60 {
                    self.sparkline_data.remove(0);
                }
            }
        }
    }

    /// Handle a keyboard action.
    pub fn handle_key_action(&mut self, action: KeyAction) {
        match action {
            KeyAction::Quit | KeyAction::Cancel => {
                self.should_quit = true;
            }
            KeyAction::Pause => {
                self.paused = true;
            }
            KeyAction::Resume => {
                self.paused = false;
            }
            KeyAction::ToggleDetails => {
                self.show_details = !self.show_details;
            }
            KeyAction::ToggleLogs => {
                self.show_logs = !self.show_logs;
            }
            KeyAction::ScrollUp => {
                self.log_auto_scroll = false;
                if self.log_scroll_offset > 0 {
                    self.log_scroll_offset -= 1;
                }
            }
            KeyAction::ScrollDown => {
                self.log_scroll_offset += 1;
                let max_offset = self.logs.len().saturating_sub(1);
                if self.log_scroll_offset >= max_offset {
                    self.log_scroll_offset = max_offset;
                    self.log_auto_scroll = true;
                }
            }
            KeyAction::PageUp => {
                self.page_up(10);
            }
            KeyAction::PageDown => {
                self.page_down(10);
            }
            KeyAction::Home => {
                self.scroll_home();
            }
            KeyAction::End => {
                self.scroll_end();
            }
            KeyAction::None => {}
        }
    }

    /// Scroll logs to the bottom.
    fn scroll_logs_to_bottom(&mut self) {
        self.log_scroll_offset = self.logs.len().saturating_sub(1);
    }

    /// Scroll logs up by a page.
    pub fn page_up(&mut self, page_size: usize) {
        self.log_auto_scroll = false;
        self.log_scroll_offset = self.log_scroll_offset.saturating_sub(page_size);
    }

    /// Scroll logs down by a page.
    pub fn page_down(&mut self, page_size: usize) {
        self.log_scroll_offset += page_size;
        let max_offset = self.logs.len().saturating_sub(1);
        if self.log_scroll_offset >= max_offset {
            self.log_scroll_offset = max_offset;
            self.log_auto_scroll = true;
        }
    }

    /// Scroll logs to the very top.
    pub fn scroll_home(&mut self) {
        self.log_auto_scroll = false;
        self.log_scroll_offset = 0;
    }

    /// Scroll logs to the very bottom.
    pub fn scroll_end(&mut self) {
        self.log_auto_scroll = true;
        self.scroll_logs_to_bottom();
    }

    /// Get the elapsed time since calculation started.
    #[must_use]
    pub fn elapsed(&self) -> Option<Duration> {
        self.start_time.map(|t| t.elapsed())
    }

    /// Compute the adaptive 60/40 layout.
    ///
    /// Returns (header, `progress_panel`, `info_panel`, footer) rects.
    #[must_use]
    pub fn compute_layout(area: Rect) -> (Rect, Rect, Rect, Rect) {
        let outer = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // header
                Constraint::Min(5),    // main content
                Constraint::Length(2), // footer
            ])
            .split(area);

        let header = outer[0];
        let footer = outer[2];

        // Split main content 60/40
        let main = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(60), // progress panel
                Constraint::Percentage(40), // info panel
            ])
            .split(outer[1]);

        (header, main[0], main[1], footer)
    }

    /// Compute the info panel sub-layout (metrics + sparkline left, logs right).
    #[must_use]
    pub fn compute_info_layout(info_area: Rect) -> (Rect, Rect) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(40), // metrics + sparkline
                Constraint::Percentage(60), // logs
            ])
            .split(info_area);

        (chunks[0], chunks[1])
    }

    /// Compute metrics sub-layout (metrics top, sparkline bottom).
    #[must_use]
    pub fn compute_metrics_layout(metrics_area: Rect) -> (Rect, Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(6), // metrics
                Constraint::Min(3),    // sparkline
            ])
            .split(metrics_area);

        (chunks[0], chunks[1])
    }

    /// Render the full TUI view.
    pub fn render(&self, frame: &mut ratatui::Frame) {
        let (header_area, progress_area, info_area, footer_area) =
            Self::compute_layout(frame.area());

        // Header
        let algo_display = if self.algorithms.is_empty() {
            "N/A".to_string()
        } else {
            self.algorithms.join(", ")
        };
        render_header(frame, header_area, self.n_value, &algo_display);

        // Progress panel (60% top)
        render_progress(frame, progress_area, &self.algorithms, &self.progress);

        // Info panel (40% bottom)
        if self.show_logs {
            let (metrics_col, logs_col) = Self::compute_info_layout(info_area);

            // Left column: metrics + sparkline
            let (metrics_rect, sparkline_rect) = Self::compute_metrics_layout(metrics_col);

            let elapsed_secs = self.elapsed().map_or(0.0, |d| d.as_secs_f64());
            render_metrics(
                frame,
                metrics_rect,
                elapsed_secs,
                self.memory_mb,
                self.cpu_percent,
            );
            render_sparkline(frame, sparkline_rect, &self.sparkline_data, "Throughput");

            // Right column: logs
            render_logs(frame, logs_col, &self.logs, self.log_scroll_offset);
        } else {
            // No logs, show full metrics + sparkline
            let (metrics_rect, sparkline_rect) = Self::compute_metrics_layout(info_area);
            let elapsed_secs = self.elapsed().map_or(0.0, |d| d.as_secs_f64());
            render_metrics(
                frame,
                metrics_rect,
                elapsed_secs,
                self.memory_mb,
                self.cpu_percent,
            );
            render_sparkline(frame, sparkline_rect, &self.sparkline_data, "Throughput");
        }

        // Footer
        render_footer(frame, footer_area);
    }

    /// Set up the terminal for TUI mode.
    ///
    /// Returns a configured Terminal or an error.
    pub fn setup_terminal() -> io::Result<Terminal<CrosstermBackend<io::Stdout>>> {
        terminal::enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        Terminal::new(backend)
    }

    /// Tear down the terminal, restoring normal mode.
    pub fn teardown_terminal(
        terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    ) -> io::Result<()> {
        terminal::disable_raw_mode()?;
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;
        terminal.show_cursor()?;
        Ok(())
    }

    /// Run the TUI event loop.
    ///
    /// This sets up the terminal, runs the main loop (poll events, update, render),
    /// and tears down on exit.
    pub fn run(&mut self) -> io::Result<()> {
        let mut terminal = Self::setup_terminal()?;

        let tick_rate = Duration::from_millis(250);

        loop {
            // Render
            terminal.draw(|frame| {
                self.render(frame);
            })?;

            if self.should_quit {
                break;
            }

            // Poll for events with tick rate timeout
            if event::poll(tick_rate)? {
                match event::read()? {
                    Event::Key(key_event) => {
                        let action = map_key(key_event);
                        self.handle_key_action(action);
                    }
                    Event::Resize(w, h) => {
                        self.terminal_width = w;
                        self.terminal_height = h;
                    }
                    _ => {}
                }
            }

            // Process any pending messages from the calculation
            self.update();
        }

        Self::teardown_terminal(&mut terminal)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::messages::SystemMetrics;
    use crossbeam_channel::unbounded;

    fn make_app() -> (TuiApp, crossbeam_channel::Sender<TuiMessage>) {
        let (tx, rx) = unbounded();
        let app = TuiApp::new(rx);
        (app, tx)
    }

    #[test]
    fn initial_state() {
        let (app, _tx) = make_app();
        assert!(!app.should_quit);
        assert!(!app.paused);
        assert!(app.progress.is_empty());
        assert!(app.algorithms.is_empty());
        assert!(app.logs.is_empty());
        assert!(app.start_time.is_none());
        assert_eq!(app.generation(), 0);
        assert!(app.show_details);
        assert!(app.show_logs);
    }

    #[test]
    fn handle_progress_message() {
        let (mut app, tx) = make_app();
        tx.send(TuiMessage::Progress {
            index: 0,
            progress: 0.5,
            algorithm: "FastDoubling".to_string(),
        })
        .unwrap();
        app.update();
        assert_eq!(app.progress.len(), 1);
        assert!((app.progress[0] - 0.5).abs() < f64::EPSILON);
        assert_eq!(app.algorithms[0], "FastDoubling");
    }

    #[test]
    fn handle_progress_sparse_index() {
        let (mut app, tx) = make_app();
        tx.send(TuiMessage::Progress {
            index: 3,
            progress: 0.8,
            algorithm: "Matrix".to_string(),
        })
        .unwrap();
        app.update();
        assert_eq!(app.progress.len(), 4);
        assert!((app.progress[3] - 0.8).abs() < f64::EPSILON);
        // Earlier entries should be zero-initialized
        assert!((app.progress[0]).abs() < f64::EPSILON);
    }

    #[test]
    fn handle_log_message() {
        let (mut app, tx) = make_app();
        tx.send(TuiMessage::Log("test log".to_string())).unwrap();
        app.update();
        assert_eq!(app.logs.len(), 1);
        assert_eq!(app.logs[0], "test log");
    }

    #[test]
    fn handle_started_increments_generation() {
        let (mut app, tx) = make_app();
        // Add some state
        tx.send(TuiMessage::Progress {
            index: 0,
            progress: 0.5,
            algorithm: "Test".to_string(),
        })
        .unwrap();
        app.update();
        assert_eq!(app.generation(), 0);

        tx.send(TuiMessage::Started).unwrap();
        app.update();
        assert_eq!(app.generation(), 1);
        // Progress should be cleared on new generation
        assert!(app.progress.is_empty());
        assert!(app.algorithms.is_empty());
    }

    #[test]
    fn handle_quit() {
        let (mut app, tx) = make_app();
        tx.send(TuiMessage::Quit).unwrap();
        app.update();
        assert!(app.should_quit);
    }

    #[test]
    fn handle_complete() {
        let (mut app, tx) = make_app();
        tx.send(TuiMessage::Complete {
            algorithm: "FastDoubling".to_string(),
            duration: Duration::from_millis(100),
        })
        .unwrap();
        app.update();
        assert_eq!(app.completed.len(), 1);
        assert_eq!(app.completed[0].0, "FastDoubling");
    }

    #[test]
    fn handle_resize() {
        let (mut app, _tx) = make_app();
        app.handle_message(TuiMessage::Resize {
            width: 120,
            height: 40,
        });
        assert_eq!(app.terminal_width, 120);
        assert_eq!(app.terminal_height, 40);
    }

    #[test]
    fn handle_error() {
        let (mut app, _tx) = make_app();
        app.handle_message(TuiMessage::Error("something broke".to_string()));
        assert_eq!(app.errors.len(), 1);
        assert_eq!(app.errors[0], "something broke");
        // Error also appears in logs
        assert!(app.logs.iter().any(|l| l.contains("something broke")));
    }

    #[test]
    fn handle_system_metrics() {
        let (mut app, _tx) = make_app();
        app.handle_message(TuiMessage::SystemMetrics(SystemMetrics {
            cpu_percent: 75.0,
            memory_mb: 2048.0,
            throughput_bits_per_sec: 500_000.0,
        }));
        assert!((app.cpu_percent - 75.0).abs() < f64::EPSILON);
        assert!((app.memory_mb - 2048.0).abs() < f64::EPSILON);
        // Throughput should be added to sparkline
        assert_eq!(app.sparkline_data.len(), 1);
    }

    #[test]
    fn key_action_quit() {
        let (mut app, _tx) = make_app();
        app.handle_key_action(KeyAction::Quit);
        assert!(app.should_quit);
    }

    #[test]
    fn key_action_pause_resume() {
        let (mut app, _tx) = make_app();
        assert!(!app.paused);
        app.handle_key_action(KeyAction::Pause);
        assert!(app.paused);
        app.handle_key_action(KeyAction::Resume);
        assert!(!app.paused);
    }

    #[test]
    fn key_action_toggle_details() {
        let (mut app, _tx) = make_app();
        assert!(app.show_details);
        app.handle_key_action(KeyAction::ToggleDetails);
        assert!(!app.show_details);
        app.handle_key_action(KeyAction::ToggleDetails);
        assert!(app.show_details);
    }

    #[test]
    fn key_action_toggle_logs() {
        let (mut app, _tx) = make_app();
        assert!(app.show_logs);
        app.handle_key_action(KeyAction::ToggleLogs);
        assert!(!app.show_logs);
    }

    #[test]
    fn scroll_up_down() {
        let (mut app, _tx) = make_app();
        // Add some logs
        for i in 0..20 {
            app.handle_message(TuiMessage::Log(format!("log {i}")));
        }
        // Auto-scroll should have us at the bottom
        assert!(app.log_auto_scroll);

        // Scroll up
        app.handle_key_action(KeyAction::ScrollUp);
        assert!(!app.log_auto_scroll);

        // Scroll down to bottom re-enables auto-scroll
        for _ in 0..30 {
            app.handle_key_action(KeyAction::ScrollDown);
        }
        assert!(app.log_auto_scroll);
    }

    #[test]
    fn page_up_down() {
        let (mut app, _tx) = make_app();
        for i in 0..50 {
            app.handle_message(TuiMessage::Log(format!("log {i}")));
        }
        app.page_up(10);
        assert!(!app.log_auto_scroll);

        app.page_down(100);
        assert!(app.log_auto_scroll);
    }

    #[test]
    fn scroll_home_end() {
        let (mut app, _tx) = make_app();
        for i in 0..20 {
            app.handle_message(TuiMessage::Log(format!("log {i}")));
        }
        app.scroll_home();
        assert_eq!(app.log_scroll_offset, 0);
        assert!(!app.log_auto_scroll);

        app.scroll_end();
        assert!(app.log_auto_scroll);
    }

    #[test]
    fn sparkline_ring_buffer() {
        let (mut app, _tx) = make_app();
        for i in 0..70 {
            app.handle_message(TuiMessage::SparklineData(i as f64));
        }
        assert_eq!(app.sparkline_data.len(), 60);
    }

    #[test]
    fn elapsed_none_before_start() {
        let (app, _tx) = make_app();
        assert!(app.elapsed().is_none());
    }

    #[test]
    fn elapsed_some_after_start() {
        let (mut app, _tx) = make_app();
        app.handle_message(TuiMessage::Started);
        assert!(app.elapsed().is_some());
    }

    #[test]
    fn layout_computation() {
        let area = Rect::new(0, 0, 80, 24);
        let (header, progress, info, footer) = TuiApp::compute_layout(area);

        // Header should be at the top
        assert_eq!(header.y, 0);
        assert_eq!(header.height, 3);

        // Footer at the bottom
        assert_eq!(footer.height, 2);
        assert_eq!(footer.y + footer.height, area.height);

        // Progress and info fill the middle
        assert!(progress.height > 0);
        assert!(info.height > 0);
        assert_eq!(
            header.height + progress.height + info.height + footer.height,
            area.height
        );
    }

    #[test]
    fn info_layout_computation() {
        let area = Rect::new(0, 0, 100, 20);
        let (metrics, logs) = TuiApp::compute_info_layout(area);
        assert!(metrics.width > 0);
        assert!(logs.width > 0);
        assert_eq!(metrics.width + logs.width, area.width);
    }

    #[test]
    fn log_cap_at_500() {
        let (mut app, _tx) = make_app();
        for i in 0..510 {
            app.handle_message(TuiMessage::Log(format!("msg {i}")));
        }
        assert_eq!(app.logs.len(), 500);
    }

    #[test]
    fn cancel_action_quits() {
        let (mut app, _tx) = make_app();
        app.handle_key_action(KeyAction::Cancel);
        assert!(app.should_quit);
    }

    #[test]
    fn set_n_value() {
        let (mut app, _tx) = make_app();
        app.set_n(1000);
        assert_eq!(app.n_value, 1000);
    }
}
