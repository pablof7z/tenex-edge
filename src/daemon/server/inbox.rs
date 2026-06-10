use super::session::resolve_session;
use super::*;

#[derive(serde::Deserialize, Default)]
struct InboxParams {
    #[serde(default)]
    session: Option<String>,
    #[serde(default)]
    env_session: Option<String>,
    #[serde(default)]
    cwd: Option<String>,
    #[serde(default)]
    agent: Option<String>,
}

pub(super) async fn rpc_inbox(
    state: &Arc<DaemonState>,
    params: &serde_json::Value,
) -> Result<serde_json::Value> {
    let p: InboxParams = serde_json::from_value(params.clone()).unwrap_or_default();
    let rec = resolve_session(
        state,
        p.session.as_deref(),
        p.env_session.as_deref(),
        p.cwd.as_deref(),
        p.agent.as_deref(),
    )?;
    let _ = fetch_mentions_into_inbox(state, &rec).await;

    let rows = state.with_store(|s| {
        let rows = s.drain_inbox(&rec.session_id).unwrap_or_default();
        for r in &rows {
            s.mark_mention_seen(&rec.agent_pubkey, &r.mention_event_id, now_secs())
                .ok();
        }
        rows
    });
    let pending = state.with_store(|s| s.list_pending_agents().unwrap_or_default());
    let rows_json = state.with_store(|s| rows_to_json(s, &rows));

    Ok(serde_json::json!({
        "rows": rows_json,
        "pending_agents": pending.iter().map(|p| serde_json::json!({"slug": p.slug, "pubkey": p.pubkey})).collect::<Vec<_>>(),
    }))
}

#[derive(serde::Deserialize, Default)]
struct TurnStartParams {
    session: String,
    #[serde(default)]
    transcript: Option<String>,
}

pub(super) async fn rpc_turn_start(
    state: &Arc<DaemonState>,
    params: &serde_json::Value,
) -> Result<serde_json::Value> {
    let p: TurnStartParams =
        serde_json::from_value(params.clone()).context("parsing turn_start params")?;
    if p.session.is_empty() {
        return Ok(serde_json::json!({ "context": serde_json::Value::Null }));
    }

    let prev_started = state.with_store(|s| {
        let (_, prev) = s.get_turn_state(&p.session).unwrap_or((false, 0));
        s.mark_turn_start(&p.session, now_secs()).ok();
        if let Some(path) = p.transcript.as_deref().filter(|x| !x.is_empty()) {
            s.set_session_transcript(&p.session, path).ok();
        }
        prev
    });

    let rec = match state.with_store(|s| s.get_session(&p.session).ok().flatten()) {
        Some(r) => r,
        None => return Ok(serde_json::json!({ "context": serde_json::Value::Null })),
    };
    // Self-fetch stored mentions (relay), then assemble via the SHARED cli.rs
    // function so the injected text is byte-identical to the pre-daemon CLI and
    // cannot drift.
    let _ = fetch_mentions_into_inbox(state, &rec).await;
    let context = crate::cli::assemble_turn_start_context(&state.store, &rec, prev_started)
        .map(serde_json::Value::String)
        .unwrap_or(serde_json::Value::Null);
    Ok(serde_json::json!({ "context": context }))
}

#[derive(serde::Deserialize, Default)]
struct TurnCheckParams {
    #[serde(default)]
    session: Option<String>,
    #[serde(default)]
    env_session: Option<String>,
    #[serde(default)]
    cwd: Option<String>,
    #[serde(default)]
    agent: Option<String>,
}

pub(super) fn rpc_turn_check(
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
    )?;
    let context = crate::cli::assemble_turn_check_context(&state.store, &rec.session_id)
        .map(serde_json::Value::String)
        .unwrap_or(serde_json::Value::Null);
    Ok(serde_json::json!({ "context": context }))
}

#[derive(serde::Deserialize)]
struct TurnEndParams {
    session: String,
}

pub(super) fn rpc_turn_end(
    state: &Arc<DaemonState>,
    params: &serde_json::Value,
) -> Result<serde_json::Value> {
    let p: TurnEndParams =
        serde_json::from_value(params.clone()).context("parsing turn_end params")?;
    if !p.session.is_empty() {
        state.with_store(|s| {
            s.mark_turn_end(&p.session).ok();
        });
    }
    Ok(serde_json::json!({ "ok": true }))
}

// ── user_prompt ──────────────────────────────────────────────────────────────

/// Publish a kind:1 OP signed by the human user's nsec. The event records the
/// user's prompt on the Nostr fabric as a root note (no `e` tag) in the NIP-29
/// group, p-tagging the agent that will process it.
pub(super) async fn rpc_user_prompt(
    state: &Arc<DaemonState>,
    params: &serde_json::Value,
) -> Result<serde_json::Value> {
    use nostr_sdk::prelude::{EventBuilder, Keys, Kind, Tag};

    #[derive(serde::Deserialize, Default)]
    struct P {
        #[serde(default)]
        session: Option<String>,
        #[serde(default)]
        env_session: Option<String>,
        #[serde(default)]
        cwd: Option<String>,
        #[serde(default)]
        prompt: Option<String>,
        #[serde(default)]
        agent: Option<String>,
    }
    let p: P = serde_json::from_value(params.clone()).unwrap_or_default();

    let nsec = match &state.cfg.user_nsec {
        Some(n) => n.clone(),
        None => anyhow::bail!("userNsec not set in ~/.tenex/config.json"),
    };
    let user_keys = Keys::parse(&nsec).context("parsing userNsec")?;

    let rec = resolve_session(
        state,
        p.session.as_deref(),
        p.env_session.as_deref(),
        p.cwd.as_deref(),
        p.agent.as_deref(),
    )?;
    let body = p.prompt.unwrap_or_default();

    // Include `session-id` so the event is session-scoped on the wire.  Without
    // it, any inbox-routing path (live subscription or fetch) would fan out this
    // event to ALL sessions of the agent, leaking one session's user prompt into
    // a sibling session's inbox.
    let builder = EventBuilder::new(Kind::from(1u16), body).tags([
        Tag::parse(["h", &rec.project])?,
        Tag::parse(["p", &rec.agent_pubkey])?,
        Tag::parse(["session-id", &rec.session_id])?,
    ]);
    let event_id = state.transport.publish_signed(builder, &user_keys).await?;

    // Pre-mark the event as already consumed for this session.  The user-prompt
    // is operator-signed so handle_incoming/fetch_mentions skip it via the owners
    // check, but suppress_inbox_event is a belt-and-suspenders guard for any
    // fetch that predates or races the in-memory owners set.
    let eid = event_id.to_hex();
    let sid = rec.session_id.clone();
    state.with_store(|s| s.suppress_inbox_event(&sid, &eid).ok());

    Ok(serde_json::json!({ "event_id": eid }))
}

// ── startup fetch of stored mentions (offline delivery) ──────────────────────

pub(super) async fn fetch_mentions_into_inbox(
    state: &Arc<DaemonState>,
    rec: &crate::state::SessionRecord,
) -> Result<()> {
    use nostr_sdk::prelude::{Filter, Kind, PublicKey};
    let me = rec.agent_pubkey.clone();
    let pk = PublicKey::from_hex(&me)?;
    let filter = Filter::new().kind(Kind::from(1u16)).pubkey(pk).limit(50);
    if let Ok(events) = state.transport.fetch(filter, Duration::from_secs(3)).await {
        for ev in events {
            // User-prompt events are operator-signed (same key as userNsec owners).
            // They are not inter-agent Mentions; skip them here.
            if state.owners.contains(&ev.pubkey.to_hex()) {
                continue;
            }
            if let Some(DomainEvent::Mention(m)) = state.codec.decode(&ev) {
                if m.to_pubkey != me {
                    continue;
                }
                let to = me.clone();
                let routed = state.with_store(|s| route_mention_into(s, &to, &m, &ev));
                if routed {
                    state.mention_notify.notify_waiters();
                }
            }
        }
    }
    Ok(())
}

pub(super) fn rows_to_json(store: &Store, rows: &[InboxRow]) -> Vec<serde_json::Value> {
    rows.iter()
        .map(|r| {
            serde_json::json!({
                "from_slug": r.from_slug,
                "project": r.project,
                "body": r.body,
                "mention_event_id": r.mention_event_id,
                "from_session": r.from_session,
                // Fully-qualified handle the receiver passes to `--recipient`.
                "reply_to": crate::cli::mention_reply_handle(store, r),
            })
        })
        .collect()
}
