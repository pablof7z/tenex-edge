use super::*;

pub(in crate::daemon::server) fn drive_turn_started(
    state: &Arc<DaemonState>,
    session: &crate::state::Session,
    at: u64,
    transcript_ref: Option<String>,
) -> Result<()> {
    let transcript_ref = transcript_ref.or_else(|| session.transcript_path.clone());
    let applied = state.with_store(|store| {
        store.apply_session_turn_started(
            &session.pubkey,
            session.runtime_generation,
            at,
            transcript_ref.as_deref(),
        )
    })?;
    if !applied {
        anyhow::bail!("turn_start lost runtime generation for {}", session.pubkey);
    }
    Ok(())
}

pub(in crate::daemon::server) fn drive_turn_ended(
    state: &Arc<DaemonState>,
    session: &crate::state::Session,
    at: u64,
) -> Result<()> {
    let applied = state.with_store(|store| {
        store.apply_session_turn_ended(&session.pubkey, session.runtime_generation, at)
    })?;
    if !applied {
        anyhow::bail!("turn_end lost runtime generation for {}", session.pubkey);
    }
    Ok(())
}
