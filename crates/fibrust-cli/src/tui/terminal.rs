//! Terminal setup, teardown, and main TUI loop.

use std::io::{stdout, Stdout};
use std::time::Duration;

use anyhow::Result;
use crossterm::{
    event::{self, Event as CrosstermEvent, KeyCode, KeyEvent, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};

use super::app::App;
use super::event::Event;
use super::ui;

/// Type alias for our terminal.
pub type Tui = Terminal<CrosstermBackend<Stdout>>;

/// Initialize the terminal for TUI mode.
pub fn init_terminal() -> Result<Tui> {
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let terminal = Terminal::new(backend)?;
    Ok(terminal)
}

/// Restore the terminal to its original state.
pub fn restore_terminal(terminal: &mut Tui) -> Result<()> {
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}

/// Poll for events with a timeout.
fn poll_event(timeout: Duration) -> Result<Option<Event>> {
    if event::poll(timeout)? {
        match event::read()? {
            CrosstermEvent::Key(key) => Ok(Some(Event::Key(key))),
            CrosstermEvent::Resize(w, h) => Ok(Some(Event::Resize(w, h))),
            _ => Ok(None),
        }
    } else {
        Ok(Some(Event::Tick))
    }
}

/// Main entry point for running the TUI.
pub fn run_tui() -> Result<()> {
    // Initialize terminal
    let mut terminal = init_terminal()?;

    // Create app state
    let mut app = App::new();

    // Pre-warm the system
    fibrust_core::prewarm_system();

    // Main loop
    let tick_rate = Duration::from_millis(100);

    loop {
        // Draw UI
        terminal.draw(|frame| ui::render(&mut app, frame))?;

        // Handle events
        if let Some(event) = poll_event(tick_rate)? {
            match event {
                Event::Key(key) => {
                    if handle_key_event(&mut app, key) {
                        break;
                    }
                }
                Event::Tick => {
                    app.tick();
                }
                Event::Resize(_, _) => {
                    // Terminal will auto-resize on next draw
                }
                _ => {}
            }
        }

        // Check if app should quit
        if app.should_quit {
            break;
        }
    }

    // Restore terminal
    restore_terminal(&mut terminal)?;
    Ok(())
}

/// Handle keyboard input. Returns true if the app should quit.
fn handle_key_event(app: &mut App, key: KeyEvent) -> bool {
    // Global shortcuts
    match key.code {
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            return true;
        }
        KeyCode::Char('q') if !app.is_editing() => {
            return true;
        }
        KeyCode::Esc => {
            if app.is_editing() {
                app.cancel_edit();
            } else if app.mode == super::state::AppMode::Help {
                app.mode = super::state::AppMode::Idle;
            } else {
                return true;
            }
        }
        KeyCode::Char('?') | KeyCode::F(1) if !app.is_editing() => {
            app.toggle_help();
        }
        _ => {}
    }

    // Mode-specific handling
    match app.mode {
        super::state::AppMode::Idle => handle_idle_keys(app, key),
        super::state::AppMode::Editing => handle_editing_keys(app, key),
        super::state::AppMode::Running => handle_running_keys(app, key),
        super::state::AppMode::Complete => handle_complete_keys(app, key),
        super::state::AppMode::Help => {} // Only Esc exits help, handled above
    }

    false
}

fn handle_idle_keys(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Char('r') | KeyCode::Enter => {
            app.start_calculation();
        }
        KeyCode::Tab => {
            app.next_field();
        }
        KeyCode::BackTab => {
            app.prev_field();
        }
        KeyCode::Char('n') => {
            app.focus_n();
        }
        KeyCode::Char('a') => {
            app.focus_algorithm();
        }
        KeyCode::Up | KeyCode::Char('k') => {
            app.select_prev_algorithm();
        }
        KeyCode::Down | KeyCode::Char('j') => {
            app.select_next_algorithm();
        }
        KeyCode::Char(c) if c.is_ascii_digit() => {
            // Direct algorithm selection with 1-5
            if let Some(idx) = c.to_digit(10) {
                let idx = idx as usize;
                if idx >= 1 && idx <= 5 {
                    app.input.algorithm_index = idx - 1;
                }
            }
        }
        _ => {
            // Start editing N field if typing a digit
            if let KeyCode::Char(c) = key.code {
                if c.is_ascii_digit() {
                    app.start_edit_n();
                    app.input.n_str.push(c);
                }
            }
        }
    }
}

fn handle_editing_keys(app: &mut App, key: KeyEvent) {
    match app.input.focus {
        super::state::EditField::N => match key.code {
            KeyCode::Char(c) if c.is_ascii_digit() => {
                app.input.n_str.push(c);
            }
            KeyCode::Backspace => {
                app.input.n_str.pop();
            }
            KeyCode::Delete => {
                app.input.n_str.clear();
            }
            KeyCode::Enter => {
                app.confirm_edit();
                app.start_calculation();
            }
            KeyCode::Tab => {
                app.confirm_edit();
                app.next_field();
            }
            _ => {}
        },
        super::state::EditField::Algorithm => match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                app.select_prev_algorithm();
            }
            KeyCode::Down | KeyCode::Char('j') => {
                app.select_next_algorithm();
            }
            KeyCode::Enter | KeyCode::Char(' ') => {
                app.confirm_edit();
            }
            KeyCode::Tab => {
                app.confirm_edit();
                app.next_field();
            }
            KeyCode::Char(c) if c.is_ascii_digit() => {
                if let Some(idx) = c.to_digit(10) {
                    let idx = idx as usize;
                    if idx >= 1 && idx <= 5 {
                        app.input.algorithm_index = idx - 1;
                    }
                }
            }
            _ => {}
        },
    }
}

fn handle_running_keys(app: &mut App, key: KeyEvent) {
    // Allow cancellation with Esc (handled in main handler)
    if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
        app.cancel_calculation();
    }
}

fn handle_complete_keys(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Char('r') | KeyCode::Enter => {
            app.start_calculation();
        }
        KeyCode::Char('n') => {
            app.reset();
        }
        KeyCode::Tab => {
            app.next_field();
        }
        _ => {}
    }
}
