use super::*;
use crate::cli::interactive::agent_picker::AgentProvenance;
use crate::session::Harness;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{backend::TestBackend, layout::Rect, Terminal, TerminalOptions, Viewport};

fn row(name: &str) -> AgentPickerRow {
    AgentPickerRow {
        name: name.into(),
        description: format!("{name} description"),
        description_harness: None,
        provenance: None,
    }
}

fn key(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::NONE)
}

#[test]
fn launch_mode_only_returns_launch_or_cancel_actions() {
    let mut state = PickerState::new(vec![row("editor")], PickerMode::Launch);

    assert_eq!(state.handle_key(key(KeyCode::Char('e')), 10), None);
    assert_eq!(state.query, "e");
    assert_eq!(
        state.handle_key(key(KeyCode::Enter), 10),
        Some(PickerAction::Launch(0))
    );
}

#[test]
fn management_mode_reserves_edit_delete_and_uses_slash_for_filtering() {
    let mut edit = PickerState::new(vec![row("one")], PickerMode::Manage);
    assert_eq!(
        edit.handle_key(key(KeyCode::Char('e')), 10),
        Some(PickerAction::Edit(0))
    );

    let mut delete = PickerState::new(vec![row("one")], PickerMode::Manage);
    assert_eq!(
        delete.handle_key(key(KeyCode::Char('d')), 10),
        Some(PickerAction::Delete(0))
    );

    let mut filter = PickerState::new(vec![row("editor"), row("writer")], PickerMode::Manage);
    filter.handle_key(key(KeyCode::Char('/')), 10);
    filter.handle_key(key(KeyCode::Char('e')), 10);
    assert!(filter.filtering);
    assert_eq!(filter.query, "e");
    assert_eq!(filter.visible, vec![0, 1]);
}

#[test]
fn viewport_caps_at_sixteen_agent_rows() {
    assert_eq!(viewport_height(40, 30), 18);
    assert_eq!(viewport_height(40, 3), 5);
    assert_eq!(option_rows(18), 16);
}

#[test]
fn renderer_orders_description_and_colored_provenance() {
    let state = PickerState::new(
        vec![AgentPickerRow {
            name: "writer".into(),
            description: "Drafts release notes".into(),
            description_harness: None,
            provenance: Some(AgentProvenance {
                label: "Claude profile".into(),
                harness: Harness::ClaudeCode,
            }),
        }],
        PickerMode::Launch,
    );
    let backend = TestBackend::new(100, 5);
    let mut terminal = Terminal::with_options(
        backend,
        TerminalOptions {
            viewport: Viewport::Fixed(Rect::new(0, 0, 100, 5)),
        },
    )
    .unwrap();

    let completed = terminal.draw(|frame| render::draw(frame, &state)).unwrap();
    let line = completed.buffer.content()[100..200]
        .iter()
        .map(|cell| cell.symbol())
        .collect::<String>();
    assert!(line.contains("Drafts release notes · Claude profile"));
    let provenance_column = line[..line.find("Claude profile").unwrap()].chars().count();
    assert_eq!(
        completed.buffer.content()[100 + provenance_column].fg,
        crate::console_style::harness_ratatui_color(Harness::ClaudeCode)
    );
}

#[test]
fn renderer_colors_generic_agent_descriptions_by_harness() {
    let state = PickerState::new(
        vec![AgentPickerRow {
            name: "codex".into(),
            description: "Generic Codex agent".into(),
            description_harness: Some(Harness::Codex),
            provenance: None,
        }],
        PickerMode::Launch,
    );
    let backend = TestBackend::new(80, 5);
    let mut terminal = Terminal::with_options(
        backend,
        TerminalOptions {
            viewport: Viewport::Fixed(Rect::new(0, 0, 80, 5)),
        },
    )
    .unwrap();

    let completed = terminal.draw(|frame| render::draw(frame, &state)).unwrap();
    let line = completed.buffer.content()[80..160]
        .iter()
        .map(|cell| cell.symbol())
        .collect::<String>();
    let description_column = line[..line.find("Generic Codex agent").unwrap()]
        .chars()
        .count();
    assert_eq!(
        completed.buffer.content()[80 + description_column].fg,
        crate::console_style::harness_ratatui_color(Harness::Codex)
    );
}
