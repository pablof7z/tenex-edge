use super::app::{input_escape_key, App};
use super::data::SessionRow;
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};
use std::time::Duration;

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

#[tokio::test]
async fn kill_confirmation_stages_exact_marked_session_ids_and_cancels() {
    let mut app = App::new(Duration::from_secs(2));
    app.sessions = vec![
        SessionRow {
            session_id: "te-exact-1".into(),
            handle: "opal-codex".into(),
            title: "first".into(),
            ..SessionRow::default()
        },
        SessionRow {
            session_id: "te-exact-2".into(),
            handle: "river-claude".into(),
            title: "second".into(),
            ..SessionRow::default()
        },
    ];
    app.marked.insert("te-exact-2".into());

    app.handle_key(key(KeyCode::Char('K'), KeyModifiers::NONE))
        .await
        .unwrap();
    let pending = app.pending_kill.as_ref().unwrap();
    assert_eq!(pending.len(), 1);
    assert_eq!(pending[0].session_id, "te-exact-2");
    assert!(pending[0].label.contains("@river-claude"));

    app.handle_key(key(KeyCode::Down, KeyModifiers::NONE))
        .await
        .unwrap();
    assert!(app.pending_kill.is_none());
}
