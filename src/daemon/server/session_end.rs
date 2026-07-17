use super::*;

#[derive(serde::Deserialize)]
#[serde(rename_all = "snake_case")]
enum SessionEndCause {
    Manual,
    HarnessHook,
}

#[derive(serde::Deserialize)]
struct SessionEndParams {
    session: String,
    cause: SessionEndCause,
}

pub(in crate::daemon::server) async fn rpc_session_end(
    state: &Arc<DaemonState>,
    params: &serde_json::Value,
) -> Result<serde_json::Value> {
    let params: SessionEndParams =
        serde_json::from_value(params.clone()).context("parsing session_end params")?;
    let anchor = CallerAnchor {
        explicit: Some(&params.session),
        ..CallerAnchor::default()
    };
    let Some(session) = resolve_session_inner(state, &anchor, ResolveScope::Strict).ok() else {
        return Ok(serde_json::json!({"ended": false}));
    };
    if matches!(params.cause, SessionEndCause::HarnessHook)
        && endpoint_locator(state, &session)
            .is_some_and(|locator| locator.locator_kind == crate::state::LOCATOR_PTY)
    {
        // The PTY supervisor owns the only atomic observation of child status
        // plus attachment state. A harness hook may arrive just before that
        // observation and must not pre-classify a headed clean exit as headless.
        return Ok(serde_json::json!({"ended": false, "deferred": true}));
    }
    let ended = end_runtime_generation(
        state,
        &session.pubkey,
        session.runtime_generation,
        crate::state::StopReason::HeadlessExit,
    )
    .await?;
    Ok(serde_json::json!({"ended": ended}))
}

/// End exactly one runtime incarnation. The explicit reason prevents endpoint
/// exit, operator stop, and idle eviction from collapsing into one lifecycle.
pub(in crate::daemon::server) async fn end_runtime_generation(
    state: &Arc<DaemonState>,
    pubkey: &str,
    runtime_generation: u64,
    reason: crate::state::StopReason,
) -> Result<bool> {
    let Some(session) = state.with_store(|store| store.get_session(pubkey))? else {
        return Ok(false);
    };
    if session.runtime_generation != runtime_generation || !session.is_running() {
        return Ok(false);
    }
    let ended =
        super::managed_lifecycle::stop_generation(state, &session, reason, now_secs()).await?;
    if ended {
        super::subscriptions::reconcile_subs_logged(state, "session_end").await;
    }
    Ok(ended)
}

#[derive(serde::Deserialize)]
struct SessionKillParams {
    #[serde(default)]
    session: Option<String>,
    #[serde(default)]
    pty_id: Option<String>,
    #[serde(default)]
    forget: bool,
}

pub(in crate::daemon::server) async fn rpc_session_kill(
    state: &Arc<DaemonState>,
    params: &serde_json::Value,
) -> Result<serde_json::Value> {
    let params: SessionKillParams =
        serde_json::from_value(params.clone()).context("parsing session_kill params")?;
    let selector = params
        .session
        .as_deref()
        .filter(|session| !session.is_empty());
    let public = selector
        .map(|session| {
            state.with_store(|store| super::resolution::resolve_public_session(store, session))
        })
        .transpose()?
        .flatten();
    let Some(session) = public else {
        return kill_unbound_endpoint(params.pty_id.as_deref(), params.forget);
    };
    if params.forget {
        return forget_session(state, &session).await;
    }

    let stop = match stop_local_process(state, &session).await {
        Ok(note) => note,
        Err(error) => {
            return Ok(serde_json::json!({
                "killed": false,
                "ended": false,
                "reason": format!("{error:#}"),
            }));
        }
    };
    let transitioned = super::managed_lifecycle::stop_generation(
        state,
        &session,
        crate::state::StopReason::OperatorKill,
        now_secs(),
    )
    .await?;
    let ended = transitioned
        || state
            .with_store(|store| store.get_session(&session.pubkey))?
            .is_some_and(|current| {
                current.runtime_generation == session.runtime_generation && !current.is_running()
            });

    Ok(serde_json::json!({
        "killed": true,
        "ended": ended,
        "note": stop,
        "cleanup_confirmed": true,
        "cleanup_failures": [],
    }))
}

async fn forget_session(
    state: &Arc<DaemonState>,
    selected: &crate::state::Session,
) -> Result<serde_json::Value> {
    // Recovery revocation is the first durable write. The standing lane makes
    // the channel/signing snapshots complete with respect to an admission that
    // was already in flight; future reservations fail on recovery_state.
    let (current, channels, signing_keys) = {
        let _lane = state.standing_sync.lock().await;
        let current = revoke_current_generation(state, &selected.pubkey)?;
        super::engine_lifecycle::cancel_session(state, &current.pubkey, current.runtime_generation);
        let channels = super::membership_cleanup::recorded_channels(state, &current.pubkey);
        let signing_keys = state.session_signing_keys(&current.pubkey);
        (current, channels, signing_keys)
    };

    let stop = match stop_local_process(state, &current).await {
        Ok(note) => note,
        Err(error) => {
            return Ok(serde_json::json!({
                "killed": false,
                "ended": false,
                "recovery_revoked": true,
                "reason": format!(
                    "recovery was revoked, but runtime termination was not confirmed: {error:#}"
                ),
            }));
        }
    };

    let finalized = {
        let _lane = state.standing_sync.lock().await;
        state.with_store(|store| {
            store.finalize_session_recovery_revocation(
                &current.pubkey,
                current.runtime_generation,
                now_secs(),
            )
        })?
    };
    if !finalized {
        return Ok(serde_json::json!({
            "killed": true,
            "ended": false,
            "recovery_revoked": true,
            "note": stop,
            "reason": "runtime terminated, but recovery finalization lost its generation fence",
        }));
    }

    let cleanup_failures = revoke_operator_session(state, &current, signing_keys, channels).await;
    Ok(serde_json::json!({
        "killed": true,
        "ended": true,
        "recovery_revoked": true,
        "note": stop,
        "cleanup_confirmed": cleanup_failures.is_empty(),
        "cleanup_failures": cleanup_failures,
    }))
}

fn revoke_current_generation(
    state: &Arc<DaemonState>,
    pubkey: &str,
) -> Result<crate::state::Session> {
    loop {
        let current = state
            .with_store(|store| store.get_session(pubkey))?
            .with_context(|| format!("session {pubkey} disappeared during recovery revocation"))?;
        if !state.with_store(|store| {
            store.revoke_session_recovery_if_generation(pubkey, current.runtime_generation)
        })? {
            continue;
        }
        let fenced = state
            .with_store(|store| store.get_session(pubkey))?
            .with_context(|| format!("session {pubkey} disappeared after recovery revocation"))?;
        if fenced.runtime_generation == current.runtime_generation
            && fenced.recovery_state == crate::state::RecoveryState::Revoked
        {
            return Ok(fenced);
        }
    }
}

fn kill_unbound_endpoint(pty_id: Option<&str>, forget: bool) -> Result<serde_json::Value> {
    let Some(pty_id) = pty_id else {
        return Ok(serde_json::json!({
            "killed": false, "ended": false, "reason": "no local session matched"
        }));
    };
    let Some(endpoint) = crate::pty::read_all_metadata()
        .into_iter()
        .find(|metadata| metadata.id == pty_id && crate::pty::is_live(&metadata.id))
    else {
        return Ok(serde_json::json!({
            "killed": false, "ended": false, "reason": "no live local endpoint matched"
        }));
    };
    crate::pty::kill(&endpoint.id)
        .with_context(|| format!("killing unbound PTY endpoint {}", endpoint.id))?;
    let cleanup_failures = forget
        .then(|| vec!["endpoint has no session identity; recovery cannot be forgotten".to_string()])
        .unwrap_or_default();
    Ok(serde_json::json!({
        "killed": true,
        "ended": false,
        "note": format!("pty={}", endpoint.id),
        "cleanup_confirmed": cleanup_failures.is_empty(),
        "cleanup_failures": cleanup_failures,
    }))
}

async fn revoke_operator_session(
    state: &Arc<DaemonState>,
    session: &crate::state::Session,
    signing_keys: Result<nostr_sdk::Keys>,
    channels: Vec<String>,
) -> Vec<String> {
    let now = now_secs();
    let mut failures = Vec::new();
    match signing_keys {
        Ok(keys) => {
            crate::status_seam::drive(
                &state.status,
                state.fabric_provider(),
                &keys,
                &state.store,
                crate::status_seam::DriveMeta {
                    trigger: "operator_session_revoke",
                },
                |status| status.on_session_revoked(&session.pubkey, now),
            )
            .await;
        }
        Err(error) => failures.push(format!("status expiration: {error:#}")),
    }
    failures.extend(
        super::membership_cleanup::remove_revoked_session_memberships(
            state,
            &session.pubkey,
            channels,
        )
        .await,
    );
    failures
}

pub(super) async fn stop_local_process(
    state: &Arc<DaemonState>,
    session: &crate::state::Session,
) -> Result<String> {
    if session.runtime_state == crate::state::RuntimeState::Stopped {
        return Ok("runtime already stopped".into());
    }
    if let Some(locator) = endpoint_locator(state, session) {
        match locator.locator_kind.as_str() {
            crate::state::LOCATOR_PTY => {
                if crate::pty::is_live(&locator.locator_value) {
                    crate::pty::kill(&locator.locator_value).with_context(|| {
                        format!("killing PTY endpoint {}", locator.locator_value)
                    })?;
                    wait_for_process_exit(|| !crate::pty::is_live(&locator.locator_value)).await?;
                }
            }
            crate::state::LOCATOR_ACP => {
                use crate::session_host::transport::{EndpointRef, SessionTransport};
                let endpoint = EndpointRef {
                    kind: crate::session_host::transport::TransportKind::Acp,
                    endpoint_id: locator.locator_value.clone(),
                };
                if crate::session_host::transport::AcpTransport.is_live(&endpoint) {
                    crate::session_host::transport::AcpTransport
                        .kill(&endpoint)
                        .await?;
                    wait_for_process_exit(|| {
                        !crate::session_host::transport::AcpTransport.is_live(&EndpointRef {
                            kind: crate::session_host::transport::TransportKind::Acp,
                            endpoint_id: locator.locator_value.clone(),
                        })
                    })
                    .await?;
                }
            }
            _ => unreachable!("endpoint_locator returned a non-runtime locator"),
        }
        state.with_store(|store| {
            store.clear_runtime_locator_if_generation(
                &session.pubkey,
                &locator.locator_kind,
                session.runtime_generation,
            )
        })?;
        return Ok(format!("endpoint={}", locator.locator_value));
    }
    if let Some(pid) = session.child_pid {
        if !super::engine_lifecycle::pid_alive(pid) {
            return Ok(format!("pid={pid} (already exited)"));
        }
        nix::sys::signal::kill(
            nix::unistd::Pid::from_raw(pid),
            Some(nix::sys::signal::Signal::SIGTERM),
        )
        .with_context(|| format!("sending SIGTERM to pid {pid}"))?;
        wait_for_process_exit(|| !super::engine_lifecycle::pid_alive(pid)).await?;
        return Ok(format!("pid={pid}"));
    }
    anyhow::bail!(
        "runtime generation {} for {} has no tracked process endpoint",
        session.runtime_generation,
        session.pubkey
    )
}

async fn wait_for_process_exit(mut exited: impl FnMut() -> bool) -> Result<()> {
    for _ in 0..100 {
        if exited() {
            return Ok(());
        }
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    }
    anyhow::bail!("process termination was not confirmed within 5 seconds")
}

fn endpoint_locator(
    state: &Arc<DaemonState>,
    session: &crate::state::Session,
) -> Option<crate::state::SessionLocator> {
    state
        .with_store(|store| {
            store
                .runtime_locator_for_session(
                    &session.pubkey,
                    session.runtime_generation,
                    crate::state::LOCATOR_PTY,
                )
                .and_then(|pty| match pty {
                    Some(pty) => Ok(Some(pty)),
                    None => store.runtime_locator_for_session(
                        &session.pubkey,
                        session.runtime_generation,
                        crate::state::LOCATOR_ACP,
                    ),
                })
        })
        .ok()?
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::RegisterSession;

    fn registration(pubkey: &str, at: u64) -> RegisterSession {
        RegisterSession {
            pubkey: pubkey.into(),
            harness: "codex".into(),
            agent_slug: "codex".into(),
            channel_h: "root".into(),
            child_pid: None,
            transcript_path: None,
            now: at,
        }
    }

    #[tokio::test]
    async fn forget_fences_generation_reserved_after_selected_runtime_exits() {
        let state = DaemonState::new_for_test().await;
        let pubkey = "pk-forget-race";
        let first = state
            .with_store(|store| store.reserve_session(&registration(pubkey, 1)))
            .unwrap();
        let selected = state
            .with_store(|store| store.get_session(pubkey))
            .unwrap()
            .unwrap();

        state
            .with_store(|store| {
                store.mark_runtime_stopped_if_generation(
                    pubkey,
                    first,
                    crate::state::StopReason::Crash,
                    2,
                )
            })
            .unwrap();
        let second = state
            .with_store(|store| store.reserve_session(&registration(pubkey, 3)))
            .unwrap();

        let fenced = revoke_current_generation(&state, &selected.pubkey).unwrap();
        assert_eq!(fenced.runtime_generation, second);
        assert_eq!(fenced.recovery_state, crate::state::RecoveryState::Revoked);
        assert!(state
            .with_store(|store| store.reserve_session(&registration(pubkey, 4)))
            .is_err());
    }

    #[tokio::test]
    async fn stopped_runtime_needs_no_stale_pid_signal_before_finalize() {
        let state = DaemonState::new_for_test().await;
        let pubkey = "pk-stopped-forget";
        let generation = state
            .with_store(|store| {
                store.reserve_session(&RegisterSession {
                    child_pid: Some(i32::MAX),
                    ..registration(pubkey, 1)
                })
            })
            .unwrap();
        state
            .with_store(|store| {
                store.mark_runtime_stopped_if_generation(
                    pubkey,
                    generation,
                    crate::state::StopReason::Crash,
                    2,
                )
            })
            .unwrap();
        let stopped = state
            .with_store(|store| store.get_session(pubkey))
            .unwrap()
            .unwrap();

        assert_eq!(
            stop_local_process(&state, &stopped).await.unwrap(),
            "runtime already stopped"
        );
    }
}
