//! Keyboard shortcut handling.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// TUI keyboard actions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyAction {
    Quit,
    Pause,
    Resume,
    ToggleDetails,
    ToggleLogs,
    Cancel,
    ScrollUp,
    ScrollDown,
    PageUp,
    PageDown,
    Home,
    End,
    None,
}

/// Map a key event to an action.
#[must_use]
pub fn map_key(key: KeyEvent) -> KeyAction {
    match key.code {
        KeyCode::Char('q') | KeyCode::Esc => KeyAction::Quit,
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => KeyAction::Cancel,
        KeyCode::Char('p') => KeyAction::Pause,
        KeyCode::Char('r') => KeyAction::Resume,
        KeyCode::Char('d') => KeyAction::ToggleDetails,
        KeyCode::Char('l') => KeyAction::ToggleLogs,
        KeyCode::Up => KeyAction::ScrollUp,
        KeyCode::Down => KeyAction::ScrollDown,
        KeyCode::PageUp => KeyAction::PageUp,
        KeyCode::PageDown => KeyAction::PageDown,
        KeyCode::Home => KeyAction::Home,
        KeyCode::End => KeyAction::End,
        _ => KeyAction::None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quit_keys() {
        let event = KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE);
        assert_eq!(map_key(event), KeyAction::Quit);

        let event = KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE);
        assert_eq!(map_key(event), KeyAction::Quit);
    }

    #[test]
    fn ctrl_c_cancels() {
        let event = KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL);
        assert_eq!(map_key(event), KeyAction::Cancel);
    }

    #[test]
    fn pause_resume() {
        let event = KeyEvent::new(KeyCode::Char('p'), KeyModifiers::NONE);
        assert_eq!(map_key(event), KeyAction::Pause);

        let event = KeyEvent::new(KeyCode::Char('r'), KeyModifiers::NONE);
        assert_eq!(map_key(event), KeyAction::Resume);
    }

    #[test]
    fn toggle_keys() {
        let event = KeyEvent::new(KeyCode::Char('d'), KeyModifiers::NONE);
        assert_eq!(map_key(event), KeyAction::ToggleDetails);

        let event = KeyEvent::new(KeyCode::Char('l'), KeyModifiers::NONE);
        assert_eq!(map_key(event), KeyAction::ToggleLogs);
    }

    #[test]
    fn scroll_keys() {
        let event = KeyEvent::new(KeyCode::Up, KeyModifiers::NONE);
        assert_eq!(map_key(event), KeyAction::ScrollUp);

        let event = KeyEvent::new(KeyCode::Down, KeyModifiers::NONE);
        assert_eq!(map_key(event), KeyAction::ScrollDown);
    }

    #[test]
    fn page_keys() {
        let event = KeyEvent::new(KeyCode::PageUp, KeyModifiers::NONE);
        assert_eq!(map_key(event), KeyAction::PageUp);

        let event = KeyEvent::new(KeyCode::PageDown, KeyModifiers::NONE);
        assert_eq!(map_key(event), KeyAction::PageDown);
    }

    #[test]
    fn home_end_keys() {
        let event = KeyEvent::new(KeyCode::Home, KeyModifiers::NONE);
        assert_eq!(map_key(event), KeyAction::Home);

        let event = KeyEvent::new(KeyCode::End, KeyModifiers::NONE);
        assert_eq!(map_key(event), KeyAction::End);
    }

    #[test]
    fn unknown_key() {
        let event = KeyEvent::new(KeyCode::Char('z'), KeyModifiers::NONE);
        assert_eq!(map_key(event), KeyAction::None);
    }
}
