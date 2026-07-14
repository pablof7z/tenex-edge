use super::PickerState;
use ratatui::{
    layout::{Constraint, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Clear, List, ListItem, Paragraph},
    Frame,
};

const ACCENT: Color = Color::Indexed(45);
const MUTED: Color = Color::Indexed(245);
const ERROR: Color = Color::Indexed(203);
const HELP: &str = "enter attach · ⇧K kill · type filter · ↑↓ move · esc";

pub(super) fn draw(frame: &mut Frame<'_>, state: &PickerState) {
    let area = frame.area();
    frame.render_widget(Clear, area);
    if area.height < 3 {
        frame.render_widget(Paragraph::new("Sessions"), area);
        return;
    }

    let [title_area, options_area, help_area] = Layout::vertical([
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
            Span::styled("Sessions", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw("  Filter: "),
            query,
        ])),
        title_area,
    );

    let rows = usize::from(options_area.height / 2);
    let content_width = usize::from(options_area.width.saturating_sub(4));
    let now = crate::util::now_secs();
    let items = state
        .window(rows)
        .map(|(visible_index, choice)| {
            let focused = visible_index == state.cursor;
            let [mut first, mut second] =
                super::super::layout::lines(&choice.row, now, content_width, focused);
            first.spans.insert(
                0,
                Span::styled(
                    if focused { "❯ " } else { "  " },
                    Style::default().fg(if focused { ACCENT } else { MUTED }),
                ),
            );
            second.spans.insert(0, Span::raw("    "));
            ListItem::new(vec![first, second])
        })
        .collect::<Vec<_>>();
    if items.is_empty() {
        frame.render_widget(
            Paragraph::new("  No matching sessions").style(Style::default().fg(MUTED)),
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
    let footer = state
        .notice
        .as_ref()
        .map(|notice| (notice.as_str(), ERROR))
        .unwrap_or((HELP, MUTED));
    frame.render_widget(
        Paragraph::new(format!("{} · {position}", footer.0)).style(Style::default().fg(footer.1)),
        help_area,
    );
}
