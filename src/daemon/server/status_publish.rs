//! Generic outbox drainer. The per-session status **decision** now lives in the
//! Trellis [`crate::reconcile::status`] reconciler (the single change-only publish
//! authority); the runtime engine feeds it inputs and enqueues the emitted
//! publish/expire effects here. The old second heartbeat timer
//! (`spawn_status_heartbeat_publisher`, a direct `set_status` path with no dedup)
//! was DELETED — TTL re-arm is now the reconciler's `on_tick`, published through
//! this one outbox executor.

use super::*;

#[cfg(test)]
mod tests;

/// Drain the generic outbox: publish each queued signed event JSON via the
/// transport with a checked relay verdict, marking it published only after a
/// relay accepts it. Failed attempts stay pending with an error and retry count.
/// Woken instantly by `outbox_notify` on every enqueue, and polled every 2s as a
/// fallback.
pub(in crate::daemon::server) fn spawn_outbox_drainer(state: Arc<DaemonState>) {
    use nostr_sdk::prelude::{Event, JsonUtil};
    tokio::spawn(async move {
        loop {
            // Publish the backlog while we keep making progress (so a startup
            // burst clears fast); stop if a whole batch failed to avoid a tight spin.
            loop {
                let items = state.with_store(|s| s.peek_outbox(32).unwrap_or_default());
                if items.is_empty() {
                    break;
                }
                let mut progressed = false;
                for item in items {
                    match Event::from_json(&item.event_json) {
                        Ok(ev) => match state.transport.publish_event_checked(&ev).await {
                            Ok(_) => {
                                state.with_store(|s| s.mark_published(item.local_id).ok());
                                progressed = true;
                            }
                            Err(e) => {
                                state.with_store(|s| {
                                    s.mark_failed(item.local_id, &format!("{e:#}")).ok()
                                });
                            }
                        },
                        Err(e) => {
                            // A row we can never parse is dead; record the error.
                            // It stays pending but won't block other rows.
                            state.with_store(|s| {
                                s.mark_failed(item.local_id, &format!("bad event json: {e}"))
                                    .ok()
                            });
                        }
                    }
                }
                if !progressed {
                    break;
                }
            }
            tokio::select! {
                _ = state.outbox_notify.notified() => {}
                _ = tokio::time::sleep(Duration::from_secs(2)) => {}
            }
        }
    });
}

// ── session lifecycle ─────────────────────────────────────────────────────────
