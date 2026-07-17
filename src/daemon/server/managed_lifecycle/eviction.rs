use super::*;
use crate::session_host::transport::{EndpointRef, SessionTransport, TransportKind};

pub(super) async fn evict_due_idle_sessions(state: &Arc<DaemonState>) {
    let candidates = state
        .with_store(|store| store.list_due_idle_evictions(now_secs()))
        .unwrap_or_default();
    for candidate in candidates {
        if let Err(error) = evict_one(state, candidate).await {
            tracing::warn!(%error, "idle session eviction failed safely");
        }
    }
}

pub(super) async fn reconcile_stopping(state: &Arc<DaemonState>) {
    let sessions = state
        .with_store(|store| store.list_stopping_sessions())
        .unwrap_or_default();
    for session in sessions {
        if session.stop_reason != Some(StopReason::IdleEvicted) {
            tracing::error!(
                pubkey = %session.pubkey,
                reason = ?session.stop_reason,
                "recovering stopping session with invalid ownership marker"
            );
        }
        if let Err(error) = finish_idle_eviction(state, session).await {
            tracing::warn!(%error, "stopping session reconciliation remains retryable");
        }
    }
}

async fn evict_one(state: &Arc<DaemonState>, candidate: Session) -> Result<()> {
    let Some(stopping) = state.with_store(|store| {
        store.reserve_due_idle_eviction(
            &candidate.pubkey,
            candidate.runtime_generation,
            candidate.lifecycle_epoch,
            candidate.attachment_epoch,
            now_secs(),
        )
    })?
    else {
        return Ok(());
    };
    finish_idle_eviction(state, stopping).await
}

async fn finish_idle_eviction(state: &Arc<DaemonState>, stopping: Session) -> Result<()> {
    let endpoint = runtime_endpoint(state, &stopping);
    match endpoint.as_ref().map(|(kind, id)| (*kind, id.as_str())) {
        Some((TransportKind::Pty, id)) => {
            match crate::pty::kill_if_headless_at(id, stopping.attachment_epoch) {
                Ok(crate::pty::ConditionalKillOutcome::Killed { .. }) => {}
                Ok(crate::pty::ConditionalKillOutcome::PresentationChanged { presentation }) => {
                    if !super::presentation::apply(state, &stopping, presentation)? {
                        ensure_not_stuck_stopping(state, &stopping)?;
                    }
                    return Ok(());
                }
                Err(error) => {
                    if crate::pty::is_live(id) {
                        let cancelled = state.with_store(|store| {
                            store.cancel_idle_eviction_on_presentation_change(
                                &stopping.pubkey,
                                stopping.runtime_generation,
                                stopping.lifecycle_epoch,
                                stopping.attachment_epoch,
                                PresentationState::Unavailable,
                                now_secs(),
                            )
                        })?;
                        if !cancelled {
                            ensure_not_stuck_stopping(state, &stopping)?;
                        }
                        return Err(error.into());
                    }
                    // A closed socket is not proof that the runtime is gone. Use
                    // the persisted supervisor identity, including its instance
                    // token, to confirm exit or terminate it before committing
                    // the durable stopped edge. Missing metadata is safe only
                    // when the recorded supervisor PID is already absent.
                    match crate::pty::terminate_owned_supervisor(id) {
                        Ok(true) => {}
                        Ok(false)
                            if stopping
                                .child_pid
                                .is_some_and(super::super::engine_lifecycle::pid_alive) =>
                        {
                            return Err(error.into());
                        }
                        Ok(false) => {}
                        Err(ownership_error) => return Err(ownership_error),
                    }
                }
            }
        }
        Some((TransportKind::Acp, id)) => {
            crate::session_host::transport::AcpTransport
                .kill(&EndpointRef {
                    kind: TransportKind::Acp,
                    endpoint_id: id.to_string(),
                })
                .await?;
        }
        None => {
            if stopping
                .child_pid
                .is_some_and(super::super::engine_lifecycle::pid_alive)
            {
                anyhow::bail!(
                    "refusing to signal an unbound live process without an owned runtime endpoint"
                );
            }
        }
    }
    let stopped_at = now_secs();
    cancel_session(state, &stopping.pubkey, stopping.runtime_generation);
    let stopped = state.with_store(|store| {
        store.finalize_runtime_stopped_if_epoch(
            &stopping.pubkey,
            stopping.runtime_generation,
            stopping.lifecycle_epoch,
            StopReason::IdleEvicted,
            stopped_at,
        )
    })?;
    if let Some(stopped) = stopped {
        state.with_store(|store| {
            store.clear_runtime_locator_if_generation(
                &stopped.pubkey,
                match endpoint.as_ref().map(|(kind, _)| kind) {
                    Some(TransportKind::Pty) => crate::state::LOCATOR_PTY,
                    Some(TransportKind::Acp) => crate::state::LOCATOR_ACP,
                    None => crate::state::LOCATOR_PID,
                },
                stopped.runtime_generation,
            )
        })?;
        super::emit_stopped(state, &stopped, stopped_at);
    }
    Ok(())
}

fn ensure_not_stuck_stopping(state: &Arc<DaemonState>, expected: &Session) -> Result<()> {
    let current = state.with_store(|store| store.get_session(&expected.pubkey))?;
    if current.is_some_and(|session| {
        session.runtime_generation == expected.runtime_generation
            && session.lifecycle_epoch == expected.lifecycle_epoch
            && session.runtime_state == RuntimeState::Stopping
    }) {
        anyhow::bail!("stopping lifecycle edge was not cancelled")
    }
    Ok(())
}

fn runtime_endpoint(
    state: &Arc<DaemonState>,
    session: &Session,
) -> Option<(TransportKind, String)> {
    state
        .with_store(|store| store.locators_for_pubkey(&session.pubkey))
        .ok()?
        .into_iter()
        .filter(|locator| locator.runtime_generation == session.runtime_generation)
        .find_map(|locator| match locator.locator_kind.as_str() {
            crate::state::LOCATOR_PTY => Some((TransportKind::Pty, locator.locator_value)),
            crate::state::LOCATOR_ACP => Some((TransportKind::Acp, locator.locator_value)),
            _ => None,
        })
}
