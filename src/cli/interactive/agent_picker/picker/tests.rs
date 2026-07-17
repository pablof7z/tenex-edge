use super::*;
use crate::cli::interactive::agent_picker::AgentProvenance;
use crate::session::Harness;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{backend::TestBackend, layout::Rect, Terminal, TerminalOptions, Viewport};

fn row(name: &str) -> AgentPickerRow {
    AgentPickerRow {
        name: name.into(),
        description: format!("{name} description"),
        provenance: None,
        status: None,
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
fn viewport_uses_terminal_height_up_to_forty_agent_rows() {
    assert_eq!(viewport_height(30), 30);
    assert_eq!(viewport_height(50), 42);
    assert_eq!(viewport_height(1), 1);
    assert_eq!(option_rows(42), 40);
}

#[test]
fn renderer_orders_description_and_colored_provenance() {
    let state = PickerState::new(
        vec![AgentPickerRow {
            name: "writer".into(),
            description: "Drafts release notes".into(),
            provenance: Some(AgentProvenance {
                label: "Claude profile".into(),
                harness: Harness::ClaudeCode,
            }),
            status: None,
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
    let description_column = line[..line.find("Drafts release notes").unwrap()]
        .chars()
        .count();
    assert_eq!(
        completed.buffer.content()[100 + description_column].fg,
        ratatui::style::Color::Indexed(245)
    );
    let provenance_column = line[..line.find("Claude profile").unwrap()].chars().count();
    assert_eq!(
        completed.buffer.content()[100 + provenance_column].fg,
        crate::console_style::harness_ratatui_color(Harness::ClaudeCode)
    );
}

#[test]
fn manage_status_tracks_the_focused_harness_configuration() {
    let mut state = PickerState::new(
        vec![
            AgentPickerRow {
                name: "reviewer".into(),
                description: "Reviews".into(),
                provenance: None,
                status: Some(AgentProvenance {
                    label: "Harness config: claude-acp · acp · per-session key".into(),
                    harness: Harness::ClaudeCode,
                }),
            },
            AgentPickerRow {
                name: "builder".into(),
                description: "Builds".into(),
                provenance: None,
                status: Some(AgentProvenance {
                    label: "Harness config: codex-app · app-server · persistent key".into(),
                    harness: Harness::Codex,
                }),
            },
        ],
        PickerMode::Manage,
    );
    let backend = TestBackend::new(100, 5);
    let mut terminal = Terminal::with_options(
        backend,
        TerminalOptions {
            viewport: Viewport::Fixed(Rect::new(0, 0, 100, 5)),
        },
    )
    .unwrap();

    let first = terminal.draw(|frame| render::draw(frame, &state)).unwrap();
    let first_status = first.buffer.content()[400..500]
        .iter()
        .map(|cell| cell.symbol())
        .collect::<String>();
    assert!(first_status.contains("Harness config: claude-acp · acp · per-session key"));
    assert_eq!(
        first.buffer.content()[400].fg,
        crate::console_style::harness_ratatui_color(Harness::ClaudeCode)
    );

    state.handle_key(key(KeyCode::Down), 3);
    let second = terminal.draw(|frame| render::draw(frame, &state)).unwrap();
    let second_status = second.buffer.content()[400..500]
        .iter()
        .map(|cell| cell.symbol())
        .collect::<String>();
    assert!(second_status.contains("Harness config: codex-app · app-server · persistent key"));
    assert_eq!(
        second.buffer.content()[400].fg,
        crate::console_style::harness_ratatui_color(Harness::Codex)
    );
}
