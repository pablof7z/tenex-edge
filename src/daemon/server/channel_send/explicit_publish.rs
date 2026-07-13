use super::super::*;

pub(super) fn note_explicit_chat_published(state: &Arc<DaemonState>, pubkey: &str, at: u64) {
    if let Err(e) = state.with_store(|s| s.mark_session_explicit_chat_published(pubkey, at)) {
        tracing::warn!(
            pubkey,
            error = %e,
            "channel_send: failed to persist explicit-publish marker; using in-memory auto-reply guard"
        );
    }
    auto_reply::note_explicit_publish(pubkey);
}
