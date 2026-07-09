use super::app::App;
use super::data::SessionRow;
use crate::util::{now_secs, relative_time};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap},
};

pub(super) fn render(f: &mut ratatui::Frame, app: &mut App) {
    let area = f.area();
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2),
            Constraint::Min(1),
            Constraint::Length(1),
        ])
        .split(area);
    render_header(f, rows[0], app);
    let main = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(36), Constraint::Percentage(64)])
        .split(rows[1]);
    render_sessions(f, main[0], app);
    render_panes(f, main[1], app);
    render_status(f, rows[2], app);
}

fn render_header(f: &mut ratatui::Frame, area: Rect, app: &App) {
    let mode = if app.input_mode { "input" } else { "control" };
    let panes = app.panes.len();
    f.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(
                "tenex-edge tui",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("  mode="),
            Span::styled(mode, Style::default().fg(Color::Yellow)),
            Span::raw("  panes="),
            Span::styled(panes.to_string(), Style::default().fg(Color::Yellow)),
        ])),
        area,
    );
}

fn render_sessions(f: &mut ratatui::Frame, area: Rect, app: &App) {
    let items = app.sessions.iter().map(session_item).collect::<Vec<_>>();
    let mut state = ListState::default();
    if !app.sessions.is_empty() {
        state.select(Some(app.selected));
    }
    let title = format!(" sessions ({}) ", app.sessions.len());
    f.render_stateful_widget(
        List::new(items)
            .block(Block::default().borders(Borders::ALL).title(title))
            .highlight_symbol(">")
            .highlight_style(Style::default().fg(Color::Yellow)),
        area,
        &mut state,
    );
}

fn session_item(row: &SessionRow) -> ListItem<'static> {
    let seen = if row.last_seen == 0 {
        "unknown".to_string()
    } else {
        relative_time(row.last_seen, now_secs())
    };
    let pty = match (row.pty_id.as_ref(), row.pty_live) {
        (Some(_), true) => Span::styled("PTY", Style::default().fg(Color::Green)),
        (Some(_), false) => Span::styled("PTY-", Style::default().fg(Color::DarkGray)),
        (None, _) => Span::styled("view", Style::default().fg(Color::DarkGray)),
    };
    let state = if row.busy {
        Span::styled("busy", Style::default().fg(Color::Red))
    } else {
        Span::styled("idle", Style::default().fg(Color::DarkGray))
    };
    let channels = if row.channels.is_empty() {
        "-".to_string()
    } else {
        row.channels.join(",")
    };
    ListItem::new(vec![
        Line::from(vec![
            Span::styled(
                row.agent.clone(),
                Style::default().add_modifier(Modifier::BOLD),
            ),
            Span::raw(" "),
            pty,
            Span::raw(" "),
            state,
            Span::raw(" "),
            Span::styled(seen, Style::default().fg(Color::DarkGray)),
        ]),
        Line::from(vec![Span::styled(
            row.title_with_activity(),
            Style::default().fg(Color::White),
        )]),
        Line::from(Span::styled(channels, Style::default().fg(Color::DarkGray))),
    ])
}

fn render_panes(f: &mut ratatui::Frame, area: Rect, app: &mut App) {
    if app.panes.is_empty() {
        render_details(f, area, app);
        return;
    }
    let areas = pane_areas(area, app.panes.len());
    for (idx, pane) in app.panes.iter_mut().enumerate() {
        let Some(area) = areas.get(idx).copied() else {
            break;
        };
        let active = app.active_pane == Some(idx);
        let border = if app.input_mode && active {
            Color::Green
        } else if active {
            Color::Yellow
        } else {
            Color::DarkGray
        };
        let inner_rows = area.height.saturating_sub(2).max(1);
        let inner_cols = area.width.saturating_sub(2).max(1);
        pane.resize(inner_rows, inner_cols);
        let mut title = format!(" {} ", pane.title());
        if !pane.connected() {
            title.push_str(" disconnected ");
        }
        f.render_widget(
            Paragraph::new(pane.lines(inner_cols, inner_rows))
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(border))
                        .title(title),
                )
                .wrap(Wrap { trim: false }),
            area,
        );
    }
}

fn render_details(f: &mut ratatui::Frame, area: Rect, app: &App) {
    let lines = match app.sessions.get(app.selected) {
        Some(row) => vec![
            Line::from(vec![
                Span::styled("agent ", Style::default().fg(Color::DarkGray)),
                Span::raw(row.agent.clone()),
            ]),
            Line::from(vec![
                Span::styled("title ", Style::default().fg(Color::DarkGray)),
                Span::raw(row.display_title().to_string()),
            ]),
            Line::from(vec![
                Span::styled("activity ", Style::default().fg(Color::DarkGray)),
                Span::raw(row.activity.clone()),
            ]),
            Line::from(vec![
                Span::styled("channels ", Style::default().fg(Color::DarkGray)),
                Span::raw(row.channels.join(",")),
            ]),
            Line::from(vec![
                Span::styled("pty ", Style::default().fg(Color::DarkGray)),
                Span::raw(
                    row.pty_id
                        .as_deref()
                        .map(|id| format!("{id} live={}", row.pty_live))
                        .unwrap_or_else(|| "none".to_string()),
                ),
            ]),
            Line::from(vec![
                Span::styled("cwd ", Style::default().fg(Color::DarkGray)),
                Span::raw(row.cwd.clone().unwrap_or_default()),
            ]),
            Line::from(vec![
                Span::styled("command ", Style::default().fg(Color::DarkGray)),
                Span::raw(row.command.join(" ")),
            ]),
        ],
        None => vec![Line::raw("No sessions found.")],
    };
    f.render_widget(
        Paragraph::new(lines)
            .block(Block::default().borders(Borders::ALL).title(" details "))
            .wrap(Wrap { trim: false }),
        area,
    );
}

fn render_status(f: &mut ratatui::Frame, area: Rect, app: &App) {
    let hints = if app.input_mode {
        "Esc/Ctrl-G controls"
    } else {
        "up/down/jk select  enter/a attach  o open pane  tab switch  1-9 focus  x close  K,K kill  r refresh  q quit"
    };
    let status = if app.status.is_empty() {
        hints.to_string()
    } else {
        format!("{}  {}", app.status, hints)
    };
    f.render_widget(
        Paragraph::new(Line::from(Span::styled(
            status,
            Style::default().fg(Color::DarkGray),
        ))),
        area,
    );
}

fn pane_areas(area: Rect, count: usize) -> Vec<Rect> {
    let cols = match count {
        0 | 1 => 1,
        2..=4 => 2,
        _ => 3,
    };
    let rows = count.div_ceil(cols);
    let row_constraints = even_constraints(rows);
    let row_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(row_constraints)
        .split(area);
    let mut out = Vec::new();
    for row in row_chunks.iter() {
        let col_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(even_constraints(cols))
            .split(*row);
        out.extend(col_chunks.iter().copied());
    }
    out.truncate(count);
    out
}

fn even_constraints(n: usize) -> Vec<Constraint> {
    (0..n)
        .map(|_| Constraint::Ratio(1, n.max(1) as u32))
        .collect()
}
