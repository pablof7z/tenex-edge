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

enum TuiExit {
    Quit,
    Attach(String),     // full session_id
    AttachPane(String), // direct pane_id (used after spawn)
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

    let exit_action = {
        let _terminal = TuiTerminal::enter()?;
        let mut next_refresh = Instant::now() + refresh;
        let mut result = TuiExit::Quit;
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
                                result = TuiExit::Attach(data.live[selected].session_id.clone());
                                break;
                            }
                            match selected_resume_sid(&data, selected) {
                                Some(sid) => {
                                    status_msg = "Resuming...".to_string();
                                    draw_tui(&data, selected, &status_msg, &mut scroll)?;
                                    match resume_in_tui(&sid) {
                                        Ok(pane) => {
                                            result = TuiExit::AttachPane(pane);
                                            break;
                                        }
                                        Err(msg) => {
                                            status_msg = msg;
                                            if let Ok(fresh) = fetch_tui_data() {
                                                data = fresh;
                                            }
                                            next_refresh = Instant::now() + refresh;
                                        }
                                    }
                                }
                                None => {
                                    status_msg = "Press [n] to spawn this agent.".to_string();
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
                                    Ok(pane) => {
                                        result = TuiExit::AttachPane(pane);
                                        break;
                                    }
                                    Err(msg) => {
                                        status_msg = msg;
                                        if let Ok(fresh) = fetch_tui_data() {
                                            data = fresh;
                                        }
                                        next_refresh = Instant::now() + refresh;
                                    }
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
                                        let pane = v["pane_id"].as_str().unwrap_or("?").to_string();
                                        // Switch directly to the new pane.
                                        result = TuiExit::AttachPane(pane);
                                        break;
                                    }
                                    Err(e) => {
                                        status_msg = format!("Spawn failed: {e}");
                                        // Refresh immediately after failed spawn attempt.
                                        if let Ok(fresh) = fetch_tui_data() {
                                            data = fresh;
                                        }
                                        next_refresh = Instant::now() + refresh;
                                    }
                                }
                            }
                        }
                        _ => {}
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

        result
    }; // _terminal dropped here — raw mode disabled, alternate screen exited

    match exit_action {
        TuiExit::Attach(session_id) => attach_session(&session_id),
        TuiExit::AttachPane(pane_id) => attach_pane(&pane_id),
        TuiExit::Quit => Ok(()),
    }
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

/// Create (or reuse) a per-client grouped "view" session that shares the base
/// session's windows but keeps its OWN current-window pointer, then point it at
/// `window`. Returns the view session name, or `None` if creation failed.
///
/// Why: tmux clients attached to the *same* session mirror each other's current
/// window — a single window pointer is shared. Spawned agents all live as
/// windows inside one shared `tenex` session, so two terminals attaching to it
/// would snap to whichever window was selected last. A grouped session
/// (`new-session -t <base>`) shares the window set but selects independently, so
/// each client sees its own agent. `destroy-unattached` reaps the view when the
/// client detaches so they don't accumulate.
fn ensure_view_session(base: &str, window: &str) -> Option<String> {
    let view = format!("{base}-view-{}", std::process::id());

    // Create the grouped view session if it doesn't already exist (idempotent
    // across attach/detach cycles within one client process). Silence stderr:
    // `has-session` prints "can't find session: <view>" on the (expected) miss,
    // which otherwise leaks alarming noise to the user's terminal.
    let exists = std::process::Command::new("tmux")
        .args(["has-session", "-t", &view])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false);
    if !exists {
        let created = std::process::Command::new("tmux")
            .args(["new-session", "-d", "-t", base, "-s", &view])
            .status()
            .map(|s| s.success())
            .unwrap_or(false);
        if !created {
            return None;
        }
        // Self-destruct the view once its client detaches. We use a
        // client-detached hook rather than `destroy-unattached on`: the latter
        // reaps the session the instant it exists (it is created DETACHED, i.e.
        // zero clients), so the subsequent select-window / attach would race and
        // fail with "can't find session". The hook only fires on a real detach,
        // so the view survives until we attach and is cleaned up afterward.
        let _ = std::process::Command::new("tmux")
            .args([
                "set-hook",
                "-t",
                &view,
                "client-detached",
                &format!("kill-session -t {view}"),
            ])
            .status();
    }

    // Select the target window within THIS view (independent of other clients).
    let _ = std::process::Command::new("tmux")
        .args(["select-window", "-t", &format!("{view}:{window}")])
        .status();

    Some(view)
}

fn attach_pane(pane_id: &str) -> Result<()> {
    // Resolve the pane to its owning session + window so we can build a
    // per-client grouped view (see `ensure_view_session`). Falls back to the
    // raw pane id if resolution fails (e.g. tmux listing changed underneath us).
    let location = resolve_pane_location(pane_id);

    let in_tmux = std::env::var("TMUX").map(|v| !v.is_empty()).unwrap_or(false);
    if in_tmux {
        // Point THIS client at its own grouped view of the target window, so it
        // doesn't mirror (or get mirrored by) other clients viewing the same
        // base session.
        let target = match &location {
            Some((base, window)) => match ensure_view_session(base, window) {
                Some(view) => view,
                None => pane_id.to_string(),
            },
            None => pane_id.to_string(),
        };
        let status = std::process::Command::new("tmux")
            .args(["switch-client", "-t", &target])
            .status()
            .context("tmux switch-client")?;
        if status.success() {
            return Ok(());
        }
        eprintln!("tmux switch-client failed for target {target}");
        return Ok(());
    }

    // Not inside tmux: attach to a per-client grouped view session and exec so
    // this process is replaced by the tmux client.
    let target = match &location {
        Some((base, window)) => ensure_view_session(base, window)
            .unwrap_or_else(|| format!("{base}:{window}")),
        None => {
            eprintln!(
                "Pane {pane_id} not found in any tmux session. Run: tmux attach-session -t tenex"
            );
            return Ok(());
        }
    };

    use std::os::unix::process::CommandExt;
    let err = std::process::Command::new("tmux")
        .args(["attach-session", "-t", &target])
        .exec(); // replaces this process; only returns on error
    anyhow::bail!("exec tmux attach-session: {err}");
}
