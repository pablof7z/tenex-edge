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
        .min(24);
    let fixed = 3 + 10 + name_width + 2 + 12 + 2;
    let description_width = width.saturating_sub(fixed);
    let items = state
        .window(usize::from(area.height))
        .map(|(visible_index, row)| {
            let focused = visible_index == state.cursor;
            let marked = state.selected.contains(&state.visible[visible_index]);
            let source = format!("[{}]", row.source_short_label());
            ListItem::new(Line::from(vec![
                Span::styled(
                    marker(focused, marked),
                    Style::default().fg(if focused { ACCENT } else { MUTED }),
                ),
                Span::styled(format!("{source:<10}"), Style::default().fg(MUTED)),
                Span::styled(
                    format!("{:<name_width$}", truncate(&row.name, name_width)),
                    Style::default().add_modifier(Modifier::BOLD),
                ),
                Span::raw("  "),
                Span::styled(
                    format!("{:<12}", truncate(row.harness_label(), 12)),
                    harness_style(row),
                ),
                Span::raw("  "),
                Span::styled(
                    truncate(&row.description_summary(), description_width),
                    Style::default().fg(MUTED),
                ),
            ]))
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
