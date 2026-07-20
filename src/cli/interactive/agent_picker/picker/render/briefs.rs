use super::{focus_style, harness_style, marker, truncate, ACCENT, MUTED};
use crate::cli::interactive::agent_picker::picker::PickerState;
use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{List, ListItem, Paragraph},
    Frame,
};

pub(super) fn draw(frame: &mut Frame<'_>, state: &PickerState, area: Rect) {
    let width = usize::from(area.width);
    let name_width = state
        .visible
        .iter()
        .map(|&index| state.rows[index].name.len())
        .max()
        .unwrap_or(0)
        .clamp(8, 32);
    let items = state
        .window(usize::from(area.height / 2).max(1))
        .map(|(visible_index, row)| {
            let focused = visible_index == state.cursor;
            let marked = state.selected.contains(&state.visible[visible_index]);
            let harness = row.harness_label();
            let source = row.source_label();
            let description_width = width.saturating_sub(6).min(112);
            ListItem::new(vec![
                Line::from(vec![
                    Span::styled(
                        marker(focused, marked),
                        Style::default().fg(if focused { ACCENT } else { MUTED }),
                    ),
                    Span::styled(
                        format!("{:<name_width$}", truncate(&row.name, name_width)),
                        Style::default().add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(format!("  {source:<19}"), Style::default().fg(MUTED)),
                    Span::styled(harness.to_string(), harness_style(row)),
                ]),
                Line::from(Span::styled(
                    format!(
                        "      {}",
                        truncate(&row.description_summary(), description_width)
                    ),
                    Style::default().fg(MUTED),
                )),
            ])
            .style(focus_style(focused))
        })
        .collect::<Vec<_>>();
    if items.is_empty() {
        frame.render_widget(
            Paragraph::new("  No agents match this search").style(Style::default().fg(MUTED)),
            area,
        );
    } else {
        frame.render_widget(List::new(items), area);
    }
}
