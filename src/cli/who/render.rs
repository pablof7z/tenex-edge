use super::*;

pub(super) fn render_who_once(snapshot: &WhoSnapshot) -> String {
    let mut out = String::new();

    let scope = if snapshot.project == "*" {
        "all projects".to_string()
    } else {
        snapshot.project.clone()
    };
    let _ = writeln!(out, "{}", scope.bold());
    let _ = writeln!(out);

    if snapshot.rows.is_empty() {
        let _ = writeln!(
            out,
            "(no live agents{})",
            if snapshot.all {
                ""
            } else {
                " — start a session, or run with --all to include stale"
            }
        );
    } else if snapshot.project == "*" {
        for row in &snapshot.rows {
            render_who_row(&mut out, row, true);
        }
    } else {
        for row in &snapshot.rows {
            render_who_row(&mut out, row, false);
        }
    }

    if snapshot.project != "*" && !snapshot.other_projects.is_empty() {
        let _ = writeln!(out);
        let _ = writeln!(
            out,
            "{} other agent(s) in other projects:",
            snapshot.other_projects.len()
        );
        for op in &snapshot.other_projects {
            match &op.about {
                Some(about) if !about.is_empty() => {
                    let _ = writeln!(out, "  * {} - {}", op.project, about);
                }
                _ => {
                    let _ = writeln!(out, "  * {}", op.project);
                }
            }
        }
    }

    if !snapshot.spawnable.is_empty() {
        let _ = writeln!(out);
        for row in &snapshot.spawnable {
            let label = format!("{}@{}", row.slug, row.host);
            let tag = format!("[spawnable via {}]", row.command);
            let _ = writeln!(out, "{}  {}", label.dimmed(), tag.dimmed());
        }
    }

    out
}

pub(super) fn render_who_for_stdout(snapshot: &WhoSnapshot) -> String {
    if io::stdout().is_terminal() {
        render_who_once(snapshot)
    } else {
        render_who_plain(snapshot)
    }
}

pub(super) fn render_who_plain(snapshot: &WhoSnapshot) -> String {
    let mut out = String::new();

    let scope = if snapshot.project == "*" {
        "all projects".to_string()
    } else {
        snapshot.project.clone()
    };
    let _ = writeln!(out, "# tenex-edge who");
    let _ = writeln!(out);
    let _ = writeln!(out, "Project: {scope}");
    let _ = writeln!(out);

    let _ = writeln!(out, "## Sessions");
    let _ = writeln!(
        out,
        "Message active sessions with `tenex-edge inbox send --to <agent@project|session-id> --subject \"...\" --message \"...\"`."
    );
    let _ = writeln!(out);
    if snapshot.rows.is_empty() {
        let _ = writeln!(
            out,
            "{}",
            if snapshot.all {
                "_No sessions visible._"
            } else {
                "_No live sessions visible. Run `tenex-edge who --all` to include stale sessions._"
            }
        );
    } else {
        let _ = writeln!(out, "| Agent | Session | Host | Title | Status |");
        let _ = writeln!(out, "|---|---:|---|---|---|");
        for row in &snapshot.rows {
            render_who_markdown_row(&mut out, row, snapshot.project == "*");
        }
    }

    let _ = writeln!(out);
    let _ = writeln!(out, "## Agents (for new sessions)");
    let _ = writeln!(
        out,
        "Start a new session with `tenex-edge inbox new-session --agent <slug>`."
    );
    let _ = writeln!(out);
    if snapshot.spawnable.is_empty() {
        let _ = writeln!(out, "_No local spawnable agents configured._");
    } else {
        let _ = writeln!(out, "| Agent | Host | Command |");
        let _ = writeln!(out, "|---|---|---|");
        for row in &snapshot.spawnable {
            let _ = writeln!(
                out,
                "| {} | {} | `{}` |",
                md_cell(&row.slug),
                md_cell(&row.host),
                md_cell(&row.command)
            );
        }
    }

    if snapshot.project != "*" {
        let _ = writeln!(out);
        let _ = writeln!(out, "## Other projects");
        if snapshot.other_projects.is_empty() {
            let _ = writeln!(out);
            let _ = writeln!(out, "_No other projects visible._");
        } else {
            let _ = writeln!(out);
            for op in &snapshot.other_projects {
                let _ = writeln!(out, "- {}", md_text(&op.project));
            }
        }
    }

    out
}

fn render_who_markdown_row(out: &mut String, row: &WhoRow, include_project: bool) {
    let agent = if include_project {
        format!("{}@{}", row.slug, row.project)
    } else {
        row.slug.clone()
    };
    let host = if row.remote {
        format!("{}, remote", row.host)
    } else {
        row.host.clone()
    };
    let host = rel_cwd_bracket(&row.rel_cwd)
        .map(|dir| format!("{host} [{dir}]"))
        .unwrap_or(host);
    let title = if row.status.trim().is_empty() {
        "—".to_string()
    } else {
        row.status.trim().to_string()
    };
    let mut status = if row.active {
        let activity = row.activity.trim();
        if activity.is_empty() {
            "working".to_string()
        } else {
            activity.to_string()
        }
    } else {
        "idle".to_string()
    };
    if !row.fresh {
        status.push_str(" (stale)");
    }
    if row.unread > 0 {
        let _ = write!(status, ", {} unread", row.unread);
    }

    let _ = writeln!(
        out,
        "| {} | `{}` | {} | {} | {} |",
        md_cell(&agent),
        session_short_code(&row.session_id),
        md_cell(&host),
        md_cell(&title),
        md_cell(&status)
    );
}

fn md_cell(input: &str) -> String {
    md_text(input).replace('|', r"\|")
}

fn md_text(input: &str) -> String {
    input
        .replace('\n', " ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn render_who_row(out: &mut String, row: &WhoRow, include_project: bool) {
    let stale = if row.fresh {
        String::new()
    } else {
        format!(" {}", "(stale)".dimmed())
    };
    // Always show which host the agent runs on. Same-machine agents get a plain
    // `(hostname)`; a true remote (peer on a different host than the daemon) is
    // flagged `(hostname, remote)` so cross-machine sessions stay distinguishable.
    let host = if row.remote {
        format!(" {}", format!("({}, remote)", row.host).dimmed())
    } else {
        format!(" {}", format!("({})", row.host).dimmed())
    };
    let dir = rel_cwd_bracket(&row.rel_cwd)
        .map(|d| format!(" {}", format!("[{d}]").dimmed()))
        .unwrap_or_default();
    let unread = if row.unread > 0 {
        format!(" {}", format!("◉{}", row.unread).yellow())
    } else {
        String::new()
    };
    let name = if include_project {
        format!("{}@{}", row.slug, row.project).cyan().to_string()
    } else {
        row.slug.cyan().to_string()
    };
    let _ = writeln!(
        out,
        "{} [session {}]{}{}{}{} - {}",
        name,
        session_short_code(&row.session_id).yellow(),
        dir,
        host,
        stale,
        unread,
        status_colored(&row.status, &row.activity, row.active),
    );
}

/// The `[dir]` to show for a row's `rel_cwd`: `None` when empty or the project
/// root (`.`), so the project root renders without a bracket (§8e).
fn rel_cwd_bracket(rel_cwd: &str) -> Option<&str> {
    let r = rel_cwd.trim();
    if r.is_empty() || r == "." {
        None
    } else {
        Some(r)
    }
}

pub(super) fn draw_who_live(snapshot: &WhoSnapshot, refresh: Duration) -> Result<()> {
    let refresh_ms = refresh.as_millis();
    let mut screen = render_who_once(snapshot);
    let _ = writeln!(
        screen,
        "{}",
        format!("  --live  refresh {refresh_ms}ms  q/esc/ctrl-c to quit").dimmed()
    );
    let mut stdout = io::stdout();
    execute!(stdout, MoveTo(0, 0), Clear(ClearType::All))?;
    for line in screen.lines() {
        write!(stdout, "{line}\r\n")?;
    }
    stdout.flush()?;
    Ok(())
}

/// Plain (no-ANSI) status label: the persistent title, the live activity while
/// mid-turn (`title — activity`), and an idle marker when not mid-turn. Used for
/// injected context blocks where ANSI must not leak. Empty title falls back to
/// the activity, then to a bare "working"/"idle" word.
pub(super) fn status_plain(title: &str, activity: &str, active: bool) -> String {
    let t = title.trim();
    let a = activity.trim();
    match (t.is_empty(), active) {
        (true, true) if !a.is_empty() => a.to_string(),
        (true, true) => "working".to_string(),
        (true, false) => "idle".to_string(),
        (false, true) if !a.is_empty() => format!("{t} — {a}"),
        (false, true) => t.to_string(),
        (false, false) => format!("{t} · idle"),
    }
}

/// Terminal status label: like [`status_plain`] but dims the live activity and
/// the idle marker so the persistent title stays prominent.
fn status_colored(title: &str, activity: &str, active: bool) -> String {
    let t = title.trim();
    let a = activity.trim();
    match (t.is_empty(), active) {
        (true, true) if !a.is_empty() => a.dimmed().to_string(),
        (true, true) => "working".dimmed().to_string(),
        (true, false) => "idle".dimmed().to_string(),
        (false, true) if !a.is_empty() => format!("{} {}", t, format!("— {a}").dimmed()),
        (false, true) => t.to_string(),
        (false, false) => format!("{} {}", t, "· idle".dimmed()),
    }
}

pub(super) fn should_quit_live(event: TermEvent) -> bool {
    let TermEvent::Key(key) = event else {
        return false;
    };
    matches!(key.code, KeyCode::Esc | KeyCode::Char('q'))
        || (matches!(key.code, KeyCode::Char('c')) && key.modifiers.contains(KeyModifiers::CONTROL))
}

pub(super) struct LiveTerminal;

impl LiveTerminal {
    pub(super) fn enter() -> Result<Self> {
        terminal::enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, Hide)?;
        Ok(Self)
    }
}

impl Drop for LiveTerminal {
    fn drop(&mut self) {
        let _ = terminal::disable_raw_mode();
        let _ = execute!(io::stdout(), Show, LeaveAlternateScreen);
    }
}
