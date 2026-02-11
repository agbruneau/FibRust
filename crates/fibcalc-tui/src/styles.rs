//! TUI styles and color themes.

use ratatui::style::{Color, Modifier, Style};

/// Color theme for the TUI.
pub struct ColorTheme {
    pub primary: Color,
    pub secondary: Color,
    pub success: Color,
    pub error: Color,
    pub warning: Color,
    pub text: Color,
    pub muted: Color,
    pub border: Color,
}

impl Default for ColorTheme {
    fn default() -> Self {
        Self {
            primary: Color::Cyan,
            secondary: Color::Blue,
            success: Color::Green,
            error: Color::Red,
            warning: Color::Yellow,
            text: Color::White,
            muted: Color::DarkGray,
            border: Color::Gray,
        }
    }
}

impl ColorTheme {
    /// Get the style for a header.
    #[must_use]
    pub fn header_style(&self) -> Style {
        Style::default()
            .fg(self.primary)
            .add_modifier(Modifier::BOLD)
    }

    /// Get the style for normal text.
    #[must_use]
    pub fn text_style(&self) -> Style {
        Style::default().fg(self.text)
    }

    /// Get the style for muted text.
    #[must_use]
    pub fn muted_style(&self) -> Style {
        Style::default().fg(self.muted)
    }

    /// Get the style for success text.
    #[must_use]
    pub fn success_style(&self) -> Style {
        Style::default().fg(self.success)
    }

    /// Get the style for error text.
    #[must_use]
    pub fn error_style(&self) -> Style {
        Style::default().fg(self.error)
    }
}
