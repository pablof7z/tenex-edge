//! The host-neutral CLI surface (M1 §6).

use crate::codec::{Codec, Kind1Codec, SubScope};
use crate::config::{self, Config};
use crate::domain::{AgentRef, DomainEvent, Mention};
use crate::identity;
use crate::project;
use crate::runtime::{self, EngineParams};
use crate::state::Store;
use crate::transport::Transport;
use crate::util::{now_secs, short_id, slugify_host};
use anyhow::{bail, Context, Result};
use clap::{Parser, Subcommand};
use crossterm::{
    cursor::{Hide, MoveTo, Show},
    event::{self, Event as TermEvent, KeyCode, KeyModifiers},
    execute,
    terminal::{self, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen},
};
use nostr_sdk::prelude::RelayPoolNotification;
use owo_colors::OwoColorize;
use std::fmt::Write as _;
use std::io::{self, Write as _};
use std::path::PathBuf;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

#[derive(Parser)]
#[command(
    name = "tenex-edge",
    about = "Citizenship for your agents: identity + awareness on the Nostr fabric."
)]
pub struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Start a session: publish identity, begin presence/awareness in the background.
    SessionStart {
        #[arg(long)]
        agent: String,
        /// Adopt the host's session id (e.g. Claude Code's). Generated if absent.
        #[arg(long)]
        session_id: Option<String>,
        /// Working directory to resolve the project from (default: cwd).
        #[arg(long)]
        cwd: Option<PathBuf>,
        /// Host PID to watch; the background process stops if it dies.
        #[arg(long)]
        watch_pid: Option<i32>,
    },
    /// End a session cleanly (stops the background process, goes idle).
    SessionEnd {
        #[arg(long)]
        session: String,
    },
    /// Mention another agent or a specific session.
    SendMessage {
        /// session-id (or prefix), agent slug, slug@project, or hex pubkey.
        recipient: String,
        message: String,
        /// My session id; if omitted, resolved from the current directory.
        #[arg(long)]
        session: Option<String>,
    },
    /// List peers currently visible (with session-id prefixes for targeting).
    Who {
        #[arg(long)]
        project: Option<String>,
        /// Include peers whose heartbeat has stopped (stale).
        #[arg(long)]
        all: bool,
        /// Keep a full-screen live view open, refreshing automatically.
        #[arg(long)]
        live: bool,
        /// Refresh interval for --live, in milliseconds.
        #[arg(long, default_value = "1000")]
        refresh_ms: u64,
    },
    /// Manage which agents this computer authorizes (owner-scoped allow/block).
    Acl {
        #[command(subcommand)]
        action: Option<AclAction>,
    },
    /// Stream all fabric activity, colorized.
    Tail {
        #[arg(long)]
        project: Option<String>,
    },
    /// Print + drain pending mentions for a session (used by the injection hook).
    Inbox {
        /// Session id; if omitted, resolved from the current directory.
        #[arg(long)]
        session: Option<String>,
    },
    /// Block until a mention arrives for this session, then print it and exit.
    /// Run with run_in_background=true; the agent is woken when this exits.
    WaitForMention {
        /// Session id; if omitted, resolved from the current directory.
        #[arg(long)]
        session: Option<String>,
        /// Exit after this many seconds even if no mention arrives (0 = infinite).
        #[arg(long, default_value = "300")]
        timeout: u64,
    },
    /// Mark a session as working on a turn (used by the turn-start hook, e.g.
    /// Claude Code's UserPromptSubmit). Outputs fabric context for the agent:
    /// inbox messages, and presence/status changes since the last turn.
    TurnStart {
        #[arg(long)]
        session: String,
        /// Path to the host conversation transcript (JSONL) for this session.
        #[arg(long)]
        transcript: Option<String>,
        /// Emit JSON {"systemMessage": "..."} instead of plain text (for Codex).
        #[arg(long)]
        json: bool,
    },
    /// Check for new inbox messages mid-turn (used by PostToolUse hook).
    /// Read-only: does not drain the inbox; turn-start at the next prompt drains.
    TurnCheck {
        /// Session id; if omitted, resolved from the current directory.
        #[arg(long)]
        session: Option<String>,
        /// Emit JSON {"systemMessage": "..."} instead of plain text (for Codex).
        #[arg(long)]
        json: bool,
    },
    /// Mark a session idle (used by the turn-end hook, e.g. Claude Code's Stop).
    /// The engine clears the agent's status on its next poll.
    TurnEnd {
        #[arg(long)]
        session: String,
    },
    /// Connectivity check: publish a test note to the configured relays and read it back.
    Doctor,
    /// Internal: the detached per-session engine. Not for direct use.
    #[command(name = "__run-session", hide = true)]
    RunSession {
        #[arg(long)]
        agent: String,
        #[arg(long)]
        session_id: String,
        #[arg(long)]
        project: String,
        #[arg(long)]
        watch_pid: Option<i32>,
    },
}

#[derive(Subcommand)]
enum AclAction {
    /// List pending (unauthorized) + authorized + blocked agents.
    List,
    /// Authorize an agent (pubkey or pending-list slug) to be seen/trusted.
    Allow { target: String },
    /// Block an agent (pubkey or pending-list slug).
    Block { target: String },
}

pub async fn run(cli: Cli) -> Result<()> {
    match cli.cmd {
        Cmd::SessionStart {
            agent,
            session_id,
            cwd,
            watch_pid,
        } => session_start(agent, session_id, cwd, watch_pid),
        Cmd::SessionEnd { session } => session_end(session),
        Cmd::SendMessage {
            recipient,
            message,
            session,
        } => send_message(recipient, message, session).await,
        Cmd::Who {
            project,
            all,
            live,
            refresh_ms,
        } => {
            if live {
                who_live(project, all, Duration::from_millis(refresh_ms.max(100)))
            } else {
                who(project, all)
            }
        }
        Cmd::Acl { action } => acl(action).await,
        Cmd::Tail { project } => tail(project).await,
        Cmd::Inbox { session } => inbox(session).await,
        Cmd::WaitForMention { session, timeout } => wait_for_mention(session, timeout).await,
        Cmd::Doctor => doctor().await,
        Cmd::TurnStart {
            session,
            transcript,
            json,
        } => turn_start(session, transcript, json).await,
        Cmd::TurnCheck { session, json } => turn_check(session, json),
        Cmd::TurnEnd { session } => turn_end(session),
        Cmd::RunSession {
            agent,
            session_id,
            project,
            watch_pid,
        } => run_session(agent, session_id, project, watch_pid).await,
    }
}

/// A peer is "live" only while heartbeats keep it fresh (3× the default 30s tick).
const PEER_FRESH_SECS: u64 = 90;

fn store_path() -> PathBuf {
    config::edge_home().join("state.db")
}

fn open_store() -> Result<Store> {
    Store::open(&store_path())
}

/// Resolve the caller's session: explicit id if given, else the most-recent
/// alive session for the project of the current directory. Lets agents that
/// don't know their session id just run `tenex-edge inbox` / `send-message`.
fn resolve_session(store: &Store, explicit: Option<String>) -> Result<crate::state::SessionRecord> {
    if let Some(id) = explicit {
        return store
            .get_session(&id)?
            .with_context(|| format!("unknown session {id}"));
    }
    // Host adapters can export this so an agent resolves ITS OWN session even
    // when several agents share a project.
    if let Ok(id) = std::env::var("TENEX_EDGE_SESSION") {
        if !id.is_empty() {
            if let Some(rec) = store.get_session(&id)? {
                return Ok(rec);
            }
        }
    }
    let project = project::resolve(&std::env::current_dir()?);
    store
        .latest_alive_session_for_project(&project)?
        .with_context(|| format!("no active tenex-edge session for project {project:?} (run session-start, or pass --session)"))
}

fn gen_session_id() -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    format!("te-{:x}-{}", nanos, std::process::id())
}

// ── session-start ────────────────────────────────────────────────────────────

fn session_start(
    agent: String,
    session_id: Option<String>,
    cwd: Option<PathBuf>,
    watch_pid: Option<i32>,
) -> Result<()> {
    let cfg = Config::load().context("loading ~/.tenex/config.json")?;
    let edge = config::edge_home();
    config::ensure_dir(&edge)?;
    let id = identity::load_or_create(&edge, &agent, now_secs())?;
    // Our own fleet is auto-authorized: ensure this agent is on the allowlist.
    let _ = crate::acl::allow(&id.pubkey_hex(), &agent);
    let cwd = cwd.unwrap_or(std::env::current_dir()?);
    let project = project::resolve(&cwd);
    let session_id = session_id.unwrap_or_else(gen_session_id);

    let store = open_store()?;
    store.upsert_session(&crate::state::SessionRecord {
        session_id: session_id.clone(),
        agent_slug: agent.clone(),
        agent_pubkey: id.pubkey_hex(),
        project: project.clone(),
        host: cfg.host.clone(),
        child_pid: None,
        watch_pid,
        created_at: now_secs(),
        alive: true,
    })?;

    // Fork the detached engine: re-exec ourselves as `__run-session`.
    let exe = std::env::current_exe().context("locating own executable")?;
    let mut command = std::process::Command::new(exe);
    command
        .arg("__run-session")
        .arg("--agent")
        .arg(&agent)
        .arg("--session-id")
        .arg(&session_id)
        .arg("--project")
        .arg(&project)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null());
    if let Some(pid) = watch_pid {
        command.arg("--watch-pid").arg(pid.to_string());
    }
    detach(&mut command);
    let child = command.spawn().context("forking background engine")?;

    // Record the engine pid so session-end can stop it; mark live immediately.
    let mut rec = store
        .get_session(&session_id)?
        .expect("just-written session");
    rec.child_pid = Some(child.id() as i32);
    store.upsert_session(&rec)?;
    store.touch_session(&session_id, now_secs())?;

    // The session id is the only thing the host needs back.
    println!("{session_id}");
    Ok(())
}

#[cfg(unix)]
fn detach(command: &mut std::process::Command) {
    use std::os::unix::process::CommandExt;
    command.process_group(0); // own process group: survives terminal Ctrl-C / parent exit
}
#[cfg(not(unix))]
fn detach(_command: &mut std::process::Command) {}

// ── __run-session (the engine) ───────────────────────────────────────────────

async fn run_session(
    agent: String,
    session_id: String,
    project: String,
    watch_pid: Option<i32>,
) -> Result<()> {
    let cfg = Config::load()?;
    let edge = config::edge_home();
    let id = identity::load_or_create(&edge, &agent, now_secs())?;
    let _ = crate::acl::allow(&id.pubkey_hex(), &agent);

    let heartbeat = env_duration("TENEX_EDGE_HEARTBEAT_MS", Duration::from_secs(30));
    let obs_interval = env_duration("TENEX_EDGE_OBS_MS", Duration::from_secs(5));
    let status_ttl = Duration::from_secs(env_u64("TENEX_EDGE_STATUS_TTL_S", 90));
    // Turn-driven distillation cadence: first summary 30s into a turn (so quick
    // turns cost nothing), then refresh every 5m while it keeps running.
    let turn_first = Duration::from_secs(env_u64("TENEX_EDGE_TURN_FIRST_S", 30));
    let turn_repeat = Duration::from_secs(env_u64("TENEX_EDGE_TURN_REPEAT_S", 300));

    let params = EngineParams {
        agent_slug: agent,
        agent_pubkey: id.pubkey_hex(),
        keys: id.keys.clone(),
        project,
        session_id,
        host: cfg.host,
        owners: cfg.whitelisted_pubkeys,
        relays: cfg.relays,
        watch_pid,
        store_path: store_path(),
        heartbeat,
        obs_interval,
        status_ttl,
        turn_first,
        turn_repeat,
    };
    runtime::run_session(params).await
}

// ── session-end ──────────────────────────────────────────────────────────────

fn session_end(session: String) -> Result<()> {
    let store = open_store()?;
    if let Some(rec) = store.get_session(&session)? {
        if let Some(pid) = rec.child_pid {
            // SIGTERM -> engine publishes idle status and exits cleanly.
            let _ = nix::sys::signal::kill(
                nix::unistd::Pid::from_raw(pid),
                nix::sys::signal::Signal::SIGTERM,
            );
        }
        store.mark_session_dead(&session)?;
        eprintln!("session {session} ended");
    } else {
        eprintln!("no such session: {session}");
    }
    Ok(())
}

// ── send-message ─────────────────────────────────────────────────────────────

async fn send_message(recipient: String, message: String, session: Option<String>) -> Result<()> {
    let cfg = Config::load()?;
    let store = open_store()?;
    let rec = resolve_session(&store, session)?;
    let edge = config::edge_home();
    let id = identity::load_or_create(&edge, &rec.agent_slug, now_secs())?;

    let (to_pubkey, target_session) = resolve_recipient(&store, &rec.project, &recipient)?;

    let mention = DomainEvent::Mention(Mention {
        from: AgentRef::new(id.pubkey_hex(), rec.agent_slug.clone()),
        to_pubkey: to_pubkey.clone(),
        project: rec.project.clone(),
        body: message,
        target_session: target_session.clone(),
    });

    let transport = Transport::connect(&cfg.relays, id.keys.clone()).await?;
    let codec = Kind1Codec;
    transport.publish_builder(codec.encode(&mention)?).await?;
    transport.shutdown().await;

    match target_session {
        Some(s) => println!(
            "mentioned {} (session {})",
            short_id(&to_pubkey),
            short_id(&s)
        ),
        None => println!("mentioned {}", short_id(&to_pubkey)),
    }
    Ok(())
}

fn resolve_recipient(
    store: &Store,
    my_project: &str,
    target: &str,
) -> Result<(String, Option<String>)> {
    // 1. slug@project
    if let Some((slug, proj)) = target.split_once('@') {
        let pk = store
            .resolve_agent_pubkey(slug, Some(proj))?
            .with_context(|| {
                format!("can't resolve {slug}@{proj} (no presence/profile seen yet)")
            })?;
        return Ok((pk, None));
    }
    // 2. raw hex pubkey
    if target.len() == 64 && target.chars().all(|c| c.is_ascii_hexdigit()) {
        return Ok((target.to_string(), None));
    }
    // 3. session-id prefix (target a specific session of an agent) — check
    //    foreign peers first, then my own sessions on this machine.
    if target.len() >= 6 {
        if let Some(ps) = store.find_peer_session_by_prefix(target)? {
            return Ok((ps.pubkey, Some(ps.session_id)));
        }
        if let Some(s) = store.find_session_by_prefix(target)? {
            return Ok((s.agent_pubkey, Some(s.session_id)));
        }
    }
    // 4. bare agent slug in my project
    if let Some(pk) = store.resolve_agent_pubkey(target, Some(my_project))? {
        return Ok((pk, None));
    }
    bail!("can't resolve recipient {target:?} (try `tenex-edge who`)")
}

// ── who ──────────────────────────────────────────────────────────────────────

fn who(project: Option<String>, all: bool) -> Result<()> {
    let store = open_store()?;
    let snapshot = load_who_snapshot(&store, project.as_deref(), all, now_secs())?;
    print!("{}", render_who_once(&snapshot));
    Ok(())
}

fn who_live(project: Option<String>, all: bool, refresh: Duration) -> Result<()> {
    let refresh = refresh.max(Duration::from_millis(100));
    let store = open_store()?;
    let _terminal = LiveTerminal::enter()?;
    let mut next_draw = Instant::now();

    loop {
        let now = Instant::now();
        if now >= next_draw {
            let snapshot = load_who_snapshot(&store, project.as_deref(), all, now_secs())?;
            draw_who_live(&snapshot, refresh)?;
            next_draw = Instant::now() + refresh;
        }

        let wait = next_draw
            .saturating_duration_since(Instant::now())
            .min(Duration::from_millis(100));
        if event::poll(wait)? {
            if should_quit_live(event::read()?) {
                break;
            }
        }
    }

    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct WhoSnapshot {
    project: Option<String>,
    all: bool,
    now: u64,
    rows: Vec<WhoRow>,
}

impl WhoSnapshot {
    fn live_count(&self) -> usize {
        self.rows.iter().filter(|r| r.fresh).count()
    }

    fn stale_count(&self) -> usize {
        self.rows.len().saturating_sub(self.live_count())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct WhoRow {
    source: WhoSource,
    fresh: bool,
    slug: String,
    project: String,
    status: String,
    host: String,
    session_id: String,
    age_secs: Option<u64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum WhoSource {
    Local,
    Peer,
}

fn load_who_snapshot(
    store: &Store,
    project: Option<&str>,
    all: bool,
    now: u64,
) -> Result<WhoSnapshot> {
    let since = if all {
        0
    } else {
        now.saturating_sub(PEER_FRESH_SECS)
    };

    let mut mine = store.list_my_live_sessions(since)?;
    if let Some(project) = project {
        mine.retain(|s| s.project == project);
    }
    let my_ids: std::collections::HashSet<String> =
        mine.iter().map(|s| s.session_id.clone()).collect();

    let peers = store
        .list_peer_sessions(project, since)?
        .into_iter()
        .filter(|p| !my_ids.contains(&p.session_id));

    let mut rows = Vec::new();
    for s in mine {
        let age_secs = store
            .session_last_seen(&s.session_id)
            .ok()
            .flatten()
            .map(|last_seen| now.saturating_sub(last_seen));
        rows.push(WhoRow {
            source: WhoSource::Local,
            fresh: age_secs.map(|age| age <= PEER_FRESH_SECS).unwrap_or(true),
            slug: s.agent_slug,
            project: s.project.clone(),
            status: status_for(store, &s.agent_pubkey, &s.project),
            host: s.host,
            session_id: s.session_id,
            age_secs,
        });
    }
    for p in peers {
        let age = now.saturating_sub(p.last_seen);
        rows.push(WhoRow {
            source: WhoSource::Peer,
            fresh: age <= PEER_FRESH_SECS,
            slug: p.slug,
            project: p.project.clone(),
            status: status_for(store, &p.pubkey, &p.project),
            host: p.host,
            session_id: p.session_id,
            age_secs: Some(age),
        });
    }

    Ok(WhoSnapshot {
        project: project.map(str::to_string),
        all,
        now,
        rows,
    })
}

fn status_for(store: &Store, pubkey: &str, project: &str) -> String {
    store
        .get_agent_status(pubkey, project)
        .ok()
        .flatten()
        .unwrap_or_default()
}

fn render_who_once(snapshot: &WhoSnapshot) -> String {
    if snapshot.rows.is_empty() {
        return format!(
            "(no live agents{})\n",
            if snapshot.all {
                ""
            } else {
                " — start a session, or run with --all to include stale"
            }
        );
    }

    let mut out = String::new();
    let _ = writeln!(out, "{}", "agents:".bold());
    for row in &snapshot.rows {
        let dot = if row.fresh {
            "●".green().to_string()
        } else {
            "○".dimmed().to_string()
        };
        let status = render_status_colored(&row.status);
        match row.source {
            WhoSource::Local => {
                let _ = writeln!(
                    out,
                    "  {dot} {}@{}{}  {}  session {}  {}",
                    row.slug.cyan(),
                    slugify_host(&row.host),
                    status,
                    row.project.dimmed(),
                    short_id(&row.session_id).yellow(),
                    "(this machine)".dimmed()
                );
            }
            WhoSource::Peer => {
                let age = row.age_secs.unwrap_or(0);
                let _ = writeln!(
                    out,
                    "  {dot} {}@{}{}  {}  session {}  ({}s ago)",
                    row.slug.cyan(),
                    slugify_host(&row.host),
                    status,
                    row.project.dimmed(),
                    short_id(&row.session_id).yellow(),
                    age
                );
            }
        }
    }
    out
}

fn render_status_colored(status: &str) -> String {
    if status.trim().is_empty() {
        format!(" — {}", "idle".dimmed())
    } else {
        format!(" — {status}")
    }
}

fn draw_who_live(snapshot: &WhoSnapshot, refresh: Duration) -> Result<()> {
    let (width, height) = terminal::size().unwrap_or((100, 30));
    let screen = render_who_live(snapshot, width as usize, height as usize, refresh);
    let mut stdout = io::stdout();
    execute!(stdout, MoveTo(0, 0), Clear(ClearType::All))?;
    write!(stdout, "{screen}")?;
    stdout.flush()?;
    Ok(())
}

fn render_who_live(
    snapshot: &WhoSnapshot,
    width: usize,
    height: usize,
    refresh: Duration,
) -> String {
    let width = width.max(40);
    let height = height.max(8);
    let agent_w = if width < 80 { 20 } else { 28 };
    let project_w = if width < 80 { 10 } else { 14 };
    let session_w = 10;
    let seen_w = if width < 80 { 8 } else { 10 };
    let fixed = 2 + agent_w + 2 + project_w + 2 + session_w + 2 + seen_w + 2;
    let status_w = width.saturating_sub(fixed).max(12);
    let max_rows = height.saturating_sub(7).max(1);

    let mut out = String::new();
    let scope = snapshot.project.as_deref().unwrap_or("*");
    let mode = if snapshot.all { "all" } else { "fresh" };
    let refresh_ms = refresh.as_millis();
    let _ = writeln!(out, "tenex-edge who --live");
    let _ = writeln!(
        out,
        "{}",
        fit_plain(
            &format!(
                "project: {scope}   mode: {mode}   refresh: {refresh_ms}ms   q/esc/ctrl-c quits"
            ),
            width
        )
    );
    let _ = writeln!(
        out,
        "{}",
        fit_plain(
            &format!(
                "{} agent(s): {} live, {} stale",
                snapshot.rows.len(),
                snapshot.live_count(),
                snapshot.stale_count()
            ),
            width
        )
    );
    let _ = writeln!(out);

    if snapshot.rows.is_empty() {
        let message = if snapshot.all {
            "(no agents)"
        } else {
            "(no live agents — start a session, or run with --all to include stale)"
        };
        let _ = writeln!(out, "{}", fit_plain(message, width));
        return out;
    }

    let _ = writeln!(
        out,
        "{}",
        fit_plain(
            &format!(
                "  {}  {}  {}  {}  {}",
                pad_fit("AGENT@HOST", agent_w),
                pad_fit("PROJECT", project_w),
                pad_fit("STATUS", status_w),
                pad_fit("SESSION", session_w),
                pad_fit("SEEN", seen_w)
            ),
            width
        )
    );

    for row in snapshot.rows.iter().take(max_rows) {
        let dot = if row.fresh { "●" } else { "○" };
        let agent_at_host = format!("{}@{}", row.slug, slugify_host(&row.host));
        let _ = writeln!(
            out,
            "{}",
            fit_plain(
                &format!(
                    "{dot} {}  {}  {}  {}  {}",
                    pad_fit(&agent_at_host, agent_w),
                    pad_fit(&row.project, project_w),
                    pad_fit(&status_plain(&row.status), status_w),
                    pad_fit(&short_id(&row.session_id), session_w),
                    pad_fit(&seen_label(row), seen_w),
                ),
                width
            )
        );
    }

    let hidden = snapshot.rows.len().saturating_sub(max_rows);
    if hidden > 0 {
        let _ = writeln!(out, "{}", fit_plain(&format!("... {hidden} more"), width));
    }
    out
}

fn status_plain(status: &str) -> String {
    if status.trim().is_empty() {
        "idle".to_string()
    } else {
        status.trim().to_string()
    }
}

fn seen_label(row: &WhoRow) -> String {
    match row.source {
        WhoSource::Local => row
            .age_secs
            .map(|age| format!("local {age}s"))
            .unwrap_or_else(|| "local".to_string()),
        WhoSource::Peer => row
            .age_secs
            .map(|age| format!("{age}s ago"))
            .unwrap_or_else(|| "?".to_string()),
    }
}

fn pad_fit(value: &str, width: usize) -> String {
    let fitted = fit_plain(value, width);
    format!("{fitted:<width$}")
}

fn fit_plain(value: &str, width: usize) -> String {
    if width == 0 {
        return String::new();
    }
    let char_count = value.chars().count();
    if char_count <= width {
        return value.to_string();
    }
    if width <= 3 {
        return ".".repeat(width);
    }
    let mut out = value.chars().take(width - 3).collect::<String>();
    out.push_str("...");
    out
}

fn should_quit_live(event: TermEvent) -> bool {
    let TermEvent::Key(key) = event else {
        return false;
    };
    matches!(key.code, KeyCode::Esc | KeyCode::Char('q'))
        || (matches!(key.code, KeyCode::Char('c')) && key.modifiers.contains(KeyModifiers::CONTROL))
}

struct LiveTerminal;

impl LiveTerminal {
    fn enter() -> Result<Self> {
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

#[cfg(test)]
mod who_tests {
    use super::*;

    fn local_session(id: &str) -> crate::state::SessionRecord {
        crate::state::SessionRecord {
            session_id: id.to_string(),
            agent_slug: "coder".to_string(),
            agent_pubkey: "pk-coder".to_string(),
            project: "proj".to_string(),
            host: "laptop".to_string(),
            child_pid: Some(42),
            watch_pid: None,
            created_at: 1,
            alive: true,
        }
    }

    #[test]
    fn who_snapshot_merges_local_and_peer_sessions() {
        let store = Store::open_memory().unwrap();
        store
            .upsert_session(&local_session("local-session"))
            .unwrap();
        store.touch_session("local-session", 1_000).unwrap();
        store
            .upsert_peer_session(
                "local-session",
                "pk-coder",
                "coder",
                "proj",
                "laptop",
                1_000,
            )
            .unwrap();
        store
            .upsert_peer_session(
                "remote-session",
                "pk-reviewer",
                "reviewer",
                "proj",
                "tower",
                995,
            )
            .unwrap();
        store
            .set_agent_status("pk-reviewer", "proj", "reviewing the patch", 995)
            .unwrap();

        let snapshot = load_who_snapshot(&store, Some("proj"), false, 1_000).unwrap();

        assert_eq!(snapshot.rows.len(), 2);
        assert!(snapshot
            .rows
            .iter()
            .any(|r| r.source == WhoSource::Local && r.slug == "coder"));
        assert!(snapshot
            .rows
            .iter()
            .any(|r| r.source == WhoSource::Peer && r.slug == "reviewer"));
        assert!(!snapshot
            .rows
            .iter()
            .any(|r| r.source == WhoSource::Peer && r.session_id == "local-session"));

        let once = render_who_once(&snapshot);
        assert!(once.contains("@laptop"));
        assert!(once.contains("proj"));
    }

    #[test]
    fn live_renderer_includes_status_and_controls() {
        let snapshot = WhoSnapshot {
            project: Some("proj".to_string()),
            all: false,
            now: 1_000,
            rows: vec![WhoRow {
                source: WhoSource::Peer,
                fresh: true,
                slug: "reviewer".to_string(),
                project: "proj".to_string(),
                status: "reviewing the patch".to_string(),
                host: "tower".to_string(),
                session_id: "remote-session".to_string(),
                age_secs: Some(5),
            }],
        };

        let rendered = render_who_live(&snapshot, 100, 20, Duration::from_millis(1000));

        assert!(rendered.contains("tenex-edge who --live"));
        assert!(rendered.contains("q/esc/ctrl-c quits"));
        assert!(rendered.contains("reviewer"));
        assert!(rendered.contains("reviewing the patch"));
    }

    #[test]
    fn fit_plain_truncates_to_width() {
        assert_eq!(fit_plain("abcdef", 4), "a...");
        assert_eq!(fit_plain("abcdef", 3), "...");
        assert_eq!(fit_plain("abc", 4), "abc");
    }
}

// ── acl (owner-scoped agent authorization) ───────────────────────────────────

async fn acl(action: Option<AclAction>) -> Result<()> {
    let store = open_store()?;
    match action {
        Some(AclAction::Allow { target }) => {
            let (pk, slug) = resolve_acl_target(&store, &target)?;
            crate::acl::allow(&pk, &slug)?;
            store.remove_pending_agent(&pk).ok();
            println!("authorized {} ({})", slug.cyan(), short_id(&pk));
        }
        Some(AclAction::Block { target }) => {
            let (pk, slug) = resolve_acl_target(&store, &target)?;
            crate::acl::block(&pk, &slug)?;
            store.remove_pending_agent(&pk).ok();
            println!("blocked {} ({})", slug, short_id(&pk));
        }
        Some(AclAction::List) | None => {
            let pending = store.list_pending_agents()?;
            println!(
                "{}",
                "pending (claim you as owner, awaiting your decision):".bold()
            );
            if pending.is_empty() {
                println!("  (none)");
            } else {
                for p in &pending {
                    println!(
                        "  {} {}  ({})  host {}",
                        "?".yellow(),
                        p.slug.cyan(),
                        short_id(&p.pubkey),
                        p.host.dimmed()
                    );
                }
                println!(
                    "\n  allow:  tenex-edge acl allow <slug|pubkey>\n  block:  tenex-edge acl block <slug|pubkey>"
                );
            }
            let allowed = crate::acl::allowed();
            let blocked = crate::acl::blocked();
            println!(
                "\n{} {} authorized, {} blocked",
                "acl:".bold(),
                allowed.len(),
                blocked.len()
            );
        }
    }
    Ok(())
}

/// Resolve an `acl` target (pubkey, or a pending-agent slug) to (pubkey, slug).
fn resolve_acl_target(store: &Store, target: &str) -> Result<(String, String)> {
    if target.len() == 64 && target.chars().all(|c| c.is_ascii_hexdigit()) {
        let slug = store
            .list_pending_agents()?
            .into_iter()
            .find(|p| p.pubkey == target)
            .map(|p| p.slug)
            .unwrap_or_else(|| "agent".to_string());
        return Ok((target.to_string(), slug));
    }
    // else treat as a pending-agent slug
    let m = store
        .list_pending_agents()?
        .into_iter()
        .find(|p| p.slug == target);
    match m {
        Some(p) => Ok((p.pubkey, p.slug)),
        None => bail!("no pending agent named {target:?}; use a pubkey or `tenex-edge acl list`"),
    }
}

// ── inbox ────────────────────────────────────────────────────────────────────

async fn inbox(session: Option<String>) -> Result<()> {
    let store = open_store()?;
    let rec = resolve_session(&store, session)?;
    // Self-fetch: pull recent stored mentions addressed to me straight from the
    // relay, so receive works even in one-shot sessions where the background
    // engine hasn't caught up yet. Best-effort; never blocks the drain.
    let _ = fetch_mentions_into_inbox(&store, &rec).await;
    let rows = store.drain_inbox(&rec.session_id)?;
    for r in &rows {
        println!("[mention from {}@{}] {}", r.from_slug, r.project, r.body);
        // Mark seen per-agent so it never resurfaces in a later session.
        store
            .mark_mention_seen(&rec.agent_pubkey, &r.mention_event_id, now_secs())
            .ok();
    }
    // Surface agents awaiting the human's authorization decision, so the agent
    // can tell its human (the injection hook prints this into context).
    let pending = store.list_pending_agents().unwrap_or_default();
    if !pending.is_empty() {
        let names: Vec<String> = pending
            .iter()
            .map(|p| format!("{} ({})", p.slug, short_id(&p.pubkey)))
            .collect();
        println!(
            "[tenex-edge] {} unauthorized agent(s) claim your owner: {}. \
They are NOT visible until you decide — tell your human to run `tenex-edge acl` to allow or block them.",
            pending.len(),
            names.join(", ")
        );
    }
    Ok(())
}

async fn fetch_mentions_into_inbox(store: &Store, rec: &crate::state::SessionRecord) -> Result<()> {
    use nostr_sdk::prelude::{Filter, Kind, PublicKey};
    let cfg = Config::load()?;
    let id = identity::load_or_create(&config::edge_home(), &rec.agent_slug, now_secs())?;
    let me = id.pubkey_hex();
    let pk = PublicKey::from_hex(&me)?;
    let transport = Transport::connect(&cfg.relays, id.keys.clone()).await?;
    let codec = Kind1Codec;
    let filter = Filter::new().kind(Kind::from(1u16)).pubkey(pk).limit(50);
    if let Ok(events) = transport.fetch(filter, Duration::from_secs(3)).await {
        for ev in events {
            if let Some(DomainEvent::Mention(m)) = codec.decode(&ev) {
                if m.to_pubkey != me {
                    continue;
                }
                // Skip mentions this agent already received in a prior session.
                if store.is_mention_seen(&me, &ev.id.to_hex()).unwrap_or(false) {
                    continue;
                }
                // Deliver to this session if it's the target, or if untargeted.
                let deliver = m
                    .target_session
                    .as_deref()
                    .map(|t| t == rec.session_id)
                    .unwrap_or(true);
                if deliver {
                    let _ = store.enqueue_mention(&crate::state::InboxRow {
                        mention_event_id: ev.id.to_hex(),
                        target_session: rec.session_id.clone(),
                        from_pubkey: m.from.pubkey.clone(),
                        from_slug: m.from.slug.clone(),
                        project: m.project.clone(),
                        body: m.body.clone(),
                        created_at: now_secs(),
                    });
                }
            }
        }
    }
    transport.shutdown().await;
    Ok(())
}

// ── wait-for-mention ─────────────────────────────────────────────────────────

async fn wait_for_mention(session: Option<String>, timeout: u64) -> Result<()> {
    let store = open_store()?;
    let rec = resolve_session(&store, session)?;
    // Self-fetch handles the engine warmup race (same as inbox).
    let _ = fetch_mentions_into_inbox(&store, &rec).await;

    let deadline = if timeout > 0 {
        Some(std::time::Instant::now() + Duration::from_secs(timeout))
    } else {
        None
    };

    loop {
        let rows = store.drain_inbox(&rec.session_id)?;
        if !rows.is_empty() {
            for r in &rows {
                println!("[mention from {}@{}] {}", r.from_slug, r.project, r.body);
                store
                    .mark_mention_seen(&rec.agent_pubkey, &r.mention_event_id, now_secs())
                    .ok();
            }
            println!("[tenex-edge] Run `tenex-edge wait-for-mention` with run_in_background=true to receive the next mention.");
            return Ok(());
        }
        if deadline
            .map(|d| std::time::Instant::now() >= d)
            .unwrap_or(false)
        {
            return Ok(());
        }
        tokio::time::sleep(Duration::from_millis(500)).await;
    }
}

// ── turn-start / turn-check / turn-end ───────────────────────────────────────

async fn turn_start(session: String, transcript: Option<String>, json_out: bool) -> Result<()> {
    if session.is_empty() {
        return Ok(());
    }
    let store = open_store()?;

    let (_, prev_turn_started_at) = store.get_turn_state(&session)?;
    let first_turn = prev_turn_started_at == 0;

    store.mark_turn_start(&session, now_secs())?;
    if let Some(ref path) = transcript {
        if !path.is_empty() {
            store.set_session_transcript(&session, path).ok();
        }
    }

    let rec = match store.get_session(&session)? {
        Some(r) => r,
        None => return Ok(()), // race with session-start, nothing to inject yet
    };

    let _ = fetch_mentions_into_inbox(&store, &rec).await;

    let mut blocks: Vec<String> = Vec::new();

    if first_turn {
        blocks.push(
            "[tenex-edge] Run `tenex-edge wait-for-mention` with run_in_background=true \
             so you are woken when a mention arrives. Re-run it each time one is received."
                .to_string(),
        );
    }

    // Drain inbox (authoritative delivery; turn_check only peeks).
    let inbox_rows = store.drain_inbox(&rec.session_id)?;
    if !inbox_rows.is_empty() {
        let mut text = String::from("Messages from other agents (tenex-edge):");
        for r in &inbox_rows {
            let _ = write!(text, "\n[mention from {}@{}] {}", r.from_slug, r.project, r.body);
            store
                .mark_mention_seen(&rec.agent_pubkey, &r.mention_event_id, now_secs())
                .ok();
        }
        blocks.push(text);
    }

    // Pending ACL agents (unknown agents claiming this owner).
    let pending = store.list_pending_agents().unwrap_or_default();
    if !pending.is_empty() {
        let names: Vec<String> = pending
            .iter()
            .map(|p| format!("{} ({})", p.slug, short_id(&p.pubkey)))
            .collect();
        blocks.push(format!(
            "[tenex-edge] {} unauthorized agent(s) claim your owner: {}. \
             They are NOT visible until you decide — tell your human to run \
             `tenex-edge acl` to allow or block them.",
            pending.len(),
            names.join(", ")
        ));
    }

    // Peer presence — full roster on the first turn; deltas on subsequent turns.
    if first_turn {
        let snapshot = load_who_snapshot(&store, None, false, now_secs())?;
        if !snapshot.rows.is_empty() {
            let who_text = render_who_plain(&snapshot);
            blocks.push(format!(
                "tenex-edge fabric — agents you can message with \
                 `tenex-edge send-message --recipient <agent|session-id> --message \"...\"`:\n{}",
                who_text.trim_end()
            ));
        }
    } else {
        // Only surface new arrivals and status changes since the last turn began.
        let fresh_since = now_secs().saturating_sub(PEER_FRESH_SECS);
        let new_peers =
            store.list_new_peer_sessions(prev_turn_started_at, fresh_since).unwrap_or_default();
        let status_changes =
            store.list_status_changes_since(prev_turn_started_at).unwrap_or_default();

        let mut delta: Vec<String> = Vec::new();
        for p in &new_peers {
            let age = now_secs().saturating_sub(p.last_seen);
            delta.push(format!(
                "  ● {}@{} joined  {}  session {}  ({age}s ago)",
                p.slug,
                slugify_host(&p.host),
                p.project,
                short_id(&p.session_id),
            ));
        }
        for (slug, project, text) in &status_changes {
            delta.push(format!("  ↻ {slug}@{project} — {text}"));
        }
        if !delta.is_empty() {
            blocks.push(format!(
                "tenex-edge fabric — changes since your last turn:\n{}",
                delta.join("\n")
            ));
        }
    }

    if !blocks.is_empty() {
        emit_context(&blocks.join("\n\n"), json_out);
    }
    Ok(())
}

/// Mid-turn inbox check for PostToolUse hooks. Read-only: only peeks the inbox
/// without draining. No writes to state.db.
fn turn_check(session: Option<String>, json_out: bool) -> Result<()> {
    let store = open_store()?;
    let rec = resolve_session(&store, session)?;

    let rows = store.peek_inbox(&rec.session_id)?;
    if rows.is_empty() {
        return Ok(());
    }

    let mut text = String::from("[tenex-edge] Message(s) arrived while you were working:");
    for r in &rows {
        let _ = write!(text, "\n[mention from {}@{}] {}", r.from_slug, r.project, r.body);
    }
    emit_context(&text, json_out);
    Ok(())
}

fn render_who_plain(snapshot: &WhoSnapshot) -> String {
    let mut out = String::new();
    let _ = writeln!(out, "agents:");
    for row in &snapshot.rows {
        let dot = if row.fresh { "●" } else { "○" };
        let status = status_plain(&row.status);
        match row.source {
            WhoSource::Local => {
                let _ = writeln!(
                    out,
                    "  {dot} {}@{} — {status}  {}  session {}  (this machine)",
                    row.slug,
                    slugify_host(&row.host),
                    row.project,
                    short_id(&row.session_id),
                );
            }
            WhoSource::Peer => {
                let age = row.age_secs.unwrap_or(0);
                let _ = writeln!(
                    out,
                    "  {dot} {}@{} — {status}  {}  session {}  ({age}s ago)",
                    row.slug,
                    slugify_host(&row.host),
                    row.project,
                    short_id(&row.session_id),
                );
            }
        }
    }
    out
}

fn emit_context(content: &str, json_out: bool) {
    if json_out {
        let obj = serde_json::json!({"systemMessage": content});
        println!("{obj}");
    } else {
        println!("{content}");
    }
}

fn turn_end(session: String) -> Result<()> {
    if session.is_empty() {
        return Ok(());
    }
    let store = open_store()?;
    store.mark_turn_end(&session)?;
    Ok(())
}

// ── doctor ───────────────────────────────────────────────────────────────────

async fn doctor() -> Result<()> {
    use nostr_sdk::prelude::{Alphabet, EventBuilder, Filter, Keys, Kind, SingleLetterTag, Tag};
    let cfg = Config::load()?;
    println!("relays: {:?}", cfg.relays);
    let keys = Keys::generate();
    println!("probe pubkey: {}", keys.public_key().to_hex());
    let t = format!("te-doctor-{}", now_secs());

    let transport = Transport::connect(&cfg.relays, keys).await?;
    let builder = EventBuilder::new(Kind::from(1u16), format!("tenex-edge doctor {t}"))
        .tags([Tag::parse(["h", &t])?]);
    match transport.publish_builder(builder).await {
        Ok(id) => println!("publish: OK ({})", short_id(&id.to_hex())),
        Err(e) => println!("publish: ERR {e:#}"),
    }
    tokio::time::sleep(Duration::from_secs(1)).await;
    let f = Filter::new()
        .kind(Kind::from(1u16))
        .custom_tag(SingleLetterTag::lowercase(Alphabet::H), &t)
        .limit(5);
    match transport.fetch(f, Duration::from_secs(5)).await {
        Ok(evs) => println!("read-back: {} event(s) with #h={t}", evs.len()),
        Err(e) => println!("read-back: ERR {e:#}"),
    }
    transport.shutdown().await;
    Ok(())
}

// ── tail ─────────────────────────────────────────────────────────────────────

async fn tail(project: Option<String>) -> Result<()> {
    let cfg = Config::load()?;
    let reader = Transport::connect(&cfg.relays, nostr_sdk::prelude::Keys::generate()).await?;
    let codec = Kind1Codec;
    let scope = SubScope {
        authors: Vec::new(),
        project: project.clone(),
        mentions_to: None,
        owners: Vec::new(),
    };
    reader.subscribe(codec.filters(&scope)).await?;
    let mut notifications = reader.notifications();

    let scope_label = project.as_deref().unwrap_or("*");
    eprintln!(
        "{} tailing project {} … (Ctrl-C to stop)",
        "tenex-edge".bold(),
        scope_label.cyan()
    );

    loop {
        tokio::select! {
            _ = tokio::signal::ctrl_c() => break,
            n = notifications.recv() => match n {
                Ok(RelayPoolNotification::Event { event, .. }) => {
                    if let Some(de) = codec.decode(&event) {
                        println!("{}", render(&de));
                    }
                }
                Ok(_) => {}
                Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
                Err(_) => {}
            }
        }
    }
    reader.shutdown().await;
    Ok(())
}

fn render(de: &DomainEvent) -> String {
    match de {
        DomainEvent::Profile(p) => {
            format!(
                "{} {}@{}",
                "id  ".dimmed(),
                p.agent.slug.cyan(),
                p.host.dimmed()
            )
        }
        DomainEvent::Presence(p) => format!(
            "{} {}@{} {} ({})",
            "live".green(),
            p.agent.slug.cyan(),
            slugify_host(&p.host),
            short_id(&p.session_id).yellow(),
            p.project.dimmed()
        ),
        DomainEvent::Activity(a) => {
            format!("{} {}: {}", "act ".blue(), a.agent.slug.cyan(), a.text)
        }
        DomainEvent::Status(s) if s.is_idle() => {
            format!("{} {} idle", "stat".dimmed(), s.agent.slug.cyan())
        }
        DomainEvent::Status(s) => {
            format!("{} {}: {}", "stat".magenta(), s.agent.slug.cyan(), s.text)
        }
        DomainEvent::Mention(m) => format!(
            "{} {} -> {}{}: {}",
            "msg ".yellow(),
            m.from.slug.cyan(),
            short_id(&m.to_pubkey),
            m.target_session
                .as_deref()
                .map(|s| format!(" ({})", short_id(s)))
                .unwrap_or_default(),
            m.body
        ),
    }
}

fn env_duration(key: &str, default: Duration) -> Duration {
    std::env::var(key)
        .ok()
        .and_then(|v| v.parse().ok())
        .map(Duration::from_millis)
        .unwrap_or(default)
}
fn env_u64(key: &str, default: u64) -> u64 {
    std::env::var(key)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}
