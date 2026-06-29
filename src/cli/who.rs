use super::*;

mod awareness;
mod channel;
mod render;
mod snapshot;

// Public re-exports for the crate and cli module
pub(super) use awareness::{
    render_awareness_snapshot, render_awareness_update_since_check,
    render_awareness_update_since_turn,
};
pub(crate) use awareness::new_agent_block;
pub use snapshot::{load_who_snapshot, WhoSnapshot};

/// The `tenex-edge who` full fabric view — the SAME format the hook injection
/// renders (decision A: ONE format, `who` is just the full snapshot projection),
/// plus the COMPLETE invitable roster (the full snapshot shows every summonable
/// agent; the hook delta only shows newly-added ones). `None` when the channel
/// is not yet materialized. Rendered daemon-side; the thin client prints it.
pub(crate) fn render_fabric_snapshot(
    store: &crate::state::Store,
    scope: &str,
    now: u64,
    self_slug: &str,
    self_pubkey: &str,
    local_host: &str,
    edge_home: &std::path::Path,
) -> Option<String> {
    let mut out = awareness::render_fabric_view(store, scope, now, self_slug, self_pubkey, local_host);
    if let Some(section) = invitable_section(edge_home) {
        out.push_str("\n\n");
        out.push_str(&section);
    }
    Some(out)
}

/// The "Agents you can invite" section for the full `who` snapshot: every local
/// keystore agent with its byline. Mirrors `tenex-edge agents`.
fn invitable_section(edge_home: &std::path::Path) -> Option<String> {
    let roster = crate::identity::list_invitable_agents(edge_home);
    if roster.is_empty() {
        return None;
    }
    let mut out = String::from("Agents you can invite (tenex-edge invite <slug>):");
    for (slug, byline, _) in roster {
        match byline.as_deref().map(str::trim).filter(|b| !b.is_empty()) {
            Some(b) => {
                let _ = write!(out, "\n- @{slug} - {b}");
            }
            None => {
                let _ = write!(out, "\n- @{slug}");
            }
        }
    }
    Some(out)
}

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

fn who_value_via_daemon(project: &Option<String>, all_projects: bool) -> Result<serde_json::Value> {
    crate::daemon::blocking::call("who", who_params(project, all_projects))
}

pub(super) fn who(project: Option<String>, all_projects: bool) -> Result<()> {
    let v = who_value_via_daemon(&project, all_projects)?;
    // Prefer the unified fabric view (same format as the hook injection). The
    // daemon includes it whenever a current channel resolves; `--all-projects`
    // (no single scope) falls back to the cross-project snapshot table.
    if let Some(fabric) = v.get("fabric").and_then(|x| x.as_str()) {
        println!("{fabric}");
        return Ok(());
    }
    let snapshot: WhoSnapshot = serde_json::from_value(v)?;
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
            let v = who_value_via_daemon(&project, all_projects)?;
            if let Some(fabric) = v.get("fabric").and_then(|x| x.as_str()) {
                render::draw_fabric_live(fabric, refresh)?;
            } else {
                let snapshot: WhoSnapshot = serde_json::from_value(v)?;
                render::draw_who_live(&snapshot, refresh)?;
            }
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
