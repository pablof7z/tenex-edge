use super::chat_target::resolve_chat_target;
use super::*;
use crate::state::Message;
use crate::util::{truncate_words, CHAT_RENDER_WORD_LIMIT};

mod read_scope;
#[cfg(test)]
mod tests;

use read_scope::{chat_read_scopes_for_store, ChatCursor};

/// Upper bound on chat-log rows pulled per channel for a read (the slicing below
/// narrows to the requested window).
const CHAT_READ_CAP: u32 = 10_000;

#[derive(serde::Deserialize, Default)]
pub(in crate::daemon::server) struct ChatReadParams {
    #[serde(default)]
    id: Option<String>,
    #[serde(default)]
    channel: Option<String>,
    #[serde(default)]
    since: Option<u64>,
    #[serde(default)]
    limit: Option<u64>,
    #[serde(default)]
    offset: Option<u64>,
    #[serde(default)]
    tail: bool,
    #[serde(default)]
    live: bool,
}

pub(in crate::daemon::server) async fn handle_chat_read<W: AsyncWriteExt + Unpin>(
    state: &Arc<DaemonState>,
    id: u64,
    params: &serde_json::Value,
    writer: &mut W,
) -> Result<()> {
    let p: ChatReadParams = serde_json::from_value(params.clone()).unwrap_or_default();
    if let Some(event_id) = p.id.as_deref().filter(|s| !s.trim().is_empty()) {
        let row = state
            .with_store(|s| s.get_message_by_prefix(event_id))
            .with_context(|| format!("reading chat message {event_id}"))?
            .with_context(|| format!("chat message not found: {event_id}"))?;
        let json = chat_row_to_json(state, &row, false);
        if write_json(writer, &Response::item(id, json)).await.is_ok() {
            let _ = write_json(writer, &Response::end(id)).await;
        }
        return Ok(());
    }

    let rec = resolve_session_inner(
        state,
        &CallerAnchor::from_params(params),
        ResolveScope::Project,
    )?;
    let target = resolve_chat_target(state, &rec, p.channel.as_deref(), "chat read")?;
    let scope = target.channel_h;
    let since = p.since.unwrap_or(0);
    let offset = p.offset.unwrap_or(0);

    let _ = ensure_subscription(state, &scope).await;
    let read_scopes = chat_read_scopes(state, &scope);
    let mut rx = if p.live {
        Some(state.tail_subscribe())
    } else {
        None
    };
    let live_started_at = now_secs();
    let live_floor = live_started_at.max(since);

    let rows = state.with_store(|s| {
        let mut rows: Vec<Message> = read_scopes
            .iter()
            .flat_map(|sc| {
                s.chat_messages_for_channel(sc, since, CHAT_READ_CAP)
                    .unwrap_or_default()
            })
            .collect();
        rows.sort_by(|a, b| {
            a.created_at
                .cmp(&b.created_at)
                .then_with(|| a.message_id.cmp(&b.message_id))
        });
        if p.tail {
            let limit = p.limit.unwrap_or(10) as usize;
            let start = rows
                .len()
                .saturating_sub(limit.saturating_add(offset as usize));
            let end = rows.len().saturating_sub(offset as usize);
            rows = rows.get(start..end).unwrap_or(&[]).to_vec();
        } else {
            let start = offset as usize;
            let end = p
                .limit
                .map(|limit| start.saturating_add(limit as usize))
                .unwrap_or(rows.len())
                .min(rows.len());
            rows = rows.get(start..end).unwrap_or(&[]).to_vec();
        }
        rows
    });
    let mut seen: std::collections::HashSet<String> =
        rows.iter().map(|r| r.message_id.clone()).collect();
    let mut cursors: std::collections::HashMap<String, ChatCursor> = read_scopes
        .iter()
        .map(|scope| (scope.clone(), ChatCursor::new(live_floor)))
        .collect();
    for row in &rows {
        cursors
            .entry(row.channel_h.clone())
            .or_insert_with(|| ChatCursor::new(live_floor))
            .observe(row);
    }

    for row in rows {
        let json = chat_row_to_json(state, &row, true);
        if write_json(writer, &Response::item(id, json)).await.is_err() {
            let _ = write_json(writer, &Response::end(id)).await;
            return Ok(());
        }
    }

    let Some(ref mut rx) = rx else {
        let _ = write_json(writer, &Response::end(id)).await;
        return Ok(());
    };

    loop {
        match rx.recv().await {
            Ok(TailEvent::Msg {
                project: ev_project,
                ..
            }) if read_scopes.contains(&ev_project) => {
                let cursor = cursors
                    .entry(ev_project.clone())
                    .or_insert_with(|| ChatCursor::new(live_floor))
                    .clone();
                let rows = state.with_store(|s| {
                    s.chat_messages_for_channel_after(
                        &ev_project,
                        cursor.created_at,
                        &cursor.id,
                        CHAT_READ_CAP,
                    )
                    .unwrap_or_default()
                });
                for row in rows {
                    cursors
                        .entry(row.channel_h.clone())
                        .or_insert_with(|| ChatCursor::new(live_floor))
                        .observe(&row);
                    if !seen.insert(row.message_id.clone()) {
                        continue;
                    }
                    let json = chat_row_to_json(state, &row, true);
                    if write_json(writer, &Response::item(id, json)).await.is_err() {
                        let _ = write_json(writer, &Response::end(id)).await;
                        return Ok(());
                    }
                }
            }
            Ok(_) => {}
            Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
            Err(tokio::sync::broadcast::error::RecvError::Lagged(skipped)) => {
                let _ =
                    write_json(writer, &stream_lag_error(id, "chat read --live", skipped)).await;
                return Ok(());
            }
        }
    }
    let _ = write_json(writer, &Response::end(id)).await;
    Ok(())
}

fn chat_read_scopes(state: &Arc<DaemonState>, scope: &str) -> Vec<String> {
    state.with_store(|s| chat_read_scopes_for_store(s, scope))
}

/// Render a canonical chat message into the CLI's chat-line JSON, resolving the
/// author's slug from the materialized profile/session caches.
fn chat_row_to_json(state: &Arc<DaemonState>, row: &Message, truncate: bool) -> serde_json::Value {
    let (from_slug, host, mentioned_session) = chat_row_refs(state, row);
    chat_log_row_to_json(
        row,
        &from_slug,
        &host,
        mentioned_session.as_deref(),
        truncate,
    )
}

pub(in crate::daemon::server) fn chat_log_row_to_json(
    row: &Message,
    from_slug: &str,
    host: &str,
    mentioned_session: Option<&str>,
    truncate: bool,
) -> serde_json::Value {
    let (body, truncated) = if truncate {
        truncate_words(&row.body, CHAT_RENDER_WORD_LIMIT)
    } else {
        (row.body.trim().to_string(), false)
    };
    serde_json::json!({
        "event_id": &row.message_id,
        "full_event_id": &row.message_id,
        "from_pubkey": &row.author_pubkey,
        "from_slug": from_slug,
        "host": host,
        "project": &row.channel_h,
        "body": body,
        "truncated": truncated,
        "created_at": row.created_at,
        "from_session": row.author_session.as_deref().unwrap_or(""),
        "mentioned_session": mentioned_session.unwrap_or(""),
    })
}

fn chat_row_refs(state: &Arc<DaemonState>, row: &Message) -> (String, String, Option<String>) {
    let local_host = state.host.clone();
    state.with_store(|s| {
        let profile = s.get_profile(&row.author_pubkey).ok().flatten();
        let session = row
            .author_session
            .as_deref()
            .and_then(|sid| s.get_session(sid).ok().flatten());
        let from_slug = profile
            .as_ref()
            .map(|p| p.slug.as_str())
            .filter(|slug| !slug.is_empty())
            .or_else(|| session.as_ref().map(|rec| rec.agent_slug.as_str()))
            .filter(|slug| !slug.is_empty())
            .map(str::to_string)
            .unwrap_or_else(|| pubkey_short(&row.author_pubkey));
        let host = profile
            .as_ref()
            .map(|p| p.host.clone())
            .filter(|h| !h.is_empty())
            .or_else(|| session.as_ref().map(|_| local_host))
            .unwrap_or_default();
        let mentioned_session = s
            .message_recipients(&row.message_id)
            .unwrap_or_default()
            .into_iter()
            .find_map(|r| r.target_session);
        (from_slug, host, mentioned_session)
    })
}

// ── tail (streaming) ──────────────────────────────────────────────────────────

/// Parameters for the `tail` RPC.
#[derive(serde::Deserialize, Default)]
pub(in crate::daemon::server) struct TailParams {
    #[serde(default)]
    project: Option<String>,
    /// Number of backfill events (recent messages + roster snapshot), default 20.
    #[serde(default)]
    backfill: Option<u64>,
    /// Return only events after this unix timestamp.
    #[serde(default)]
    since: Option<u64>,
}

pub(in crate::daemon::server) async fn handle_tail<W: AsyncWriteExt + Unpin>(
    state: &Arc<DaemonState>,
    id: u64,
    params: &serde_json::Value,
    writer: &mut W,
) -> Result<()> {
    let p: TailParams = serde_json::from_value(params.clone()).unwrap_or_default();
    let project = p.project.clone();
    let backfill_n = p.backfill.unwrap_or(20);
    let since = p.since.unwrap_or(0);

    // Ensure the requested project is in the union subscription.
    if let Some(pr) = &project {
        let _ = ensure_subscription(state, pr).await;
    }

    // Subscribe BEFORE backfill so we don't miss events that arrive during query.
    let mut rx = state.tail_subscribe();

    {
        *state.open_clients.lock().unwrap() += 1;
    }
    let _guard = ClientGuard(state.clone());

    // ── Backfill ────────────────────────────────────────────────────────────
    if backfill_n > 0 {
        let backfill_events = build_backfill(state, project.as_deref(), backfill_n, since);
        for ev in backfill_events {
            if write_json(writer, &Response::item(id, serde_json::to_value(&ev)?))
                .await
                .is_err()
            {
                let _ = write_json(writer, &Response::end(id)).await;
                return Ok(());
            }
        }
    }

    // ── Live stream ─────────────────────────────────────────────────────────
    loop {
        match rx.recv().await {
            Ok(ev) => {
                if tail_event_matches_project(&ev, project.as_deref())
                    && ev.ts() >= since
                    && write_json(writer, &Response::item(id, serde_json::to_value(&ev)?))
                        .await
                        .is_err()
                {
                    break;
                }
            }
            Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
            Err(tokio::sync::broadcast::error::RecvError::Lagged(skipped)) => {
                let _ = write_json(writer, &stream_lag_error(id, "tail", skipped)).await;
                return Ok(());
            }
        }
    }
    let _ = write_json(writer, &Response::end(id)).await;
    Ok(())
}

fn stream_lag_error(id: u64, stream: &str, skipped: u64) -> Response {
    Response::err(
        id,
        "stream_lagged",
        format!(
            "{stream} dropped {skipped} live event(s); reconnect to resume from persisted history"
        ),
    )
}

/// True when the event belongs to the requested project scope (or no filter).
pub(in crate::daemon::server) fn tail_event_matches_project(
    ev: &TailEvent,
    project: Option<&str>,
) -> bool {
    let Some(pr) = project else {
        return true;
    };
    let ev_project = match ev {
        TailEvent::Msg { project, .. } => project.as_str(),
        TailEvent::Sync { project, .. } => project.as_str(),
        TailEvent::Turn { project, .. } => project.as_str(),
        TailEvent::Status { project, .. } => project.as_str(),
        TailEvent::Join { project, .. } => project.as_str(),
        TailEvent::Leave { project, .. } => project.as_str(),
        TailEvent::Sess { project, .. } => project.as_str(),
        TailEvent::Proj { project, .. } => project.as_str(),
        // Profiles are cross-project; always include.
        TailEvent::Profile { .. } => return true,
    };
    ev_project == pr
}

/// Build the backfill event list from the materialized caches.
///
/// Returns recent chat lines from `messages` as `Msg` events + a roster
/// snapshot built from live `relay_status` rows (peers AND local agents read
/// identically) and this daemon's own live sessions, sorted ascending by time.
pub(in crate::daemon::server) fn build_backfill(
    state: &Arc<DaemonState>,
    project: Option<&str>,
    limit: u64,
    since: u64,
) -> Vec<TailEvent> {
    let mut events: Vec<TailEvent> = Vec::new();
    let now = now_secs();
    let cap = limit.min(u32::MAX as u64) as u32;

    // ── Recent chat lines from messages ──────────────────────────────────────
    let chat_rows: Vec<Message> = state.with_store(|s| match project {
        Some(pr) => s
            .chat_messages_for_channel(pr, since, cap)
            .unwrap_or_default(),
        None => s.recent_chat_messages(since, cap).unwrap_or_default(),
    });
    for row in chat_rows {
        let (from_slug, _, to_session) = chat_row_refs(state, &row);
        let to = state.with_store(|s| {
            s.message_recipients(&row.message_id)
                .unwrap_or_default()
                .into_iter()
                .next()
                .map(|r| pubkey_short(&r.recipient_pubkey))
                .unwrap_or_else(|| "project-chat".to_string())
        });
        events.push(TailEvent::Msg {
            ts: row.created_at,
            project: row.channel_h.clone(),
            from: from_slug,
            from_session: row.author_session.clone(),
            to,
            to_session,
            body: row.body.chars().take(200).collect(),
        });
    }

    // ── Roster snapshot: live status rows (peers + local agents) ─────────────
    if let Some(pr) = project {
        let statuses = state.with_store(|s| s.live_status_for_channel(pr, now).unwrap_or_default());
        for st in statuses {
            let host = state
                .with_store(|s| s.get_profile(&st.pubkey))
                .ok()
                .flatten()
                .map(|p| p.host)
                .unwrap_or_default();
            events.push(TailEvent::Join {
                ts: st.last_seen,
                project: st.channel_h.clone(),
                agent: st.slug.clone(),
                host,
                session: st.pubkey.clone(),
                rel_cwd: String::new(),
            });
            if !st.title.is_empty() || st.busy {
                events.push(TailEvent::Status {
                    ts: st.last_seen,
                    project: st.channel_h.clone(),
                    agent: st.slug.clone(),
                    text: st.title.clone(),
                    active: st.busy,
                });
            }
        }
    }

    // ── This daemon's own live sessions as synthetic Sess/Turn events ────────
    let mine = state.with_store(|s| s.list_alive_sessions().unwrap_or_default());
    for rec in mine {
        if project.map(|pr| rec.channel_h != pr).unwrap_or(false) {
            continue;
        }
        events.push(TailEvent::Sess {
            ts: rec.created_at,
            project: rec.channel_h.clone(),
            agent: rec.agent_slug.clone(),
            session: rec.session_id.clone(),
            state: "start".into(),
            rel_cwd: String::new(),
        });
        if rec.working {
            events.push(TailEvent::Turn {
                ts: rec.turn_started_at,
                project: rec.channel_h.clone(),
                agent: rec.agent_slug.clone(),
                session: rec.session_id.clone(),
                state: "working".into(),
                elapsed_s: None,
            });
        }
    }

    // Sort ascending by timestamp.
    events.sort_by_key(|e| e.ts());
    events
}
