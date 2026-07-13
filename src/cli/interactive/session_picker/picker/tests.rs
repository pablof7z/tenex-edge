use super::*;
use crate::cli::interactive::session_picker::data::SessionRow;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{backend::TestBackend, layout::Rect, Terminal, TerminalOptions, Viewport};

fn choice(handle: &str, activity: &str) -> SessionChoice {
    SessionChoice {
        label: format!("@{handle}"),
        row: SessionRow {
            handle: handle.into(),
            activity: activity.into(),
            ..SessionRow::default()
        },
    }
}

fn key(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::NONE)
}

#[test]
fn viewport_is_exactly_half_height_when_the_picker_can_fit() {
    assert_eq!(viewport_height(50), 25);
    assert_eq!(viewport_height(31), 15);
    assert_eq!(viewport_height(7), 7);
}

#[test]
fn filtering_uses_hidden_fields_and_prefers_handle_matches() {
    let cwd = SessionChoice {
        label: "@opal".into(),
        row: SessionRow {
            handle: "opal".into(),
            cwd: Some("/repo/edge".into()),
            ..SessionRow::default()
        },
    };
    let mut state = PickerState::new(vec![
        cwd,
        choice("delta-codex", "ordinary work"),
        choice("other-codex", "reviewing delta output"),
    ]);

    for character in "rpedge".chars() {
        state.handle_key(key(KeyCode::Char(character)), 10);
    }
    assert_eq!(state.visible, vec![0]);

    for _ in 0..6 {
        state.handle_key(key(KeyCode::Backspace), 10);
    }
    for character in "delta".chars() {
        state.handle_key(key(KeyCode::Char(character)), 10);
    }
    assert_eq!(state.visible[0], 1);
}

#[test]
fn selection_controls_never_create_selectable_filler_rows() {
    let mut state = PickerState::new(vec![choice("one", ""), choice("two", "")]);
    state.handle_key(key(KeyCode::Char(' ')), 20);
    assert_eq!(state.selected, BTreeSet::from([0]));

    state.handle_key(key(KeyCode::Right), 20);
    assert_eq!(state.selected, BTreeSet::from([0, 1]));

    state.handle_key(key(KeyCode::Left), 20);
    assert!(state.selected.is_empty());
    assert_eq!(state.window(20).count(), 2);
}

#[test]
fn cursor_scrolls_inside_the_reserved_option_area() {
    let choices = (0..8)
        .map(|index| choice(&format!("session-{index}"), ""))
        .collect();
    let mut state = PickerState::new(choices);
    for _ in 0..5 {
        state.handle_key(key(KeyCode::Down), 3);
    }
    assert_eq!(state.cursor, 5);
    assert_eq!(state.offset, 3);
    assert_eq!(state.window(3).count(), 3);
}

#[test]
fn renderer_keeps_unused_option_rows_blank_in_fixed_height_frame() {
    let state = PickerState::new(vec![choice("one", "working")]);
    let backend = TestBackend::new(80, 12);
    let mut terminal = Terminal::with_options(
        backend,
        TerminalOptions {
            viewport: Viewport::Fixed(Rect::new(0, 0, 80, 12)),
        },
    )
    .unwrap();

    let completed = terminal
        .draw(|frame| render::draw(frame, &state, "SESSION  STATE  CURRENT WORK"))
        .unwrap();

    assert_eq!(completed.area.height, 12);
    let rows = completed
        .buffer
        .content()
        .chunks(80)
        .map(|cells| cells.iter().map(|cell| cell.symbol()).collect::<String>())
        .collect::<Vec<_>>();

    assert!(rows[0].starts_with("Select sessions to kill"));
    assert!(rows[2].starts_with("❯ [ ] @one"));
    assert!(rows[3..11].iter().all(|row| row.trim().is_empty()));
    assert!(rows[11].starts_with("type filter"));
}
