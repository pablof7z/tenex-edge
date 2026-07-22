use super::{caret, MUTED};
use crate::cli::agents::{AgentKind, AgentRow};
use ratatui::{
    style::{Modifier, Style},
    text::{Line, Span},
};

pub(super) fn kind_label(kind: AgentKind) -> &'static str {
    match kind {
        AgentKind::Configured => "configured",
        AgentKind::NativeProfile => "native profile",
        AgentKind::Generic => "generic",
    }
}

pub(super) fn lines(
    row: &AgentRow,
    width: usize,
    focused: bool,
    selected: bool,
) -> [Line<'static>; 2] {
    let label = crate::cli::agents::harness_name(row.harness);
    let name_width = row.slug.chars().count();
    let padding = width
        .saturating_sub(4 + name_width + label.chars().count())
        .max(2);
    let name_style = if focused {
        Style::default()
            .fg(super::ACCENT)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().add_modifier(Modifier::BOLD)
    };
    [
        Line::from(vec![
            caret(focused),
            Span::styled(
                if selected { "✓ " } else { "＋ " },
                Style::default().fg(crate::console_style::harness_ratatui_color(row.harness)),
            ),
            Span::styled(row.slug.clone(), name_style),
            Span::raw(" ".repeat(padding)),
            Span::styled(label, Style::default().fg(MUTED)),
        ]),
        Line::from(Span::styled(
            format!("    {}", row.summary(width.saturating_sub(4))),
            Style::default().fg(MUTED),
        )),
    ]
}
