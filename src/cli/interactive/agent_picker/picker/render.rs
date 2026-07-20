mod briefs;
mod index;
mod inspector;

use super::{DeleteScope, PendingDelete, PickerState, PickerView};
use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Clear, Paragraph},
    Frame,
};
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

pub(super) const ACCENT: Color = Color::Indexed(45);
pub(super) const MUTED: Color = Color::Indexed(245);
pub(super) const ERROR: Color = Color::Indexed(203);
pub(super) const FOCUS_BG: Color = Color::Indexed(236);
const WIDE_INSPECTOR: u16 = 88;

pub(super) fn draw(frame: &mut Frame<'_>, state: &PickerState) {
    let area = frame.area();
    frame.render_widget(Clear, area);
    if area.height < 3 {
        frame.render_widget(Paragraph::new("Agents"), area);
        return;
    }
    let [title, body, footer] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Min(1),
        Constraint::Length(1),
    ])
    .areas(area);
    draw_title(frame, state, title);
    match state.view {
        PickerView::Inspector if area.width >= WIDE_INSPECTOR => {
            inspector::draw(frame, state, body)
        }
        PickerView::Inspector | PickerView::Briefs => briefs::draw(frame, state, body),
        PickerView::Index => index::draw(frame, state, body),
    }
    draw_footer(frame, state, footer);
}

pub(super) fn option_capacity(view: PickerView, area: Rect) -> usize {
    let body = usize::from(area.height.saturating_sub(2));
    match view {
        PickerView::Briefs => body / 2,
        PickerView::Inspector if area.width < WIDE_INSPECTOR => body / 2,
        PickerView::Inspector | PickerView::Index => body,
    }
    .max(1)
}

fn draw_title(frame: &mut Frame<'_>, state: &PickerState, area: Rect) {
    let count = if state.query.is_empty() {
        format!("{} available", state.visible.len())
    } else {
        format!("{} matches", state.visible.len())
    };
    let mut spans = vec![
        Span::styled(
            "Launch an agent",
            Style::default().add_modifier(Modifier::BOLD),
        ),
        Span::styled(format!("  {count}"), Style::default().fg(MUTED)),
        Span::styled("   View  ", Style::default().fg(MUTED)),
    ];
    for view in [PickerView::Inspector, PickerView::Briefs, PickerView::Index] {
        let style = if state.view == view {
            Style::default().fg(ACCENT).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(MUTED)
        };
        spans.push(Span::styled(
            format!("{} {}", view.number(), view.label()),
            style,
        ));
        if view != PickerView::Index {
            spans.push(Span::styled("  ", Style::default().fg(MUTED)));
        }
    }
    if state.filtering {
        spans.push(Span::styled("   Search  ", Style::default().fg(MUTED)));
        spans.push(Span::styled(
            if state.query.is_empty() {
                "type to filter"
            } else {
                state.query.as_str()
            },
            Style::default().fg(ACCENT),
        ));
    } else if area.width >= 82 {
        spans.push(Span::styled("   / search", Style::default().fg(MUTED)));
    }
    frame.render_widget(Paragraph::new(Line::from(spans)), area);
}

fn draw_footer(frame: &mut Frame<'_>, state: &PickerState, area: Rect) {
    let line = if let Some(notice) = delete_notice(state) {
        Line::from(Span::styled(notice, Style::default().fg(ERROR)))
    } else {
        let position = if state.visible.is_empty() {
            "0/0".to_string()
        } else {
            format!("{}/{}", state.cursor + 1, state.visible.len())
        };
        let help = if state.filtering {
            "enter launch  ·  type search  ·  esc clear"
        } else if !state.selected.is_empty() {
            "space unmark  ·  d delete marked  ·  esc close"
        } else if area.width >= 90 {
            "enter launch  ·  e edit  ·  space mark  ·  d delete  ·  / search  ·  1–3 view  ·  esc"
        } else {
            "enter launch  ·  e edit  ·  / search  ·  1–3 view  ·  esc"
        };
        Line::from(vec![
            Span::styled(help, Style::default().fg(MUTED)),
            Span::styled(format!("  ·  {position}"), Style::default().fg(MUTED)),
        ])
    };
    frame.render_widget(Paragraph::new(line), area);
}

fn delete_notice(state: &PickerState) -> Option<String> {
    match state.pending_delete.as_ref()? {
        PendingDelete::Nothing { index } => Some(format!(
            "{} is built in — nothing to delete · any key cancels",
            state.rows[*index].name
        )),
        PendingDelete::ChooseScope { index } => Some(format!(
            "Delete {}: a agent config · p native profile · b both · esc cancel",
            state.rows[*index].name
        )),
        PendingDelete::Confirm { plan } => {
            let what = if let [(index, scope)] = plan.as_slice() {
                let target = match scope {
                    DeleteScope::Agent => "agent configuration",
                    DeleteScope::Profile => "native profile",
                    DeleteScope::Both => "agent configuration and native profile",
                };
                format!("{target} for {}", state.rows[*index].name)
            } else {
                format!("{} marked agents", plan.len())
            };
            Some(format!("Delete {what}? y/d confirm · esc cancel"))
        }
    }
}

pub(super) fn harness_style(row: &super::AgentPickerRow) -> Style {
    let color = row
        .status
        .as_ref()
        .map(|status| crate::console_style::harness_ratatui_color(status.harness))
        .unwrap_or(MUTED);
    Style::default().fg(color).add_modifier(Modifier::BOLD)
}

pub(super) fn focus_style(focused: bool) -> Style {
    if focused {
        Style::default().bg(FOCUS_BG)
    } else {
        Style::default()
    }
}

pub(super) fn marker(focused: bool, marked: bool) -> String {
    match (focused, marked) {
        (true, true) => "❯✓ ".to_string(),
        (true, false) => "❯  ".to_string(),
        (false, true) => " ✓ ".to_string(),
        (false, false) => "   ".to_string(),
    }
}

pub(super) fn truncate(value: &str, width: usize) -> String {
    if UnicodeWidthStr::width(value) <= width {
        return value.to_string();
    }
    if width == 0 {
        return String::new();
    }
    let target = width.saturating_sub(1);
    let mut used = 0;
    let mut output = String::new();
    for character in value.chars() {
        let character_width = character.width().unwrap_or(0);
        if used + character_width > target {
            break;
        }
        output.push(character);
        used += character_width;
    }
    output.push('…');
    output
}
