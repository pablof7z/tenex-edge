//! Idle-exit watcher: shuts the daemon down after it has had no open clients
//! and no live sessions for a grace period. Extracted from `server.rs`
//! (issue #12).

use super::super::*;

/// Grace window before an idle daemon exits. Overridable via
/// `TENEX_EDGE_DAEMON_GRACE_S` (default 120s).
fn grace() -> Duration {
    Duration::from_secs(env_u64("TENEX_EDGE_DAEMON_GRACE_S", 120))
}

pub fn spawn_idle_watcher(state: Arc<DaemonState>) {
    tokio::spawn(async move {
        loop {
            state.liveness_changed.notified().await;
            if is_idle(&state) {
                let grace_secs = grace().as_secs();
                tracing::info!(grace_secs, "daemon idle; starting grace-period countdown");
                tokio::select! {
                    _ = tokio::time::sleep(grace()) => {
                        if is_idle(&state) {
                            tracing::info!("grace period elapsed; daemon exiting");
                            state.shutdown.notify_waiters();
                            return;
                        }
                    }
                    _ = state.liveness_changed.notified() => {}
                }
            }
        }
    });
}

fn is_idle(state: &Arc<DaemonState>) -> bool {
    *state.open_clients.lock().unwrap() == 0 && state.live_session_count() == 0
}
