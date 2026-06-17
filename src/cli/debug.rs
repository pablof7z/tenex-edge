use super::*;
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Terminal,
};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet, HashMap};

pub(super) struct HookTailOpts {
    pub(super) project: Option<String>,
    pub(super) session: Option<String>,
    pub(super) panes: usize,
    pub(super) refresh: Duration,
}

#[derive(Clone, Debug)]
struct DebugLine {
    ts_ms: u128,
    kind: DebugKind,
    title: String,
    detail: String,
    ok: Option<bool>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum DebugKind {
    Hook,
    Inject,
    Command,
    Error,
    Session,
}

#[derive(Clone, Debug, Default)]
struct SessionPane {
    session: String,
    short: String,
    project: String,
    agent: String,
    host: String,
    lines: Vec<DebugLine>,
}

#[derive(Clone, Debug, Default)]
pub(crate) struct HookTailSnapshot {
    panes: Vec<SessionPane>,
    unscoped: Vec<DebugLine>,
    projects: Vec<String>,
    sessions: Vec<String>,
}

struct HookTailState {
    project_filter: Option<String>,
    session_filter: Option<String>,
    pane_limit: usize,
    focused: usize,
    focus_mode: bool,
    status: String,
}

struct TuiTerminal;

impl TuiTerminal {
    fn enter() -> Result<Self> {
        terminal::enable_raw_mode()?;
        execute!(io::stdout(), EnterAlternateScreen, Hide)?;
        Ok(Self)
    }
}

impl Drop for TuiTerminal {
    fn drop(&mut self) {
        let _ = terminal::disable_raw_mode();
        let _ = execute!(io::stdout(), Show, LeaveAlternateScreen);
    }
}

pub(super) fn hook_tail(opts: HookTailOpts) -> Result<()> {
    let mut state = HookTailState {
        project_filter: opts.project,
        session_filter: opts.session,
        pane_limit: opts.panes.clamp(1, 24),
        focused: 0,
        focus_mode: false,
        status: String::new(),
    };

    let refresh = opts.refresh.max(Duration::from_millis(100));
    let mut snapshot = load_hook_tail_snapshot(&state.project_filter, &state.session_filter);

    let _terminal = TuiTerminal::enter()?;
    let mut terminal = Terminal::new(CrosstermBackend::new(io::stdout()))?;
    let mut next_refresh = Instant::now();

    loop {
        if state.focused >= snapshot.panes.len().max(1) {
            state.focused = snapshot.panes.len().saturating_sub(1);
        }

        terminal.draw(|f| render_hook_tail(f, &snapshot, &state))?;

        let wait = next_refresh
            .saturating_duration_since(Instant::now())
            .min(Duration::from_millis(100));

        if event::poll(wait)? {
            if let TermEvent::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => break,
                    KeyCode::Char('+') | KeyCode::Char('=') => {
                        state.pane_limit = (state.pane_limit + 1).min(24);
                    }
                    KeyCode::Char('-') => {
                        state.pane_limit = state.pane_limit.saturating_sub(1).max(1);
                        state.focused = state.focused.min(state.pane_limit.saturating_sub(1));
                    }
                    KeyCode::Tab | KeyCode::Right => {
                        let n = snapshot.panes.len().min(state.pane_limit).max(1);
                        state.focused = (state.focused + 1) % n;
                    }
                    KeyCode::BackTab | KeyCode::Left => {
                        let n = snapshot.panes.len().min(state.pane_limit).max(1);
                        state.focused = if state.focused == 0 {
                            n - 1
                        } else {
                            state.focused - 1
                        };
                    }
                    KeyCode::Enter | KeyCode::Char('f') => {
                        state.focus_mode = !state.focus_mode;
                    }
                    KeyCode::Char('a') => {
                        state.project_filter = None;
                        state.session_filter = None;
                        state.status = "filters cleared".to_string();
                    }
                    KeyCode::Char('p') => {
                        state.project_filter =
                            cycle_filter(state.project_filter.as_deref(), &snapshot.projects);
                        state.status = match &state.project_filter {
                            Some(p) => format!("project filter: {p}"),
                            None => "project filter cleared".to_string(),
                        };
                    }
                    KeyCode::Char('s') => {
                        state.session_filter =
                            cycle_filter(state.session_filter.as_deref(), &snapshot.sessions);
                        state.status = match &state.session_filter {
                            Some(s) => format!("session filter: {s}"),
                            None => "session filter cleared".to_string(),
                        };
                    }
                    _ => {}
                }
            }
        }

        if Instant::now() >= next_refresh {
            snapshot = load_hook_tail_snapshot(&state.project_filter, &state.session_filter);
            next_refresh = Instant::now() + refresh;
        }
    }

    Ok(())
}

fn cycle_filter(current: Option<&str>, values: &[String]) -> Option<String> {
    if values.is_empty() {
        return None;
    }
    let Some(cur) = current else {
        return values.first().cloned();
    };
    let next = values
        .iter()
        .position(|v| v == cur)
        .map(|i| i + 1)
        .unwrap_or(0);
    values.get(next).cloned()
}

pub(crate) fn load_hook_tail_snapshot(
    project_filter: &Option<String>,
    session_filter: &Option<String>,
) -> HookTailSnapshot {
    let mut panes: BTreeMap<String, SessionPane> = BTreeMap::new();
    seed_live_sessions(&mut panes);

    let home = crate::config::edge_home();
    read_hook_log(&home.join("hook-calls.jsonl"), &mut panes);
    let unscoped = read_command_log(&home.join("command-calls.jsonl"), &mut panes);

    let mut projects = BTreeSet::new();
    let mut sessions = BTreeSet::new();
    for pane in panes.values() {
        if !pane.project.is_empty() {
            projects.insert(pane.project.clone());
        }
        if !pane.session.is_empty() {
            sessions.insert(pane.short.clone());
        }
    }

    let mut panes: Vec<SessionPane> = panes
        .into_values()
        .filter(|p| match project_filter {
            Some(filter) => p.project == *filter,
            None => true,
        })
        .filter(|p| match session_filter {
            Some(filter) => p.session == *filter || p.short == *filter,
            None => true,
        })
        .collect();
    for pane in &mut panes {
        pane.lines.sort_by_key(|l| l.ts_ms);
        if pane.lines.is_empty() {
            pane.lines.push(DebugLine {
                ts_ms: 0,
                kind: DebugKind::Session,
                title: "live session".to_string(),
                detail: "no hook or command telemetry yet".to_string(),
                ok: None,
            });
        }
    }
    panes.sort_by(|a, b| latest_ts(b).cmp(&latest_ts(a)).then(a.short.cmp(&b.short)));

    HookTailSnapshot {
        panes,
        unscoped,
        projects: projects.into_iter().collect(),
        sessions: sessions.into_iter().collect(),
    }
}

fn seed_live_sessions(panes: &mut BTreeMap<String, SessionPane>) {
    let Ok(v) = crate::daemon::blocking::call(
        "who",
        serde_json::json!({
            "project": null,
            "all": false,
            "all_projects": true,
            "cwd": std::env::current_dir().ok().map(|p| p.to_string_lossy().to_string()),
        }),
    ) else {
        return;
    };
    for row in v["rows"].as_array().cloned().unwrap_or_default() {
        let session = row["session_id"].as_str().unwrap_or("").to_string();
        if session.is_empty() {
            continue;
        }
        let pane = panes
            .entry(session.clone())
            .or_insert_with(|| new_pane(&session));
        pane.project = row["project"].as_str().unwrap_or("").to_string();
        pane.agent = row["slug"].as_str().unwrap_or("").to_string();
        pane.host = row["host"].as_str().unwrap_or("").to_string();
    }
}

fn read_hook_log(path: &std::path::Path, panes: &mut BTreeMap<String, SessionPane>) {
    let Ok(raw) = std::fs::read_to_string(path) else {
        return;
    };
    let mut hook_sessions: HashMap<String, String> = HashMap::new();
    for line in raw.lines() {
        let Ok(v) = serde_json::from_str::<Value>(line) else {
            continue;
        };
        if v["schema"].as_str() != Some("tenex-edge.hook-call.v1") {
            continue;
        }
        let call_id = v["call_id"].as_str().unwrap_or("").to_string();
        let ts_ms = ts_ms(&v);
        match v["phase"].as_str().unwrap_or("") {
            "received" => {
                let host = v["hook"]["host"].as_str().unwrap_or("");
                let hook_type = v["hook"]["type"].as_str().unwrap_or("");
                let stdin_json = &v["stdin"]["json"];
                let session = hook_session(stdin_json).unwrap_or_else(|| "unscoped".to_string());
                hook_sessions.insert(call_id, session.clone());
                let pane = panes
                    .entry(session.clone())
                    .or_insert_with(|| new_pane(&session));
                fill_pane_from_hook(pane, host, stdin_json);
                pane.lines.push(DebugLine {
                    ts_ms,
                    kind: DebugKind::Hook,
                    title: format!("hook {host}/{hook_type}"),
                    detail: summarize_hook_payload(stdin_json),
                    ok: None,
                });
            }
            "note" => {
                let note = v["note"].as_str().unwrap_or("note");
                let detail = &v["detail"];
                let session = detail["session"]
                    .as_str()
                    .map(str::to_string)
                    .or_else(|| hook_sessions.get(&call_id).cloned())
                    .unwrap_or_else(|| "unscoped".to_string());
                let pane = panes
                    .entry(session.clone())
                    .or_insert_with(|| new_pane(&session));
                let kind = if note == "context-injection" {
                    DebugKind::Inject
                } else {
                    DebugKind::Hook
                };
                pane.lines.push(DebugLine {
                    ts_ms,
                    kind,
                    title: note.to_string(),
                    detail: if note == "context-injection" {
                        detail["text"].as_str().unwrap_or("").to_string()
                    } else {
                        detail.to_string()
                    },
                    ok: None,
                });
            }
            "finished" => {
                let session = hook_sessions
                    .get(&call_id)
                    .cloned()
                    .unwrap_or_else(|| "unscoped".to_string());
                let pane = panes
                    .entry(session.clone())
                    .or_insert_with(|| new_pane(&session));
                let ok = v["result"]["ok"].as_bool();
                pane.lines.push(DebugLine {
                    ts_ms,
                    kind: if ok == Some(false) {
                        DebugKind::Error
                    } else {
                        DebugKind::Hook
                    },
                    title: "hook finished".to_string(),
                    detail: v["result"]["error"]
                        .as_str()
                        .unwrap_or_else(|| if ok == Some(true) { "ok" } else { "unknown" })
                        .to_string(),
                    ok,
                });
            }
            _ => {}
        }
    }
}

fn read_command_log(
    path: &std::path::Path,
    panes: &mut BTreeMap<String, SessionPane>,
) -> Vec<DebugLine> {
    let Ok(raw) = std::fs::read_to_string(path) else {
        return Vec::new();
    };
    let mut received: HashMap<String, Value> = HashMap::new();
    let mut unscoped = Vec::new();
    for line in raw.lines() {
        let Ok(v) = serde_json::from_str::<Value>(line) else {
            continue;
        };
        if v["schema"].as_str() != Some("tenex-edge.command-call.v1") {
            continue;
        }
        let call_id = v["call_id"].as_str().unwrap_or("").to_string();
        match v["phase"].as_str().unwrap_or("") {
            "received" => {
                received.insert(call_id, v);
            }
            "finished" => {
                let Some(start) = received.get(&call_id) else {
                    continue;
                };
                let project = command_project(start);
                let agent = start["env"]["TENEX_EDGE_AGENT"]
                    .as_str()
                    .or_else(|| start["env"]["TENEX_EDGE_AGENT_FALLBACK"].as_str())
                    .unwrap_or("")
                    .to_string();
                let session = command_session(start)
                    .or_else(|| infer_command_session(panes, &agent, &project));
                let argv = start["command"]["argv"]
                    .as_array()
                    .map(|a| {
                        a.iter()
                            .filter_map(|v| v.as_str())
                            .collect::<Vec<_>>()
                            .join(" ")
                    })
                    .unwrap_or_default();
                let ok = v["result"]["ok"].as_bool();
                let detail = v["result"]["error"]
                    .as_str()
                    .map(|e| format!("{argv}\n{e}"))
                    .unwrap_or(argv);
                let entry = DebugLine {
                    ts_ms: ts_ms(&v),
                    kind: if ok == Some(false) {
                        DebugKind::Error
                    } else {
                        DebugKind::Command
                    },
                    title: "tenex-edge command".to_string(),
                    detail,
                    ok,
                };
                if let Some(session) = session {
                    let pane = panes
                        .entry(session.clone())
                        .or_insert_with(|| new_pane(&session));
                    if !project.is_empty() {
                        pane.project = project;
                    }
                    if !agent.is_empty() {
                        pane.agent = agent;
                    }
                    pane.lines.push(entry);
                } else {
                    unscoped.push(entry);
                }
            }
            _ => {}
        }
    }
    unscoped
}

fn new_pane(session: &str) -> SessionPane {
    SessionPane {
        session: session.to_string(),
        short: if session == "unscoped" {
            "unscoped".to_string()
        } else {
            SessionId::from(session).to_string()
        },
        ..SessionPane::default()
    }
}

fn fill_pane_from_hook(pane: &mut SessionPane, host: &str, stdin_json: &Value) {
    if pane.host.is_empty() {
        pane.host = host.to_string();
    }
    if pane.project.is_empty() {
        pane.project = stdin_json["cwd"]
            .as_str()
            .map(|cwd| crate::project::resolve(std::path::Path::new(cwd)))
            .unwrap_or_default();
    }
}

fn hook_session(v: &Value) -> Option<String> {
    [
        "session_id",
        "sessionId",
        "conversation_id",
        "conversationId",
        "thread_id",
        "threadId",
    ]
    .iter()
    .find_map(|key| v[*key].as_str())
    .filter(|s| !s.is_empty())
    .map(str::to_string)
}

fn command_session(v: &Value) -> Option<String> {
    v["env"]["TENEX_EDGE_SESSION"]
        .as_str()
        .or_else(|| v["command"]["explicit_session"].as_str())
        .filter(|s| !s.is_empty())
        .map(str::to_string)
}

fn command_project(v: &Value) -> String {
    v["process"]["cwd"]
        .as_str()
        .map(|cwd| crate::project::resolve(std::path::Path::new(cwd)))
        .unwrap_or_default()
}

fn infer_command_session(
    panes: &BTreeMap<String, SessionPane>,
    agent: &str,
    project: &str,
) -> Option<String> {
    if agent.is_empty() || project.is_empty() {
        return None;
    }
    let matches = panes
        .values()
        .filter(|p| p.agent == agent && p.project == project && !p.session.is_empty())
        .map(|p| p.session.clone())
        .collect::<Vec<_>>();
    if matches.len() == 1 {
        matches.into_iter().next()
    } else {
        None
    }
}

fn summarize_hook_payload(v: &Value) -> String {
    let cwd = v["cwd"].as_str().unwrap_or("");
    let prompt = v["prompt"].as_str().unwrap_or("");
    let transcript = v["transcript_path"].as_str().unwrap_or("");
    let mut out = String::new();
    if !cwd.is_empty() {
        let _ = write!(out, "cwd: {cwd}");
    }
    if !transcript.is_empty() {
        let sep = if out.is_empty() { "" } else { "\n" };
        let _ = write!(out, "{sep}transcript: {transcript}");
    }
    if !prompt.is_empty() {
        let sep = if out.is_empty() { "" } else { "\n" };
        let _ = write!(out, "{sep}prompt: {prompt}");
    }
    if out.is_empty() {
        v.to_string()
    } else {
        out
    }
}

fn ts_ms(v: &Value) -> u128 {
    v["timestamp"]["unix_ms"]
        .as_u64()
        .map(|n| n as u128)
        .unwrap_or(0)
}

fn latest_ts(pane: &SessionPane) -> u128 {
    pane.lines.iter().map(|l| l.ts_ms).max().unwrap_or(0)
}

fn render_hook_tail(f: &mut ratatui::Frame, snapshot: &HookTailSnapshot, state: &HookTailState) {
    let area = f.area();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2),
            Constraint::Min(1),
            Constraint::Length(1),
        ])
        .split(area);

    let project = state.project_filter.as_deref().unwrap_or("*");
    let session = state.session_filter.as_deref().unwrap_or("*");
    let title = Line::from(vec![
        Span::styled(
            "tenex-edge debug hook-tail",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("  project="),
        Span::styled(project, Style::default().fg(Color::Yellow)),
        Span::raw("  session="),
        Span::styled(session, Style::default().fg(Color::Yellow)),
        Span::raw("  panes="),
        Span::styled(
            state.pane_limit.to_string(),
            Style::default().fg(Color::Yellow),
        ),
    ]);
    f.render_widget(Paragraph::new(title), chunks[0]);

    if state.focus_mode {
        render_focus(f, chunks[1], snapshot, state);
    } else {
        render_grid(f, chunks[1], snapshot, state);
    }

    let status = if state.status.is_empty() {
        "[q] quit  [tab] focus pane  [enter/f] zoom  [+/-] panes  [p] project  [s] session  [a] all"
            .to_string()
    } else {
        format!(
            "{}  {}",
            state.status,
            "[q] quit  [tab] focus pane  [enter/f] zoom  [+/-] panes  [p] project  [s] session  [a] all"
        )
    };
    f.render_widget(
        Paragraph::new(Line::from(Span::styled(
            status,
            Style::default().fg(Color::DarkGray),
        ))),
        chunks[2],
    );
}

fn render_grid(
    f: &mut ratatui::Frame,
    area: Rect,
    snapshot: &HookTailSnapshot,
    state: &HookTailState,
) {
    let count = snapshot.panes.len().min(state.pane_limit);
    if count == 0 {
        render_empty(f, area, snapshot);
        return;
    }
    let rects = grid_rects(area, count);
    for (i, rect) in rects.into_iter().enumerate() {
        if let Some(pane) = snapshot.panes.get(i) {
            render_pane(f, rect, pane, i == state.focused, false);
        }
    }
}

fn render_focus(
    f: &mut ratatui::Frame,
    area: Rect,
    snapshot: &HookTailSnapshot,
    state: &HookTailState,
) {
    if let Some(pane) = snapshot.panes.get(state.focused) {
        render_pane(f, area, pane, true, true);
    } else {
        render_empty(f, area, snapshot);
    }
}

fn render_empty(f: &mut ratatui::Frame, area: Rect, snapshot: &HookTailSnapshot) {
    let mut lines = vec![Line::from("No session telemetry yet.")];
    if !snapshot.unscoped.is_empty() {
        lines.push(Line::from(""));
        lines.push(Line::from("Recent unscoped commands:"));
        for line in snapshot.unscoped.iter().rev().take(8) {
            lines.push(render_debug_line(line, true));
        }
    }
    f.render_widget(
        Paragraph::new(lines).block(Block::default().borders(Borders::ALL).title("debug")),
        area,
    );
}

fn render_pane(
    f: &mut ratatui::Frame,
    area: Rect,
    pane: &SessionPane,
    focused: bool,
    expanded: bool,
) {
    let mut title = format!("{}", pane.short);
    if !pane.agent.is_empty() || !pane.project.is_empty() {
        title = format!(
            "{} {}@{}",
            title,
            if pane.agent.is_empty() {
                "?"
            } else {
                &pane.agent
            },
            if pane.project.is_empty() {
                "?"
            } else {
                &pane.project
            },
        );
    }
    let border = if focused {
        Color::Yellow
    } else {
        Color::DarkGray
    };
    let mut lines = Vec::new();
    let take = if expanded {
        pane.lines.len()
    } else {
        (area.height as usize).saturating_sub(2).max(1)
    };
    let start = pane.lines.len().saturating_sub(take);
    for line in pane.lines.iter().skip(start) {
        lines.push(render_debug_line(line, expanded));
    }
    f.render_widget(
        Paragraph::new(lines)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(border))
                    .title(title),
            )
            .wrap(Wrap { trim: true }),
        area,
    );
}

fn render_debug_line(line: &DebugLine, expanded: bool) -> Line<'static> {
    let color = match line.kind {
        DebugKind::Hook => Color::Blue,
        DebugKind::Inject => Color::Green,
        DebugKind::Command => Color::Cyan,
        DebugKind::Error => Color::Red,
        DebugKind::Session => Color::DarkGray,
    };
    let status = match line.ok {
        Some(true) => " ok",
        Some(false) => " fail",
        None => "",
    };
    let detail = if expanded {
        line.detail.replace('\n', " / ")
    } else {
        truncate(&line.detail.replace('\n', " / "), 180)
    };
    Line::from(vec![
        Span::styled(
            kind_label(line.kind),
            Style::default().fg(color).add_modifier(Modifier::BOLD),
        ),
        Span::raw(status.to_string()),
        Span::raw(" "),
        Span::styled(line.title.clone(), Style::default().fg(Color::White)),
        Span::raw(" "),
        Span::styled(detail, Style::default().fg(Color::Gray)),
    ])
}

fn kind_label(kind: DebugKind) -> &'static str {
    match kind {
        DebugKind::Hook => "hook",
        DebugKind::Inject => "inj ",
        DebugKind::Command => "cmd ",
        DebugKind::Error => "err ",
        DebugKind::Session => "sess",
    }
}

fn truncate(s: &str, max: usize) -> String {
    let mut out = String::new();
    for (idx, ch) in s.chars().enumerate() {
        if idx >= max {
            out.push_str("...");
            return out;
        }
        out.push(ch);
    }
    out
}

fn grid_rects(area: Rect, count: usize) -> Vec<Rect> {
    let cols = (count as f64).sqrt().ceil() as usize;
    let cols = cols.max(1);
    let rows = count.div_ceil(cols).max(1);
    let row_constraints = even_constraints(rows);
    let col_constraints = even_constraints(cols);
    let row_rects = Layout::default()
        .direction(Direction::Vertical)
        .constraints(row_constraints)
        .split(area);
    let mut rects = Vec::new();
    for row in row_rects.iter() {
        let cols_rects = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(col_constraints.clone())
            .split(*row);
        for col in cols_rects.iter() {
            if rects.len() < count {
                rects.push(*col);
            }
        }
    }
    rects
}

fn even_constraints(n: usize) -> Vec<Constraint> {
    (0..n)
        .map(|_| Constraint::Ratio(1, n as u32))
        .collect::<Vec<_>>()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn command_session_prefers_env_then_explicit_flag() {
        let v = serde_json::json!({
            "env": {"TENEX_EDGE_SESSION": "env-session"},
            "command": {"explicit_session": "flag-session"},
            "process": {"cwd": "/tmp"}
        });
        assert_eq!(command_session(&v).as_deref(), Some("env-session"));

        let v = serde_json::json!({
            "env": {},
            "command": {"explicit_session": "flag-session"},
            "process": {"cwd": "/tmp"}
        });
        assert_eq!(command_session(&v).as_deref(), Some("flag-session"));
    }

    #[test]
    fn hook_session_accepts_codex_field_variants() {
        let v = serde_json::json!({"conversation_id": "codex-session"});
        assert_eq!(hook_session(&v).as_deref(), Some("codex-session"));
    }

    #[test]
    fn command_session_can_infer_unique_live_agent_project() {
        let mut panes = BTreeMap::new();
        panes.insert(
            "session-a".to_string(),
            SessionPane {
                session: "session-a".to_string(),
                project: "proj".to_string(),
                agent: "coder".to_string(),
                ..SessionPane::default()
            },
        );
        assert_eq!(
            infer_command_session(&panes, "coder", "proj").as_deref(),
            Some("session-a")
        );

        panes.insert(
            "session-b".to_string(),
            SessionPane {
                session: "session-b".to_string(),
                project: "proj".to_string(),
                agent: "coder".to_string(),
                ..SessionPane::default()
            },
        );
        assert_eq!(infer_command_session(&panes, "coder", "proj"), None);
    }
}
