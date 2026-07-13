use super::PickerState;
use ratatui::{
    layout::{Constraint, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Clear, List, ListItem, Paragraph},
    Frame,
};

const ACCENT: Color = Color::Indexed(45);
const SUCCESS: Color = Color::Indexed(78);
const MUTED: Color = Color::Indexed(245);
const HELP: &str = "type filter · ↑↓ move · space toggle · → all · ← none · enter · esc";

pub(super) fn draw(frame: &mut Frame<'_>, state: &PickerState, header: &str) {
    let area = frame.area();
    frame.render_widget(Clear, area);
    if area.height < 4 {
        frame.render_widget(Paragraph::new("Select sessions to kill"), area);
        return;
    }

    let [title_area, header_area, options_area, help_area] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Min(1),
        Constraint::Length(1),
    ])
    .areas(area);

    let query = if state.query.is_empty() {
        Span::styled("type to filter", Style::default().fg(MUTED))
    } else {
        Span::styled(state.query.as_str(), Style::default().fg(ACCENT))
    };
    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(
                "Select sessions to kill",
                Style::default().add_modifier(Modifier::BOLD),
            ),
            Span::raw("  Filter: "),
            query,
            Span::styled(
                format!("  {} selected", state.selected.len()),
                Style::default().fg(MUTED),
            ),
        ])),
        title_area,
    );
    frame.render_widget(
        Paragraph::new(format!("      {header}")).style(Style::default().fg(MUTED)),
        header_area,
    );

    let rows = usize::from(options_area.height);
    let items = state
        .window(rows)
        .map(|(visible_index, choice_index, choice)| {
            let focused = visible_index == state.cursor;
            let prefix = if focused {
                "❯"
            } else if visible_index == state.offset && state.offset > 0 {
                "↑"
            } else if visible_index + 1 == state.offset + rows
                && visible_index + 1 < state.visible.len()
            {
                "↓"
            } else {
                " "
            };
            let checked = state.selected.contains(&choice_index);
            ListItem::new(Line::from(vec![
                Span::styled(
                    format!("{prefix} "),
                    Style::default().fg(if focused { ACCENT } else { MUTED }),
                ),
                Span::styled(
                    if checked { "[x] " } else { "[ ] " },
                    Style::default().fg(if checked { SUCCESS } else { MUTED }),
                ),
                Span::styled(
                    choice.label.as_str(),
                    if focused {
                        Style::default().fg(ACCENT)
                    } else {
                        Style::default()
                    },
                ),
            ]))
        })
        .collect::<Vec<_>>();
    if items.is_empty() {
        frame.render_widget(
            Paragraph::new("      No matching sessions").style(Style::default().fg(MUTED)),
            options_area,
        );
    } else {
        frame.render_widget(List::new(items), options_area);
    }

    let position = if state.visible.is_empty() {
        "0/0".to_string()
    } else {
        format!("{}/{}", state.cursor + 1, state.visible.len())
    };
    frame.render_widget(
        Paragraph::new(format!("{HELP} · {position}")).style(Style::default().fg(MUTED)),
        help_area,
    );
}
