use super::super::resolution::work_root_for;
use super::super::*;
use std::sync::Arc;

/// Spawn a local agent that was p-tagged in a kind:9 message but had no alive
/// session. Idempotency: `first_sight` prevents duplicate spawns within a run;
/// `has_alive` prevents re-spawn across restarts when the previous spawn registered.
/// Delivery: `rpc_session_start` calls `ensure_subscription`, which triggers a
/// relay replay of recent kind:9 events; those are re-materialized against the
/// now-alive session and delivered via `ring_doorbells`.
pub(super) async fn handle(
    state: &Arc<DaemonState>,
    mentioned_pk: &str,
    project: &str,
    body: &str,
) {
    let has_alive = state.with_store(|s| {
        s.list_alive_sessions()
            .unwrap_or_default()
            .into_iter()
            .any(|rec| {
                rec.agent_pubkey == mentioned_pk
                    && s.is_session_joined_channel(&rec.session_id, project)
                        .unwrap_or(rec.channel_h == project)
            })
    });
    if has_alive {
        tracing::debug!(
            mentioned_pk = %crate::util::pubkey_short(mentioned_pk),
            project,
            "agent already has alive session - skipping spawn"
        );
        return;
    }

    // Resolve the mentioned pubkey to a known ordinal identity row. Local
    // derivation-root keys are not fabric agent identities under the roster model.
    let Some(idn) = state.with_store(|s| {
        s.get_identity_for_channel(mentioned_pk, project)
            .ok()
            .flatten()
            .or_else(|| s.get_identity(mentioned_pk).ok().flatten())
    }) else {
        return;
    };
    let (agent_slug, ordinal) = (idn.agent_slug, idn.ordinal);

    // Resume vs fresh: if this identity previously ran in this channel and left a
    // bound native session, RESUME that harness (restores its conversation);
    // otherwise spawn fresh with the exact ordinal.
    let bound = state.with_store(|s| {
        s.resolve_identity_pubkey_for_channel(mentioned_pk, project)
            .ok()
            .flatten()
    });
    if let Some(route) = bound.filter(|r| !r.native_id.is_empty()) {
        tracing::info!(
            agent = %route.agent_slug,
            project,
            native_id = %route.native_id,
            "resuming bound native session"
        );
        let resume_work_root = state.with_store(|s| work_root_for(s, project));
        match spawn_headless_mention(
            state,
            &agent_slug,
            &resume_work_root,
            project,
            body,
            Some(&route.native_id),
            None,
        )
        .await
        {
            Ok(true) => return,
            Ok(false) => {}
            Err(e) => {
                tracing::warn!(agent = %agent_slug, project, error = %e, "headless resume failed - falling back to PTY resume");
            }
        }
        if let Err(e) =
            crate::session_host::resume_agent(state, &agent_slug, project, &route.native_id).await
        {
            tracing::warn!(agent = %agent_slug, project, error = %e, "session resume failed - falling through to fresh spawn");
        } else {
            return;
        }
    }

    let is_member =
        state.with_store(|s| s.is_channel_member(project, mentioned_pk).unwrap_or(false));
    if !is_member {
        let (_, _, members) = state.provider.fetch_group_state(project).await;
        if !members.contains(mentioned_pk) {
            tracing::info!(agent = %agent_slug, ordinal, project, "provisioning ordinal pubkey into channel via mgmt key");
            if !state
                .provider
                .grant_member_confirmed(project, mentioned_pk)
                .await
                .is_confirmed()
            {
                tracing::warn!(agent = %agent_slug, ordinal, project, "mgmt-key add_member was not confirmed - skipping spawn");
                return;
            }
        }
    }

    let work_root = state.with_store(|s| work_root_for(s, project));
    let has_path = state.with_store(|s| s.project_root(&work_root).ok().flatten().is_some());
    if !has_path {
        tracing::warn!(agent = %agent_slug, work_root = %work_root, project, "no local project root found - cannot spawn");
        return;
    }

    let group_arg = Some(project);
    tracing::info!(
        agent = %agent_slug,
        ordinal,
        project,
        work_root = %work_root,
        "spawning agent on mention"
    );
    match spawn_headless_mention(
        state,
        &agent_slug,
        &work_root,
        project,
        body,
        None,
        Some(ordinal),
    )
    .await
    {
        Ok(true) => return,
        Ok(false) => {}
        Err(e) => {
            tracing::warn!(agent = %agent_slug, project, error = %e, "headless spawn failed - falling back to PTY spawn");
        }
    }
    match crate::session_host::spawn_agent(
        state,
        &agent_slug,
        &work_root,
        Vec::new(),
        None,
        group_arg,
        None,
        Some(ordinal),
    )
    .await
    {
        Ok(pty_id) => {
            tracing::info!(agent = %agent_slug, pty_id = %pty_id, project, "agent spawned successfully")
        }
        Err(e) => tracing::warn!(agent = %agent_slug, project, error = %e, "agent spawn failed"),
    }
}

async fn spawn_headless_mention(
    state: &Arc<DaemonState>,
    agent_slug: &str,
    work_root: &str,
    channel_h: &str,
    body: &str,
    resume_id: Option<&str>,
    ordinal: Option<u32>,
) -> anyhow::Result<bool> {
    if !crate::session_host::agent_supports_headless_exec(agent_slug) {
        return Ok(false);
    }
    let prompt = mention_prompt(body);
    let launch = crate::session_host::spawn_agent_exec(
        state,
        agent_slug,
        work_root,
        &prompt,
        resume_id,
        None,
        Some(channel_h),
        None,
        ordinal,
    )
    .await?;
    tracing::info!(
        agent = %agent_slug,
        exec_id = %launch.id,
        pid = launch.pid(),
        log = %launch.log_path.display(),
        "headless agent spawned on mention"
    );
    reap_headless_on_exit(
        state.clone(),
        agent_slug.to_string(),
        channel_h.to_string(),
        launch,
    );
    Ok(true)
}

fn reap_headless_on_exit(
    state: Arc<DaemonState>,
    agent_slug: String,
    project: String,
    launch: crate::session_host::ExecLaunch,
) {
    let crate::session_host::ExecLaunch {
        id,
        mut child,
        log_path,
    } = launch;
    let pid = child.id() as i32;
    tokio::spawn(async move {
        let waited = tokio::task::spawn_blocking(move || child.wait()).await;
        match waited {
            Ok(Ok(status)) => {
                tracing::info!(
                    agent = %agent_slug,
                    project = %project,
                    exec_id = %id,
                    pid,
                    status = %status,
                    log = %log_path.display(),
                    "headless agent exited"
                );
            }
            Ok(Err(e)) => {
                tracing::warn!(
                    agent = %agent_slug,
                    project = %project,
                    exec_id = %id,
                    pid,
                    error = %e,
                    log = %log_path.display(),
                    "headless agent wait failed"
                );
            }
            Err(e) => {
                tracing::warn!(
                    agent = %agent_slug,
                    project = %project,
                    exec_id = %id,
                    pid,
                    error = %e,
                    log = %log_path.display(),
                    "headless agent wait task failed"
                );
            }
        }
        if let Err(e) = super::super::rpc_session_end(
            &state,
            &serde_json::json!({
                "session": pid.to_string(),
            }),
        )
        .await
        {
            tracing::warn!(
                agent = %agent_slug,
                project = %project,
                exec_id = %id,
                pid,
                error = %e,
                "headless agent session_end failed"
            );
        }
    });
}

fn mention_prompt(body: &str) -> String {
    let body = body.trim();
    let body = if body.is_empty() {
        "You were mentioned in tenex-edge. Check your channel context and respond if needed."
    } else {
        body
    };
    format!(
        "{body}\n\n[reply via `tenex-edge chat write --message \"...\"` - replies do not auto-publish]"
    )
}
