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
fn selected_resume_sid(data: &TuiData, selected: usize) -> Option<String> {
    if selected < data.live.len() {
        return Some(data.live[selected].session_id.clone());
    }
    let resume_base = data.live.len() + data.spawnable.len();
    if selected >= resume_base {
        return data
            .resumable
            .get(selected - resume_base)
            .map(|r| r.session_id.clone());
    }
    None
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
    session_id: String,    // full raw id for RPC calls
    session_short: String, // short display code (6 chars)
    title: String,
    alive: bool,
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
                session_id: raw_id,
                session_short,
                title: r["title"].as_str().unwrap_or("").to_string(),
                alive: r["alive"].as_bool().unwrap_or(false),
            }
        })
        .collect();

    Ok(TuiData {
        live,
        spawnable,
        resumable,
    })
}

fn draw_tui(data: &TuiData, selected: usize, status: &str, scroll: &mut usize) -> Result<()> {
    use owo_colors::OwoColorize as _;

    // Build the scrollable body as a flat list of lines, recording which line
    // holds the selected row so the viewport can keep it visible.
    let mut body: Vec<String> = Vec::new();
    let mut sel_line: Option<usize> = None;

    body.push(format!("  {}", "Live sessions".bold()));
    if data.live.is_empty() {
        body.push(format!("    {}", "(none)".dimmed()));
    } else {
        for (i, row) in data.live.iter().enumerate() {
            let is_sel = i == selected;
            if is_sel {
                sel_line = Some(body.len());
            }
            let cursor = if is_sel { "►" } else { " " };
            let label = format!("{}@{}", row.slug, row.host);
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

    body.push(String::new());
    body.push(format!("  {}", "Spawnable (no session)".bold()));
    if data.spawnable.is_empty() {
        body.push(format!("    {}", "(none)".dimmed()));
    } else {
        for (i, row) in data.spawnable.iter().enumerate() {
            let abs_idx = data.live.len() + i;
            let is_sel = abs_idx == selected;
            if is_sel {
                sel_line = Some(body.len());
            }
            let cursor = if is_sel { "►" } else { " " };
            let label = format!("{}@{}", row.slug, row.host);
            let tag = format!("[spawnable via {}]", row.command);
            if is_sel {
                body.push(format!("  {} {}  {}", cursor, label.bold(), tag.dimmed()));
            } else {
                body.push(format!("  {} {}  {}", cursor, label.dimmed(), tag.dimmed()));
            }
        }
    }

    body.push(String::new());
    body.push(format!("  {}", "Resumable (no live pane)".bold()));
    if data.resumable.is_empty() {
        body.push(format!("    {}", "(none)".dimmed()));
    } else {
        for (i, row) in data.resumable.iter().enumerate() {
            let abs_idx = data.live.len() + data.spawnable.len() + i;
            let is_sel = abs_idx == selected;
            if is_sel {
                sel_line = Some(body.len());
            }
            let cursor = if is_sel { "►" } else { " " };
            let label = row.slug.clone();
            let session_tag = format!("[session {}]", row.session_short);
            let state_tag = if row.alive { "[stale pane]" } else { "[exited]" };
            let title = if row.title.trim().is_empty() {
                String::new()
            } else {
                row.title.trim().to_string()
            };
            if is_sel {
                body.push(format!(
                    "  {} {}  {}  {} {}",
                    cursor,
                    label.magenta().bold(),
                    session_tag.yellow(),
                    title,
                    state_tag.dimmed(),
                ));
            } else {
                body.push(format!(
                    "  {} {}  {}  {} {}",
                    cursor,
                    label.magenta(),
                    session_tag.dimmed(),
                    title.dimmed(),
                    state_tag.dimmed(),
                ));
            }
        }
    }

    // Viewport math: fixed chrome is title+rule+blank (top, 3 lines) and
    // blank+help+optional-status (bottom). The body scrolls within the rest.
    let (_, term_rows) = terminal::size().unwrap_or((80, 24));
    let top_chrome = 3usize;
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

    let mut out = String::new();
    let _ = writeln!(out, "{}{}", "tenex-edge tmux".bold(), more.dimmed());
    let _ = writeln!(out, "{}", "─".repeat(60).dimmed());
    let _ = writeln!(out);
    for line in &body[*scroll..end] {
        let _ = writeln!(out, "{line}");
    }
    let _ = writeln!(out);
    let _ = writeln!(
        out,
        "  {}",
        "[↑↓/jk] move   [a/↵] attach   [n] spawn   [r] resume   [q] quit".dimmed()
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

    // Initial fetch before entering raw mode: fail fast if daemon is down.
    let mut data = fetch_tui_data()?;

    {
        let _terminal = TuiTerminal::enter()?;
        let mut next_refresh = Instant::now() + refresh;
        let mut scroll: usize = 0;

        loop {
            let total = data.live.len() + data.spawnable.len() + data.resumable.len();
            if total > 0 && selected >= total {
                selected = total - 1;
            }

            draw_tui(&data, selected, &status_msg, &mut scroll)?;

            let wait = next_refresh
                .saturating_duration_since(Instant::now())
                .min(Duration::from_millis(100));

            if event::poll(wait)? {
                if let TermEvent::Key(key) = event::read()? {
                    // A pane to attach to this iteration. We attach as a blocking
                    // child (suspending the TUI) and resume the TUI afterward, so
                    // detaching (Ctrl-b d) returns here instead of exiting.
                    let mut pending_attach: Option<String> = None;
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => break,
                        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                            break
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
                        // Enter / a: attach if the row has a live tmux pane,
                        // otherwise resume it (a local session not in tmux is
                        // still replayable). Spawnables hint to use 'n'.
                        KeyCode::Enter | KeyCode::Char('a') => {
                            if selected < data.live.len() && data.live[selected].attachable {
                                match pane_for_session(&data.live[selected].session_id) {
                                    Some(p) => pending_attach = Some(p),
                                    None => status_msg = "Session pane not found.".to_string(),
                                }
                            } else {
                                match selected_resume_sid(&data, selected) {
                                    Some(sid) => {
                                        status_msg = "Resuming...".to_string();
                                        draw_tui(&data, selected, &status_msg, &mut scroll)?;
                                        match resume_in_tui(&sid) {
                                            Ok(pane) => pending_attach = Some(pane),
                                            Err(msg) => status_msg = msg,
                                        }
                                    }
                                    None => {
                                        status_msg = "Press [n] to spawn this agent.".to_string();
                                    }
                                }
                            }
                        }
                        // r: resume the selected session — works on any local
                        // Live row (incl. [no tmux]) and any Resumable row.
                        KeyCode::Char('r') => {
                            if let Some(sid) = selected_resume_sid(&data, selected) {
                                status_msg = "Resuming...".to_string();
                                draw_tui(&data, selected, &status_msg, &mut scroll)?;
                                match resume_in_tui(&sid) {
                                    Ok(pane) => pending_attach = Some(pane),
                                    Err(msg) => status_msg = msg,
                                }
                            }
                        }
                        KeyCode::Char('n') if selected >= data.live.len() => {
                            let spawnable_idx = selected - data.live.len();
                            if spawnable_idx < data.spawnable.len() {
                                let slug = data.spawnable[spawnable_idx].slug.clone();
                                let project = crate::project::resolve(
                                    &std::env::current_dir().unwrap_or_default(),
                                );
                                status_msg = format!("Spawning {slug}...");
                                draw_tui(&data, selected, &status_msg, &mut scroll)?;
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
                                    Err(e) => status_msg = format!("Spawn failed: {e}"),
                                }
                            }
                        }
                        _ => {}
                    }

                    // Attach (blocking) then return to the TUI. Suspend the
                    // alternate-screen/raw-mode so the tmux client owns the tty;
                    // re-enter when it detaches.
                    if let Some(pane) = pending_attach {
                        TuiTerminal::suspend();
                        let res = attach_pane_blocking(&pane);
                        TuiTerminal::resume();
                        status_msg = match res {
                            Ok(()) => String::new(),
                            Err(e) => format!("Attach failed: {e:#}"),
                        };
                        if let Ok(fresh) = fetch_tui_data() {
                            data = fresh;
                        }
                        next_refresh = Instant::now() + refresh;
                    }
                }
            }

            // Periodic refresh.
            if Instant::now() >= next_refresh {
                if let Ok(fresh) = fetch_tui_data() {
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
