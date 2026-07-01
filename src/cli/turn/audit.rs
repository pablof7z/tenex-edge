use crate::state::{RelayEvent, Session, Status, Store};

const AUDIT_ROW_LIMIT: usize = 30;
const AUDIT_CHAT_LIMIT: u32 = 20;

pub(crate) fn turn_start_audit(
    store: &std::sync::Mutex<Store>,
    rec: &Session,
    prev_turn_started_at: u64,
    now: u64,
    context: Option<&str>,
) -> serde_json::Value {
    let first_turn = rec.seen_cursor == 0;
    let ambient_since = if first_turn {
        rec.created_at.max(rec.seen_cursor)
    } else {
        rec.seen_cursor
    };
    let s = store.lock().expect("store mutex poisoned");
    let after = s.get_session(&rec.session_id).ok().flatten();
    let joined = s
        .list_session_joined_channels(&rec.session_id)
        .unwrap_or_default();
    let awareness_since = (!first_turn).then_some(rec.seen_cursor);
    serde_json::json!({
        "kind": "turn_start",
        "now": now,
        "session": session_json(rec),
        "turn": {
            "first_turn": first_turn,
            "prev_turn_started_at": prev_turn_started_at,
            "ambient_since": ambient_since,
        },
        "cursors": {
            "seen_before": rec.seen_cursor,
            "seen_after": after.as_ref().map(|r| r.seen_cursor),
            "turn_started_after": after.as_ref().map(|r| r.turn_started_at),
        },
        "joined_channels": joined_json(&joined),
        "evaluated": evaluated_json(&s, rec, &joined, ambient_since, awareness_since, now),
        "output": output_json(context),
    })
}

pub(crate) fn turn_check_audit(
    store: &std::sync::Mutex<Store>,
    rec: &Session,
    delta_since: Option<u64>,
    cursor_advanced: bool,
    now: u64,
    context: Option<&str>,
) -> serde_json::Value {
    let s = store.lock().expect("store mutex poisoned");
    let after = s.get_session(&rec.session_id).ok().flatten();
    let joined = s
        .list_session_joined_channels(&rec.session_id)
        .unwrap_or_default();
    serde_json::json!({
        "kind": "turn_check",
        "now": now,
        "session": session_json(rec),
        "delta_gate": {
            "session_was_working": rec.working,
            "cursor_before": rec.seen_cursor,
            "cursor_after": after.as_ref().map(|r| r.seen_cursor),
            "cursor_advanced": cursor_advanced,
            "since": delta_since,
        },
        "joined_channels": joined_json(&joined),
        "evaluated": match delta_since {
            Some(since) => evaluated_json(&s, rec, &joined, since, Some(since), now),
            None => serde_json::json!({
                "reason": "delta cursor did not advance; only direct inbox claim was evaluated",
                "pending_inbox_after": inbox_after_json(&s, &rec.session_id),
            }),
        },
        "output": output_json(context),
    })
}

fn evaluated_json(
    s: &Store,
    rec: &Session,
    joined: &[(String, u64)],
    ambient_since: u64,
    awareness_since: Option<u64>,
    now: u64,
) -> serde_json::Value {
    let ambient = joined
        .iter()
        .take(AUDIT_ROW_LIMIT)
        .map(|(channel_h, joined_at)| {
            let since = ambient_since.max(*joined_at);
            serde_json::json!({
                "channel_h": channel_h,
                "since": since,
                "rows": chat_json(s, channel_h, since, Some(&rec.agent_pubkey)),
            })
        })
        .collect::<Vec<_>>();
    serde_json::json!({
        "pending_inbox_after": inbox_after_json(s, &rec.session_id),
        "ambient_chat": ambient,
        "awareness": awareness_json(s, rec, awareness_since, now),
    })
}

fn awareness_json(s: &Store, rec: &Session, since: Option<u64>, now: u64) -> serde_json::Value {
    let mut channels = vec![rec.channel_h.clone()];
    channels.extend(
        s.list_channels()
            .unwrap_or_default()
            .into_iter()
            .filter(|c| c.parent == rec.channel_h)
            .map(|c| c.channel_h),
    );
    channels.sort();
    channels.dedup();
    let status_rows = channels
        .iter()
        .flat_map(|channel_h| {
            s.live_status_for_channel(channel_h, now)
                .unwrap_or_default()
                .into_iter()
        })
        .filter(|st| st.pubkey != rec.agent_pubkey)
        .filter(|st| since.map(|cursor| st.updated_at > cursor).unwrap_or(true))
        .take(AUDIT_ROW_LIMIT)
        .map(status_json)
        .collect::<Vec<_>>();
    serde_json::json!({
        "mode": if since.is_some() { "delta" } else { "snapshot" },
        "since_exclusive": since,
        "candidate_channels": channels.into_iter().take(AUDIT_ROW_LIMIT).collect::<Vec<_>>(),
        "status_rows_updated_after_since": status_rows,
        "active_channels_updated_at_ge_since": since
            .map(|cursor| s.active_channels_since(cursor).unwrap_or_default())
            .unwrap_or_default()
            .into_iter()
            .take(AUDIT_ROW_LIMIT)
            .collect::<Vec<_>>(),
        "activity_rows_created_after_since": since
            .map(|cursor| chat_json(s, &rec.channel_h, cursor, Some(&rec.agent_pubkey)))
            .unwrap_or_default(),
    })
}

fn session_json(rec: &Session) -> serde_json::Value {
    serde_json::json!({
        "session_id": rec.session_id,
        "agent_slug": rec.agent_slug,
        "agent_pubkey": rec.agent_pubkey,
        "channel_h": rec.channel_h,
        "harness": rec.harness,
        "alive": rec.alive,
        "working": rec.working,
        "created_at": rec.created_at,
        "last_seen": rec.last_seen,
        "turn_started_at": rec.turn_started_at,
        "seen_cursor": rec.seen_cursor,
        "title": rec.title,
        "activity": rec.activity,
    })
}

fn joined_json(rows: &[(String, u64)]) -> Vec<serde_json::Value> {
    rows.iter()
        .take(AUDIT_ROW_LIMIT)
        .map(|(channel_h, joined_at)| {
            serde_json::json!({ "channel_h": channel_h, "joined_at": joined_at })
        })
        .collect()
}

fn inbox_after_json(s: &Store, session_id: &str) -> serde_json::Value {
    match s.drain_pending_for_session(session_id) {
        Ok(rows) => serde_json::json!({
            "ok": true,
            "count": rows.len(),
            "event_ids": rows.into_iter().take(AUDIT_ROW_LIMIT).map(|r| r.event_id).collect::<Vec<_>>(),
        }),
        Err(e) => serde_json::json!({ "ok": false, "error": format!("{e:#}") }),
    }
}

fn chat_json(
    s: &Store,
    channel_h: &str,
    since: u64,
    exclude_pubkey: Option<&str>,
) -> Vec<serde_json::Value> {
    s.chat_for_channel(channel_h, since, AUDIT_CHAT_LIMIT)
        .unwrap_or_default()
        .into_iter()
        .filter(|ev| exclude_pubkey != Some(ev.pubkey.as_str()))
        .take(AUDIT_ROW_LIMIT)
        .map(event_json)
        .collect()
}

fn event_json(ev: RelayEvent) -> serde_json::Value {
    serde_json::json!({
        "id": ev.id,
        "kind": ev.kind,
        "pubkey": ev.pubkey,
        "channel_h": ev.channel_h,
        "created_at": ev.created_at,
        "content": truncate(&ev.content, 240),
    })
}

fn status_json(st: Status) -> serde_json::Value {
    serde_json::json!({
        "pubkey": st.pubkey,
        "slug": st.slug,
        "channel_h": st.channel_h,
        "title": st.title,
        "activity": st.activity,
        "busy": st.busy,
        "last_seen": st.last_seen,
        "updated_at": st.updated_at,
        "expiration": st.expiration,
    })
}

fn output_json(context: Option<&str>) -> serde_json::Value {
    serde_json::json!({
        "emitted": context.is_some(),
        "bytes": context.map(str::len).unwrap_or(0),
        "text": context,
    })
}

fn truncate(s: &str, limit: usize) -> String {
    if s.len() <= limit {
        return s.to_string();
    }
    let mut out = s.chars().take(limit.saturating_sub(1)).collect::<String>();
    out.push_str("...");
    out
}
