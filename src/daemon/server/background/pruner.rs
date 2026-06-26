//! Peer-session pruner: emits `Leave` tail events for peer presences that have
//! expired out of the store. Extracted from `server.rs` (issue #12).

use super::super::*;

/// Every 30s, drop peer sessions older than `PRUNE_PEER_AFTER_SECS` from the
/// store and emit a `Leave` tail event for each `(pubkey, project)` pair that
/// was tracked in memory but is no longer live.
pub fn spawn_pruner(state: Arc<DaemonState>) {
    tokio::spawn(async move {
        let mut tick = tokio::time::interval(Duration::from_secs(30));
        loop {
            tick.tick().await;
            let now = now_secs();
            let before = now.saturating_sub(PRUNE_PEER_AFTER_SECS);

            // Identify which peer sessions will be pruned by checking the map
            // against sessions that are about to expire.
            let tracked_keys: Vec<(String, String)> = {
                let map = state.peer_sessions.lock().unwrap();
                // We'll emit Leave for (pubkey, project) pairs in our map that are
                // no longer live in the store. Cross-reference below.
                map.keys().cloned().collect()
            };

            // Which (pubkey, project) pairs are still live in the store.
            let still_alive: std::collections::HashSet<(String, String)> = state
                .with_store(|s| s.list_peer_sessions(None, before).unwrap_or_default())
                .into_iter()
                .map(|p| (p.pubkey, p.project))
                .collect();

            // Prune from DB.
            state.with_store(|s| {
                let _ = s.prune_peer_sessions(before);
            });

            // Emit Leave for pairs that were in our map but are now expired.
            let to_leave: Vec<((String, String), PeerTracked)> = {
                let mut map = state.peer_sessions.lock().unwrap();
                let expired: Vec<(String, String)> = tracked_keys
                    .into_iter()
                    .filter(|k| !still_alive.contains(k))
                    .collect();
                let mut leaves = Vec::new();
                for key in expired {
                    if let Some(tracked) = map.remove(&key) {
                        leaves.push((key, tracked));
                    }
                }
                leaves
            };
            for ((pubkey, _project), tracked) in to_leave {
                let online_s = now.saturating_sub(tracked.first_seen);
                state.emit_tail(TailEvent::Leave {
                    ts: now,
                    project: tracked.project,
                    agent: tracked.slug,
                    host: tracked.host,
                    session: pubkey,
                    online_s,
                });
            }
        }
    });
}
