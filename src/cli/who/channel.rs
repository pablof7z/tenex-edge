use super::*;

pub(super) fn channel_status_map(
    store: &Store,
    channel: &str,
    now: u64,
) -> std::collections::HashMap<String, crate::session::DerivedStatus> {
    let since = now.saturating_sub(crate::session::STATUS_TTL_SECS);
    let mut map = std::collections::HashMap::new();
    // Peers first so a local session of the same agent overrides it.
    for snap in store
        .peer_session_snapshots(Some(channel), since)
        .unwrap_or_default()
    {
        map.insert(
            snap.agent_pubkey.clone(),
            crate::session::derive_status(&snap, now),
        );
    }
    for snap in store
        .live_session_snapshots(Some(channel), since)
        .unwrap_or_default()
    {
        let pubkey = store
            .session_pubkey_for_session(snap.session_id.as_str())
            .unwrap_or_else(|| snap.agent_pubkey.clone());
        map.insert(pubkey, crate::session::derive_status(&snap, now));
    }
    map
}
