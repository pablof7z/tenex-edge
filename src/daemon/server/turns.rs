use super::*;

#[derive(serde::Deserialize, Default)]
pub(in crate::daemon::server) struct TurnStartParams {
    session: String,
    #[serde(default)]
    transcript: Option<String>,
}

pub(in crate::daemon::server) async fn rpc_turn_start(
    state: &Arc<DaemonState>,
    params: &serde_json::Value,
) -> Result<serde_json::Value> {
    let p: TurnStartParams =
        serde_json::from_value(params.clone()).context("parsing turn_start params")?;
    if p.session.is_empty() {
        return Ok(serde_json::json!({ "context": serde_json::Value::Null }));
    }
    // Hooks speak the harness id; resolve to the canonical session_state id or the
    // transition below updates ZERO rows (harness id is only an alias). This is the
    // single owner of the turn-start transition — the runtime engine only OBSERVES
    // turn_state and never opens/closes the turn itself.
    let session = state.with_store(|s| s.canonical_session_id(&p.session));

    let prev_started = state.with_store(|s| {
        let (_, prev) = s.get_turn_state(&session).unwrap_or((false, 0));
        // Canonical transition: busy=1, turn_id+1, activity cleared, version bump +
        // status_outbox enqueue. Also writes turn_state so turn_check_due() works.
        s.start_turn(&session, now_secs()).ok();
        if let Some(path) = p.transcript.as_deref().filter(|x| !x.is_empty()) {
            s.set_session_transcript(&session, path).ok();
            // Snapshot the last assistant text so rpc_turn_end can poll until a
            // *new* (different) response appears — Claude Code writes the
            // transcript after the stop hook fires, so reading at stop time often
            // returns the previous turn's content.
            let baseline = crate::transcript::read_last_assistant_text(std::path::Path::new(path))
                .unwrap_or_default();
            s.set_last_assistant_text_at_turn_start(&session, &baseline)
                .ok();
        }
        prev
    });
    state.status_outbox_notify.notify_waiters();

    let rec = match state.with_store(|s| s.get_session(&session).ok().flatten()) {
        Some(r) => r,
        None => return Ok(serde_json::json!({ "context": serde_json::Value::Null })),
    };

    // Emit Turn{working} for the live tail feed. Key on the routing scope
    // (channel when set, else the per-session room) so the tail reflects the
    // room the session actually publishes into after a `channels switch`.
    state.emit_tail(TailEvent::Turn {
        ts: now_secs(),
        project: rec.route_scope().to_string(),
        agent: rec.agent_slug.clone(),
        session: rec.session_id.clone(),
        state: "working".into(),
        elapsed_s: None,
    });

    // Warm the kind:0 profile cache for identities the synchronous context
    // renderer needs to name. Chat senders and entity mentions are message-local;
    // group members are roster-local and otherwise fall back to raw pubkey shorts
    // in the first-turn awareness block.
    let to_warm: Vec<String> = state.with_store(|s| {
        let mut v: Vec<String> = Vec::new();
        for r in s.peek_chat(&session).unwrap_or_default() {
            if r.from_slug.is_empty() {
                v.push(r.from_pubkey);
            }
            // Also names referenced by `nostr:npub1…` mentions in the body.
            v.extend(crate::profile::body_mention_pubkeys(&r.body));
        }
        for (pubkey, _) in s.list_group_members(rec.route_scope()).unwrap_or_default() {
            v.push(pubkey);
        }
        v.sort();
        v.dedup();
        v
    });
    crate::profile::warm(state, &to_warm).await;

    // Assemble via the SHARED cli.rs function so the injected text is byte-identical
    // to the pre-daemon CLI and cannot drift.
    let context = crate::cli::assemble_turn_start_context(&state.store, &rec, prev_started)
        .map(serde_json::Value::String)
        .unwrap_or(serde_json::Value::Null);
    Ok(serde_json::json!({ "context": context }))
}

#[derive(serde::Deserialize, Default)]
pub(in crate::daemon::server) struct TurnCheckParams {
    #[serde(default)]
    session: Option<String>,
    #[serde(default)]
    env_session: Option<String>,
    #[serde(default)]
    cwd: Option<String>,
    #[serde(default)]
    agent: Option<String>,
    #[serde(default)]
    group: Option<String>,
}

pub(in crate::daemon::server) fn rpc_turn_check(
    state: &Arc<DaemonState>,
    params: &serde_json::Value,
) -> Result<serde_json::Value> {
    let p: TurnCheckParams = serde_json::from_value(params.clone()).unwrap_or_default();
    let rec = resolve_session(
        state,
        p.session.as_deref(),
        p.env_session.as_deref(),
        p.cwd.as_deref(),
        p.agent.as_deref(),
        p.group.as_deref(),
    )?;
    let now = now_secs();
    // Rate-limit the sibling-session delta to at most once per 60s per session
    // (the cursor write is safe: the daemon is the single store writer). `None`
    // → the floor hasn't passed (or not mid-turn), so only the inbox peek runs.
    let delta_since =
        state.with_store(|s| s.turn_check_due(&rec.session_id, now, 60).unwrap_or(None));
    let context =
        crate::cli::assemble_turn_check_context(&state.store, &rec, &state.host, delta_since, now)
            .map(serde_json::Value::String)
            .unwrap_or(serde_json::Value::Null);
    Ok(serde_json::json!({ "context": context }))
}

#[derive(serde::Deserialize)]
pub(in crate::daemon::server) struct TurnEndParams {
    session: String,
    /// The agent's turn output (last assistant text), read from the transcript
    /// by the stop hook. Published as kind:9 chat into the session's room.
    #[serde(default)]
    reply: Option<String>,
}

pub(in crate::daemon::server) async fn rpc_turn_end(
    state: &Arc<DaemonState>,
    params: &serde_json::Value,
) -> Result<serde_json::Value> {
    let p: TurnEndParams =
        serde_json::from_value(params.clone()).context("parsing turn_end params")?;
    if !p.session.is_empty() {
        // Hooks speak the harness id; resolve to canonical or end_turn no-ops.
        // Single owner of the turn-end transition (runtime only observes).
        let session = state.with_store(|s| s.canonical_session_id(&p.session));
        // Read turn_started_at BEFORE marking end, so we can compute elapsed.
        let (was_working, turn_started_at) =
            state.with_store(|s| s.get_turn_state(&session).unwrap_or((false, 0)));
        state.with_store(|s| {
            // Canonical transition: busy=0, activity cleared, TITLE retained,
            // version bump + status_outbox enqueue. Also clears turn_state.
            s.end_turn(&session, now_secs()).ok();
        });
        state.status_outbox_notify.notify_waiters();

        let rec = state.with_store(|s| s.get_session(&session).ok().flatten());

        // Publish the agent's turn output as kind:9 chat into the room where it
        // worked: per-session rooms and explicit task channels publish final
        // replies; bare project sessions still do not spam the root group.
        // Signed by the durable agent key (or the selected transient session key
        // when a duplicate same-agent signer is active in this route scope).
        //
        // Gated on `was_working` so it is IDEMPOTENT against duplicate stop hooks
        // and client retries: `end_turn` above clears the turn, so a second
        // turn_end (e.g. the blocking client retried after its 2s timeout) reads
        // `was_working == false` and never republishes. The publish timeout is
        // kept below that client retry window so the first call resolves first.
        // Best-effort: a relay/parse hiccup must not fail turn-end.
        if was_working {
            if let (Some(rec), Some(reply)) = (rec.as_ref(), p.reply.as_deref()) {
                let reply = reply.trim();
                let publish_reply = !rec.channel.is_empty()
                    || state
                        .with_store(|s| s.is_session_room(&rec.project))
                        .unwrap_or(false);
                // Skip when the reply equals the turn-start baseline — the
                // transcript's last assistant text is unchanged, so this turn
                // produced no new response (e.g. a tool-only turn) and mirroring
                // it would re-publish the PREVIOUS turn's output.
                let baseline =
                    state.with_store(|s| s.get_last_assistant_text_at_turn_start(&session));
                let is_fresh = reply != baseline.trim();
                if !reply.is_empty() && publish_reply && is_fresh {
                    // Idempotency against duplicate stop hooks / client retries is
                    // provided by the `was_working` gate above (a retry sees the
                    // turn already ended), so the timeout only needs to bound a
                    // hung relay — not race the client retry window.
                    // Timeout kept below the blocking hook client's 2s read
                    // deadline so the daemon fail-opens (and replies) before the
                    // client gives up and retries the stop hook.
                    let publish = publish_agent_reply(state, rec, reply);
                    let debug = std::env::var("TENEX_EDGE_DEBUG").is_ok();
                    match tokio::time::timeout(std::time::Duration::from_millis(1500), publish)
                        .await
                    {
                        Ok(Ok(())) => {}
                        Ok(Err(e)) if debug => {
                            eprintln!("[daemon] agent reply publish skipped: {e:#}")
                        }
                        Err(_) if debug => eprintln!("[daemon] agent reply publish timed out"),
                        _ => {}
                    }
                }
            }
        }

        if was_working {
            let now = now_secs();
            let elapsed_s = if turn_started_at > 0 {
                Some(now.saturating_sub(turn_started_at))
            } else {
                None
            };
            if let Some(rec) = rec.as_ref() {
                state.emit_tail(TailEvent::Turn {
                    ts: now,
                    project: rec.route_scope().to_string(),
                    agent: rec.agent_slug.clone(),
                    session: rec.session_id.clone(),
                    state: "idle".into(),
                    elapsed_s,
                });
            }
            crate::tmux::ring_doorbells(state.clone());
        }
    }
    Ok(serde_json::json!({ "ok": true }))
}

// ── doctor ───────────────────────────────────────────────────────────────────
