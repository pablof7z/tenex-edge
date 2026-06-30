use super::*;
use crate::session::{DerivedStatus, Liveness};
use crate::state::Status;

/// Live status for every agent in `channel`, keyed by signing pubkey. Both local
/// and remote agents read identically out of `relay_status` now — the daemon
/// publishes its own kind:30315 like everyone else, so there is no local-vs-peer
/// fork. `live_status_for_channel` already drops NIP-40-expired rows, so every
/// returned row is live.
pub(super) fn channel_status_map(
    store: &Store,
    channel: &str,
    now: u64,
) -> std::collections::HashMap<String, DerivedStatus> {
    match store.live_status_for_channel(channel, now) {
        Ok(rows) => rows
            .into_iter()
            .map(|s| (s.pubkey.clone(), derive_from_status(&s, now)))
            .collect(),
        // A read failure must not silently render as an empty channel ("no
        // agents present"): log loudly and return empty so the gap is visible
        // in the logs rather than masquerading as a quiet channel.
        Err(e) => {
            tracing::error!(
                channel = %channel,
                error = ?e,
                "awareness: live status read failed; channel may falsely appear empty"
            );
            std::collections::HashMap::new()
        }
    }
}

/// Project a relay-confirmed [`Status`] row into the shared [`DerivedStatus`]
/// view every reader renders. A row returned by `live_status_for_channel` is
/// live by construction (expiration >= now); `activity` is suppressed when the
/// agent is not busy so an idle session never shows a stale "doing now" line.
pub(super) fn derive_from_status(s: &Status, now: u64) -> DerivedStatus {
    DerivedStatus {
        busy: s.busy,
        liveness: Liveness::Live,
        title: s.title.clone(),
        activity: if s.busy {
            s.activity.clone()
        } else {
            String::new()
        },
        lifecycle: crate::domain::Lifecycle::Active,
        age_secs: now.saturating_sub(s.last_seen),
    }
}
