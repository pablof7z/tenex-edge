use super::*;

pub(in crate::daemon::server) fn status_from_snapshot(
    snap: &SessionSnapshot,
    now: u64,
) -> crate::domain::Status {
    let d = derive_status(snap, now);
    crate::domain::Status {
        agent: crate::domain::AgentRef::new(snap.agent_pubkey.clone(), snap.agent_slug.clone()),
        project: snap.project.clone(),
        session_id: snap.session_id.clone(),
        host: snap.host.clone(),
        title: snap.title.clone(),
        activity: d.activity,
        busy: d.busy,
        rel_cwd: snap.rel_cwd.clone(),
        expires_at: Some(now + crate::domain::STATUS_TTL_SECS),
    }
}

/// Heartbeat re-arm: every `HEARTBEAT_SECS`, re-publish the current kind:30315 for
/// every live locally-hosted session so its NIP-40 `expiration` is pushed forward
/// to `now + STATUS_TTL_SECS`. The outbox only fires on state CHANGES; a live-but-
/// idle session produces none, so without this its relay event would expire after
/// `STATUS_TTL_SECS` and read as gone despite the runtime heartbeating `last_seen`
/// locally. This is the piece that turns store-side freshness into relay liveness.
pub(in crate::daemon::server) fn spawn_status_heartbeat_publisher(state: Arc<DaemonState>) {
    tokio::spawn(async move {
        let mut tick = tokio::time::interval(Duration::from_secs(crate::domain::HEARTBEAT_SECS));
        loop {
            tick.tick().await;
            let now = now_secs();
            let fresh_since = now.saturating_sub(crate::domain::STATUS_TTL_SECS);
            let snaps =
                state.with_store(|s| s.all_live_local_snapshots(fresh_since).unwrap_or_default());
            for snap in snaps {
                // Sign with the selected session identity so a heartbeat cannot
                // collapse two duplicate sessions back onto the durable author.
                let keys = match state
                    .keys_for_session(snap.session_id.as_str())
                    .or_else(|| state.keys_for(&snap.agent_pubkey))
                {
                    Some(k) => k,
                    None => continue,
                };
                let status = status_from_snapshot(&snap, now);
                if let Ok(eid) = state.provider.set_status(&status, &keys).await {
                    let signer_pubkey = keys.public_key().to_hex();
                    state.with_store(|s| {
                        s.confirm_local_presence(&snap, &signer_pubkey, &eid.to_hex(), now)
                            .ok();
                    });
                }
            }
        }
    });
}

/// Drain the `status_outbox`: publish each pending kind:30315 via the provider's
/// `set_status`, recording the native event id (or a retryable failure). Woken
/// instantly by `status_outbox_notify` on every transition, and polled every 2s
/// as a fallback for transitions enqueued by the runtime (distill/seed/heartbeat).
pub(in crate::daemon::server) fn spawn_status_outbox_drainer(state: Arc<DaemonState>) {
    tokio::spawn(async move {
        loop {
            // Drain the backlog while we keep making progress (so a startup burst
            // clears fast); stop if a whole batch failed to avoid a tight spin.
            loop {
                let items = state.with_store(|s| s.pending_status_outbox(32).unwrap_or_default());
                if items.is_empty() {
                    break;
                }
                let mut progressed = false;
                for item in items {
                    let now = now_secs();
                    // Only locally-hosted agents have signing keys; a row for an
                    // unhosted agent can never publish — record and skip it.
                    // Sign with the session key for transient duplicates; fall
                    // back to the durable agent key for the default signer.
                    let keys = match state
                        .keys_for_session(&item.session_id)
                        .or_else(|| state.keys_for(&item.snapshot.agent_pubkey))
                    {
                        Some(k) => k,
                        None => {
                            state.with_store(|s| {
                                s.mark_status_failed(
                                    &item.session_id,
                                    item.state_version,
                                    "no signing keys for agent",
                                )
                                .ok();
                            });
                            continue;
                        }
                    };
                    let status = status_from_snapshot(&item.snapshot, now);
                    match state.provider.set_status(&status, &keys).await {
                        Ok(eid) => {
                            let signer_pubkey = keys.public_key().to_hex();
                            state.with_store(|s| {
                                s.mark_status_published(
                                    &item.session_id,
                                    item.state_version,
                                    &eid.to_hex(),
                                )
                                .ok();
                                s.confirm_local_presence(
                                    &item.snapshot,
                                    &signer_pubkey,
                                    &eid.to_hex(),
                                    now,
                                )
                                .ok();
                            });
                            progressed = true;
                        }
                        Err(e) => {
                            if std::env::var("TENEX_EDGE_DEBUG").is_ok() {
                                eprintln!(
                                    "[daemon] status publish failed for {}: {e:#}",
                                    item.session_id
                                );
                            }
                            state.with_store(|s| {
                                s.mark_status_failed(
                                    &item.session_id,
                                    item.state_version,
                                    &format!("{e:#}"),
                                )
                                .ok();
                            });
                        }
                    }
                }
                if !progressed {
                    break;
                }
            }
            tokio::select! {
                _ = state.status_outbox_notify.notified() => {}
                _ = tokio::time::sleep(Duration::from_secs(2)) => {}
            }
        }
    });
}

// ── session lifecycle ─────────────────────────────────────────────────────────
