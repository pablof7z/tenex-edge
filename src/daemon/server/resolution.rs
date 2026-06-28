use super::*;
use crate::state::{Session, Store};

pub(in crate::daemon::server) fn resolve_session(
    state: &Arc<DaemonState>,
    explicit: Option<&str>,
    env_session: Option<&str>,
    cwd: Option<&str>,
    agent: Option<&str>,
    group: Option<&str>,
) -> Result<Session> {
    resolve_session_inner(state, explicit, env_session, cwd, agent, group, true)
}

/// The project channel a routing scope belongs under: a top-level channel is its
/// own work root; a sub-channel (task/session room) maps to its parent.
pub(in crate::daemon::server) fn work_root_for(s: &Store, scope: &str) -> String {
    match s.channel_parent(scope).ok().flatten() {
        Some(p) if !p.is_empty() => p,
        _ => scope.to_string(),
    }
}

/// Resolve the caller's session. `allow_project_fallback` controls the LAST
/// resort: when the caller carries no session/agent signal at all, `true` picks
/// the project's latest-alive session (fine for host-facing commands run from a
/// repo), while `false` errors instead — used by `whoami`, which is only
/// meaningful when actually run *as* an agent and must not silently bind to some
/// arbitrary sibling session when run from a bare terminal.
pub(in crate::daemon::server) fn resolve_session_inner(
    state: &Arc<DaemonState>,
    explicit: Option<&str>,
    env_session: Option<&str>,
    cwd: Option<&str>,
    agent: Option<&str>,
    group: Option<&str>,
    allow_project_fallback: bool,
) -> Result<Session> {
    if let Some(id) = explicit.filter(|s| !s.is_empty()) {
        return state
            .with_store(|s| s.get_session(id))
            .with_context(|| format!("unknown session {id}"))?
            .with_context(|| format!("unknown session {id}"));
    }
    if let Some(id) = env_session.filter(|s| !s.is_empty()) {
        if let Some(rec) = state.with_store(|s| s.get_session(id)).ok().flatten() {
            return Ok(rec);
        }
    }
    let cwd = cwd
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_default());
    // An explicit group id IS the channel a session is stored under (a subgroup
    // session). Otherwise the project is the work-root channel derived from cwd; a
    // session stored under a child channel still belongs to that work root, so the
    // scope match below also accepts sub-channels whose parent is the project.
    let explicit_group = group.filter(|g| !g.is_empty());
    let work_root = explicit_group.is_none();
    let project = explicit_group
        .map(|g| g.to_string())
        .unwrap_or_else(|| crate::project::resolve(&cwd).unwrap_or_default());
    let want_agent = agent.filter(|a| !a.is_empty());

    // `list_alive_sessions` is newest-first, so the first match is the latest.
    let pick = state.with_store(|s| {
        s.list_alive_sessions()
            .unwrap_or_default()
            .into_iter()
            .find(|rec| {
                let scope_ok = rec.channel_h == project
                    || (work_root
                        && s.channel_parent(&rec.channel_h).ok().flatten().as_deref()
                            == Some(project.as_str()));
                let agent_ok = want_agent.map(|a| rec.agent_slug == a).unwrap_or(true);
                scope_ok && agent_ok
            })
    });
    if let Some(rec) = pick {
        return Ok(rec);
    }
    if let Some(agent) = want_agent {
        anyhow::bail!(
            "no active tenex-edge session for agent {agent:?} in project {project:?} (run session-start, or pass --session)"
        );
    }
    if !allow_project_fallback {
        anyhow::bail!(
            "not running as a tenex-edge agent: no --session, TENEX_EDGE_SESSION, or TENEX_EDGE_AGENT in scope"
        );
    }
    anyhow::bail!(
        "no active tenex-edge session for project {project:?} (run session-start, or pass --session)"
    )
}
