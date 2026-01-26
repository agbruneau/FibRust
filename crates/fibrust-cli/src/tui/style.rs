//! TUI styling and color definitions.

use ratatui::style::{Color, Modifier, Style};

/// Color palette for the TUI.
pub mod colors {
    use super::Color;

    // Primary colors
    pub const PRIMARY: Color = Color::Cyan;
    pub const SECONDARY: Color = Color::Yellow;
    pub const ACCENT: Color = Color::Magenta;

    // Status colors
    pub const SUCCESS: Color = Color::Green;
    pub const WARNING: Color = Color::Yellow;
    pub const ERROR: Color = Color::Red;
    pub const RUNNING: Color = Color::Blue;

    // UI elements
    pub const BORDER: Color = Color::DarkGray;
    pub const BORDER_FOCUSED: Color = Color::Cyan;
    pub const TEXT: Color = Color::White;
    pub const TEXT_DIM: Color = Color::Gray;
    pub const HIGHLIGHT_BG: Color = Color::Cyan;
    pub const HIGHLIGHT_FG: Color = Color::Black;

    // Progress bar
    pub const PROGRESS_FILLED: Color = Color::Green;
    pub const PROGRESS_EMPTY: Color = Color::DarkGray;

    // Table
    pub const TABLE_HEADER: Color = Color::Yellow;
    pub const TABLE_SELECTED: Color = Color::Cyan;
}

/// Pre-defined styles.
pub mod styles {
    use super::*;

    pub fn title() -> Style {
        Style::default()
            .fg(colors::PRIMARY)
            .add_modifier(Modifier::BOLD)
    }

    pub fn header() -> Style {
        Style::default()
            .fg(colors::PRIMARY)
            .add_modifier(Modifier::BOLD)
    }

    pub fn text() -> Style {
        Style::default().fg(colors::TEXT)
    }

    pub fn text_dim() -> Style {
        Style::default().fg(colors::TEXT_DIM)
    }

    pub fn label() -> Style {
        Style::default()
            .fg(colors::SECONDARY)
            .add_modifier(Modifier::BOLD)
    }

    pub fn value() -> Style {
        Style::default().fg(colors::TEXT)
    }

    pub fn border() -> Style {
        Style::default().fg(colors::BORDER)
    }

    pub fn border_focused() -> Style {
        Style::default().fg(colors::BORDER_FOCUSED)
    }

    pub fn success() -> Style {
        Style::default().fg(colors::SUCCESS)
    }

    pub fn warning() -> Style {
        Style::default().fg(colors::WARNING)
    }

    pub fn error() -> Style {
        Style::default().fg(colors::ERROR)
    }

    pub fn running() -> Style {
        Style::default()
            .fg(colors::RUNNING)
            .add_modifier(Modifier::BOLD)
    }

    pub fn selected() -> Style {
        Style::default()
            .fg(colors::HIGHLIGHT_FG)
            .bg(colors::HIGHLIGHT_BG)
    }

    pub fn shortcut_key() -> Style {
        Style::default()
            .fg(colors::SECONDARY)
            .add_modifier(Modifier::BOLD)
    }

    pub fn shortcut_desc() -> Style {
        Style::default().fg(colors::TEXT_DIM)
    }

    pub fn input_field() -> Style {
        Style::default().fg(colors::TEXT)
    }

    pub fn input_field_focused() -> Style {
        Style::default()
            .fg(colors::PRIMARY)
            .add_modifier(Modifier::BOLD)
    }

    pub fn progress_filled() -> Style {
        Style::default().fg(colors::PROGRESS_FILLED)
    }

    pub fn progress_empty() -> Style {
        Style::default().fg(colors::PROGRESS_EMPTY)
    }

    pub fn table_header() -> Style {
        Style::default()
            .fg(colors::TABLE_HEADER)
            .add_modifier(Modifier::BOLD)
    }

    pub fn table_row() -> Style {
        Style::default().fg(colors::TEXT)
    }

    pub fn table_selected() -> Style {
        Style::default()
            .fg(colors::HIGHLIGHT_FG)
            .bg(colors::TABLE_SELECTED)
    }
}
