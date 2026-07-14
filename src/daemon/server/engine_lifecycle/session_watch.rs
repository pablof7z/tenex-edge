use super::*;

pub(super) fn started(
    state: &Arc<DaemonState>,
    pubkey: &str,
    channel_h: &str,
    pid: Option<i32>,
    source: &'static str,
) {
    apply(
        state,
        crate::reconcile::InputFact::SessionStarted {
            pubkey: pubkey.to_string(),
            channel_h: Some(channel_h.to_string()),
            pid,
            at: now_secs(),
        },
        source,
    );
}

pub(super) fn exited(
    state: &Arc<DaemonState>,
    pubkey: &str,
    pid: Option<i32>,
    source: &'static str,
) {
    exited_at(state, pubkey, pid, now_secs(), source);
}

pub(super) fn exited_at(
    state: &Arc<DaemonState>,
    pubkey: &str,
    pid: Option<i32>,
    at: u64,
    source: &'static str,
) {
    apply(
        state,
        crate::reconcile::InputFact::ProcessExited {
            pubkey: Some(pubkey.to_string()),
            pid: pid.unwrap_or(0),
            at,
        },
        source,
    );
}

fn apply(state: &Arc<DaemonState>, fact: crate::reconcile::InputFact, source: &'static str) {
    if let Err(e) = state
        .session_watch
        .lock()
        .expect("session_watch mutex poisoned")
        .apply(&fact)
    {
        tracing::warn!(source, error = ?e, "session_watch fact application failed");
    }
}
