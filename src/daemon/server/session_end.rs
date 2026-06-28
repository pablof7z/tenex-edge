use super::*;

#[derive(serde::Deserialize)]
pub(in crate::daemon::server) struct SessionEndParams {
    session: String,
}

pub(in crate::daemon::server) fn rpc_session_end(
    state: &Arc<DaemonState>,
    params: &serde_json::Value,
) -> Result<serde_json::Value> {
    let p: SessionEndParams =
        serde_json::from_value(params.clone()).context("parsing session_end params")?;
    // `get_session` is alias-resolving, so the raw hook id (an alias) yields the
    // canonical session row; every mutation below keys on `rec.session_id`.
    let rec = state.with_store(|s| s.get_session(&p.session).ok().flatten());
    let existed = rec.is_some();
    if let Some(ref rec) = rec {
        cancel_session(state, &rec.session_id);

        // Release the ordinal reservation + any derived signing key before marking
        // the session dead. Fire-and-forget relay removal keeps session_end fast.
        let session_key = state.release_session_signer(&rec.session_id);
        // Mark the bound identity dead but KEEP the row: a later mention to this
        // ordinal resumes its bound native session (issue #47).
        state.with_store(|s| s.mark_identity_dead_for_session(&rec.session_id).ok());
        if let Some(sk) = session_key {
            let provider = state.provider.clone();
            // Remove from the session's CURRENT routing scope (its channel_h) — the
            // group the relay actually has the agent in. Membership is relay-
            // materialized, so we only issue the relay remove; the 39002 reflection
            // updates the local member cache.
            let scope = rec.channel_h.clone();
            let session_pubkey = sk.public_key().to_hex();
            tokio::spawn(async move {
                let removed = provider.nip29_remove_member(&scope, &session_pubkey).await;
                if std::env::var("TENEX_EDGE_DEBUG").is_ok() {
                    eprintln!(
                        "[daemon] session-end NIP-29 remove {}: {}",
                        crate::util::pubkey_short(&session_pubkey),
                        if removed {
                            "accepted"
                        } else {
                            "skipped/failed (best-effort)"
                        },
                    );
                }
            });
        }

        // Mark the canonical session dead (alive=0, working=0). Its final published
        // kind:30315 ages off via NIP-40 expiration.
        state.with_store(|s| s.mark_dead(&rec.session_id).ok());
        state.outbox_notify.notify_waiters();
        state.emit_tail(TailEvent::Sess {
            ts: now_secs(),
            project: rec.channel_h.clone(),
            agent: rec.agent_slug.clone(),
            session: rec.session_id.clone(),
            state: "end".into(),
            rel_cwd: String::new(),
        });
    }
    Ok(serde_json::json!({ "ended": existed }))
}
