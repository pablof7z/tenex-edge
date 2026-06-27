use super::*;

pub(in crate::daemon::server) fn resolve_session(
    state: &Arc<DaemonState>,
    explicit: Option<&str>,
    env_session: Option<&str>,
    cwd: Option<&str>,
    agent: Option<&str>,
    group: Option<&str>,
) -> Result<crate::state::SessionRecord> {
    resolve_session_inner(state, explicit, env_session, cwd, agent, group, true)
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
) -> Result<crate::state::SessionRecord> {
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
    // A subgroup session is stored under its child group id (`h`), not the
    // working-directory project. When the caller is inside such a session its
    // pane exports `TENEX_EDGE_CHANNEL`; prefer it over the cwd-derived project so
    // the (agent, project) lookup finds the subgroup session rather than a
    // sibling parent-project session.
    // When the caller carries an explicit group, that group id IS the project a
    // session is stored under (a `tenex-edge launch`-spawned subgroup session).
    // Otherwise the project is the bare work-root from `cwd` — and a
    // human-initiated session is stored under a per-session room minted beneath
    // it (issue #6), so an exact `project` match misses. `work_root` drives the
    // room-aware fallback below for exactly that case.
    let explicit_group = group.filter(|g| !g.is_empty());
    let work_root = explicit_group.is_none();
    let project = explicit_group
        .map(|g| g.to_string())
        .unwrap_or_else(|| crate::project::resolve(&cwd).unwrap_or_default());
    if let Some(agent) = agent.filter(|a| !a.is_empty()) {
        if let Some(rec) =
            state.with_store(|s| s.latest_alive_session_for_agent_in_project(agent, &project))?
        {
            return Ok(rec);
        }
        if work_root {
            if let Some(rec) = state
                .with_store(|s| s.latest_alive_session_under_work_root(&project, Some(agent)))?
            {
                return Ok(rec);
            }
        }
        anyhow::bail!(
            "no active tenex-edge session for agent {agent:?} in project {project:?} (run session-start, or pass --session)"
        );
    }
    if !allow_project_fallback {
        anyhow::bail!(
            "not running as a tenex-edge agent: no --session, TENEX_EDGE_SESSION, or TENEX_EDGE_AGENT in scope"
        );
    }
    if let Some(rec) = state.with_store(|s| s.latest_alive_session_for_project(&project))? {
        return Ok(rec);
    }
    if work_root {
        if let Some(rec) =
            state.with_store(|s| s.latest_alive_session_under_work_root(&project, None))?
        {
            return Ok(rec);
        }
    }
    anyhow::bail!(
        "no active tenex-edge session for project {project:?} (run session-start, or pass --session)"
    )
}

// ── who ──────────────────────────────────────────────────────────────────────
