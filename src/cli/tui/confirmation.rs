use super::app::App;
use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
};

pub(super) fn render(f: &mut ratatui::Frame, area: Rect, app: &App) {
    let Some(targets) = &app.pending_kill else {
        return;
    };
    let popup = centered_rect(84, 70, area);
    let mut lines = vec![Line::styled(
        "Press K to confirm. Any other key cancels.",
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD),
    )];
    lines.push(Line::raw(""));
    for target in targets {
        lines.push(Line::from(vec![
            Span::raw(target.label.clone()),
            Span::styled(
                format!("  [{}]", target.session_id),
                Style::default().fg(Color::DarkGray),
            ),
        ]));
    }
    f.render_widget(Clear, popup);
    f.render_widget(
        Paragraph::new(lines)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Red))
                    .title(format!(" kill {} session(s) ", targets.len())),
            )
            .wrap(Wrap { trim: false }),
        popup,
    );
}

fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let vertical = Layout::vertical([
        Constraint::Percentage((100 - percent_y) / 2),
        Constraint::Percentage(percent_y),
        Constraint::Percentage((100 - percent_y) / 2),
    ])
    .split(area);
    Layout::horizontal([
        Constraint::Percentage((100 - percent_x) / 2),
        Constraint::Percentage(percent_x),
        Constraint::Percentage((100 - percent_x) / 2),
    ])
    .split(vertical[1])[1]
}
