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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_theme_colors() {
        let theme = ColorTheme::default();
        assert_eq!(theme.primary, Color::Cyan);
        assert_eq!(theme.secondary, Color::Blue);
        assert_eq!(theme.success, Color::Green);
        assert_eq!(theme.error, Color::Red);
        assert_eq!(theme.warning, Color::Yellow);
        assert_eq!(theme.text, Color::White);
        assert_eq!(theme.muted, Color::DarkGray);
        assert_eq!(theme.border, Color::Gray);
    }

    #[test]
    fn header_style_uses_primary_bold() {
        let theme = ColorTheme::default();
        let style = theme.header_style();
        assert_eq!(style.fg, Some(Color::Cyan));
        assert!(style.add_modifier.contains(Modifier::BOLD));
    }

    #[test]
    fn text_style_uses_text_color() {
        let theme = ColorTheme::default();
        let style = theme.text_style();
        assert_eq!(style.fg, Some(Color::White));
    }

    #[test]
    fn muted_style_uses_muted_color() {
        let theme = ColorTheme::default();
        let style = theme.muted_style();
        assert_eq!(style.fg, Some(Color::DarkGray));
    }

    #[test]
    fn success_style_uses_success_color() {
        let theme = ColorTheme::default();
        let style = theme.success_style();
        assert_eq!(style.fg, Some(Color::Green));
    }

    #[test]
    fn error_style_uses_error_color() {
        let theme = ColorTheme::default();
        let style = theme.error_style();
        assert_eq!(style.fg, Some(Color::Red));
    }

    #[test]
    fn custom_theme_colors() {
        let theme = ColorTheme {
            primary: Color::Magenta,
            secondary: Color::White,
            success: Color::LightGreen,
            error: Color::LightRed,
            warning: Color::LightYellow,
            text: Color::Gray,
            muted: Color::Black,
            border: Color::White,
        };
        let style = theme.header_style();
        assert_eq!(style.fg, Some(Color::Magenta));

        let style = theme.error_style();
        assert_eq!(style.fg, Some(Color::LightRed));

        let style = theme.success_style();
        assert_eq!(style.fg, Some(Color::LightGreen));

        let style = theme.text_style();
        assert_eq!(style.fg, Some(Color::Gray));

        let style = theme.muted_style();
        assert_eq!(style.fg, Some(Color::Black));
    }
}
