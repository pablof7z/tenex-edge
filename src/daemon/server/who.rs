use super::*;
use std::collections::BTreeSet;

#[path = "who/agent.rs"]
mod agent_view;
#[path = "who/human.rs"]
mod human_view;

#[derive(serde::Deserialize, Default)]
pub(in crate::daemon::server) struct WhoParams {
    #[serde(default)]
    workspace: Option<String>,
    #[serde(default)]
    all_workspaces: bool,
    #[serde(default)]
    cwd: Option<String>,
    #[serde(default)]
    harness_session: Option<String>,
    #[serde(default)]
    pty_session: Option<String>,
    #[serde(default)]
    watch_pid: Option<i32>,
    #[serde(default)]
    agent: Option<String>,
    #[serde(default)]
    group: Option<String>,
    #[serde(default)]
    human_color: bool,
    /// `who --expired`: list this machine's dead/old sessions by public handle so a
    /// user can pick one to resume, instead of the live fabric snapshot.
    #[serde(default)]
    expired: bool,
}

/// Cap on the expired-session listing — the resume-candidate window.
const EXPIRED_SESSION_LIMIT: u32 = 100;

/// `who`: build the snapshot with the SAME function the CLI used. The client
/// renders it with the existing renderers, so output is byte-identical. The
/// daemon resolves the current workspace the same way the old CLI did.
pub(in crate::daemon::server) fn rpc_who(
    state: &Arc<DaemonState>,
    params: &serde_json::Value,
) -> Result<serde_json::Value> {
    let p: WhoParams = serde_json::from_value(params.clone()).unwrap_or_default();
    if p.expired {
        let host = state.host.clone();
        let rows = state.with_store(|s| {
            crate::expired_sessions::load_expired_sessions(s, &host, EXPIRED_SESSION_LIMIT)
        });
        return Ok(serde_json::json!({ "expired": rows }));
    }
    let anchor = CallerAnchor::from_params(params);
    let has_exact_anchor = p.pty_session.as_deref().filter(|s| !s.is_empty()).is_some()
        || p.harness_session
            .as_deref()
            .filter(|s| !s.is_empty())
            .is_some()
        || p.watch_pid.is_some();
    let caller_rec = if has_exact_anchor {
        match resolve_session_inner(state, &anchor, ResolveScope::Strict) {
            Ok(rec) => Some(rec),
            Err(_) if p.all_workspaces => None,
            Err(error) => return Err(error),
        }
    } else if !p.all_workspaces
        && (p.agent.as_deref().filter(|s| !s.is_empty()).is_some()
            || p.group.as_deref().filter(|s| !s.is_empty()).is_some())
    {
        anyhow::bail!(
            "who needs an exact live session anchor; agent/channel env alone is not session context"
        );
    } else {
        None
    };
    let current_root = if p.all_workspaces {
        None
    } else if let Some(rec) = caller_rec.as_ref() {
        Some(rec.channel_h.clone())
    } else {
        Some(p.workspace.clone().unwrap_or_else(|| {
            let cwd = p
                .cwd
                .clone()
                .map(std::path::PathBuf::from)
                .unwrap_or_else(|| std::env::current_dir().unwrap_or_default());
            crate::workspace::resolve(&cwd).unwrap_or_default()
        }))
    };
    let now = now_secs();
    let host = state.host.clone();
    // This daemon's own management pubkey, excluded from every rendered roster so
    // the backend key never appears as a channel member (its kind:0 is absent on a
    // cold cache, so identity — not a fetched profile — is the reliable signal).
    let backend_pk = state.backend_pubkey().unwrap_or_default();
    let snapshot = state.with_store(|s| {
        crate::who_snapshot::load_who_snapshot(s, current_root.as_deref(), now, &host)
    })?;
    let mut out = serde_json::to_value(&snapshot)?;

    // Exact agent sessions get the structured agent awareness projection. Bare
    // operator invocations keep the terminal-oriented fabric renderer.
    if let Some(scope) = current_root.as_deref() {
        // Reuse the exact caller session, when present, for both the fabric
        // `(you)` match and the folded-in `<self />` row (issue #99).
        // Deliberately no root-scan fallback: `who` must not masquerade as a
        // session just because some live sibling exists in the same repository.
        let rec = caller_rec.as_ref();
        // Issue #98: the caller's ONE authoritative agent-instance identity — the
        // selected pubkey + ordinal label every publisher signs with. Computed
        // OUTSIDE `with_store` because `session_instance` locks the store itself.
        let instance = rec.map(|rec| state.session_instance(rec));
        let (self_slug, self_pubkey) = instance
            .as_ref()
            .map(|i| (i.display_slug(), i.pubkey.clone()))
            .unwrap_or_default();
        // `who` is an explicit orientation command. Even from inside an agent
        // session, render the full view rather than that session's delta cursor.
        let render_cursor = 0;
        let fabric = if let Some(agent_session) = rec {
            let roots = state.with_store(root_channels)?;
            Some(agent_view::render(
                state,
                &roots,
                agent_session,
                now,
                &host,
                &backend_pk,
                false,
            ))
        } else {
            state.with_store(|s| {
                crate::fabric_context::render_fabric_context(
                    s,
                    crate::fabric_context::FabricContextInput {
                        session: None,
                        scope,
                        cursor: render_cursor,
                        now,
                        self_slug: &self_slug,
                        self_pubkey: &self_pubkey,
                        backend_pubkey: &backend_pk,
                        local_host: &host,
                        forced_messages: &[],
                        warnings: &[],
                        force: true,
                    },
                )
            })
        };
        if let Some(fabric) = fabric {
            out["fabric"] = serde_json::Value::String(fabric);
            if rec.is_none() {
                let human = state.with_store(|s| {
                    crate::fabric_context::render_fabric_context_human(
                        s,
                        crate::fabric_context::FabricContextInput {
                            session: rec,
                            scope,
                            cursor: 0,
                            now,
                            self_slug: &self_slug,
                            self_pubkey: &self_pubkey,
                            backend_pubkey: &backend_pk,
                            local_host: &host,
                            forced_messages: &[],
                            warnings: &[],
                            force: true,
                        },
                        p.human_color,
                    )
                });
                if let Some(mut human) = human {
                    human_view::append_other_roots(
                        &mut human,
                        &snapshot.other_roots,
                        p.human_color,
                    );
                    out["fabric_human"] = serde_json::Value::String(human);
                }
            }
            if let Some(rec) = rec {
                if let Err(e) = cursor::drive_cursor_request(
                    state,
                    "who",
                    cursor::seed_from_session(rec),
                    cursor::fact_from_session(rec, now, true),
                ) {
                    tracing::error!(
                        session = %rec.session_id,
                        error = ?e,
                        "who: advancing session fabric cursor failed"
                    );
                }
            }
        }
    } else if p.all_workspaces {
        let roots = state.with_store(root_channels)?;
        if let Some(rec) = caller_rec.as_ref() {
            let fabric = agent_view::render(state, &roots, rec, now, &host, &backend_pk, true);
            out["fabric"] = serde_json::Value::String(fabric);
        } else {
            let fabric = state.with_store(|s| {
                crate::fabric_context::render_fabric_all_workspaces(
                    s,
                    &roots,
                    now,
                    &host,
                    &backend_pk,
                )
            });
            out["fabric"] = serde_json::Value::String(fabric);
            let human = state.with_store(|s| {
                crate::fabric_context::render_fabric_all_workspaces_human(
                    s,
                    &roots,
                    now,
                    &host,
                    &backend_pk,
                    p.human_color,
                )
            });
            out["fabric_human"] = serde_json::Value::String(human);
        }
    }
    Ok(out)
}

/// Top-level root channels (`parent` empty), non-archived — the set
/// `--all-workspaces` fans its unified fabric render across.
fn root_channels(store: &crate::state::Store) -> Result<Vec<String>> {
    let mut roots = store
        .reader()
        .list_channels()?
        .into_iter()
        .filter(|c| c.parent.is_empty() && !c.is_archived())
        .map(|c| c.channel_h)
        .collect::<BTreeSet<_>>();
    roots.extend(
        store
            .list_workspace_bindings()?
            .into_iter()
            .map(|binding| binding.channel_h),
    );
    Ok(roots.into_iter().collect())
}

#[cfg(test)]
mod tests;
