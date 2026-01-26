//! Header widget with title and keyboard shortcuts.

use ratatui::{
    layout::Rect,
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::tui::app::App;
use crate::tui::style::styles;

/// Render the header bar.
pub fn render_header(app: &App, frame: &mut Frame, area: Rect) {
    let title = format!(" FibRust v{} ", app.system_info.version);

    let shortcuts = vec![
        Span::styled("[", styles::text_dim()),
        Span::styled("q", styles::shortcut_key()),
        Span::styled("] Quit  ", styles::shortcut_desc()),
        Span::styled("[", styles::text_dim()),
        Span::styled("r", styles::shortcut_key()),
        Span::styled("] Run  ", styles::shortcut_desc()),
        Span::styled("[", styles::text_dim()),
        Span::styled("?", styles::shortcut_key()),
        Span::styled("] Help", styles::shortcut_desc()),
    ];

    let header_line = Line::from(shortcuts);

    let block = Block::default()
        .title(Span::styled(title, styles::title()))
        .title_alignment(ratatui::layout::Alignment::Left)
        .borders(Borders::ALL)
        .border_style(styles::border());

    let paragraph = Paragraph::new(header_line)
        .block(block)
        .alignment(ratatui::layout::Alignment::Right);

    frame.render_widget(paragraph, area);
}
