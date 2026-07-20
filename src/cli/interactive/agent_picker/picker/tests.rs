use super::*;
use crate::cli::interactive::agent_picker::AgentProvenance;
use crate::session::Harness;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{backend::TestBackend, layout::Rect, Terminal, TerminalOptions, Viewport};
use std::collections::BTreeSet;

fn row(name: &str) -> AgentPickerRow {
    AgentPickerRow {
        name: name.into(),
        description: format!("{name} description"),
        status: None,
        has_configured: false,
        has_native_profile: false,
    }
}

fn key(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::NONE)
}

#[test]
fn edit_delete_and_filter_keys_are_distinct() {
    let mut edit = PickerState::new(vec![row("one")], 0);
    assert_eq!(
        edit.handle_key(key(KeyCode::Char('e')), 10),
        Some(PickerAction::Edit(0))
    );

    let mut delete = PickerState::new(
        vec![AgentPickerRow {
            has_configured: true,
            ..row("one")
        }],
        0,
    );
    assert_eq!(delete.handle_key(key(KeyCode::Char('d')), 10), None);
    assert_eq!(
        delete.handle_key(key(KeyCode::Char('y')), 10),
        Some(PickerAction::Delete(vec![(0, DeleteScope::Agent)]))
    );

    let mut filter = PickerState::new(vec![row("editor"), row("writer")], 0);
    filter.handle_key(key(KeyCode::Char('/')), 10);
    filter.handle_key(key(KeyCode::Char('e')), 10);
    assert!(filter.filtering);
    assert_eq!(filter.query, "e");
    assert_eq!(filter.visible, vec![0, 1]);
}

#[test]
fn pending_delete_confirm_only_accepts_y_or_d_and_esc_cancels() {
    let mut state = PickerState::new(
        vec![AgentPickerRow {
            has_configured: true,
            ..row("one")
        }],
        0,
    );
    state.handle_key(key(KeyCode::Char('d')), 10);
    assert!(state.pending_delete.is_some());

    // An unrelated key neither confirms nor dismisses the prompt.
    assert_eq!(state.handle_key(key(KeyCode::Char('x')), 10), None);
    assert!(state.pending_delete.is_some());

    assert_eq!(state.handle_key(key(KeyCode::Esc), 10), None);
    assert!(state.pending_delete.is_none());

    state.handle_key(key(KeyCode::Char('d')), 10);
    assert_eq!(
        state.handle_key(key(KeyCode::Char('d')), 10),
        Some(PickerAction::Delete(vec![(0, DeleteScope::Agent)]))
    );
}

#[test]
fn space_toggles_multi_select_and_bulk_deletes_selected_rows() {
    let mut state = PickerState::new(
        vec![
            AgentPickerRow {
                has_configured: true,
                ..row("one")
            },
            AgentPickerRow {
                has_native_profile: true,
                ..row("two")
            },
            row("three"), // left unselected; generic, nothing to delete
        ],
        0,
    );

    state.handle_key(key(KeyCode::Char(' ')), 10);
    state.move_down(1);
    state.handle_key(key(KeyCode::Char(' ')), 10);
    assert_eq!(state.selected, BTreeSet::from([0usize, 1]));

    state.handle_key(key(KeyCode::Char('d')), 10);
    assert_eq!(
        state.handle_key(key(KeyCode::Char('y')), 10),
        Some(PickerAction::Delete(vec![
            (0, DeleteScope::Both),
            (1, DeleteScope::Both)
        ]))
    );
    // The selection is cleared once the delete is dispatched.
    assert!(state.selected.is_empty());
}

#[test]
fn initial_cursor_is_honored_and_clamped() {
    let state = PickerState::new(vec![row("one"), row("two"), row("three")], 2);
    assert_eq!(state.cursor, 2);

    let clamped = PickerState::new(vec![row("one")], 5);
    assert_eq!(clamped.cursor, 0);
}

#[test]
fn viewport_fits_the_roster_without_filling_a_tall_terminal() {
    assert_eq!(viewport_height(30, 11), 24);
    assert_eq!(viewport_height(50, 40), 32);
    assert_eq!(viewport_height(1, 11), 1);
    assert_eq!(viewport_height(30, 1), 8);
}

#[test]
fn number_keys_switch_between_all_three_proposals() {
    let mut state = PickerState::new(vec![row("one")], 0);
    assert_eq!(state.view, PickerView::Inspector);

    state.handle_key(key(KeyCode::Char('2')), 10);
    assert_eq!(state.view, PickerView::Briefs);
    state.handle_key(key(KeyCode::Char('3')), 10);
    assert_eq!(state.view, PickerView::Index);
    state.handle_key(key(KeyCode::Char('1')), 10);
    assert_eq!(state.view, PickerView::Inspector);

    state.handle_key(key(KeyCode::Char('/')), 10);
    state.handle_key(key(KeyCode::Char('2')), 10);
    assert_eq!(state.view, PickerView::Inspector);
    assert_eq!(state.query, "2");
}

#[test]
fn descriptions_are_cleaned_for_terminal_presentation() {
    let mut profile = row("architect");
    profile.description =
        "Designs systems.\\n\\n<example>\\nThis internal example should stay hidden.".into();

    assert_eq!(profile.clean_description(), "Designs systems.");
}

#[test]
fn proposal_rows_keep_semantic_harness_colors() {
    let row = AgentPickerRow {
        name: "reviewer".into(),
        description: "Reviews".into(),
        status: Some(AgentProvenance {
            label: "Claude".into(),
            harness: Harness::ClaudeCode,
        }),
        has_configured: false,
        has_native_profile: false,
    };

    assert_eq!(
        render::harness_style(&row).fg,
        Some(crate::console_style::harness_ratatui_color(
            Harness::ClaudeCode
        ))
    );
}

#[test]
fn every_proposal_renders_real_capability_metadata() {
    let mut state = PickerState::new(
        vec![
            AgentPickerRow {
                name: "reviewer".into(),
                description: "Reviews".into(),
                status: Some(AgentProvenance {
                    label: "Claude".into(),
                    harness: Harness::ClaudeCode,
                }),
                has_configured: false,
                has_native_profile: false,
            },
            AgentPickerRow {
                name: "builder".into(),
                description: "Builds".into(),
                status: Some(AgentProvenance {
                    label: "Codex".into(),
                    harness: Harness::Codex,
                }),
                has_configured: false,
                has_native_profile: false,
            },
        ],
        0,
    );
    let backend = TestBackend::new(120, 16);
    let mut terminal = Terminal::with_options(
        backend,
        TerminalOptions {
            viewport: Viewport::Fixed(Rect::new(0, 0, 120, 16)),
        },
    )
    .unwrap();

    for view in [PickerView::Inspector, PickerView::Briefs, PickerView::Index] {
        state.view = view;
        let rendered = terminal.draw(|frame| render::draw(frame, &state)).unwrap();
        let screen = rendered
            .buffer
            .content()
            .chunks(120)
            .map(|line| line.iter().map(|cell| cell.symbol()).collect::<String>())
            .collect::<Vec<_>>()
            .join("\n");
        assert!(screen.contains(view.label()));
        assert!(screen.contains("reviewer"));
        assert!(screen.contains("Claude"));
        assert!(screen.contains(if view == PickerView::Index {
            "[core]"
        } else {
            "built in"
        }));
        assert!(screen.contains("enter launch"));
    }
}

#[test]
fn inspector_collapses_to_briefs_on_narrow_terminals() {
    let state = PickerState::new(vec![row("one"), row("two")], 0);
    let backend = TestBackend::new(70, 8);
    let mut terminal = Terminal::with_options(
        backend,
        TerminalOptions {
            viewport: Viewport::Fixed(Rect::new(0, 0, 70, 8)),
        },
    )
    .unwrap();

    let rendered = terminal.draw(|frame| render::draw(frame, &state)).unwrap();
    let screen = rendered
        .buffer
        .content()
        .chunks(70)
        .map(|line| line.iter().map(|cell| cell.symbol()).collect::<String>())
        .collect::<Vec<_>>()
        .join("\n");
    assert!(screen.contains("one description"));
    assert!(screen.contains("two description"));
}
