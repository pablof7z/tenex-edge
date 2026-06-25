use super::*;
use crate::session::{derive_status, DeltaKind, SessionSnapshot, StatusDeltaItem};

mod channel;
mod delta;
mod render;
mod snapshot;

// Public re-exports for the crate and cli module
pub(super) use channel::render_channel_context;
pub(super) use delta::{build_status_delta, push_turn_fabric_block};
pub use snapshot::{load_who_snapshot, WhoSnapshot};

// ── who ──────────────────────────────────────────────────────────────────────

/// `who` params for the daemon RPC. The daemon resolves the current project the
/// same way the old CLI did (`all_projects ? None : resolve(cwd)`).
fn who_params(project: &Option<String>, all_projects: bool) -> serde_json::Value {
    serde_json::json!({
        "project": project,
        "all_projects": all_projects,
        "env_session": std::env::var("TENEX_EDGE_SESSION").ok(),
        "agent": crate::cli::agent_env_slug(),
        "group": crate::cli::channel_env(),
        "cwd": std::env::current_dir().ok().map(|p| p.to_string_lossy().to_string()),
    })
}

fn who_snapshot_via_daemon(project: &Option<String>, all_projects: bool) -> Result<WhoSnapshot> {
    let v = crate::daemon::blocking::call("who", who_params(project, all_projects))?;
    Ok(serde_json::from_value(v)?)
}

pub(super) fn who(project: Option<String>, all_projects: bool) -> Result<()> {
    let snapshot = who_snapshot_via_daemon(&project, all_projects)?;
    print!("{}", render::render_who_for_stdout(&snapshot));
    Ok(())
}

pub(super) fn who_live(project: Option<String>, all_projects: bool) -> Result<()> {
    let refresh = Duration::from_millis(1000);
    let _terminal = render::LiveTerminal::enter()?;
    let mut next_draw = Instant::now();

    loop {
        let now = Instant::now();
        if now >= next_draw {
            let snapshot = who_snapshot_via_daemon(&project, all_projects)?;
            render::draw_who_live(&snapshot, refresh)?;
            next_draw = Instant::now() + refresh;
        }

        let wait = next_draw
            .saturating_duration_since(Instant::now())
            .min(Duration::from_millis(100));
        if event::poll(wait)? && render::should_quit_live(event::read()?) {
            break;
        }
    }

    Ok(())
}

/// `whoami`: print this session's own identity card. Resolves the current
/// session daemon-side (explicit `--session` → `TENEX_EDGE_SESSION` env → the
/// cwd's project), then renders the same agent/channel/host vocabulary used by
/// `who` and the hook-injected fabric context.
pub(super) async fn whoami(session: Option<String>, json: bool) -> Result<()> {
    let params = serde_json::json!({
        "session": session,
        "env_session": std::env::var("TENEX_EDGE_SESSION").ok(),
        "agent": crate::cli::agent_env_slug(),
        "group": crate::cli::channel_env(),
        "cwd": std::env::current_dir().ok().map(|p| p.to_string_lossy().to_string()),
    });
    let v = super::daemon_call_async("whoami", params).await?;
    if json {
        println!("{}", serde_json::to_string_pretty(&v)?);
    } else {
        print!("{}", render::render_whoami(&v));
    }
    Ok(())
}

#[cfg(test)]
mod tests;
