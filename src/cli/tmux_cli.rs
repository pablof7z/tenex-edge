use super::*;

// ── tmux_run ──────────────────────────────────────────────────────────────────

/// Entry point for `tenex-edge tmux <action>`.
pub(super) async fn tmux_run(action: TmuxAction) -> Result<()> {
    match action {
        TmuxAction::Status => tmux_status().await,
        TmuxAction::Send { session } => tmux_send(session).await,
        TmuxAction::Spawn { agent, project } => tmux_spawn(agent, project).await,
        TmuxAction::Attach { session } => tmux_attach(session).await,
        TmuxAction::Resume { session } => tmux_resume(session).await,
    }
}

// ── status ────────────────────────────────────────────────────────────────────

async fn tmux_status() -> Result<()> {
    use owo_colors::OwoColorize as _;

    let v = crate::daemon::blocking::call("tmux_status", serde_json::json!({}))
        .context("tmux_status RPC")?;

    let endpoints = v["endpoints"].as_array().cloned().unwrap_or_default();

    if endpoints.is_empty() {
        println!("No tmux endpoints registered.");
        return Ok(());
    }

    println!(
        "{:<22} {:<8} {:<12} {}",
        "session".bold(),
        "pane".bold(),
        "command".bold(),
        "alive".bold()
    );
    for ep in &endpoints {
        let sid = ep["session_id"].as_str().unwrap_or("");
        let pane = ep["pane_id"].as_str().unwrap_or("");
        let cmd = ep["pane_command"].as_str().unwrap_or("");
        let alive = ep["alive"].as_bool().unwrap_or(false);
        let alive_str = if alive {
            "yes".green().to_string()
        } else {
            "DEAD".red().to_string()
        };
        println!("{sid:<22} {pane:<8} {cmd:<12} {alive_str}");
    }
    Ok(())
}

// ── send (manual doorbell) ────────────────────────────────────────────────────

async fn tmux_send(session: String) -> Result<()> {
    let v = crate::daemon::blocking::call("tmux_send", serde_json::json!({ "session": session }))
        .context("tmux_send RPC")?;

    let injected = v["injected"].as_bool().unwrap_or(false);
    if injected {
        println!("Doorbell injected.");
    } else {
        let reason = v["reason"].as_str().unwrap_or("unknown");
        println!("Doorbell not sent: {reason}");
    }
    Ok(())
}

// ── spawn ─────────────────────────────────────────────────────────────────────

async fn tmux_spawn(agent: String, project: Option<String>) -> Result<()> {
    let project = project
        .unwrap_or_else(|| crate::project::resolve(&std::env::current_dir().unwrap_or_default()));
    let v = crate::daemon::blocking::call(
        "tmux_spawn",
        serde_json::json!({ "agent": agent, "project": project }),
    )
    .context("tmux_spawn RPC")?;

    let pane_id = v["pane_id"].as_str().unwrap_or("?");
    println!("Spawned pane {pane_id} for agent {agent} in project {project}.");
    Ok(())
}

// ── attach ────────────────────────────────────────────────────────────────────

async fn tmux_attach(session: String) -> Result<()> {
    attach_session(&session)
}

// ── resume ────────────────────────────────────────────────────────────────────

async fn tmux_resume(session: String) -> Result<()> {
    let pane = resume_to_pane(&session)?;
    match pane {
        Some(pane_id) => attach_pane(&pane_id),
        None => Ok(()),
    }
}

/// Session id of the currently-selected row IF it is resumable — any local Live
/// row (attachable or not: an in-tmux session can still be replayed) or any
/// Resumable row. `None` for Spawnable rows. The daemon makes the final call on
/// whether a token exists; this just maps cursor → session id.
fn selected_resume_sid(
    live: &[&LiveRow],
    spawnable_count: usize,
    resumable: &[&ResumeRow],
    selected: usize,
) -> Option<String> {
    if selected < live.len() {
        return Some(live[selected].session_id.clone());
    }
    let resume_base = live.len() + spawnable_count;
    if selected >= resume_base {
        return resumable
            .get(selected - resume_base)
            .map(|r| r.session_id.clone());
    }
    None
}

const SEVEN_DAYS: u64 = 7 * 24 * 3600;

fn compute_project_tabs(data: &TuiData) -> ProjectTabs {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    // Projects with at least one live session — always visible, sorted.
    let live_projects: std::collections::BTreeSet<String> = data
        .live
        .iter()
        .filter(|r| !r.project.is_empty())
        .map(|r| r.project.clone())
        .collect();

    // Track latest created_at per project from resumable sessions.
    let mut last_active: std::collections::BTreeMap<String, u64> =
        std::collections::BTreeMap::new();
    for row in &data.resumable {
        if !row.project.is_empty() {
            let e = last_active.entry(row.project.clone()).or_insert(0);
            *e = (*e).max(row.created_at);
        }
    }

    let mut visible_recent: Vec<String> = Vec::new();
    let mut hidden: Vec<String> = Vec::new();

    for (proj, t) in &last_active {
        if live_projects.contains(proj) {
            continue; // already in live_projects
        }
        if now.saturating_sub(*t) < SEVEN_DAYS {
            visible_recent.push(proj.clone());
        } else {
            hidden.push(proj.clone());
        }
    }
    visible_recent.sort();
    hidden.sort();

    // Also include live projects in last_active so they show up consistently.
    for proj in &live_projects {
        last_active.entry(proj.clone()).or_insert(now);
    }

    let mut visible: Vec<String> = live_projects.into_iter().collect();
    visible.extend(visible_recent);

    ProjectTabs { visible, hidden }
}

fn tab_project<'a>(tabs: &'a [String], tab_idx: usize) -> Option<&'a str> {
    if tab_idx == 0 {
        None
    } else {
        tabs.get(tab_idx - 1).map(|s| s.as_str())
    }
}

fn filter_live<'a>(data: &'a TuiData, project_filter: Option<&str>) -> Vec<&'a LiveRow> {
    data.live
        .iter()
        .filter(|r| project_filter.map(|p| r.project == p).unwrap_or(true))
        .collect()
}

fn filter_resumable<'a>(
    data: &'a TuiData,
    project_filter: Option<&str>,
    exited_hours: Option<u64>,
) -> Vec<&'a ResumeRow> {
    let hours = match exited_hours {
        None => return vec![],
        Some(h) => h,
    };
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let cutoff = now.saturating_sub(hours * 3600);
    data.resumable
        .iter()
        .filter(|r| {
            r.created_at >= cutoff
                && project_filter.map(|p| r.project == p).unwrap_or(true)
        })
        .collect()
}

fn update_tabs_after_refresh(data: &TuiData, pt: &mut ProjectTabs, tab_idx: &mut usize) {
    let mut new_pt = compute_project_tabs(data);
    // Preserve the currently-selected project tab even if it became "hidden"
    // (e.g., selected via fuzzy search but older than 7 days).
    let current_proj = if *tab_idx > 0 {
        pt.visible.get(*tab_idx - 1).cloned()
    } else {
        None
    };
    if let Some(proj) = current_proj {
        if let Some(idx) = new_pt.visible.iter().position(|p| *p == proj) {
            *tab_idx = idx + 1;
        } else if let Some(hi) = new_pt.hidden.iter().position(|p| *p == proj) {
            // Was hidden but user has it selected — keep it visible.
            let pinned = new_pt.hidden.remove(hi);
            new_pt.visible.push(pinned);
            *tab_idx = new_pt.visible.len();
        } else {
            *tab_idx = 0;
        }
    }
    *pt = new_pt;
}

/// Compute fuzzy matches for `query` across all projects (visible + hidden).
/// Case-insensitive substring match; visible projects listed first.
fn fuzzy_matches(pt: &ProjectTabs, query: &str) -> Vec<String> {
    let q = query.to_lowercase();
    pt.visible
        .iter()
        .chain(pt.hidden.iter())
        .filter(|p| p.to_lowercase().contains(&q))
        .cloned()
        .collect()
}

fn draw_search(
    pt: &ProjectTabs,
    query: &str,
    sel: usize,
    scroll: &mut usize,
) -> Result<()> {
    use owo_colors::OwoColorize as _;

    let matches = fuzzy_matches(pt, query);

    let mut body: Vec<String> = Vec::new();
    let mut sel_line: Option<usize> = None;
    for (i, proj) in matches.iter().enumerate() {
        let is_sel = i == sel;
        if is_sel {
            sel_line = Some(body.len());
        }
        let cursor = if is_sel { "►" } else { " " };
        // Mark hidden projects dimmed
        let is_hidden = pt.hidden.contains(proj);
        if is_sel {
            body.push(format!("  {} {}", cursor, proj.bold()));
        } else if is_hidden {
            body.push(format!("  {} {}", cursor, proj.dimmed()));
        } else {
            body.push(format!("  {} {}", cursor, proj));
        }
    }
    if matches.is_empty() {
        body.push(format!("    {}", "(no matches)".dimmed()));
    }

    let (_, term_rows) = terminal::size().unwrap_or((80, 24));
    // chrome: title + search input + sep + blank = 4 top; blank + help = 2 bottom
    let top_chrome = 4usize;
    let bottom_chrome = 2usize;
    let viewport = (term_rows as usize)
        .saturating_sub(top_chrome + bottom_chrome)
        .max(1);

    if let Some(s) = sel_line {
        if s < *scroll {
            *scroll = s;
        } else if s >= *scroll + viewport {
            *scroll = s + 1 - viewport;
        }
    }
    let max_scroll = body.len().saturating_sub(viewport);
    if *scroll > max_scroll {
        *scroll = max_scroll;
    }
    let end = (*scroll + viewport).min(body.len());

    let mut out = String::new();
    let _ = writeln!(out, "{}", "tenex-edge tmux".bold());
    let _ = writeln!(out, "  / {}_", query);
    let _ = writeln!(out, "{}", "─".repeat(60).dimmed());
    let _ = writeln!(out);
    for line in &body[*scroll..end] {
        let _ = writeln!(out, "{line}");
    }
    let _ = writeln!(out);
    let _ = writeln!(
        out,
        "  {}",
        "[↑↓] move  [↵] select  [esc] cancel".dimmed()
    );

    let mut stdout = io::stdout();
    execute!(stdout, MoveTo(0, 0), Clear(ClearType::All))?;
    for line in out.lines() {
        write!(stdout, "{line}\r\n")?;
    }
    stdout.flush()?;
    Ok(())
}

/// TUI variant: resume `session`, returning the new pane id or an `Err(message)`
/// suitable for the status line (never writes to stderr, which raw mode mangles).
fn resume_in_tui(session: &str) -> std::result::Result<String, String> {
    let v = crate::daemon::blocking::call("tmux_resume", serde_json::json!({ "session": session }))
        .map_err(|e| format!("Resume failed: {e}"))?;
    match v["pane_id"].as_str() {
        Some(p) => Ok(p.to_string()),
        None => Err(format!(
            "Cannot resume: {}",
            v["error"].as_str().unwrap_or("unknown error")
        )),
    }
}

/// Ask the daemon to resume `session`, returning the new pane id (or `None`,
/// after printing the error). Shared by the CLI verb and the TUI.
fn resume_to_pane(session: &str) -> Result<Option<String>> {
    let v =
        crate::daemon::blocking::call("tmux_resume", serde_json::json!({ "session": session }))
            .context("tmux_resume RPC")?;
    match v["pane_id"].as_str() {
        Some(p) => Ok(Some(p.to_string())),
        None => {
            let err = v["error"].as_str().unwrap_or("unknown error");
            eprintln!("Cannot resume: {err}");
            Ok(None)
        }
    }
}

// ── shared attach logic ───────────────────────────────────────────────────────

/// Resolve a session id to its live tmux pane id via the daemon, or `None`.
fn pane_for_session(session_id: &str) -> Option<String> {
    let v = crate::daemon::blocking::call("tmux_attach", serde_json::json!({ "session": session_id }))
        .ok()?;
    v["pane_id"].as_str().map(str::to_string)
}

fn attach_session(session_id: &str) -> Result<()> {
    let v =
        crate::daemon::blocking::call("tmux_attach", serde_json::json!({ "session": session_id }))
            .context("tmux_attach RPC")?;

    let pane_id = match v["pane_id"].as_str() {
        Some(p) => p.to_string(),
        None => {
            let err = v["error"].as_str().unwrap_or("unknown error");
            eprintln!("Cannot attach: {err}");
            return Ok(());
        }
    };

    attach_pane(&pane_id)
}

// ── TUI ───────────────────────────────────────────────────────────────────────

struct TuiTerminal;

impl TuiTerminal {
    fn enter() -> Result<Self> {
        terminal::enable_raw_mode()?;
        execute!(io::stdout(), EnterAlternateScreen, Hide)?;
        Ok(Self)
    }

    /// Temporarily restore the normal terminal so a child process (e.g. a tmux
    /// client) can own the tty, without dropping our guard. Pair with `resume`.
    fn suspend() {
        let _ = terminal::disable_raw_mode();
        let _ = execute!(io::stdout(), Show, LeaveAlternateScreen);
    }

    /// Re-enter the alternate-screen raw-mode TUI after a `suspend`.
    fn resume() {
        let _ = terminal::enable_raw_mode();
        let _ = execute!(io::stdout(), EnterAlternateScreen, Hide);
    }
}

impl Drop for TuiTerminal {
    fn drop(&mut self) {
        let _ = terminal::disable_raw_mode();
        let _ = execute!(io::stdout(), Show, LeaveAlternateScreen);
    }
}

struct LiveRow {
    slug: String,
    host: String,
    project: String,
    session_id: String,    // full raw id for RPC calls
    session_short: String, // short display code (6 chars)
    status: String,
    attachable: bool, // has a live tmux endpoint
}

struct SpawnRow {
    slug: String,
    host: String,
    command: String,
}

struct ResumeRow {
    slug: String,
    project: String,
    session_id: String,    // full raw id for RPC calls
    session_short: String, // short display code (6 chars)
    title: String,
    created_at: u64,
}

/// Tabs computed from live data: visible projects ordered by activity (live
/// first, then recently-active), plus hidden projects (>7 days inactive).
struct ProjectTabs {
    /// Projects shown in the tab bar. Order: projects with live sessions first
    /// (alphabetically), then recently-active projects (alphabetically).
    visible: Vec<String>,
    /// Projects with no activity in the past 7 days. Only reachable via search.
    hidden: Vec<String>,
}

impl PartialEq for ProjectTabs {
    fn eq(&self, other: &Self) -> bool {
        self.visible == other.visible && self.hidden == other.hidden
    }
}

enum TuiMode {
    Normal,
    Search { query: String, sel: usize },
}

struct TuiData {
    live: Vec<LiveRow>,
    spawnable: Vec<SpawnRow>,
    resumable: Vec<ResumeRow>,
}

fn fetch_tui_data() -> Result<TuiData> {
    let v = crate::daemon::blocking::call(
        "who",
        serde_json::json!({
            "project": null,
            "all": false,
            "all_projects": true,
            "cwd": std::env::current_dir().ok().map(|p| p.to_string_lossy().to_string()),
        }),
    )?;

    let live = v["rows"]
        .as_array()
        .cloned()
        .unwrap_or_default()
        .iter()
        .filter(|r| !r["remote"].as_bool().unwrap_or(false))
        .map(|r| {
            let raw_id = r["session_id"].as_str().unwrap_or("").to_string();
            let session_short = SessionId::from(raw_id.as_str()).to_string();
            LiveRow {
                slug: r["slug"].as_str().unwrap_or("").to_string(),
                host: r["host"].as_str().unwrap_or("").to_string(),
                project: r["project"].as_str().unwrap_or("").to_string(),
                session_id: raw_id,
                session_short,
                status: r["status"].as_str().unwrap_or("").to_string(),
                attachable: r["attachable"].as_bool().unwrap_or(false),
            }
        })
        .collect();

    let spawnable = v["spawnable"]
        .as_array()
        .cloned()
        .unwrap_or_default()
        .iter()
        .map(|r| SpawnRow {
            slug: r["slug"].as_str().unwrap_or("").to_string(),
            host: r["host"].as_str().unwrap_or("").to_string(),
            command: r["command"].as_str().unwrap_or("").to_string(),
        })
        .collect();

    // Resumable (dead, but replayable) sessions come from a dedicated RPC.
    // Fail soft: an older daemon without it just yields an empty section.
    let resumable = crate::daemon::blocking::call("tmux_resumable", serde_json::json!({}))
        .ok()
        .and_then(|rv| rv["resumable"].as_array().cloned())
        .unwrap_or_default()
        .iter()
        .map(|r| {
            let raw_id = r["session_id"].as_str().unwrap_or("").to_string();
            let session_short = SessionId::from(raw_id.as_str()).to_string();
            ResumeRow {
                slug: r["slug"].as_str().unwrap_or("").to_string(),
                project: r["project"].as_str().unwrap_or("").to_string(),
                session_id: raw_id,
                session_short,
                title: r["title"].as_str().unwrap_or("").to_string(),
                created_at: r["created_at"].as_u64().unwrap_or(0),
            }
        })
        .collect();

    Ok(TuiData {
        live,
        spawnable,
        resumable,
    })
}

fn draw_tui(
    data: &TuiData,
    selected: usize,
    status: &str,
    scroll: &mut usize,
    tabs: &[String],
    tab_idx: usize,
    exited_hours: Option<u64>,
) -> Result<()> {
    use owo_colors::OwoColorize as _;

    let project_filter = tab_project(tabs, tab_idx);
    let fl = filter_live(data, project_filter);
    let fr = filter_resumable(data, project_filter, exited_hours);

    // Build the scrollable body as a flat list of lines, recording which line
    // holds the selected row so the viewport can keep it visible.
    let mut body: Vec<String> = Vec::new();
    let mut sel_line: Option<usize> = None;

    // Live sessions section
    body.push(format!("  {}", "Live sessions".bold()));
    if fl.is_empty() {
        body.push(format!("    {}", "(none)".dimmed()));
    } else {
        for (i, row) in fl.iter().enumerate() {
            let is_sel = i == selected;
            if is_sel {
                sel_line = Some(body.len());
            }
            let cursor = if is_sel { "►" } else { " " };
            // In All tab, show slug@project so project context is clear.
            let label = if project_filter.is_none() {
                format!("{}@{}", row.slug, row.project)
            } else {
                format!("{}@{}", row.slug, row.host)
            };
            let session_tag = format!("[session {}]", row.session_short);
            let status_str = if row.status.trim().is_empty() {
                "idle".to_string()
            } else {
                row.status.trim().to_string()
            };
            if !row.attachable {
                body.push(format!(
                    "  {} {}  {}  {} {}",
                    cursor,
                    label.dimmed(),
                    session_tag.dimmed(),
                    status_str.dimmed(),
                    "[no tmux]".dimmed(),
                ));
            } else if is_sel {
                body.push(format!(
                    "  {} {}  {}  {}",
                    cursor,
                    label.cyan().bold(),
                    session_tag.yellow(),
                    status_str,
                ));
            } else {
                body.push(format!(
                    "  {} {}  {}  {}",
                    cursor,
                    label.cyan(),
                    session_tag.yellow(),
                    status_str.dimmed(),
                ));
            }
        }
    }

    // Agents section (spawnable)
    body.push(String::new());
    body.push(format!("  {}", "Agents".bold()));
    if data.spawnable.is_empty() {
        body.push(format!("    {}", "(none)".dimmed()));
    } else {
        for (i, row) in data.spawnable.iter().enumerate() {
            let abs_idx = fl.len() + i;
            let is_sel = abs_idx == selected;
            if is_sel {
                sel_line = Some(body.len());
            }
            let cursor = if is_sel { "►" } else { " " };
            let label = format!("{}@{}", row.slug, row.host);
            let tag = format!("[{}]", row.command);
            if is_sel {
                body.push(format!("  {} {}  {}", cursor, label.bold(), tag.dimmed()));
            } else {
                body.push(format!("  {} {}  {}", cursor, label.dimmed(), tag.dimmed()));
            }
        }
    }

    // Exited sessions section (only when exited_hours is Some)
    if let Some(hours) = exited_hours {
        body.push(String::new());
        body.push(format!(
            "  {} {}",
            "Exited sessions".bold(),
            format!("(past {hours}h)").dimmed()
        ));
        if fr.is_empty() {
            body.push(format!("    {}", "(none)".dimmed()));
        } else {
            for (i, row) in fr.iter().enumerate() {
                let abs_idx = fl.len() + data.spawnable.len() + i;
                let is_sel = abs_idx == selected;
                if is_sel {
                    sel_line = Some(body.len());
                }
                let cursor = if is_sel { "►" } else { " " };
                let label = if project_filter.is_none() {
                    format!("{}@{}", row.slug, row.project)
                } else {
                    row.slug.clone()
                };
                let session_tag = format!("[session {}]", row.session_short);
                let title = if row.title.trim().is_empty() {
                    String::new()
                } else {
                    row.title.trim().to_string()
                };
                if is_sel {
                    body.push(format!(
                        "  {} {}  {}  {}",
                        cursor,
                        label.magenta().bold(),
                        session_tag.yellow(),
                        title,
                    ));
                } else {
                    body.push(format!(
                        "  {} {}  {}  {}",
                        cursor,
                        label.magenta(),
                        session_tag.dimmed(),
                        title.dimmed(),
                    ));
                }
            }
        }
    }

    // Viewport math: fixed chrome is title+tabs+rule+blank (top, 4 lines) and
    // blank+help+optional-status (bottom). The body scrolls within the rest.
    let (_, term_rows) = terminal::size().unwrap_or((80, 24));
    let top_chrome = 4usize;
    let bottom_chrome = if status.is_empty() { 2 } else { 3 };
    let viewport = (term_rows as usize)
        .saturating_sub(top_chrome + bottom_chrome)
        .max(1);

    // Keep the selected line in view; clamp the offset to valid range.
    if let Some(s) = sel_line {
        if s < *scroll {
            *scroll = s;
        } else if s >= *scroll + viewport {
            *scroll = s + 1 - viewport;
        }
    }
    let max_scroll = body.len().saturating_sub(viewport);
    if *scroll > max_scroll {
        *scroll = max_scroll;
    }
    let end = (*scroll + viewport).min(body.len());

    // Header line carries scroll affordances so the user knows there's more.
    let above = *scroll;
    let below = body.len().saturating_sub(end);
    let mut more = String::new();
    if above > 0 {
        more.push_str(&format!("  ↑{above} more above"));
    }
    if below > 0 {
        more.push_str(&format!("  ↓{below} more below"));
    }

    // Build tab bar: [All] [proj1] [proj2] ...
    let tab_bar = {
        let mut s = String::from("  ");
        if tab_idx == 0 {
            s.push_str(&"[All]".bold().to_string());
        } else {
            s.push_str(&"[All]".dimmed().to_string());
        }
        for (i, tab) in tabs.iter().enumerate() {
            s.push(' ');
            let label = format!("[{tab}]");
            if tab_idx == i + 1 {
                s.push_str(&label.bold().to_string());
            } else {
                s.push_str(&label.dimmed().to_string());
            }
        }
        s
    };

    let exited_hint = match exited_hours {
        None => "[e] show exited".to_string(),
        Some(h) => format!("[e] hide exited  [-/+] {h}h"),
    };

    let mut out = String::new();
    let _ = writeln!(out, "{}{}", "tenex-edge tmux".bold(), more.dimmed());
    let _ = writeln!(out, "{tab_bar}");
    let _ = writeln!(out, "{}", "─".repeat(60).dimmed());
    let _ = writeln!(out);
    for line in &body[*scroll..end] {
        let _ = writeln!(out, "{line}");
    }
    let _ = writeln!(out);
    let _ = writeln!(
        out,
        "  {}",
        format!("[↑↓] move  [←→] tab  [/] search  [a/↵] attach  [n] spawn  {exited_hint}  [q] quit")
            .dimmed()
    );
    if !status.is_empty() {
        let _ = writeln!(out, "  {status}");
    }

    let mut stdout = io::stdout();
    execute!(stdout, MoveTo(0, 0), Clear(ClearType::All))?;
    for line in out.lines() {
        write!(stdout, "{line}\r\n")?;
    }
    stdout.flush()?;
    Ok(())
}

/// Interactive TUI for `tenex-edge tmux` (bare, no subcommand).
/// Shows live sessions and spawnable agents; lets the user attach or spawn.
pub(super) fn tmux_tui() -> Result<()> {
    let refresh = Duration::from_secs(2);
    let mut selected: usize = 0;
    let mut status_msg = String::new();
    let mut tab_idx: usize = 0;
    let mut show_exited: bool = false;
    let mut exited_hours: u64 = 4;
    let mut mode = TuiMode::Normal;

    // Initial fetch before entering raw mode: fail fast if daemon is down.
    let mut data = fetch_tui_data()?;
    let mut pt = compute_project_tabs(&data);

    {
        let _terminal = TuiTerminal::enter()?;
        let mut next_refresh = Instant::now() + refresh;
        let mut scroll: usize = 0;

        loop {
            // ── draw ──────────────────────────────────────────────────────
            match &mode {
                TuiMode::Search { query, sel } => {
                    draw_search(&pt, query, *sel, &mut scroll)?;
                }
                TuiMode::Normal => {
                    let exited_opt = if show_exited { Some(exited_hours) } else { None };
                    // Compute filtered totals (borrows released at end of block).
                    let total = {
                        let pf = tab_project(&pt.visible, tab_idx);
                        filter_live(&data, pf).len()
                            + data.spawnable.len()
                            + filter_resumable(&data, pf, exited_opt).len()
                    };
                    if total > 0 && selected >= total {
                        selected = total - 1;
                    }
                    draw_tui(
                        &data,
                        selected,
                        &status_msg,
                        &mut scroll,
                        &pt.visible,
                        tab_idx,
                        exited_opt,
                    )?;
                }
            }

            let wait = next_refresh
                .saturating_duration_since(Instant::now())
                .min(Duration::from_millis(100));

            let mut should_break = false;
            let mut pending_attach: Option<String> = None;

            if event::poll(wait)? {
                if let TermEvent::Key(key) = event::read()? {
                    match &mut mode {
                        // ── search mode ───────────────────────────────────
                        TuiMode::Search { query, sel } => {
                            match key.code {
                                KeyCode::Esc => {
                                    mode = TuiMode::Normal;
                                    scroll = 0;
                                }
                                KeyCode::Enter => {
                                    let matches = fuzzy_matches(&pt, query);
                                    if let Some(proj) = matches.get(*sel).cloned() {
                                        if let Some(idx) =
                                            pt.visible.iter().position(|p| *p == proj)
                                        {
                                            tab_idx = idx + 1;
                                        } else {
                                            // Hidden project: inject into visible temporarily.
                                            pt.hidden.retain(|p| p != &proj);
                                            pt.visible.push(proj);
                                            tab_idx = pt.visible.len();
                                        }
                                        selected = 0;
                                    }
                                    mode = TuiMode::Normal;
                                    scroll = 0;
                                    status_msg.clear();
                                }
                                KeyCode::Up | KeyCode::Char('k') => {
                                    *sel = sel.saturating_sub(1);
                                }
                                KeyCode::Down | KeyCode::Char('j') => {
                                    let n = fuzzy_matches(&pt, query).len();
                                    if *sel + 1 < n {
                                        *sel += 1;
                                    }
                                }
                                KeyCode::Backspace => {
                                    query.pop();
                                    *sel = 0;
                                }
                                KeyCode::Char(c)
                                    if !key.modifiers.contains(KeyModifiers::CONTROL) =>
                                {
                                    query.push(c);
                                    *sel = 0;
                                }
                                _ => {}
                            }
                        }
                        // ── normal mode ───────────────────────────────────
                        TuiMode::Normal => {
                            let exited_opt = if show_exited { Some(exited_hours) } else { None };
                            // We need filtered views. Use a block so borrows of
                            // `data` are released before any `data = fresh` below.
                            let total = {
                                let pf = tab_project(&pt.visible, tab_idx);
                                filter_live(&data, pf).len()
                                    + data.spawnable.len()
                                    + filter_resumable(&data, pf, exited_opt).len()
                            };
                            {
                                let pf = tab_project(&pt.visible, tab_idx);
                                let fl = filter_live(&data, pf);
                                let fr = filter_resumable(&data, pf, exited_opt);

                                match key.code {
                                    KeyCode::Char('q') | KeyCode::Esc => {
                                        should_break = true;
                                    }
                                    KeyCode::Char('c')
                                        if key.modifiers.contains(KeyModifiers::CONTROL) =>
                                    {
                                        should_break = true;
                                    }
                                    KeyCode::Up | KeyCode::Char('k') => {
                                        selected = selected.saturating_sub(1);
                                        status_msg.clear();
                                    }
                                    KeyCode::Down | KeyCode::Char('j') => {
                                        if total > 0 && selected + 1 < total {
                                            selected += 1;
                                        }
                                        status_msg.clear();
                                    }
                                    // Left/right: switch project tabs.
                                    KeyCode::Left => {
                                        if tab_idx > 0 {
                                            tab_idx -= 1;
                                            selected = 0;
                                            scroll = 0;
                                            status_msg.clear();
                                        }
                                    }
                                    KeyCode::Right => {
                                        if tab_idx < pt.visible.len() {
                                            tab_idx += 1;
                                            selected = 0;
                                            scroll = 0;
                                            status_msg.clear();
                                        }
                                    }
                                    // /: enter fuzzy project search.
                                    KeyCode::Char('/') => {
                                        mode = TuiMode::Search {
                                            query: String::new(),
                                            sel: 0,
                                        };
                                        scroll = 0;
                                    }
                                    // e: toggle exited sessions.
                                    KeyCode::Char('e') => {
                                        show_exited = !show_exited;
                                        status_msg.clear();
                                    }
                                    // +/= / -: adjust the hours window (only when exited is shown).
                                    KeyCode::Char('+') | KeyCode::Char('=') if show_exited => {
                                        exited_hours = match exited_hours {
                                            h if h >= 48 => h + 24,
                                            h if h >= 12 => h + 6,
                                            h => h + 1,
                                        };
                                        status_msg.clear();
                                    }
                                    KeyCode::Char('-') if show_exited => {
                                        exited_hours = match exited_hours {
                                            h if h > 48 => h - 24,
                                            h if h > 12 => h - 6,
                                            h => h.saturating_sub(1).max(1),
                                        };
                                        status_msg.clear();
                                    }
                                    KeyCode::Enter | KeyCode::Char('a') => {
                                        if selected < fl.len() && fl[selected].attachable {
                                            match pane_for_session(&fl[selected].session_id) {
                                                Some(p) => pending_attach = Some(p),
                                                None => {
                                                    status_msg =
                                                        "Session pane not found.".to_string()
                                                }
                                            }
                                        } else {
                                            match selected_resume_sid(
                                                &fl,
                                                data.spawnable.len(),
                                                &fr,
                                                selected,
                                            ) {
                                                Some(sid) => {
                                                    status_msg = "Resuming...".to_string();
                                                    draw_tui(
                                                        &data,
                                                        selected,
                                                        &status_msg,
                                                        &mut scroll,
                                                        &pt.visible,
                                                        tab_idx,
                                                        exited_opt,
                                                    )?;
                                                    match resume_in_tui(&sid) {
                                                        Ok(pane) => pending_attach = Some(pane),
                                                        Err(msg) => status_msg = msg,
                                                    }
                                                }
                                                None => {
                                                    status_msg =
                                                        "Press [n] to spawn.".to_string();
                                                }
                                            }
                                        }
                                    }
                                    KeyCode::Char('r') => {
                                        if let Some(sid) = selected_resume_sid(
                                            &fl,
                                            data.spawnable.len(),
                                            &fr,
                                            selected,
                                        ) {
                                            status_msg = "Resuming...".to_string();
                                            draw_tui(
                                                &data,
                                                selected,
                                                &status_msg,
                                                &mut scroll,
                                                &pt.visible,
                                                tab_idx,
                                                exited_opt,
                                            )?;
                                            match resume_in_tui(&sid) {
                                                Ok(pane) => pending_attach = Some(pane),
                                                Err(msg) => status_msg = msg,
                                            }
                                        }
                                    }
                                    KeyCode::Char('n') if selected >= fl.len() => {
                                        let si = selected - fl.len();
                                        if si < data.spawnable.len() {
                                            let slug = data.spawnable[si].slug.clone();
                                            let project = crate::project::resolve(
                                                &std::env::current_dir().unwrap_or_default(),
                                            );
                                            status_msg = format!("Spawning {slug}...");
                                            draw_tui(
                                                &data,
                                                selected,
                                                &status_msg,
                                                &mut scroll,
                                                &pt.visible,
                                                tab_idx,
                                                exited_opt,
                                            )?;
                                            match crate::daemon::blocking::call(
                                                "tmux_spawn",
                                                serde_json::json!({
                                                    "agent": slug,
                                                    "project": project,
                                                }),
                                            ) {
                                                Ok(v) => {
                                                    pending_attach =
                                                        v["pane_id"].as_str().map(str::to_string);
                                                }
                                                Err(e) => {
                                                    status_msg = format!("Spawn failed: {e}")
                                                }
                                            }
                                        }
                                    }
                                    _ => {}
                                }
                                // fl, fr, pf borrows of `data` are released here.
                            }
                        }
                    }
                }
            }

            if should_break {
                break;
            }

            // Attach (blocking) then return to the TUI.
            if let Some(pane) = pending_attach {
                TuiTerminal::suspend();
                let res = attach_pane_blocking(&pane);
                TuiTerminal::resume();
                status_msg = match res {
                    Ok(()) => String::new(),
                    Err(e) => format!("Attach failed: {e:#}"),
                };
                if let Ok(fresh) = fetch_tui_data() {
                    update_tabs_after_refresh(&fresh, &mut pt, &mut tab_idx);
                    data = fresh;
                }
                next_refresh = Instant::now() + refresh;
            }

            // Periodic refresh.
            if Instant::now() >= next_refresh {
                if let Ok(fresh) = fetch_tui_data() {
                    update_tabs_after_refresh(&fresh, &mut pt, &mut tab_idx);
                    data = fresh;
                }
                next_refresh = Instant::now() + refresh;
            }
        }
    }; // _terminal dropped here — raw mode disabled, alternate screen exited

    Ok(())
}

/// Resolve a pane id (e.g. "%7") to `(session_name, window_index)` by scanning
/// every pane in every session. Returns `None` if the pane is gone.
fn resolve_pane_location(pane_id: &str) -> Option<(String, String)> {
    let out = std::process::Command::new("tmux")
        .args([
            "list-panes",
            "-a",
            "-F",
            "#{pane_id} #{session_name} #{window_index}",
        ])
        .output()
        .ok()?;
    let stdout = String::from_utf8_lossy(&out.stdout);
    stdout.lines().find_map(|line| {
        let mut parts = line.splitn(3, ' ');
        let pid = parts.next()?;
        let session = parts.next()?;
        let window = parts.next()?;
        if pid == pane_id {
            Some((session.to_string(), window.to_string()))
        } else {
            None
        }
    })
}

/// The tmux session that owns `pane_id` (one session per agent now), or `None`
/// if the pane is gone.
fn session_of_pane(pane_id: &str) -> Option<String> {
    resolve_pane_location(pane_id).map(|(session, _window)| session)
}

/// Attach to the session owning `pane_id` as a BLOCKING child, returning when the
/// user detaches (Ctrl-b d) or the session ends. `$TMUX` is stripped from the
/// child so it works even when the caller is itself inside tmux (nested attach) —
/// this is what lets the `tenex-edge tmux` TUI stay running underneath and be
/// returned to afterward. No grouped "view" session is needed: each agent is its
/// own single-window session, so there is no current-window pointer to mirror.
fn attach_pane_blocking(pane_id: &str) -> Result<()> {
    let Some(session) = session_of_pane(pane_id) else {
        anyhow::bail!("pane {pane_id} not found in any tmux session");
    };
    std::process::Command::new("tmux")
        .args(["attach-session", "-t", &session])
        .env_remove("TMUX")
        .status()
        .context("tmux attach-session")?;
    Ok(())
}

/// Attach by replacing this process (for the one-shot CLI verbs, where returning
/// to a shell on detach is the right behavior). Inside tmux it switches the
/// current client; outside it execs `attach-session`.
fn attach_pane(pane_id: &str) -> Result<()> {
    let Some(session) = session_of_pane(pane_id) else {
        eprintln!("Pane {pane_id} not found in any tmux session.");
        return Ok(());
    };

    let in_tmux = std::env::var("TMUX").map(|v| !v.is_empty()).unwrap_or(false);
    if in_tmux {
        let status = std::process::Command::new("tmux")
            .args(["switch-client", "-t", &session])
            .status()
            .context("tmux switch-client")?;
        if !status.success() {
            eprintln!("tmux switch-client failed for session {session}");
        }
        return Ok(());
    }

    use std::os::unix::process::CommandExt;
    let err = std::process::Command::new("tmux")
        .args(["attach-session", "-t", &session])
        .exec(); // replaces this process; only returns on error
    anyhow::bail!("exec tmux attach-session: {err}");
}
