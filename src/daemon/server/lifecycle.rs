use super::session::ensure_group_and_membership;
use super::*;

pub(super) async fn spawn_session(state: &Arc<DaemonState>, params: EngineParams) -> Result<()> {
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
    let transport = state.transport.clone();
    let store = state.store.clone();
    tokio::spawn(async move {
        let res = runtime::run_session_in_daemon(params, transport, store, cancel).await;
        if let Err(e) = res {
            if std::env::var("TENEX_EDGE_DEBUG").is_ok() {
                eprintln!("[daemon] session {sid} task error: {e:#}");
            }
        }
        st.sessions.lock().unwrap().remove(&sid);
        prune_hosted(&st);
        st.liveness_changed.notify_waiters();
    });
    Ok(())
}

fn prune_hosted(state: &Arc<DaemonState>) {
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

pub(super) fn cancel_session(state: &Arc<DaemonState>, session_id: &str) -> bool {
    if let Some(h) = state.sessions.lock().unwrap().get(session_id) {
        h.cancel.notify_waiters();
        true
    } else {
        false
    }
}

pub(super) async fn ensure_subscription(state: &Arc<DaemonState>, project: &str) -> Result<()> {
    {
        let mut projs = state.subscribed_projects.lock().unwrap();
        if !projs.iter().any(|p| p == project) {
            projs.push(project.to_string());
        }
    }
    resubscribe(state).await
}

/// Rebuild and apply the union subscription across all hosted agents/projects.
pub(super) async fn resubscribe(state: &Arc<DaemonState>) -> Result<()> {
    let mut authors: Vec<String> = crate::acl::allowed().into_iter().collect();
    authors.extend(state.hosted_pubkeys());
    authors.sort();
    authors.dedup();

    let projects = state.subscribed_projects.lock().unwrap().clone();
    let owners = state.owners.clone();
    let hosted = state.hosted_pubkeys();

    for project in &projects {
        if hosted.is_empty() {
            let scope = SubScope {
                authors: authors.clone(),
                project: Some(project.clone()),
                mentions_to: None,
                owners: owners.clone(),
            };
            state
                .transport
                .subscribe(state.codec.filters(&scope))
                .await?;
        } else {
            for me in &hosted {
                let scope = SubScope {
                    authors: authors.clone(),
                    project: Some(project.clone()),
                    mentions_to: Some(me.clone()),
                    owners: owners.clone(),
                };
                state
                    .transport
                    .subscribe(state.codec.filters(&scope))
                    .await?;
            }
        }
    }
    Ok(())
}

/// Revive sessions a previous daemon left alive (skew re-exec / crash). For each
/// `alive=1` row: respawn the engine task if its `watch_pid` is still alive,
/// else mark it dead (so `who`/presence don't lie after a restart).
pub(super) async fn reconcile_sessions(state: &Arc<DaemonState>) {
    let alive = state.with_store(|s| s.list_alive_sessions().unwrap_or_default());
    for rec in alive {
        let pid_ok = rec.watch_pid.map(pid_alive).unwrap_or(false);
        if !pid_ok {
            state.with_store(|s| {
                s.mark_session_dead(&rec.session_id).ok();
            });
            continue;
        }
        let id = match identity::load_or_create(&config::edge_home(), &rec.agent_slug, now_secs()) {
            Ok(i) => i,
            Err(_) => continue,
        };
        // Re-establish ownership/membership + the group-state subscription for
        // revived sessions. Idempotent: the owned_groups/group_members cache
        // persists across restarts, so already-owned groups skip republishing.
        ensure_group_and_membership(state, &rec.project, &id.pubkey_hex()).await;
        let ep = engine_params_for(
            &state.cfg,
            &id,
            &rec.agent_slug,
            &rec.session_id,
            &rec.project,
            &rec.rel_cwd,
            rec.watch_pid,
        );
        let _ = spawn_session(state, ep).await;
    }
}

pub(super) fn engine_params_for(
    cfg: &Config,
    id: &AgentIdentity,
    agent_slug: &str,
    session_id: &str,
    project: &str,
    rel_cwd: &str,
    watch_pid: Option<i32>,
) -> EngineParams {
    EngineParams {
        agent_slug: agent_slug.to_string(),
        agent_pubkey: id.pubkey_hex(),
        keys: id.keys.clone(),
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
        turn_repeat: Duration::from_secs(env_u64("TENEX_EDGE_TURN_REPEAT_S", 300)),
    }
}

fn pid_alive(pid: i32) -> bool {
    nix::sys::signal::kill(nix::unistd::Pid::from_raw(pid), None).is_ok()
}

fn env_duration(key: &str, default: Duration) -> Duration {
    std::env::var(key)
        .ok()
        .and_then(|v| v.parse().ok())
        .map(Duration::from_millis)
        .unwrap_or(default)
}
