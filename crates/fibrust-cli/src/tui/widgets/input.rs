//! Input configuration widget.

use ratatui::{
    layout::Rect,
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::tui::app::App;
use crate::tui::state::{Algorithm, AppMode, EditField};
use crate::tui::style::styles;

/// Render the input configuration panel.
pub fn render_input(app: &App, frame: &mut Frame, area: Rect) {
    let is_editing_n = app.mode == AppMode::Editing && app.input.focus == EditField::N;
    let is_editing_algo = app.mode == AppMode::Editing && app.input.focus == EditField::Algorithm;

    // N input field
    let n_display = if app.input.n_str.is_empty() {
        "_".to_string()
    } else {
        format_with_commas(&app.input.n_str)
    };

    let n_style = if is_editing_n || app.input.focus == EditField::N {
        styles::input_field_focused()
    } else {
        styles::input_field()
    };

    let cursor = if is_editing_n { "_" } else { "" };

    let mut lines = vec![
        Line::from(vec![
            Span::styled("n: ", styles::label()),
            Span::styled("[", styles::text_dim()),
            Span::styled(format!("{}{}", n_display, cursor), n_style),
            Span::styled("]", styles::text_dim()),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Algorithm: ", styles::label()),
        ]),
    ];

    // Algorithm options
    for (i, algo) in Algorithm::all_variants().iter().enumerate() {
        let is_selected = i == app.input.algorithm_index;
        let marker = if is_selected { ">" } else { " " };
        let radio = if is_selected { "(*)" } else { "( )" };

        let style = if is_selected && (is_editing_algo || app.input.focus == EditField::Algorithm) {
            styles::selected()
        } else if is_selected {
            styles::value()
        } else {
            styles::text_dim()
        };

        lines.push(Line::from(vec![
            Span::styled(format!(" {} {} ", marker, radio), style),
            Span::styled(format!("{} ", algo.name()), style),
            Span::styled(format!("- {}", algo.description()), styles::text_dim()),
        ]));
    }

    let border_style = if is_editing_n || is_editing_algo {
        styles::border_focused()
    } else {
        styles::border()
    };

    let block = Block::default()
        .title(Span::styled(" INPUT ", styles::header()))
        .borders(Borders::ALL)
        .border_style(border_style);

    let paragraph = Paragraph::new(lines).block(block);

    frame.render_widget(paragraph, area);
}

fn format_with_commas(s: &str) -> String {
    let chars: Vec<char> = s.chars().collect();
    let mut result = String::new();
    for (i, c) in chars.iter().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            result.push(',');
        }
        result.push(*c);
    }
    result.chars().rev().collect()
}
