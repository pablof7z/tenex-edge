use super::*;

/// Advance a session's awareness cursor exactly once for the observed row.
///
/// The compare-and-update happens inside SQLite, so concurrent hook calls cannot
/// both claim the same delta window.
pub(in crate::daemon::server) fn drive_cursor_request(
    state: &Arc<DaemonState>,
    session: &crate::state::Session,
    at: u64,
    working: bool,
) -> Result<Option<u64>> {
    let before = session.seen_cursor;
    if !working || at <= before {
        return Ok(None);
    }
    let advanced =
        state.with_store(|store| store.advance_cursor_if_current(&session.pubkey, before, at))?;
    Ok(advanced.then_some(before))
}
