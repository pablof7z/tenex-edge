use super::*;
use crate::state::ConfirmedAdmissionCommit;

/// Finalize relay-confirmed membership while the caller holds `standing_sync`.
/// A stale or failed primary commit first becomes durable cleanup work, so an
/// unconfirmed compensation is retried by the standing coordinator.
pub(in crate::daemon::server) async fn commit_confirmed_admission(
    state: &Arc<DaemonState>,
    pubkey: &str,
    channel: &str,
    runtime_generation: u64,
    lifecycle_epoch: u64,
) -> Result<bool> {
    let now = now_secs();
    let primary = state.with_store(|store| {
        store.commit_confirmed_session_admission(
            pubkey,
            channel,
            runtime_generation,
            lifecycle_epoch,
            now,
        )
    });
    match primary {
        Ok(ConfirmedAdmissionCommit::Committed) => Ok(true),
        Ok(ConfirmedAdmissionCommit::Superseded) => {
            tracing::warn!(pubkey = %pubkey_short(pubkey), %channel, lifecycle_epoch, "stale admission was superseded by newer member standing");
            Ok(false)
        }
        Ok(ConfirmedAdmissionCommit::CleanupDue(due)) => {
            compensate_due_admission(state, &due).await;
            Ok(false)
        }
        Err(primary_error) => {
            let fallback = state.with_store(|store| {
                store.schedule_confirmed_admission_cleanup(
                    pubkey,
                    channel,
                    runtime_generation,
                    lifecycle_epoch,
                    now_secs(),
                )
            });
            match fallback {
                Ok(ConfirmedAdmissionCommit::Committed) => {
                    tracing::warn!(pubkey = %pubkey_short(pubkey), %channel, %primary_error, "admission commit reported an error but its exact durable state is present");
                    Ok(true)
                }
                Ok(ConfirmedAdmissionCommit::Superseded) => {
                    tracing::warn!(pubkey = %pubkey_short(pubkey), %channel, %primary_error, "failed admission commit was superseded by newer member standing");
                    Ok(false)
                }
                Ok(ConfirmedAdmissionCommit::CleanupDue(due)) => {
                    compensate_due_admission(state, &due).await;
                    Err(primary_error).context("confirmed admission could not be committed")
                }
                Err(cleanup_error) => Err(anyhow::anyhow!(
                    "confirmed admission commit failed ({primary_error:#}); durable cleanup persistence also failed ({cleanup_error:#})"
                )),
            }
        }
    }
}

async fn compensate_due_admission(state: &Arc<DaemonState>, due: &crate::state::SessionStanding) {
    let removal = state
        .provider
        .remove_member_confirmed(&due.channel_h, &due.pubkey)
        .await;
    if !removal.is_confirmed() {
        tracing::warn!(
            pubkey = %pubkey_short(&due.pubkey),
            channel = %due.channel_h,
            ?removal,
            "admission compensation remains durably due"
        );
        return;
    }
    match state.with_store(|store| {
        store.mark_session_standing_absent_if_epoch(
            &due.pubkey,
            &due.channel_h,
            due.state,
            due.standing_epoch,
            due.session_lifecycle_epoch,
            now_secs(),
        )
    }) {
        Ok(true) => tracing::info!(
            pubkey = %pubkey_short(&due.pubkey),
            channel = %due.channel_h,
            "stale confirmed admission was removed"
        ),
        Ok(false) => tracing::debug!(
            pubkey = %pubkey_short(&due.pubkey),
            channel = %due.channel_h,
            "admission compensation was superseded while removal completed"
        ),
        Err(error) => tracing::error!(
            pubkey = %pubkey_short(&due.pubkey),
            channel = %due.channel_h,
            %error,
            "confirmed admission removal could not be persisted; cleanup remains due"
        ),
    }
}

pub(super) async fn reconcile_expired(state: &Arc<DaemonState>) {
    let _lane = state.standing_sync.lock().await;
    let mut changed = false;
    let due = match state.with_store(|store| store.list_due_retained_standing(now_secs())) {
        Ok(due) => due,
        Err(error) => {
            tracing::error!(%error, "standing reconciliation scan failed");
            return;
        }
    };
    for standing in due {
        let current = state
            .with_store(|store| store.get_session_standing(&standing.pubkey, &standing.channel_h))
            .ok()
            .flatten();
        if current.as_ref() != Some(&standing) {
            continue;
        }
        let outcome = state
            .provider
            .remove_member_confirmed(&standing.channel_h, &standing.pubkey)
            .await;
        if !outcome.is_confirmed() {
            tracing::warn!(
                pubkey = %pubkey_short(&standing.pubkey),
                channel = %standing.channel_h,
                outcome = ?outcome,
                "standing removal was not confirmed; retaining retry deadline"
            );
            continue;
        }
        if let Err(error) = state.with_store(|store| {
            store.mark_session_standing_absent_if_epoch(
                &standing.pubkey,
                &standing.channel_h,
                standing.state,
                standing.standing_epoch,
                standing.session_lifecycle_epoch,
                now_secs(),
            )
        }) {
            tracing::error!(%error, "confirmed standing removal could not be persisted");
        } else {
            changed = true;
        }
    }
    drop(_lane);
    if changed {
        super::super::subscriptions::reconcile_subs_logged(state, "standing_expired").await;
    }
}

pub(super) async fn reconcile_running(state: &Arc<DaemonState>) {
    let sessions = state.with_store(|store| store.list_running_sessions().unwrap_or_default());
    for session in sessions {
        let routes = state
            .with_store(|store| store.list_session_routes(&session.pubkey))
            .unwrap_or_default();
        for (channel, _) in routes {
            let member = state
                .with_store(|store| store.get_session_standing(&session.pubkey, &channel))
                .ok()
                .flatten()
                .is_some_and(|standing| standing.state == crate::state::StandingState::Member);
            if member {
                continue;
            }
            repair_one(state, &session, &channel).await;
        }
    }
}

async fn repair_one(state: &Arc<DaemonState>, session: &Session, channel: &str) {
    let _lane = state.standing_sync.lock().await;
    let confirmed = if channel == session.channel_h {
        let parent =
            (!session.readiness_parent.is_empty()).then_some(session.readiness_parent.as_str());
        matches!(
            tokio::time::timeout(
                Duration::from_secs(15),
                state.provider.ensure_channel_ready(
                    crate::fabric::nip29::readiness::ChannelCtx {
                        channel,
                        expect_member: &session.pubkey,
                        parent_hint: parent,
                        name: None,
                        repair_whitelisted_admins: true,
                    },
                ),
            )
            .await,
            Ok(gate) if !matches!(gate, crate::fabric::nip29::readiness::ChannelGate::Degraded)
        )
    } else {
        state
            .provider
            .grant_member_confirmed(channel, &session.pubkey)
            .await
            .is_confirmed()
    };
    if !confirmed {
        tracing::warn!(pubkey = %session.pubkey, %channel, "running session standing remains retryable");
        return;
    }
    match commit_confirmed_admission(
        state,
        &session.pubkey,
        channel,
        session.runtime_generation,
        session.lifecycle_epoch,
    )
    .await
    {
        Ok(true) => {}
        Ok(false) => {
            tracing::warn!(pubkey = %session.pubkey, %channel, "running-standing repair became stale")
        }
        Err(error) => {
            tracing::error!(pubkey = %session.pubkey, %channel, %error, "running-standing repair persistence failed")
        }
    }
}
