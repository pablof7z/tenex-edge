use super::*;

pub(super) fn started(
    state: &Arc<DaemonState>,
    session_id: &str,
    channel_h: &str,
    agent_pubkey: &str,
    pid: Option<i32>,
    source: &'static str,
) {
    apply(
        state,
        crate::reconcile::InputFact::SessionStarted {
            session_id: session_id.to_string(),
            channel_h: Some(channel_h.to_string()),
            agent_pubkey: Some(agent_pubkey.to_string()),
            pid,
            at: now_secs(),
        },
        source,
    );
}

pub(super) fn exited(
    state: &Arc<DaemonState>,
    session_id: &str,
    pid: Option<i32>,
    source: &'static str,
) {
    exited_at(state, session_id, pid, now_secs(), source);
}

pub(super) fn exited_at(
    state: &Arc<DaemonState>,
    session_id: &str,
    pid: Option<i32>,
    at: u64,
    source: &'static str,
) {
    apply(
        state,
        crate::reconcile::InputFact::ProcessExited {
            session_id: Some(session_id.to_string()),
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
