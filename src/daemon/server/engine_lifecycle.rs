use super::*;

pub(in crate::daemon::server) async fn spawn_session(
    state: &Arc<DaemonState>,
    params: EngineParams,
) -> Result<()> {
    let session_id = params.session_id.clone();
    let pubkey = params.agent_pubkey.clone();
    let project = params.project.clone();

    state.hosted.lock().unwrap().insert(
        pubkey.clone(),
        HostedAgent {
            keys: params.keys.clone(),
        },
    );
    ensure_subscription(state, &project).await?;

    let cancel = Arc::new(Notify::new());
    state.sessions.lock().unwrap().insert(
        session_id.clone(),
        SessionHandle {
            cancel: cancel.clone(),
        },
    );
    state.liveness_changed.notify_waiters();

    let st = state.clone();
    let sid = session_id.clone();
    let proj = project.clone();
    let provider = state.provider.clone();
    let store = state.store.clone();
    tokio::spawn(async move {
        let res = runtime::run_session_in_daemon(params, provider, store, cancel).await;
        if let Err(e) = res {
            if std::env::var("TENEX_EDGE_DEBUG").is_ok() {
                eprintln!("[daemon] session {sid} task error: {e:#}");
            }
        }
        // Engine self-exit path: remove a transient duplicate signer from the
        // NIP-29 group. The Mutex pop is atomic: if rpc_session_end already
        // removed the key, this finds None and avoids a duplicate publish.
        {
            let maybe_key = st.release_session_signer(&sid, &pubkey, &proj);
            if let Some(sk) = maybe_key {
                let session_pubkey = sk.public_key().to_hex();
                st.provider
                    .nip29_remove_member(&proj, &session_pubkey)
                    .await;
                st.with_store(|s| {
                    s.remove_group_member(&proj, &session_pubkey).ok();
                });
            }
        }
        // Clear the DB routing row regardless of whether the in-memory key was
        // still present (graceful end clears it; self-exit may too).
        st.with_store(|s| {
            s.remove_session_pubkeys_for_session(&sid).ok();
        });
        st.sessions.lock().unwrap().remove(&sid);
        prune_hosted(&st);
        st.liveness_changed.notify_waiters();
    });
    Ok(())
}

pub(in crate::daemon::server) fn prune_hosted(state: &Arc<DaemonState>) {
    let live: std::collections::HashSet<String> = state
        .with_store(|s| s.list_alive_sessions().unwrap_or_default())
        .into_iter()
        .map(|r| r.agent_pubkey)
        .collect();
    state
        .hosted
        .lock()
        .unwrap()
        .retain(|pk, _| live.contains(pk));
}

pub(in crate::daemon::server) fn cancel_session(
    state: &Arc<DaemonState>,
    session_id: &str,
) -> bool {
    if let Some(h) = state.sessions.lock().unwrap().get(session_id) {
        h.cancel.notify_waiters();
        true
    } else {
        false
    }
}

pub(in crate::daemon::server) async fn ensure_subscription(
    state: &Arc<DaemonState>,
    project: &str,
) -> Result<()> {
    {
        let mut projs = state.subscribed_projects.lock().unwrap();
        if !projs.iter().any(|p| p == project) {
            projs.push(project.to_string());
        }
    }
    // Bounded: `resubscribe` opens a relay subscription, which can hang on a slow
    // or unreachable relay. `ensure_subscription` is awaited on hook-critical
    // paths (session_start, spawn_session), so a hang would block the editor.
    // The intent (the project is in `subscribed_projects`) is recorded above; on
    // timeout we fail open and the next session event re-runs resubscribe.
    match tokio::time::timeout(std::time::Duration::from_secs(5), resubscribe(state)).await {
        Ok(r) => r,
        Err(_) => {
            if std::env::var("TENEX_EDGE_DEBUG").is_ok() {
                eprintln!("[daemon] resubscribe timed out for {project} (best-effort)");
            }
            Ok(())
        }
    }
}

/// Rebuild and apply the union subscription across all hosted agents/projects.
pub(in crate::daemon::server) async fn resubscribe(state: &Arc<DaemonState>) -> Result<()> {
    let hosted = state.hosted_pubkeys();
    let session_pks = state.live_session_pubkeys();

    // Authors for kind:0 / kind:30315 include both durable agent keys and active
    // transient session keys so peers receive session-signed presence.
    let mut authors: Vec<String> = hosted.clone();
    authors.extend(session_pks.clone());
    authors.sort_unstable();
    authors.dedup();

    let projects = state.subscribed_projects.lock().unwrap().clone();
    let owners = state.owners.clone();

    // All pubkeys that should receive p-tagged mentions: durable + session.
    let mut all_me: Vec<String> = hosted.clone();
    all_me.extend(session_pks);
    all_me.sort_unstable();
    all_me.dedup();

    for project in &projects {
        if all_me.is_empty() {
            let scope = crate::fabric::Scope {
                authors: authors.clone(),
                project: Some(project.clone()),
                mentions_to: None,
                owners: owners.clone(),
            };
            state.provider.subscribe(scope).await?;
        } else {
            for me in &all_me {
                let scope = crate::fabric::Scope {
                    authors: authors.clone(),
                    project: Some(project.clone()),
                    mentions_to: Some(me.clone()),
                    owners: owners.clone(),
                };
                state.provider.subscribe(scope).await?;
            }
        }
    }

    Ok(())
}

/// Revive sessions a previous daemon left behind (skew re-exec / crash),
/// rebuilding from the canonical `session_state` aggregate. For each ACTIVE
/// session: respawn the engine task if its watched pid is still alive, else end
/// the canonical session AND mark the runtime row dead (so `who`/presence don't
/// lie after a restart). `watch_pid` lives in the kept `sessions` runtime table
/// (session_state carries no pid), so it is joined per session.
pub(in crate::daemon::server) async fn reconcile_sessions(state: &Arc<DaemonState>) {
    let now = now_secs();
    let snaps = state.with_store(|s| s.live_session_snapshots(None, 0).unwrap_or_default());
    for snap in snaps {
        let session_id = snap.session_id.as_str().to_owned();
        let watch_pid = state
            .with_store(|s| s.get_session(&session_id).ok().flatten())
            .and_then(|r| r.watch_pid);
        let pid_ok = watch_pid.map(pid_alive).unwrap_or(false);
        if !pid_ok {
            // Read the persisted session pubkey BEFORE deleting its row — it is
            // the authoritative value. Re-deriving from session_aliases is only a
            // fallback for rows written before this column existed; preferring the
            // stored pubkey avoids any chance of removing the wrong key (and thus
            // stranding the real one as a live member) if the recovered anchor
            // ever diverges from what session_start used.
            let stored_pubkey = state.with_store(|s| s.session_pubkey_for_session(&session_id));
            state.with_store(|s| {
                s.end_session(&session_id, now).ok();
                s.mark_session_dead(&session_id).ok();
                // Clear DB routing row for the dead session's transient pubkey.
                s.remove_session_pubkeys_for_session(&session_id).ok();
            });
            // Crash-GC: remove the session pubkey from the NIP-29 group.
            if let Some(nsec) = state.cfg.session_ikm_nsec().cloned() {
                if let Ok(op_keys) = nostr_sdk::prelude::Keys::parse(&nsec) {
                    let session_pubkey = stored_pubkey.unwrap_or_else(|| {
                        // Fallback: re-derive. Anchor recovered from session_aliases:
                        //   claude-code / codex → (harness, native_id)
                        //   opencode → anchor = session_id (resume alias only)
                        //   unknown / no rows → ("unknown", session_id)
                        let (harness_kind, anchor) =
                            state.with_store(|s| s.get_session_derivation_anchor(&session_id));
                        identity::derive_session_keys(
                            op_keys.secret_key(),
                            &snap.project,
                            &snap.agent_slug,
                            &harness_kind,
                            &anchor,
                        )
                        .public_key()
                        .to_hex()
                    });
                    let provider = state.provider.clone();
                    let store = state.store.clone();
                    let project = snap.project.clone();
                    tokio::spawn(async move {
                        provider
                            .nip29_remove_member(&project, &session_pubkey)
                            .await;
                        store
                            .lock()
                            .unwrap()
                            .remove_group_member(&project, &session_pubkey)
                            .ok();
                    });
                }
            }
            continue;
        }
        let id = match identity::load_or_create(&config::edge_home(), &snap.agent_slug, now) {
            Ok(i) => i,
            Err(_) => continue,
        };
        // Re-establish ownership/membership + the group-state subscription for
        // revived sessions, through the one channel-provisioning primitive. The
        // session's scope may be a top-level project (root channel) or a subgroup;
        // its stored parent (if any) is the readiness gate's parent_hint.
        // Idempotent: the owned_groups/group_members cache persists across
        // restarts, so already-ready channels fast-path.
        let parent_hint = state.with_store(|s| s.group_parent(&snap.project).unwrap_or(None));
        state
            .provider
            .ensure_channel_ready(crate::fabric::nip29::readiness::ChannelCtx {
                channel: &snap.project,
                expect_member: &id.pubkey_hex(),
                parent_hint: parent_hint.as_deref(),
            })
            .await;

        let (harness_kind, anchor) =
            state.with_store(|s| s.get_session_derivation_anchor(&session_id));
        let signer = match select_session_signer(
            state,
            &session_id,
            &id.pubkey_hex(),
            &snap.agent_slug,
            &snap.project,
            &harness_kind,
            &anchor,
        ) {
            Ok(signer) => signer,
            Err(e) => {
                if std::env::var("TENEX_EDGE_DEBUG").is_ok() {
                    eprintln!(
                        "[daemon] signer selection failed while reconciling {}: {e:#}",
                        session_id
                    );
                }
                continue;
            }
        };
        if let Some(session_pubkey) = signer.transient_pubkey() {
            if let Err(e) = admit_transient_signer(state, &snap.project, session_pubkey).await {
                if std::env::var("TENEX_EDGE_DEBUG").is_ok() {
                    eprintln!(
                        "[daemon] transient signer admission failed while reconciling {}: {e:#}",
                        session_id
                    );
                }
                state.release_session_signer(&session_id, &id.pubkey_hex(), &snap.project);
                state.with_store(|s| s.remove_session_pubkeys_for_session(&session_id).ok());
                continue;
            }
        }

        if let Err(e) = ensure_subscription(state, &snap.project).await {
            if std::env::var("TENEX_EDGE_DEBUG").is_ok() {
                eprintln!(
                    "[daemon] ensure_subscription({}) failed: {e:#}",
                    snap.project
                );
            }
        }
        let ep = engine_params_for(
            &state.cfg,
            &id,
            &snap.agent_slug,
            &session_id,
            &snap.project,
            &snap.rel_cwd,
            watch_pid,
            signer.session_keys(),
        );
        let _ = spawn_session(state, ep).await;
    }
    // Any registration/end transitions above enqueued publishes.
    state.status_outbox_notify.notify_waiters();
}

#[allow(clippy::too_many_arguments)]
pub(in crate::daemon::server) fn engine_params_for(
    cfg: &Config,
    id: &AgentIdentity,
    agent_slug: &str,
    session_id: &str,
    project: &str,
    rel_cwd: &str,
    watch_pid: Option<i32>,
    // Derived keypair for a duplicate live session in the same routing scope.
    // `None` keeps the durable agent key as the default signer.
    session_keys: Option<Keys>,
) -> EngineParams {
    EngineParams {
        agent_slug: agent_slug.to_string(),
        agent_pubkey: id.pubkey_hex(),
        keys: id.keys.clone(),
        session_keys,
        project: project.to_string(),
        session_id: session_id.to_string(),
        host: cfg.host.clone(),
        rel_cwd: rel_cwd.to_string(),
        owners: cfg.whitelisted_pubkeys.clone(),
        relays: cfg.relays.clone(),
        watch_pid,
        store_path: store_path(),
        heartbeat: env_duration("TENEX_EDGE_HEARTBEAT_MS", Duration::from_secs(30)),
        obs_interval: env_duration("TENEX_EDGE_OBS_MS", Duration::from_secs(5)),
        status_ttl: Duration::from_secs(env_u64("TENEX_EDGE_STATUS_TTL_S", 90)),
        turn_first: Duration::from_secs(env_u64("TENEX_EDGE_TURN_FIRST_S", 30)),
        // 0 = disabled: the title re-distills on each new user message, so an
        // in-turn safety re-distill is opt-in (set TENEX_EDGE_TURN_REPEAT_S > 0).
        turn_repeat: Duration::from_secs(env_u64("TENEX_EDGE_TURN_REPEAT_S", 0)),
    }
}

pub(in crate::daemon::server) fn pid_alive(pid: i32) -> bool {
    nix::sys::signal::kill(nix::unistd::Pid::from_raw(pid), None).is_ok()
}
