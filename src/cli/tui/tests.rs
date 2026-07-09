use super::app::input_escape_key;
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};

fn key(code: KeyCode, modifiers: KeyModifiers) -> KeyEvent {
    KeyEvent {
        code,
        modifiers,
        kind: KeyEventKind::Press,
        state: KeyEventState::NONE,
    }
}

#[test]
fn input_escape_accepts_reliable_local_controls() {
    assert!(input_escape_key(key(KeyCode::Esc, KeyModifiers::NONE)));
    assert!(input_escape_key(key(
        KeyCode::Char('g'),
        KeyModifiers::CONTROL
    )));
    assert!(input_escape_key(key(
        KeyCode::Char('G'),
        KeyModifiers::CONTROL
    )));
    assert!(input_escape_key(key(
        KeyCode::Char(']'),
        KeyModifiers::CONTROL
    )));
    assert!(input_escape_key(key(
        KeyCode::Char('\u{1d}'),
        KeyModifiers::NONE
    )));
}

#[test]
fn input_escape_does_not_steal_regular_text_or_alt_escape() {
    assert!(!input_escape_key(key(
        KeyCode::Char('g'),
        KeyModifiers::NONE
    )));
    assert!(!input_escape_key(key(
        KeyCode::Char(']'),
        KeyModifiers::NONE
    )));
    assert!(!input_escape_key(key(KeyCode::Esc, KeyModifiers::ALT)));
}
