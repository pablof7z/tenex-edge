use super::*;

pub(in crate::daemon::server) fn drive_turn_started(
    state: &Arc<DaemonState>,
    session: &crate::state::Session,
    at: u64,
    transcript_ref: Option<String>,
) -> Result<()> {
    let transcript_ref = transcript_ref.or_else(|| session.transcript_path.clone());
    state.with_store(|store| {
        store.apply_turn_projection(&session.pubkey, true, at, transcript_ref.as_deref())
    })
}

pub(in crate::daemon::server) fn drive_turn_ended(
    state: &Arc<DaemonState>,
    session: &crate::state::Session,
) -> Result<()> {
    state.with_store(|store| {
        store.apply_turn_projection(
            &session.pubkey,
            false,
            0,
            session.transcript_path.as_deref(),
        )
    })
}
