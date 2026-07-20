use super::{focus_style, harness_style, marker, truncate, ACCENT, MUTED};
use crate::cli::interactive::agent_picker::picker::PickerState;
use ratatui::{
    layout::{Constraint, Layout, Margin, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Padding, Paragraph, Wrap},
    Frame,
};

pub(super) fn draw(frame: &mut Frame<'_>, state: &PickerState, area: Rect) {
    let [list_area, detail_area] =
        Layout::horizontal([Constraint::Length(50), Constraint::Min(38)]).areas(area);
    let list_width = usize::from(list_area.width.saturating_sub(2));
    let items = state
        .window(usize::from(list_area.height))
        .map(|(visible_index, row)| {
            let focused = visible_index == state.cursor;
            let marked = state.selected.contains(&state.visible[visible_index]);
            let harness = row.harness_label();
            let harness_width = harness.len().min(12);
            let name_width = list_width.saturating_sub(4 + harness_width + 1);
            ListItem::new(Line::from(vec![
                Span::styled(
                    marker(focused, marked),
                    Style::default().fg(if focused { ACCENT } else { MUTED }),
                ),
                Span::styled(
                    format!("{:<name_width$}", truncate(&row.name, name_width)),
                    Style::default().add_modifier(Modifier::BOLD),
                ),
                Span::raw(" "),
                Span::styled(truncate(harness, harness_width), harness_style(row)),
            ]))
            .style(focus_style(focused))
        })
        .collect::<Vec<_>>();
    if items.is_empty() {
        frame.render_widget(
            Paragraph::new("  No agents match this search").style(Style::default().fg(MUTED)),
            list_area,
        );
    } else {
        frame.render_widget(List::new(items), list_area);
    }

    let block = Block::new()
        .borders(Borders::LEFT)
        .border_style(Style::default().fg(MUTED))
        .padding(Padding::horizontal(2));
    let inner = block.inner(detail_area);
    frame.render_widget(block, detail_area);
    let Some(row) = state.current_row() else {
        return;
    };
    let [name, description, metadata, actions, _rest] = Layout::vertical([
        Constraint::Length(2),
        Constraint::Length(4),
        Constraint::Length(4),
        Constraint::Length(1),
        Constraint::Min(0),
    ])
    .areas(inner.inner(Margin::new(0, 1)));
    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(
                row.name.clone(),
                Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!("  {}", row.source_label()),
                Style::default().fg(MUTED),
            ),
        ])),
        name,
    );
    frame.render_widget(
        Paragraph::new(row.clean_description())
            .style(Style::default().fg(ratatui::style::Color::White))
            .wrap(Wrap { trim: true }),
        description,
    );
    let profile = match (row.has_configured, row.has_native_profile) {
        (true, true) => "configured + native profile",
        (true, false) => "mosaico configuration",
        (false, true) => "native profile",
        (false, false) => "built-in defaults",
    };
    frame.render_widget(
        Paragraph::new(vec![
            Line::from(vec![
                Span::styled("Harness   ", Style::default().fg(MUTED)),
                Span::styled(row.harness_label().to_string(), harness_style(row)),
            ]),
            Line::from(vec![
                Span::styled("Source    ", Style::default().fg(MUTED)),
                Span::raw(row.source_label()),
            ]),
            Line::from(vec![
                Span::styled("Launches  ", Style::default().fg(MUTED)),
                Span::raw(profile),
            ]),
        ]),
        metadata,
    );
    frame.render_widget(
        Paragraph::new("enter launch   e edit   space mark").style(Style::default().fg(MUTED)),
        actions,
    );
}
