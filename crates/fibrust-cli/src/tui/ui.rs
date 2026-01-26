//! Main UI rendering function for the TUI.

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};

use super::app::App;
use super::state::AppMode;
use super::style::styles;
use super::widgets;

/// Main render function that draws the entire UI.
pub fn render(app: &mut App, frame: &mut Frame) {
    let area = frame.area();

    // Main layout: vertical split
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Header
            Constraint::Length(10), // System + Input (side by side)
            Constraint::Length(8),  // Progress
            Constraint::Length(8),  // Comparison
            Constraint::Min(6),     // Result
        ])
        .split(area);

    // Render header
    widgets::render_header(app, frame, main_chunks[0]);

    // System info and Input side by side
    let top_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(main_chunks[1]);

    widgets::render_system_info(app, frame, top_chunks[0]);
    widgets::render_input(app, frame, top_chunks[1]);

    // Progress
    widgets::render_progress(app, frame, main_chunks[2]);

    // Comparison
    widgets::render_comparison(app, frame, main_chunks[3]);

    // Result
    widgets::render_result(app, frame, main_chunks[4]);

    // Help overlay if active
    if app.mode == AppMode::Help {
        render_help_overlay(frame, area);
    }
}

/// Render the help overlay.
fn render_help_overlay(frame: &mut Frame, area: Rect) {
    // Calculate centered popup area
    let popup_width = 60.min(area.width.saturating_sub(4));
    let popup_height = 20.min(area.height.saturating_sub(4));
    let popup_x = (area.width.saturating_sub(popup_width)) / 2;
    let popup_y = (area.height.saturating_sub(popup_height)) / 2;

    let popup_area = Rect::new(popup_x, popup_y, popup_width, popup_height);

    // Clear the background
    frame.render_widget(Clear, popup_area);

    let help_text = vec![
        Line::from(Span::styled("Keyboard Shortcuts", styles::title())),
        Line::from(""),
        Line::from(vec![
            Span::styled("  q / Esc      ", styles::shortcut_key()),
            Span::styled("Quit application", styles::text()),
        ]),
        Line::from(vec![
            Span::styled("  r / Enter    ", styles::shortcut_key()),
            Span::styled("Run calculation", styles::text()),
        ]),
        Line::from(vec![
            Span::styled("  ? / F1       ", styles::shortcut_key()),
            Span::styled("Toggle this help", styles::text()),
        ]),
        Line::from(""),
        Line::from(Span::styled("Navigation", styles::label())),
        Line::from(vec![
            Span::styled("  Tab          ", styles::shortcut_key()),
            Span::styled("Next field", styles::text()),
        ]),
        Line::from(vec![
            Span::styled("  n            ", styles::shortcut_key()),
            Span::styled("Focus n input", styles::text()),
        ]),
        Line::from(vec![
            Span::styled("  a            ", styles::shortcut_key()),
            Span::styled("Focus algorithm selector", styles::text()),
        ]),
        Line::from(""),
        Line::from(Span::styled("Algorithm Selection", styles::label())),
        Line::from(vec![
            Span::styled("  Up/Down      ", styles::shortcut_key()),
            Span::styled("Navigate algorithms", styles::text()),
        ]),
        Line::from(vec![
            Span::styled("  1-5          ", styles::shortcut_key()),
            Span::styled("Select algorithm directly", styles::text()),
        ]),
        Line::from(""),
        Line::from(Span::styled("Input", styles::label())),
        Line::from(vec![
            Span::styled("  0-9          ", styles::shortcut_key()),
            Span::styled("Enter digits", styles::text()),
        ]),
        Line::from(vec![
            Span::styled("  Backspace    ", styles::shortcut_key()),
            Span::styled("Delete last digit", styles::text()),
        ]),
    ];

    let block = Block::default()
        .title(Span::styled(" Help ", styles::title()))
        .borders(Borders::ALL)
        .border_style(styles::border_focused());

    let paragraph = Paragraph::new(help_text)
        .block(block)
        .wrap(Wrap { trim: false });

    frame.render_widget(paragraph, popup_area);
}
