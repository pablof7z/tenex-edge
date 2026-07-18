use super::super::*;

/// Set the active publishing channel. When `leave_previous` is true this is the
/// user-facing `channel switch` semantics: leave the previous active channel
/// and join the new one. `channel create` uses `false` so creating a room can
/// move focus into it without dropping the parent from passive context.
pub(in crate::daemon::server) fn set_active_session_channel(
    state: &Arc<DaemonState>,
    pubkey: &str,
    new_channel: &str,
) -> Result<()> {
    state.with_store(|s| -> Result<()> {
        let current = s
            .get_session(pubkey)
            .context("set_active_session_channel: reading current session")?
            .with_context(|| format!("set_active_session_channel: no live session for {pubkey}"))?;
        if !current.is_running()
            || current.recovery_state == crate::state::RecoveryState::Revoked
            || !s.has_session_route(pubkey, new_channel)?
        {
            anyhow::bail!("set_active_session_channel: session lifecycle changed");
        }
        s.set_session_channel(pubkey, new_channel)
            .context("set_active_session_channel: repointing active channel")?;
        Ok(())
    })?;
    Ok(())
}
